use launch_types_common::Output;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launch {
    #[serde(default)]
    pub children: Vec<LaunchChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareArg {
    pub name: String,
    pub default: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Let {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    pub file: String,
    pub r#if: Option<String>,
    pub unless: Option<String>,
    #[serde(default)]
    pub arg: Vec<IncludeArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeArg {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
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
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: Option<String>,
    pub from: Option<String>,
    pub sep: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default)]
    pub env: Vec<Env>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub scoped: Option<bool>,
    pub r#if: Option<String>,
    pub unless: Option<String>,
    #[serde(default)]
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
    Arg(DeclareArg),
    PushRosNamespace(PushRosNamespace),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEnv {
    pub name: String,
    pub value: String,
    pub r#if: Option<String>,
    pub unless: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsetEnv {
    pub name: String,
    pub r#if: Option<String>,
    pub unless: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Env {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRosNamespace {
    pub namespace: String,
}
