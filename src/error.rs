use crate::token::*;

#[derive(Debug)]
pub struct LoxErr {
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub token: Option<Token>
}

impl LoxErr {
    pub fn error(line: usize, col: usize, message: &str) -> LoxErr {
        let err = LoxErr {line, col, message: message.to_string(), token: None};
        // err.report("");
        err
    }

    pub fn report(&self, loc: &str) {
        eprintln!("[line {} col {}] Error {}: {}", self.line, self.col, loc, self.message);
    }

    pub fn error_at_token(token: &Token, message: &str) -> LoxErr {
        let err = if token.ttype == TokenType::Eof {
            LoxErr { line: token.line, col: token.col, message: format!(" at end - {}", message), token: Some(token.clone()) }
        } else {
            LoxErr {line: token.line, col: token.col, message: format!(" at {} - {}", token.lexeme, message), token: Some(token.clone()) }
        };
        // err.report("");
        err
    }

    pub fn error_runtime(token: &Token, message: &str) -> LoxErr {
        let err = if token.ttype == TokenType::Eof {
            LoxErr { line: token.line, col: token.col, message: format!(" at end - {}", message), token: None}
        } else {
            LoxErr {line: token.line, col: token.col, message: format!(" at {} - {}", token.lexeme, message), token: None }
        };
        // err.report("");
        err
    }
}
