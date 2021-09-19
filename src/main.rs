use std::sync::RwLock;

use clap::Clap;
use lazy_static::lazy_static;
use opts::Opts;

mod chunk;
mod compiler;
mod disassembler;
mod error;
mod file;
mod opts;
mod repl;
mod scanner;
mod token;
mod value;
mod vm;

lazy_static! {
    static ref TRACE_EXECUTION: RwLock<bool> = RwLock::new(false);
}

fn main() {
    let opts: Opts = Opts::parse();

    {
        let mut guard = TRACE_EXECUTION.write().unwrap();
        *guard = opts.trace_execution;
    }

    match opts.script {
        Some(path) => file::run_file(&path),
        None => repl::repl().unwrap(),
    }
}
