use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(Rc<String>),
}

impl Value {
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::String(s1), _) => Ok(Value::String(Rc::new(format!(
                "{}{}",
                s1,
                other.to_string()
            )))),
            (_, Value::String(s2)) => Ok(Value::String(Rc::new(format!(
                "{}{}",
                self.to_string(),
                s2
            )))),
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
            (Value::String(s), Value::Number(n)) | (Value::Number(n), Value::String(s)) => {
                let count = repeat_count(*n)?;
                Ok(Value::String(Rc::new(s.repeat(count))))
            }
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

fn repeat_count(value: f64) -> Result<usize, String> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 {
        return Err("string repeat count must be a non-negative whole number".to_string());
    }

    if value > usize::MAX as f64 {
        return Err("string repeat count is too large".to_string());
    }

    Ok(value as usize)
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_for_print())
    }
}
