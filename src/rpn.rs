use crate::parser::{ASTVisitor, Binary, Expr, ExprID, Literal, Operator, Unary};

enum RpnToken {
    Operator(super::parser::Operator),
    Number(f64),
    Not,
    Minus,
}

pub struct RpnBuilder {
    pub ops: Vec<RpnToken>,
}

fn error(msg: &str) {
    eprintln!("RPN error: {msg}");
}
impl ASTVisitor for RpnBuilder {
    fn visit_binary(&mut self, arena: &[Expr], binary: &Binary) {
        self.ops.push(RpnToken::Operator(binary.operator));
    }
    fn visit_literal(&mut self, literal: &Literal) {
        match literal {
            Literal::Number(n) => self.ops.push(RpnToken::Number(*n)),
            Literal::String(_) => error("Invalid Token"),
            Literal::True => error("Invalid Token"),
            Literal::False => error("Invalid Token"),
            Literal::Nil => error("Invalid Token"),
        }
    }
    fn visit_unary(&mut self, arena: &[Expr], unary: &Unary) {
        match unary {
            Unary::Not(_) => self.ops.push(RpnToken::Not),
            Unary::Minus(_) => self.ops.push(RpnToken::Minus),
        }
    }
}
