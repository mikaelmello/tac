use chunk::{Chunk, Instruction};
use disassembler::Disassembler;
use value::Value;

mod chunk;
mod disassembler;
mod value;

fn main() {
    let mut chunk = Chunk::new();
    chunk.add_constant(Value::F64(19923.0));

    chunk.write(Instruction::RETURN, 0);
    chunk.write(Instruction::CONSTANT(0), 0);

    let disassembler = Disassembler::new(&chunk);
    disassembler.disassemble("Test Chunk");
}
