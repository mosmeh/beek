use anyhow::{anyhow, Result};
use beek::interpreter::env::Environment;
use beek::interpreter::{exec_stmt, EvalError};
use beek::language::{parse, Number};
use beek::repl::{Repl, Response};
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

        let last_result = if script_given {
            run_script(&opt.script.unwrap(), &mut env)
        } else if files_given {
            opt.file.iter().try_fold(None, |last, file| {
                let script = std::fs::read_to_string(file)?;
                let value = run_script(&script, &mut env)?;
                Ok(value.or(last))
            })
        } else if stdin_given {
            std::io::stdin()
                .lock()
                .lines()
                .try_fold(None, |last, line| {
                    let value = run_script(&line?, &mut env)?;
                    Ok(value.or(last))
                })
        } else {
            unreachable!()
        };

        if let Some(last_result) = last_result? {
            println!("{}", last_result);
        }

        return Ok(());
    }

    run_repl()
}

fn run_script(script: &str, env: &mut Environment) -> Result<Option<Number>> {
    let stmts = parse(script).map_err(|err| anyhow!(err.to_string()))?;

    stmts
        .iter()
        .try_fold(None, |last, stmt| {
            let value = exec_stmt(&stmt, env)?;
            Ok(value.or(last))
        })
        .map_err(|err: EvalError| anyhow!(err))
}

fn run_repl() -> Result<()> {
    let repl = Rc::new(RefCell::new(Repl::new()));

    let mut editor = Editor::new();
    let helper = RLHelper(repl.clone());
    editor.set_helper(Some(helper));

    let history_file = config_dir().map(|dir| dir.join("history"));
    if let Some(history_file) = history_file.as_ref() {
        let _ = editor.load_history(history_file);
    }

    loop {
        match editor.readline("> ") {
            Ok(line) => {
                editor.add_history_entry(line.as_str());

                match repl.borrow_mut().run(&line) {
                    Response::Message(msg) => println!("{}", msg),
                    Response::ClearScreen => println!("\x1Bc"),
                    Response::Quit => break,
                    _ => (),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(anyhow!(err)),
        }
    }

    // history is a non-essential feature, so we just ignore errors
    if let Some(history_file) = history_file {
        if let Some(parent) = history_file.parent() {
            if std::fs::create_dir_all(parent).is_ok() {
                let _ = editor.save_history(&history_file);
            }
        }
    }

    Ok(())
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(env!("CARGO_PKG_NAME")))
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
