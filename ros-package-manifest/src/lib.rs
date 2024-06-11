use std::borrow::Cow;
use strong_xml::{XmlRead, XmlWrite};

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "package")]
pub struct Package<'a> {
    #[xml(attr = "format")]
    pub format: u32,

    #[xml(child = "name")]
    pub name: Name<'a>,

    #[xml(child = "version")]
    pub version: Version<'a>,

    #[xml(child = "description")]
    pub description: Description<'a>,

    #[xml(child = "license")]
    pub license: License<'a>,

    #[xml(child = "maintainer")]
    pub maintainer: Vec<Maintainer<'a>>,

    #[xml(child = "depend")]
    pub depend: Vec<Depend<'a>>,

    #[xml(child = "buildtool_depend")]
    pub buildtool_depend: Vec<BuildToolDepend<'a>>,

    #[xml(child = "exec_depend")]
    pub exec_depend: Vec<ExecDepend<'a>>,

    #[xml(child = "test_depend")]
    pub test_depend: Vec<TestDepend<'a>>,

    #[xml(child = "export")]
    pub export: Export<'a>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "name")]
pub struct Name<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "version")]
pub struct Version<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "description")]
pub struct Description<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "license")]
pub struct License<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "maintainer")]
pub struct Maintainer<'a> {
    #[xml(attr = "email")]
    pub email: Cow<'a, str>,
    #[xml(text)]
    pub name: Cow<'a, str>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "depend")]
pub struct Depend<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "buildtool_depend")]
pub struct BuildToolDepend<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "exec_depend")]
pub struct ExecDepend<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "test_depend")]
pub struct TestDepend<'a>(#[xml(text)] pub Cow<'a, str>);

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "export")]
pub struct Export<'a> {
    #[xml(child = "build_type")]
    pub build_type: BuildType<'a>,
}

#[derive(Debug, Clone, XmlRead, XmlWrite)]
#[xml(tag = "build_type")]
pub struct BuildType<'a>(#[xml(text)] pub Cow<'a, str>);
