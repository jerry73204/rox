use crate::{
    Executable, Group, GroupChild, Include, IncludeArg, Launch, LaunchArg, LaunchChild, Let, Node,
    NodeChild, Output, Param, Remap, SetEnv, UnsetEnv,
};
use anyhow::{bail, ensure, Context, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct LaunchProfile {
    pub execs: Vec<ParsedExecutable>,
    pub nodes: Vec<ParsedNode>,
}

pub fn load_launch_file<P, I>(path: P, args: I) -> Result<LaunchProfile>
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
    let profile = LaunchProfile { execs, nodes };

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
        let reader = BufReader::new(
            File::open(path).with_context(|| format!("Unable to open {}", path.display()))?,
        );
        serde_xml_rs::from_reader(reader)
            .with_context(|| format!("Unable to parse {}", path.display()))?
    } else {
        bail!(
            "The launch file must ends with '.xml' or '.yaml': {}",
            path.display()
        );
    };

    state.push_wd(parent.to_path_buf());
    state.push_scope();

    for (name, value) in args {
        state.insert_var(name, value);
    }

    parse_launch(&launch, state)?;

    state.pop_scope();
    state.pop_wd();

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
                    state.get_var_or_default(name, default);
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
    if scoped {
        state.push_scope();
    }

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

    if scoped {
        state.pop_scope();
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
    execs: Vec<ParsedExecutable>,
    nodes: Vec<ParsedNode>,
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
        let ret = match self.eval(text)?.as_str() {
            "true" => true,
            "false" => false,
            _ => bail!(""),
        };
        Ok(ret)
    }

    pub fn eval(&self, text: &str) -> Result<String> {
        todo!();
    }

    pub fn push_wd(&mut self, wd: PathBuf) {
        self.work_dirs.push(wd);
    }

    pub fn pop_wd(&mut self) {
        self.work_dirs.pop();
    }

    pub fn cwd(&self) -> &Path {
        self.work_dirs.last().unwrap()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop().unwrap();
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

    pub fn get_var_or_default(&mut self, name: &str, default: &str) -> &str {
        self.current_scope_mut()
            .var
            .entry(name.to_string())
            .or_insert_with(|| default.to_string())
    }

    pub fn get_env_or_default(&mut self, name: &str, default: &str) -> &str {
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

#[derive(Debug, Clone)]
pub struct ParsedExecutable {
    pub cmd: PathBuf,
    pub cwd: Option<PathBuf>,
    pub name: Option<String>,
    pub args: Option<String>,
    pub shell: Option<String>,
    pub launch_prefix: Option<String>,
    pub output: Option<Output>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ParsedNode {
    pub pkg: String,
    pub exec: String,
    pub name: Option<String>,
    pub ros_args: Option<String>,
    pub args: Option<String>,
    pub namespace: Option<String>,
    pub launch_prefix: Option<String>,
    pub output: Option<Output>,
    pub env: HashMap<String, String>,
    pub param: Vec<Param>,
    pub remap: Vec<Remap>,
}
