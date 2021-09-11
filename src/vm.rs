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
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let a = match $self.stack.pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let res = a $oper b;
        match res {
            Ok(val) => $self.stack.push(val),
            Err(msg) => Err($self.report_rte(msg))?,
        };
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
        self.stack.clear();
        self.ip = 0;

        Compiler::compile(source, &mut chunk)?;

        self.current_chunk.insert(chunk);

        self.run()
    }

    fn current_chunk(&self) -> TACResult<&Chunk> {
        match self.current_chunk.as_ref() {
            Some(c) => Ok(c),
            None => return Err(self.report_rte("No chunk to run code from".into())),
        }
    }

    fn run(&mut self) -> TACResult<()> {
        loop {
            let instruction = match self.current_chunk()?.code.get(self.ip) {
                Some(i) => *i,
                None => {
                    return Err(self.report_rte(
                        "Instruction pointer reached end of code without a finishing statement"
                            .into(),
                    ))
                }
            };

            #[cfg(feature = "debug_trace_execution")]
            {
                let dis = Disassembler::new(self.current_chunk()?);
                dis.instruction(self.ip, &instruction);
            }

            self.ip += 1;

            match instruction {
                Instruction::RETURN => return self.r#return(),
                Instruction::ADD => binary_op!(self, +),
                Instruction::SUBTRACT => binary_op!(self, -),
                Instruction::MULTIPLY => binary_op!(self, *),
                Instruction::DIVIDE => binary_op!(self, /),
                Instruction::NEGATE => self.negate()?,
                Instruction::CONSTANT(addr) => self.constant(addr)?,
            }
        }
    }

    fn r#return(&mut self) -> TACResult<()> {
        let value = self
            .stack
            .pop()
            .ok_or_else(|| self.report_rte("No value in the stack to return".into()))?;

        println!("{}", value);
        return Ok(());
    }

    fn negate(&mut self) -> TACResult<()> {
        match self.stack.last_mut().map(Value::arithmetic_negate) {
            Some(Ok(_)) => Ok(()),
            Some(Err(msg)) => Err(self.report_rte(msg)),
            None => Err(self.report_rte(format!(
                "Can not apply unary operator '-' because there is not a value in the stack"
            ))),
        }
    }

    fn constant(&mut self, addr: u16) -> TACResult<()> {
        let value = self.read_constant(addr)?;
        self.stack.push(value);
        Ok(())
    }

    fn read_constant(&mut self, addr: u16) -> TACResult<Value> {
        Ok(self.current_chunk()?.get_constant(addr))
    }

    fn report_rte(&self, message: String) -> TACError {
        let line = self.current_chunk().unwrap().get_line(self.ip);
        eprintln!("{}", message);
        eprintln!("[line {}] in script", line);

        TACError::RuntimeError
    }
}
