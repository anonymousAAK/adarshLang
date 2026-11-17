//! AdarshLang — a Hinglish-flavoured programming language, implemented in Rust.
//!
//! Pipeline: [`lexer`] → [`parser`] → [`semantics`] → [`interp`].
//! This is a port of the original Python implementation in `adarsh_lang/`
//! and aims to produce identical output for the same programs.

pub mod ast;
pub mod builtins;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod value;

pub use error::AdarshError;
pub use interp::{Flow, Interpreter};
pub use loader::{FsLoader, ModuleLoader, NoLoader};
pub use value::Value;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// Lex, parse, analyse and run `source`, writing `dikhao` output to `out`.
/// Equivalent to Python's `adarshlang_compile_and_run`.
pub fn run(source: &str, loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Result<Flow, AdarshError> {
    let program = parser::parse(source)?;
    let mut sema = semantics::Analyzer::new(loader.clone());
    sema.analyze_program(&program, &semantics::root_scope())?;
    let mut interp = Interpreter::new(loader, out);
    interp.run(&program)
}

/// A `Write` sink that collects output into a shared buffer.
#[derive(Clone, Default)]
pub struct SharedBuf(pub Rc<RefCell<Vec<u8>>>);

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Run `source` and return everything it printed (or the error).
/// Imports are disabled. Handy for tests and embedding (e.g. the web playground).
pub fn run_to_string(source: &str) -> Result<String, AdarshError> {
    let buf = SharedBuf::default();
    run(source, Rc::new(NoLoader), Box::new(buf.clone()))?;
    let bytes = buf.0.borrow().clone();
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// A persistent session for the REPL: definitions survive between snippets.
pub struct Session {
    pub interp: Interpreter,
    sema: semantics::Analyzer,
    sema_scope: semantics::SemScopeRef,
}

impl Session {
    pub fn new(loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Self {
        Session {
            interp: Interpreter::new(loader.clone(), out),
            sema: semantics::Analyzer::new(loader),
            sema_scope: semantics::root_scope(),
        }
    }

    /// Python's `execute_with_state`.
    pub fn execute(&mut self, source: &str) -> Result<Flow, AdarshError> {
        let program = parser::parse(source)?;
        self.sema.analyze_program(&program, &self.sema_scope)?;
        self.interp.run(&program)
    }
}
