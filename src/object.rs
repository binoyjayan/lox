use crate::callable::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Identifier(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Func(Callable),
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
            Self::Func(_c) => write!(f, "callable"),
            Self::Nil => write!(f, "<func>"),
            Self::IllegalOperation => write!(f, "illegal-op"),
        }
    }
}
