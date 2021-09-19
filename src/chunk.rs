use crate::value::Value;
use std::{collections::HashMap, convert::TryFrom};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    Return,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    Negate,
    Call(u16),
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    GetVar(u16),
    GetOrCreateVar(u16),
    Assign,
    JumpIf(u16),
    Goto(u16),
    Pop,
    Print(bool),
    Constant(u16),
    Halt,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LineStart {
    offset: usize,
    line: usize,
}

impl LineStart {
    pub fn new(offset: usize, line: usize) -> Self {
        Self { offset, line }
    }
}

#[derive(Default)]
pub struct Chunk {
    pub code: Vec<Instruction>,
    constants: Vec<Value>,
    names: Vec<String>,
    names_rev: HashMap<String, u16>,
    lines: Vec<LineStart>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, i: Instruction, line: usize) -> usize {
        if let Some(line_start) = self.lines.last() {
            assert!(
                line_start.line <= line,
                "Line of new instruction cannot be smaller than previous instruction"
            )
        }

        let index = self.code.len();

        self.code.push(i);

        match self.lines.last() {
            Some(cur_line) if cur_line.line == line => {}
            _ => self.lines.push(LineStart::new(index, line)),
        };

        index
    }

    pub fn add_constant(&mut self, value: Value) -> Result<u16, &'static str> {
        let index = self.constants.len();

        match u16::try_from(index) {
            Ok(index) => {
                self.constants.push(value);
                Ok(index)
            }
            Err(_) => Err("Could not add constant, reached limit of u16 max size"),
        }
    }

    pub fn get_constant(&self, addr: u16) -> Value {
        let idx = usize::from(addr);
        self.constants
            .get(idx)
            .copied()
            .expect("Could not get constant")
    }

    pub fn add_name(&mut self, name: &str) -> Result<u16, &'static str> {
        if let Some(addr) = self.names_rev.get(name) {
            return Ok(*addr);
        }

        let index = self.names.len();

        match u16::try_from(index) {
            Ok(index) => {
                self.names.push(name.to_string());
                self.names_rev.insert(name.to_string(), index);
                Ok(index)
            }
            Err(_) => Err("Could not add name, reached limit of u16 max size"),
        }
    }

    pub fn get_name(&self, addr: u16) -> &str {
        let idx = usize::from(addr);
        self.names.get(idx).expect("Could not get name")
    }

    pub fn get_line(&self, instruction_idx: usize) -> usize {
        if instruction_idx >= self.code.len() {
            eprintln!(
                "Trying to get line of instruction {} where only {} exist",
                instruction_idx,
                self.code.len()
            );
        }

        if instruction_idx >= self.code.len() {
            eprintln!(
                "Trying to get line of instruction {} when no line information exists",
                instruction_idx,
            );
        }

        let mut left = 0;
        let mut right = self.lines.len() - 1;

        let mut line = self.lines.last().expect("Lines is empty").line;

        while left <= right {
            let mid = (left + right) / 2;

            match self.lines.get(mid) {
                Some(mid_line) => {
                    if instruction_idx >= mid_line.offset {
                        line = mid_line.line;

                        if mid == 0 {
                            break;
                        }
                        left = mid + 1;
                    } else {
                        right = mid - 1;
                    }
                }
                None => panic!("Invalid mid index when looking for line"),
            }
        }

        line
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use crate::chunk::Instruction;

    #[test]
    fn instruction_is_at_most_64_bits() {
        // An instruction should be at most 64 bits; anything bigger and we've mis-defined some
        // variant
        assert!(size_of::<Instruction>() <= 4);
    }
}
