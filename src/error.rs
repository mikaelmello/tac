#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TACError {
    CompileError,
    RuntimeError,
}

pub type TACResult<T> = Result<T, TACError>;
