mod foreign_function;
mod interpreter;
mod lexer;
mod parser;
use std::{fs::read, path::PathBuf, process::exit};

use clap::Parser;
use colored::Colorize;

use crate::{
    interpreter::Interpreter,
    parser::{ASTBuilder, Primary},
};

#[derive(Clone, Debug, Parser)]
struct CliArgs {
    source_path: Option<PathBuf>,
}

use reedline::{DefaultPrompt, Reedline, Signal};

fn run(source: &str, source_name: String, interpreter: &mut Interpreter) -> bool {
    let tokens = lexer::lex(source.to_owned(), source_name.clone());
    if let Err(err) = tokens {
        eprintln!("{} {}", "Lexing Error:".red(), err);
    } else if let Ok(tokens) = tokens {
        let ast = ASTBuilder::parse(tokens);
        for err in &ast.1 {
            eprintln!(
                "{} {}",
                "Parsing Error:".red(),
                err.get_formated_error(&source_name)
            );
        }
        if ast.1.is_empty() {
            let result = interpreter::eval(ast.0.clone(), interpreter);
            match result {
                Ok(v) => {
                    if v != Primary::Nil {
                        println!("{}: {}", "Ok".green(), v)
                    }
                }
                Err(err) => eprintln!(
                    "{} {}",
                    "Runtime Error:".red(),
                    err.get_formated_error(&ast.0, &source_name)
                ),
            }
        }
    }
    // println!("{:#?}", ast.0);
    false
}

fn run_repl(interpreter: &mut Interpreter) {
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
                run(&(x + ";"), "prompt".to_string(), interpreter);
            }
            _ => break,
        }
    }
}

fn run_file(source: PathBuf, interpreter: &mut Interpreter) {
    let buf = read(&source).unwrap();
    let as_str = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
    run(&as_str, source.display().to_string(), interpreter);
}
fn main() {
    let mut interpreter = Interpreter::new();
    let args = CliArgs::parse();
    if let Some(source) = args.source_path {
        if source.extension().and_then(|ext| ext.to_str()) != Some("brod") {
            eprintln!("Error: Only .brod files are accepted");
            exit(1);
        }
        println!("Running script {} ...", source.display());
        run_file(source, &mut interpreter);
    } else {
        println!("Running prompt ...");
        run_repl(&mut interpreter);
    }
    // Lexer::new("Source".to_string()).lex();
}
