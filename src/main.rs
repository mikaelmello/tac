use chunk::{Chunk, Instruction};
use disassembler::Disassembler;
use error::TACResult;
use value::Value;
use vm::VirtualMachine;

mod chunk;
mod disassembler;
mod error;
mod value;
mod vm;

fn main() -> TACResult<()> {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value::F64(1.2));

    chunk.write(Instruction::CONSTANT(constant), 0);

    let constant = chunk.add_constant(Value::F64(3.4));
    chunk.write(Instruction::CONSTANT(constant), 0);

    chunk.write(Instruction::ADD, 0);

    let constant = chunk.add_constant(Value::F64(5.6));
    chunk.write(Instruction::CONSTANT(constant), 0);

    chunk.write(Instruction::DIVIDE, 0);
    chunk.write(Instruction::NEGATE, 0);
    chunk.write(Instruction::RETURN, 0);

    let mut vm = VirtualMachine::new();
    vm.interpret(chunk)?;

    // let disassembler = Disassembler::new(&chunk);
    // disassembler.disassemble("Test Chunk");

    Ok(())
}
