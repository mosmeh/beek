use beek::repl::*;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let mut repl = Repl::new();

    let mut rl = Editor::<()>::new();
    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                match repl.run(&line) {
                    Response::Message(msg) => println!("{}", msg),
                    Response::ClearScreen => println!("\x1Bc"),
                    Response::Quit => break,
                    _ => (),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("{}", format!("Error: {:?}", err).red());
                break;
            }
        }
    }
}
