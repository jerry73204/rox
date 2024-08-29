use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    pub dir: PathBuf,
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    ros_repo::resolve(args.dir)?;
    Ok(())
}
