use crate::token::*;

#[derive(Debug)]
pub enum LoxResult {
    ParseError {
        token: Token,
        message: String,
    },
    RuntimeError {
        token: Token,
        message: String,
    },
    SystemError {
        message: String,
    },
    Error {
        line: usize,
        col: usize,
        message: String,
    },
    Break,
}

impl LoxResult {
    fn report(&self, loc: &str) {
        match self {
            LoxResult::ParseError { token, message }
            | LoxResult::RuntimeError { token, message } => {
                eprintln!(
                    "[line {} col {}] Error{}: {}",
                    token.line, token.col, loc, message
                );
            }
            LoxResult::SystemError { message } => {
                eprintln!("Error{}: {}", loc, message);
            }
            LoxResult::Error { line, col, message } => {
                eprintln!("[line {} col {}] Error{}: {}", line, col, loc, message);
            }
            LoxResult::Break => {}
        }
    }

    pub fn error(line: usize, col: usize, message: &str) -> LoxResult {
        let err = LoxResult::Error {
            line,
            col,
            message: message.to_string(),
        };
        err.report("");
        err
    }

    pub fn error_at_token(token: &Token, message: &str) -> LoxResult {
        let loc = if token.ttype == TokenType::Eof {
            "at eof".to_string()
        } else {
            format!(" at '{}'", token.lexeme)
        };
        let err = LoxResult::ParseError {
            message: message.to_string(),
            token: token.clone(),
        };
        err.report(&loc);
        err
    }

    pub fn error_runtime(token: &Token, message: &str) -> LoxResult {
        let err = LoxResult::RuntimeError {
            message: message.to_string(),
            token: token.clone(),
        };
        err.report(&format!(" at '{}'", token.lexeme));
        err
    }

    pub fn system_error(message: &str) -> LoxResult {
        let err = LoxResult::SystemError {
            message: message.to_string(),
        };
        err.report("");
        err
    }
}
