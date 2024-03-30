use std::{collections::VecDeque, io::Read};

use anyhow::{Context, Ok, Result};
use clap::Parser;

use crate::code_gen::generate_code;

mod lexical;
mod code_gen;
mod parsing;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    file_name: String
}

fn main() -> Result<()> {
    let arg = Args::parse();
    let file_name = arg.file_name;

    let mut file = std::fs::File::open(file_name)
        .context("Failed to read the File")?;

    println!("Successfully read file");
    println!("file size = {}", file.metadata().unwrap().len());

    let mut content = String::new();
    file.read_to_string(&mut content).context("Failed to read file content")?;

    println!("Lexical Analysis being performed...");
    let tokens = lexical::tokenize(&content);
    println!("Lexical Analysis Completed.\n");
    println!("Parsing being performed...");

    let tokens = VecDeque::from(tokens);
    let nodes = parsing::parse(tokens);
    println!("Parsing completed.");
/*
    for node in nodes.stmts.iter() {
        println!("Node {:?}\n", node);
    }
*/

    println!();
    println!("Code Generation being performed...");

    generate_code(nodes);

    println!("Code Generation Completed");

    Ok(())
}
