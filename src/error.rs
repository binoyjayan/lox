#[derive(Debug)]
pub struct LoxErr {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl LoxErr {
    pub fn error(line: usize, col: usize, message: String) -> LoxErr {
        LoxErr {line, col, message}
    }

    pub fn report(&self, loc: String) {
        eprintln!("[line {} col {}] Error {}: {}", self.line, self.col, loc, self.message);
    }
}
