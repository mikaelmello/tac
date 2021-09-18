use std::collections::HashMap;

use crate::{
    chunk::{Chunk, Instruction},
    compiler::Compiler,
    disassembler::Disassembler,
    error::{TACError, TACResult},
    value::Value,
};

#[derive(Default, Debug)]
pub struct Frame {
    stack: Vec<Value>,
    st: HashMap<String, usize>,
}

pub struct VirtualMachine {
    chunk: Chunk,
    frames: Vec<Frame>,
    ip: usize,
}

macro_rules! binary_op {
    ($self:expr,$oper:tt) => {{
        let b = match $self.current_stack().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let a = match $self.current_stack().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let res = a $oper b;
        match res {
            Ok(val) => $self.current_stack().push(val),
            Err(msg) => return Err($self.report_rte(msg)),
        };
    }};
}

macro_rules! binary_op_f {
    ($self:expr,$oper:ident) => {{
        let b = match $self.current_stack().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let a = match $self.current_stack().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let res = Value::$oper(a, b);
        match res {
            Ok(val) => $self.current_stack().push(val),
            Err(msg) => return Err($self.report_rte(msg)),
        };
    }};
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            frames: vec![],
            ip: 0,
        }
    }

    pub fn interpret(&mut self, source: &str) -> TACResult<()> {
        self.chunk = Chunk::new();
        self.frames.clear();
        self.frames.push(Frame::default());
        self.ip = 0;

        Compiler::compile(source, &mut self.chunk)?;

        self.run()
    }

    fn current_frame(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    fn current_stack(&mut self) -> &mut Vec<Value> {
        &mut self.current_frame().stack
    }

    fn run(&mut self) -> TACResult<()> {
        loop {
            let instruction = match self.chunk.code.get(self.ip) {
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
                let dis = Disassembler::new(&self.chunk);
                dis.instruction(self.ip, &instruction);
            }

            self.ip += 1;

            match instruction {
                Instruction::Halt => return Ok(()),
                Instruction::Return => return self.r#return(),
                Instruction::Negate => self.negate()?,
                Instruction::Not => self.not()?,
                Instruction::Constant(addr) => self.constant(addr)?,
                Instruction::True => self.current_stack().push(Value::Bool(true)),
                Instruction::False => self.current_stack().push(Value::Bool(false)),
                Instruction::Add => binary_op!(self, +),
                Instruction::Subtract => binary_op!(self, -),
                Instruction::Multiply => binary_op!(self, *),
                Instruction::Divide => binary_op!(self, /),
                Instruction::Equal => binary_op_f!(self, eq),
                Instruction::Greater => binary_op_f!(self, gt),
                Instruction::Less => binary_op_f!(self, lt),
                Instruction::Print(nl) => self.print(nl)?,
                Instruction::Pop => self.pop()?,
                Instruction::Goto(ip) => self.ip = ip as usize,
                Instruction::JumpIf(ip) => self.jump_if(ip)?,
            }
        }
    }

    fn r#return(&mut self) -> TACResult<()> {
        Ok(())
    }

    fn print(&mut self, nl: bool) -> TACResult<()> {
        let value = self
            .current_stack()
            .pop()
            .ok_or_else(|| self.report_rte("No value in the stack to print".into()))?;

        let suffix = match nl {
            true => "\n",
            false => "",
        };

        print!("{}{}", value, suffix);
        Ok(())
    }

    fn pop(&mut self) -> TACResult<()> {
        match self.current_stack().pop() {
            Some(_) => Ok(()),
            None => Err(self.report_rte("No value in the stack to pop".into())),
        }
    }

    fn jump_if(&mut self, ip: u16) -> TACResult<()> {
        let value = self
            .current_stack()
            .pop()
            .ok_or_else(|| self.report_rte("No value in the stack to check condition".into()))?;

        match value {
            Value::Bool(true) => {
                self.ip = ip as usize;
                Ok(())
            }
            Value::Bool(false) => Ok(()),
            v => Err(self.report_rte(format!(
                "Invalid type '{}' for condition, 'bool' required.",
                v.type_info(),
            ))),
        }
    }

    fn negate(&mut self) -> TACResult<()> {
        match self
            .current_stack()
            .last_mut()
            .map(Value::arithmetic_negate)
        {
            Some(Ok(_)) => Ok(()),
            Some(Err(msg)) => Err(self.report_rte(msg)),
            None => Err(self.report_rte(
                "Can not apply unary operator '-' because there is not a value in the stack"
                    .to_string(),
            )),
        }
    }

    fn not(&mut self) -> TACResult<()> {
        match self.current_stack().last_mut().map(Value::logic_negate) {
            Some(Ok(_)) => Ok(()),
            Some(Err(msg)) => Err(self.report_rte(msg)),
            None => Err(self.report_rte(
                "Can not apply unary operator '-' because there is not a value in the stack"
                    .to_string(),
            )),
        }
    }

    fn constant(&mut self, addr: u16) -> TACResult<()> {
        let value = self.read_constant(addr)?;
        self.current_stack().push(value);
        Ok(())
    }

    fn read_constant(&mut self, addr: u16) -> TACResult<Value> {
        Ok(self.chunk.get_constant(addr))
    }

    fn report_rte(&self, message: String) -> TACError {
        let line = self.chunk.get_line(self.ip);
        eprintln!("{}", message);
        eprintln!("[line {}] in script", line);

        TACError::RuntimeError
    }
}
