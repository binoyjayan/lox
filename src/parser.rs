use crate::error::*;
use crate::expr::*;
use crate::object::*;
use crate::stmt::*;
use crate::token::*;
use std::rc::Rc;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    had_error: bool,
}

// Grammar for expressions:

// When the first symbol in the body of the rule is the same as
// the head of the rule means that the production is left-recursive
// Use left recursive production rules for left-associative operations,
// and right recursive rules for right-associative operations.

// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                  | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                  | "(" expression ")" ;
//
// Terminal	       Code to match and consume a token
// Nonterminal	   Call to that rule’s function
// |               if or switch statement
// * or +          while or for loop
// ?               if statement

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            had_error: false,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Rc<Stmt>>, LoxResult> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            // Ignore error returned by 'declaration' so it does not get thrown
            // from this function thereby aborting it execution. Instead continue
            // parsing since we already synchronized.
            if let Ok(stmt) = self.declaration() {
                stmts.push(stmt)
            }
        }
        Ok(stmts)
    }

    fn declaration(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let result = if self.matches(&[TokenType::Fun]) {
            self.fun_declaration("function")
        } else if self.matches(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn var_declaration(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.matches(&[TokenType::Equal]) {
            Some(Rc::new(self.expression()?))
        } else {
            None
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ',' after variable declaration",
        )?;
        Ok(Rc::new(Stmt::Var(Rc::new(VarStmt { name, initializer }))))
    }

    fn fun_declaration(&mut self, kind: &str) -> Result<Rc<Stmt>, LoxResult> {
        let mut params = Vec::new();
        let name = self.consume(&TokenType::Identifier, &format!("Expect '{}' name.", kind))?;
        self.consume(
            &TokenType::LeftParen,
            &format!("Expect '(' after '{}' name.", kind),
        )?;

        if !self.check(&TokenType::RightParen) {
            params.push(self.consume(&TokenType::Identifier, "Expect parameter name")?);
            while self.matches(&[TokenType::Comma]) {
                if params.len() >= 255 {
                    self.parse_error(&self.peek(), "Can't have more than 255 parameters");
                } else {
                    params.push(self.consume(&TokenType::Identifier, "Expect parameter name")?);
                }
            }
        }
        self.consume(&TokenType::RightParen, "Expect ')' after parameters")?;

        // Parse function body
        self.consume(
            &TokenType::LeftBrace,
            &format!("Expect '{{' before '{}' body", kind),
        )?;
        let body = self.block()?;
        Ok(Rc::new(Stmt::Function(Rc::new(FunctionStmt {
            name,
            params: Rc::new(params),
            body: Rc::new(body),
        }))))
    }

    fn statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matches(&[TokenType::If]) {
            return Ok(Rc::new(self.if_statement()?));
        }
        if self.matches(&[TokenType::Print]) {
            return Ok(Rc::new(self.print_statement()?));
        }
        if self.matches(&[TokenType::Return]) {
            return Ok(Rc::new(self.return_statement()?));
        }
        if self.matches(&[TokenType::While]) {
            return Ok(Rc::new(self.while_statement()?));
        }
        if self.matches(&[TokenType::Break]) {
            return Ok(Rc::new(self.break_statement()?));
        }
        if self.matches(&[TokenType::LeftBrace]) {
            return Ok(Rc::new(Stmt::Block(Rc::new(BlockStmt {
                statements: Rc::new(self.block()?),
            }))));
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = Rc::new(self.expression()?);
        self.consume(&TokenType::RightParen, "Expect ')' after 'if'.")?;
        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };
        Ok(Stmt::If(Rc::new(IfStmt {
            condition,
            then_branch,
            else_branch,
        })))
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxResult> {
        let value = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Rc::new(PrintStmt {
            expression: Rc::new(value),
        })))
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxResult> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(&TokenType::Semicolon) {
            value = Some(Rc::new(self.expression()?));
        }
        self.consume(&TokenType::Semicolon, "Expect ';' after return statement.")?;
        Ok(Stmt::Return(Rc::new(ReturnStmt { keyword, value })))
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after while condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While(Rc::new(WhileStmt {
            condition: Rc::new(condition),
            body,
        })))
    }

    fn break_statement(&mut self) -> Result<Stmt, LoxResult> {
        let token = self.previous();
        self.consume(&TokenType::Semicolon, "Expect ';' after break statement.")?;

        Ok(Stmt::Break(Rc::new(BreakStmt { token })))
    }

    fn for_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.")?;
        // Parse optional 'initializer'
        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        // Parse optional condition
        let condition = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after for loop condition.",
        )?;

        // Parse optional 'increment' or 'step'
        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::RightParen, "Expect ')' after for loop.")?;
        let mut body = self.statement()?;

        // Build a block statement
        if let Some(inc) = increment {
            body = Rc::new(Stmt::Block(Rc::new(BlockStmt {
                statements: Rc::new(vec![
                    body,
                    Rc::new(Stmt::Expression(Rc::new(ExpressionStmt {
                        expression: Rc::new(inc),
                    }))),
                ]),
            })))
        }
        // Force condition to true if not specified
        let condition = if let Some(cond) = condition {
            cond
        } else {
            Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Bool(true)),
            }))
        };
        // Build a while statement with the condition and body
        body = Rc::new(Stmt::While(Rc::new(WhileStmt {
            condition: Rc::new(condition),
            body,
        })));

        // Build a block statement around the while statement to add initializer
        if let Some(init) = initializer {
            body = Rc::new(Stmt::Block(Rc::new(BlockStmt {
                statements: Rc::new(vec![init, body]),
            })));
        }
        Ok(body)
    }

    fn expression_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Rc::new(Stmt::Expression(Rc::new(ExpressionStmt {
            expression: Rc::new(expr),
        }))))
    }

    fn block(&mut self) -> Result<Vec<Rc<Stmt>>, LoxResult> {
        let mut stmts = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn expression(&mut self) -> Result<Expr, LoxResult> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxResult> {
        let expr = self.logical_or()?;
        if self.matches(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            if let Expr::Variable(expr) = expr {
                return Ok(Expr::Assign(Rc::new(AssignExpr {
                    name: expr.name.clone(),
                    value: Rc::new(value),
                })));
            }
            // Report but do not throw the error because the parser
            // does not need to panic and synchronize
            self.parse_error(&equals, "Invalid assignment  target.");
        }
        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.logical_and()?;
        while self.matches(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.logical_and()?;
            expr = Expr::Logical(Rc::new(LogicalExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.equality()?;
        while self.matches(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Rc::new(LogicalExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.comparison()?;

        // The ( ... )* loop in the rule maps to a while loop.
        // Grab the matched operator token to track which kind
        // of equality expression is available. Then call comparison()
        // again to parse the right-hand operand. Combine the operator
        // and its two operands into a new 'Expr::Binary' syntax
        // tree node, and then loop around. For each iteration, store
        // the resulting expression back in the same expr local variable.
        // By zipping through a sequence of equality expressions, a
        // left-associative nested tree of binary operator is nodes created.
        // If an an equality operator is not found, break the loop.

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.term()?;
        let compare_operators = [
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ];
        while self.matches(&compare_operators) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    // In order of precedence, first addition and subtraction
    fn term(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.factor()?;
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    // Multiplication and then division
    fn factor(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.unary()?;
        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Rc::new(BinaryExpr {
                left: Rc::new(expr),
                operator,
                right: Rc::new(right),
            }));
        }
        Ok(expr)
    }

    // If encountered a unary operator, recursively call unary
    // recursively again to parse the expression.
    fn unary(&mut self) -> Result<Expr, LoxResult> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(Rc::new(UnaryExpr {
                operator,
                right: Rc::new(right),
            })));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(Rc::new(expr))?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    // Process function call arguments and consume the closing parenthesis
    fn finish_call(&mut self, callee: Rc<Expr>) -> Result<Expr, LoxResult> {
        let mut arguments = Vec::new();
        // If there are arguments to the function
        if !self.check(&TokenType::RightParen) {
            arguments.push(Rc::new(self.expression()?));
            while self.matches(&[TokenType::Comma]) {
                if arguments.len() >= 255 {
                    self.parse_error(&self.peek(), "Can't have more than 255 arguments");
                } else {
                    arguments.push(Rc::new(self.expression()?));
                }
            }
        }

        let paren = self.consume(&TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(Expr::Call(Rc::new(CallExpr {
            callee,
            paren,
            arguments,
        })))
    }

    // Reached highest level of precedence after crawling up the
    // precedence hierarchy. Most of the primary rules are terminals.
    fn primary(&mut self) -> Result<Expr, LoxResult> {
        if self.matches(&[TokenType::False]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Bool(false)),
            })));
        }
        if self.matches(&[TokenType::True]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Bool(true)),
            })));
        }
        if self.matches(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: Some(Object::Nil),
            })));
        }
        if self.matches(&[TokenType::Number, TokenType::StringLiteral]) {
            return Ok(Expr::Literal(Rc::new(LiteralExpr {
                value: match self.previous().literal {
                    Some(l) => Some(l),
                    None => Some(Object::Nil),
                },
            })));
        }
        if self.matches(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(Rc::new(VariableExpr {
                name: self.previous(),
            })));
        }
        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect `)` after expression")?;
            return Ok(Expr::Grouping(Rc::new(GroupingExpr {
                expression: Rc::new(expr),
            })));
        }
        // Encountered a token that can’t start an expression.
        Err(self.parse_error(&self.peek(), "Expression expected"))
    }

    // Synchronize the recursive descent parser by discarding
    // token right until the beginning of the next statement
    // i.e. when a semicolon or any of the special keywords is seen.
    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().ttype == TokenType::Semicolon {
                return;
            }
            match self.peek().ttype {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }

    pub fn parse_error(&mut self, token: &Token, message: &str) -> LoxResult {
        self.had_error = true;
        LoxResult::error_at_token(token, message)
    }

    pub fn success(&self) -> bool {
        !self.had_error
    }

    // Check to see if the current token has any of the given types.
    // If so, consume the token and return true
    fn matches(&mut self, types: &[TokenType]) -> bool {
        for ttype in types {
            if self.check(ttype) {
                self.advance();
                return true;
            }
        }
        false
    }

    // Similar to match() in that it checks to see if the next token is
    // of the expected type. If so, it consumes the token.
    // If some other token is encountered, report an error.
    fn consume(&mut self, ttype: &TokenType, message: &str) -> Result<Token, LoxResult> {
        if self.check(ttype) {
            Ok(self.advance())
        } else {
            Err(self.parse_error(&self.peek(), message))
        }
    }

    // Returns true if the current token is of the given type
    fn check(&self, ttype: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().ttype == ttype
    }

    // Consumes the current token and return it
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // check if we have run out of tokens
    fn is_at_end(&self) -> bool {
        self.peek().ttype == TokenType::Eof
    }

    // return the current token yet to be consumed
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    // return the most recently consumed token
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
