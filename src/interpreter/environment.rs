use crate::diagnostic::Span;
use crate::interpreter::functions::Function;
use crate::interpreter::interpreter::{InterpretorError, LocatedRTObject, RTObject};
use crate::parser::parser::Primary;
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct Environment {
    stack: Vec<HashMap<String, RTObject>>,
    // pub functions: HashMap<String, Rc<Function>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            // Start with global scope
            stack: vec![HashMap::new()],
            // functions: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, value: RTObject) {
        let last = self.stack.len() - 1;
        self.stack[last].insert(name, value);
    }

    pub fn _check(&mut self, name: &String) -> bool {
        let last = self.stack.len();
        for i in 0..self.stack.len() {
            let r = self.stack[last - i].get(name);
            if r.is_some() {
                return true;
            }
        }
        false
    }

    pub fn assign(
        &mut self,
        name: &str,
        value: &RTObject,
        ident_span: Span,
    ) -> Result<(), InterpretorError> {
        let last = self.stack.len();
        for i in 0..self.stack.len() {
            let r = self.stack[last - i - 1].get(name);
            if r.is_some() {
                let m = self.stack[last - i - 1].get_mut(name).unwrap();
                *m = value.clone();
                return Ok(());
            }
        }
        Err(InterpretorError::UnDeclaredIdentifier(Box::new(
            Primary::Identifier(name.to_string())
                .located(ident_span)
                .to_object(),
        )))
    }

    pub fn get(&self, name: &str, ident: &LocatedRTObject) -> Result<RTObject, InterpretorError> {
        let last = self.stack.len();
        for i in 0..self.stack.len() {
            let r = self.stack[last - i - 1].get(name);
            if r.is_some() {
                return Ok(r.unwrap().clone());
            }
        }

        Err(InterpretorError::UnDeclaredIdentifier(Box::new(
            ident.clone(),
        )))
    }
    pub fn push(&mut self) {
        self.stack.push(HashMap::new());
    }
    pub fn pop(&mut self) {
        self.stack.pop();
    }
}
