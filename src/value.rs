use std::{
    fmt::{Display, Formatter},
    ops,
};

#[derive(Clone, Copy, Debug)]
pub enum Value {
    F64(f64),
    U64(u64),
    I64(i64),
    Bool(bool),
    Char(char),
}

impl Value {
    pub fn arithmetic_negate(&mut self) -> Result<(), String> {
        match self {
            Value::F64(val) => Ok(*val = -(*val)),
            Value::U64(_) => Err("It is not possible to negate a number of type u64".into()),
            Value::I64(val) => Ok(*val = -(*val)),
            Value::Bool(_) => Err("It is not possible to arithmetically negate a boolean".into()),
            Value::Char(_) => Err("It is not possible to negate a character".into()),
        }
    }

    pub fn logic_negate(&mut self) -> Result<(), String> {
        match self {
            Value::Bool(val) => Ok(*val = !(*val)),
            val => Err(format!(
                "Operator '!' not supported for value of type {}",
                val.type_info()
            )),
        }
    }

    pub fn lt(a: Value, b: Value) -> Result<Value, String> {
        match (a, b) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a < b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::Bool(a < b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a < b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a < b)),
            (a, b) => Err(format!(
                "Operator '<' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }

    pub fn gt(a: Value, b: Value) -> Result<Value, String> {
        match (a, b) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a > b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::Bool(a > b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a > b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a > b)),
            (a, b) => Err(format!(
                "Operator '>' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }

    pub fn eq(a: Value, b: Value) -> Result<Value, String> {
        match (a, b) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::Bool(a == b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::Bool(a == b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::Bool(a == b)),
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
            (Value::Char(a), Value::Char(b)) => Ok(Value::Bool(a == b)),
            (a, b) => Err(format!(
                "Operator '==' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }

    pub fn type_info(&self) -> &'static str {
        match self {
            Value::F64(_) => "f64",
            Value::U64(_) => "u64",
            Value::I64(_) => "i64",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::F64(val) => write!(f, "{}", val),
            Value::U64(val) => write!(f, "{}", val),
            Value::I64(val) => write!(f, "{}", val),
            Value::Bool(val) => write!(f, "{}", val),
            Value::Char(val) => write!(f, "{}", val),
        }
    }
}

impl ops::Add<Value> for Value {
    type Output = Result<Value, String>;

    fn add(self, _rhs: Value) -> Result<Value, String> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a + b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a + b)),
            (a, b) => Err(format!(
                "Operator '+' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }
}

impl ops::Sub<Value> for Value {
    type Output = Result<Value, String>;

    fn sub(self, _rhs: Value) -> Result<Value, String> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a - b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a - b)),
            (a, b) => Err(format!(
                "Operator '-' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }
}

impl ops::Mul<Value> for Value {
    type Output = Result<Value, String>;

    fn mul(self, _rhs: Value) -> Result<Value, String> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a * b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a * b)),
            (a, b) => Err(format!(
                "Operator '*' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }
}

impl ops::Div<Value> for Value {
    type Output = Result<Value, String>;

    fn div(self, _rhs: Value) -> Result<Value, String> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a / b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a / b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a / b)),
            (a, b) => Err(format!(
                "Operator '/' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }
}

impl ops::Rem<Value> for Value {
    type Output = Result<Value, String>;

    fn rem(self, _rhs: Value) -> Result<Value, String> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a % b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a % b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a % b)),
            (a, b) => Err(format!(
                "Operator '%' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use crate::value::Value;

    #[test]
    fn value_is_at_most_128_bits() {
        assert!(size_of::<Value>() <= 16);
    }
}
