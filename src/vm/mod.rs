use std::rc::Rc;

use crate::parser::parser::Primary;
use crate::vm::instructions::Ops;

pub mod compiler;
pub mod instructions;
pub mod runtime;
use crate::interpreter::interpreter::RTObject;

pub struct VM {
    ops: Ops,
    stack: Vec<Primary>,
    objects: Vec<Rc<RTObject>>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ops: Ops::new(),
            stack: vec![],
            objects: vec![],
        }
    }
}
