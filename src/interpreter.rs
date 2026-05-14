use crate::parser::{ASTVisitor, Terminal, AST};

pub struct Interpreter {
    stack: Vec<Terminal>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(4096),
        }
    }
    pub fn ret(&self) -> Option<Terminal> {
        self.stack.first().cloned()
    }
}

impl ASTVisitor for Interpreter {
    fn visit_ternary(&mut self, _arena: &[crate::parser::Expr], _ternary: &crate::parser::Ternary) {
        let right = self.stack.pop().unwrap();
        let middle = self.stack.pop().unwrap();
        let left = self.stack.pop().unwrap();

        match (left, middle, right) {
            (Terminal::Boolean(b), x, y) => {
                if b {
                    self.stack.push(x);
                } else {
                    self.stack.push(y);
                }
            }
            (Terminal::Number(n), x, y) => {
                if n != 0.0 {
                    self.stack.push(x);
                } else {
                    self.stack.push(y);
                }
            }
            _ => todo!(),
        }
    }

    fn visit_binary(&mut self, _arena: &[crate::parser::Expr], binary: &crate::parser::Binary) {
        let right = self.stack.pop().unwrap();
        let left = self.stack.pop().unwrap();
        use crate::parser::Operator::*;
        match (left, right) {
            (Terminal::Number(left_n), Terminal::Number(right_n)) => match binary.operator {
                Equal => self.stack.push(Terminal::Boolean(left_n == right_n)),
                NotEqual => self.stack.push(Terminal::Boolean(left_n != right_n)),
                Lesser => self.stack.push(Terminal::Boolean(left_n < right_n)),
                LesserEqual => self.stack.push(Terminal::Boolean(left_n <= right_n)),
                Greater => self.stack.push(Terminal::Boolean(left_n > right_n)),
                GreaterEqual => self.stack.push(Terminal::Boolean(left_n >= right_n)),
                Plus => self.stack.push(Terminal::Number(left_n + right_n)),
                Minus => self.stack.push(Terminal::Number(left_n - right_n)),
                Slash => self.stack.push(Terminal::Number(left_n / right_n)),
                Star => self.stack.push(Terminal::Number(left_n * right_n)),
            },
            (Terminal::String(left_s), Terminal::String(right_s)) => match binary.operator {
                Equal => self.stack.push(Terminal::Boolean(left_s == right_s)),
                NotEqual => self.stack.push(Terminal::Boolean(left_s != right_s)),
                Plus => self.stack.push(Terminal::String(left_s + &right_s)),
                x => todo!(),
            },
            (Terminal::Boolean(left_bool), Terminal::Boolean(right_bool)) => {
                match binary.operator {
                    Equal => self.stack.push(Terminal::Boolean(left_bool == right_bool)),
                    NotEqual => self.stack.push(Terminal::Boolean(left_bool != right_bool)),
                    Lesser => self.stack.push(Terminal::Boolean(left_bool < right_bool)),
                    LesserEqual => self.stack.push(Terminal::Boolean(left_bool <= right_bool)),
                    Greater => self.stack.push(Terminal::Boolean(left_bool > right_bool)),
                    GreaterEqual => self.stack.push(Terminal::Boolean(left_bool >= right_bool)),
                    _ => todo!(),
                }
            }
            (Terminal::Nil, Terminal::Nil) => match binary.operator {
                Equal => self.stack.push(Terminal::Boolean(true)),
                NotEqual => self.stack.push(Terminal::Boolean(false)),
                _ => todo!(),
            },
            (x, y) => todo!(),
        }
    }

    fn visit_terminal(&mut self, literal: &crate::parser::Terminal) {
        self.stack.push(literal.clone());
    }

    fn visit_unary(&mut self, _arena: &[crate::parser::Expr], unary: &crate::parser::Unary) {
        match unary {
            crate::parser::Unary::Not(_) => {
                let last = self.stack.pop();
                match last {
                    Some(Terminal::Number(n)) => {
                        self.stack.push(Terminal::Boolean(n != 0.0));
                    }
                    Some(Terminal::Boolean(b)) => self.stack.push(Terminal::Boolean(!b)),
                    Some(Terminal::Nil) => todo!(),
                    Some(Terminal::String(s)) => todo!(),
                    None => todo!(),
                }
            }
            crate::parser::Unary::Minus(_) => {
                let last = self.stack.pop();
                match last {
                    Some(Terminal::Number(n)) => {
                        self.stack.push(Terminal::Number(-n));
                    }
                    Some(Terminal::Boolean(b)) => todo!(),
                    Some(Terminal::Nil) => todo!(),
                    Some(Terminal::String(s)) => todo!(),
                    None => todo!(),
                }
            }
        }
    }
}

pub fn eval(ast: AST) {
    let mut interpreter = Interpreter::new();
    for root in &ast.roots {
        ast.traverse_lrn(*root, &mut interpreter);
        if interpreter.ret().is_some() {
            println!("{}", interpreter.ret().unwrap());
        }
        interpreter.stack.clear();
    }
}
