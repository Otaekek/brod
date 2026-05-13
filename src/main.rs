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
use colored::Colorize;

use crate::parser::ASTBuilder;

#[derive(Clone, Debug, Parser)]
struct CliArgs {
    source_path: Option<PathBuf>,
}

use reedline::{DefaultPrompt, Reedline, Signal};

fn run(source: &str, source_name: String) -> bool {
    let tokens = lexer::lex(source.to_owned(), source_name);
    let ast = ASTBuilder::parse(tokens);
    for err in &ast.1 {
        eprintln!("{} {}", "Error:".red(), err);
    }
    if ast.1.is_empty() {
        println!("{}", "Ok".green());
    }
    println!("{:#?}", ast.0);
    // println!("{:#?}", rpn::RpnCalculator::solve(&ast));
    false
}

fn run_repl() {
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::new(
        reedline::DefaultPromptSegment::Basic("Brod".to_string()),
        reedline::DefaultPromptSegment::CurrentDateTime,
    );
    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                break;
            }
            Ok(Signal::Success(x)) => {
                run(&(x), "prompt".to_string());
            }
            _ => break,
        }
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
        run_repl();
    }
    // Lexer::new("Source".to_string()).lex();
}
