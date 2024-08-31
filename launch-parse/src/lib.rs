pub mod context;

use ament_index::index::AmentIndex;
use eyre::{bail, ensure, Context};
use launch_format::{
    Arg, Env, Executable, Group, GroupChild, Include, IncludeArg, Launch, LaunchChild, Let, Node,
    NodeChild, SetEnv, UnsetEnv,
};
use launch_subst::{SubstBlock, Substitution};
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

pub fn load_launch_file<P, I>(path: P, args: I) -> eyre::Result<context::Launch>
where
    I: IntoIterator<Item = (String, String)>,
    P: AsRef<Path>,
{
    let mut state = State {
        work_dirs: vec![],
        scopes: vec![],
        execs: vec![],
        nodes: vec![],
        ament_index: ament_index::index::ament_index()?,
    };

    load_launch_file_private(path, args, &mut state)?;

    let State { execs, nodes, .. } = state;
    let profile = context::Launch { execs, nodes };

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

    state
        .with_wd(parent.to_path_buf(), |state| {
            state.with_scope(|state| {
                for (name, value) in args {
                    state.insert_var(name, state.eval(&value)?);
                }
                parse_launch(&launch, state)
            })
        })
        .with_context(|| format!("unable to parse launch file {}", path.display()))?;

    Ok(())
}

fn parse_launch(launch: &Launch, state: &mut State) -> eyre::Result<()> {
    for child in &launch.children {
        match child {
            LaunchChild::Arg(Arg {
                name,
                value,
                default,
                ..
            }) => match (value, default) {
                (None, None) => {
                    ensure!(
                        state.contains_var(name),
                        r#"The argument "{name}" is required but not provided."#
                    );
                }
                (None, Some(default)) => {
                    state.get_var_or_insert(name, default);
                }
                (Some(value), _) => {
                    state.insert_var(name.to_string(), state.eval(value)?);
                }
            },
            LaunchChild::Let(Let { name, value }) => {
                state.insert_var(name.to_string(), state.eval(value)?);
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
                    state.insert_var(name.to_string(), state.eval(value)?);
                }
                GroupChild::Arg(Arg {
                    name,
                    value,
                    default,
                    ..
                }) => match (value, default) {
                    (None, None) => {
                        ensure!(
                            state.contains_var(name),
                            r#"The argument "{name}" is required but not provided."#
                        );
                    }
                    (None, Some(default)) => {
                        state.get_var_or_insert(name, default);
                    }
                    (Some(value), _) => {
                        state.insert_var(name.to_string(), value.to_string());
                    }
                },
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

    state.nodes.push(context::Node {
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
    load_launch_file_private(path, args, state)?;

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
        state.insert_env(name.to_string(), value.to_string());
    }

    Ok(())
}

fn parse_unset_env(unset_env: &UnsetEnv, state: &mut State) -> eyre::Result<()> {
    let UnsetEnv { name, r#if, unless } = unset_env;
    let yes = state.eval_if_unless(r#if.as_deref(), unless.as_deref())?;
    if yes {
        let value = state.remove_env(name);
        if value.is_none() {
            todo!();
        }
    }
    Ok(())
}

struct State {
    work_dirs: Vec<PathBuf>,
    scopes: Vec<Scope>,
    execs: Vec<context::Executable>,
    nodes: Vec<context::Node>,
    ament_index: &'static AmentIndex,
}

impl State {
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

    pub fn subst<'a>(&'a self, subst: &'a Substitution) -> eyre::Result<Cow<'a, str>> {
        let Substitution { command, args } = subst;

        let text: Cow<'a, str> = match command.as_str() {
            "env" => {
                let [arg] = args.as_slice() else {
                    todo!();
                };
                let name = self.subst_blocks(&arg.0)?;
                let Some(value) = self.get_env(&name) else {
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
                let Some(value) = self.get_var(&name) else {
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
        self.scopes.push(Scope::default());
        let output = f(self);
        self.scopes.pop().unwrap();
        output
    }

    pub fn contains_var(&self, name: &str) -> bool {
        self.current_scope().var.contains_key(name)
    }

    pub fn contains_env(&self, name: &str) -> bool {
        self.current_scope().env.contains_key(name)
    }

    pub fn insert_var(&mut self, name: String, value: String) {
        self.current_scope_mut().var.insert(name, value);
    }

    pub fn insert_env(&mut self, name: String, value: String) {
        self.current_scope_mut().var.insert(name, value);
    }

    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.current_scope().var.get(name).map(|v| v.as_str())
    }

    pub fn get_var_or_insert(&mut self, name: &str, default: &str) -> &str {
        self.current_scope_mut()
            .var
            .entry(name.to_string())
            .or_insert_with(|| default.to_string())
    }

    pub fn get_env(&self, name: &str) -> Option<&str> {
        self.current_scope().env.get(name).map(|v| v.as_str())
    }

    pub fn get_env_or_insert(&mut self, name: &str, default: &str) -> &str {
        self.current_scope_mut()
            .env
            .entry(name.to_string())
            .or_insert_with(|| default.to_string())
    }

    pub fn remove_env(&mut self, name: &str) -> Option<String> {
        self.current_scope_mut().env.remove(name)
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }
}

struct Scope {
    var: HashMap<String, String>,
    env: HashMap<String, String>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            var: HashMap::new(),
            env: HashMap::new(),
        }
    }
}
