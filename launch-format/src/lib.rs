use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};
use strong_xml::{XmlRead, XmlWrite};

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "launch")]
pub struct Launch {
    #[xml(child = "")]
    pub children: Vec<LaunchChild>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchChild {
    #[xml(tag = "arg")]
    Arg(LaunchArg),
    #[xml(tag = "let")]
    Let(Let),
    #[xml(tag = "executable")]
    Executable(Executable),
    #[xml(tag = "node")]
    Node(Node),
    #[xml(tag = "group")]
    Group(Group),
    #[xml(tag = "include")]
    Include(Include),
    #[xml(tag = "set-env")]
    SetEnv(SetEnv),
    #[xml(tag = "unset-env")]
    UnsetEnv(UnsetEnv),
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "arg")]
pub struct LaunchArg {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "value")]
    pub value: Option<String>,
    #[xml(attr = "default")]
    pub default: Option<String>,
    #[xml(attr = "description")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "let")]
pub struct Let {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "value")]
    pub value: String,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "include")]
pub struct Include {
    #[xml(attr = "file")]
    pub file: String,

    #[xml(attr = "if")]
    pub r#if: Option<String>,

    #[xml(attr = "unless")]
    pub unless: Option<String>,

    #[xml(child = "")]
    pub arg: Vec<IncludeArg>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "arg")]
pub struct IncludeArg {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "value")]
    pub value: String,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "node")]
pub struct Node {
    #[xml(attr = "pkg")]
    pub pkg: String,

    #[xml(attr = "exec")]
    pub exec: String,

    #[xml(attr = "name")]
    pub name: Option<String>,

    #[xml(attr = "ros-arg")]
    pub ros_args: Option<String>,

    #[xml(attr = "arg")]
    pub args: Option<String>,

    #[xml(attr = "namespace")]
    pub namespace: Option<String>,

    #[xml(attr = "launch-prefix")]
    pub launch_prefix: Option<String>,

    #[xml(attr = "output")]
    pub output: Option<Output>,

    #[xml(attr = "if")]
    pub r#if: Option<String>,

    #[xml(attr = "unless")]
    pub unless: Option<String>,

    #[xml(child = "")]
    pub children: Vec<NodeChild>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeChild {
    #[xml(tag = "env")]
    Env(Env),
    #[xml(tag = "param")]
    Param(Param),
    #[xml(tag = "remap")]
    Remap(Remap),
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "remap")]
pub struct Remap {
    #[xml(attr = "from")]
    pub from: String,
    #[xml(attr = "to")]
    pub to: String,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "param")]
pub struct Param {
    #[xml(attr = "name")]
    pub name: Option<String>,
    #[xml(attr = "from")]
    pub from: Option<String>,
    #[xml(attr = "sep")]
    pub sep: Option<String>,
    #[xml(attr = "value")]
    pub value: String,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "executable")]
pub struct Executable {
    #[xml(attr = "cmd")]
    pub cmd: String,

    #[xml(attr = "cwd")]
    pub cwd: Option<String>,

    #[xml(attr = "name")]
    pub name: Option<String>,

    #[xml(attr = "args")]
    pub args: Option<String>,

    #[xml(attr = "shell")]
    pub shell: Option<String>,

    #[xml(attr = "launch-prefix")]
    pub launch_prefix: Option<String>,

    #[xml(attr = "output")]
    pub output: Option<Output>,

    #[xml(attr = "if")]
    pub r#if: Option<String>,

    #[xml(attr = "unless")]
    pub unless: Option<String>,

    #[xml(child = "env")]
    pub env: Vec<Env>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "group")]
pub struct Group {
    #[xml(attr = "scoped")]
    pub scoped: Option<bool>,

    #[xml(attr = "if")]
    pub r#if: Option<String>,

    #[xml(attr = "unless")]
    pub unless: Option<String>,

    #[xml(child = "")]
    pub children: Vec<GroupChild>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GroupChild {
    #[xml(tag = "executable")]
    Executable(Executable),
    #[xml(tag = "node")]
    Node(Node),
    #[xml(tag = "group")]
    Group(Group),
    #[xml(tag = "include")]
    Include(Include),
    #[xml(tag = "set-env")]
    SetEnv(SetEnv),
    #[xml(tag = "unset-env")]
    UnsetEnv(UnsetEnv),
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "set-env")]
pub struct SetEnv {
    #[xml(attr = "name")]
    pub name: String,

    #[xml(attr = "value")]
    pub value: String,

    #[xml(attr = "if")]
    pub r#if: Option<String>,

    #[xml(attr = "unless")]
    pub unless: Option<String>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "unset-env")]
pub struct UnsetEnv {
    #[xml(attr = "name")]
    pub name: String,

    #[xml(attr = "if")]
    pub r#if: Option<String>,

    #[xml(attr = "unless")]
    pub unless: Option<String>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite, Serialize, Deserialize)]
#[xml(tag = "env")]
pub struct Env {
    #[xml(attr = "name")]
    pub name: String,

    #[xml(attr = "value")]
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Output {
    Log,
    Screen,
}

impl FromStr for Output {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let output = match s {
            "log" => Self::Log,
            "screen" => Self::Screen,
            _ => bail!("unexpected output attribute {s}"),
        };
        Ok(output)
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Output::Log => "log",
            Output::Screen => "screen",
        };
        write!(f, "{text}")
    }
}
