use core::fmt::{Debug, Display};
use std::fmt;
use std::rc;

use crate::error::*;
use crate::interpreter::*;
use crate::object::*;

pub trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult>;
}

#[derive(Clone)]
pub struct Callable {
    pub func: rc::Rc<dyn LoxCallable>,
}

impl Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<callable>")
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<callable>")
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            rc::Rc::as_ptr(&self.func) as *const (),
            rc::Rc::as_ptr(&other.func) as *const (),
        )
    }
}

impl LoxCallable for Callable {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {
        self.func.call(interpreter, arguments)
    }
    fn arity(&self) -> usize {
        self.func.arity()
    }
}
