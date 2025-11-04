use crate::value::Value;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "runtime: {}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

pub fn execute(bytecode: &[u8]) -> Result<(), RuntimeError> {
    let mut vm = VM::new();
    vm.load(bytecode)?;
    vm.run()
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
    }
}

struct CallFrame {
    return_address: usize,
    locals: Vec<Option<Value>>,
}

struct FunctionMetadata {
    arity: u16,
    local_count: u16,
    code_offset: u32,
}

struct VM {
    constants: Vec<Value>,
    globals: [Option<Value>; 7],
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    functions: Vec<FunctionMetadata>,
    code: Vec<u8>,
    ip: usize,
}

impl VM {
    fn new() -> Self {
        Self {
            constants: Vec::new(),
            globals: [None, None, None, None, None, None, None],
            stack: Vec::new(),
            call_stack: Vec::new(),
            functions: Vec::new(),
            code: Vec::new(),
            ip: 0,
        }
    }
    
    fn load(&mut self, bytecode: &[u8]) -> Result<(), RuntimeError> {
        let mut pos = 0;
        
        // Verify magic
        if bytecode.len() < 4 || &bytecode[pos..pos+4] != b"BRBC" {
            return Err(RuntimeError::new("invalid bytecode file"));
        }
        pos += 4;
        
        // Read version and flags
        if bytecode.len() < pos + 4 {
            return Err(RuntimeError::new("invalid bytecode header"));
        }
        let version = u16::from_le_bytes([bytecode[pos], bytecode[pos+1]]);
        // let flags = u16::from_le_bytes([bytecode[pos+2], bytecode[pos+3]]);
        pos += 4;
        
        // Read constant pool
        if bytecode.len() < pos + 4 {
            return Err(RuntimeError::new("invalid constant pool header"));
        }
        let const_count = u32::from_le_bytes([
            bytecode[pos], bytecode[pos+1], bytecode[pos+2], bytecode[pos+3]
        ]);
        pos += 4;
        
        for _ in 0..const_count {
            if bytecode.len() <= pos {
                return Err(RuntimeError::new("invalid constant entry"));
            }
            let tag = bytecode[pos];
            pos += 1;
            
            match tag {
                1 => { // Number
                    if bytecode.len() < pos + 8 {
                        return Err(RuntimeError::new("invalid number constant"));
                    }
                    let bytes = [
                        bytecode[pos], bytecode[pos+1], bytecode[pos+2], bytecode[pos+3],
                        bytecode[pos+4], bytecode[pos+5], bytecode[pos+6], bytecode[pos+7]
                    ];
                    let num = f64::from_le_bytes(bytes);
                    self.constants.push(Value::Number(num));
                    pos += 8;
                }
                2 => { // String
                    if bytecode.len() < pos + 4 {
                        return Err(RuntimeError::new("invalid string constant"));
                    }
                    let len = u32::from_le_bytes([
                        bytecode[pos], bytecode[pos+1], bytecode[pos+2], bytecode[pos+3]
                    ]) as usize;
                    pos += 4;
                    
                    if bytecode.len() < pos + len {
                        return Err(RuntimeError::new("invalid string data"));
                    }
                    let bytes = bytecode[pos..pos+len].to_vec();
                    pos += len;
                    
                    let s = String::from_utf8(bytes)
                        .map_err(|_| RuntimeError::new("invalid UTF-8 in string constant"))?;
                    self.constants.push(Value::String(Rc::new(s)));
                }
                _ => return Err(RuntimeError::new("unknown constant type")),
            }
        }
        
        // Read function table (only for v4+)
        if version >= 4 {
            if bytecode.len() < pos + 4 {
                return Err(RuntimeError::new("invalid function table header"));
            }
            let func_count = u32::from_le_bytes([
                bytecode[pos], bytecode[pos+1], bytecode[pos+2], bytecode[pos+3]
            ]);
            pos += 4;
            
            for _ in 0..func_count {
                if bytecode.len() < pos + 14 {
                    return Err(RuntimeError::new("invalid function entry"));
                }
                
                let name_const_idx = u32::from_le_bytes([
                    bytecode[pos], bytecode[pos+1], bytecode[pos+2], bytecode[pos+3]
                ]);
                let arity = u16::from_le_bytes([bytecode[pos+4], bytecode[pos+5]]);
                let local_count = u16::from_le_bytes([bytecode[pos+6], bytecode[pos+7]]);
                let code_offset = u32::from_le_bytes([
                    bytecode[pos+8], bytecode[pos+9], bytecode[pos+10], bytecode[pos+11]
                ]);
                pos += 12;
                
                // Function name is stored but not needed at runtime
                // (it's in the constant pool for reference, but we use index-based lookup)
                let _name_const = &self.constants[name_const_idx as usize];
                
                self.functions.push(FunctionMetadata {
                    arity,
                    local_count,
                    code_offset,
                });
            }
        }
        
        // Read code section
        if bytecode.len() < pos + 4 {
            return Err(RuntimeError::new("invalid code section header"));
        }
        let code_size = u32::from_le_bytes([
            bytecode[pos], bytecode[pos+1], bytecode[pos+2], bytecode[pos+3]
        ]);
        pos += 4;
        
        if bytecode.len() < pos + code_size as usize {
            return Err(RuntimeError::new("invalid code data"));
        }
        self.code = bytecode[pos..pos+code_size as usize].to_vec();
        self.ip = 0;
        
        Ok(())
    }
    
    fn run(&mut self) -> Result<(), RuntimeError> {
        while self.ip < self.code.len() {
            let op = self.code[self.ip];
            self.ip += 1;
            
            match op {
                0x01 => return Ok(()), // HALT
                0x02 => self.op_load_const()?,
                0x03 => self.op_load_global()?,
                0x04 => self.op_store_global()?,
                0x05 => self.op_add()?,
                0x06 => self.op_sub()?,
                0x07 => self.op_mul()?,
                0x08 => self.op_div()?,
                0x09 => self.op_print()?,
                0x0A => self.op_input()?,
                0x0B => self.op_jump()?,
                0x0C => self.op_jump_if_false()?,
                0x0D => self.op_hitmeup()?, // HITMEUP (user function or built-in)
                0x0E => self.op_untilwemeetagain()?, // UNTILWEMEETAGAIN (return)
                0x0F => self.op_tax_local()?, // TAX_LOCAL
                0x10 => self.op_bigback_local()?, // BIGBACK_LOCAL
                0x11 => self.op_poopy()?, // POOPY
                0x12 => return Ok(()), // YOUSHALLNOTPASS (same as HALT)
                _ => return Err(RuntimeError::new(&format!("unknown opcode: 0x{:02x}", op))),
            }
        }
        
        Ok(())
    }
    
    fn op_load_const(&mut self) -> Result<(), RuntimeError> {
        let idx = self.read_u32();
        if idx >= self.constants.len() as u32 {
            return Err(RuntimeError::new("constant index out of bounds"));
        }
        let value = self.constants[idx as usize].clone();
        self.stack.push(value);
        Ok(())
    }
    
    fn op_load_global(&mut self) -> Result<(), RuntimeError> {
        let idx = self.read_u8();
        if idx >= 7 {
            return Err(RuntimeError::new("global index out of bounds"));
        }
        let value = self.globals[idx as usize].clone()
            .ok_or_else(|| {
                let names = ["aura", "peak", "goon", "mog", "npc", "sigma", "gyatt"];
                RuntimeError::new(&format!("unset braincell: {}", names[idx as usize]))
            })?;
        self.stack.push(value);
        Ok(())
    }
    
    fn op_store_global(&mut self) -> Result<(), RuntimeError> {
        let idx = self.read_u8();
        if idx >= 7 {
            return Err(RuntimeError::new("global index out of bounds"));
        }
        let value = self.stack.pop()
            .ok_or_else(|| RuntimeError::new("stack underflow"))?;
        self.globals[idx as usize] = Some(value);
        Ok(())
    }
    
    fn op_add(&mut self) -> Result<(), RuntimeError> {
        let right = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let left = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let result = left.add(&right)
            .map_err(|e| RuntimeError::new(&e))?;
        self.stack.push(result);
        Ok(())
    }
    
    fn op_sub(&mut self) -> Result<(), RuntimeError> {
        let right = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let left = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let result = left.sub(&right)
            .map_err(|e| RuntimeError::new(&e))?;
        self.stack.push(result);
        Ok(())
    }
    
    fn op_mul(&mut self) -> Result<(), RuntimeError> {
        let right = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let left = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let result = left.mul(&right)
            .map_err(|e| RuntimeError::new(&e))?;
        self.stack.push(result);
        Ok(())
    }
    
    fn op_div(&mut self) -> Result<(), RuntimeError> {
        let right = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let left = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let result = left.div(&right)
            .map_err(|e| RuntimeError::new(&e))?;
        self.stack.push(result);
        Ok(())
    }
    
    fn op_print(&mut self) -> Result<(), RuntimeError> {
        let value = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        println!("{}", value.format_for_print());
        Ok(())
    }
    
    fn op_input(&mut self) -> Result<(), RuntimeError> {
        use std::io::{self, Write};
        use std::rc::Rc;
        
        io::stdout().flush().map_err(|_| RuntimeError::new("failed to flush stdout"))?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .map_err(|_| RuntimeError::new("failed to read from stdin"))?;
        
        // Strip trailing newline/carriage return
        let trimmed = input.trim_end();
        self.stack.push(Value::String(Rc::new(trimmed.to_string())));
        Ok(())
    }
    
    fn op_jump(&mut self) -> Result<(), RuntimeError> {
        let target = self.read_u32();
        if target >= self.code.len() as u32 {
            return Err(RuntimeError::new("jump target out of bounds"));
        }
        self.ip = target as usize;
        Ok(())
    }
    
    fn op_jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let value = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        let target = self.read_u32();
        
        if !is_truthy(&value) {
            if target >= self.code.len() as u32 {
                return Err(RuntimeError::new("jump target out of bounds"));
            }
            self.ip = target as usize;
        }
        Ok(())
    }
    
    fn read_u8(&mut self) -> u8 {
        if self.ip >= self.code.len() {
            return 0;
        }
        let val = self.code[self.ip];
        self.ip += 1;
        val
    }
    
    fn read_u16(&mut self) -> u16 {
        if self.ip + 2 > self.code.len() {
            return 0;
        }
        let bytes = [self.code[self.ip], self.code[self.ip+1]];
        self.ip += 2;
        u16::from_le_bytes(bytes)
    }
    
    fn read_u32(&mut self) -> u32 {
        if self.ip + 4 > self.code.len() {
            return 0;
        }
        let bytes = [
            self.code[self.ip],
            self.code[self.ip+1],
            self.code[self.ip+2],
            self.code[self.ip+3],
        ];
        self.ip += 4;
        u32::from_le_bytes(bytes)
    }
    
    fn op_hitmeup(&mut self) -> Result<(), RuntimeError> {
        let func_idx = self.read_u32();
        
        // Check call stack depth
        if self.call_stack.len() >= 256 {
            return Err(RuntimeError::new("call stack overflow"));
        }
        
        // Built-in functions (0 and 1)
        if func_idx == 0 {
            // TRANSFORM(string -> number)
            let value = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
            match value {
                Value::String(s) => {
                    let num = s.parse::<f64>()
                        .map_err(|_| RuntimeError::new("TRANSFORM: invalid number string"))?;
                    self.stack.push(Value::Number(num));
                }
                _ => return Err(RuntimeError::new("TRANSFORM: expected string argument")),
            }
            return Ok(());
        } else if func_idx == 1 {
            // RIZZED(string length)
            let value = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
            match value {
                Value::String(s) => {
                    let len = s.chars().count() as f64;
                    self.stack.push(Value::Number(len));
                }
                _ => return Err(RuntimeError::new("RIZZED: expected string argument")),
            }
            return Ok(());
        }
        
        // User-defined function
        if func_idx < 2 || func_idx >= 2 + self.functions.len() as u32 {
            return Err(RuntimeError::new("function index out of bounds"));
        }
        
        let func = &self.functions[(func_idx - 2) as usize];
        
        // Validate argument count
        if self.stack.len() < func.arity as usize {
            return Err(RuntimeError::new("not enough arguments on stack"));
        }
        
        // Push call frame
        let frame = CallFrame {
            return_address: self.ip,
            locals: vec![None; func.local_count as usize],
        };
        self.call_stack.push(frame);
        
        // Set up locals from stack arguments
        let frame = self.call_stack.last_mut().unwrap();
        for i in 0..func.arity {
            let val = self.stack.pop().unwrap();
            frame.locals[i as usize] = Some(val);
        }
        
        // Jump to function start
        self.ip = func.code_offset as usize;
        
        Ok(())
    }
    
    fn op_untilwemeetagain(&mut self) -> Result<(), RuntimeError> {
        let frame = self.call_stack.pop().ok_or_else(|| RuntimeError::new("return outside of function"))?;
        
        // Get return value (top of stack should be the return value)
        let ret_val = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        
        // Restore instruction pointer
        self.ip = frame.return_address;
        
        // Push return value back onto stack
        self.stack.push(ret_val);
        
        Ok(())
    }
    
    fn op_tax_local(&mut self) -> Result<(), RuntimeError> {
        let local_idx = self.read_u16();
        let frame = self.call_stack.last_mut().ok_or_else(|| RuntimeError::new("local access outside of function"))?;
        
        if local_idx >= frame.locals.len() as u16 {
            return Err(RuntimeError::new("local index out of bounds"));
        }
        
        let value = frame.locals[local_idx as usize].clone()
            .ok_or_else(|| RuntimeError::new("unset local variable"))?;
        
        self.stack.push(value);
        Ok(())
    }
    
    fn op_bigback_local(&mut self) -> Result<(), RuntimeError> {
        let local_idx = self.read_u16();
        let frame = self.call_stack.last_mut().ok_or_else(|| RuntimeError::new("local assignment outside of function"))?;
        
        if local_idx >= frame.locals.len() as u16 {
            return Err(RuntimeError::new("local index out of bounds"));
        }
        
        let value = self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        frame.locals[local_idx as usize] = Some(value);
        
        Ok(())
    }
    
    fn op_poopy(&mut self) -> Result<(), RuntimeError> {
        self.stack.pop().ok_or_else(|| RuntimeError::new("stack underflow"))?;
        Ok(())
    }
}

