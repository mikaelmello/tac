use crate::{
    chunk::{Chunk, Instruction},
    disassembler::Disassembler,
    error::{error_at, TACError, TACResult},
    scanner::Scanner,
    token::{Token, TokenKind},
    value::Value,
};

#[derive(Copy, Clone, PartialOrd, PartialEq)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

impl Precedence {
    fn next(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::None,
        }
    }
}

type CompilerFn<'source, 'c> = fn(&mut Compiler<'source, 'c>);

#[derive(Copy, Clone)]
struct CompilerRule<'source, 'c> {
    prefix: Option<CompilerFn<'source, 'c>>,
    infix: Option<CompilerFn<'source, 'c>>,
    precedence: Precedence,
}

impl<'source, 'c>
    From<(
        Option<CompilerFn<'source, 'c>>,
        Option<CompilerFn<'source, 'c>>,
        Precedence,
    )> for CompilerRule<'source, 'c>
{
    fn from(
        (prefix, infix, precedence): (
            Option<CompilerFn<'source, 'c>>,
            Option<CompilerFn<'source, 'c>>,
            Precedence,
        ),
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

pub struct Compiler<'source, 'c> {
    scanner: Scanner<'source>,
    chunk: &'c mut Chunk,
    had_error: bool,
    panic_mode: bool,
    current: Token<'source>,
    previous: Token<'source>,
}

impl<'source, 'c> Compiler<'source, 'c> {
    pub fn compile(source: &'source str, chunk: &'c mut Chunk) -> TACResult<()> {
        let mut compiler = Self {
            scanner: Scanner::new(source),
            chunk,
            had_error: false,
            panic_mode: false,
            current: Token::synthetic(""),
            previous: Token::synthetic(""),
        };

        compiler.advance();
        compiler.expression();
        compiler.consume(TokenKind::Eof, "Expect end of expression");
        compiler.end();

        match compiler.had_error {
            false => Ok(()),
            true => Err(TACError::CompileError),
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let rule = self.get_rule(self.previous.kind);
        let prefix_rule = rule.prefix;

        let prefix_rule = match prefix_rule {
            Some(rule) => rule,
            None => {
                return self.error("Missing expression");
            }
        };

        prefix_rule(self);

        if precedence <= self.get_rule(self.current.kind).precedence {
            self.advance();

            let infix_rule = self
                .get_rule(self.previous.kind)
                .infix
                .expect("Expect infix rule");

            infix_rule(self);
        }
    }

    fn unary(&mut self) {
        let kind = self.previous.kind;
        self.parse_precedence(Precedence::Primary);

        match kind {
            TokenKind::Minus => self.emit_instruction(Instruction::NEGATE),
            TokenKind::Bang => self.emit_instruction(Instruction::NOT),
            _ => panic!("Unreachable - match arm not covered in unary()"),
        }
    }

    fn binary(&mut self) {
        let kind = self.previous.kind;
        self.parse_precedence(Precedence::Primary);

        match kind {
            TokenKind::Plus => self.emit_instruction(Instruction::ADD),
            TokenKind::Minus => self.emit_instruction(Instruction::SUBTRACT),
            TokenKind::Slash => self.emit_instruction(Instruction::DIVIDE),
            TokenKind::Star => self.emit_instruction(Instruction::MULTIPLY),
            TokenKind::EqualEqual => self.emit_instruction(Instruction::EQUAL),
            TokenKind::BangEqual => self.emit_instructions(&[Instruction::EQUAL, Instruction::NOT]),
            TokenKind::Greater => self.emit_instruction(Instruction::GREATER),
            TokenKind::GreaterEqual => {
                self.emit_instructions(&[Instruction::LESS, Instruction::NOT])
            }
            TokenKind::Less => self.emit_instruction(Instruction::LESS),
            TokenKind::LessEqual => {
                self.emit_instructions(&[Instruction::GREATER, Instruction::NOT])
            }
            _ => panic!("Unreachable - match arm not covered in binary()"),
        }
    }

    fn literal(&mut self) {
        match self.previous.kind {
            TokenKind::False => self.emit_instruction(Instruction::FALSE),
            TokenKind::True => self.emit_instruction(Instruction::TRUE),
            _ => panic!("Unreachable - match arm not covered in literal()"),
        }
    }

    fn number(&mut self) {
        enum Type {
            U64,
            I64,
            F64,
        }

        let token = self.previous;
        let lexeme = token.lexeme;

        let mut chars = lexeme.char_indices().peekable();
        let number_begin = 0;
        let mut number_end = lexeme.len();
        let mut nt = Type::I64;

        while let Some((i, c)) = chars.peek() {
            if c.is_ascii_alphabetic() {
                number_end = *i;
                break;
            }
            let c = chars.next().unwrap().1;
            if c == '.' {
                nt = Type::F64;
            }
        }

        let suffix_begin = number_end;
        let suffix_end = lexeme.len();

        let number = &lexeme[number_begin..number_end];
        let suffix = &lexeme[suffix_begin..suffix_end];

        let type_info = match (suffix, nt) {
            ("u64", Type::F64) => Err("Cannot set u64 suffix to a float number".into()),
            ("i64", Type::F64) => Err("Cannot set i64 suffix to a float number".into()),
            ("u64", _) => Ok(Type::U64),
            ("i64", _) => Ok(Type::I64),
            ("f64", _) => Ok(Type::F64),
            ("", t) => Ok(t),
            (suffix, _) => Err(format!("Invalid suffix '{}'", suffix)),
        };

        macro_rules! parse_number {
            ($type:ty,$value_type:ident) => {{
                let result = number.parse::<$type>();
                match result {
                    Ok(val) => {
                        let value = Value::$value_type(val);
                        self.make_constant(value);
                    }
                    Err(_) => self.error(&format!(
                        "It was not possible to parse number to type {}",
                        stringify!($type)
                    )),
                };
            }};
        }

        match type_info {
            Ok(t) => match t {
                Type::U64 => parse_number!(u64, U64),
                Type::I64 => parse_number!(i64, I64),
                Type::F64 => parse_number!(f64, F64),
            },
            Err(msg) => {
                self.error(&msg);
                return;
            }
        };
    }

    fn get_rule(&mut self, kind: TokenKind) -> CompilerRule<'source, 'c> {
        let rule: (
            Option<CompilerFn<'source, 'c>>,
            Option<CompilerFn<'source, 'c>>,
            Precedence,
        ) = match kind {
            TokenKind::LeftParen => (None, None, Precedence::None),
            TokenKind::RightParen => (None, None, Precedence::None),
            TokenKind::LeftBrace => (None, None, Precedence::None),
            TokenKind::RightBrace => (None, None, Precedence::None),
            TokenKind::Comma => (None, None, Precedence::None),
            TokenKind::Dot => (None, None, Precedence::None),
            TokenKind::Minus => (Some(Self::unary), Some(Self::binary), Precedence::Term),
            TokenKind::Plus => (None, Some(Self::binary), Precedence::Term),
            TokenKind::Semicolon => (None, None, Precedence::None),
            TokenKind::Slash => (None, Some(Self::binary), Precedence::Factor),
            TokenKind::Star => (None, Some(Self::binary), Precedence::Factor),
            TokenKind::Bang => (Some(Self::unary), None, Precedence::None),
            TokenKind::BangEqual => (None, Some(Self::binary), Precedence::Equality),
            TokenKind::Equal => (None, None, Precedence::None),
            TokenKind::EqualEqual => (None, Some(Self::binary), Precedence::Equality),
            TokenKind::Greater => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::GreaterEqual => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::Less => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::LessEqual => (None, Some(Self::binary), Precedence::Comparison),
            TokenKind::Identifier => (None, None, Precedence::None),
            TokenKind::String => (None, None, Precedence::None),
            TokenKind::Number => (Some(Self::number), None, Precedence::None),
            TokenKind::If => (None, None, Precedence::None),
            TokenKind::Print => (None, None, Precedence::None),
            TokenKind::Return => (None, None, Precedence::None),
            TokenKind::LeftBracket => (None, None, Precedence::None),
            TokenKind::RightBracket => (None, None, Precedence::None),
            TokenKind::Percent => (None, None, Precedence::None),
            TokenKind::ShiftLeft => (None, None, Precedence::None),
            TokenKind::ShiftRight => (None, None, Precedence::None),
            TokenKind::Char => (None, None, Precedence::None),
            TokenKind::IfFalse => (None, None, Precedence::None),
            TokenKind::Goto => (None, None, Precedence::None),
            TokenKind::Param => (None, None, Precedence::None),
            TokenKind::Call => (None, None, Precedence::None),
            TokenKind::True => (Some(Self::literal), None, Precedence::None),
            TokenKind::False => (Some(Self::literal), None, Precedence::None),
            TokenKind::PrintLn => (None, None, Precedence::None),
            TokenKind::Scan => (None, None, Precedence::None),
            TokenKind::U64KW => (None, None, Precedence::None),
            TokenKind::I64KW => (None, None, Precedence::None),
            TokenKind::F64KW => (None, None, Precedence::None),
            TokenKind::CharKW => (None, None, Precedence::None),
            TokenKind::BoolKW => (None, None, Precedence::None),
            TokenKind::Error => (None, None, Precedence::None),
            TokenKind::Eof => (None, None, Precedence::None),
        };

        CompilerRule::from(rule)
    }

    fn make_constant(&mut self, value: Value) {
        match self.chunk.add_constant(value) {
            Ok(idx) => self.emit_instruction(Instruction::CONSTANT(idx)),
            Err(msg) => self.error(msg),
        }
    }

    fn end(&mut self) {
        self.emit_instruction(Instruction::RETURN);
        #[cfg(feature = "debug_print_code")]
        if self.had_error {
            let disassembler = Disassembler::new(self.chunk);
            disassembler.disassemble("Code");
        }
    }

    fn emit_instruction(&mut self, instruction: Instruction) {
        self.chunk.write(instruction, self.previous.line);
    }

    fn emit_instructions(&mut self, instructions: &[Instruction]) {
        for i in instructions {
            self.emit_instruction(*i);
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) {
        if self.current.kind == kind {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            self.current = self.scanner.next_token();

            if self.current.kind != TokenKind::Error {
                break;
            }

            self.error_at_current(self.current.lexeme);
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current, message)
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous, message)
    }

    fn error_at(&mut self, token: Token<'source>, message: &str) {
        if self.panic_mode {
            return;
        }

        error_at(token, message);
        self.had_error = true;
        self.panic_mode = true;
    }
}
