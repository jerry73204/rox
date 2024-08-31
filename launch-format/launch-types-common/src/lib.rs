use eyre::bail;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

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
