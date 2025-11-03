#[derive(Debug, Clone)]
pub struct CompileError {
    pub filename: String,
    pub line: usize,
    pub col: usize,
    pub message: String,
}

impl CompileError {
    pub fn new(filename: &str, line: usize, col: usize, message: &str) -> Self {
        Self {
            filename: filename.to_string(),
            line,
            col,
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}: {}", self.filename, self.line, self.col, self.message)
    }
}

impl std::error::Error for CompileError {}

