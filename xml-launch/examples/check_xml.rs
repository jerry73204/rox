use anyhow::{bail, Result};
use clap::Parser;
use itertools::Itertools;
use std::path::PathBuf;

#[derive(Parser)]
struct Opts {
    pub input_file: PathBuf,
    pub args: Vec<String>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let args: Vec<_> = opts
        .args
        .into_iter()
        .map(|arg| -> Result<_> {
            let Some((name, value)) = arg.split_once(":=") else {
                bail!("'{arg}' is not a valid assignment. It should be in NAME:=VALUE format.");
            };
            Ok((name.to_string(), value.to_string()))
        })
        .try_collect()?;

    let profile = xml_launch::load_launch_file(opts.input_file, args)?;
    dbg!(profile);

    Ok(())
}
