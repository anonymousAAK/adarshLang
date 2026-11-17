//! AdarshLang: a Hinglish-flavoured programming language.
//!
//! Pipeline: [`lexer`] → [`parser`] → [`semantics`] → [`interp`].
//!
//! ```
//! let out = adarshlang::run_to_string("dikhao(\"Namaste {2 + 3}\");").unwrap();
//! assert_eq!(out, "Namaste 5\n");
//! ```

pub mod ast;
pub mod builtins;
pub mod error;
pub mod interp;
pub mod lexer;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod value;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use error::{AdarshError, ErrorKind};
pub use interp::{Flow, Interpreter};
pub use loader::{FsLoader, MemLoader, ModuleLoader, NoLoader};
pub use value::Value;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse and statically check `source` without running it.
pub fn check(name: &str, source: &str, loader: Rc<dyn ModuleLoader>) -> Result<Vec<ast::Node>, AdarshError> {
    let file = error::register_source(name, source);
    let program = parser::parse(source, file)?;
    semantics::Analyzer::new(loader).analyze_program(&program, &semantics::root_scope())?;
    Ok(program)
}

/// Check and run a program. `dikhao` output goes to `out`.
pub fn run(name: &str, source: &str, loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Result<(), AdarshError> {
    let program = check(name, source, loader.clone())?;
    let mut interp = Interpreter::new(loader, out);
    interp.run(&program).map(|_| ())
}

/// A `Write` sink that collects output into a shared buffer.
#[derive(Clone, Default)]
pub struct SharedBuf(pub Rc<RefCell<Vec<u8>>>);

impl SharedBuf {
    pub fn take_string(&self) -> String {
        String::from_utf8_lossy(&std::mem::take(&mut *self.0.borrow_mut())).into_owned()
    }
}

impl Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Run `source` (no imports) and return what it printed, or the error.
pub fn run_to_string(source: &str) -> Result<String, AdarshError> {
    let buf = SharedBuf::default();
    run("main.aak", source, Rc::new(NoLoader), Box::new(buf.clone()))?;
    Ok(buf.take_string())
}

/// Output plus the rendered error (if any), as the playground and tests show it.
pub fn run_capture(
    name: &str,
    source: &str,
    loader: Rc<dyn ModuleLoader>,
    input: Vec<String>,
) -> (String, Option<String>) {
    let buf = SharedBuf::default();
    let result = check(name, source, loader.clone()).and_then(|program| {
        let mut interp = Interpreter::new(loader, Box::new(buf.clone()));
        let mut lines = input.into_iter();
        interp.input = Box::new(move || lines.next());
        interp.run(&program).map(|_| ())
    });
    (buf.take_string(), result.err().map(|e| e.render()))
}

/// A persistent session for the REPL: definitions survive between snippets.
pub struct Session {
    pub interp: Interpreter,
    sema: semantics::Analyzer,
    sema_scope: semantics::SemScopeRef,
    counter: usize,
}

impl Session {
    pub fn new(loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Self {
        Session {
            interp: Interpreter::new(loader.clone(), out),
            sema: semantics::Analyzer::new(loader),
            sema_scope: semantics::root_scope(),
            counter: 0,
        }
    }

    /// Run one snippet; returns the value of the last statement.
    pub fn execute(&mut self, source: &str) -> Result<Flow, AdarshError> {
        self.counter += 1;
        let file = error::register_source(&format!("<repl {}>", self.counter), source);
        let program = parser::parse(source, file)?;
        self.sema.analyze_program(&program, &self.sema_scope)?;
        self.interp.run(&program)
    }
}
