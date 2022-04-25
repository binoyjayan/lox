use crate::error::*;
use crate::object::*;
use crate::token::*;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and".into(), TokenType::And);
        m.insert("class".into(), TokenType::Class);
        m.insert("else".into(), TokenType::Else);
        m.insert("false".into(), TokenType::False);
        m.insert("for".into(), TokenType::For);
        m.insert("fun".into(), TokenType::Fun);
        m.insert("if".into(), TokenType::If);
        m.insert("nil".into(), TokenType::Nil);
        m.insert("or".into(), TokenType::Or);
        m.insert("print".into(), TokenType::Print);
        m.insert("return".into(), TokenType::Return);
        m.insert("super".into(), TokenType::Super);
        m.insert("this".into(), TokenType::This);
        m.insert("true".into(), TokenType::True);
        m.insert("var".into(), TokenType::Var);
        m.insert("while".into(), TokenType::While);
        m.insert("break".into(), TokenType::Break);
        m
    };
}

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            col: 0,
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn scan_token(&mut self) -> Result<(), LoxResult> {
        let c = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            ',' => self.add_token(TokenType::Comma, None),
            '.' => self.add_token(TokenType::Dot, None),
            '-' => self.add_token(TokenType::Minus, None),
            '+' => self.add_token(TokenType::Plus, None),
            ';' => self.add_token(TokenType::Semicolon, None),
            '*' => self.add_token(TokenType::Star, None),
            '!' => self.add_token_twin('=', TokenType::BangEqual, TokenType::Bang),
            '=' => self.add_token_twin('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.add_token_twin('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.add_token_twin('=', TokenType::GreaterEqual, TokenType::Greater),
            '/' => self.handle_slash()?,
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line += 1;
                self.col = 0;
            }
            '"' => self.handle_string()?,
            _ => self.handle_longer_lexemes(c)?,
        }
        Ok(())
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, LoxResult> {
        let mut had_error: Option<LoxResult> = None;

        while !self.is_at_end() {
            // We are at the beginning of the next lexeme
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => {}
                Err(e) => {
                    // Error is already reported
                    had_error = Some(e);
                }
            }
        }

        self.start = self.current;

        self.add_token(TokenType::Eof, None);
        if let Some(e) = had_error {
            return Err(e);
        }
        Ok(self.tokens.clone())
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.col += 1;
        self.source[self.current - 1]
    }

    // Generic function for all tokens. Should have wrapper around it for identifier, numeric, etc
    fn add_token(&mut self, ttype: TokenType, literal: Option<Object>) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.tokens
            .push(Token::new(ttype, lexeme, literal, self.line, self.col));
    }

    // Add two character tokens [ == != ]
    fn add_token_twin(&mut self, next: char, twin_type: TokenType, single_type: TokenType) {
        let matches_second = self.matches(next);
        self.add_token(
            if matches_second {
                twin_type
            } else {
                single_type
            },
            None,
        );
    }

    // Handle slash character separately since it can be comment or a division operator
    fn handle_slash(&mut self) -> Result<(), LoxResult> {
        if self.matches('/') {
            while self.peek() != '\n' && !self.is_at_end() {
                self.advance();
            }
        } else if self.matches('*') {
            self.scan_comment()?;
        } else {
            self.add_token(TokenType::Slash, None)
        }
        Ok(())
    }

    fn scan_comment(&mut self) -> Result<(), LoxResult> {
        while !self.is_at_end() {
            match self.peek() {
                '*' => {
                    self.advance();
                    if self.matches('/') {
                        return Ok(());
                    }
                }
                '/' => {
                    self.advance();
                    if self.matches('*') {
                        // Handle nested comments
                        self.scan_comment()?;
                    }
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                }
                _ => {
                    self.advance();
                }
            }
        }
        // Reached the end without finding a matching '*/'
        Err(LoxResult::error(
            self.line,
            self.col,
            "Unterminated block comment",
        ))
    }

    fn handle_string(&mut self) -> Result<(), LoxResult> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LoxResult::error(self.line, self.col, "Unterminated string"));
        }

        self.advance();

        let s = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token(TokenType::StringLiteral, Some(Object::Str(s)));
        Ok(())
    }

    fn handle_longer_lexemes(&mut self, c: char) -> Result<(), LoxResult> {
        if c.is_digit(10) {
            self.handle_number()
        } else if Self::is_alphabetic(c) {
            self.handle_identifier()
        } else {
            return Err(LoxResult::error(
                self.line,
                self.col,
                &format!("scanner can't handle {}", c),
            ));
        }
        Ok(())
    }

    fn handle_number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the "."
            self.advance();
        }

        while self.peek().is_digit(10) {
            self.advance();
        }

        let s: String = self.source[self.start..self.current].iter().collect();
        let val: f64 = s.parse().unwrap();
        self.add_token(TokenType::Number, Some(Object::Number(val)))
    }

    fn handle_identifier(&mut self) {
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let val: String = self.source[self.start..self.current].iter().collect();

        let (ttype, literal) = match KEYWORDS.get(&val) {
            Some(kw_ttype) => (*kw_ttype, None),
            None => (TokenType::Identifier, Some(Object::Identifier(val))),
        };

        self.add_token(ttype, literal)
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }
        // if expected, advance
        self.current += 1;
        self.col += 1;
        true
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    // lookahead - similar to advance but does not consume the character
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn is_alphabetic(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}
