use chunk::{Chunk, Instruction};
use disassembler::Disassembler;

mod chunk;
mod disassembler;
mod value;

fn main() {
    let mut chunk = Chunk::new();

    chunk.write(Instruction::Return);

    let disassembler = Disassembler::new(&chunk);
    disassembler.disassemble("Test Chunk");
}
