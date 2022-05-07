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
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
        klass: Rc<LoxClass>,
    ) -> Result<Object, LoxResult> {
        Ok(Object::Instance(Rc::new(LoxInstance::new(&klass))))
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
        _interpreter: &Interpreter,
        _arguments: Vec<Object>,
    ) -> Result<Object, LoxResult> {
        Err(LoxResult::system_error("Can't call a class"))
    }
    fn arity(&self) -> usize {
        // self.params.len()
        0
    }
}
