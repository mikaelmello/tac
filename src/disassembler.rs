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

        match instruction {
            Instruction::Return => println!("RETURN"),
        }
    }
}
