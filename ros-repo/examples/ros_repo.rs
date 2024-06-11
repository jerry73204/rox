use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    pub dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    ros_repo::resolve(args.dir)?;
    Ok(())
}
