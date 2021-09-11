use std::{
    fmt::{Display, Formatter},
    ops,
};

use crate::error::TACResult;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    F64(f64),
}

impl Value {}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::F64(val) => write!(f, "{}", val),
        }
    }
}

impl ops::Add<Value> for Value {
    type Output = TACResult<Value>;

    fn add(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
        }
    }
}

impl ops::Sub<Value> for Value {
    type Output = TACResult<Value>;

    fn sub(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
        }
    }
}

impl ops::Mul<Value> for Value {
    type Output = TACResult<Value>;

    fn mul(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
        }
    }
}

impl ops::Div<Value> for Value {
    type Output = TACResult<Value>;

    fn div(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a / b)),
        }
    }
}

impl ops::Rem<Value> for Value {
    type Output = TACResult<Value>;

    fn rem(self, _rhs: Value) -> TACResult<Value> {
        match (self, _rhs) {
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a % b)),
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
