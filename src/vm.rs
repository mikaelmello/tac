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
            Err(msg) => return Err($self.report_rte(msg)),
        };
    }};
}

macro_rules! binary_op_f {
    ($self:expr,$oper:ident) => {{
        let b = match $self.stack.pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let a = match $self.stack.pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let res = Value::$oper(a, b);
        match res {
            Ok(val) => $self.stack.push(val),
            Err(msg) => return Err($self.report_rte(msg)),
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

        self.current_chunk = Some(chunk);

        self.run()
    }

    fn current_chunk(&self) -> TACResult<&Chunk> {
        match self.current_chunk.as_ref() {
            Some(c) => Ok(c),
            None => Err(self.report_rte("No chunk to run code from".into())),
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
                Instruction::Halt => return Ok(()),
                Instruction::Return => return self.r#return(),
                Instruction::Negate => self.negate()?,
                Instruction::Not => self.not()?,
                Instruction::Constant(addr) => self.constant(addr)?,
                Instruction::True => self.stack.push(Value::Bool(true)),
                Instruction::False => self.stack.push(Value::Bool(false)),
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
            .stack
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
        match self.stack.pop() {
            Some(_) => Ok(()),
            None => Err(self.report_rte("No value in the stack to pop".into())),
        }
    }

    fn jump_if(&mut self, ip: u16) -> TACResult<()> {
        let value = self
            .stack
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
        match self.stack.last_mut().map(Value::arithmetic_negate) {
            Some(Ok(_)) => Ok(()),
            Some(Err(msg)) => Err(self.report_rte(msg)),
            None => Err(self.report_rte(
                "Can not apply unary operator '-' because there is not a value in the stack"
                    .to_string(),
            )),
        }
    }

    fn not(&mut self) -> TACResult<()> {
        match self.stack.last_mut().map(Value::logic_negate) {
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
