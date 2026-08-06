use brod::{
    interpreter::{foreign_function::register_foreign_functions, interpreter::Interpreter},
    load_prelude,
    parser::parser::AST,
    run_file, run_repl,
};
use clap::Parser;

use std::{path::PathBuf, process::exit};

#[derive(Clone, Debug, Parser)]
struct CliArgs {
    source_path: Option<PathBuf>,
    #[arg(short, long)]
    interactive: bool,
    #[arg(long)]
    no_prelude: bool,
}
fn main() {
    let mut interpreter = Interpreter::new();
    let mut ast = AST::default();
    register_foreign_functions(&mut ast);
    let args = CliArgs::parse();
    if !args.no_prelude {
        load_prelude(&mut interpreter, &mut ast);
    }
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
