use crate::expr::*;
use crate::token::*;
use crate::error::*;
use crate::object::*;
use crate::scanner::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
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
        Self { tokens, current: 0}
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => None,
        }
    }

    fn expression(&mut self) -> Result<Expr, LoxErr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, LoxErr> {
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
            expr = Expr::Binary(BinaryExpr { left: Box::new(expr), operator, right: Box::new(right)});
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.term()?;
        let compare_operators = [
            TokenType::Greater, TokenType::GreaterEqual,
            TokenType::Less, TokenType::LessEqual
        ];
        while self.matches(&compare_operators) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(BinaryExpr {left: Box::new(expr), operator, right: Box::new(right)});
        }
        Ok(expr)
    }

    // In order of precedence, first addition and subtraction
    fn term(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.factor()?;
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(BinaryExpr{left: Box::new(expr), operator, right: Box::new(right)});
        }
        Ok(expr)
    }

    // Multiplication and then division
    fn factor(&mut self) -> Result<Expr, LoxErr> {
        let mut expr = self.unary()?;
        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(BinaryExpr{left: Box::new(expr), operator, right: Box::new(right)});
        }
        Ok(expr)
    }

    // If encountered a unary operator, recursively call unary 
    // recursively again to parse the expression.
    fn unary(&mut self) -> Result<Expr, LoxErr> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(UnaryExpr { operator, right: Box::new(right)}))
        }
        self.primary()
        // self.call()
    }

    // Reached highest level of precedence after crawling up the
    // precedence hierarchy. Most of the primary rules are terminals.
    fn primary(&mut self) -> Result<Expr, LoxErr> {
        if self.matches(&[TokenType::False]) {
            return Ok(Expr::Literal(LiteralExpr{value: Some(Object::Bool(false))}));
        }
        if self.matches(&[TokenType::True]) {
            return Ok(Expr::Literal(LiteralExpr{value: Some(Object::Bool(true))}));
        }
        if self.matches(&[TokenType::Nil]) {
            return Ok(Expr::Literal(LiteralExpr{value: Some(Object::Nil)}));
        }
        if self.matches(&[TokenType::Number, TokenType::StringLiteral]) {
            return Ok(Expr::Literal(
                LiteralExpr {
                    value: match self.previous().literal {
                        Some(l) => Some(l),
                        None => Some(Object::Nil),
                    }
            }));
        }
        // if self.matches(&[TokenType::Identifier]) {
        //     return Ok(Expr::Variable(self.previous()));
        // }
        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect `)` after expression")?;
            return Ok(Expr::Grouping(GroupingExpr { expression: Box::new(expr)}));
        }
        // Encountered a token that can’t start an expression.
        Err(LoxErr::error_at_token(&self.peek(), "Expression expected"))
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
                TokenType::Class | 
                TokenType::Fun | 
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
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
    fn consume(&mut self, ttype: &TokenType, message: &str) -> Result<Token, LoxErr> {
        if self.check(ttype) {
            Ok(self.advance())
        } else {
            Err(LoxErr::error_at_token(&self.peek(), message))
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
