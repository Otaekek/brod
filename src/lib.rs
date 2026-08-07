pub mod containers;
pub mod diagnostic;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod vm;
use crate::{
    diagnostic::render,
    interpreter::interpreter::Interpreter,
    parser::{
        parser::{AST, ASTBuilder},
        resolver,
    },
};

use colored::Colorize;
use std::{fs::read, path::PathBuf};

use reedline::{DefaultPrompt, Reedline, Signal};

fn run(source: &str, source_name: String, interpreter: &mut Interpreter, ast: &mut AST) -> bool {
    let tokens = lexer::lexer::lex(source.to_owned());
    if let Err(err) = tokens {
        eprintln!("{}", render(&err, source, &source_name));
    } else if let Ok(tokens) = tokens {
        let res = ASTBuilder::parse(tokens, ast);

        for err in &res.1 {
            eprintln!("{}", render(err, source, &source_name));
        }
        if res.1.is_empty() {
            let mut resolver = resolver::Resolver::init();
            match resolver.resolve(ast) {
                Ok(()) => {
                    let result = interpreter::interpreter::eval(ast.clone(), interpreter);
                    match result {
                        Ok(v) => {
                            println!("{}: {}", "Ok".green(), v)
                        }
                        Err(err) => eprintln!("{}", render(&err, source, &source_name)),
                    }
                }
                Err(err) => eprintln!("{}", render(&err, source, &source_name)),
            }
        }
    }
    false
}

pub fn run_repl(interpreter: &mut Interpreter, ast: &mut AST) {
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
                run(&(x + ";"), "prompt".to_string(), interpreter, ast);
            }
            _ => break,
        }
    }
}

pub fn run_file(source: PathBuf, interpreter: &mut Interpreter, ast: &mut AST) {
    let buf = read(&source).unwrap();
    let as_str = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
    run(&as_str, source.display().to_string(), interpreter, ast);
}

const PRELUDE_PATH: &str = "prelude/prelude.brod";

pub fn load_prelude(interpreter: &mut Interpreter, ast: &mut AST) {
    let path = PathBuf::from(PRELUDE_PATH);
    match read(&path) {
        Ok(buf) => {
            let source = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
            run(&source, path.display().to_string(), interpreter, ast);
        }
        Err(_) => {
            eprintln!(
                "{}: could not find prelude at {} — continuing without it (use --no-prelude to silence this)",
                "Warning".yellow(),
                path.display()
            );
        }
    }
}
