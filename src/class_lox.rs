use std::fmt;

pub struct LoxClass {
    name: String,    
}

impl LoxClass {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
        }
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
        }
    }
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}


