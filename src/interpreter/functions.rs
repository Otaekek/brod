use crate::{
    interpreter::interpreter::{
        Environment, Instance, Interpreter, InterpretorError, LocatedRTObject, RTObject,
    },
    parser::parser::{AST, ClassDefinition, Primary, Statement},
};
#[derive(Clone, Debug)]
pub struct ForeignFunction {
    function: fn(&[RTObject], &mut Environment) -> Result<RTObject, InterpretorError>,
    _num_arguments: usize,
}

impl ForeignFunction {
    pub fn new(
        function: fn(&[RTObject], &mut Environment) -> Result<RTObject, InterpretorError>,
        num_arguments: usize,
    ) -> Self {
        Self {
            function,
            _num_arguments: num_arguments,
        }
    }
    pub fn call(
        &self,
        arguments: &[RTObject],
        env: &mut Environment,
    ) -> Result<RTObject, InterpretorError> {
        (self.function)(arguments, env)
    }
}

#[derive(Clone, Debug)]
pub struct ResidentFunction {
    pub arguments: Vec<String>,
    pub statement: Statement,
}

impl ResidentFunction {
    pub fn call(
        &self,
        ast: &AST,
        interpreter: &mut Interpreter,
        arguments: &[RTObject],
    ) -> Result<RTObject, InterpretorError> {
        if arguments.len() != self.arguments.len() {
            return Err(InterpretorError::InvalidSignature);
        }

        interpreter.environment.push();

        for (name, value) in self.arguments.iter().zip(arguments.iter()) {
            interpreter.environment.add(name.clone(), value.clone());
        }
        let ret = interpreter.eval_statement(ast, &self.statement);
        interpreter.environment.pop();
        ret
    }
}

#[derive(Clone, Debug)]
pub struct ConstructorFunction {
    pub class: ClassDefinition,
}
impl ConstructorFunction {
    pub fn call(
        &self,
        ast: &AST,
        interpreter: &mut Interpreter,
        arguments: &[RTObject],
    ) -> Result<RTObject, InterpretorError> {
        let mut env = Environment::new();
        for n in &self.class.fields {
            env.add(n.clone(), RTObject::Primary(Primary::Nil));
        }
        for f in &self.class.functions {
            env.functions.insert(
                f.name.clone(),
                std::rc::Rc::new(Function::Resident(ResidentFunction {
                    arguments: f.arguments.clone(),
                    statement: f.statement.clone(),
                })),
            );
        }

        let id = interpreter.instance_arena.alloc(Instance {
            name: self.class.name.clone(),
            env,
        });
        interpreter.my_self.push(id);
        if arguments.len() != self.class.constructor.arguments.len() {
            return Err(InterpretorError::InvalidSignature);
        }
        interpreter.environment.push();

        for (name, value) in self
            .class
            .constructor
            .arguments
            .iter()
            .zip(arguments.iter())
        {
            interpreter.environment.add(name.clone(), value.clone());
        }
        interpreter.eval_statement(ast, &self.class.constructor.statement)?;
        interpreter.environment.pop();
        interpreter.my_self.pop();
        Ok(RTObject::Class(id))
    }
}

#[derive(Clone, Debug)]
pub enum Function {
    Foreign(ForeignFunction),
    Resident(ResidentFunction),
    Constructor(ConstructorFunction),
}

impl Function {
    pub fn call(
        &self,
        ast: &AST,
        interpreter: &mut Interpreter,
        arguments: &[RTObject],
    ) -> Result<LocatedRTObject, InterpretorError> {
        // TODO: Locate callee
        let ret = {
            match self {
                Function::Foreign(foreign_function) => foreign_function
                    .call(arguments, &mut interpreter.environment)
                    .map(|x| x.located(0, 0)),
                Function::Resident(resident_function) => resident_function
                    .call(ast, interpreter, arguments)
                    .map(|x| x.located(0, 0)),
                Function::Constructor(constructor_function) => constructor_function
                    .call(ast, interpreter, arguments)
                    .map(|x| x.located(0, 0)),
            }
        };

        match ret {
            Err(err) => match err {
                InterpretorError::Return(ret) => Ok(ret),
                x => Err(x),
            },
            x => x,
        }
    }
}
