use std::collections::HashMap;

use crate::parser::{
    Declaration, ExprID, LocatedPrimary, Operator, Primary, Statement, Unary, AST,
};

pub struct Interpreter {
    variables: HashMap<String, LocatedPrimary>,
}

#[derive(Debug)]
pub enum InterpretorError {
    ForbiddenUnaryOperation(Unary, LocatedPrimary),
    ForbiddenBinaryOperation(Operator, LocatedPrimary, LocatedPrimary),
    FobbiddenTernay,
    UnDeclaredIdentifier(LocatedPrimary),
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
                write!(f, "Forbiden operator: {} on type {}", unary, terminal.inner)
            }
            InterpretorError::ForbiddenBinaryOperation(operator, terminal, terminal1) => write!(
                f,
                "Forbiden operator: {} on type {} and {} in file {} from {} to {}",
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
                write!(f, "Undeclared Variable {}", v.inner)
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
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn eval_unary(
        &mut self,
        ast: &AST,
        unary: Unary,
    ) -> Result<LocatedPrimary, InterpretorError> {
        match unary {
            crate::parser::Unary::Not(v) => {
                let last = self.eval(ast, v)?;
                match &last.inner {
                    Primary::Number(v) => {
                        Ok(Primary::Number(*v).located(last.token_start, last.token_end))
                    }
                    Primary::Boolean(v) => {
                        Ok(Primary::Boolean(!v).located(last.token_start, last.token_end))
                    }
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
            crate::parser::Unary::Minus(v) => {
                let last = self.eval(ast, v)?;
                match &last.inner {
                    Primary::Number(v) => {
                        Ok(Primary::Number(*v).located(last.token_start, last.token_end))
                    }
                    Primary::Boolean(v) => {
                        Ok(Primary::Boolean(!v).located(last.token_start, last.token_end))
                    }
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
    ) -> Result<LocatedPrimary, InterpretorError> {
        use crate::parser::Operator::*;
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
                    Ok(Primary::String(n.to_string() + s.as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator, left, right,
                    ))
                }
            }
            _ => Err(InterpretorError::ForbiddenBinaryOperation(
                operator, left, right,
            )),
        };
        Ok(ret?.located(token_start, token_end))
    }

    pub fn eval_declaration(
        &mut self,
        ast: &AST,
        input: Declaration,
    ) -> Result<Primary, InterpretorError> {
        match input {
            Declaration::Statement(statement) => self.eval_statement(ast, statement),
            Declaration::Assignment(ident, expr_id) => {
                let value = self.eval(ast, expr_id)?;
                self.variables.insert(ident, value.clone());
                Ok(value.inner)
            }
            Declaration::Empty => Ok(Primary::Nil),
        }
    }
    pub fn eval_statement(
        &mut self,
        ast: &AST,
        input: Statement,
    ) -> Result<Primary, InterpretorError> {
        match input {
            Statement::ExprStatement(expr_id) => Ok(self.eval(ast, expr_id)?.inner),
            Statement::PrintStatement(items) => {
                for x in items {
                    let r = self.eval(ast, x)?;
                    match &r.inner {
                        Primary::Identifier(s) => {
                            let v = self.variables.get(s);
                            match v {
                                Some(v) => println!("{}", v.inner),
                                None => return Err(InterpretorError::UnDeclaredIdentifier(r)),
                            }
                        }
                        x => println!("{x}"),
                    }
                }
                Ok(Primary::Nil)
            }
        }
    }
    pub fn eval(&mut self, ast: &AST, root: ExprID) -> Result<LocatedPrimary, InterpretorError> {
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
                match left.inner {
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

pub fn eval(ast: AST, interpreter: &mut Interpreter) -> Result<Primary, InterpretorError> {
    let mut last = Primary::Nil;
    for root in &ast.roots {
        let ret = interpreter.eval_declaration(&ast, root.clone())?;
        last = ret;
    }
    Ok(last)
}
