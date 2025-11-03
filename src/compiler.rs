use crate::parser::{Expr, Statement, BinaryOp};
use std::collections::HashMap;

pub fn compile(statements: Vec<Statement>) -> Result<Vec<u8>, String> {
    let mut compiler = Compiler::new();
    
    // Compile all statements
    for stmt in statements {
        compiler.compile_statement(&stmt);
    }
    
    // Add HALT at the end
    compiler.emit_op(0x01); // HALT
    
    // Build the bytecode
    compiler.write_bytecode()
}

struct Compiler {
    constants: Vec<Constant>,
    const_map: HashMap<Constant, u32>,
    code: Vec<u8>,
}

#[derive(Debug, Clone)]
enum Constant {
    Number(f64),
    String(Vec<u8>),
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Number(a), Constant::Number(b)) => a == b,
            (Constant::String(a), Constant::String(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Constant {}

impl std::hash::Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Constant::Number(n) => n.to_bits().hash(state),
            Constant::String(s) => s.hash(state),
        }
    }
}

impl Compiler {
    fn new() -> Self {
        Self {
            constants: Vec::new(),
            const_map: HashMap::new(),
            code: Vec::new(),
        }
    }
    
    fn compile_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assign(cell_idx, expr) => {
                self.compile_expr(expr);
                self.emit_op(0x04); // STORE_GLOBAL
                self.emit_u8(*cell_idx);
            }
            Statement::Print(expr) => {
                self.compile_expr(expr);
                self.emit_op(0x09); // PRINT
            }
            Statement::If { condition, then_block, else_block } => {
                self.compile_expr(condition);
                
                // JUMP_IF_FALSE to else/end
                self.emit_op(0x0C); // JUMP_IF_FALSE
                let jump_pos = self.code.len();
                self.emit_u32(0); // placeholder
                
                // Compile then block
                for stmt in then_block {
                    self.compile_statement(stmt);
                }
                
                if else_block.is_some() {
                    // Jump over else block
                    self.emit_op(0x0B); // JUMP to end
                    let jump_end_pos = self.code.len();
                    self.emit_u32(0); // placeholder
                    
                    // Backpatch JUMP_IF_FALSE to else block start
                    let else_start = self.code.len() as u32;
                    self.code[jump_pos..jump_pos+4].copy_from_slice(&else_start.to_le_bytes());
                    
                    // Compile else block
                    for stmt in else_block.as_ref().unwrap() {
                        self.compile_statement(stmt);
                    }
                    
                    // Backpatch JUMP to end
                    let end_pos = self.code.len() as u32;
                    self.code[jump_end_pos..jump_end_pos+4].copy_from_slice(&end_pos.to_le_bytes());
                } else {
                    // Backpatch JUMP_IF_FALSE to end
                    let end_pos = self.code.len() as u32;
                    self.code[jump_pos..jump_pos+4].copy_from_slice(&end_pos.to_le_bytes());
                }
            }
            Statement::While { condition, body } => {
                let loop_start = self.code.len() as u32;
                
                // Compile condition
                self.compile_expr(condition);
                
                // JUMP_IF_FALSE to end
                self.emit_op(0x0C); // JUMP_IF_FALSE
                let jump_pos = self.code.len();
                self.emit_u32(0); // placeholder
                
                // Compile body
                for stmt in body {
                    self.compile_statement(stmt);
                }
                
                // Jump back to start (absolute offset)
                self.emit_op(0x0B); // JUMP
                self.emit_u32(loop_start);
                
                // Backpatch JUMP_IF_FALSE to end
                let end_pos = self.code.len() as u32;
                self.code[jump_pos..jump_pos+4].copy_from_slice(&end_pos.to_le_bytes());
            }
        }
    }
    
    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                let idx = self.add_const(Constant::Number(*n));
                self.emit_op(0x02); // LOAD_CONST
                self.emit_u32(idx);
            }
            Expr::String(s) => {
                let bytes = s.as_bytes().to_vec();
                let idx = self.add_const(Constant::String(bytes));
                self.emit_op(0x02); // LOAD_CONST
                self.emit_u32(idx);
            }
            Expr::Variable(cell_idx) => {
                self.emit_op(0x03); // LOAD_GLOBAL
                self.emit_u8(*cell_idx);
            }
            Expr::Binary { op, left, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                
                let opcode = match op {
                    BinaryOp::Add => 0x05,
                    BinaryOp::Subtract => 0x06,
                    BinaryOp::Multiply => 0x07,
                    BinaryOp::Divide => 0x08,
                };
                self.emit_op(opcode);
            }
            Expr::FunctionCall { name, arg } => {
                if name == "TOUCHY" {
                    if let Some(prompt) = arg {
                        // TOUCHY(<prompt>) - print prompt without newline, then read input
                        self.compile_expr(prompt);
                        self.emit_op(0x0D); // PRINT_NOPLN (moved to 0x0D)
                        self.emit_op(0x0A); // INPUT
                    } else {
                        // TOUCHY() - just read input
                        self.emit_op(0x0A); // INPUT
                    }
                } else {
                    panic!("Unknown function: {}", name);
                }
            }
        }
    }
    
    fn add_const(&mut self, constant: Constant) -> u32 {
        if let Some(&idx) = self.const_map.get(&constant) {
            return idx;
        }
        
        let idx = self.constants.len() as u32;
        self.constants.push(constant.clone());
        self.const_map.insert(constant, idx);
        idx
    }
    
    fn emit_op(&mut self, op: u8) {
        self.code.push(op);
    }
    
    fn emit_u8(&mut self, val: u8) {
        self.code.push(val);
    }
    
    fn emit_u32(&mut self, val: u32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }
    
    fn write_bytecode(&self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        
        // Header: "BRBC" + version (3) + flags (0)
        result.extend_from_slice(b"BRBC");
        result.extend_from_slice(&3u16.to_le_bytes()); // version
        result.extend_from_slice(&0u16.to_le_bytes()); // flags
        
        // Constant pool
        result.extend_from_slice(&(self.constants.len() as u32).to_le_bytes());
        
        for constant in &self.constants {
            match constant {
                Constant::Number(n) => {
                    result.push(1); // tag: Number
                    result.extend_from_slice(&n.to_le_bytes());
                }
                Constant::String(bytes) => {
                    result.push(2); // tag: String
                    result.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    result.extend_from_slice(bytes);
                }
            }
        }
        
        // Code section
        result.extend_from_slice(&(self.code.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.code);
        
        Ok(result)
    }
}

