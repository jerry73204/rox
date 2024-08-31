use launch_types_xml as xml;

impl From<xml::Launch> for crate::Launch {
    fn from(src: xml::Launch) -> Self {
        let xml::Launch { children } = src;

        Self {
            children: children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<xml::LaunchChild> for crate::LaunchChild {
    fn from(src: xml::LaunchChild) -> Self {
        match src {
            xml::LaunchChild::Arg(arg) => crate::DeclareArg::from(arg).into(),
            xml::LaunchChild::Let(let_) => crate::Let::from(let_).into(),
            xml::LaunchChild::Executable(exec) => crate::Executable::from(exec).into(),
            xml::LaunchChild::Node(node) => crate::Node::from(node).into(),
            xml::LaunchChild::Group(group) => crate::Group::from(group).into(),
            xml::LaunchChild::Include(include) => crate::Include::from(include).into(),
            xml::LaunchChild::SetEnv(setenv) => crate::SetEnv::from(setenv).into(),
            xml::LaunchChild::UnsetEnv(unsetenv) => crate::UnsetEnv::from(unsetenv).into(),
        }
    }
}

impl From<xml::DeclareArg> for crate::DeclareArg {
    fn from(src: xml::DeclareArg) -> Self {
        let xml::DeclareArg {
            name,
            default,
            description,
        } = src;
        Self {
            name,
            default,
            description,
        }
    }
}

impl From<xml::Let> for crate::Let {
    fn from(src: xml::Let) -> Self {
        let xml::Let { name, value } = src;
        Self { name, value }
    }
}

impl From<xml::Executable> for crate::Executable {
    fn from(src: xml::Executable) -> Self {
        let xml::Executable {
            cmd,
            cwd,
            name,
            args,
            shell,
            launch_prefix,
            output,
            r#if,
            unless,
            env,
        } = src;
        Self {
            cmd,
            cwd,
            name,
            args,
            shell,
            launch_prefix,
            output,
            r#if,
            unless,
            env: env.into_iter().map(crate::Env::from).collect(),
        }
    }
}

impl From<xml::Node> for crate::Node {
    fn from(src: xml::Node) -> Self {
        let xml::Node {
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
        } = src;
        Self {
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
            children: children.into_iter().map(crate::NodeChild::from).collect(),
        }
    }
}

impl From<xml::Group> for crate::Group {
    fn from(src: xml::Group) -> Self {
        let xml::Group {
            scoped,
            r#if,
            unless,
            children,
        } = src;
        Self {
            scoped,
            r#if,
            unless,
            children: children.into_iter().map(crate::GroupChild::from).collect(),
        }
    }
}

impl From<xml::Include> for crate::Include {
    fn from(src: xml::Include) -> Self {
        let xml::Include {
            file,
            r#if,
            unless,
            arg,
        } = src;
        Self {
            file,
            r#if,
            unless,
            arg: arg.into_iter().map(crate::IncludeArg::from).collect(),
        }
    }
}

impl From<xml::SetEnv> for crate::SetEnv {
    fn from(src: xml::SetEnv) -> Self {
        let xml::SetEnv {
            name,
            value,
            r#if,
            unless,
        } = src;
        Self {
            name,
            value,
            r#if,
            unless,
        }
    }
}

impl From<xml::UnsetEnv> for crate::UnsetEnv {
    fn from(src: xml::UnsetEnv) -> Self {
        let xml::UnsetEnv { name, r#if, unless } = src;
        Self { name, r#if, unless }
    }
}

impl From<xml::GroupChild> for crate::GroupChild {
    fn from(src: xml::GroupChild) -> Self {
        match src {
            xml::GroupChild::Executable(exec) => crate::Executable::from(exec).into(),
            xml::GroupChild::Node(node) => crate::Node::from(node).into(),
            xml::GroupChild::Group(group) => crate::Group::from(group).into(),
            xml::GroupChild::Include(include) => crate::Include::from(include).into(),
            xml::GroupChild::SetEnv(setenv) => crate::SetEnv::from(setenv).into(),
            xml::GroupChild::UnsetEnv(unsetenv) => crate::UnsetEnv::from(unsetenv).into(),
            xml::GroupChild::Let(let_) => crate::Let::from(let_).into(),
            xml::GroupChild::Arg(arg) => crate::DeclareArg::from(arg).into(),
            xml::GroupChild::PushRosNamespace(push_ros_namespace) => {
                crate::PushRosNamespace::from(push_ros_namespace).into()
            }
        }
    }
}

impl From<xml::Env> for crate::Env {
    fn from(src: xml::Env) -> Self {
        let xml::Env { name, value } = src;
        Self { name, value }
    }
}

impl From<xml::IncludeArg> for crate::IncludeArg {
    fn from(src: xml::IncludeArg) -> Self {
        let xml::IncludeArg { name, value } = src;
        Self { name, value }
    }
}

impl From<xml::PushRosNamespace> for crate::PushRosNamespace {
    fn from(src: xml::PushRosNamespace) -> Self {
        let xml::PushRosNamespace { namespace } = src;
        Self { namespace }
    }
}

impl From<xml::NodeChild> for crate::NodeChild {
    fn from(src: xml::NodeChild) -> Self {
        match src {
            xml::NodeChild::Env(env) => crate::Env::from(env).into(),
            xml::NodeChild::Param(param) => crate::Param::from(param).into(),
            xml::NodeChild::Remap(remap) => crate::Remap::from(remap).into(),
        }
    }
}

impl From<xml::Param> for crate::Param {
    fn from(src: xml::Param) -> Self {
        let xml::Param {
            name,
            from,
            sep,
            value,
        } = src;
        Self {
            name,
            from,
            sep,
            value,
        }
    }
}

impl From<xml::Remap> for crate::Remap {
    fn from(src: xml::Remap) -> Self {
        let xml::Remap { from, to } = src;
        Self { from, to }
    }
}
