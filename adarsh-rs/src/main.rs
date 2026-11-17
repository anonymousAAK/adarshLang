//! `adarsh` command-line entry point.
//!
//!   adarsh FILE.aak     run a program
//!   adarsh [--repl]     interactive REPL (finish a snippet with a blank line)

use adarshlang::{value::py_str, AdarshError, Flow, FsLoader, Session};
use std::io::{BufRead, BufWriter, Write};
use std::process::ExitCode;
use std::rc::Rc;

/// Deep AdarshLang recursion needs a deep native stack.
const STACK_SIZE: usize = 512 * 1024 * 1024;

fn run_file(path: &str) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: cannot read '{path}': {e}");
            return ExitCode::from(2);
        }
    };
    let out = BufWriter::with_capacity(64 * 1024, std::io::stdout());
    match adarshlang::run(&source, Rc::new(FsLoader), Box::new(out)) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = std::io::stdout().flush();
            eprintln!("{}: {}", e.kind_name(), e);
            ExitCode::from(1)
        }
    }
}

fn run_repl() -> ExitCode {
    println!("AdarshLang REPL shuru! Blank line run kare, 'exit' se bahar niklo.");
    let mut session = Session::new(Rc::new(FsLoader), Box::new(std::io::stdout()));
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let mut buffer: Vec<String> = Vec::new();
    loop {
        print!("{}", if buffer.is_empty() { "adarsh> " } else { "... " });
        let _ = std::io::stdout().flush();
        let Some(Ok(line)) = lines.next() else {
            println!();
            break;
        };
        if line.trim() == "exit" {
            break;
        }
        let blank = line.trim().is_empty();
        buffer.push(line);
        if !blank {
            continue;
        }
        let source = buffer.join("\n").trim().to_string();
        buffer.clear();
        if source.is_empty() {
            continue;
        }
        match session.execute(&source) {
            Ok(Flow::Return(v)) => println!("{}", py_str(&v)),
            Ok(Flow::Normal(v)) if !matches!(v, adarshlang::Value::Null) => println!("{}", py_str(&v)),
            Ok(_) => {}
            Err(AdarshError::Semantic(m)) => println!("Soch: {m}"),
            Err(AdarshError::User(v)) => println!("Pakda: {}", py_str(&v)),
            Err(AdarshError::Parse(m)) => println!("Samjho: {m}"),
            Err(e) => println!("Galti: {e}"),
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let child = std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || {
            if args.len() < 2 || args[1] == "--repl" {
                run_repl()
            } else {
                run_file(&args[1])
            }
        })
        .expect("spawn interpreter thread");
    child.join().unwrap_or(ExitCode::FAILURE)
}
