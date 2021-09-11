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
        print!("{:04} ", idx);

        let line = self.chunk.get_line(idx);

        if idx > 0 && line == self.chunk.get_line(idx - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", line);
        }

        match instruction {
            Instruction::RETURN => println!("RETURN"),
            Instruction::ADD => println!("ADD"),
            Instruction::SUBTRACT => println!("SUBTRACT"),
            Instruction::MULTIPLY => println!("MULTIPLY"),
            Instruction::DIVIDE => println!("DIVIDE"),
            Instruction::NEGATE => println!("NEGATE"),
            Instruction::NOT => println!("NOT"),
            Instruction::CONSTANT(addr) => self.constant("CONSTANT", *addr),
            Instruction::TRUE => println!("TRUE"),
            Instruction::FALSE => println!("FALSE"),
            Instruction::EQUAL => println!("EQUAL"),
            Instruction::GREATER => println!("GREATER"),
            Instruction::LESS => println!("LESS"),
            Instruction::PRINT(nl) => println!("PRINT nl:{}", nl),
            Instruction::POP => println!("POP"),
            Instruction::HALT => println!("HALT"),
        }
    }

    fn constant(&self, name: &str, addr: u16) {
        let value = self.chunk.get_constant(addr);
        println!("{:16} {:4} '{}'", name, addr, value);
    }
}
