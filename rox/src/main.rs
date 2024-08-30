use clap::Parser;
use eyre::bail;
use itertools::Itertools;
use std::path::PathBuf;

#[derive(Parser)]
enum Args {
    Repo(Repo),
    Launch(Launch),
}

#[derive(Parser)]
struct Repo {
    pub dir: PathBuf,
}

#[derive(Parser)]
struct Launch {
    pub file: PathBuf,
    pub args: Vec<String>,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();

    match args {
        Args::Repo(repo) => {
            ros_repo::resolve(repo.dir.join("src"))?;
        }
        Args::Launch(launch) => {
            let args: Vec<_> = launch
                .args
                .into_iter()
                .map(|arg| {
                    let Some((name, value)) = arg.split_once(":=") else {
                        bail!(
                            "'{arg}' is not a valid assignment. \
                               It should be in NAME:=VALUE format."
                        );
                    };
                    eyre::Ok((name.to_string(), value.to_string()))
                })
                .try_collect()?;

            launch_parse::load_launch_file(launch.file, args)?;
        }
    }

    Ok(())
}
