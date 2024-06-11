use anyhow::bail;
use std::env::VarError;

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

impl Substitution {
    pub fn eval(&self) -> anyhow::Result<String> {
        let text = match self {
            Substitution::Env { variable } => std::env::var(variable)?,
            Substitution::OptEnv {
                variable,
                default_value,
            } => match std::env::var(variable) {
                Ok(value) => value,
                Err(VarError::NotPresent) => match default_value {
                    Some(value) => value.to_string(),
                    None => bail!("the value of '{variable}' is not set"),
                },
                Err(VarError::NotUnicode(_value)) => {
                    bail!("the value of '{variable}' is not Unicode")
                }
            },
            Substitution::Find { pkg } => todo!(),
            Substitution::Anon { name } => todo!(),
            Substitution::Arg { name } => todo!(),
            Substitution::Eval { expr } => todo!(),
            Substitution::DirName => todo!(),
            Substitution::Other { args } => todo!(),
        };

        Ok(text)
    }
}
