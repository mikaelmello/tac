pub enum Instruction {
    Return,
}

pub struct Chunk {
    code: Vec<Instruction>,
}

impl Chunk {
    pub fn new() -> Self {
        Self { code: vec![] }
    }

    pub fn write(&mut self, i: Instruction) {
        self.code.push(i);
    }

    pub fn code(&self) -> &[Instruction] {
        &self.code
    }
}
