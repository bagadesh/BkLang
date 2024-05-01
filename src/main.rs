
use anyhow::{Ok, Result};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    file_name: String
}

fn main() -> Result<()> {
    let arg = Args::parse();
    let file_name = arg.file_name;
    let _ = hydrogen::main(file_name);
    Ok(())
}
