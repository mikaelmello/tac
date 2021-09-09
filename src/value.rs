use std::fmt::{Display, Formatter};

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

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use crate::value::Value;

    #[test]
    fn value_is_at_most_128_bits() {
        assert!(size_of::<Value>() <= 16);
    }
}
