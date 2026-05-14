use crate::parser::{ASTVisitor, ExprID, Operator, Terminal, Unary, AST};

pub struct Interpreter {}

#[derive(Debug)]
pub enum InterpretorError {
    DivideByZero,
    ForbiddenUnaryOperation(Unary, Terminal),
    ForbiddenBinaryOperation(Operator, Terminal, Terminal),
    FobbiddenTernay,
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
        Self {}
    }

    pub fn eval_unary(&mut self, ast: &AST, unary: Unary) -> Result<Terminal, InterpretorError> {
        match unary {
            crate::parser::Unary::Not(v) => {
                let last = self.eval(ast, v)?;
                match &last {
                    Terminal::Number(v) => Ok(Terminal::Number(*v)),
                    Terminal::Boolean(v) => Ok(Terminal::Boolean(!v)),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
            crate::parser::Unary::Minus(v) => {
                let last = self.eval(ast, v)?;
                match &last {
                    Terminal::Number(v) => Ok(Terminal::Number(*v)),
                    Terminal::Boolean(v) => Ok(Terminal::Boolean(!v)),
                    _ => Err(InterpretorError::ForbiddenUnaryOperation(unary, last)),
                }
            }
        }
    }
    pub fn eval_binary(
        &mut self,
        operator: Operator,
        left: Terminal,
        right: Terminal,
    ) -> Result<Terminal, InterpretorError> {
        use crate::parser::Operator::*;
        match (&left, &right) {
            (Terminal::Number(left_n), Terminal::Number(right_n)) => match operator {
                Equal => Ok(Terminal::Boolean(left_n == right_n)),
                NotEqual => Ok(Terminal::Boolean(left_n != right_n)),
                Lesser => Ok(Terminal::Boolean(left_n < right_n)),
                LesserEqual => Ok(Terminal::Boolean(left_n <= right_n)),
                Greater => Ok(Terminal::Boolean(left_n > right_n)),
                GreaterEqual => Ok(Terminal::Boolean(left_n >= right_n)),
                Plus => Ok(Terminal::Number(left_n + right_n)),
                Minus => Ok(Terminal::Number(left_n - right_n)),
                Slash => Ok(Terminal::Number(left_n / right_n)),
                Star => Ok(Terminal::Number(left_n * right_n)),
            },
            (Terminal::String(left_s), Terminal::String(right_s)) => match operator {
                Equal => Ok(Terminal::Boolean(left_s == right_s)),
                NotEqual => Ok(Terminal::Boolean(left_s != right_s)),
                Plus => Ok(Terminal::String(left_s.to_owned() + right_s)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator, left, right,
                )),
            },
            (Terminal::Boolean(left_bool), Terminal::Boolean(right_bool)) => match operator {
                Equal => Ok(Terminal::Boolean(left_bool == right_bool)),
                NotEqual => Ok(Terminal::Boolean(left_bool != right_bool)),
                Lesser => Ok(Terminal::Boolean(left_bool < right_bool)),
                LesserEqual => Ok(Terminal::Boolean(left_bool <= right_bool)),
                Greater => Ok(Terminal::Boolean(left_bool > right_bool)),
                GreaterEqual => Ok(Terminal::Boolean(left_bool >= right_bool)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator, left, right,
                )),
            },
            (Terminal::Nil, Terminal::Nil) => match operator {
                Equal => Ok(Terminal::Boolean(true)),
                NotEqual => Ok(Terminal::Boolean(false)),
                _ => Err(InterpretorError::ForbiddenBinaryOperation(
                    operator, left, right,
                )),
            },
            (Terminal::String(s), Terminal::Number(n)) => {
                if operator == Plus {
                    Ok(Terminal::String(s.clone() + n.to_string().as_str()))
                } else {
                    Err(InterpretorError::ForbiddenBinaryOperation(
                        operator, left, right,
                    ))
                }
            }
            (Terminal::Number(n), Terminal::String(s)) => {
                if operator == Plus {
                    Ok(Terminal::String(s.clone() + n.to_string().as_str()))
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
    pub fn eval(&mut self, ast: &AST, root: ExprID) -> Result<Terminal, InterpretorError> {
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
                    Terminal::Boolean(v) => match v {
                        true => self.eval(ast, ternary.middle),
                        false => self.eval(ast, ternary.right),
                    },
                    Terminal::Number(v) => match v > 0.0 {
                        true => self.eval(ast, ternary.middle),
                        false => self.eval(ast, ternary.right),
                    },
                    _ => Err(InterpretorError::FobbiddenTernay),
                }
            }
            crate::parser::Expr::Statement(expr_id) => self.eval(ast, *expr_id),
            crate::parser::Expr::PrintStatement(items) => {
                for x in items {
                    let r = self.eval(ast, *x)?;
                    println!("{r}");
                }
                Ok(Terminal::Nil)
            }
        }
    }
}

pub fn eval(ast: AST) -> Result<Terminal, InterpretorError> {
    let mut interpreter = Interpreter::new();
    let mut last = Terminal::Nil;
    for root in &ast.roots {
        let ret = interpreter.eval(&ast, *root)?;
        last = ret;
    }
    Ok(last)
}
