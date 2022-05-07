use crate::error::*;
use crate::lox_class::*;
use crate::object::*;
use crate::token::*;
use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;
use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: HashMap<String, Object>,
}

impl LoxInstance {
    pub fn new(klass: &Rc<LoxClass>) -> Self {
        Self {
            klass: Rc::clone(klass),
            fields: HashMap::new(),
        }
    }
    pub fn get(&mut self, name: &Token) -> Result<Object, LoxResult> {
        if let hash_map::Entry::Occupied(o) = self.fields.entry(name.lexeme.clone()) {
            Ok(o.get().clone())
        } else {
            Err(LoxResult::error_runtime(
                name,
                &format!("Undefined property {}", name.lexeme),
            ))
        }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "instance of {}", self.klass.name)
    }
}
