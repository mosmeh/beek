use beek::repl::*;
use colored::Colorize;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::{Context, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let repl = Rc::new(RefCell::new(Repl::new()));

    let mut rl = Editor::new();
    let helper = RLHelper(repl.clone());
    rl.set_helper(Some(helper));

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                match repl.borrow_mut().run(&line) {
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

#[derive(Helper, Hinter, Validator, Highlighter)]
struct RLHelper(Rc<RefCell<Repl>>);

impl Completer for RLHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _: &Context<'_>,
    ) -> Result<(usize, Vec<String>), ReadlineError> {
        let (left, _) = line.split_at(pos);
        let start = left
            .trim_end_matches(|c: char| c.is_alphanumeric() || c == '_')
            .len();
        let (_, prefix) = left.split_at(start);
        let candidates = self
            .0
            .borrow()
            .completion_candidates()
            .filter_map(|x| {
                if x.starts_with(prefix) {
                    Some(x.to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok((start, candidates))
    }
}
