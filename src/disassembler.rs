use crate::chunk::{Chunk, Instruction};

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk }
    }

    pub fn disassemble(self, name: &str) {
        println!("=== {} ===", name);

        for (idx, instruction) in self.chunk.code.iter().enumerate() {
            self.instruction(idx, instruction);
        }
    }

    pub fn instruction(&self, idx: usize, instruction: &'a Instruction) {
        eprint!("{:04} ", idx);

        let line = self.chunk.get_line(idx);

        if idx > 0 && line == self.chunk.get_line(idx - 1) {
            eprint!("   | ");
        } else {
            eprint!("{:4} ", line);
        }

        match instruction {
            Instruction::Return => eprintln!("RETURN"),
            Instruction::Add => eprintln!("ADD"),
            Instruction::Subtract => eprintln!("SUBTRACT"),
            Instruction::Multiply => eprintln!("MULTIPLY"),
            Instruction::Divide => eprintln!("DIVIDE"),
            Instruction::Negate => eprintln!("NEGATE"),
            Instruction::Modulo => eprintln!("MODULO"),
            Instruction::ShiftLeft => eprintln!("SHIFT_LEFT"),
            Instruction::ShiftRight => eprintln!("SHIFT_RIGHT"),
            Instruction::Not => eprintln!("NOT"),
            Instruction::Constant(addr) => self.constant("CONSTANT", *addr),
            Instruction::GetOrCreateVar(addr) => self.name("GET_OR_CREATE_VAR", *addr),
            Instruction::GetVar(addr) => self.name("GET_VAR", *addr),
            Instruction::True => eprintln!("TRUE"),
            Instruction::False => eprintln!("FALSE"),
            Instruction::Equal => eprintln!("EQUAL"),
            Instruction::Greater => eprintln!("GREATER"),
            Instruction::Less => eprintln!("LESS"),
            Instruction::Print(nl) => eprintln!("PRINT nl:{}", nl),
            Instruction::Pop => eprintln!("POP"),
            Instruction::Halt => eprintln!("HALT"),
            Instruction::Goto(ip) => eprintln!("JUMP {:04}", ip),
            Instruction::JumpIf(ip) => eprintln!("JUMP {:04}", ip),
            Instruction::Assign => eprintln!("ASSIGN"),
        }
    }

    fn constant(&self, name: &str, addr: u16) {
        let value = self.chunk.get_constant(addr);
        eprintln!("{:16} {:4} '{}'", name, addr, value);
    }

    fn name(&self, name: &str, addr: u16) {
        let value = self.chunk.get_name(addr);
        eprintln!("{:16} {:4} '{}'", name, addr, value);
    }
}
