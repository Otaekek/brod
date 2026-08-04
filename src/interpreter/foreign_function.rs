use crate::{
    interpreter::{
        environment::Environment,
        functions::ForeignFunction,
        interpreter::{Interpreter, InterpretorError, RTObject},
    },
    parser::parser::Primary,
};

pub fn sin(input: &[RTObject], _env: &mut Environment) -> Result<RTObject, InterpretorError> {
    match input[0].get_primary()? {
        Primary::Number(n) => Ok(Primary::Number(n.sin()).to_object()),
        _ => Err(InterpretorError::FobbiddenTernay(None)),
    }
}

pub fn cos(input: &[RTObject], _env: &mut Environment) -> Result<RTObject, InterpretorError> {
    match input[0].get_primary()? {
        Primary::Number(n) => Ok(Primary::Number(n.cos()).to_object()),
        _ => Err(InterpretorError::FobbiddenTernay(None)),
    }
}

pub fn abs(input: &[RTObject], _env: &mut Environment) -> Result<RTObject, InterpretorError> {
    match input[0].get_primary()? {
        Primary::Number(n) => Ok(Primary::Number(n.abs()).to_object()),
        _ => Err(InterpretorError::FobbiddenTernay(None)),
    }
}

pub fn exit(input: &[RTObject], _env: &mut Environment) -> Result<RTObject, InterpretorError> {
    if !input.is_empty() {
        println!("{}", input[0]);
    }
    std::process::exit(0);
}

pub fn now(_input: &[RTObject], _env: &mut Environment) -> Result<RTObject, InterpretorError> {
    let now = chrono::Utc::now().to_string();
    Ok(Primary::String(now).to_object())
}

pub fn env(_: &[RTObject], env: &mut Environment) -> Result<RTObject, InterpretorError> {
    println!("{:#?}", env);
    Ok(RTObject::default())
}

pub fn init_foreign_functions(interpreter: &mut Interpreter) {
    interpreter.bind_forein("sin", ForeignFunction::new(sin, 1));
    interpreter.bind_forein("cos", ForeignFunction::new(cos, 1));
    interpreter.bind_forein("abs", ForeignFunction::new(abs, 1));
    interpreter.bind_forein("env", ForeignFunction::new(env, 0));
    interpreter.bind_forein("now", ForeignFunction::new(now, 0));
    interpreter.bind_forein("exit", ForeignFunction::new(exit, 0));
}
