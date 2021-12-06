use std::{
    fs,
    path::{Path, PathBuf},
};

use program_test::ProgramTest;
use similar::{ChangeTag, TextDiff};

mod differ;
mod program_test;

fn main() {
    let diff = differ::diff(
        "Hello World\nThis is the second line.\nThis is the third.",
        "Hallo Welt\nThis is the second line.\nThis is life.\nMoar and more",
    );

    let path = Path::new("./programs");

    for entry in fs::read_dir(path).expect("Unable to list files") {
        let entry = entry.expect("unable to get entry");
        let file_name = entry.path();

        let test = ProgramTest::new(file_name);
        test.run();
        //test_program(entry.path());
    }
}
