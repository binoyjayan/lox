use crate::error::LoxResult;
use crate::object::Object;
use crate::token::Token;
use std::cell::RefCell;
use std::collections::hash_map;
use std::collections::HashMap;
use std::rc::Rc;
pub struct Environment {
    pub values: HashMap<String, Object>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_enclosing(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, LoxResult> {
        if let Some(obj) = self.values.get(&name.lexeme.to_string()) {
            Ok((*obj).clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(LoxResult::error_runtime(
                name,
                &format!("Undefined variable '{}'", name.lexeme),
            ))
        }
    }

    // Unlike 'get', 'get_at' exactly knows where the variable is instead of
    // walking the entire environment chain.
    // Walk a fixed number of hops up the parent chain and return the environment there.
    // Once we have the environment, get_at returns the value of the variable in
    // that environment's map.
    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Object, LoxResult> {
        if distance == 0 {
            Ok(self.values.get(&name.lexeme).unwrap().clone())
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(distance - 1, name)
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<Object, LoxResult> {
        if let hash_map::Entry::Occupied(mut entry) = self.values.entry(name.lexeme.clone()) {
            entry.insert(value.clone());
            return Ok(value);
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value.clone())?;
            return Ok(value);
        }
        return Err(LoxResult::error_runtime(
            name,
            &format!("Undefined variable '{}'", name.lexeme),
        ));
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Object,
    ) -> Result<(), LoxResult> {
        if distance == 0 {
            self.values.insert(name.lexeme.clone(), value.clone());
        } else {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(distance - 1, name, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::token::TokenType;

    use super::*;

    #[test]
    fn test_variable_definition() {
        let mut e = Environment::new();
        e.define("var_bool", Object::Bool(true));
        e.define("var_num", Object::Number(123.));
        assert!(e.values.contains_key(&"var_bool".to_string()));
        assert!(e.values.contains_key(&"var_num".to_string()));
        assert_eq!(
            e.values.get(&"var_bool".to_string()).unwrap(),
            &Object::Bool(true)
        );
        assert_eq!(
            e.values.get(&"var_num".to_string()).unwrap(),
            &Object::Number(123.)
        );
    }

    #[test]
    fn test_variable_redefinition() {
        let mut e = Environment::new();
        e.define("same_name", Object::Bool(true));
        e.define("same_name", Object::Number(123.));
        assert_eq!(
            e.values.get(&"same_name".to_string()).unwrap(),
            &Object::Number(123.)
        );
    }

    #[test]
    fn test_variable_lookup_ok() {
        let tok = Token::new(
            TokenType::Identifier,
            "var_str".to_string(),
            Some(Object::Identifier("var_str".to_string())),
            1,
            1,
        );
        let mut e = Environment::new();
        e.define("var_str", Object::Str("str_val".to_string()));
        assert_eq!(e.get(&tok).unwrap(), Object::Str("str_val".to_string()));
    }

    #[test]
    fn test_variable_lookup_failed() {
        let tok = Token::new(TokenType::Identifier, "var_str".to_string(), None, 1, 1);
        let e = Environment::new();
        assert!(e.get(&tok).is_err());
    }

    #[test]
    fn test_variable_assignment() {
        let mut e = Environment::new();
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        e.define("var_num", Object::Number(123.));
        assert!(e.assign(&token, Object::Number(456.)).is_ok());
        assert_eq!(e.get(&token).unwrap(), Object::Number(456.));
    }

    #[test]
    fn test_variable_assignment_failed() {
        let mut e = Environment::new();
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        assert!(e.assign(&token, Object::Number(456.)).is_err());
    }

    #[test]
    fn test_enclosing_env() {
        let e = Rc::new(RefCell::new(Environment::new()));
        let f = Environment::new_enclosing(Rc::clone(&e));
        assert_eq!(f.enclosing.unwrap().borrow().values, e.borrow().values);
    }

    #[test]
    fn test_read_var_enclosed_env() {
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        let e = Rc::new(RefCell::new(Environment::new()));
        e.borrow_mut().define("var_num", Object::Number(123.));
        let f = Environment::new_enclosing(Rc::clone(&e));
        assert_eq!(f.get(&token).unwrap(), Object::Number(123.));
    }

    #[test]
    fn test_write_var_enclosed_env() {
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        let e = Rc::new(RefCell::new(Environment::new()));
        e.borrow_mut().define("var_num", Object::Number(123.));
        let mut f = Environment::new_enclosing(Rc::clone(&e));
        assert!(f.assign(&token, Object::Number(456.)).is_ok());
        assert_eq!(f.get(&token).unwrap(), Object::Number(456.));
    }
}
