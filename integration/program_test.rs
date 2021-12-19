use std::{
    fs::{self},
    path::PathBuf,
    process::Command,
};

use console::style;

use crate::differ::print_diff;

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
        let expected_output = Self::get_expected_output(&source_code);

        // awful hack
        let output = Command::new("carxgo")
            .args(["run", "--quiet", self.file_name.to_str().unwrap()])
            .env("RUSTFLAGS", "-Awarnings")
            .current_dir(
                // ...
                self.file_name
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
            .output()
            .expect("failed to execute process");

        let output = String::from_utf8(output.stderr).unwrap();

        if output == expected_output {
            println!("{}", style("OK").green());
        } else {
            println!("{}", style("FAILED").red());
            print_diff(&expected_output, &output);
        }

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
