use crate::error::*;
use crate::expr::*;
use crate::interpreter::*;
use crate::stmt::*;
use crate::token::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    // Two RefCells needed to make both vector and hashmap mutable
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
    in_loop: RefCell<bool>,
    had_error: RefCell<bool>,
    current_function: RefCell<FunctionType>,
}

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
            had_error: RefCell::new(false),
            in_loop: RefCell::new(false),
            current_function: RefCell::new(FunctionType::None),
        }
    }
    pub fn resolve(&self, stmts: &Rc<Vec<Rc<Stmt>>>) -> Result<(), LoxResult> {
        for stmt in stmts.deref() {
            self.resolve_stmt(stmt.clone())?;
        }
        Ok(())
    }
    fn resolve_stmt(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self)
    }
    fn resolve_expr(&self, expr: Rc<Expr>) -> Result<(), LoxResult> {
        expr.accept(expr.clone(), self)
    }
    fn begin_scope(&self) {
        self.scopes.borrow_mut().push(RefCell::new(HashMap::new()))
    }
    fn end_scope(&self) {
        self.scopes.borrow_mut().pop();
    }
    fn declare(&self, name: &Token) {
        if !self.scopes.borrow().is_empty() {
            // Add variable to the innermost scope so it shadows outer ones if any
            // Mark it as false (not ready yet). Value associated with a key in the
            // scope map represents whether or not we have finished resolving the initializer
            if let Some(scope) = self.scopes.borrow().last() {
                // Report error if the variable is being redefined
                if scope.borrow().contains_key(&name.lexeme) {
                    self.resolve_error(
                        name,
                        "A variable with the same name already exists in this scope",
                    );
                }
                scope.borrow_mut().insert(name.lexeme.clone(), false);
            }
        }
    }
    fn define(&self, name: &Token) {
        // After declaring the variable, resolve its initializer expr in the same scope
        // where the new variable now exists but is unavailable. Once the initializer
        // expression is done, the variable is ready by doing this:
        if let Some(scope) = self.scopes.borrow().last() {
            if scope.borrow().contains_key(&name.lexeme) {
                scope.borrow_mut().insert(name.lexeme.clone(), true);
            }
        }
    }
    // Helper to resolve a variable by starting at innermost scope and working outwards
    // If the variable was found in the current scope, return pass '0'
    // if it in the immediate enclosing scope, pass  '1' and so on.
    fn resolve_local(&self, expr: Rc<Expr>, name: &Token) {
        for (scope, map) in self.scopes.borrow().iter().rev().enumerate() {
            if map.borrow().contains_key(&name.lexeme) {
                self.interpreter.resolve(expr.clone(), scope)
            }
        }
    }

    // Unlike variable, define functions eagerly so that a function
    // can recursively refer to itself.
    fn resolve_function(
        &self,
        _: Rc<Stmt>,
        function: &FunctionStmt,
        ftype: FunctionType,
    ) -> Result<(), LoxResult> {
        let enclosing_function = self.current_function.replace(ftype);
        self.begin_scope();
        for param in function.params.deref() {
            self.declare(param);
            self.define(param);
        }
        self.resolve(&function.body)?;
        self.end_scope();
        self.current_function.replace(enclosing_function);
        Ok(())
    }

    pub fn resolve_error(&self, token: &Token, message: &str) {
        self.had_error.replace(true);
        LoxResult::error_runtime(token, message);
    }

    pub fn success(&self) -> bool {
        !*self.had_error.borrow()
    }
}

impl<'a> StmtVisitor<()> for Resolver<'a> {
    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }
    fn visit_class_stmt(&self, _: Rc<Stmt>, stmt: &ClassStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        Ok(())
    }
    fn visit_expression_stmt(&self, _: Rc<Stmt>, stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())
    }
    // Functions both bind names and introduce a scope.
    // The name of the fn itself is bound in the surrounding scope where it is declared.
    // When we step into the function's body, we also bind its parameters into
    // that inner function's scope
    fn visit_function_stmt(&self, base: Rc<Stmt>, stmt: &FunctionStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(base, stmt, FunctionType::Function)
    }
    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.then_branch.clone())?;
        if let Some(else_branch) = stmt.else_branch.clone() {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }
    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expression.clone())
    }
    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt) -> Result<(), LoxResult> {
        if *self.current_function.borrow() == FunctionType::None {
            self.resolve_error(&stmt.keyword, "Can't return from top-level code")
        }
        if let Some(value) = stmt.value.clone() {
            self.resolve_expr(value)?;
        }
        Ok(())
    }
    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        if let Some(init) = stmt.initializer.clone() {
            self.resolve_expr(init)?;
        }
        self.define(&stmt.name);
        Ok(())
    }
    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt) -> Result<(), LoxResult> {
        let nesting_prev = self.in_loop.replace(true);
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_stmt(stmt.body.clone())?;
        self.in_loop.replace(nesting_prev);
        Ok(())
    }
    fn visit_break_stmt(&self, _: Rc<Stmt>, stmt: &BreakStmt) -> Result<(), LoxResult> {
        if !*self.in_loop.borrow() {
            self.resolve_error(&stmt.token, "break statements are not allowed here");
        }
        Ok(())
    }
}

impl<'a> ExprVisitor<()> for Resolver<'a> {
    // Resolve the expression for the assigned value in case it also contains
    // references to other variable. Then use the existing 'resolve_local' method
    // to resolve the variable that is being assigned to.
    fn visit_assign_expr(&self, base: Rc<Expr>, expr: &AssignExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_local(base, &expr.name);
        Ok(())
    }
    fn visit_literal_expr(&self, _: Rc<Expr>, _expr: &LiteralExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())
    }
    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.callee.clone())?;
        for arg in expr.arguments.clone() {
            self.resolve_expr(arg)?;
        }
        Ok(())
    }
    // property dispatch is dynamic since the property name is not resolved here
    // i.e. only the object is resolved not the token after '.'
    fn visit_get_expr(&self, _: Rc<Expr>, expr: &GetExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.object.clone())
    }
    fn visit_grouping_expr(&self, _: Rc<Expr>, expr: &GroupingExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.expression.clone())
    }
    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())
    }
    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr) -> Result<(), LoxResult> {
        self.resolve_expr(expr.right.clone())
    }
    fn visit_variable_expr(&self, base: Rc<Expr>, expr: &VariableExpr) -> Result<(), LoxResult> {
        // Disallow shadowing variable in its own initializer (var a = a;)
        if !self.scopes.borrow().is_empty()
            && self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.lexeme)
                == Some(&false)
        {
            Err(LoxResult::error_runtime(
                &expr.name,
                "Can't read local variable its own initializer",
            ))
        } else {
            self.resolve_local(base, &expr.name);
            Ok(())
        }
    }
}
