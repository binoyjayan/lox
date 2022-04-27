use std::fmt;
use std::rc::Rc;

use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::interpreter::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;

pub struct LoxFunction {
    name: Token,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
}

impl LoxFunction {
    pub fn new(declaration: &FunctionStmt) -> Self {
        Self {
            name: declaration.name.clone(),
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn call(&self, interpreter: &Interpreter, arguments: Vec<Object>) -> Result<Object, LoxResult> {
        let mut e = Environment::new_enclosing(Rc::clone(&interpreter.globals));
        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            e.define(&param.lexeme, arg.clone());
        }
        match interpreter.execute_block(&self.body, e) {
            Err(LoxResult::ReturnValue { value: v }) => Ok(v),
            Ok(_) => Ok(Object::Nil),
            Err(e) => Err(e),
        }
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let param_list = self
            .params
            .iter()
            .map(|p| p.lexeme.clone())
            .collect::<Vec<String>>()
            .join(", ");

        // <fun foo(a, b, c)>
        write!(f, "<fun {}({param_list})>", self.name)
    }
}

impl fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{self}")
    }
}

impl Clone for LoxFunction {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
        }
    }
}

impl PartialEq for LoxFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.ttype == other.name.ttype
            && Rc::ptr_eq(&self.params, &other.params)
            && Rc::ptr_eq(&self.body, &other.body)
    }
}
