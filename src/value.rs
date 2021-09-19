use std::{
    convert::TryInto,
    fmt::{Display, Formatter},
    ops::{self, Shl, Shr},
};

#[derive(Clone, Copy, Debug)]
pub enum Value {
    F64(f64),
    U64(u64),
    I64(i64),
    Bool(bool),
    Char(char),
    Addr(usize),
}

impl Value {
    pub fn arithmetic_negate(&mut self) -> Result<(), String> {
        match self {
            Value::F64(val) => {
                *val = -(*val);
                Ok(())
            }
            Value::U64(_) => Err("It is not possible to negate a number of type u64".into()),
            Value::I64(val) => {
                *val = -(*val);
                Ok(())
            }
            Value::Bool(_) => Err("It is not possible to arithmetically negate a boolean".into()),
            Value::Char(_) => Err("It is not possible to negate a character".into()),
            Value::Addr(_) => Err("It is not possible to negate an address".into()),
        }
    }

    pub fn logic_negate(&mut self) -> Result<(), String> {
        match self {
            Value::Bool(val) => {
                *val = !(*val);
                Ok(())
            }
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

    pub fn is_numeric_zero(&self) -> bool {
        match *self {
            Value::F64(v) if v == 0.0 || v == -0.0 => true,
            Value::I64(0) | Value::U64(0) => true,
            _ => false,
        }
    }

    pub fn type_info(&self) -> &'static str {
        match self {
            Value::F64(_) => "f64",
            Value::U64(_) => "u64",
            Value::I64(_) => "i64",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::Addr(_) => "addr",
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
            Value::Addr(val) => write!(f, "addr({})", val),
        }
    }
}

impl ops::Add<Value> for Value {
    type Output = Result<Value, String>;

    fn add(self, rhs: Value) -> Result<Value, String> {
        match (self, rhs) {
            // TODO: handle warnings/errors in regard to overflow
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a.wrapping_add(b))),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_add(b))),
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

    fn sub(self, rhs: Value) -> Result<Value, String> {
        match (self, rhs) {
            // TODO: handle warnings/errors in regard to overflow
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a.wrapping_sub(b))),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_sub(b))),
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

    fn mul(self, rhs: Value) -> Result<Value, String> {
        match (self, rhs) {
            // TODO: handle warnings/errors in regard to overflow
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a.wrapping_mul(b))),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a.wrapping_mul(b))),
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

    fn div(self, rhs: Value) -> Result<Value, String> {
        match (self, rhs) {
            // TODO: handle warnings/errors in regard to overflow
            (Value::F64(_) | Value::I64(_) | Value::U64(_), b) if b.is_numeric_zero() => {
                Err("Division by 0".to_string())
            }
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

    fn rem(self, rhs: Value) -> Result<Value, String> {
        match (self, rhs) {
            (Value::F64(_) | Value::I64(_) | Value::U64(_), b) if b.is_numeric_zero() => {
                Err("Division by 0".to_string())
            }
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

impl ops::Shl<Value> for Value {
    type Output = Result<Value, String>;

    fn shl(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::U64(a), Value::U64(b)) => {
                let b = b.try_into().unwrap_or(u32::MAX);
                Ok(Value::U64(a.wrapping_shl(b)))
            }
            (Value::U64(a), Value::I64(b)) => {
                if b < 0 {
                    return Value::U64(a).shr(Value::I64(-b));
                }
                let b = b.try_into().unwrap_or(u32::MAX);

                Ok(Value::U64(a.wrapping_shl(b)))
            }
            (Value::I64(a), Value::I64(b)) => {
                if b < 0 {
                    return Value::I64(a).shr(Value::I64(-b));
                }
                let b = b.try_into().unwrap_or(u32::MAX);

                Ok(Value::I64(a.wrapping_shl(b)))
            }
            (a, b) => Err(format!(
                "Operator '<<' not supported between values of type '{}' and '{}'",
                a.type_info(),
                b.type_info()
            )),
        }
    }
}

impl ops::Shr<Value> for Value {
    type Output = Result<Value, String>;

    fn shr(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Value::U64(a), Value::U64(b)) => {
                let b = b.try_into().unwrap_or(u32::MAX);
                Ok(Value::U64(a.wrapping_shr(b)))
            }
            (Value::U64(a), Value::I64(b)) => {
                if b < 0 {
                    return Value::U64(a).shl(Value::I64(-b));
                }
                let b = b.try_into().unwrap_or(u32::MAX);

                Ok(Value::U64(a.wrapping_shr(b)))
            }
            (Value::I64(a), Value::I64(b)) => {
                if b < 0 {
                    return Value::I64(a).shl(Value::I64(-b));
                }
                let b = b.try_into().unwrap_or(u32::MAX);

                Ok(Value::I64(a.wrapping_shr(b)))
            }
            (a, b) => Err(format!(
                "Operator '<<' not supported between values of type '{}' and '{}'",
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
