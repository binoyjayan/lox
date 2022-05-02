use crate::lox_class::*;
use std::fmt;
use std::{fmt::Display, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub struct LoxInstance {
    klass: Rc<LoxClass>,
}

impl LoxInstance {
    pub fn new(klass: &Rc<LoxClass>) -> Self {
        Self {
            klass: Rc::clone(klass),
        }
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "instance of {}", self.klass.name)
    }
}
