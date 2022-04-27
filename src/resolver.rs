use crate::error::*;
use crate::expr::*;
use crate::interpreter::*;
use crate::stmt::*;
use crate::token::*;
use std::cell::RefCell;
use std::collections::HashMap;

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
    fn resolve(&self, stmts: &[Stmt]) -> Result<(), LoxResult> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }
    fn resolve_stmt(&self, stmt: &Stmt) -> Result<(), LoxResult> {
        stmt.accept(self)
    }
    fn resolve_expr(&self, expr: &Expr) -> Result<(), LoxResult> {
        expr.accept(self)
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
    fn resolve_local(&self, _expr: &VariableExpr, name: &Token) {
        for (_scope, map) in self.scopes.borrow().iter().rev().enumerate() {
            if map.borrow().contains_key(&name.lexeme) {
                // self.interpreter.resolve_var(expr, scope)
            }
        }
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_block_stmt(&self, stmt: &BlockStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }
    fn visit_expression_stmt(&self, _stmt: &ExpressionStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_function_stmt(&self, _stmt: &FunctionStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_if_stmt(&self, _stmt: &IfStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_print_stmt(&self, _stmt: &PrintStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_return_stmt(&self, _stmt: &ReturnStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<(), LoxResult> {
        self.declare(&stmt.name)?;
        if let Some(init) = &stmt.initializer {
            self.resolve_expr(&init)?;
        }
        self.define(&stmt.name)?;
        Ok(())
    }
    fn visit_while_stmt(&self, _stmt: &WhileStmt) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_break_stmt(&self, _stmt: &BreakStmt) -> Result<(), LoxResult> {
        Ok(())
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_assign_expr(&self, _expr: &AssignExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_literal_expr(&self, _expr: &LiteralExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_binary_expr(&self, _expr: &BinaryExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_call_expr(&self, _expr: &CallExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_grouping_expr(&self, _expr: &GroupingExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_logical_expr(&self, _expr: &LogicalExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_unary_expr(&self, _expr: &UnaryExpr) -> Result<(), LoxResult> {
        Ok(())
    }
    fn visit_variable_expr(&self, expr: &VariableExpr) -> Result<(), LoxResult> {
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
            self.resolve_local(expr, &expr.name);
            Ok(())
        }
    }
}
