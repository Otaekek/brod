use crate::{
    interpreter::{Environment, ForeignFunction, Interpreter, InterpretorError},
    parser::{LocatedPrimary, Primary},
};

pub fn sin(input: &[LocatedPrimary], _env: &mut Environment) -> Result<Primary, InterpretorError> {
    match input[0].inner {
        Primary::Number(n) => Ok(Primary::Number(n.sin())),
        _ => Err(InterpretorError::FobbiddenTernay),
    }
}

pub fn cos(input: &[LocatedPrimary], _env: &mut Environment) -> Result<Primary, InterpretorError> {
    match input[0].inner {
        Primary::Number(n) => Ok(Primary::Number(n.cos())),
        _ => Err(InterpretorError::FobbiddenTernay),
    }
}

pub fn abs(input: &[LocatedPrimary], _env: &mut Environment) -> Result<Primary, InterpretorError> {
    match input[0].inner {
        Primary::Number(n) => Ok(Primary::Number(n.abs())),
        _ => Err(InterpretorError::FobbiddenTernay),
    }
}

pub fn init_foreign_functions(interpreter: &mut Interpreter) {
    interpreter.bind_forein("sin", ForeignFunction::new(sin, 1));
    interpreter.bind_forein("cos", ForeignFunction::new(cos, 1));
    interpreter.bind_forein("abs", ForeignFunction::new(abs, 1));
}
