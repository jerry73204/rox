use eyre::bail;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launch {
    #[serde(default, rename = "$value")]
    pub children: Vec<LaunchChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LaunchChild {
    Arg(Arg),
    Let(Let),
    Executable(Executable),
    Node(Node),
    Group(Group),
    Include(Include),
    SetEnv(SetEnv),
    UnsetEnv(UnsetEnv),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arg {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@value")]
    pub value: Option<String>,

    #[serde(rename = "@default")]
    pub default: Option<String>,

    #[serde(rename = "@description")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Let {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    #[serde(rename = "@file")]
    pub file: String,

    #[serde(rename = "@if")]
    pub r#if: Option<String>,

    #[serde(rename = "@unless")]
    pub unless: Option<String>,

    #[serde(default)]
    pub arg: Vec<IncludeArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeArg {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "@pkg")]
    pub pkg: String,

    #[serde(rename = "@exec")]
    pub exec: String,

    #[serde(rename = "@name")]
    pub name: Option<String>,

    #[serde(rename = "@ros-arg")]
    pub ros_args: Option<String>,

    #[serde(rename = "@arg")]
    pub args: Option<String>,

    #[serde(rename = "@namespace")]
    pub namespace: Option<String>,

    #[serde(rename = "@launch-prefix")]
    pub launch_prefix: Option<String>,

    #[serde(rename = "@output")]
    pub output: Option<Output>,

    #[serde(rename = "@if")]
    pub r#if: Option<String>,

    #[serde(rename = "@unless")]
    pub unless: Option<String>,

    #[serde(default, rename = "$value")]
    pub children: Vec<NodeChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeChild {
    Env(Env),
    Param(Param),
    Remap(Remap),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remap {
    #[serde(rename = "@from")]
    pub from: String,

    #[serde(rename = "@to")]
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    #[serde(rename = "@name")]
    pub name: Option<String>,

    #[serde(rename = "@from")]
    pub from: Option<String>,

    #[serde(rename = "@sep")]
    pub sep: Option<String>,

    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Executable {
    #[serde(rename = "@cmd")]
    pub cmd: String,

    #[serde(rename = "@cwd")]
    pub cwd: Option<String>,

    #[serde(rename = "@name")]
    pub name: Option<String>,

    #[serde(rename = "@args")]
    pub args: Option<String>,

    #[serde(rename = "@shell")]
    pub shell: Option<String>,

    #[serde(rename = "@launch-prefix")]
    pub launch_prefix: Option<String>,

    #[serde(rename = "@output")]
    pub output: Option<Output>,

    #[serde(rename = "@if")]
    pub r#if: Option<String>,

    #[serde(rename = "@unless")]
    pub unless: Option<String>,

    #[serde(default)]
    pub env: Vec<Env>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "@scoped")]
    pub scoped: Option<bool>,

    #[serde(rename = "@if")]
    pub r#if: Option<String>,

    #[serde(rename = "@unless")]
    pub unless: Option<String>,

    #[serde(default, rename = "$value")]
    pub children: Vec<GroupChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GroupChild {
    Executable(Executable),
    Node(Node),
    Group(Group),
    Include(Include),
    SetEnv(SetEnv),
    UnsetEnv(UnsetEnv),
    Let(Let),
    Arg(Arg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEnv {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@value")]
    pub value: String,

    #[serde(rename = "@if")]
    pub r#if: Option<String>,

    #[serde(rename = "@unless")]
    pub unless: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsetEnv {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@if")]
    pub r#if: Option<String>,

    #[serde(rename = "@unless")]
    pub unless: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Env {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Output {
    Log,
    Screen,
}

impl FromStr for Output {
    type Err = eyre::Error;

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
