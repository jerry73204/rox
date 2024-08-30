#[derive(Debug, Clone)]
pub enum SubstBlock {
    Text(String),
    Substitution(Substitution),
}

#[derive(Debug, Clone)]
pub enum Substitution {
    Env {
        variable: String,
    },
    OptEnv {
        variable: String,
        default_value: Option<String>,
    },
    Find {
        pkg: String,
    },
    FindPkgShare {
        pkg: String,
    },
    Anon {
        name: String,
    },
    Arg {
        name: String,
    },
    Eval {
        expr: String,
    },
    DirName,
    Other {
        args: Vec<String>,
    },
}
