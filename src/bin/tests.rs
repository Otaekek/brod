use std::{path::PathBuf, str::FromStr};

use brod::{
    interpreter::{foreign_function::register_foreign_functions, interpreter::Interpreter},
    load_prelude,
    parser::parser::AST,
};

pub fn main() {
    for file in std::fs::read_dir("./tests").unwrap() {
        let name = "./tests/".to_string() + &file.unwrap().file_name().into_string().unwrap();
        let mut ast = AST::default();
        let mut interpreter = Interpreter::new();
        register_foreign_functions(&mut ast);
        load_prelude(&mut interpreter, &mut ast);
        println!("Testing {}", name);
        brod::run_file(
            PathBuf::from_str(&name).unwrap(),
            &mut interpreter,
            &mut ast,
        );
    }
}
