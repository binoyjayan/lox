use crate::functions_lox::*;
use crate::functions_native::*;
use crate::lox_class::*;
use crate::lox_instance::LoxInstance;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Identifier(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Func(Rc<LoxFunction>),
    Class(Rc<LoxClass>),
    Instance(Rc<LoxInstance>),
    Native(Rc<LoxNative>),
    Nil,
    IllegalOperation,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Identifier(s) => write!(f, "{}", s),
            Self::Str(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Func(c) => write!(f, "{}", c),
            Self::Class(c) => write!(f, "{}", c),
            Self::Instance(c) => write!(f, "{}", c),
            Self::Native(c) => write!(f, "{}", c),
            Self::Nil => write!(f, "nil"),
            Self::IllegalOperation => write!(f, "illegal-op"),
        }
    }
}
