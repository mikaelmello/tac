use std::collections::{hash_map::Entry, HashMap};

use crate::{
    chunk::{Chunk, Instruction},
    compiler::Compiler,
    disassembler::Disassembler,
    error::{TACError, TACResult},
    value::Value,
};

type SymbolTable = HashMap<u16, usize>;

#[derive(Default, Debug)]
pub struct Frame {
    stack: Vec<Value>,
    st: SymbolTable,
}

pub struct VirtualMachine {
    chunk: Chunk,
    frames: Vec<Frame>,
    ip: usize,
}

macro_rules! binary_op {
    ($self:expr,$oper:tt) => {{
        let b = match $self.get_current_stack_mut().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let a = match $self.get_current_stack_mut().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let res = a $oper b;
        match res {
            Ok(val) => $self.get_current_stack_mut().push(val),
            Err(msg) => return Err($self.report_rte(msg)),
        };
    }};
}

macro_rules! binary_op_f {
    ($self:expr,$oper:ident) => {{
        let b = match $self.get_current_stack_mut().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let a = match $self.get_current_stack_mut().pop() {
            Some(val) => val,
            None => return Err($self.report_rte(format!("Can not apply operator '{}' because there are not enough values in the stack", stringify!($oper)))),
        };
        let res = Value::$oper(a, b);
        match res {
            Ok(val) => $self.get_current_stack_mut().push(val),
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

    fn get_current_frame(&self) -> &Frame {
        self.frames.last().unwrap()
    }

    fn get_current_frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    fn get_current_stack(&self) -> &Vec<Value> {
        &self.get_current_frame().stack
    }

    fn get_current_stack_mut(&mut self) -> &mut Vec<Value> {
        &mut self.get_current_frame_mut().stack
    }

    fn get_current_st(&self) -> &SymbolTable {
        &self.get_current_frame().st
    }

    fn get_current_st_mut(&mut self) -> &mut SymbolTable {
        &mut self.get_current_frame_mut().st
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
                Instruction::GetVar(name_addr) => self.get_var(name_addr)?,
                Instruction::GetOrCreateVar(name_addr) => self.get_or_create_var(name_addr),
                Instruction::True => self.get_current_stack_mut().push(Value::Bool(true)),
                Instruction::False => self.get_current_stack_mut().push(Value::Bool(false)),
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
                Instruction::Assign => self.assign()?,
            }
        }
    }

    fn r#return(&mut self) -> TACResult<()> {
        Ok(())
    }

    fn assign(&mut self) -> TACResult<()> {
        let value = self
            .get_current_stack_mut()
            .pop()
            .ok_or_else(|| self.report_rte("No value in the stack to assign".into()))?;

        let addr = self
            .get_current_stack_mut()
            .pop()
            .ok_or_else(|| self.report_rte("No address in the stack to assign to".into()))?;

        if let Value::Addr(addr) = addr {
            let map = match self.get_current_stack_mut().get_mut(addr) {
                Some(val) => val,
                None => {
                    return Err(
                        self.report_rte("Assignment target points to invalid stack address".into())
                    )
                }
            };

            *map = value;
            Ok(())
        } else {
            Err(self.report_rte("Assignment target in stack is not valid".into()))
        }
    }

    fn print(&mut self, nl: bool) -> TACResult<()> {
        let value = self
            .get_current_stack_mut()
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
        match self.get_current_stack_mut().pop() {
            Some(_) => Ok(()),
            None => Err(self.report_rte("No value in the stack to pop".into())),
        }
    }

    fn jump_if(&mut self, ip: u16) -> TACResult<()> {
        let value = self
            .get_current_stack_mut()
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
            .get_current_stack_mut()
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
        match self
            .get_current_stack_mut()
            .last_mut()
            .map(Value::logic_negate)
        {
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
        self.get_current_stack_mut().push(value);
        Ok(())
    }

    fn get_var(&mut self, name_addr: u16) -> TACResult<()> {
        let addr = match self.get_current_st().get(&name_addr) {
            Some(addr) => *addr,
            None => {
                return Err(self.report_rte(format!(
                    "Variable {} is undefined",
                    self.chunk.get_name(name_addr)
                )))
            }
        };

        if let Some(value) = self.get_current_stack().get(addr).copied() {
            self.get_current_stack_mut().push(value);

            Ok(())
        } else {
            Err(self.report_rte(format!(
                "Variable {} has invalid address on symbol table",
                self.chunk.get_name(name_addr)
            )))
        }
    }

    fn get_or_create_var(&mut self, name_addr: u16) {
        let cur_sp = self.get_current_stack_mut().len();
        let addr = match self.get_current_st_mut().entry(name_addr) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                entry.insert(cur_sp);
                self.get_current_stack_mut().push(Value::U64(0));
                cur_sp
            }
        };

        self.get_current_stack_mut().push(Value::Addr(addr));
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
