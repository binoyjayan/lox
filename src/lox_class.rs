use crate::callable::*;
use crate::error::*;
use crate::interpreter::*;
use crate::lox_instance::LoxInstance;
use crate::object::*;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub struct LoxClass {
    pub name: String,
    pub methods: HashMap<String, Object>,
}

impl LoxClass {
    pub fn new(name: &str, methods: HashMap<String, Object>) -> Self {
        Self {
            name: name.to_owned(),
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
        // 'cloned()' is same as 'map(|obj| obj.clone())'
        self.methods.get(&name).cloned()
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}

impl fmt::Debug for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{self}")
    }
}

impl Clone for LoxClass {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            methods: self.methods.clone(),
        }
    }
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
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
