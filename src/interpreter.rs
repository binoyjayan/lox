// The AST Tree-walk Interpreter
use std::cell;
use std::result;
use crate::expr::*;
use crate::stmt::*;
use crate::error::*;
use crate::token::*;
use crate::object::*;
use crate::environment::*;

pub struct Interpreter {
    environment: cell::RefCell<Environment>
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: cell::RefCell::new(Environment::new())
        }
    }
    pub fn interpret(&self, stmts: &[Stmt]) -> Result<(), LoxErr> {
        for s in stmts {
            if let Err(e) = self.execute(s) {
                e.report("");
                return Err(e);
            }
        }
        Ok(())
    }

    fn execute(&self, stmt: &Stmt) -> Result<(), LoxErr> {
        stmt.accept(self)
    }

    fn evaluate(&self, expr: &Expr) -> Result<Object, LoxErr> {
        expr.accept(self)
    }
}

impl Interpreter {
    // Being a dynamically typed language, perform implicit type conversions
    // for all types for the purposes of determining truthiness. false and
    // nil are falsey, and everything else is truthy
    fn is_truthy(value: &Object) -> bool {
        if let Object::Bool(b) = value {
            *b
        } else {
            !matches!(value, Object::Nil)
        }
    }    
}

impl StmtVisitor<()> for Interpreter {
    fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<(), LoxErr> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }
    fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<(), LoxErr> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{}", value);
        Ok(())
    }
    fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), LoxErr> {
        let value = if let Some(initilalizer) = &stmt.initializer {
            self.evaluate(initilalizer)?
        } else {
            Object::Nil
        };
        self.environment.borrow_mut().define(&stmt.name.lexeme, value);
        Ok(())
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_assign_expr(&self, expr: &AssignExpr) -> Result<Object, LoxErr> {
        let value =self.evaluate(&expr.value)?;
        self.environment.borrow_mut().assign(&expr.name, value)      
    }

    // Simplest all expression. Just convert the literal to a 'value'
    // Do not call this when an identifier is encountered.
    fn visit_literal_expr(&self, expr: &LiteralExpr) -> result::Result<Object, LoxErr> {
        Ok(match &expr.value {
            Some(v) => v.clone(),
            None => Object::Nil,
        })
    }

    // Evaluate left and right subexpressions first and then perform arithmetic,
    // logical or equality operations. The arithmetic operation produces result
    // whose type is same as  the operands. However, the logical and equality
    // operators produce a boolean result.
    fn visit_binary_expr(&self, expr: &BinaryExpr) -> Result<Object, LoxErr> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        let ttype = expr.operator.ttype;

        let result = match (left, right) {
            (Object::Number(left), Object::Number(right)) => match ttype {
                TokenType::Minus => Object::Number(left - right),
                TokenType::Slash => Object::Number(left / right),
                TokenType::Star => Object::Number(left * right),
                TokenType::Plus => Object::Number(left + right),
                TokenType::Greater => Object::Bool(left > right),
                TokenType::GreaterEqual => Object::Bool(left >= right),
                TokenType::Less => Object::Bool(left < right),
                TokenType::LessEqual => Object::Bool(left <= right),
                TokenType::BangEqual => Object::Bool(left != right),
                TokenType::EqualEqual => Object::Bool(left == right),
                _ => {
                    Object::IllegalOperation
                }
            },
            (Object::Number(left), Object::Str(right)) => match ttype {
                TokenType::Plus => Object::Str(format!("{left}{right}")),
                TokenType::Star => Object::Str(right.repeat(left as usize)),
                _ => Object::IllegalOperation,
            },
            (Object::Str(left), Object::Number(right)) => match ttype {
                TokenType::Plus => Object::Str(format!("{left}{right}")),
                TokenType::Star => Object::Str(left.repeat(right as usize)),
                _ => Object::IllegalOperation,
            },
            (Object::Str(left), Object::Str(right)) => match ttype {
                TokenType::Plus => Object::Str(format!("{left}{right}")),
                TokenType::BangEqual => Object::Bool(left != right),
                TokenType::EqualEqual => Object::Bool(left == right),
                _ => Object::IllegalOperation,
            },
            (Object::Bool(left), Object::Bool(right)) => match ttype {
                TokenType::BangEqual => Object::Bool(left != right),
                TokenType::EqualEqual => Object::Bool(left == right),
                _ => Object::IllegalOperation,
            },
            (Object::Nil, Object::Nil) => match ttype {
                TokenType::BangEqual => Object::Bool(false),
                TokenType::EqualEqual => Object::Bool(true),
                _ => Object::IllegalOperation,
            },
            (Object::Nil, _) => match ttype {
                TokenType::Equal => Object::Bool(false),
                TokenType::BangEqual => Object::Bool(true),
                _ => Object::IllegalOperation,
            },
            _ => Object::IllegalOperation,
        };

        if result == Object::IllegalOperation {
            Err(LoxErr::error_runtime(&expr.operator, "Illegal operation"))
        } else {
            Ok(result)
        }
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr) -> Result<Object, LoxErr> {
        self.evaluate(&expr.expression)
    }

    // unary expressions have a single subexpression must be evaluated first
    // Then apply the unary operator itself to the result. Here, the minus ('-')
    // operator negates the subexpression, whereas the Bang ('!') operator 
    // inverts the truth value.
    fn visit_unary_expr(&self, expr: &UnaryExpr) -> Result<Object, LoxErr> {
        let right = self.evaluate(&expr.right)?;
        match expr.operator.ttype {
            TokenType::Minus => {
                if let Object::Number(n) = right {
                    Ok(Object::Number(-n))
                } else {
                    println!("Negation operation is not allowed on '{}'", right);
                    Ok(Object::Nil)
                }
            },
            TokenType::Bang => Ok(Object::Bool(!Self::is_truthy(&right))),
            _ => Err(LoxErr::error_at_token(&expr.operator, "Unreachable")),
        }
    }   

    fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<Object, LoxErr> {
        self.environment.borrow().get(&expr.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // helpers
    fn make_literal(o: Object) -> Box<Expr> {
        Box::new(Expr::Literal(LiteralExpr { value: Some(o) }))
    }        
    fn make_token(ttype: TokenType, lexeme: &str) -> Token {
        Token::new(ttype,lexeme.to_string(),None, 1, 0)
    }

    #[test]
    fn test_unary_minus() {
        let interpreter = Interpreter::new();
        let unary_expr = UnaryExpr {
            operator: make_token(TokenType::Minus, "-"),
            right: make_literal(Object::Number(123.))
        };
        let result = interpreter.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(-123.0)));
    }

    #[test]
    fn test_unary_not() {
        let interpreter = Interpreter::new();
        let unary_expr = UnaryExpr {
            operator: make_token(TokenType::Bang, "!"),
            right: make_literal(Object::Bool(false)),
        };
        let result = interpreter.visit_unary_expr(&unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_binary_sub() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(321.)),
            operator: make_token(TokenType::Minus, "-"),
            right: make_literal(Object::Number(123.))
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(198.)));
    }

    #[test]
    fn test_binary_div() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(100.)),
            operator: make_token(TokenType::Slash, "/"),
            right: make_literal(Object::Number(10.))
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(10.)));
    }

    #[test]
    fn test_binary_mul() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(100.)),
            operator: make_token(TokenType::Star, "*"),
            right: make_literal(Object::Number(10.))
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(1000.)));
    }

    #[test]
    fn test_binary_add() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(100.)),
            operator: make_token(TokenType::Plus, "+"),
            right: make_literal(Object::Number(10.))
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(110.)));
    }

    #[test]
    fn test_binary_concat() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Str("Hello, ".to_string())),
            operator: make_token(TokenType::Plus, "+"),
            right: make_literal(Object::Str("World!".to_string())),
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Str("Hello, World!".to_string())));
    }

    #[test]
    fn test_binary_illegal() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(321.)),
            operator: make_token(TokenType::Minus, "-"),
            right: make_literal(Object::Bool(true))
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_eq_nil() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Nil),
            operator: make_token(TokenType::EqualEqual, "=="),
            right: make_literal(Object::Nil)
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_binary_eq_str() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Str("Hello".to_string())),
            operator: make_token(TokenType::EqualEqual, "=="),
            right: make_literal(Object::Str("Hello".to_string())),
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_binary_ne_str() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Str("Hello".to_string())),
            operator: make_token(TokenType::EqualEqual, "=="),
            right: make_literal(Object::Str("World".to_string())),
        };
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(false)));
    }

    // Use a helper to test '==' and '!=', '>', '>=', '<', and '<='
    fn run_comparisons(tok: &Token, nums: Vec<f64>, value: f64, results: Vec<bool>) {
        let interpreter = Interpreter::new();

        for (num, r) in nums.iter().zip(results) {
            let binary_expr = BinaryExpr {
                left: make_literal(Object::Number(*num)),
                operator: tok.clone(),
                right: make_literal(Object::Number(value)),
            };
            let result = interpreter.visit_binary_expr(&binary_expr);
            assert!(result.is_ok());
            assert_eq!(result.ok(), Some(Object::Bool(r)), "Testing {} {} {}", num, tok.lexeme, value);
        }
    }

    #[test]
    fn test_binary_eq() {
        let numbers = vec![14., 15., 16.];
        let results = vec![ false, true, false];
        run_comparisons(&make_token(TokenType::EqualEqual, "=="), numbers, 15., results);
    }

    #[test]
    fn test_binary_ne() {
        let numbers = vec![14., 15., 16.];
        let results = vec![ true, false, true];
        run_comparisons(&make_token(TokenType::BangEqual, "!="), numbers, 15., results);
    }

    #[test]
    fn test_binary_gt() {
        let numbers = vec![14., 15., 16.];
        let results = vec![ false, false, true];
        run_comparisons(&make_token(TokenType::Greater, ">"), numbers, 15., results);
    }

    #[test]
    fn test_binary_ge() {
        let numbers = vec![14., 15., 16.];
        let results = vec![ false, true, true];
        run_comparisons(&make_token(TokenType::GreaterEqual, ">="), numbers, 15., results);
    }

    #[test]
    fn test_binary_lt() {
        let numbers = vec![14., 15., 16.];
        let results = vec![ true, false, false];
        run_comparisons(&make_token(TokenType::Less, "<"), numbers, 15., results);
    }

    #[test]
    fn test_binary_le() {
        let numbers = vec![14., 15., 16.];
        let results = vec![ true, true, false];
        run_comparisons(&make_token(TokenType::LessEqual, "<="), numbers, 15., results);
    }

    #[test]
    fn test_nested_expr() {
        //-123 * (45.67) = -5617.41
        let binary_expr = BinaryExpr {
            left: Box::new(Expr::Unary({
                UnaryExpr {
                    operator: make_token(TokenType::Minus, "-"),
                    right: make_literal(Object::Number(123.)),
                }
            })),
            operator: make_token(TokenType::Star, "*"),
            right: make_literal(Object::Number(45.67)),
        };
        let interpreter = Interpreter::new();
        let result = interpreter.visit_binary_expr(&binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(-5617.41)));
    }

    #[test]
    fn test_var_stmt() {
        let token = Token::new(
            TokenType::Identifier,
             "var_num".to_string(),
              None, 1, 1
        );
        let var_stmt = VarStmt {
            name: token.clone(),
            initializer: None,
        };
        let interpreter = Interpreter::new();
        assert!(interpreter.visit_var_stmt(&var_stmt).is_ok());
        assert_eq!(interpreter.environment.borrow().get(&token).unwrap(), Object::Nil);
    }

    #[test]
    fn test_var_stmt_initialized() {
        let token = Token::new(
            TokenType::Identifier,
             "var_num".to_string(),
              None, 1, 1
        );
        let var_stmt = VarStmt {
            name: token.clone(),
            initializer: Some(*make_literal(Object::Number(123.))),
        };
        let interpreter = Interpreter::new();
        assert!(interpreter.visit_var_stmt(&var_stmt).is_ok());
        assert_eq!(interpreter.environment.borrow().get(&token).unwrap(), Object::Number(123.));
    }

    #[test]
    fn test_variable_expression() {
        // First define a variable
        let token = Token::new(
            TokenType::Identifier,
             "var_num".to_string(),
              None, 1, 1
        );
        let var_stmt = VarStmt {
            name: token.clone(),
            initializer: Some(*make_literal(Object::Number(123.))),
        };
        let interpreter = Interpreter::new();
        assert!(interpreter.visit_var_stmt(&var_stmt).is_ok());

        // Now use the defined variable in an expression
        let var_expr = VariableExpr { name: token.clone() };
        assert!(interpreter.visit_variable_expr(&var_expr).is_ok());
        assert_eq!(interpreter.visit_variable_expr(&var_expr).unwrap(), Object::Number(123.));
    }
    #[test]
    fn test_variable_expression_undefined() {
        let token = Token::new(
            TokenType::Identifier,
             "var_num".to_string(),
              None, 1, 1
        );
        let interpreter = Interpreter::new();
        // Try to use an undefined variable in an expression
        let var_expr = VariableExpr { name: token.clone() };
        assert!(interpreter.visit_variable_expr(&var_expr).is_err());
    }
}
