use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::callable::*;
use crate::environment::*;
use crate::error::*;
use crate::interpreter::*;
use crate::lox_class::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;

pub struct LoxFunction {
    name: Token,
    is_initializer: bool,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Rc<Stmt>>>,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(
        declaration: &FunctionStmt,
        closure: &Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name: declaration.name.clone(),
            is_initializer,
            params: Rc::clone(&declaration.params),
            body: Rc::clone(&declaration.body),
            closure: Rc::clone(closure),
        }
    }
    // Create a new environment nestled inside the method's original closure
    // Like a closure within a closure. When the method is called, that will
    // become the parent of the methods body's environment
    pub fn bind(&self, instance: &Object) -> Object {
        let env = RefCell::new(Environment::new_enclosing(Rc::clone(&self.closure)));
        env.borrow_mut().define("this", instance.clone());
        Object::Func(Rc::new(Self {
            name: self.name.clone(),
            is_initializer: self.is_initializer,
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::new(env),
        }))
    }
}

impl LoxCallable for LoxFunction {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        _klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        let mut e = Environment::new_enclosing(Rc::clone(&self.closure));
        for (param, arg) in self.params.iter().zip(arguments.iter()) {
            e.define(&param.lexeme, arg.clone());
        }
        match interpreter.execute_block(&self.body, e) {
            Err(LoxResult::ReturnValue { value: v }) => Ok(v),
            Err(e) => Err(e),
            Ok(_) => {
                if self.is_initializer {
                    // If the function is an initializer, then return 'this' instance
                    self.closure.borrow().get_at(0, "this")
                } else {
                    Ok(Object::Nil)
                }
            }
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
            is_initializer: self.is_initializer,
            params: Rc::clone(&self.params),
            body: Rc::clone(&self.body),
            closure: Rc::clone(&self.closure),
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
