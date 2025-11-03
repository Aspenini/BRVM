use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
}

impl Value {
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::String(s1), _) => Ok(Value::String(Rc::new(format!("{}{}", s1, other.to_string())))),
            (_, Value::String(s2)) => Ok(Value::String(Rc::new(format!("{}{}", self.to_string(), s2)))),
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
        }
    }
    
    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 - n2)),
            _ => Err("subtraction requires both operands to be numbers".to_string()),
        }
    }
    
    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 * n2)),
            _ => Err("multiplication requires both operands to be numbers".to_string()),
        }
    }
    
    pub fn div(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(n1), Value::Number(n2)) => {
                if *n2 == 0.0 {
                    Err("division by zero".to_string())
                } else {
                    Ok(Value::Number(n1 / n2))
                }
            }
            _ => Err("division requires both operands to be numbers".to_string()),
        }
    }
    
    pub fn format_for_print(&self) -> String {
        match self {
            Value::Number(n) => format!("{}", n),
            Value::String(s) => s.to_string(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_for_print())
    }
}

