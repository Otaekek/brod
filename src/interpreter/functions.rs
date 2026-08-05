use crate::{
    diagnostic::Span,
    interpreter::{
        environment::Environment,
        interpreter::{Instance, InstanceId, Interpreter, InterpretorError, LocatedRTObject, RTObject},
    },
    parser::parser::{AST, ClassDefinition, Primary, StatementID},
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
    pub name: String,
    pub arguments: Vec<String>,
    pub statement: StatementID,
}

impl ResidentFunction {
    pub fn call(
        &self,
        ast: &AST,
        interpreter: &mut Interpreter,
        arguments: &[RTObject],
        call_span: Span,
        self_id: Option<InstanceId>,
    ) -> Result<RTObject, InterpretorError> {
        if arguments.len() != self.arguments.len() {
            return Err(InterpretorError::InvalidSignature {
                name: self.name.clone(),
                expected: self.arguments.len(),
                actual: arguments.len(),
                span: Some(call_span),
            });
        }

        interpreter.environment.push();
        if let Some(id) = self_id {
            interpreter
                .environment
                .add("self".to_string(), RTObject::Class(id));
        }

        for (name, value) in self.arguments.iter().zip(arguments.iter()) {
            interpreter.environment.add(name.clone(), value.clone());
        }
        let ret = interpreter.eval_statement(ast, &ast.statement_arena[self.statement]);
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
        call_span: Span,
    ) -> Result<RTObject, InterpretorError> {
        let mut env = Environment::new();
        for n in &self.class.fields {
            env.add(n.clone(), RTObject::Primary(Primary::Nil));
        }
        for f in &self.class.functions {
            env.functions.insert(
                f.name.clone(),
                std::rc::Rc::new(Function::Resident(ResidentFunction {
                    name: f.name.clone(),
                    arguments: f.arguments.clone(),
                    statement: f.statement,
                })),
            );
        }

        let id = interpreter.instance_arena.alloc(Instance {
            name: self.class.name.clone(),
            env,
        });
        if arguments.len() != self.class.constructor.arguments.len() {
            return Err(InterpretorError::InvalidSignature {
                name: self.class.name.clone(),
                expected: self.class.constructor.arguments.len(),
                actual: arguments.len(),
                span: Some(call_span),
            });
        }
        interpreter.environment.push();
        interpreter
            .environment
            .add("self".to_string(), RTObject::Class(id));

        for (name, value) in self
            .class
            .constructor
            .arguments
            .iter()
            .zip(arguments.iter())
        {
            interpreter.environment.add(name.clone(), value.clone());
        }
        interpreter.eval_statement(ast, &ast.statement_arena[self.class.constructor.statement])?;
        interpreter.environment.pop();
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
        call_span: Span,
        self_id: Option<InstanceId>,
    ) -> Result<LocatedRTObject, InterpretorError> {
        let ret = {
            match self {
                Function::Foreign(foreign_function) => foreign_function
                    .call(arguments, &mut interpreter.environment)
                    .map(|x| x.located(call_span)),
                Function::Resident(resident_function) => resident_function
                    .call(ast, interpreter, arguments, call_span, self_id)
                    .map(|x| x.located(call_span)),
                Function::Constructor(constructor_function) => constructor_function
                    .call(ast, interpreter, arguments, call_span)
                    .map(|x| x.located(call_span)),
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
