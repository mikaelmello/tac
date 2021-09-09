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

        for (idx, instruction) in self.chunk.code().into_iter().enumerate() {
            self.disassemble_instruction(idx, instruction);
        }
    }

    pub fn disassemble_instruction(&self, idx: usize, instruction: &'a Instruction) {
        print!("{:04} ", idx);

        let line = self.chunk.get_line(idx);

        if idx > 0 && line == self.chunk.get_line(idx - 1) {
            print!("   | ");
        } else {
            print!("{:4} ", line);
        }

        match instruction {
            Instruction::RETURN => println!("RETURN"),
            Instruction::CONSTANT(addr) => self.constant("CONSTANT", *addr),
        }
    }

    fn constant(&self, name: &str, addr: u16) {
        let value = self.chunk.get_constant(addr);
        println!("{:16} {:4} '{}'", name, addr, value);
    }
}
