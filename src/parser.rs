use crate::lexer::Token;
use crate::error::CompileError;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Variable(u8), // braincell index
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        arg: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(u8, Expr), // braincell index, expression
    Print(Expr),
    If {
        condition: Expr,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
}

pub struct Parser<'a> {
    tokens: Vec<Token>,
    position: usize,
    filename: &'a str,
}

pub fn parse(tokens: Vec<Token>, filename: &str) -> Result<Vec<Statement>, CompileError> {
    let mut parser = Parser::new(tokens, filename);
    parser.parse_program()
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>, filename: &'a str) -> Self {
        Self {
            tokens,
            position: 0,
            filename,
        }
    }
    
    fn parse_program(&mut self) -> Result<Vec<Statement>, CompileError> {
        // Must start with LOCK IN
        if !self.consume(Token::Lock)? || !self.consume(Token::In)? {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "program must start with LOCK IN",
            ));
        }
        
        let mut statements = Vec::new();
        
        while !self.check(&Token::Its) {
            statements.push(self.parse_statement()?);
        }
        
        // Must end with ITS OVER
        if !self.consume(Token::Its)? || !self.consume(Token::Over)? {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "program must end with ITS OVER",
            ));
        }
        
        if !self.check(&Token::Eof) {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "unexpected token after ITS OVER",
            ));
        }
        
        Ok(statements)
    }
    
    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        if self.consume(Token::Fanumtax)? {
            // FANUMTAX <braincell> FR <expr>
            let Token::Braincell(cell_idx) = self.expect_braincell()? else {
                unreachable!();
            };
            
            if !self.consume(Token::Fr)? {
                return Err(CompileError::new(
                    self.filename,
                    self.get_line(),
                    self.get_col(),
                    "expected FR after braincell",
                ));
            }
            
            let expr = self.parse_expression()?;
            Ok(Statement::Assign(cell_idx, expr))
        } else if self.consume(Token::Say)? {
            // SAY <expr>
            let expr = self.parse_expression()?;
            Ok(Statement::Print(expr))
        } else if self.consume(Token::Ongod)? {
            // ONGOD <expr> ... (NO CAP ...)? DEADASS
            self.parse_if()
        } else if self.consume(Token::Skibidi)? {
            // SKIBIDI <expr> ... RIZZUP
            self.parse_while()
        } else {
            Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "expected statement",
            ))
        }
    }
    
    fn parse_if(&mut self) -> Result<Statement, CompileError> {
        // ONGOD <expr> ... (NO CAP ...)? DEADASS
        let condition = self.parse_expression()?;
        
        let mut then_block = Vec::new();
        while !matches!(self.current_token(), Some(Token::No | Token::Deadass)) {
            then_block.push(self.parse_statement()?);
        }
        
        let else_block = if self.consume(Token::No)? {
            if !self.consume(Token::Cap)? {
                return Err(CompileError::new(
                    self.filename,
                    self.get_line(),
                    self.get_col(),
                    "expected CAP after NO",
                ));
            }
            
            let mut else_stmt = Vec::new();
            while !matches!(self.current_token(), Some(Token::Deadass)) {
                else_stmt.push(self.parse_statement()?);
            }
            Some(else_stmt)
        } else {
            None
        };
        
        if !self.consume(Token::Deadass)? {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "expected DEADASS to close ONGOD block",
            ));
        }
        
        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }
    
    fn parse_while(&mut self) -> Result<Statement, CompileError> {
        // SKIBIDI <expr> ... RIZZUP
        let condition = self.parse_expression()?;
        
        let mut body = Vec::new();
        while !matches!(self.current_token(), Some(Token::Rizzup)) {
            body.push(self.parse_statement()?);
        }
        
        if !self.consume(Token::Rizzup)? {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "expected RIZZUP to close SKIBIDI block",
            ));
        }
        
        Ok(Statement::While {
            condition,
            body,
        })
    }
    
    fn parse_expression(&mut self) -> Result<Expr, CompileError> {
        self.parse_binary_expression(0)
    }
    
    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expr, CompileError> {
        let mut expr = self.parse_term()?;
        
        loop {
            let op = self.current_binary_op();
            let Some((op, precedence)) = op else {
                break;
            };
            
            if precedence < min_precedence {
                break;
            }
            
            self.advance();
            let rhs = self.parse_binary_expression(precedence + 1)?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(rhs),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_term(&mut self) -> Result<Expr, CompileError> {
        let token = self.current_token().cloned();
        match token {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expr::String(s))
            }
            Some(Token::Braincell(idx)) => {
                self.advance();
                Ok(Expr::Variable(idx))
            }
            Some(Token::Touchy) => {
                self.advance();
                self.parse_function_call("TOUCHY")
            }
            _ => Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "expected expression",
            )),
        }
    }
    
    fn parse_function_call(&mut self, name: &str) -> Result<Expr, CompileError> {
        // Expect opening parenthesis
        if !matches!(self.current_token(), Some(Token::LParen)) {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                &format!("expected '(' after {}", name),
            ));
        }
        self.advance();
        
        // Parse optional argument
        let arg = if matches!(self.current_token(), Some(Token::RParen)) {
            None
        } else {
            let expr = self.parse_expression()?;
            Some(Box::new(expr))
        };
        
        // Expect closing parenthesis
        if !matches!(self.current_token(), Some(Token::RParen)) {
            return Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                &format!("expected ')' after {} argument", name),
            ));
        }
        self.advance();
        
        Ok(Expr::FunctionCall {
            name: name.to_string(),
            arg,
        })
    }
    
    fn current_binary_op(&self) -> Option<(BinaryOp, u8)> {
        match self.current_token()? {
            Token::Add => Some((BinaryOp::Add, 1)),         // 💀
            Token::Subtract => Some((BinaryOp::Subtract, 1)), // 😭
            Token::Multiply => Some((BinaryOp::Multiply, 2)), // 😏
            Token::Divide => Some((BinaryOp::Divide, 2)),   // 🚡
            _ => None,
        }
    }
    
    fn expect_braincell(&mut self) -> Result<Token, CompileError> {
        if let Some(Token::Braincell(_)) = self.current_token() {
            let token = self.current_token().unwrap().clone();
            self.advance();
            Ok(token)
        } else {
            Err(CompileError::new(
                self.filename,
                self.get_line(),
                self.get_col(),
                "expected braincell name",
            ))
        }
    }
    
    fn consume(&mut self, expected: Token) -> Result<bool, CompileError> {
        if self.check(&expected) {
            self.advance();
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn check(&self, expected: &Token) -> bool {
        self.current_token().map_or(false, |t| std::mem::discriminant(t) == std::mem::discriminant(expected))
    }
    
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    
    fn get_line(&self) -> usize {
        1 // Simplified for now
    }
    
    fn get_col(&self) -> usize {
        1 // Simplified for now
    }
}

