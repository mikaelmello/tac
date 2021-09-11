use std::fs;

pub fn run_file(path: &str) {
    let _ = fs::read_to_string(path).expect("Something went wrong reading the file");
}
