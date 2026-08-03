use crate::arena::{Arena, Id};
use crate::diagnostic::{Diagnostic, Span, Stage};
use crate::interpreter::functions::ConstructorFunction;
use crate::interpreter::functions::ForeignFunction;
use crate::interpreter::functions::Function;
use crate::interpreter::functions::ResidentFunction;
use crate::{
    interpreter::foreign_function::init_foreign_functions,
    parser::parser::{
        AST, ClassDefinition, Declaration, ExprID, FunctionCall, FunctionDefinition,
        LocatedPrimary, Operator, Primary, Statement, Unary,
    },
};
use std::{collections::HashMap, fmt::Display, rc::Rc};
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

pub type InstanceId = Id<Instance>;
pub type InstanceArena = Arena<Instance>;

#[derive(Debug, Clone)]
pub enum RTObject {
    Primary(Primary),
    Class(InstanceId),
}

impl Display for RTObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RTObject::Primary(primary) => write!(f, "{}", primary),
            RTObject::Class(id) => write!(f, "<instance #{}>", id),
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
    pub span: Span,
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
            RTObject::Primary(primary) => Ok(primary.located(self.span)),
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
    pub fn located(self, span: Span) -> LocatedRTObject {
        LocatedRTObject { inner: self, span }
    }
    pub fn locate_with(self, primary: &LocatedPrimary) -> LocatedRTObject {
        LocatedRTObject {
            inner: self,
            span: primary.span,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Environment {
    stack: Vec<HashMap<String, RTObject>>,
    pub functions: HashMap<String, Rc<Function>>,
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
        Err(InterpretorError::UnDeclaredIdentifier(Box::new(
            Primary::Identifier(name.clone())
                .located(Span::point(0))
                .to_object(),
        )))
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

        Err(InterpretorError::UnDeclaredIdentifier(Box::new(
            ident.clone(),
        )))
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
    pub instance_arena: InstanceArena,
    pub my_self: Vec<InstanceId>,
    head: usize,
}

#[derive(Debug, Clone)]
pub enum InterpretorError {
    UnexpectedType(Box<LocatedRTObject>, String),
    ForbiddenUnaryOperation(Unary, Box<LocatedRTObject>),
    ForbiddenBinaryOperation(Operator, Box<LocatedRTObject>, Box<LocatedRTObject>),
    FobbiddenTernay,
    ForbidenBreak,
    NotCallable(Box<LocatedRTObject>),
    UnknownFunction(String, Option<Span>),
    InvalidSignature,
    ForbidenContinue,
    Ungetable(Box<LocatedRTObject>),
    Return(LocatedRTObject),
    UnDeclaredIdentifier(Box<LocatedRTObject>),
}
impl Diagnostic for InterpretorError {
    fn stage(&self) -> Stage {
        Stage::Runtime
    }
    fn span(&self) -> Option<Span> {
        match self {
            InterpretorError::UnexpectedType(v, _)
            | InterpretorError::ForbiddenUnaryOperation(_, v)
            | InterpretorError::NotCallable(v)
            | InterpretorError::Ungetable(v)
            | InterpretorError::UnDeclaredIdentifier(v) => Some(v.span),
            InterpretorError::ForbiddenBinaryOperation(_, left, right) => Some(Span::new(
                left.span.start.min(right.span.start),
                left.span.end.max(right.span.end),
            )),
            InterpretorError::UnknownFunction(_, span) => *span,
            InterpretorError::FobbiddenTernay
            | InterpretorError::ForbidenBreak
            | InterpretorError::ForbidenContinue
            | InterpretorError::InvalidSignature
            | InterpretorError::Return(_) => None,
        }
    }
    fn message(&self) -> String {
        match self {
            InterpretorError::ForbiddenUnaryOperation(unary, terminal) => {
                format!("Forbidden operator: {} on {}", unary, terminal.inner)
            }
            InterpretorError::ForbiddenBinaryOperation(operator, terminal, terminal1) => format!(
                "Forbidden operator: {} on {} and {}",
                operator, terminal.inner, terminal1.inner
            ),
            InterpretorError::FobbiddenTernay => {
                "left hand side of a ternary should be a boolean or a number".to_string()
            }
            InterpretorError::UnDeclaredIdentifier(v) => {
                let name = match &v.inner {
                    RTObject::Primary(Primary::Identifier(s)) => s.clone(),
                    other => other.to_string(),
                };
                format!("Undeclared variable: {}", name)
            }
            InterpretorError::UnexpectedType(located_primary, expected) => {
                format!(
                    "Invalid type: {}, Expected : {}",
                    located_primary.inner, expected
                )
            }
            InterpretorError::ForbidenBreak => "Break should be in while loop".to_string(),
            InterpretorError::ForbidenContinue => "Continue should be in while loop".to_string(),
            InterpretorError::NotCallable(located_primary) => {
                format!("{} is not callable", located_primary.inner)
            }
            InterpretorError::InvalidSignature => "invalid number of arguments".to_string(),
            InterpretorError::UnknownFunction(name, _) => format!("unknown function: {}", name),
            InterpretorError::Return(_) => {
                "invalid return: return should be in function".to_string()
            }
            InterpretorError::Ungetable(located_rtobject) => {
                format!("{} is not getable", located_rtobject.inner)
            }
        }
    }
}

impl Interpreter {
    pub fn declare_function(&mut self, definition: FunctionDefinition) {
        self.environment.functions.insert(
            definition.name,
            Rc::new(Function::Resident(ResidentFunction {
                arguments: definition.arguments,
                statement: definition.statement,
            })),
        );
    }

    pub fn declare_constructor(&mut self, class: ClassDefinition) {
        self.environment.functions.insert(
            class.constructor.name.clone(),
            Rc::new(Function::Constructor(ConstructorFunction { class: class })),
        );
    }
    pub fn bind_forein(&mut self, name: &str, function: ForeignFunction) {
        self.environment
            .functions
            .insert(name.to_string(), Rc::new(Function::Foreign(function)));
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

        match &ast.expr_arena[function_call.func] {
            crate::parser::parser::Expr::Terminal(located_primary) => {
                let callee = match &located_primary.inner {
                    Primary::Identifier(s) => s,
                    _ => {
                        return Err(InterpretorError::NotCallable(Box::new(
                            located_primary.clone().to_object(),
                        )));
                    }
                };
                match self.environment.functions.get(callee).cloned() {
                    Some(function) => function.call(ast, self, &arguments),
                    None => Err(InterpretorError::UnknownFunction(
                        callee.clone(),
                        Some(located_primary.span),
                    )),
                }
            }
            crate::parser::parser::Expr::Get(expr_id, name) => {
                let expr = self.eval(ast, *expr_id)?;
                match expr.inner {
                    RTObject::Class(id) => {
                        match self.instance_arena[id].env.functions.get(name).cloned() {
                            Some(function) => function.call(ast, self, &arguments),
                            None => Err(InterpretorError::UnknownFunction(name.clone(), None)),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                let evaluated = self.eval(ast, function_call.func)?;
                Err(InterpretorError::NotCallable(Box::new(evaluated)))
            }
        }
    }
    pub fn new() -> Self {
        let mut ret = Self {
            environment: Environment::new(),
            instance_arena: InstanceArena::default(),
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
                        .located(last.span)
                        .to_object()),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(
                        unary,
                        Box::new(last),
                    )),
                }
            }
            crate::parser::parser::Unary::Minus(v) => {
                let last = self.eval(ast, v)?;
                match &last.get_primary()? {
                    Primary::Number(v) => Ok(Primary::Number(-*v)
                        .located(last.span)
                        .to_object()),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(
                        unary,
                        Box::new(last),
                    )),
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
        let span = Span::new(
            left.span.start.min(right.span.start),
            left.span.end.max(right.span.end),
        );
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
                    Box::new(left.to_object()),
                    Box::new(right.to_object()),
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
                    Box::new(left.to_object()),
                    Box::new(right.to_object()),
                )),
            },
            (Primary::Nil, Primary::Nil) => match operator {
                Equal => Ok(Primary::Boolean(true)),
                NotEqual => Ok(Primary::Boolean(false)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator,
                    Box::new(left.to_object()),
                    Box::new(right.to_object()),
                )),
            },
            (Primary::String(s), Primary::Number(n)) => {
                if operator == Plus {
                    Ok(Primary::String(s.clone() + n.to_string().as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator,
                        Box::new(left.to_object()),
                        Box::new(right.to_object()),
                    ))
                }
            }
            (Primary::Number(n), Primary::String(s)) => {
                if operator == Plus {
                    Ok(Primary::String(n.to_string() + s.as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator,
                        Box::new(left.to_object()),
                        Box::new(right.to_object()),
                    ))
                }
            }
            _ => Err(InterpretorError::ForbiddenBinaryOperation(
                operator,
                Box::new(left.to_object()),
                Box::new(right.to_object()),
            )),
        };
        Ok(ret?.located(span).to_object())
    }

    pub fn eval_declaration(
        &mut self,
        ast: &AST,
        input: &Declaration,
    ) -> Result<RTObject, InterpretorError> {
        match input {
            Declaration::Statement(statement) => self.eval_statement(ast, statement),
            Declaration::VarDecl(ident, expr_id) => {
                let value = self.eval(ast, *expr_id)?;
                self.environment.add(ident.clone(), value.inner.clone());
                Ok(value.inner)
            }
            Declaration::Empty => Ok(RTObject::Primary(Primary::Nil)),
            Declaration::Comment(_) => Ok(RTObject::Primary(Primary::Nil)),
            Declaration::FunctionDefinition(function_definition) => {
                self.declare_function(function_definition.clone());
                Ok(RTObject::Primary(Primary::Nil))
            }
            Declaration::ClassDefinition(class) => {
                self.declare_constructor(class.clone());
                Ok(RTObject::Primary(Primary::Nil))
            }
        }
    }
    pub fn eval_statement(
        &mut self,
        ast: &AST,
        input: &Statement,
    ) -> Result<RTObject, InterpretorError> {
        match input {
            Statement::ExprStatement(expr_id) => Ok(self.eval(ast, *expr_id)?.inner),
            Statement::PrintStatement(items) => {
                for x in items {
                    let r = self.eval(ast, *x)?;
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
                        Primary::Boolean(true) => return self.eval_statement(ast, stmt),
                        Primary::Boolean(false) => {}

                        _ => {
                            return Err(InterpretorError::UnexpectedType(
                                Box::new(expr),
                                "Boolean".to_string(),
                            ));
                        }
                    };
                }
                if let Some(stmt) = else_stmt {
                    return self.eval_statement(ast, &ast.statement_arena[*stmt]);
                }

                Ok(RTObject::default())
            }
            Statement::Whileloop(expr_id, stmt_id) => {
                loop {
                    let r = self.eval(ast, *expr_id)?;
                    match r.get_primary()? {
                        Primary::Boolean(true) => {}
                        Primary::Boolean(false) => break,
                        _ => {
                            return Err(InterpretorError::UnexpectedType(
                                Box::new(r),
                                "Boolean".to_string(),
                            ));
                        }
                    };
                    match self.eval_statement(ast, &ast.statement_arena[*stmt_id]) {
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
                    self.eval(ast, *item)?
                } else {
                    RTObject::default().located(Span::point(0))
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
                            inner: RTObject::Class(*self.my_self.last().unwrap()),
                            span: Span::point(0),
                        });
                    }
                }
                Primary::Identifier(_) => unreachable!("Should be evaluated already"),
                _ => Err(InterpretorError::Ungetable(Box::new(expr))),
            },
            RTObject::Class(id) => {
                // TODO: Locate
                self.instance_arena[id]
                    .env
                    .get(name, &expr.inner.located(Span::point(0)))
                    .map(|x| x.located(Span::point(0)))
            }
        }
    }
}

pub fn eval(ast: AST, interpreter: &mut Interpreter) -> Result<RTObject, InterpretorError> {
    let mut last = RTObject::default();
    while interpreter.head < ast.roots.len() {
        let root = &ast.roots[interpreter.head];
        interpreter.head += 1;
        let ret = interpreter.eval_declaration(&ast, root)?;
        last = ret;
    }
    Ok(last)
}
