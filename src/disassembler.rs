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
            Instruction::RETURN => eprintln!("RETURN"),
            Instruction::ADD => eprintln!("ADD"),
            Instruction::SUBTRACT => eprintln!("SUBTRACT"),
            Instruction::MULTIPLY => eprintln!("MULTIPLY"),
            Instruction::DIVIDE => eprintln!("DIVIDE"),
            Instruction::NEGATE => eprintln!("NEGATE"),
            Instruction::NOT => eprintln!("NOT"),
            Instruction::CONSTANT(addr) => self.constant("CONSTANT", *addr),
            Instruction::TRUE => eprintln!("TRUE"),
            Instruction::FALSE => eprintln!("FALSE"),
            Instruction::EQUAL => eprintln!("EQUAL"),
            Instruction::GREATER => eprintln!("GREATER"),
            Instruction::LESS => eprintln!("LESS"),
            Instruction::PRINT(nl) => eprintln!("PRINT nl:{}", nl),
            Instruction::POP => eprintln!("POP"),
            Instruction::HALT => eprintln!("HALT"),
            Instruction::GOTO(ip) => eprintln!("JUMP {:04}", ip),
        }
    }

    fn constant(&self, name: &str, addr: u16) {
        let value = self.chunk.get_constant(addr);
        eprintln!("{:16} {:4} '{}'", name, addr, value);
    }
}
