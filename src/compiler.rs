use crate::parser::{Program, Expr, Statement, BinaryOp, Function};
use std::collections::HashMap;

pub fn compile(program: Program) -> Result<Vec<u8>, String> {
    let mut compiler = Compiler::new();
    
    // First, compile all functions (they go AFTER main in the final bytecode)
    for func in &program.functions {
        compiler.compile_function(func)?;
    }
    
    // Get function code
    let function_code_parts = std::mem::take(&mut compiler.function_code_parts);
    
    // Now compile main statements
    for stmt in &program.main_statements {
        compiler.compile_main_statement(stmt)?;
    }
    
    // Add HALT at the end of main
    compiler.emit_op(0x01); // HALT
    
    // Get main code size before appending functions
    let mut main_code_size = compiler.code.len() as u32;
    
    // Now update function code offsets and append function code
    for (i, func_code) in function_code_parts.into_iter().enumerate() {
        let size = func_code.len() as u32;
        compiler.functions[i].code_offset = main_code_size;
        main_code_size += size; // Track cumulative offset for next function
        
        compiler.code.extend_from_slice(&func_code);
    }
    
    // Build the bytecode
    compiler.write_bytecode()
}

struct FunctionInfo {
    name: String,
    arity: u16,
    local_count: u16,
    code_offset: u32,
}

struct Compiler {
    constants: Vec<Constant>,
    const_map: HashMap<Constant, u32>,
    functions: Vec<FunctionInfo>,
    function_map: HashMap<String, u32>, // name -> function index
    code: Vec<u8>,
    
    // For function compilation
    current_locals: HashMap<String, u16>,
    function_code_parts: Vec<Vec<u8>>, // Store function code separately
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
            functions: Vec::new(),
            function_map: HashMap::new(),
            code: Vec::new(),
            current_locals: HashMap::new(),
            function_code_parts: Vec::new(),
        }
    }
    
    fn compile_function(&mut self, func: &Function) -> Result<(), String> {
        // Save current state
        let saved_code = std::mem::take(&mut self.code);
        let saved_locals = std::mem::take(&mut self.current_locals);
        
        // Ensure function name is in constant pool
        let name_bytes = func.name.as_bytes().to_vec();
        self.add_const(Constant::String(name_bytes));
        
        // Allocate parameters as locals
        for (idx, param) in func.params.iter().enumerate() {
            self.current_locals.insert(param.clone(), idx as u16);
        }
        
        // Compile function body
        for stmt in &func.body {
            self.compile_statement(stmt)?;
        }
        
        // If function doesn't end with RETURN, add default return ""
        // Check if last statement is a Return
        let has_return = matches!(func.body.last(), Some(Statement::Return(_)));
        if !has_return {
            // Emit default return of empty string
            let empty_str = self.add_const(Constant::String(b"".to_vec()));
            self.emit_op(0x02); // LOAD_CONST
            self.emit_u32(empty_str);
            self.emit_op(0x0E); // UNTILWEMEETAGAIN
        } else {
            // Make sure last statement emitted UNTILWEMEETAGAIN
            // (it was already handled in compile_statement)
        }
        
        // Store function code
        let func_code = std::mem::take(&mut self.code);
        // Built-ins are 0 and 1, so user functions start at 2
        let func_index = 2 + self.functions.len() as u32;
        
        self.function_map.insert(func.name.clone(), func_index);
        
        self.functions.push(FunctionInfo {
            name: func.name.clone(),
            arity: func.params.len() as u16,
            local_count: self.current_locals.len() as u16,
            code_offset: 0, // Will be fixed later
        });
        
        // Store function code for later
        self.function_code_parts.push(func_code);
        
        // Restore state
        self.code = saved_code;
        self.current_locals = saved_locals;
        
        Ok(())
    }
    
    fn compile_main_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        self.compile_statement(stmt)
    }
    
    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Assign(var_name, expr) => {
                self.compile_expr(expr)?;
                self.emit_store(&var_name)?;
            }
            Statement::Copy { dest, source } => {
                self.compile_expr(source)?;
                self.emit_store(dest)?;
            }
            Statement::Print(expr) => {
                self.compile_expr(expr)?;
                self.emit_op(0x09); // PRINT
            }
            Statement::Return(expr) => {
                self.compile_expr(expr)?;
                self.emit_op(0x0E); // UNTILWEMEETAGAIN
            }
            Statement::Halt => {
                self.emit_op(0x12); // YOUSHALLNOTPASS
            }
            Statement::If { condition, then_block, else_block } => {
                self.compile_expr(condition)?;
                
                // JUMP_IF_FALSE to else/end
                self.emit_op(0x0C); // JUMP_IF_FALSE
                let jump_pos = self.code.len();
                self.emit_u32(0); // placeholder
                
                // Compile then block
                for stmt in then_block {
                    self.compile_statement(stmt)?;
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
                        self.compile_statement(stmt)?;
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
                self.compile_expr(condition)?;
                
                // JUMP_IF_FALSE to end
                self.emit_op(0x0C); // JUMP_IF_FALSE
                let jump_pos = self.code.len();
                self.emit_u32(0); // placeholder
                
                // Compile body
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
                
                // Jump back to start (absolute offset)
                self.emit_op(0x0B); // JUMP
                self.emit_u32(loop_start);
                
                // Backpatch JUMP_IF_FALSE to end
                let end_pos = self.code.len() as u32;
                self.code[jump_pos..jump_pos+4].copy_from_slice(&end_pos.to_le_bytes());
            }
        }
        Ok(())
    }
    
    fn emit_store(&mut self, var_name: &str) -> Result<(), String> {
        // If we're in a function context, all variables are locals
        if !self.current_locals.is_empty() {
            // Get or allocate local index
            let local_idx = if let Some(&idx) = self.current_locals.get(var_name) {
                idx
            } else {
                // Create new local
                let idx = self.current_locals.len() as u16;
                self.current_locals.insert(var_name.to_string(), idx);
                idx
            };
            self.emit_op(0x10); // BIGBACK_LOCAL
            self.emit_u16(local_idx);
        } else {
            // It's a global braincell
            let braincell_idx = self.get_braincell_index(var_name)?;
            self.emit_op(0x04); // STORE_GLOBAL
            self.emit_u8(braincell_idx);
        }
        Ok(())
    }
    
    fn get_braincell_index(&self, name: &str) -> Result<u8, String> {
        let names = ["aura", "peak", "goon", "mog", "npc", "sigma", "gyatt"];
        names.iter()
            .position(|&n| n == name)
            .map(|idx| idx as u8)
            .ok_or_else(|| format!("unknown braincell: {}", name))
    }
    
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
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
            Expr::Variable(var_name) => {
                self.emit_load(var_name)?;
            }
            Expr::Binary { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                
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
                    // TOUCHY - just read input (prompts removed in v4 for simplicity)
                    self.emit_op(0x0A); // INPUT
                } else if name == "TRANSFORM" {
                    self.compile_expr(arg.as_ref().ok_or_else(|| "TRANSFORM requires argument".to_string())?)?;
                    // Emit call to built-in function index 0
                    self.emit_op(0x0D); // HITMEUP
                    self.emit_u32(0); // built-in TRANSFORM
                } else if name == "RIZZED" {
                    self.compile_expr(arg.as_ref().ok_or_else(|| "RIZZED requires argument".to_string())?)?;
                    // Emit call to built-in function index 1
                    self.emit_op(0x0D); // HITMEUP
                    self.emit_u32(1); // built-in RIZZED
                } else {
                    return Err(format!("Unknown function: {}", name));
                }
            }
            Expr::UserFunctionCall { name, args } => {
                // Compile all arguments
                for arg in args {
                    self.compile_expr(arg)?;
                }
                
                // Look up function index
                let func_idx = *self.function_map.get(name)
                    .ok_or_else(|| format!("undefined function: {}", name))?;
                
                // Emit HITMEUP with function index and argument count
                self.emit_op(0x0D); // HITMEUP
                self.emit_u32(func_idx);
                // Note: arity is stored in function info, VM will validate
            }
        }
        Ok(())
    }
    
    fn emit_load(&mut self, var_name: &str) -> Result<(), String> {
        // Check if it's a local variable
        if let Some(&local_idx) = self.current_locals.get(var_name) {
            self.emit_op(0x0F); // TAX_LOCAL
            self.emit_u16(local_idx);
        } else {
            // It's a global braincell
            let braincell_idx = self.get_braincell_index(var_name)?;
            self.emit_op(0x03); // LOAD_GLOBAL
            self.emit_u8(braincell_idx);
        }
        Ok(())
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
    
    fn emit_u16(&mut self, val: u16) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }
    
    fn emit_u32(&mut self, val: u32) {
        self.code.extend_from_slice(&val.to_le_bytes());
    }
    
    fn write_bytecode(&self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        
        // Header: "BRBC" + version (4) + flags (0)
        result.extend_from_slice(b"BRBC");
        result.extend_from_slice(&4u16.to_le_bytes()); // version 4
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
        
        // Function table
        result.extend_from_slice(&(self.functions.len() as u32).to_le_bytes());
        for func in &self.functions {
            // Add function name to constant pool for lookup
            let name_bytes = func.name.as_bytes().to_vec();
            let name_const_idx = self.constants.iter()
                .position(|c| match c {
                    Constant::String(s) => s == &name_bytes,
                    _ => false,
                })
                .unwrap() as u32;
            
            result.extend_from_slice(&name_const_idx.to_le_bytes());
            result.extend_from_slice(&func.arity.to_le_bytes());
            result.extend_from_slice(&func.local_count.to_le_bytes());
            result.extend_from_slice(&func.code_offset.to_le_bytes());
        }
        
        // Code section
        result.extend_from_slice(&(self.code.len() as u32).to_le_bytes());
        result.extend_from_slice(&self.code);
        
        Ok(result)
    }
}

