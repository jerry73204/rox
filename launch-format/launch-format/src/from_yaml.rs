use launch_types_yaml as yaml;

impl From<yaml::Launch> for crate::Launch {
    fn from(src: yaml::Launch) -> Self {
        let yaml::Launch { children } = src;

        Self {
            children: children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl From<yaml::LaunchChild> for crate::LaunchChild {
    fn from(src: yaml::LaunchChild) -> Self {
        match src {
            yaml::LaunchChild::Arg(arg) => crate::DeclareArg::from(arg).into(),
            yaml::LaunchChild::Let(let_) => crate::Let::from(let_).into(),
            yaml::LaunchChild::Executable(exec) => crate::Executable::from(exec).into(),
            yaml::LaunchChild::Node(node) => crate::Node::from(node).into(),
            yaml::LaunchChild::Group(group) => crate::Group::from(group).into(),
            yaml::LaunchChild::Include(include) => crate::Include::from(include).into(),
            yaml::LaunchChild::SetEnv(setenv) => crate::SetEnv::from(setenv).into(),
            yaml::LaunchChild::UnsetEnv(unsetenv) => crate::UnsetEnv::from(unsetenv).into(),
        }
    }
}

impl From<yaml::DeclareArg> for crate::DeclareArg {
    fn from(src: yaml::DeclareArg) -> Self {
        let yaml::DeclareArg {
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

impl From<yaml::Let> for crate::Let {
    fn from(src: yaml::Let) -> Self {
        let yaml::Let { name, value } = src;
        Self { name, value }
    }
}

impl From<yaml::Executable> for crate::Executable {
    fn from(src: yaml::Executable) -> Self {
        let yaml::Executable {
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

impl From<yaml::Node> for crate::Node {
    fn from(src: yaml::Node) -> Self {
        let yaml::Node {
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

impl From<yaml::Group> for crate::Group {
    fn from(src: yaml::Group) -> Self {
        let yaml::Group {
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

impl From<yaml::Include> for crate::Include {
    fn from(src: yaml::Include) -> Self {
        let yaml::Include {
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

impl From<yaml::SetEnv> for crate::SetEnv {
    fn from(src: yaml::SetEnv) -> Self {
        let yaml::SetEnv {
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

impl From<yaml::UnsetEnv> for crate::UnsetEnv {
    fn from(src: yaml::UnsetEnv) -> Self {
        let yaml::UnsetEnv { name, r#if, unless } = src;
        Self { name, r#if, unless }
    }
}

impl From<yaml::GroupChild> for crate::GroupChild {
    fn from(src: yaml::GroupChild) -> Self {
        match src {
            yaml::GroupChild::Executable(exec) => crate::Executable::from(exec).into(),
            yaml::GroupChild::Node(node) => crate::Node::from(node).into(),
            yaml::GroupChild::Group(group) => crate::Group::from(group).into(),
            yaml::GroupChild::Include(include) => crate::Include::from(include).into(),
            yaml::GroupChild::SetEnv(setenv) => crate::SetEnv::from(setenv).into(),
            yaml::GroupChild::UnsetEnv(unsetenv) => crate::UnsetEnv::from(unsetenv).into(),
            yaml::GroupChild::Let(let_) => crate::Let::from(let_).into(),
            yaml::GroupChild::Arg(arg) => crate::DeclareArg::from(arg).into(),
            yaml::GroupChild::PushRosNamespace(push_ros_namespace) => {
                crate::PushRosNamespace::from(push_ros_namespace).into()
            }
        }
    }
}

impl From<yaml::Env> for crate::Env {
    fn from(src: yaml::Env) -> Self {
        let yaml::Env { name, value } = src;
        Self { name, value }
    }
}

impl From<yaml::IncludeArg> for crate::IncludeArg {
    fn from(src: yaml::IncludeArg) -> Self {
        let yaml::IncludeArg { name, value } = src;
        Self { name, value }
    }
}

impl From<yaml::PushRosNamespace> for crate::PushRosNamespace {
    fn from(src: yaml::PushRosNamespace) -> Self {
        let yaml::PushRosNamespace { namespace } = src;
        Self { namespace }
    }
}

impl From<yaml::NodeChild> for crate::NodeChild {
    fn from(src: yaml::NodeChild) -> Self {
        match src {
            yaml::NodeChild::Env(env) => crate::Env::from(env).into(),
            yaml::NodeChild::Param(param) => crate::Param::from(param).into(),
            yaml::NodeChild::Remap(remap) => crate::Remap::from(remap).into(),
        }
    }
}

impl From<yaml::Param> for crate::Param {
    fn from(src: yaml::Param) -> Self {
        let yaml::Param {
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

impl From<yaml::Remap> for crate::Remap {
    fn from(src: yaml::Remap) -> Self {
        let yaml::Remap { from, to } = src;
        Self { from, to }
    }
}
