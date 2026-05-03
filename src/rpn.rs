use std::fmt::Display;

use crate::parser::{ASTVisitor, Binary, Expr, ExprID, Operator, Terminal, Unary};

#[derive(Copy, Clone)]
enum RpnToken {
    Operator(super::parser::Operator),
    Number(f64),
    Not,
    Minus,
}
impl Display for RpnToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpnToken::Operator(operator) => match operator {
                Operator::Equal => write!(f, "{}", "="),
                Operator::NotEqual => write!(f, "{}", "!="),
                // Operator::Assignment => write!(f, "{}", "=="),
                Operator::Lesser => write!(f, "{}", "<"),
                Operator::LesserEqual => write!(f, "{}", "<="),
                Operator::Greater => write!(f, "{}", ">"),
                Operator::GreaterEqual => write!(f, "{}", ">="),
                Operator::Plus => write!(f, "{}", "+"),
                Operator::Minus => write!(f, "{}", "-"),
                Operator::Slash => write!(f, "{}", "/"),
                Operator::Star => write!(f, "{}", "*"),
            },
            RpnToken::Number(n) => write!(f, "{}", n),
            RpnToken::Not => write!(f, "{}", "+"),
            RpnToken::Minus => write!(f, "{}", "-"),
        }
    }
}
#[derive(Default)]
pub struct RpnCalculator {
    ops: Vec<RpnToken>,
}

impl Display for RpnCalculator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for op in self.ops.iter() {
            write!(f, "{} ", op)?;
        }
        Ok(())
    }
}
impl RpnCalculator {
    fn internal_solve(ops: &[RpnToken]) -> f64 {
        let op = ops[0];
        let left = ops[1];
        Self::internal_solve(&ops[2..])
    }
    pub fn solve(mut self) -> f64 {
        self.ops.reverse();
        let mut left = self.ops.first().unwrap();

        0.0
    }
}
fn error(msg: &str) {
    eprintln!("RPN error: {msg}");
}
impl ASTVisitor for RpnCalculator {
    fn visit_binary(&mut self, arena: &[Expr], binary: &Binary) {
        self.ops.push(RpnToken::Operator(binary.operator));
    }
    fn visit_literal(&mut self, literal: &Terminal) {
        match literal {
            Terminal::Number(n) => self.ops.push(RpnToken::Number(*n)),
            Terminal::String(_) => error("Invalid Token"),
            Terminal::True => error("Invalid Token"),
            Terminal::False => error("Invalid Token"),
            Terminal::Nil => error("Invalid Token"),
        }
    }
    fn visit_unary(&mut self, arena: &[Expr], unary: &Unary) {
        match unary {
            Unary::Not(_) => self.ops.push(RpnToken::Not),
            Unary::Minus(_) => self.ops.push(RpnToken::Minus),
        }
    }
}
