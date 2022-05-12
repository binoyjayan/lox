use crate::callable::*;
use crate::error::*;
use crate::interpreter::*;
use crate::lox_instance::LoxInstance;
use crate::object::*;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct LoxClass {
    pub name: String,
    pub superclass: Option<Rc<LoxClass>>,
    pub methods: HashMap<String, Object>,
}

impl LoxClass {
    pub fn new(
        name: &str,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, Object>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            superclass,
            methods,
        }
    }
    pub fn instantiate(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Rc<LoxClass>,
    ) -> Result<Object, LoxResult> {
        let instance = Object::Instance(Rc::new(LoxInstance::new(&klass)));
        if let Some(Object::Func(initializer)) = self.find_method("init".to_string()) {
            if let Object::Func(func) = initializer.bind(&instance) {
                func.call(interpreter, arguments, None)?;
            }
        }
        Ok(instance)
    }

    pub fn find_method(&self, name: String) -> Option<Object> {
        if let Some(method) = self.methods.get(&name) {
            Some(method.clone())
        } else if let Some(superclass) = &self.superclass {
            superclass.find_method(name)
        } else {
            None
        }
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(
        &self,
        interpreter: &Interpreter,
        arguments: Vec<Object>,
        klass: Option<Rc<LoxClass>>,
    ) -> Result<Object, LoxResult> {
        self.instantiate(interpreter, arguments, klass.unwrap())
    }
    fn arity(&self) -> usize {
        // A class does not need to have an initializer, but if it does,
        // then use that arity, else use 0
        if let Some(Object::Func(initializer)) = self.find_method("init".to_string()) {
            initializer.arity()
        } else {
            0
        }
    }
}
