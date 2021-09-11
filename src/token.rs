use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenKind {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Percent,

    // One Or Two Character Tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    ShiftLeft,
    ShiftRight,

    // Literals.
    Identifier,
    String,
    Number,
    Char,

    // Keywords.
    If,
    IfFalse,
    Goto,
    Param,
    Call,
    Return,
    True,
    False,
    Print,
    PrintLn,
    Scan,
    U64KW,
    I64KW,
    F64KW,
    CharKW,
    BoolKW,

    // Special.
    Error,
    Eof,
}

pub struct Token<'source> {
    pub kind: TokenKind,
    pub lexeme: &'source str,
    pub line: usize,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
