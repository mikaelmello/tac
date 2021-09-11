use clap::Clap;
use opts::Opts;

mod chunk;
mod disassembler;
mod error;
mod file;
mod opts;
mod repl;
mod value;
mod vm;

fn main() {
    let opts: Opts = Opts::parse();

    match opts.script {
        Some(path) => file::run_file(&path),
        None => repl::repl().unwrap(),
    }
}
