use anyhow::Result;
use beek::interpreter::env::Environment;
use beek::interpreter::exec_stmt;
use beek::language::parse;
use beek::repl::{Repl, Response};
use colored::Colorize;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::{Context, Editor};
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};
use std::cell::RefCell;
use std::io::BufRead;
use std::path::PathBuf;
use std::rc::Rc;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    /// Executes script passed in as string
    script: Option<String>,

    /// File(s) containing scripts
    #[structopt(short, long, conflicts_with = "script")]
    file: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let script_given = opt.script.is_some();
    let files_given = !opt.file.is_empty();
    let stdin_given = atty::isnt(atty::Stream::Stdin);

    if script_given || files_given || stdin_given {
        colored::control::set_override(false);

        let mut env = Environment::new();

        if script_given {
            run_script(&opt.script.unwrap(), &mut env)?;
        } else if files_given {
            for file in opt.file {
                let script = std::fs::read_to_string(file)?;
                run_script(&script, &mut env)?;
            }
        } else if stdin_given {
            let stdin = std::io::stdin();
            let stdin = stdin.lock();
            for line in stdin.lines() {
                run_script(&line?, &mut env)?;
            }
        }

        return Ok(());
    }

    repl();
    Ok(())
}

fn run_script(script: &str, env: &mut Environment) -> Result<()> {
    let stmts = parse(script).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    for stmt in stmts {
        if let Some(value) = exec_stmt(&stmt, env)? {
            println!("{}", value);
        }
    }

    Ok(())
}

fn repl() {
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
