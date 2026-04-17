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
fn run(source: &str) {
    print!("{source}");
}
fn prompt() {
    let mut input_buf = String::with_capacity(1024);
    loop {
        print!("brod> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut input_buf).unwrap();
        run(&input_buf);
        input_buf.clear();
    }
}

fn run_file(source: PathBuf) {
    let buf = read(source).unwrap();
    let as_str = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
    for line in as_str.lines() {
        run(line);
    }
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
}
