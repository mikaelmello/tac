use std::{
    convert::TryFrom,
    fs::{self, File},
    io::{self, BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use console::style;
use taclib::vm::VirtualMachine;

pub struct ProgramTest {
    file_name: PathBuf,
}

impl ProgramTest {
    pub fn new(file_name: PathBuf) -> Self {
        Self { file_name }
    }

    pub fn run(self) -> bool {
        print!(
            "Testing file {}: ",
            style(self.file_name.to_str().unwrap()).bold()
        );

        let source_code = self.get_source_code();

        let mut output = Vec::new();
        let mut reader = source_code.lines().map(|l| io::Result::Ok(l.to_string()));
        let mut vm = VirtualMachine::new(&mut reader, &mut output);
        vm.interpret(&source_code).unwrap();

        let output = String::from_utf8(output).unwrap();
        let expected_output = Self::get_expected_output(&source_code);

        true
    }

    fn get_source_code(&self) -> String {
        fs::read_to_string(&self.file_name).expect("Something went wrong reading the file")
    }

    fn get_expected_output(src: &str) -> String {
        let lines = src.split('\n');
        let mut output = String::new();

        for line in lines {
            if let Some(pos) = line.find('#') {
                if line.len() > pos + 1 {
                    let line = &line[(pos + 1)..];
                    output += line.trim();
                    output.push('\n');
                }
            }
        }

        output
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
