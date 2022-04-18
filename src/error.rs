#[derive(Debug)]
pub struct LoxErr {
    line: usize,
    message: String,
}

impl LoxErr {
    pub fn error(line: usize, message: String) -> LoxErr {
        LoxErr {line, message}
    }

    pub fn report(&self, loc: String) {
        eprintln!("[line{}] Error {}: {}", self.line, loc, self.message);
    }
}
