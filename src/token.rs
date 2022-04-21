use std::fmt;
use crate::object::*;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
  
    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
  
    // Literals.
    Identifier, StringLiteral, Number,
  
    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,
  
    Eof,
 }

 
 #[derive(Clone, PartialEq)]
 pub struct Token {
     pub ttype: TokenType,
     pub lexeme: String,
     pub literal: Option<Object>,
     pub line: usize,
     pub col: usize,
 }
 
 impl Token {
     pub fn new(ttype: TokenType, lexeme: String, literal: Option<Object>, line: usize, col: usize) -> Token {
         Token {
             ttype,
             lexeme,
             literal,
             line,
             col,
        }
     }
 }

 impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let literal = match &self.literal {
            Some(l) => format!("{}", l),
            None => "None".to_string(),
        };
        write!(
            f,
            "Token {{ ty: {:?}, lexeme: '{}', literal: {}, line: {:?}, }}",
            self.ttype,
            self.lexeme,
            literal,
            self.line,
        )
    }
 }

 impl fmt::Debug for Token {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(
             f,
             "Token {{ ty: {:?}, lexeme: '{}', literal: {:?}, line: {:?}, }}",
             self.ttype,
             self.lexeme,
             self.literal,
             self.line,
         )
     }
 }
 
