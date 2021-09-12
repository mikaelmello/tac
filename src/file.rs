use std::fs;

use crate::vm::VirtualMachine;

pub fn run_file(path: &str) {
    let source = fs::read_to_string(path).expect("Something went wrong reading the file");
    let mut vm = VirtualMachine::new();

    match vm.interpret(&source) {
        Ok(_) => {}
        Err(_) => eprintln!(
            "There were errors in the program execution, please check the console log above"
        ),
    }
}
