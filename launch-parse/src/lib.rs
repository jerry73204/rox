pub mod context;

use anyhow::{bail, ensure, Context, Result};
use launch_format::{
    Executable, Group, GroupChild, Include, IncludeArg, Launch, LaunchArg, LaunchChild, Let, Node,
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
use strong_xml::XmlRead;

pub fn load_launch_file<P, I>(path: P, args: I) -> Result<context::Launch>
where
    I: IntoIterator<Item = (String, String)>,
    P: AsRef<Path>,
{
    let mut state = State {
        work_dirs: vec![],
        scopes: vec![],
        execs: vec![],
        nodes: vec![],
    };

    load_launch_file_private(path, args, &mut state)?;

    let State { execs, nodes, .. } = state;
    let profile = context::Launch { execs, nodes };

    Ok(profile)
}

fn load_launch_file_private<P, I>(path: P, args: I, state: &mut State) -> Result<()>
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
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Unable to open {}", path.display()))?;
        Launch::from_str(&text).with_context(|| format!("Unable to parse {}", path.display()))?
    } else if ext == "yaml" {
        let reader = BufReader::new(
            File::open(path).with_context(|| format!("Unable to open {}", path.display()))?,
        );
        serde_yaml::from_reader(reader)
            .with_context(|| format!("Unable to parse {}", path.display()))?
    } else {
        bail!(
            "The launch file must ends with '.xml' or '.yaml': {}",
            path.display()
        );
    };

    state.with_wd(parent.to_path_buf(), |state| {
        state.with_scope(|state| {
            for (name, value) in args {
                state.insert_var(name, value);
            }
            parse_launch(&launch, state)
        })
    })?;

    Ok(())
}

fn parse_launch(launch: &Launch, state: &mut State) -> Result<()> {
    for child in &launch.children {
        match child {
            LaunchChild::Arg(LaunchArg {
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
            LaunchChild::Let(Let { name, value }) => {
                state.insert_var(name.to_string(), value.to_string());
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

fn parse_group(group: &Group, state: &mut State) -> Result<()> {
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

    let parse_child = |children: &[_], state: &mut State| -> Result<_> {
        for child in children {
            match child {
                GroupChild::Executable(exec) => parse_executable(exec, state)?,
                GroupChild::Node(node) => parse_node(node, state)?,
                GroupChild::Group(group) => parse_group(group, state)?,
                GroupChild::Include(include) => parse_include(include, state)?,
                GroupChild::SetEnv(set_env) => parse_set_env(set_env, state)?,
                GroupChild::UnsetEnv(unset_env) => parse_unset_env(unset_env, state)?,
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

fn parse_node(node: &Node, state: &mut State) -> Result<()> {
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

    for child in children {
        match child {
            NodeChild::Env(_) => todo!(),
            NodeChild::Param(_) => todo!(),
            NodeChild::Remap(_) => todo!(),
        }
    }
    todo!();
}

fn parse_executable(exec: &Executable, state: &mut State) -> Result<()> {
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

fn parse_include(include: &Include, state: &mut State) -> Result<()> {
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

fn parse_set_env(set_env: &SetEnv, state: &mut State) -> Result<()> {
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

fn parse_unset_env(unset_env: &UnsetEnv, state: &mut State) -> Result<()> {
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
}

impl State {
    pub fn eval_if_unless(&self, r#if: Option<&str>, unless: Option<&str>) -> Result<bool> {
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

    pub fn eval_bool(&self, text: &str) -> Result<bool> {
        let text = self.eval(text)?;

        let ret = match text.as_str() {
            "true" => true,
            "false" => false,
            _ => bail!("expect 'true' or 'false', but get '{text}'"),
        };
        Ok(ret)
    }

    pub fn eval(&self, text: &str) -> Result<String> {
        let blocks = launch_subst::parse(text)?;
        let mut buf = String::new();

        for block in &blocks {
            let text: Cow<_> = match block {
                SubstBlock::Text(text) => text.into(),
                SubstBlock::Substitution(subst) => self.subst(subst)?,
            };
            buf.push_str(&text);
        }

        Ok(buf)
    }

    pub fn subst<'a>(&'a self, subst: &'a Substitution) -> Result<Cow<'a, str>> {
        let text: Cow<'a, str> = match subst {
            Substitution::Env { variable } => {
                let Some(value) = self.get_env(variable) else {
                    todo!();
                };
                value.into()
            }
            Substitution::OptEnv {
                variable,
                default_value,
            } => {
                let Some(value) = self.get_env(variable).or(default_value.as_deref()) else {
                    todo!();
                };
                value.into()
            }
            Substitution::Find { pkg } => todo!(),
            Substitution::Anon { name } => todo!(),
            Substitution::Arg { name } => todo!(),
            Substitution::Eval { expr } => todo!(),
            Substitution::DirName => todo!(),
            Substitution::Other { args } => todo!(),
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
        let value = self.current_scope().var.get(name)?;
        Some(value)
    }

    pub fn get_env(&self, name: &str) -> Option<&str> {
        let value = self.current_scope().env.get(name)?;
        Some(value)
    }

    pub fn get_var_or_insert(&mut self, name: &str, default: &str) -> &str {
        self.current_scope_mut()
            .var
            .entry(name.to_string())
            .or_insert_with(|| default.to_string())
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
