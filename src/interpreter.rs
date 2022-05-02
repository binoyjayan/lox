// The AST Tree-walk Interpreter
use crate::callable::Callable;
use crate::callable::LoxCallable;
use crate::environment::*;
use crate::error::*;
use crate::expr::*;
use crate::functions_lox::LoxFunction;
use crate::functions_native::*;
use crate::lox_class::LoxClass;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::result;

pub struct Interpreter {
    environment: RefCell<Rc<RefCell<Environment>>>,
    pub globals: Rc<RefCell<Environment>>,
    pub locals: RefCell<HashMap<Rc<Expr>, usize>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new()));
        globals.borrow_mut().define(
            "clock",
            Object::Func(Callable {
                func: Rc::new(NativeClock {}),
            }),
        );
        Interpreter {
            globals: Rc::clone(&globals),
            environment: RefCell::new(Rc::clone(&globals)),
            locals: RefCell::new(HashMap::new()),
        }
    }
    pub fn interpret(&self, stmts: &[Rc<Stmt>]) -> Result<(), LoxResult> {
        for s in stmts {
            if let Err(e) = self.execute(s.clone()) {
                return Err(e);
            }
        }
        Ok(())
    }

    fn execute(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self)
    }

    pub fn execute_block(
        &self,
        stmts: &Rc<Vec<Rc<Stmt>>>,
        environment: Environment,
    ) -> Result<(), LoxResult> {
        let previous = self.environment.replace(Rc::new(RefCell::new(environment)));
        // Execute each statment and stop on first error. if not return Ok
        let result = stmts.iter().try_for_each(|stmt| self.execute(stmt.clone()));
        // Restore the previous environment
        self.environment.replace(previous);
        result
    }

    fn evaluate(&self, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        expr.accept(expr.clone(), self)
    }

    pub fn resolve(&self, expr: Rc<Expr>, depth: usize) {
        self.locals.borrow_mut().insert(expr, depth);
    }
    fn lookup_variable(&self, name: &Token, expr: Rc<Expr>) -> Result<Object, LoxResult> {
        if let Some(distance) = self.locals.borrow().get(&expr) {
            self.environment.borrow().borrow().get_at(*distance, name)
        } else {
            self.globals.borrow().get(&name)
        }
    }

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
    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        let e = Environment::new_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)
    }
    fn visit_class_stmt(&self, _base: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        self.environment
            .borrow()
            .borrow_mut()
            .define(&stmt.name.lexeme, Object::Nil);
        let klass = Object::Class(Rc::new(LoxClass::new(&stmt.name.lexeme)));
        self.environment
            .borrow()
            .borrow_mut()
            .assign(&stmt.name, klass)?;
        Ok(())
    }
    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.evaluate(stmt.expression.clone())?;
        Ok(())
    }
    fn visit_function_stmt(&self, _: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        // Closure holds on to the surrounding variables when a function is declared.
        // Save the current environment in 'closure' which is the environment
        // that is active when a function is declared, not when it is called.
        let function = LoxFunction::new(stmt, self.environment.borrow().deref());
        self.environment.borrow().borrow_mut().define(
            &stmt.name.lexeme,
            Object::Func(Callable {
                func: Rc::new(function),
            }),
        );
        Ok(())
    }
    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        if Self::is_truthy(&self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.then_branch.clone())
        } else if let Some(else_branch) = stmt.else_branch.clone() {
            self.execute(else_branch)
        } else {
            Ok(())
        }
    }
    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        let value = self.evaluate(stmt.expression.clone())?;
        println!("{}", value);
        Ok(())
    }
    fn visit_return_stmt(&self, _base: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if let Some(value) = stmt.value.clone() {
            Err(LoxResult::return_value(self.evaluate(value)?))
        } else {
            Err(LoxResult::return_value(Object::Nil))
        }
    }
    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        let value = if let Some(initilalizer) = stmt.initializer.clone() {
            self.evaluate(initilalizer)?
        } else {
            Object::Nil
        };
        self.environment
            .borrow()
            .borrow_mut()
            .define(&stmt.name.lexeme, value);
        Ok(())
    }
    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        while Self::is_truthy(&self.evaluate(stmt.condition.clone())?) {
            match self.execute(stmt.body.clone()) {
                Err(LoxResult::Break) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }
        Ok(())
    }
    fn visit_break_stmt(&self, _: Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Err(LoxResult::Break)
    }
}

impl ExprVisitor<Object> for Interpreter {
    fn visit_assign_expr(&self, base: Rc<Expr>, expr: &AssignExpr) -> Result<Object, LoxResult> {
        let value = self.evaluate(expr.value.clone())?;
        if let Some(distance) = self.locals.borrow().get(&base) {
            self.environment.borrow().borrow_mut().assign_at(
                *distance,
                &expr.name,
                value.clone(),
            )?;
        } else {
            self.globals
                .borrow_mut()
                .assign(&expr.name, value.clone())?;
        }
        Ok(value)
    }

    // Simplest all expression. Just convert the literal to a 'value'
    // Do not call this when an identifier is encountered.
    fn visit_literal_expr(
        &self,
        _: Rc<Expr>,
        expr: &LiteralExpr,
    ) -> result::Result<Object, LoxResult> {
        Ok(match &expr.value {
            Some(v) => v.clone(),
            None => Object::Nil,
        })
    }

    // Evaluate left and right subexpressions first and then perform arithmetic,
    // logical or equality operations. The arithmetic operation produces result
    // whose type is same as  the operands. However, the logical and equality
    // operators produce a boolean result.
    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;
        let right = self.evaluate(expr.right.clone())?;
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
                _ => Object::IllegalOperation,
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
            Err(LoxResult::error_runtime(
                &expr.operator,
                "Illegal operation",
            ))
        } else {
            Ok(result)
        }
    }

    fn visit_call_expr(&self, _base: Rc<Expr>, expr: &CallExpr) -> Result<Object, LoxResult> {
        let callee = self.evaluate(expr.callee.clone())?;
        let mut arguments = Vec::new();
        for arg in expr.arguments.clone() {
            arguments.push(self.evaluate(arg)?);
        }
        if let Object::Func(function) = callee {
            if arguments.len() != function.func.arity() {
                return Err(LoxResult::error_runtime(
                    &expr.paren,
                    &format!(
                        "Expected {} arguments but got {}",
                        function.func.arity(),
                        arguments.len()
                    ),
                ));
            }
            function.func.call(self, arguments)
        } else if let Object::Class(klass) = callee {
            if arguments.len() != klass.arity() {
                return Err(LoxResult::error_runtime(
                    &expr.paren,
                    &format!(
                        "Expected {} arguments but got {}",
                        klass.arity(),
                        arguments.len()
                    ),
                ));
            }
            // klass.call(self, arguments)
            klass.instantiate(self, arguments, Rc::clone(&klass))
        } else {
            Err(LoxResult::error_runtime(
                &expr.paren,
                "Can only call functions and classes",
            ))
        }
    }

    fn visit_grouping_expr(
        &self,
        _base: Rc<Expr>,
        expr: &GroupingExpr,
    ) -> Result<Object, LoxResult> {
        self.evaluate(expr.expression.clone())
    }

    fn visit_logical_expr(&self, _base: Rc<Expr>, expr: &LogicalExpr) -> Result<Object, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;
        if expr.operator.ttype == TokenType::Or {
            // If lhs of logical or is true, do not evaluate rhs
            if Self::is_truthy(&left) {
                return Ok(left);
            }
        } else {
            // If lhs of logical and is true, do not evaluate rhs
            if !Self::is_truthy(&left) {
                return Ok(left);
            }
        }
        // evaluate rhs only if the result wasn't enough to determine the truthiness
        self.evaluate(expr.right.clone())
    }

    // unary expressions have a single subexpression must be evaluated first
    // Then apply the unary operator itself to the result. Here, the minus ('-')
    // operator negates the subexpression, whereas the Bang ('!') operator
    // inverts the truth value.
    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<Object, LoxResult> {
        let right = self.evaluate(expr.right.clone())?;
        match expr.operator.ttype {
            TokenType::Minus => {
                if let Object::Number(n) = right {
                    Ok(Object::Number(-n))
                } else {
                    println!("Negation operation is not allowed on '{}'", right);
                    Ok(Object::Nil)
                }
            }
            TokenType::Bang => Ok(Object::Bool(!Self::is_truthy(&right))),
            _ => Err(LoxResult::error_at_token(&expr.operator, "Unreachable")),
        }
    }

    fn visit_variable_expr(
        &self,
        base: Rc<Expr>,
        expr: &VariableExpr,
    ) -> Result<Object, LoxResult> {
        self.lookup_variable(&expr.name, base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // helpers
    fn make_literal(o: Object) -> Rc<Expr> {
        Rc::new(Expr::Literal(LiteralExpr { value: Some(o) }))
    }
    fn make_token(ttype: TokenType, lexeme: &str) -> Token {
        Token::new(ttype, lexeme.to_string(), None, 1, 0)
    }

    #[test]
    fn test_unary_minus() {
        let interpreter = Interpreter::new();
        let unary_expr = UnaryExpr {
            operator: make_token(TokenType::Minus, "-"),
            right: make_literal(Object::Number(123.)),
        };
        let result = interpreter.visit_unary_expr(&Expr::Unary(unary_expr), &unary_expr);
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
        let result = interpreter.visit_unary_expr(&Expr::Unary(unary_expr), &unary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Bool(true)));
    }

    #[test]
    fn test_binary_sub() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(321.)),
            operator: make_token(TokenType::Minus, "-"),
            right: make_literal(Object::Number(123.)),
        };
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(198.)));
    }

    #[test]
    fn test_binary_div() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(100.)),
            operator: make_token(TokenType::Slash, "/"),
            right: make_literal(Object::Number(10.)),
        };
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(10.)));
    }

    #[test]
    fn test_binary_mul() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(100.)),
            operator: make_token(TokenType::Star, "*"),
            right: make_literal(Object::Number(10.)),
        };
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(1000.)));
    }

    #[test]
    fn test_binary_add() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(100.)),
            operator: make_token(TokenType::Plus, "+"),
            right: make_literal(Object::Number(10.)),
        };
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
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
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Str("Hello, World!".to_string())));
    }

    #[test]
    fn test_binary_illegal() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Number(321.)),
            operator: make_token(TokenType::Minus, "-"),
            right: make_literal(Object::Bool(true)),
        };
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_eq_nil() {
        let interpreter = Interpreter::new();
        let binary_expr = BinaryExpr {
            left: make_literal(Object::Nil),
            operator: make_token(TokenType::EqualEqual, "=="),
            right: make_literal(Object::Nil),
        };
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
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
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
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
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
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
            let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
            assert!(result.is_ok());
            assert_eq!(
                result.ok(),
                Some(Object::Bool(r)),
                "Testing {} {} {}",
                num,
                tok.lexeme,
                value
            );
        }
    }

    #[test]
    fn test_binary_eq() {
        let numbers = vec![14., 15., 16.];
        let results = vec![false, true, false];
        run_comparisons(
            &make_token(TokenType::EqualEqual, "=="),
            numbers,
            15.,
            results,
        );
    }

    #[test]
    fn test_binary_ne() {
        let numbers = vec![14., 15., 16.];
        let results = vec![true, false, true];
        run_comparisons(
            &make_token(TokenType::BangEqual, "!="),
            numbers,
            15.,
            results,
        );
    }

    #[test]
    fn test_binary_gt() {
        let numbers = vec![14., 15., 16.];
        let results = vec![false, false, true];
        run_comparisons(&make_token(TokenType::Greater, ">"), numbers, 15., results);
    }

    #[test]
    fn test_binary_ge() {
        let numbers = vec![14., 15., 16.];
        let results = vec![false, true, true];
        run_comparisons(
            &make_token(TokenType::GreaterEqual, ">="),
            numbers,
            15.,
            results,
        );
    }

    #[test]
    fn test_binary_lt() {
        let numbers = vec![14., 15., 16.];
        let results = vec![true, false, false];
        run_comparisons(&make_token(TokenType::Less, "<"), numbers, 15., results);
    }

    #[test]
    fn test_binary_le() {
        let numbers = vec![14., 15., 16.];
        let results = vec![true, true, false];
        run_comparisons(
            &make_token(TokenType::LessEqual, "<="),
            numbers,
            15.,
            results,
        );
    }

    #[test]
    fn test_nested_expr() {
        //-123 * (45.67) = -5617.41
        let binary_expr = BinaryExpr {
            left: Rc::new(Expr::Unary({
                UnaryExpr {
                    operator: make_token(TokenType::Minus, "-"),
                    right: make_literal(Object::Number(123.)),
                }
            })),
            operator: make_token(TokenType::Star, "*"),
            right: make_literal(Object::Number(45.67)),
        };
        let interpreter = Interpreter::new();
        let result = interpreter.visit_binary_expr(&Expr::Binary(binary_expr), &binary_expr);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(Object::Number(-5617.41)));
    }

    #[test]
    fn test_var_stmt() {
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        let var_stmt = VarStmt {
            name: token.clone(),
            initializer: None,
        };
        let interpreter = Interpreter::new();
        assert!(interpreter
            .visit_var_stmt(&Stmt::Var(var_stmt), &var_stmt)
            .is_ok());
        assert_eq!(
            interpreter
                .environment
                .borrow()
                .borrow()
                .get(&token)
                .unwrap(),
            Object::Nil
        );
    }

    #[test]
    fn test_var_stmt_initialized() {
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        let var_stmt = VarStmt {
            name: token.clone(),
            initializer: Some(make_literal(Object::Number(123.))),
        };
        let interpreter = Interpreter::new();
        assert!(interpreter
            .visit_var_stmt(&Stmt::Var(var_stmt), &var_stmt)
            .is_ok());
        assert_eq!(
            interpreter
                .environment
                .borrow()
                .borrow()
                .get(&token)
                .unwrap(),
            Object::Number(123.)
        );
    }

    #[test]
    fn test_variable_expression() {
        // First define a variable
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        let var_stmt = VarStmt {
            name: token.clone(),
            initializer: Some(make_literal(Object::Number(123.))),
        };
        let interpreter = Interpreter::new();
        assert!(interpreter
            .visit_var_stmt(&Stmt::Var(var_stmt), &var_stmt)
            .is_ok());

        // Now use the defined variable in an expression
        let var_expr = VariableExpr {
            name: token.clone(),
        };
        assert!(interpreter
            .visit_variable_expr(&Expr::Variable(var_expr), &var_expr)
            .is_ok());
        assert_eq!(
            interpreter
                .visit_variable_expr(&Expr::Variable(var_expr), &var_expr)
                .unwrap(),
            Object::Number(123.)
        );
    }
    #[test]
    fn test_variable_expression_undefined() {
        let token = Token::new(TokenType::Identifier, "var_num".to_string(), None, 1, 1);
        let interpreter = Interpreter::new();
        // Try to use an undefined variable in an expression
        let var_expr = VariableExpr {
            name: token.clone(),
        };
        assert!(interpreter
            .visit_variable_expr(&Expr::Variable(var_expr), &var_expr)
            .is_err());
    }
}
