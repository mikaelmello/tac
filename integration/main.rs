use std::{fs, path::Path};

use program_test::ProgramTest;

mod differ;
mod program_test;
mod project;

fn main() {
    let path = Path::new("./programs");

    for entry in fs::read_dir(path).expect("Unable to list files") {
        let entry = entry.expect("unable to get entry");
        let file_name = entry.path();

        let test = ProgramTest::new(file_name);
        test.run();
    }
}
