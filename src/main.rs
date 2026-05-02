mod lexer;
mod parser;
pub mod rpn;
use std::{
    fs::read,
    io::{stdin, stdout, Write},
    path::PathBuf,
    process::exit,
};

use clap::Parser;

#[derive(Clone, Debug, Parser)]
struct CliArgs {
    source_path: Option<PathBuf>,
}
fn run(source: &str, source_name: String) -> bool {
    let tokens = lexer::lex(source.to_owned(), source_name);
    print!("{}", tokens);
    false
}

fn prompt() {
    let mut input_buf = String::with_capacity(1024);
    loop {
        print!("brod> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input_buf).unwrap();
        run(&input_buf, "prompt".to_string());
        input_buf.clear();
    }
}

fn run_file(source: PathBuf) {
    let buf = read(&source).unwrap();
    let as_str = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
    run(&as_str, source.display().to_string());
}
fn main() {
    let args = CliArgs::parse();
    if let Some(source) = args.source_path {
        if source.extension().and_then(|ext| ext.to_str()) != Some("brod") {
            eprintln!("Error: Only .brod files are accepted");
            exit(1);
        }
        println!("Running script {} ...", source.display());
        run_file(source);
    } else {
        println!("Running prompt ...");
        prompt();
    }
    // Lexer::new("Source".to_string()).lex();
}
