use crate::error::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Lock,
    In,
    Its,
    Over,
    Fanumtax,
    Fr,
    Say,
    Touchy,
    Ongod,   // if
    No,      // else (part 1)
    Cap,     // else (part 2)
    Deadass, // end if
    Skibidi, // while
    Rizzup,  // end while
    
    // Operators
    Add,      // 💀
    Subtract, // 😭
    Multiply, // 😏
    Divide,   // 🚡
    
    // Braincells
    Braincell(u8), // 0=aura, 1=peak, 2=goon, 3=mog, 4=npc, 5=sigma, 6=gyatt
    
    // Literals
    Number(f64),
    String(String),
    
    // Punctuation
    LParen,
    RParen,
    
    // Special
    Eof,
}

pub struct Lexer<'a> {
    chars: Vec<(usize, usize, char)>, // (byte_offset, char_index, char)
    position: usize,
    line: usize,
    col: usize,
    filename: &'a str,
}

const BRAINCELLS: &[(&str, u8)] = &[
    ("aura", 0),
    ("peak", 1),
    ("goon", 2),
    ("mog", 3),
    ("npc", 4),
    ("sigma", 5),
    ("gyatt", 6),
];

pub fn tokenize(input: &str, filename: &str) -> Result<Vec<Token>, CompileError> {
    let chars: Vec<(usize, usize, char)> = input.char_indices()
        .enumerate()
        .map(|(idx, (byte_pos, ch))| (byte_pos, idx, ch))
        .collect();
    
    let mut lexer = Lexer::new(chars, filename);
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token()?;
        let is_eof = matches!(token, Token::Eof);
        tokens.push(token);
        if is_eof {
            break;
        }
    }
    
    Ok(tokens)
}

impl<'a> Lexer<'a> {
    fn new(chars: Vec<(usize, usize, char)>, filename: &'a str) -> Self {
        Self {
            chars,
            position: 0,
            line: 1,
            col: 1,
            filename,
        }
    }
    
    fn next_token(&mut self) -> Result<Token, CompileError> {
        self.skip_whitespace();
        
        if self.position >= self.chars.len() {
            return Ok(Token::Eof);
        }
        
        let (_, _, ch) = self.current_char();
        
        // Check for comment line
        if ch == '🖕' {
            self.skip_line();
            return self.next_token();
        }
        
        // Operators
        if ch == '💀' {
            self.advance();
            return Ok(Token::Add);
        }
        if ch == '😭' {
            self.advance();
            return Ok(Token::Subtract);
        }
        if ch == '😏' {
            self.advance();
            return Ok(Token::Multiply);
        }
        if ch == '🚡' {
            self.advance();
            return Ok(Token::Divide);
        }
        
        // String literal
        if ch == '"' {
            return self.read_string();
        }
        
        // Parentheses
        if ch == '(' {
            self.advance();
            return Ok(Token::LParen);
        }
        if ch == ')' {
            self.advance();
            return Ok(Token::RParen);
        }
        
        // Number
        if ch.is_ascii_digit() {
            return self.read_number();
        }
        
        // Identifier
        if ch.is_ascii_alphabetic() {
            return self.read_identifier();
        }
        
        Err(CompileError::new(
            self.filename,
            self.line,
            self.col,
            &format!("unexpected character: {}", ch),
        ))
    }
    
    fn read_string(&mut self) -> Result<Token, CompileError> {
        self.advance(); // skip opening "
        let mut result = String::new();
        
        while self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                self.advance();
                if self.position >= self.chars.len() {
                    return Err(CompileError::new(
                        self.filename,
                        self.line,
                        self.col,
                        "unexpected end of string",
                    ));
                }
                let (_, _, escaped_ch) = self.current_char();
                let escaped = match escaped_ch {
                    'n' => '\n',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    c => return Err(CompileError::new(
                        self.filename,
                        self.line,
                        self.col,
                        &format!("unknown escape sequence: \\{}", c),
                    )),
                };
                result.push(escaped);
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }
        
        if self.position >= self.chars.len() {
            return Err(CompileError::new(
                self.filename,
                self.line,
                self.col,
                "unterminated string",
            ));
        }
        
        self.advance(); // skip closing "
        Ok(Token::String(result))
    }
    
    fn read_number(&mut self) -> Result<Token, CompileError> {
        let mut num_str = String::new();
        
        while self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if !ch.is_ascii_digit() {
                break;
            }
            num_str.push(ch);
            self.advance();
        }
        
        if self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if ch == '.' {
                num_str.push('.');
                self.advance();
                while self.position < self.chars.len() {
                    let (_, _, ch) = self.current_char();
                    if !ch.is_ascii_digit() {
                        break;
                    }
                    num_str.push(ch);
                    self.advance();
                }
            }
        }
        
        let num = num_str.parse::<f64>().unwrap();
        Ok(Token::Number(num))
    }
    
    fn read_identifier(&mut self) -> Result<Token, CompileError> {
        let mut ident = String::new();
        
        while self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if !ch.is_ascii_alphabetic() {
                break;
            }
            ident.push(ch);
            self.advance();
        }
        
        // Check if it's a keyword
        match ident.as_str() {
            "LOCK" => return Ok(Token::Lock),
            "IN" => return Ok(Token::In),
            "ITS" => return Ok(Token::Its),
            "OVER" => return Ok(Token::Over),
            "FANUMTAX" => return Ok(Token::Fanumtax),
            "FR" => return Ok(Token::Fr),
            "SAY" => return Ok(Token::Say),
            "TOUCHY" => return Ok(Token::Touchy),
            "ONGOD" => return Ok(Token::Ongod),
            "NO" => return Ok(Token::No),
            "CAP" => return Ok(Token::Cap),
            "DEADASS" => return Ok(Token::Deadass),
            "SKIBIDI" => return Ok(Token::Skibidi),
            "RIZZUP" => return Ok(Token::Rizzup),
            _ => {}
        }
        
        // Check if it's a braincell
        for (name, idx) in BRAINCELLS {
            if ident == *name {
                return Ok(Token::Braincell(*idx));
            }
        }
        
        Err(CompileError::new(
            self.filename,
            self.line,
            self.col,
            &format!("unknown identifier: {}", ident),
        ))
    }
    
    fn skip_whitespace(&mut self) {
        while self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }
    
    fn skip_line(&mut self) {
        while self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }
    
    fn current_char(&self) -> (usize, usize, char) {
        self.chars[self.position]
    }
    
    fn advance(&mut self) {
        if self.position < self.chars.len() {
            let (_, _, ch) = self.current_char();
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.position += 1;
    }
}
