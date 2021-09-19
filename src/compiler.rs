use std::collections::HashMap;

use crate::{
    chunk::{Chunk, Instruction},
    error::{error_at, TACError, TACResult},
    scanner::Scanner,
    token::{Token, TokenKind},
    value::Value,
};

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
            TokenKind::Star => self.assignment(),
            TokenKind::Identifier => self.label_or_assignment(),

            TokenKind::NewLine | TokenKind::Eof => return,
            TokenKind::Equal => self.error("Assignments must have a variable on the left side"),
            TokenKind::Scan => self.error("Return value of scan must be assigned to a variable"),
            k => self.error(&format!("Invalid statement with token {:?}", k)),
        }

        match self.current.kind {
            TokenKind::NewLine | TokenKind::Eof => {}
            TokenKind::BangEqual
            | TokenKind::EqualEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::ShiftLeft
            | TokenKind::ShiftRight
            | TokenKind::Bang
            | TokenKind::Ampersand => self
                .error_at_current("Three-address code programs support at most binary expressions"),
            _ => self.error_at_current("There must be at most one statement per line"),
        }
    }

    fn label_or_assignment(&mut self) {
        let identifier = self.previous;

        if self.match_advance(TokenKind::Colon) {
            if self.labels.get(identifier.lexeme).is_some() {
                self.error_at(identifier, "Redefinition of labels is not allowed");
            } else {
                self.labels.insert(identifier.lexeme, self.chunk.code.len());
            }
        } else {
            self.assignment();
        }
    }

    fn assignment(&mut self) {
        let dereference = self.previous.kind == TokenKind::Star;

        if self.previous.kind == TokenKind::Star {
            self.consume(
                TokenKind::Identifier,
                "A variable is required to be dereferenced",
            );
        }

        let identifier = match self.chunk.add_name(self.previous.lexeme) {
            Ok(addr) => addr,
            Err(_) => return self.error("The program uses too many variables (65535+)"),
        };

        if self.match_advance(TokenKind::LeftBracket) {
            if dereference {
                return self.error("Dereferenced variables can not be accessed via array indexes in the same statement");
            }

            self.array_subscript();
            self.consume(
                TokenKind::RightBracket,
                "Missing ']': Array accesses must be enclosed by brackets",
            );
        }

        self.consume(
            TokenKind::Equal,
            "Assignment statement expected, but no '=' was found",
        );

        self.emit_instruction(Instruction::GetOrCreateVar(identifier));
        self.expression();
        self.emit_instruction(Instruction::Assign);
    }

    fn array_subscript(&mut self) {}

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

        let pending = self.pending_labels.entry(label).or_insert_with(Vec::new);
        pending.push((self.chunk.code.len(), self.previous.line));

        self.emit_instruction(Instruction::JumpIf(0));
    }

    fn goto_statement(&mut self) {
        self.consume(TokenKind::Identifier, "Missing label for 'goto' statement");

        let label = self.previous.lexeme;

        let pending = self.pending_labels.entry(label).or_insert_with(Vec::new);
        pending.push((self.chunk.code.len(), self.previous.line));

        self.emit_instruction(Instruction::Goto(0));
    }

    fn print_statement(&mut self) {
        let nl = self.previous.kind == TokenKind::PrintLn;

        self.expression();
        self.emit_instruction(Instruction::Print(nl))
    }

    fn expression(&mut self) {
        if self.unary_expression().is_some() {
            return;
        }

        if self.current.kind == TokenKind::Call {
            self.advance();
            self.operand();
            self.operand();
            todo!("self.emit_instruction(Instruction::Call); return;");
        }

        if self.current.kind == TokenKind::Scan {
            self.advance();
            todo!("Scan expression");
        }

        self.operand();

        macro_rules! simple_bin_op {
            ($is:expr) => {{
                self.advance();
                self.operand();
                self.emit_instructions($is);
            }};
        }

        match self.current.kind {
            TokenKind::BangEqual => simple_bin_op!(&[Instruction::Equal, Instruction::Not]),
            TokenKind::EqualEqual => simple_bin_op!(&[Instruction::Equal]),
            TokenKind::Greater => simple_bin_op!(&[Instruction::Greater]),
            TokenKind::GreaterEqual => simple_bin_op!(&[Instruction::Less, Instruction::Less]),
            TokenKind::Less => simple_bin_op!(&[Instruction::Less]),
            TokenKind::LessEqual => simple_bin_op!(&[Instruction::Less, Instruction::Greater]),
            TokenKind::Minus => simple_bin_op!(&[Instruction::Subtract]),
            TokenKind::Plus => simple_bin_op!(&[Instruction::Add]),
            TokenKind::Star => simple_bin_op!(&[Instruction::Multiply]),
            TokenKind::Slash => simple_bin_op!(&[Instruction::Divide]),
            TokenKind::Percent => todo!("&[Instruction::Modulo]"),
            TokenKind::ShiftLeft => todo!("simple_bin_op!(&[Instruction::Shl])"),
            TokenKind::ShiftRight => todo!("simple_bin_op!(&[Instruction::Shr])"),
            _ => {}
        };
    }

    fn unary_expression(&mut self) -> Option<()> {
        let unary_op = match self.current.kind {
            TokenKind::Bang => Some(Instruction::Not),
            TokenKind::Minus => Some(Instruction::Negate),
            TokenKind::Star => todo!("Some(Instruction::Dereference"),
            TokenKind::Ampersand => todo!("Some(Instruction::Reference"),
            _ => None,
        };

        if let Some(instruction) = unary_op {
            self.advance();
            self.operand();
            self.emit_instruction(instruction);
            Some(())
        } else {
            None
        }
    }

    fn operand(&mut self) {
        self.advance();

        match self.previous.kind {
            TokenKind::Identifier => {
                let addr = match self.chunk.add_name(self.previous.lexeme) {
                    Ok(addr) => addr,
                    Err(_) => return self.error("The program uses too many variables (65535+)"),
                };
                self.emit_instruction(Instruction::GetVar(addr));
            }
            TokenKind::True => self.emit_instruction(Instruction::True),
            TokenKind::False => self.emit_instruction(Instruction::False),
            TokenKind::Char => self.char(),
            TokenKind::Number => self.number(),

            // errors
            TokenKind::String => self.error("String literals are only allowed in the data section"),
            _ => self.error("Invalid operand, expected literal value or variable name"),
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
            }
        };
    }

    fn r#char(&mut self) {
        assert_eq!(TokenKind::Char, self.previous.kind);

        let token = self.previous;
        let lexeme = token.lexeme;
        let character = lexeme.chars().nth(1);

        match character {
            Some(c) => self.make_constant(Value::Char(c)),
            None => panic!("Invalid token of kind Char"),
        }
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

        for (label, first_use) in missing_labels {
            self.error(&format!(
                "Missing label '{}', first used in line {}",
                label, first_use
            ));
        }

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
            let disassembler = crate::disassembler::Disassembler::new(self.chunk);
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

        false
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
