use crate::error::*;
use crate::expr::*;
use crate::interpreter::*;
use crate::stmt::*;
use crate::token::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

struct Resolver {
    interpreter: Interpreter,
    // Two RefCells needed to make both vector and hashmap mutable
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
        }
    }
    fn resolve(&self, stmts: &Rc<Vec<Rc<Stmt>>>) -> Result<(), LoxResult> {
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
    fn declare(&self, name: &Token) -> Result<(), LoxResult> {
        if !self.scopes.borrow().is_empty() {
            // Add variable to the innermost scope so it shadows outer ones if any
            // Mark it as false (not ready yet). Value associated with a key in the
            // scope map represents whether or not we have finished resolving the initializer
            self.scopes
                .borrow()
                .last()
                .unwrap()
                .borrow_mut()
                .insert(name.lexeme.clone(), false);
        }
        Ok(())
    }
    fn define(&self, name: &Token) -> Result<(), LoxResult> {
        // After declaring the variable, resolve its initializer expr in the same scope
        // where the new variable now exists but is unavailable. Once the initializer
        // expression is done, the varaiable is ready by doing this:
        if !self.scopes.borrow().is_empty() {
            self.scopes
                .borrow()
                .last()
                .unwrap()
                .borrow_mut()
                .insert(name.lexeme.clone(), true);
        }
        Ok(())
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
}

impl StmtVisitor<()> for Resolver {
    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }
    fn visit_expression_stmt(&self, _: Rc<Stmt>, _stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_function_stmt(&self, _: Rc<Stmt>, _stmt: &FunctionStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_if_stmt(&self, _: Rc<Stmt>, _stmt: &IfStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_print_stmt(&self, _: Rc<Stmt>, _stmt: &PrintStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_return_stmt(&self, _: Rc<Stmt>, _stmt: &ReturnStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_var_stmt(&self, _: Rc<Stmt>, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name)?;
        if let Some(init) = stmt.initializer.clone() {
            self.resolve_expr(init)?;
        }
        self.define(&stmt.name)?;
        Ok(())
    }
    fn visit_while_stmt(&self, _: Rc<Stmt>, _stmt: &WhileStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_break_stmt(&self, _: Rc<Stmt>, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Ok(())
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_assign_expr(&self, _: Rc<Expr>, _expr: &AssignExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_literal_expr(&self, _: Rc<Expr>, _expr: &LiteralExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_binary_expr(&self, _: Rc<Expr>, _expr: &BinaryExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_call_expr(&self, _: Rc<Expr>, _expr: &CallExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_grouping_expr(&self, _: Rc<Expr>, _expr: &GroupingExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_logical_expr(&self, _: Rc<Expr>, _expr: &LogicalExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_unary_expr(&self, _: Rc<Expr>, _expr: &UnaryExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_variable_expr(&self, base: Rc<Expr>, expr: &VariableExpr) -> Result<(), LoxResult> {
        // Disallow shadowing variable in its own initializer (var a = a;)
        if !self.scopes.borrow().is_empty()
            && !self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.lexeme)
                .unwrap()
                == false
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
