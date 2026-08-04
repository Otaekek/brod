use brod::{
    diagnostic::render,
    interpreter::interpreter::{self, Interpreter},
    lexer::lexer,
    parser::parser::{AST, ASTBuilder},
};
use clap::Parser;
use colored::Colorize;
use std::{fs::read, path::PathBuf, process::exit};

#[derive(Clone, Debug, Parser)]
struct CliArgs {
    source_path: Option<PathBuf>,
    /// After running the script, drop into an interactive prompt sharing its state.
    #[arg(short, long)]
    interactive: bool,
}

use reedline::{DefaultPrompt, Reedline, Signal};

fn run(source: &str, source_name: String, interpreter: &mut Interpreter, ast: &mut AST) -> bool {
    let tokens = lexer::lex(source.to_owned());
    if let Err(err) = tokens {
        eprintln!("{}", render(&err, source, &source_name));
    } else if let Ok(tokens) = tokens {
        let res = ASTBuilder::parse(tokens, ast);
        for err in &res.1 {
            eprintln!("{}", render(err, source, &source_name));
        }
        if res.1.is_empty() {
            let result = interpreter::eval(ast.clone(), interpreter);
            match result {
                Ok(v) => {
                    println!("{}: {}", "Ok".green(), v)
                }
                Err(err) => eprintln!("{}", render(&err, source, &source_name)),
            }
        }
    }
    false
}

fn run_repl(interpreter: &mut Interpreter, ast: &mut AST) {
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

fn run_file(source: PathBuf, interpreter: &mut Interpreter, ast: &mut AST) {
    let buf = read(&source).unwrap();
    let as_str = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
    run(&as_str, source.display().to_string(), interpreter, ast);
}
fn main() {
    let mut interpreter = Interpreter::new();
    let mut ast = AST::default();
    let args = CliArgs::parse();
    if let Some(source) = args.source_path {
        if source.extension().and_then(|ext| ext.to_str()) != Some("brod") {
            eprintln!("Error: Only .brod files are accepted");
            exit(1);
        }
        println!("Running script {} ...", source.display());
        run_file(source, &mut interpreter, &mut ast);
        if args.interactive {
            println!("Running prompt ...");
            run_repl(&mut interpreter, &mut ast);
        }
    } else {
        println!("Running prompt ...");
        run_repl(&mut interpreter, &mut ast);
    }
    // Lexer::new("Source".to_string()).lex();
}
