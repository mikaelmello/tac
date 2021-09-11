use crate::token::{Token, TokenKind};

#[derive(Clone, Debug, PartialEq)]
pub enum TACError {
    CompileError,
    RuntimeError,
}

pub type TACResult<T> = Result<T, TACError>;

pub fn error_at(token: Token, message: &str) {
    eprint!("[line {}] Error", token.line);

    match token.kind {
        TokenKind::Eof => eprint!(" at end"),
        TokenKind::Error => {}
        _ => eprint!(" at '{}'", token.lexeme),
    }

    eprintln!(": {}", message);
}
