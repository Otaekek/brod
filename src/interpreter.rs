use std::collections::HashMap;

use crate::parser::{ASTVisitor, ExprID, Operator, Primary, Statement, Unary, AST};

pub struct Interpreter {
    variables: HashMap<String, Primary>,
}

#[derive(Debug)]
pub enum InterpretorError {
    DivideByZero,
    ForbiddenUnaryOperation(Unary, Primary),
    ForbiddenBinaryOperation(Operator, Primary, Primary),
    FobbiddenTernay,
    UnDeclaredIdentifier(String),
}
impl InterpretorError {
    fn format_error(&self, source: &str, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpretorError::DivideByZero => write!(f, "{}", "Runtime Error: Division by zero"),
            InterpretorError::ForbiddenUnaryOperation(unary, terminal) => {
                write!(
                    f,
                    "Runtime Error: Forbiden operator: {} on type {}",
                    unary, terminal
                )
            }
            InterpretorError::ForbiddenBinaryOperation(operator, terminal, terminal1) => write!(
                f,
                "Runtime Error: Forbiden operator: {} on type {} and {}",
                operator, terminal, terminal1
            ),
            InterpretorError::FobbiddenTernay => write!(
                f,
                "{}",
                "Runtime Error: left hand side of a ternary should be a boolean or a number"
            ),
            InterpretorError::UnDeclaredIdentifier(v) => {
                write!(f, "Runtime Error: Undeclared Variable {}", v)
            }
        }
    }
    pub fn get_formated_error(&self, source: &str) -> String {
        format!(
            "{}",
            ErrorDisplay {
                error: self,
                source
            }
        )
    }
    pub fn display_error(&self, source: &str) {
        eprintln!("{}", self.get_formated_error(source));
    }
}
struct ErrorDisplay<'a> {
    error: &'a InterpretorError,
    source: &'a str,
}

impl<'a> std::fmt::Display for ErrorDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.format_error(self.source, f)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn eval_unary(&mut self, ast: &AST, unary: Unary) -> Result<Primary, InterpretorError> {
        match unary {
            crate::parser::Unary::Not(v) => {
                let last = self.eval(ast, v)?;
                match &last {
                    Primary::Number(v) => Ok(Primary::Number(*v)),
                    Primary::Boolean(v) => Ok(Primary::Boolean(!v)),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
            crate::parser::Unary::Minus(v) => {
                let last = self.eval(ast, v)?;
                match &last {
                    Primary::Number(v) => Ok(Primary::Number(*v)),
                    Primary::Boolean(v) => Ok(Primary::Boolean(!v)),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
        }
    }
    pub fn eval_binary(
        &mut self,
        operator: Operator,
        left: Primary,
        right: Primary,
    ) -> Result<Primary, InterpretorError> {
        use crate::parser::Operator::*;
        match (&left, &right) {
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
                    operator, left, right,
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
                    operator, left, right,
                )),
            },
            (Primary::Nil, Primary::Nil) => match operator {
                Equal => Ok(Primary::Boolean(true)),
                NotEqual => Ok(Primary::Boolean(false)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator, left, right,
                )),
            },
            (Primary::String(s), Primary::Number(n)) => {
                if operator == Plus {
                    Ok(Primary::String(s.clone() + n.to_string().as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator, left, right,
                    ))
                }
            }
            (Primary::Number(n), Primary::String(s)) => {
                if operator == Plus {
                    Ok(Primary::String(s.clone() + n.to_string().as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator, left, right,
                    ))
                }
            }
            _ => Err(InterpretorError::ForbiddenBinaryOperation(
                operator, left, right,
            )),
        }
    }
    pub fn eval_statement(
        &mut self,
        ast: &AST,
        input: Statement,
    ) -> Result<Primary, InterpretorError> {
        match input {
            Statement::ExprStatement(expr_id) => self.eval(ast, expr_id),
            Statement::PrintStatement(items) => {
                for x in items {
                    let r = self.eval(ast, x)?;
                    match r {
                        Primary::Identifier(s) => {
                            let v = self.variables.get(&s);
                            match v {
                                Some(v) => println!("{v}"),
                                None => return Err(InterpretorError::UnDeclaredIdentifier(s)),
                            }
                        }
                        x => println!("{x}"),
                    }
                }
                Ok(Primary::Nil)
            }
            Statement::Assignment(ident, expr_id) => {
                let value = self.eval(ast, expr_id)?;
                self.variables.insert(ident, value.clone());
                println!("{:#?}", self.variables);
                Ok(value)
            }
            Statement::Empty => Ok(Primary::Nil),
        }
    }
    pub fn eval(&mut self, ast: &AST, root: ExprID) -> Result<Primary, InterpretorError> {
        match &ast.arena[root] {
            crate::parser::Expr::Terminal(terminal) => {
                return Ok(terminal.clone());
            }
            crate::parser::Expr::Unary(unary) => self.eval_unary(ast, *unary),
            crate::parser::Expr::Binary(binary) => {
                let left = self.eval(ast, binary.left)?;
                let right = self.eval(ast, binary.right)?;
                self.eval_binary(binary.operator, left, right)
            }
            crate::parser::Expr::Ternary(ternary) => {
                let left = self.eval(ast, ternary.left)?;
                match left {
                    Primary::Boolean(v) => match v {
                        true => self.eval(ast, ternary.middle),
                        false => self.eval(ast, ternary.right),
                    },
                    Primary::Number(v) => match v > 0.0 {
                        true => self.eval(ast, ternary.middle),
                        false => self.eval(ast, ternary.right),
                    },
                    _ => Err(InterpretorError::FobbiddenTernay),
                }
            } // crate::parser::Expr::Statement(expr_id) => self.eval(ast, *expr_id),
        }
    }
}

pub fn eval(ast: AST) -> Result<Primary, InterpretorError> {
    let mut interpreter = Interpreter::new();
    let mut last = Primary::Nil;
    for root in &ast.roots {
        let ret = interpreter.eval_statement(&ast, root.clone())?;
        last = ret;
    }
    Ok(last)
}
