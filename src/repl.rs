use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io;

use crate::vm::VirtualMachine;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn repl() -> io::Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();

    let mut vm = VirtualMachine::new();
    println!("TAC {}", VERSION);

    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str());

                match vm.interpret(&line) {
                    Ok(()) => {}
                    Err(err) => println!("Error: {:?}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
