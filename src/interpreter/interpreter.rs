use crate::interpreter::functions::ConstructorFunction;
use crate::interpreter::functions::ForeignFunction;
use crate::interpreter::functions::Function;
use crate::interpreter::functions::ResidentFunction;
use crate::{
    interpreter::foreign_function::init_foreign_functions,
    parser::parser::{
        AST, ClassDefinition, Declaration, ExprID, FunctionCall, FunctionDefinition,
        LocatedPrimary, Operator, Primary, Statement, TokenID, Unary,
    },
};
use std::{collections::HashMap, fmt::Display};
impl LocatedPrimary {
    pub fn to_object(self) -> LocatedRTObject {
        RTObject::Primary(self.inner.clone()).locate_with(&self)
    }
}
impl Primary {
    pub fn to_object(self) -> RTObject {
        RTObject::Primary(self)
    }
}
#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub env: Environment,
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum RTObject {
    Primary(Primary),
    Class(Instance),
}

impl Display for RTObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RTObject::Primary(primary) => write!(f, "{}", primary),
            RTObject::Class(instance) => write!(f, "{}", instance),
        }
    }
}
impl Default for RTObject {
    fn default() -> Self {
        Self::Primary(Primary::Nil)
    }
}
#[derive(Debug, Clone)]
pub struct LocatedRTObject {
    inner: RTObject,
    // Inclusive range
    pub token_start: TokenID,
    pub token_end: TokenID,
}

impl LocatedRTObject {
    pub fn get_primary(&self) -> Result<&Primary, InterpretorError> {
        match &self.inner {
            RTObject::Primary(primary) => Ok(&primary),
            RTObject::Class(_) => Err(InterpretorError::FobbiddenTernay),
        }
    }
    pub fn get_located_primary(self) -> Result<LocatedPrimary, InterpretorError> {
        match self.inner {
            RTObject::Primary(primary) => Ok(primary.located(self.token_start, self.token_end)),
            RTObject::Class(_) => Err(InterpretorError::FobbiddenTernay),
        }
    }
}

impl RTObject {
    pub fn get_primary(&self) -> Result<&Primary, InterpretorError> {
        match &self {
            RTObject::Primary(primary) => Ok(&primary),
            RTObject::Class(_) => Err(InterpretorError::FobbiddenTernay),
        }
    }
    pub fn located(self, token_start: usize, token_end: usize) -> LocatedRTObject {
        LocatedRTObject {
            inner: self,
            token_start,
            token_end,
        }
    }
    pub fn locate_with(self, primary: &LocatedPrimary) -> LocatedRTObject {
        LocatedRTObject {
            inner: self,
            token_start: primary.token_start,
            token_end: primary.token_end,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Environment {
    stack: Vec<HashMap<String, RTObject>>,
    pub functions: HashMap<String, Function>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            // Start with global scope
            stack: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, value: RTObject) {
        let last = self.stack.len() - 1;
        self.stack[last].insert(name, value);
    }

    pub fn _check(&mut self, name: &String) -> bool {
        let last = self.stack.len();
        for i in 0..self.stack.len() {
            let r = self.stack[last - i].get(name);
            if r.is_some() {
                return true;
            }
        }
        false
    }

    pub fn assign(&mut self, name: &String, value: &RTObject) -> Result<(), InterpretorError> {
        let last = self.stack.len();
        for i in 0..self.stack.len() {
            let r = self.stack[last - i - 1].get(name);
            if r.is_some() {
                let m = self.stack[last - i - 1].get_mut(name).unwrap();
                *m = value.clone();
                return Ok(());
            }
        }
        // TODO LOCATE
        Err(InterpretorError::UnDeclaredIdentifier(
            Primary::Nil.located(0, 0).to_object(),
        ))
    }

    pub fn get(
        &self,
        name: &String,
        ident: &LocatedRTObject,
    ) -> Result<RTObject, InterpretorError> {
        let last = self.stack.len();
        for i in 0..self.stack.len() {
            let r = self.stack[last - i - 1].get(name);
            if r.is_some() {
                return Ok(r.unwrap().clone());
            }
        }

        Err(InterpretorError::UnDeclaredIdentifier(ident.clone()))
    }
    pub fn push(&mut self) {
        self.stack.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.stack.pop();
    }
}

#[derive(Debug)]
pub struct Interpreter {
    pub environment: Environment,
    pub my_self: Vec<Instance>,
    head: usize,
}

#[derive(Debug, Clone)]
pub enum InterpretorError {
    UnexpectedType(LocatedRTObject, String),
    ForbiddenUnaryOperation(Unary, LocatedRTObject),
    ForbiddenBinaryOperation(Operator, LocatedRTObject, LocatedRTObject),
    FobbiddenTernay,
    ForbidenBreak,
    NotCallable(LocatedRTObject),
    UnknownFunction(String),
    InvalidSignature,
    ForbidenContinue,
    Ungetable(LocatedRTObject),
    Return(LocatedRTObject),
    UnDeclaredIdentifier(LocatedRTObject),
}
impl InterpretorError {
    fn format_error(
        &self,
        _ast: &AST,
        source: &str,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            InterpretorError::ForbiddenUnaryOperation(unary, terminal) => {
                write!(
                    f,
                    "Forbiden operator: {} on type {:#?}",
                    unary, terminal.inner
                )
            }
            InterpretorError::ForbiddenBinaryOperation(operator, terminal, terminal1) => write!(
                f,
                "Forbiden operator: {} on type {:#?} and {:#?} in file {} from {} to {}",
                operator,
                terminal.inner,
                terminal1.inner,
                source,
                terminal.token_start,
                terminal1.token_end
            ),
            InterpretorError::FobbiddenTernay => write!(
                f,
                "{}",
                "left hand side of a ternary should be a boolean or a number"
            ),
            InterpretorError::UnDeclaredIdentifier(v) => {
                write!(f, "Undeclared Variable {:#?}", v.inner)
            }
            InterpretorError::UnexpectedType(located_primary, expected) => {
                write!(
                    f,
                    "Invalid type: {:#?}, Expected : {}",
                    located_primary.inner, expected
                )
            }
            InterpretorError::ForbidenBreak => write!(f, "{}", "Break should be in while loop"),
            InterpretorError::ForbidenContinue => {
                write!(f, "{}", "Continue should be in while loop")
            }
            InterpretorError::NotCallable(located_primary) => {
                write!(f, "{:#?} is not callable", located_primary)
            }
            InterpretorError::InvalidSignature => {
                write!(f, "invalid number of arguments")
            }
            InterpretorError::UnknownFunction(name) => {
                write!(f, "unknown function {}", name)
            }
            InterpretorError::Return(_) => {
                write!(f, "{}", "invalid return: return should be in function")
            }
            InterpretorError::Ungetable(located_rtobject) => {
                write!(f, "{:#?} is not getable", located_rtobject)
            }
        }
    }
    pub fn get_formated_error(&self, ast: &AST, source: &str) -> String {
        format!(
            "{}",
            ErrorDisplay {
                ast,
                error: self,
                source
            }
        )
    }
}
struct ErrorDisplay<'a> {
    error: &'a InterpretorError,
    source: &'a str,
    ast: &'a AST,
}

impl<'a> std::fmt::Display for ErrorDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.format_error(self.ast, self.source, f)
    }
}

impl Interpreter {
    pub fn declare_function(&mut self, definition: FunctionDefinition) {
        self.environment.functions.insert(
            definition.name,
            Function::Resident(ResidentFunction {
                arguments: definition.arguments,
                statement: definition.statement,
            }),
        );
    }

    pub fn declare_constructor(&mut self, class: ClassDefinition) {
        self.environment.functions.insert(
            class.constructor.name.clone(),
            Function::Constructor(ConstructorFunction { class: class }),
        );
    }
    pub fn bind_forein(&mut self, name: &str, function: ForeignFunction) {
        self.environment
            .functions
            .insert(name.to_string(), Function::Foreign(function));
    }

    fn function_call(
        &mut self,
        ast: &AST,
        function_call: &FunctionCall,
    ) -> Result<LocatedRTObject, InterpretorError> {
        let arguments = function_call
            .arguments
            .iter()
            .map(|expr_id| Ok(self.eval(ast, *expr_id)?.inner))
            .collect::<Result<Vec<_>, _>>()?;

        let function = match &ast.expr_arena[function_call.func] {
            crate::parser::parser::Expr::Terminal(located_primary) => {
                let callee = match &located_primary.inner {
                    Primary::Identifier(s) => s,
                    _ => return Err(InterpretorError::FobbiddenTernay),
                };
                let function = self.environment.functions.get(callee).cloned();
                function
            }
            crate::parser::parser::Expr::Get(expr_id, name) => {
                let expr = self.eval(ast, *expr_id)?;
                match expr.inner {
                    RTObject::Class(instance) => instance.env.functions.get(name).cloned(),
                    _ => unreachable!(),
                }
            }
            _ => return Err(InterpretorError::FobbiddenTernay),
        };
        match function {
            Some(function) => function.call(ast, self, &arguments),
            None => Err(InterpretorError::UnknownFunction("callee".to_string())),
        }
    }
    pub fn new() -> Self {
        let mut ret = Self {
            environment: Environment::new(),
            head: 0,
            my_self: vec![],
        };
        init_foreign_functions(&mut ret);
        ret
    }

    pub fn eval_unary(
        &mut self,
        ast: &AST,
        unary: Unary,
    ) -> Result<LocatedRTObject, InterpretorError> {
        match unary {
            crate::parser::parser::Unary::Not(v) => {
                let last = self.eval(ast, v)?;
                match last.get_primary()? {
                    Primary::Boolean(v) => Ok(Primary::Boolean(!v)
                        .located(last.token_start, last.token_end)
                        .to_object()),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
            crate::parser::parser::Unary::Minus(v) => {
                let last = self.eval(ast, v)?;
                match &last.get_primary()? {
                    Primary::Number(v) => Ok(Primary::Number(-*v)
                        .located(last.token_start, last.token_end)
                        .to_object()),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
        }
    }
    pub fn eval_binary(
        &mut self,
        operator: Operator,
        left: LocatedPrimary,
        right: LocatedPrimary,
    ) -> Result<LocatedRTObject, InterpretorError> {
        use crate::parser::parser::Operator::*;
        let token_start = left.token_start;
        let token_end = right.token_end;
        let ret = match (&left.inner, &right.inner) {
            (Primary::Number(left_n), Primary::Number(right_n)) => match operator {
                Equal => Ok(Primary::Boolean(left_n == right_n)),
                NotEqual => Ok(Primary::Boolean(left_n != right_n)),
                Lesser => Ok(Primary::Boolean(left_n < right_n)),
                LesserEqual => Ok(Primary::Boolean(left_n <= right_n)),
                Greater => Ok(Primary::Boolean(left_n > right_n)),
                GreaterEqual => Ok(Primary::Boolean(left_n >= right_n)),
                Plus => Ok(Primary::Number(left_n + right_n)),
                Minus => Ok(Primary::Number(left_n - right_n)),
                Slash => Ok(Primary::Number(left_n / right_n)),
                Star => Ok(Primary::Number(left_n * right_n)),
            },
            (Primary::String(left_s), Primary::String(right_s)) => match operator {
                Equal => Ok(Primary::Boolean(left_s == right_s)),
                NotEqual => Ok(Primary::Boolean(left_s != right_s)),
                Plus => Ok(Primary::String(left_s.to_owned() + right_s)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator,
                    left.to_object(),
                    right.to_object(),
                )),
            },
            (Primary::Boolean(left_bool), Primary::Boolean(right_bool)) => match operator {
                Equal => Ok(Primary::Boolean(left_bool == right_bool)),
                NotEqual => Ok(Primary::Boolean(left_bool != right_bool)),
                Lesser => Ok(Primary::Boolean(left_bool < right_bool)),
                LesserEqual => Ok(Primary::Boolean(left_bool <= right_bool)),
                Greater => Ok(Primary::Boolean(left_bool > right_bool)),
                GreaterEqual => Ok(Primary::Boolean(left_bool >= right_bool)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator,
                    left.to_object(),
                    right.to_object(),
                )),
            },
            (Primary::Nil, Primary::Nil) => match operator {
                Equal => Ok(Primary::Boolean(true)),
                NotEqual => Ok(Primary::Boolean(false)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator,
                    left.to_object(),
                    right.to_object(),
                )),
            },
            (Primary::String(s), Primary::Number(n)) => {
                if operator == Plus {
                    Ok(Primary::String(s.clone() + n.to_string().as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator,
                        left.to_object(),
                        right.to_object(),
                    ))
                }
            }
            (Primary::Number(n), Primary::String(s)) => {
                if operator == Plus {
                    Ok(Primary::String(n.to_string() + s.as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator,
                        left.to_object(),
                        right.to_object(),
                    ))
                }
            }
            _ => Err(InterpretorError::ForbiddenBinaryOperation(
                operator,
                left.to_object(),
                right.to_object(),
            )),
        };
        Ok(ret?.located(token_start, token_end).to_object())
    }

    pub fn eval_declaration(
        &mut self,
        ast: &AST,
        input: Declaration,
    ) -> Result<RTObject, InterpretorError> {
        match input {
            Declaration::Statement(statement) => self.eval_statement(ast, statement),
            Declaration::VarDecl(ident, expr_id) => {
                let value = self.eval(ast, expr_id)?;
                self.environment.add(ident, value.inner.clone());
                Ok(value.inner)
            }
            Declaration::Empty => Ok(RTObject::Primary(Primary::Nil)),
            Declaration::Comment(_) => Ok(RTObject::Primary(Primary::Nil)),
            Declaration::FunctionDefinition(function_definition) => {
                self.declare_function(function_definition);
                Ok(RTObject::Primary(Primary::Nil))
            }
            Declaration::ClassDefinition(class) => {
                self.declare_constructor(class);
                Ok(RTObject::Primary(Primary::Nil))
            }
        }
    }
    pub fn eval_statement(
        &mut self,
        ast: &AST,
        input: Statement,
    ) -> Result<RTObject, InterpretorError> {
        match input {
            Statement::ExprStatement(expr_id) => Ok(self.eval(ast, expr_id)?.inner),
            Statement::PrintStatement(items) => {
                for x in items {
                    let r = self.eval(ast, x)?;
                    match &r.get_primary()? {
                        Primary::Identifier(s) => {
                            let v = self
                                .environment
                                .get(s, &r.clone().get_located_primary()?.to_object())?;
                            println!("{:#?}", v);
                        }
                        x => println!("{x}"),
                    }
                }
                Ok(RTObject::Primary(Primary::Nil))
            }
            Statement::Block(declarations) => {
                let mut last = Primary::Nil.to_object();
                self.environment.push();
                for declaration in declarations {
                    match declaration {
                        Declaration::Comment(_) => continue,
                        _ => (),
                    };
                    let e_last = self.eval_declaration(ast, declaration);
                    match e_last {
                        Ok(r) => last = r,
                        Err(r) => {
                            self.environment.pop();
                            return Err(r);
                        }
                    }
                }
                self.environment.pop();
                Ok(last)
            }
            Statement::IfStatement(ifelses, else_stmt) => {
                for (expr, stmt) in ifelses.iter() {
                    let expr = self.eval(ast, *expr)?;
                    match expr.get_primary()? {
                        Primary::Boolean(true) => return self.eval_statement(ast, stmt.clone()),
                        Primary::Boolean(false) => {}

                        _ => {
                            return Err(InterpretorError::UnexpectedType(
                                expr,
                                "Boolean".to_string(),
                            ));
                        }
                    };
                }
                if let Some(stmt) = else_stmt {
                    let stmt = ast.statement_arena[stmt].clone();
                    return self.eval_statement(ast, stmt);
                }

                Ok(RTObject::default())
            }
            Statement::Whileloop(expr_id, stmt_id) => {
                let stmt = ast.statement_arena[stmt_id].clone();
                loop {
                    let r = self.eval(ast, expr_id)?;
                    match r.get_primary()? {
                        Primary::Boolean(true) => {}
                        Primary::Boolean(false) => break,
                        _ => {
                            return Err(InterpretorError::UnexpectedType(r, "Boolean".to_string()));
                        }
                    };
                    match self.eval_statement(ast, stmt.clone()) {
                        Err(InterpretorError::ForbidenBreak) => break,
                        Err(InterpretorError::ForbidenContinue) => continue,
                        _ => {}
                    };
                }
                Ok(RTObject::default())
            }
            Statement::Break(_) => Err(InterpretorError::ForbidenBreak),
            Statement::Continue(_) => Err(InterpretorError::ForbidenContinue),
            Statement::Return(item) => {
                let item = if let Some(item) = item {
                    self.eval(ast, item)?
                } else {
                    RTObject::default().located(0, 0)
                };

                Err(InterpretorError::Return(item))
            }
        }
    }
    pub fn eval(&mut self, ast: &AST, root: ExprID) -> Result<LocatedRTObject, InterpretorError> {
        match &ast.expr_arena[root] {
            crate::parser::parser::Expr::Terminal(terminal) => match &terminal.inner {
                Primary::Identifier(v) => Ok(self
                    .environment
                    .get(v, &terminal.clone().to_object())?
                    .locate_with(terminal)),
                _ => return Ok(terminal.clone().to_object()),
            },
            crate::parser::parser::Expr::Unary(unary) => self.eval_unary(ast, *unary),
            crate::parser::parser::Expr::Binary(binary) => {
                let left = self.eval(ast, binary.left)?;
                let right = self.eval(ast, binary.right)?;
                self.eval_binary(
                    binary.operator,
                    left.get_located_primary()?,
                    right.get_located_primary()?,
                )
            }
            crate::parser::parser::Expr::Ternary(ternary) => {
                let left = self.eval(ast, ternary.left)?;
                match left.get_primary()? {
                    Primary::Boolean(v) => match v {
                        true => self.eval(ast, ternary.middle),
                        false => self.eval(ast, ternary.right),
                    },
                    Primary::Number(v) => match v > &0.0 {
                        true => self.eval(ast, ternary.middle),
                        false => self.eval(ast, ternary.right),
                    },
                    _ => Err(InterpretorError::FobbiddenTernay),
                }
            }
            crate::parser::parser::Expr::LogicalAnd(logical_and) => {
                let left = self.eval(ast, logical_and.left)?;
                match &left.get_primary()? {
                    Primary::Boolean(false) => return Ok(left),
                    Primary::Boolean(true) => return self.eval(ast, logical_and.right),
                    _ => return Err(InterpretorError::FobbiddenTernay),
                }
            }
            crate::parser::parser::Expr::LogicalOr(logical_or) => {
                let left = self.eval(ast, logical_or.left)?;
                match &left.get_primary()? {
                    Primary::Boolean(true) => return Ok(left),
                    Primary::Boolean(false) => return self.eval(ast, logical_or.right),
                    _ => return Err(InterpretorError::FobbiddenTernay),
                }
            }
            crate::parser::parser::Expr::Assignment(s, expr) => {
                let expr = self.eval(ast, *expr)?;
                self.environment.assign(s, &expr.inner)?;
                Ok(expr)
            }
            crate::parser::parser::Expr::FunctionCall(function_call) => {
                self.function_call(ast, function_call)
            }
            crate::parser::parser::Expr::Get(expr_id, name) => self.eval_get(ast, *expr_id, name),
        }
    }
    pub fn eval_get(
        &mut self,
        ast: &AST,
        expr_id: ExprID,
        name: &String,
    ) -> Result<LocatedRTObject, InterpretorError> {
        let expr = self.eval(ast, expr_id)?;
        match expr.inner.clone() {
            RTObject::Primary(primary) => match primary {
                Primary::MySelf => {
                    if self.my_self.is_empty() {
                        return Err(InterpretorError::FobbiddenTernay);
                    } else {
                        return Ok(LocatedRTObject {
                            inner: RTObject::Class(self.my_self.last().unwrap().clone()),
                            token_start: 0,
                            token_end: 0,
                        });
                    }
                }
                Primary::Identifier(_) => unreachable!("Should be evaluated already"),
                _ => Err(InterpretorError::Ungetable(expr)),
            },
            RTObject::Class(instance) => {
                // TODO: Locate
                instance
                    .env
                    .get(name, &expr.inner.located(0, 0))
                    .map(|x| x.located(0, 0))
            }
        }
    }
}

pub fn eval(ast: AST, interpreter: &mut Interpreter) -> Result<RTObject, InterpretorError> {
    let mut last = RTObject::default();
    while interpreter.head < ast.roots.len() {
        let root = &ast.roots[interpreter.head];
        interpreter.head += 1;
        let ret = interpreter.eval_declaration(&ast, root.clone())?;
        last = ret;
    }
    Ok(last)
}
