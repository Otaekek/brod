use std::rc::Rc;

use clap::Error;

use crate::parser::parser::Primary;
use crate::vm::instructions::{Op, Ops};

pub mod compiler;
pub mod instructions;
pub mod runtime;
use crate::interpreter::interpreter::RTObject;
pub enum VMError {
    ErrOperand,
}
#[derive(Copy, Clone, Debug)]
pub enum VMPrimary {
    Number(f32),
    Caca,
}
pub struct VM {
    ops: Ops,
    stack: Vec<VMPrimary>,
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
    pub fn run(&mut self) -> Result<(), VMError> {
        for op in &self.ops {
            match op {
                instructions::Op::Print => println!("Hello world {:#?} ", self.stack.pop()),
                instructions::Op::Return => todo!(),
                instructions::Op::Constant(f) => {
                    self.stack.push(VMPrimary::Number(f));
                }
                instructions::Op::Add => {
                    let VMPrimary::Number(left) = self.stack.pop().unwrap() else {
                        return Err(VMError::ErrOperand);
                    };
                    let VMPrimary::Number(right) = self.stack.pop().unwrap() else {
                        return Err(VMError::ErrOperand);
                    };
                    self.stack.push(VMPrimary::Number(left + right));
                }
                instructions::Op::Mul => todo!(),
                instructions::Op::Minus => todo!(),
                instructions::Op::Divide => todo!(),
            }
        }
        return Ok(());
    }
}

#[test]
fn test_simple_run() {
    let mut ops = Ops::new();
    ops.push(Op::Constant(20.0));
    ops.push(Op::Constant(22.0));
    ops.push(Op::Add);
    ops.push(Op::Print);
    let mut vm = VM::new();
    vm.ops = ops;
    vm.run();
}
