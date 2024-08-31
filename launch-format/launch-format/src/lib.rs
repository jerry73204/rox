mod from_xml;
mod from_yaml;

use eyre::WrapErr;
use std::{fs::File, io::BufReader, path::Path};

pub use launch_types_common::Output;

pub fn load_launch_file(path: impl AsRef<Path>) -> eyre::Result<Launch> {
    let path = path.as_ref();

    let Some(ext) = path.extension() else {
        todo!();
    };

    let launch: Launch = if ext == "xml" {
        let file =
            File::open(path).wrap_err_with(|| format!("unable to open file {}", path.display()))?;
        let reader = BufReader::new(file);
        let launch: launch_types_xml::Launch = quick_xml::de::from_reader(reader)?;
        launch.into()
    } else if ext == "yaml" {
        let file =
            File::open(path).wrap_err_with(|| format!("unable to open file {}", path.display()))?;
        let reader = BufReader::new(file);
        let launch: launch_types_yaml::Launch = serde_yaml::from_reader(reader)?;
        launch.into()
    } else if ext == "py" {
        // TODO: Implement Python launch file loading
        Launch { children: vec![] }
    } else {
        todo!();
    };

    Ok(launch)
}

#[derive(Debug, Clone)]
pub struct Launch {
    pub children: Vec<LaunchChild>,
}

#[derive(Debug, Clone)]
pub enum LaunchChild {
    Arg(DeclareArg),
    Let(Let),
    Executable(Executable),
    Node(Node),
    Group(Group),
    Include(Include),
    SetEnv(SetEnv),
    UnsetEnv(UnsetEnv),
}

impl From<UnsetEnv> for LaunchChild {
    fn from(v: UnsetEnv) -> Self {
        Self::UnsetEnv(v)
    }
}

impl From<SetEnv> for LaunchChild {
    fn from(v: SetEnv) -> Self {
        Self::SetEnv(v)
    }
}

impl From<Include> for LaunchChild {
    fn from(v: Include) -> Self {
        Self::Include(v)
    }
}

impl From<Group> for LaunchChild {
    fn from(v: Group) -> Self {
        Self::Group(v)
    }
}

impl From<Node> for LaunchChild {
    fn from(v: Node) -> Self {
        Self::Node(v)
    }
}

impl From<Executable> for LaunchChild {
    fn from(v: Executable) -> Self {
        Self::Executable(v)
    }
}

impl From<Let> for LaunchChild {
    fn from(v: Let) -> Self {
        Self::Let(v)
    }
}

impl From<DeclareArg> for LaunchChild {
    fn from(v: DeclareArg) -> Self {
        Self::Arg(v)
    }
}

#[derive(Debug, Clone)]
pub struct DeclareArg {
    pub name: String,
    pub default: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Let {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Include {
    pub file: String,
    pub r#if: Option<String>,
    pub unless: Option<String>,
    pub arg: Vec<IncludeArg>,
}

#[derive(Debug, Clone)]
pub struct IncludeArg {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub pkg: String,
    pub exec: String,
    pub name: Option<String>,
    pub ros_args: Option<String>,
    pub args: Option<String>,
    pub namespace: Option<String>,
    pub launch_prefix: Option<String>,
    pub output: Option<Output>,
    pub r#if: Option<String>,
    pub unless: Option<String>,
    pub children: Vec<NodeChild>,
}

#[derive(Debug, Clone)]
pub enum NodeChild {
    Env(Env),
    Param(Param),
    Remap(Remap),
}

impl From<Remap> for NodeChild {
    fn from(v: Remap) -> Self {
        Self::Remap(v)
    }
}

impl From<Param> for NodeChild {
    fn from(v: Param) -> Self {
        Self::Param(v)
    }
}

impl From<Env> for NodeChild {
    fn from(v: Env) -> Self {
        Self::Env(v)
    }
}

#[derive(Debug, Clone)]
pub struct Remap {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Option<String>,
    pub from: Option<String>,
    pub sep: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Executable {
    pub cmd: String,
    pub cwd: Option<String>,
    pub name: Option<String>,
    pub args: Option<String>,
    pub shell: Option<String>,
    pub launch_prefix: Option<String>,
    pub output: Option<Output>,
    pub r#if: Option<String>,
    pub unless: Option<String>,
    pub env: Vec<Env>,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub scoped: Option<bool>,
    pub r#if: Option<String>,
    pub unless: Option<String>,
    pub children: Vec<GroupChild>,
}

#[derive(Debug, Clone)]
pub enum GroupChild {
    Executable(Executable),
    Node(Node),
    Group(Group),
    Include(Include),
    SetEnv(SetEnv),
    UnsetEnv(UnsetEnv),
    Let(Let),
    Arg(DeclareArg),
    PushRosNamespace(PushRosNamespace),
}

impl From<PushRosNamespace> for GroupChild {
    fn from(v: PushRosNamespace) -> Self {
        Self::PushRosNamespace(v)
    }
}

impl From<DeclareArg> for GroupChild {
    fn from(v: DeclareArg) -> Self {
        Self::Arg(v)
    }
}

impl From<Let> for GroupChild {
    fn from(v: Let) -> Self {
        Self::Let(v)
    }
}

impl From<UnsetEnv> for GroupChild {
    fn from(v: UnsetEnv) -> Self {
        Self::UnsetEnv(v)
    }
}

impl From<SetEnv> for GroupChild {
    fn from(v: SetEnv) -> Self {
        Self::SetEnv(v)
    }
}

impl From<Include> for GroupChild {
    fn from(v: Include) -> Self {
        Self::Include(v)
    }
}

impl From<Group> for GroupChild {
    fn from(v: Group) -> Self {
        Self::Group(v)
    }
}

impl From<Node> for GroupChild {
    fn from(v: Node) -> Self {
        Self::Node(v)
    }
}

impl From<Executable> for GroupChild {
    fn from(v: Executable) -> Self {
        Self::Executable(v)
    }
}

#[derive(Debug, Clone)]
pub struct SetEnv {
    pub name: String,
    pub value: String,
    pub r#if: Option<String>,
    pub unless: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UnsetEnv {
    pub name: String,
    pub r#if: Option<String>,
    pub unless: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Env {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct PushRosNamespace {
    pub namespace: String,
}
