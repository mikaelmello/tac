use std::{
    fmt::{Display, Formatter},
    ops,
};

use crate::error::{TACError, TACResult};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    F64(f64),
    U64(u64),
    I64(i64),
}

impl Value {
    pub fn negate(&mut self) -> TACResult<()> {
        match self {
            Value::F64(val) => Ok(*val = -(*val)),
            Value::U64(val) => Err(TACError::RuntimeError),
            Value::I64(val) => Ok(*val = -(*val)),
        }
    }

    pub fn type_info(&self) -> &'static str {
        match self {
            Value::F64(_) => "f64",
            Value::U64(_) => "u64",
            Value::I64(_) => "i64",
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::F64(val) => write!(f, "{}", val),
            Value::U64(val) => write!(f, "{}", val),
            Value::I64(val) => write!(f, "{}", val),
        }
    }
}

impl ops::Add<Value> for Value {
    type Output = TACResult<Value>;

    fn add(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a + b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a + b)),
            (a, b) => Err(TACError::RuntimeError),
        }
    }
}

impl ops::Sub<Value> for Value {
    type Output = TACResult<Value>;

    fn sub(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a - b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a - b)),
            (a, b) => Err(TACError::RuntimeError),
        }
    }
}

impl ops::Mul<Value> for Value {
    type Output = TACResult<Value>;

    fn mul(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a * b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a * b)),
            (a, b) => Err(TACError::RuntimeError),
        }
    }
}

impl ops::Div<Value> for Value {
    type Output = TACResult<Value>;

    fn div(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            // TODO: handle overflows that would normally panic
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a / b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a / b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a / b)),
            (a, b) => Err(TACError::RuntimeError),
        }
    }
}

impl ops::Rem<Value> for Value {
    type Output = TACResult<Value>;

    fn rem(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a % b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a % b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a % b)),
            (a, b) => Err(TACError::RuntimeError),
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
