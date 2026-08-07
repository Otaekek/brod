use crate::diagnostic::Span;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Op {
    Return,
    Constant,
    LongConstant,
    Add,
    Mul,
    Minus,
    Divide,
}

pub struct Ops {
    op_list: Vec<u8>,
    span_list: Vec<Span>,
}

impl Ops {
    pub fn new() -> Self {
        Self {
            op_list: vec![],
            span_list: vec![],
        }
    }
}
