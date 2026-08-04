use std::{path::PathBuf, str::FromStr};

use brod::{interpreter::interpreter::Interpreter, parser::parser::AST};

pub fn main() {
    for file in std::fs::read_dir("./tests").unwrap() {
        let name = "./tests/".to_string() + &file.unwrap().file_name().into_string().unwrap();
        let mut ast = AST::default();
        let mut interpreter = Interpreter::new();
        println!("Testing {}", name);
        brod::run_file(
            PathBuf::from_str(&name).unwrap(),
            &mut interpreter,
            &mut ast,
        );
    }
}
