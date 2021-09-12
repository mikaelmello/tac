use std::collections::HashMap;

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
    labels: HashMap<&'source str, usize>,
    pending_labels: HashMap<&'source str, Vec<(usize, usize)>>,
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
            labels: HashMap::new(),
            pending_labels: HashMap::new(),
        };

        compiler.advance();

        while !compiler.match_advance(TokenKind::Eof) {
            compiler.statement();

            if compiler.panic_mode {
                compiler.synchronize();
            }
        }
        compiler.end();

        match compiler.had_error {
            false => Ok(()),
            true => Err(TACError::CompileError),
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.kind != TokenKind::Eof {
            if self.previous.kind == TokenKind::NewLine {
                return;
            }

            self.advance();
        }
    }

    fn statement(&mut self) {
        self.advance();

        match self.previous.kind {
            TokenKind::Print | TokenKind::PrintLn => self.print_statement(),
            TokenKind::If | TokenKind::IfFalse => self.if_statement(),
            TokenKind::Goto => self.goto_statement(),
            TokenKind::Halt => self.emit_instruction(Instruction::Halt),

            TokenKind::NewLine => return,
            TokenKind::Equal => self.error("Assignments must have a variable on the left side"),
            TokenKind::Scan => self.error("Return value of scan must be assigned to a variable"),
            _ => self.error("Invalid statement"),
        }

        match self.current.kind {
            TokenKind::NewLine | TokenKind::Eof => return,
            _ => self.error_at_current("There must be at most one statement per line"),
        }
    }

    fn r#return(&mut self) {}

    fn if_statement(&mut self) {
        let negate = self.previous.kind == TokenKind::IfFalse;
        let statement = match self.previous.kind {
            TokenKind::If => "if",
            TokenKind::IfFalse => "ifFalse",
            _ => panic!("Invalid token in if_statement()"),
        };

        self.expression();
        self.consume(
            TokenKind::Goto,
            &format!("Missing 'goto' keyword after {} statement", statement),
        );
        self.consume(
            TokenKind::Identifier,
            &format!("Missing label after {} statement", statement),
        );

        let label = self.previous.lexeme;

        if negate {
            self.emit_instruction(Instruction::Not);
        }

        let pending = self.pending_labels.entry(label).or_insert_with(|| vec![]);
        pending.push((self.chunk.code.len(), self.previous.line));

        self.emit_instruction(Instruction::JumpIf(0));
    }

    fn goto_statement(&mut self) {
        self.consume(TokenKind::Identifier, "Missing label for 'goto' statement");

        let label = self.previous.lexeme;

        let pending = self.pending_labels.entry(label).or_insert_with(|| vec![]);
        pending.push((self.chunk.code.len(), self.previous.line));

        self.emit_instruction(Instruction::Goto(0));
    }

    fn print_statement(&mut self) {
        let nl = self.previous.kind == TokenKind::PrintLn;

        self.expression();
        self.emit_instruction(Instruction::Print(nl))
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
            TokenKind::Minus => self.emit_instruction(Instruction::Negate),
            TokenKind::Bang => self.emit_instruction(Instruction::Not),
            _ => panic!("Unreachable - match arm not covered in unary()"),
        }
    }

    fn binary(&mut self) {
        let kind = self.previous.kind;
        self.parse_precedence(Precedence::Primary);

        match kind {
            TokenKind::Plus => self.emit_instruction(Instruction::Add),
            TokenKind::Minus => self.emit_instruction(Instruction::Subtract),
            TokenKind::Slash => self.emit_instruction(Instruction::Divide),
            TokenKind::Star => self.emit_instruction(Instruction::Multiply),
            TokenKind::EqualEqual => self.emit_instruction(Instruction::Equal),
            TokenKind::BangEqual => self.emit_instructions(&[Instruction::Equal, Instruction::Not]),
            TokenKind::Greater => self.emit_instruction(Instruction::Greater),
            TokenKind::GreaterEqual => {
                self.emit_instructions(&[Instruction::Less, Instruction::Not])
            }
            TokenKind::Less => self.emit_instruction(Instruction::Less),
            TokenKind::LessEqual => {
                self.emit_instructions(&[Instruction::Greater, Instruction::Not])
            }
            _ => panic!("Unreachable - match arm not covered in binary()"),
        }
    }

    fn literal(&mut self) {
        match self.previous.kind {
            TokenKind::False => self.emit_instruction(Instruction::False),
            TokenKind::True => self.emit_instruction(Instruction::True),
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
            TokenKind::Halt => (None, None, Precedence::None),
            TokenKind::NewLine => (None, None, Precedence::None),
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
            Ok(idx) => self.emit_instruction(Instruction::Constant(idx)),
            Err(msg) => self.error(msg),
        }
    }

    fn update_pending_labels(&mut self) {
        let mut patches: Vec<(usize, usize)> = vec![];
        let mut missing_labels: Vec<(&str, usize)> = vec![];

        for (k, v) in &self.pending_labels {
            if let Some(idx) = self.labels.get(k) {
                for (instruction_idx, _) in v {
                    patches.push((*instruction_idx, *idx));
                }
            } else if let Some((_, first_use)) = v.first() {
                missing_labels.push((*k, *first_use));
            }
        }

        // for (label, first_use) in missing_labels {
        //     self.error(&format!(
        //         "Missing label '{}', first used in line {}",
        //         label, first_use
        //     ));
        // }

        for (idx, val) in patches {
            self.patch_jump(idx, val as u16)
        }
    }

    fn patch_jump(&mut self, idx: usize, val: u16) {
        match self.chunk.code.get_mut(idx) {
            Some(i) => match i {
                Instruction::Goto(idx) => *idx = val,
                Instruction::JumpIf(idx) => *idx = val,
                _ => panic!("Patching jump led to invalid instruction"),
            },
            None => panic!("Patching jump led to invalid index"),
        }
    }

    fn end(&mut self) {
        if self.chunk.code.last() != Some(&Instruction::Halt) {
            self.emit_instruction(Instruction::Halt);
        }

        self.update_pending_labels();

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
        if self.match_advance(kind) {
            return;
        }

        self.error_at_current(message);
    }

    fn match_advance(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            return true;
        }

        return false;
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        self.current.kind == kind
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
