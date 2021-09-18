use crate::token::{Token, TokenKind};

pub struct SourceChar {
    index: usize,
    c: char,
}

impl From<(usize, char)> for SourceChar {
    fn from((index, c): (usize, char)) -> Self {
        Self { index, c }
    }
}

pub struct Scanner<'source> {
    source: &'source str,
    source_chars: Vec<SourceChar>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'source> Scanner<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            source_chars: source.char_indices().map(SourceChar::from).collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn next_token(&mut self) -> Token<'source> {
        self.skip_non_tokens();

        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let c = self.advance();

        match c {
            '\n' => self.make_token(TokenKind::NewLine),
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            '[' => self.make_token(TokenKind::LeftBracket),
            ']' => self.make_token(TokenKind::RightBracket),
            ';' => self.make_token(TokenKind::Semicolon),
            ',' => self.make_token(TokenKind::Comma),
            '.' => self.make_token(TokenKind::Dot),
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            '/' => self.make_token(TokenKind::Slash),
            '*' => self.make_token(TokenKind::Star),
            '%' => self.make_token(TokenKind::Percent),
            ':' => self.make_token(TokenKind::Colon),

            '!' if self.match_advance('=') => self.make_token(TokenKind::BangEqual),
            '!' => self.make_token(TokenKind::Bang),

            '=' if self.match_advance('=') => self.make_token(TokenKind::EqualEqual),
            '=' => self.make_token(TokenKind::Equal),

            '<' if self.match_advance('<') => self.make_token(TokenKind::ShiftLeft),
            '<' if self.match_advance('=') => self.make_token(TokenKind::LessEqual),
            '<' => self.make_token(TokenKind::Less),

            '>' if self.match_advance('>') => self.make_token(TokenKind::ShiftRight),
            '>' if self.match_advance('=') => self.make_token(TokenKind::GreaterEqual),
            '>' => self.make_token(TokenKind::Greater),

            '"' => self.string(),
            '\'' => self.char(),

            c if c.is_ascii_alphabetic() || c == '_' => self.identifier(),
            c if c.is_ascii_digit() => self.number(),

            _ => self.error_token("Unexpected character"),
        }
    }

    fn string(&mut self) -> Token<'source> {
        // TODO: handle escaping
        while self.match_pred_advance(|c| c != '"') {}

        if self.is_at_end() {
            self.error_token("Unterminated string")
        } else {
            self.advance();
            self.make_token(TokenKind::String)
        }
    }

    fn r#char(&mut self) -> Token<'source> {
        // TODO: handle escaping
        let mut count = 0;
        while self.match_pred_advance(|c| c != '\'') {
            count += 1;
        }

        if self.is_at_end() {
            self.error_token("Unterminated character")
        } else {
            self.advance();

            if count > 1 {
                self.error_token("Character literal may only contain one character")
            } else {
                self.make_token(TokenKind::Char)
            }
        }
    }

    fn number(&mut self) -> Token<'source> {
        while self.match_pred_advance(|c| c.is_ascii_digit()) {}

        if self.match_advance('.') {
            while self.match_pred_advance(|c| c.is_ascii_digit()) {}
        }

        // suffix
        while self.match_pred_advance(|c| c.is_ascii_alphanumeric()) {}

        self.make_token(TokenKind::Number)
    }

    fn identifier(&mut self) -> Token<'source> {
        while self.match_pred_advance(|c| c.is_ascii_alphanumeric() || c == '_') {}

        if let Some(tk) = self.check_keyword() {
            self.make_token(tk)
        } else {
            self.make_token(TokenKind::Identifier)
        }
    }

    fn check_keyword(&self) -> Option<TokenKind> {
        match self.lexeme() {
            "if" => Some(TokenKind::If),
            "ifFalse" => Some(TokenKind::IfFalse),
            "goto" => Some(TokenKind::Goto),
            "param" => Some(TokenKind::Param),
            "call" => Some(TokenKind::Call),
            "return" => Some(TokenKind::Return),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "print" => Some(TokenKind::Print),
            "println" => Some(TokenKind::PrintLn),
            "halt" => Some(TokenKind::Halt),
            "scan" => Some(TokenKind::Scan),
            "u64" => Some(TokenKind::U64KW),
            "i64" => Some(TokenKind::I64KW),
            "f64" => Some(TokenKind::F64KW),
            "char" => Some(TokenKind::CharKW),
            "bool" => Some(TokenKind::BoolKW),
            _ => None,
        }
    }

    fn match_advance(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                self.advance();
                true
            }
            Some(_) => false,
            None => false,
        }
    }

    fn match_pred_advance(&mut self, pred: fn(char) -> bool) -> bool {
        match self.peek() {
            Some(c) if pred(c) => {
                self.advance();
                true
            }
            Some(_) => false,
            None => false,
        }
    }

    fn advance(&mut self) -> char {
        let c = self
            .peek()
            .expect("Trying to advance when there's nothing further");

        self.current += 1;

        if c == '\n' {
            self.line += 1;
        }

        c
    }

    fn peek(&self) -> Option<char> {
        self.source_chars.get(self.current).map(|sc| sc.c)
    }

    fn peek_next(&self) -> Option<char> {
        self.source_chars.get(self.current + 1).map(|sc| sc.c)
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source_chars.len()
    }

    fn lexeme(&self) -> &'source str {
        self.lexeme_at(self.start, self.current)
    }

    fn make_token(&self, kind: TokenKind) -> Token<'source> {
        Token {
            kind,
            lexeme: self.lexeme(),
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token<'static> {
        Token {
            kind: TokenKind::Error,
            lexeme: message,
            line: self.line,
        }
    }

    fn skip_non_tokens(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }

            match c.is_whitespace() {
                true => self.advance(),
                false => break,
            };

            if c == '#' {
                self.advance();
                while let Some(cc) = self.peek() {
                    if cc == '\n' {
                        break;
                    }
                    self.advance();
                }
            }
        }
    }

    fn lexeme_at(&self, start: usize, end: usize) -> &'source str {
        let left = self
            .source_chars
            .get(start)
            .map(|sc| sc.index)
            .unwrap_or(self.source.len());

        let right = self
            .source_chars
            .get(end)
            .map(|sc| sc.index)
            .unwrap_or(self.source.len());

        &self.source[left..right]
    }
}
