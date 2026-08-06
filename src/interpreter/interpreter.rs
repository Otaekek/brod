use crate::arena::{Arena, Id};
use crate::diagnostic::{Diagnostic, Span, Stage};
use crate::interpreter::environment::Environment;
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
use std::{fmt::Display, rc::Rc};
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
            RTObject::Class(_) => Err(InterpretorError::UnexpectedType(
                self.inner.clone(),
                "a primitive value".to_string(),
                Some(self.span),
            )),
        }
    }
    pub fn get_located_primary(self) -> Result<LocatedPrimary, InterpretorError> {
        let span = self.span;
        match self.inner {
            RTObject::Primary(primary) => Ok(primary.located(span)),
            class @ RTObject::Class(_) => Err(InterpretorError::UnexpectedType(
                class,
                "a primitive value".to_string(),
                Some(span),
            )),
        }
    }
}

impl RTObject {
    pub fn get_primary(&self) -> Result<&Primary, InterpretorError> {
        match &self {
            RTObject::Primary(primary) => Ok(&primary),
            RTObject::Class(_) => Err(InterpretorError::UnexpectedType(
                (*self).clone(),
                "a primitive value".to_string(),
                None,
            )),
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

#[derive(Debug)]
pub struct Interpreter {
    pub environment: Environment,
    pub instance_arena: InstanceArena,
    pub stack: Vec<RTObject>,
    pub entering_function_body: bool,
    pub in_method_or_constructor: bool,
    depth: usize,
    scope_stack: Vec<usize>,
    head: usize,
}

const FUNCTION_SCOPE: usize = 0;

#[derive(Debug, Clone)]
pub enum InterpretorError {
    UnexpectedType(RTObject, String, Option<Span>),
    ForbiddenUnaryOperation(Unary, Box<LocatedRTObject>),
    ForbiddenBinaryOperation(Operator, Box<LocatedRTObject>, Box<LocatedRTObject>),
    ForbidenBreak(Span),
    NotCallable(Box<LocatedRTObject>),
    UnknownFunction(String, Option<Span>),
    InvalidSignature {
        name: String,
        expected: usize,
        actual: usize,
        span: Option<Span>,
    },
    ForbidenContinue(Span),
    Ungetable(Box<LocatedRTObject>),
    Return(LocatedRTObject),
    UnDeclaredIdentifier(Box<LocatedRTObject>),
    SelfOutsideConstructor(Option<Span>),
    InvalidAssignmentTarget(Option<Span>),
}
impl Diagnostic for InterpretorError {
    fn stage(&self) -> Stage {
        Stage::Runtime
    }
    fn span(&self) -> Option<Span> {
        match self {
            InterpretorError::ForbiddenUnaryOperation(_, v)
            | InterpretorError::NotCallable(v)
            | InterpretorError::Ungetable(v)
            | InterpretorError::UnDeclaredIdentifier(v) => Some(v.span),
            InterpretorError::ForbiddenBinaryOperation(_, left, right) => {
                Some(left.span.union(right.span))
            }
            InterpretorError::UnknownFunction(_, span) => *span,
            InterpretorError::InvalidSignature { span, .. } => *span,
            InterpretorError::ForbidenBreak(span) | InterpretorError::ForbidenContinue(span) => {
                Some(*span)
            }
            InterpretorError::UnexpectedType(_, _, span) => *span,
            InterpretorError::SelfOutsideConstructor(span) => *span,
            InterpretorError::InvalidAssignmentTarget(span) => *span,
            InterpretorError::Return(item) => Some(item.span),
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
            InterpretorError::SelfOutsideConstructor(_) => {
                "self can only be used inside a constructor".to_string()
            }
            InterpretorError::InvalidAssignmentTarget(_) => {
                "invalid assignment target: expected a variable or a field".to_string()
            }
            InterpretorError::UnDeclaredIdentifier(v) => {
                let name = match &v.inner {
                    RTObject::Primary(Primary::Identifier(s)) => s.clone(),
                    other => other.to_string(),
                };
                format!("Undeclared variable: {}", name)
            }
            InterpretorError::UnexpectedType(value, expected, _) => {
                format!("Invalid type: {}, Expected : {}", value, expected)
            }
            InterpretorError::ForbidenBreak(_) => "Break should be in while loop".to_string(),
            InterpretorError::ForbidenContinue(_) => "Continue should be in while loop".to_string(),
            InterpretorError::NotCallable(located_primary) => {
                format!("{} is not callable", located_primary.inner)
            }
            InterpretorError::InvalidSignature {
                name,
                expected,
                actual,
                ..
            } => format!(
                "{} expects {} argument{}, got {}",
                name,
                expected,
                if *expected == 1 { "" } else { "s" },
                actual
            ),
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
    pub fn declare_function(&mut self, definition: &FunctionDefinition) {
        self.environment.functions.insert(
            definition.name.clone(),
            Rc::new(Function::Resident(ResidentFunction {
                name: definition.name.clone(),
                statement: definition.statement,
                arguments: definition.argument_names(),
            })),
        );
    }

    pub fn declare_constructor(&mut self, class: &ClassDefinition) {
        self.environment.functions.insert(
            class.constructor.name.clone(),
            Rc::new(Function::Constructor(ConstructorFunction {
                class: class.clone(),
            })),
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
        let evaluated_args = function_call
            .arguments
            .iter()
            .map(|expr_id| self.eval(ast, *expr_id))
            .collect::<Result<Vec<_>, _>>()?;

        while self.scope_stack.len() <= FUNCTION_SCOPE {
            self.scope_stack.push(0);
        }
        let saved_scope_base = self.scope_stack[FUNCTION_SCOPE];
        self.scope_stack[FUNCTION_SCOPE] = self.stack.len();

        let args_end = evaluated_args.iter().map(|v| v.span.end).max();
        let arguments: Vec<RTObject> = evaluated_args.into_iter().map(|v| v.inner).collect();

        for arg in &arguments {
            self.stack.push(arg.clone());
        }

        let ret = match &ast.expr_arena[function_call.func] {
            crate::parser::parser::Expr::Terminal(located_primary) => {
                let callee = match &located_primary.inner {
                    Primary::Identifier(s) => s,
                    _ => {
                        return Err(InterpretorError::NotCallable(Box::new(
                            located_primary.clone().to_object(),
                        )));
                    }
                };
                let call_span = located_primary.span.extended_to(args_end);
                match self.environment.functions.get(callee).cloned() {
                    Some(function) => function.call(ast, self, &arguments, call_span, None),
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
                        let call_span = expr.span.extended_to(args_end);
                        match self.instance_arena[id].env.functions.get(name).cloned() {
                            Some(function) => {
                                function.call(ast, self, &arguments, call_span, Some(id))
                            }
                            None => Err(InterpretorError::UnknownFunction(name.clone(), None)),
                        }
                    }
                    _ => Err(InterpretorError::Ungetable(Box::new(expr))),
                }
            }
            _ => {
                let evaluated = self.eval(ast, function_call.func)?;
                Err(InterpretorError::NotCallable(Box::new(evaluated)))
            }
        };

        for _ in &arguments {
            self.stack.pop();
        }
        self.scope_stack[FUNCTION_SCOPE] = saved_scope_base;
        ret
    }
    pub fn new() -> Self {
        let mut ret = Self {
            environment: Environment::new(),
            instance_arena: InstanceArena::default(),
            stack: Vec::with_capacity(4096),
            scope_stack: Vec::with_capacity(4096),
            head: 0,
            entering_function_body: false,
            in_method_or_constructor: false,
            depth: 0,
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
                    Primary::Boolean(v) => Ok(Primary::Boolean(!v).located(last.span).to_object()),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(
                        unary,
                        Box::new(last),
                    )),
                }
            }
            crate::parser::parser::Unary::Minus(v) => {
                let last = self.eval(ast, v)?;
                match &last.get_primary()? {
                    Primary::Number(v) => Ok(Primary::Number(-*v).located(last.span).to_object()),
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
        let span = left.span.union(right.span);
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
            Declaration::Statement(statement) => {
                self.eval_statement(ast, &ast.statement_arena[*statement])
            }
            Declaration::VarDecl(ident, expr_id) => {
                let value = self.eval(ast, *expr_id)?;
                // global scope, or inside a method/constructor (never resolved
                // to a stack slot): use the name-keyed environment.
                if self.depth == 0 || self.in_method_or_constructor {
                    self.environment.add(ident.clone(), value.inner.clone());
                } else {
                    // inner scope: indexed lookup on the stack
                    self.stack.push(value.inner.clone());
                }
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
                            println!("{}", v);
                        }
                        x => println!("{x}"),
                    }
                }
                Ok(RTObject::Primary(Primary::Nil))
            }
            Statement::Block(block) => {
                let is_function_body = self.entering_function_body;
                self.entering_function_body = false;

                let saved_scope_base = if is_function_body {
                    None
                } else {
                    while self.scope_stack.len() <= block.scope {
                        self.scope_stack.push(0);
                    }
                    let saved = self.scope_stack[block.scope];
                    self.scope_stack[block.scope] = self.stack.len();
                    Some(saved)
                };

                self.depth += 1;
                let stack_len_before = self.stack.len();
                let mut last = Primary::Nil.to_object();
                self.environment.push();
                for declaration_id in &block.declarations {
                    let declaration = &ast.declaration_arena[*declaration_id];
                    match declaration {
                        Declaration::Comment(_) => continue,
                        _ => (),
                    };
                    let e_last = self.eval_declaration(ast, declaration);
                    match e_last {
                        Ok(r) => last = r,
                        Err(r) => {
                            self.environment.pop();
                            self.stack.truncate(stack_len_before);
                            if let Some(saved) = saved_scope_base {
                                self.scope_stack[block.scope] = saved;
                            }
                            self.depth -= 1;
                            return Err(r);
                        }
                    }
                }
                self.environment.pop();
                self.stack.truncate(stack_len_before);
                if let Some(saved) = saved_scope_base {
                    self.scope_stack[block.scope] = saved;
                }
                self.depth -= 1;
                Ok(last)
            }
            Statement::IfStatement(ifelses, else_stmt) => {
                for (expr, stmt) in ifelses.iter() {
                    let expr = self.eval(ast, *expr)?;
                    match expr.get_primary()? {
                        Primary::Boolean(true) => {
                            return self.eval_statement(ast, &ast.statement_arena[*stmt]);
                        }
                        Primary::Boolean(false) => {}

                        _ => {
                            let span = expr.span;
                            return Err(InterpretorError::UnexpectedType(
                                expr.inner,
                                "Boolean".to_string(),
                                Some(span),
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
                            let span = r.span;
                            return Err(InterpretorError::UnexpectedType(
                                r.inner,
                                "Boolean".to_string(),
                                Some(span),
                            ));
                        }
                    };
                    match self.eval_statement(ast, &ast.statement_arena[*stmt_id]) {
                        Err(InterpretorError::ForbidenBreak(_)) => break,
                        Err(InterpretorError::ForbidenContinue(_)) => continue,
                        _ => {}
                    };
                }
                Ok(RTObject::default())
            }
            Statement::Break(token) => Err(InterpretorError::ForbidenBreak(token.span)),
            Statement::Continue(token) => Err(InterpretorError::ForbidenContinue(token.span)),
            Statement::Return(token, item) => {
                let item = if let Some(item) = item {
                    self.eval(ast, *item)?
                } else {
                    RTObject::default().located(token.span)
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
                Primary::MySelf => {
                    match self.environment.get("self", &terminal.clone().to_object()) {
                        Ok(value) => Ok(value.locate_with(terminal)),
                        Err(InterpretorError::UnDeclaredIdentifier(_)) => Err(
                            InterpretorError::SelfOutsideConstructor(Some(terminal.span)),
                        ),
                        Err(e) => Err(e),
                    }
                }
                Primary::Local((index, scope)) => {
                    return Ok(self.stack[*index + self.scope_stack[*scope]]
                        .clone()
                        .located(terminal.span));
                }
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
                    _ => {
                        let span = left.span;
                        Err(InterpretorError::UnexpectedType(
                            left.inner,
                            "Boolean or Number".to_string(),
                            Some(span),
                        ))
                    }
                }
            }
            crate::parser::parser::Expr::LogicalAnd(logical_and) => {
                let left = self.eval(ast, logical_and.left)?;
                match &left.get_primary()? {
                    Primary::Boolean(false) => return Ok(left),
                    Primary::Boolean(true) => return self.eval(ast, logical_and.right),
                    _ => {
                        let span = left.span;
                        return Err(InterpretorError::UnexpectedType(
                            left.inner,
                            "Boolean".to_string(),
                            Some(span),
                        ));
                    }
                }
            }
            crate::parser::parser::Expr::LogicalOr(logical_or) => {
                let left = self.eval(ast, logical_or.left)?;
                match &left.get_primary()? {
                    Primary::Boolean(true) => return Ok(left),
                    Primary::Boolean(false) => return self.eval(ast, logical_or.right),
                    _ => {
                        let span = left.span;
                        return Err(InterpretorError::UnexpectedType(
                            left.inner,
                            "Boolean".to_string(),
                            Some(span),
                        ));
                    }
                }
            }
            crate::parser::parser::Expr::Assignment(left, right) => {
                let right = self.eval(ast, *right)?;

                match &ast.expr_arena[*left] {
                    crate::parser::parser::Expr::Terminal(located_primary) => {
                        match &located_primary.inner {
                            Primary::Identifier(s) => {
                                self.environment.assign(&s, &right.inner, right.span)?;
                                return Ok(right);
                            }
                            Primary::Local((index, scope)) => {
                                self.stack[*index + self.scope_stack[*scope]] = right.clone().inner;
                                return Ok(right);
                            }
                            _ => {
                                return Err(InterpretorError::InvalidAssignmentTarget(Some(
                                    located_primary.span,
                                )));
                            }
                        }
                    }

                    crate::parser::parser::Expr::Get(id, s) => {
                        let object = self.eval_get_unamed(ast, *id)?;
                        match &object.inner {
                            RTObject::Class(id) => {
                                self.instance_arena[*id].env.assign(
                                    &s,
                                    &right.inner,
                                    right.span,
                                )?;
                                return Ok(right);
                            }
                            _ => {
                                return Err(InterpretorError::Ungetable(Box::new(object)));
                            }
                        }
                    }
                    _ => return Err(InterpretorError::InvalidAssignmentTarget(None)),
                }
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
        name: &str,
    ) -> Result<LocatedRTObject, InterpretorError> {
        let expr = self.eval(ast, expr_id)?;
        match expr.inner.clone() {
            RTObject::Class(id) => self.instance_arena[id]
                .env
                .get(name, &expr.inner.clone().located(expr.span))
                .map(|x| x.located(expr.span)),
            RTObject::Primary(_) => Err(InterpretorError::Ungetable(Box::new(expr))),
        }
    }

    // Similar to eval_get, but dont actually acces the variable with its name
    // since we need to modify its value
    pub fn eval_get_unamed(
        &mut self,
        ast: &AST,
        expr_id: ExprID,
    ) -> Result<LocatedRTObject, InterpretorError> {
        let expr = self.eval(ast, expr_id)?;
        match &expr.inner {
            RTObject::Class(_) => Ok(expr),
            RTObject::Primary(_) => Err(InterpretorError::Ungetable(Box::new(expr))),
        }
    }
}

pub fn eval(ast: AST, interpreter: &mut Interpreter) -> Result<RTObject, InterpretorError> {
    let mut last = RTObject::default();
    while interpreter.head < ast.roots.len() {
        let root = ast.roots[interpreter.head];
        interpreter.head += 1;
        let ret = interpreter.eval_declaration(&ast, &ast.declaration_arena[root])?;
        last = ret;
    }
    Ok(last)
}
