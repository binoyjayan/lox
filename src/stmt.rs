// This is an autogenerated file. Do not edit manually. Use gen-ast package.
// Use gen-ast package to generate this file.

use crate::error::*;
use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Expression(ExpressionStmt),
    Print(PrintStmt),
    Var(VarStmt),
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &dyn StmtVisitor<T>) -> Result<T, LoxErr> {
        match self {
            Stmt::Expression(exp) => exp.accept(visitor),
            Stmt::Print(exp) => exp.accept(visitor),
            Stmt::Var(exp) => exp.accept(visitor),
        }
    }

}

pub struct ExpressionStmt {
    pub expression: Box<Expr>,
}

pub struct PrintStmt {
    pub expression: Box<Expr>,
}

pub struct VarStmt {
    pub name: Token,
    pub initializer: Option<Expr>,
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&self, stmt: &ExpressionStmt) -> Result<T, LoxErr>;
    fn visit_print_stmt(&self, stmt: &PrintStmt) -> Result<T, LoxErr>;
    fn visit_var_stmt(&self, stmt: &VarStmt) -> Result<T, LoxErr>;
}

impl ExpressionStmt {
    pub fn accept<T>(&self, visitor: &dyn StmtVisitor<T>) -> Result<T, LoxErr> {
        visitor.visit_expression_stmt(self)
    }
}

impl PrintStmt {
    pub fn accept<T>(&self, visitor: &dyn StmtVisitor<T>) -> Result<T, LoxErr> {
        visitor.visit_print_stmt(self)
    }
}

impl VarStmt {
    pub fn accept<T>(&self, visitor: &dyn StmtVisitor<T>) -> Result<T, LoxErr> {
        visitor.visit_var_stmt(self)
    }
}

