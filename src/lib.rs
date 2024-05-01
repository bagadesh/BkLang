
use std::process::Command;
use std::{collections::VecDeque, io::Read};

use anyhow::{Context, Ok, Result};
use clap::Parser;
use tracing_subscriber::layer::SubscriberExt;

use tracing::debug;
use tracing::info;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod lexical;
mod parsing;
mod parse_validation;
mod codegen;


#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    file_name: String
}

pub fn main(file_name: String) -> Result<String> {
    println!("{}", std::env::current_dir().unwrap().display());

    let layer = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(layer)
        .with(EnvFilter::from_default_env())
        .init();

    let mut file = std::fs::File::open(file_name)
        .context("Failed to read the File")?;

    info!("Successfully read file");
    info!("file size = {}", file.metadata().unwrap().len());

    let mut content = String::new();
    file.read_to_string(&mut content).context("Failed to read file content")?;

    info!("Lexical Analysis being performed...");
    let tokens = lexical::tokenize(&content);

    for token in tokens.iter()  {
        debug!("{:?}", token);
    }
    
    info!("Lexical Analysis Completed.\n");
    info!("Parsing being performed...");

    let tokens = VecDeque::from(tokens);
    let nodes = parsing::parse(tokens);

    info!("Parsing completed.\n");

    parse_validation::parse_validation(&nodes);

    for node in nodes.funcs.iter() {
        debug!("Node {:#?}\n", node);
    }

    info!("Code Generation being performed...");
    codegen::generate(nodes);
    info!("Code Generation Completed");

    let command = Command::new("sh")
        .arg("-c")
        .arg("as -arch arm64 out.s -o out.o && ld -o out out.o -lSystem -syslibroot `xcrun -sdk macosx --show-sdk-path` -e _start -arch arm64 && ./out; echo $?")
        .output()
        .expect("Command Failed");

    let s = command.stdout;
    let s = String::from_utf8(s).unwrap().trim().to_owned();

    print!("Result {}", s);

    Ok(s)
}
