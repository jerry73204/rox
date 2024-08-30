use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package<'a> {
    #[serde(rename = "@format")]
    pub format: u32,
    pub name: Name<'a>,
    pub version: Version<'a>,
    // pub description: Description<'a>,
    #[serde(default)]
    pub license: Vec<License<'a>>,
    #[serde(default)]
    pub maintainer: Vec<Maintainer<'a>>,
    #[serde(default)]
    pub depend: Vec<Depend<'a>>,
    #[serde(default)]
    pub buildtool_depend: Vec<BuildToolDepend<'a>>,
    #[serde(default)]
    pub exec_depend: Vec<ExecDepend<'a>>,
    #[serde(default)]
    pub test_depend: Vec<TestDepend<'a>>,
    pub export: Export<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Name<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Description<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer<'a> {
    #[serde(rename = "@email")]
    pub email: Cow<'a, str>,

    #[serde(rename = "$text")]
    pub name: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Depend<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildToolDepend<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecDepend<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDepend<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export<'a> {
    pub build_type: Vec<BuildType<'a>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildType<'a>(#[serde(rename = "$text")] pub Cow<'a, str>);
