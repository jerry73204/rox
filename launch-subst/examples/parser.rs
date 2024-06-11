use clap::Parser;
use launch_subst::parse;

#[derive(Parser)]
struct Opts {
    pub expr: String,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let output = parse(&opts.expr)?;
    dbg!(output);
    Ok(())
}
