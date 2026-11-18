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
pub mod explain;
pub mod interp;
pub mod lexer;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod stdlib;
pub mod topython;
pub mod value;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use error::{AdarshError, ErrorKind};
pub use interp::{Flow, Interpreter};
pub use loader::{FsLoader, MemLoader, ModuleLoader, NoLoader};
pub use value::Value;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parse and statically check `source` without running it. Returns the program and warnings.
pub fn check_with_warnings(
    name: &str,
    source: &str,
    loader: Rc<dyn ModuleLoader>,
) -> Result<(Vec<ast::Node>, Vec<AdarshError>), AdarshError> {
    let file = error::register_source(name, source);
    let program = parser::parse(source, file)?;
    let mut sema = semantics::Analyzer::new(loader);
    sema.analyze_program(&program, &semantics::root_scope())?;
    Ok((program, sema.warnings))
}

/// Parse and statically check `source` without running it.
pub fn check(name: &str, source: &str, loader: Rc<dyn ModuleLoader>) -> Result<Vec<ast::Node>, AdarshError> {
    check_with_warnings(name, source, loader).map(|(p, _)| p)
}

/// Check and run a program. `dikhao` output goes to `out`.
pub fn run(name: &str, source: &str, loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Result<(), AdarshError> {
    let program = check(name, source, loader.clone())?;
    let mut interp = Interpreter::new(loader, out);
    match interp.run(&program) {
        Err(e) if matches!(e.kind, ErrorKind::Exit(0)) => Ok(()),
        other => other.map(|_| ()),
    }
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

/// Writes to two sinks (capture + live stream).
struct Tee(SharedBuf, Box<dyn Write>);

impl Write for Tee {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write_all(buf)?;
        self.1.write_all(buf)?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.1.flush()
    }
}

/// One recorded step of a program run (for step-through / time travel).
#[derive(Clone, Debug)]
pub struct Step {
    pub line: u32,
    pub depth: usize,
    /// visible variables at that moment (name, value as text), innermost scope first
    pub vars: Vec<(String, String)>,
    /// length of the output so far
    pub output_len: usize,
}

pub struct RunOptions {
    pub name: String,
    pub source: String,
    pub loader: Rc<dyn ModuleLoader>,
    /// lines for `padho()`
    pub input: Vec<String>,
    /// where `padho()` reads from instead of `input` (e.g. asking the web page)
    pub input_fn: Option<Box<dyn FnMut() -> Option<String>>>,
    /// also send output here as it is produced
    pub stream: Option<Box<dyn Write>>,
    /// record up to this many steps (0 = don't record)
    pub record_steps: usize,
    pub seed: Option<u64>,
}

impl Default for RunOptions {
    fn default() -> Self {
        RunOptions {
            name: "main.aak".into(),
            source: String::new(),
            loader: Rc::new(NoLoader),
            input: Vec::new(),
            input_fn: None,
            stream: None,
            record_steps: 0,
            seed: None,
        }
    }
}

#[derive(Default)]
pub struct RunResult {
    pub output: String,
    /// rendered error, if the program failed
    pub error: Option<String>,
    pub error_value: Option<AdarshError>,
    pub warnings: Vec<String>,
    pub turtle_json: String,
    pub turtle_svg: Option<String>,
    pub steps: Vec<Step>,
    pub exit_code: i32,
}

/// Check and run with every option; used by the playground, tests and tools.
pub fn run_full(opts: RunOptions) -> RunResult {
    let buf = SharedBuf::default();
    let mut result = RunResult::default();
    let (program, warnings) = match check_with_warnings(&opts.name, &opts.source, opts.loader.clone()) {
        Ok(p) => p,
        Err(e) => {
            result.error = Some(e.render());
            result.error_value = Some(e);
            result.exit_code = 1;
            return result;
        }
    };
    result.warnings = warnings.iter().map(|w| w.render()).collect();
    let out: Box<dyn Write> = match opts.stream {
        Some(s) => Box::new(Tee(buf.clone(), s)),
        None => Box::new(buf.clone()),
    };
    let mut interp = Interpreter::new(opts.loader, out);
    if let Some(seed) = opts.seed {
        interp.set_seed(seed);
    }
    match opts.input_fn {
        Some(f) => interp.input = f,
        None => {
            let mut lines = opts.input.into_iter();
            interp.input = Box::new(move || lines.next());
        }
    }
    let steps: Rc<RefCell<Vec<Step>>> = Rc::default();
    if opts.record_steps > 0 {
        let steps = steps.clone();
        let limit = opts.record_steps;
        let out_buf = buf.clone();
        interp.on_step = Some(Box::new(move |pos, scope, depth| {
            let mut st = steps.borrow_mut();
            if st.len() >= limit {
                return false;
            }
            st.push(Step {
                line: pos.line,
                depth,
                vars: interp::visible_vars(scope),
                output_len: out_buf.0.borrow().len(),
            });
            true
        }));
    }
    let r = interp.run(&program);
    interp.flush();
    if let Err(e) = r {
        match e.kind {
            ErrorKind::Exit(code) => result.exit_code = code,
            _ => {
                result.error = Some(e.render());
                result.error_value = Some(e);
                result.exit_code = 1;
            }
        }
    }
    result.output = buf.take_string();
    result.turtle_json = interp.turtle.json();
    if !interp.turtle.lines.is_empty() {
        result.turtle_svg = Some(interp.turtle.svg());
    }
    result.steps = steps.take();
    result
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
    let r = run_full(RunOptions { name: name.into(), source: source.into(), loader, input, ..Default::default() });
    (r.output, r.error)
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
        self.sema.warnings.clear();
        self.sema.analyze_program(&program, &self.sema_scope)?;
        self.interp.run(&program)
    }
}

/// Result of one test from `adarsh test`.
pub struct TestOutcome {
    pub name: String,
    pub error: Option<String>,
    pub output: String,
}

/// Run a test file: execute it, then call every global function named `test_*`
/// (in source order). A file without such functions is one test that must run cleanly.
pub fn run_tests(name: &str, source: &str, loader: Rc<dyn ModuleLoader>) -> Vec<TestOutcome> {
    let program = match check(name, source, loader.clone()) {
        Ok(p) => p,
        Err(e) => return vec![TestOutcome { name: name.into(), error: Some(e.render()), output: String::new() }],
    };
    let buf = SharedBuf::default();
    let mut interp = Interpreter::new(loader, Box::new(buf.clone()));
    if let Err(e) = interp.run(&program) {
        if !matches!(e.kind, ErrorKind::Exit(0)) {
            return vec![TestOutcome { name: name.into(), error: Some(e.render()), output: buf.take_string() }];
        }
    }
    let mut tests: Vec<(u32, u32, String)> = program
        .iter()
        .filter_map(|n| match &n.kind {
            ast::NodeKind::KaamDef(d) if d.method_of.is_none() => {
                let fname = d.name.as_deref()?;
                (fname.starts_with("test_") && d.params.iter().all(|p| p.default.is_some() || p.is_varargs))
                    .then(|| (d.pos.line, d.pos.col, fname.to_string()))
            }
            _ => None,
        })
        .collect();
    tests.sort();
    if tests.is_empty() {
        return vec![TestOutcome { name: name.into(), error: None, output: buf.take_string() }];
    }
    let setup_output = buf.take_string();
    let mut out = Vec::new();
    for (_, _, t) in tests {
        let f = interp.global.borrow().vars_get(&t);
        let result = match f {
            Some(f) => interp.call(&f, vec![]).map(|_| ()),
            None => Ok(()),
        };
        interp.flush();
        let output = buf.take_string();
        out.push(TestOutcome {
            name: t,
            error: result.err().filter(|e| !matches!(e.kind, ErrorKind::Exit(0))).map(|e| e.render()),
            output,
        });
    }
    if let Some(first) = out.first_mut() {
        first.output = setup_output + &first.output;
    }
    out
}
