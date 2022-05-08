use crate::error::*;
use crate::lox_class::*;
use crate::object::*;
use crate::token::*;
use std::cell::RefCell;
use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;
use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: RefCell<HashMap<String, Object>>,
}

impl LoxInstance {
    pub fn new(klass: &Rc<LoxClass>) -> Self {
        Self {
            klass: Rc::clone(klass),
            fields: RefCell::new(HashMap::new()),
        }
    }
    pub fn get(&self, name: &Token, this: &Rc<LoxInstance>) -> Result<Object, LoxResult> {
        if let hash_map::Entry::Occupied(o) = self.fields.borrow_mut().entry(name.lexeme.clone()) {
            Ok(o.get().clone())
        } else if let Some(method) = self.klass.find_method(name.lexeme.clone()) {
            if let Object::Func(func) = method {
                Ok(func.bind(&Object::Instance(this.clone())))
            } else {
                Err(LoxResult::error_runtime(
                    name,
                    "Cannot bind 'this' to a non-function method",
                ))
            }
        } else {
            Err(LoxResult::error_runtime(
                name,
                &format!("Undefined property {}", name.lexeme),
            ))
        }
    }
    pub fn set(&self, name: &Token, object: Object) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), object);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "instance of {}", self.klass.name)
    }
}
