pub mod output;
mod var_store;

use ament_index::index::AmentIndex;
use eyre::{bail, ensure, Context};
use launch_format::{
    DeclareArg, Env, Executable, Group, GroupChild, Include, IncludeArg, Launch, LaunchChild, Let,
    Node, NodeChild, PushRosNamespace, SetEnv, UnsetEnv,
};
use launch_subst::{SubstBlock, Substitution};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use var_store::VarStore;

pub fn load_launch_file<P, I>(path: P, args: I) -> eyre::Result<output::Launch>
where
    I: IntoIterator<Item = (String, String)>,
    P: AsRef<Path>,
{
    let mut var_store = VarStore::default();
    let mut state = State {
        work_dirs: vec![],
        namespace_segments: vec![],
        execs: vec![],
        nodes: vec![],
        ament_index: ament_index::index::ament_index()?,
        var_store: &mut var_store,
    };

    state.with_scope(|state| load_launch_file_private(path, args, state))?;

    let State { execs, nodes, .. } = state;
    let profile = output::Launch { execs, nodes };

    Ok(profile)
}

fn load_launch_file_private<P, I>(path: P, args: I, state: &mut State) -> eyre::Result<()>
where
    I: IntoIterator<Item = (String, String)>,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    let Some(ext) = path.extension() else {
        bail!(
            "The launch file must ends with '.xml' or '.yaml': {}",
            path.display()
        );
    };
    let Some(parent) = path.parent() else {
        bail!(
            "Unable to find the parent directory of the launch file: {}",
            path.display()
        );
    };

    let launch: Launch = if ext == "xml" {
        let file =
            File::open(path).with_context(|| format!("Unable to open {}", path.display()))?;
        let reader = BufReader::new(file);
        quick_xml::de::from_reader(reader)
            .with_context(|| format!("Unable to parse {}", path.display()))?
    } else if ext == "yaml" {
        let file =
            File::open(path).with_context(|| format!("Unable to open {}", path.display()))?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader)
            .with_context(|| format!("Unable to parse {}", path.display()))?
    } else {
        bail!(
            "The launch file must ends with '.xml' or '.yaml': {}",
            path.display()
        );
    };

    for (name, value) in args {
        state.var_store.insert_var(name, state.eval(&value)?);
    }

    state
        .with_wd(parent.to_path_buf(), |state| {
            for child in &launch.children {
                match child {
                    LaunchChild::Arg(DeclareArg { name, default, .. }) => {
                        match (state.var_store.contains_var(name), default) {
                            (false, None) => {
                                ensure!(
                                    state.var_store.contains_var(name),
                                    r#"the argument "{name}" is required but is not provided."#
                                );
                            }
                            (false, Some(default)) => {
                                // if name == "motion_velocity_smoother_type" {
                                //     dbg!();
                                // }
                                state
                                    .var_store
                                    .insert_var(name.to_string(), default.to_string());
                            }
                            (true, _) => {}
                        }
                    }
                    LaunchChild::Let(Let { name, value }) => {
                        state
                            .var_store
                            .insert_var(name.to_string(), state.eval(value)?);
                    }
                    LaunchChild::Executable(exec) => parse_executable(exec, state)?,
                    LaunchChild::Node(node) => parse_node(node, state)?,
                    LaunchChild::Group(group) => parse_group(group, state)?,
                    LaunchChild::Include(include) => parse_include(include, state)?,
                    LaunchChild::SetEnv(set_env) => parse_set_env(set_env, state)?,
                    LaunchChild::UnsetEnv(unset_env) => parse_unset_env(unset_env, state)?,
                }
            }

            Ok(())
        })
        .with_context(|| format!("unable to parse launch file {}", path.display()))?;

    Ok(())
}

fn parse_group(group: &Group, state: &mut State) -> eyre::Result<()> {
    let Group {
        scoped,
        r#if,
        unless,
        children,
    } = group;

    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if !yes {
        return Ok(());
    }

    let scoped = *scoped == Some(true);

    let parse_child = |children: &[_], state: &mut State| -> eyre::Result<_> {
        for child in children {
            match child {
                GroupChild::Executable(exec) => parse_executable(exec, state)?,
                GroupChild::Node(node) => parse_node(node, state)?,
                GroupChild::Group(group) => parse_group(group, state)?,
                GroupChild::Include(include) => parse_include(include, state)?,
                GroupChild::SetEnv(set_env) => parse_set_env(set_env, state)?,
                GroupChild::UnsetEnv(unset_env) => parse_unset_env(unset_env, state)?,
                GroupChild::Let(Let { name, value }) => {
                    state
                        .var_store
                        .insert_var(name.to_string(), state.eval(value)?);
                }
                GroupChild::Arg(DeclareArg { name, default, .. }) => {
                    match (state.var_store.contains_var(name), default) {
                        (false, None) => {
                            ensure!(
                                state.var_store.contains_var(name),
                                r#"The argument "{name}" is required but not provided."#
                            );
                        }
                        (false, Some(default)) => {
                            state
                                .var_store
                                .insert_var(name.to_string(), default.to_string());
                        }
                        (true, _) => {}
                    }
                }
                GroupChild::PushRosNamespace(PushRosNamespace { namespace }) => {
                    // TODO
                }
            }
        }

        Ok(())
    };

    if scoped {
        state.with_scope(|state| parse_child(children, state))?;
    } else {
        parse_child(children, state)?;
    }

    Ok(())
}

fn parse_node(node: &Node, state: &mut State) -> eyre::Result<()> {
    let Node {
        pkg,
        exec,
        name,
        ros_args,
        args,
        namespace,
        launch_prefix,
        output,
        r#if,
        unless,
        children,
    } = node;

    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if !yes {
        return Ok(());
    }

    // TODO: populate envs from state
    let mut env = HashMap::new();

    let mut param = vec![];
    let mut remap = vec![];

    for child in children {
        match child {
            NodeChild::Env(Env { name, value }) => {
                env.insert(name.to_string(), value.to_string());
            }
            NodeChild::Param(p) => param.push(p.clone()),
            NodeChild::Remap(r) => remap.push(r.clone()),
        }
    }

    state.nodes.push(output::Node {
        pkg: pkg.clone(),
        exec: exec.clone(),
        name: name.clone(),
        ros_args: ros_args.clone(),
        args: args.clone(),
        namespace: namespace.clone(),
        launch_prefix: launch_prefix.clone(),
        output: *output,
        env,
        param,
        remap,
    });

    Ok(())
}

fn parse_executable(exec: &Executable, state: &mut State) -> eyre::Result<()> {
    let Executable {
        env,
        cmd,
        cwd,
        name,
        args,
        shell,
        launch_prefix,
        output,
        r#if,
        unless,
    } = exec;

    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if !yes {
        return Ok(());
    }

    todo!();
}

fn parse_include(include: &Include, state: &mut State) -> eyre::Result<()> {
    let Include {
        file,
        r#if,
        unless,
        arg,
    } = include;

    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if !yes {
        return Ok(());
    }

    let path = state.eval(file)?;
    let args = arg
        .iter()
        .map(|IncludeArg { name, value }| (name.to_string(), value.to_string()));
    load_launch_file_private(&path, args, state)?;

    Ok(())
}

fn parse_set_env(set_env: &SetEnv, state: &mut State) -> eyre::Result<()> {
    let SetEnv {
        name,
        value,
        r#if,
        unless,
    } = set_env;

    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if yes {
        state
            .var_store
            .insert_env(name.to_string(), value.to_string());
    }

    Ok(())
}

fn parse_unset_env(unset_env: &UnsetEnv, state: &mut State) -> eyre::Result<()> {
    let UnsetEnv { name, r#if, unless } = unset_env;
    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if yes {
        let value = state.var_store.remove_env(name);
        if value.is_none() {
            todo!();
        }
    }
    Ok(())
}

struct State<'a> {
    work_dirs: Vec<PathBuf>,
    namespace_segments: Vec<String>,
    execs: Vec<output::Executable>,
    nodes: Vec<output::Node>,
    var_store: &'a mut VarStore,
    ament_index: &'static AmentIndex,
}

impl<'a> State<'a> {
    pub fn eval_if_unless(&self, r#if: Option<&str>, unless: Option<&str>) -> eyre::Result<bool> {
        let if_value = match r#if {
            Some(cond) => self.eval_bool(cond)?,
            None => true,
        };

        let unless_value = match unless {
            Some(cond) => self.eval_bool(cond)?,
            None => false,
        };

        Ok(if_value && !unless_value)
    }

    pub fn eval_bool(&self, text: &str) -> eyre::Result<bool> {
        let text = self.eval(text)?;
        let ret = match text.as_str() {
            "true" => true,
            "false" => false,
            _ => bail!("expect 'true' or 'false', but get '{text}'"),
        };
        Ok(ret)
    }

    pub fn eval(&self, text: &str) -> eyre::Result<String> {
        let blocks = launch_subst::parse(text)
            .wrap_err_with(|| format!(r#"unable to parse expression "{text}""#))?;
        let output = self
            .subst_blocks(&blocks)
            .wrap_err_with(|| format!(r#"unable to parse expression "{text}""#))?;
        Ok(output)
    }

    pub fn subst_blocks(&self, blocks: &[SubstBlock]) -> eyre::Result<String> {
        let mut buf = String::new();

        for block in blocks {
            let text: Cow<_> = match block {
                SubstBlock::Text(text) => text.into(),
                SubstBlock::Substitution(subst) => self.subst(subst)?,
            };
            buf.push_str(&text);
        }

        Ok(buf)
    }

    pub fn subst(&self, subst: &Substitution) -> eyre::Result<Cow<'_, str>> {
        let Substitution { command, args } = subst;

        let text: Cow<'_, str> = match command.as_str() {
            "env" => {
                let [arg] = args.as_slice() else {
                    todo!();
                };
                let name = self.subst_blocks(&arg.0)?;
                let Some(value) = self.var_store.get_env(&name) else {
                    todo!();
                };
                value.into()
            }
            "find-pkg-share" => {
                let [arg] = args.as_slice() else {
                    todo!();
                };
                let package_name = self.subst_blocks(&arg.0)?;
                let Some(pkg_index) = self.ament_index.packages.get(&package_name) else {
                    bail!(r#"package "{package_name}" not found"#);
                };
                pkg_index
                    .ament_dir
                    .join("share")
                    .join(package_name)
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .into()
            }
            "find" => todo!(),
            "eval" => todo!(),
            "var" => {
                let [arg] = args.as_slice() else {
                    todo!();
                };
                let name = self.subst_blocks(&arg.0)?;
                let Some(value) = self.var_store.get_var(&name) else {
                    bail!("variable \"{name}\" is used before assignment");
                };
                value.into()
            }
            _ => bail!("unknown command \"{command}\""),
        };
        Ok(text)
    }

    pub fn cwd(&self) -> &Path {
        self.work_dirs.last().unwrap()
    }

    pub fn with_wd<T, F>(&mut self, wd: PathBuf, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.work_dirs.push(wd);
        let output = f(self);
        self.work_dirs.pop();
        output
    }

    pub fn with_scope<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.var_store.push_scope();
        let output = f(self);
        self.var_store.pop_scope();
        output
    }
}
