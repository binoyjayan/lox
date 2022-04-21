use crate::token::*;

#[derive(Debug)]
pub struct LoxErr {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl LoxErr {
    pub fn error(line: usize, col: usize, message: String) -> LoxErr {
        let err = LoxErr {line, col, message};
        err.report("".to_string());
        err
    }

    pub fn report(&self, loc: String) {
        eprintln!("[line {} col {}] Error {}: {}", self.line, self.col, loc, self.message);
    }

    pub fn error_at_token(token: &Token, message: String) -> LoxErr {
        let err = if token.ttype == TokenType::Eof {
            LoxErr { line: token.line, col: token.col, message: format!(" at end - {}", message) }
        } else {
            LoxErr {line: token.line, col: token.col, message: format!(" at {} - {}", token.lexeme, message) }
        };
        err.report("".to_string());
        err
    }
}
