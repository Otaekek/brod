use std::fmt::Display;

use crate::parser::{AST, ASTVisitor, Binary, Expr, Operator, Terminal, Unary};

#[derive(Copy, Clone)]
enum RpnToken {
    Operator(super::parser::Operator),
    Number(f64),
    Minus,
}
impl Display for RpnToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpnToken::Operator(operator) => match operator {
                Operator::Equal => write!(f, "{}", "="),
                Operator::NotEqual => write!(f, "{}", "!="),
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
    pub fn solve(ast: &AST) -> Option<f64> {
        let mut rpn = Self::default();
        if ast.roots.is_empty() {
            return None;
        }
        ast.traverse_lrn(ast.roots[0], &mut rpn);
        if rpn.ops.len() < 3 {
            return None;
        }
        fn compute(left: f64, right: f64, op: Operator) -> f64 {
            match op {
                Operator::Plus => left + right,
                Operator::Minus => left - right,
                Operator::Slash => left / right,
                Operator::Star => left * right,
                _ => unreachable!(),
            }
        }
        let mut stack: Vec<f64> = Vec::with_capacity(rpn.ops.len());
        stack.resize(stack.capacity(), 0.0);
        let mut stack_idx = 0;
        for op in &rpn.ops[0..] {
            match op {
                RpnToken::Operator(operator) => {
                    stack_idx -= 1;
                    let res = compute(stack[stack_idx - 1], stack[stack_idx], *operator);
                    stack[stack_idx - 1] = res;
                }
                RpnToken::Number(n) => {
                    stack[stack_idx] = *n;
                    stack_idx += 1;
                }
                RpnToken::Minus => stack[stack_idx] = -stack[stack_idx],
            }
        }
        Some(stack[0])
    }
}
fn error(msg: &str) {
    eprintln!("RPN error: {msg}");
}
impl ASTVisitor for RpnCalculator {
    fn visit_binary(&mut self, _arena: &[Expr], binary: &Binary) {
        self.ops.push(RpnToken::Operator(binary.operator));
    }
    fn visit_literal(&mut self, literal: &Terminal) {
        match literal {
            Terminal::Number(n) => self.ops.push(RpnToken::Number(*n)),
            Terminal::String(_) => error("Invalid Token"),
            Terminal::True => error("Invalid Token"),
            Terminal::False => error("Invalid Token"),
            Terminal::Nil => error("Invalid Token"),
            Terminal::SemiColon => error("Invalid Token"),
        }
    }
    fn visit_unary(&mut self, _arena: &[Expr], unary: &Unary) {
        match unary {
            Unary::Not(_) => error("invalid token"),
            Unary::Minus(_) => self.ops.push(RpnToken::Minus),
        }
    }

    fn visit_ternary(&mut self, _arena: &[Expr], _ternary: &crate::parser::Ternary) {
        unreachable!();
    }
}
