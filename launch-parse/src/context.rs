use launch_format::{Output, Param, Remap};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Launch {
    pub execs: Vec<Executable>,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Executable {
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
pub struct Node {
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
