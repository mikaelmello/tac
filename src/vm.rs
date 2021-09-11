use crate::{
    chunk::{Chunk, Instruction},
    compiler::Compiler,
    disassembler::Disassembler,
    error::{TACError, TACResult},
    value::Value,
};

pub struct VirtualMachine {
    current_chunk: Option<Chunk>,
    stack: Vec<Value>,
    ip: usize,
}

macro_rules! binary_op {
    ($self:expr,$oper:tt) => {{
        let b = match $self.stack.pop() {
            Some(val) => val,
            None => Err($self.runtime_error())?,
        };
        let a = match $self.stack.pop() {
            Some(val) => val,
            None => Err($self.runtime_error())?,
        };
        let res = a $oper b;
        $self.stack.push(res?);
    }};
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            current_chunk: None,
            stack: vec![],
            ip: 0,
        }
    }

    pub fn interpret(&mut self, source: &str) -> TACResult<()> {
        let mut chunk = Chunk::new();
        self.stack = vec![];
        self.ip = 0;

        Compiler::compile(source, &mut chunk)?;

        self.current_chunk.insert(chunk);

        self.run()
    }

    fn current_chunk(&self) -> TACResult<&Chunk> {
        match self.current_chunk.as_ref() {
            Some(c) => Ok(c),
            None => return Err(TACError::RuntimeError),
        }
    }

    fn run(&mut self) -> TACResult<()> {
        loop {
            let instruction = match self.current_chunk()?.code.get(self.ip) {
                Some(i) => *i,
                None => return Err(TACError::RuntimeError),
            };

            #[cfg(feature = "debug_trace_execution")]
            {
                let dis = Disassembler::new(self.current_chunk()?);
                dis.instruction(self.ip, &instruction);
            }

            self.ip += 1;

            match instruction {
                Instruction::RETURN => {
                    let value = self.stack.pop().ok_or(TACError::RuntimeError)?;
                    println!("{}", value);
                    return Ok(());
                }
                Instruction::ADD => binary_op!(self, +),
                Instruction::SUBTRACT => binary_op!(self, -),
                Instruction::MULTIPLY => binary_op!(self, *),
                Instruction::DIVIDE => binary_op!(self, /),
                Instruction::NEGATE => match self.stack.last_mut() {
                    Some(val) => val.negate()?,
                    None => Err(self.runtime_error())?,
                },
                Instruction::CONSTANT(addr) => {
                    let value = self.read_constant(addr)?;
                    self.stack.push(value);
                }
            }
        }
    }

    fn read_constant(&mut self, addr: u16) -> TACResult<Value> {
        Ok(self.current_chunk()?.get_constant(addr))
    }

    fn runtime_error(&mut self) -> TACError {
        TACError::RuntimeError
    }
}
