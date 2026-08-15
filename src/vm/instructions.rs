use crate::diagnostic::Span;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Print = 0,
    Return,
    Constant(f32),
    Add,
    Mul,
    Minus,
    Divide,
}

impl Op {
    pub fn size(&self) -> usize {
        match self {
            Op::Constant(_) => 4,
            _ => 0,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Op {
        match bytes[0] {
            0 => Op::Print,
            1 => Op::Return,
            2 => Op::Constant(f32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]])),
            3 => Op::Add,
            4 => Op::Mul,
            5 => Op::Minus,
            6 => Op::Divide,
            b => panic!("unknown opcode {b}"),
        }
    }

    pub fn to_bytes(self, buf: &mut Vec<u8>) {
        match self {
            Op::Print => buf.push(0),
            Op::Return => buf.push(1),
            Op::Constant(v) => {
                buf.push(2);
                buf.extend_from_slice(&v.to_le_bytes());
            }
            Op::Add => buf.push(3),
            Op::Mul => buf.push(4),
            Op::Minus => buf.push(5),
            Op::Divide => buf.push(6),
        }
    }
}

pub struct Ops {
    pub op_list: Vec<u8>,
    pub span_list: Vec<Span>,
    pub op_idx: usize,
}

impl Ops {
    pub fn new() -> Self {
        Self {
            op_list: vec![],
            span_list: vec![],
            op_idx: 0,
        }
    }
    pub fn push(&mut self, op: Op) {
        op.to_bytes(&mut self.op_list);
    }
}

impl Iterator for Ops {
    type Item = Op;

    fn next(&mut self) -> Option<Self::Item> {
        if self.op_idx >= self.op_list.len() {
            return None;
        }
        let op = Op::from_bytes(&self.op_list[self.op_idx..]);
        self.op_idx += 1 + op.size();
        Some(op)
    }
}

pub struct OpsIter<'a> {
    ops: &'a Ops,
    idx: usize,
}

impl<'a> Iterator for OpsIter<'a> {
    type Item = Op;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.ops.op_list.len() {
            return None;
        }
        let op = Op::from_bytes(&self.ops.op_list[self.idx..]);
        self.idx += 1 + op.size();
        Some(op)
    }
}

impl<'a> IntoIterator for &'a Ops {
    type Item = Op;
    type IntoIter = OpsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        OpsIter { ops: self, idx: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ops(bytes: Vec<u8>) -> Ops {
        Ops {
            op_list: bytes,
            span_list: vec![],
            op_idx: 0,
        }
    }

    #[test]
    fn simple_ops() {
        let ops = make_ops(vec![0, 1, 3, 4, 5, 6]);
        let collected: Vec<Op> = ops.collect();
        assert!(matches!(collected[0], Op::Print));
        assert!(matches!(collected[1], Op::Return));
        assert!(matches!(collected[2], Op::Add));
        assert!(matches!(collected[3], Op::Mul));
        assert!(matches!(collected[4], Op::Minus));
        assert!(matches!(collected[5], Op::Divide));
    }

    #[test]
    fn constant_op() {
        let v = 1.5f32;
        let mut bytes = vec![2u8];
        bytes.extend_from_slice(&v.to_le_bytes());
        let ops = make_ops(bytes);
        let collected: Vec<Op> = ops.collect();
        assert!(matches!(collected[0], Op::Constant(x) if x == 1.5));
    }

    #[test]
    fn mixed_ops() {
        let v = 3.14f32;
        let mut bytes = vec![2u8];
        bytes.extend_from_slice(&v.to_le_bytes());
        bytes.push(3);
        let ops = make_ops(bytes);
        let collected: Vec<Op> = ops.collect();
        assert!(matches!(collected[0], Op::Constant(x) if x == 3.14f32));
        assert!(matches!(collected[1], Op::Add));
    }

    #[test]
    fn iter_by_ref_does_not_consume() {
        let ops = make_ops(vec![0, 1]);
        let _: Vec<Op> = (&ops).into_iter().collect();
        let second: Vec<Op> = (&ops).into_iter().collect();
        assert_eq!(second.len(), 2);
    }
}
