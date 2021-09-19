use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
};

use crate::{
    chunk::{Chunk, Instruction},
    compiler::Compiler,
    error::{TACError, TACResult},
    value::Value,
};

type SymbolTable = HashMap<u16, usize>;

#[derive(Default, Debug)]
pub struct Frame {
    stack: Vec<Value>,
    st: SymbolTable,
    ra: Option<usize>,
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

            {
                let trace_execution = crate::TRACE_EXECUTION.read().unwrap();
                if *trace_execution {
                    let dis = crate::disassembler::Disassembler::new(&self.chunk);
                    dis.instruction(self.ip, &instruction);
                }
            }

            self.ip += 1;

            match instruction {
                Instruction::Halt => return Ok(()),
                Instruction::Return => {
                    if self.r#return() {
                        return Ok(());
                    }
                }
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
                Instruction::Modulo => binary_op!(self, %),
                Instruction::ShiftLeft => binary_op!(self, <<),
                Instruction::ShiftRight => binary_op!(self, >>),
                Instruction::Equal => binary_op_f!(self, eq),
                Instruction::Greater => binary_op_f!(self, gt),
                Instruction::Less => binary_op_f!(self, lt),
                Instruction::Print(nl) => self.print(nl)?,
                Instruction::Pop => self.pop()?,
                Instruction::Goto(ip) => self.ip = ip as usize,
                Instruction::JumpIf(ip) => self.jump_if(ip)?,
                Instruction::Assign => self.assign()?,
                Instruction::Call(ip) => self.call(ip)?,
            }
        }
    }

    fn r#return(&mut self) -> bool {
        let ra = self.get_current_frame().ra;

        if let Some(ip) = ra {
            self.frames.pop();
            self.ip = ip;
            false
        } else {
            self.frames.pop();
            true
        }
    }

    fn call(&mut self, ip: u16) -> TACResult<()> {
        let param_count = self.get_current_stack_mut().pop().ok_or_else(|| {
            self.report_rte(
                "No value in the stack to define how many parameters to call function".into(),
            )
        })?;

        let mut parameters = vec![];

        if let Value::U64(count) = param_count {
            for i in 0..count {
                parameters.push(self.get_current_stack_mut().pop().ok_or_else(|| {
                    self.report_rte(format!(
                        "Method called with {} parameters but {} were found in the stack",
                        count, i
                    ))
                })?);
            }
        } else {
            return Err(self.report_rte(format!(
                "Parameter count must be defined by a variable of type u64 but found type {}",
                param_count.type_info()
            )));
        }

        // get string id of "args" name
        let args_chunk_addr = self
            .chunk
            .add_name("args")
            .map_err(|_| self.report_rte("The program uses too many variables (65535+)".into()))?;
        // get string id of "argc" name
        let argc_chunk_addr = self
            .chunk
            .add_name("argc")
            .map_err(|_| self.report_rte("The program uses too many variables (65535+)".into()))?;

        // push new empty frame
        let frame = Frame {
            ra: Some(self.ip),
            ..Default::default()
        };
        self.frames.push(frame);

        // insert "argc" variable in symbol table, address 0: beginning of the stack
        self.get_current_st_mut().insert(argc_chunk_addr, 0);

        // push argc to stack
        self.get_current_stack_mut()
            .push(Value::U64(parameters.len().try_into().unwrap()));

        if !parameters.is_empty() {
            // insert "args" variable in symbol table, address 1: just after beginning of the stack
            self.get_current_st_mut().insert(args_chunk_addr, 1);

            // push args to stack
            for p in parameters {
                self.get_current_stack_mut().push(p);
            }
        }

        self.ip = ip as usize;

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
