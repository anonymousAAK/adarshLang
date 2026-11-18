//! Errors, source positions and friendly rendering.
//!
//! Every error carries a kind, a Hinglish message and (when known) the
//! location in the source, which `render()` turns into:
//!
//! ```text
//! Galti: Zero se divide nahi kar sakte.
//!   --> main.aak:4:14
//!    |
//!  4 | badlo x = 10 / 0;
//!    |              ^
//! ```

use crate::value::Value;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

/// Position of a token/node: 1-based line and column in source file `file`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
    pub file: u32,
}

pub struct SourceFile {
    pub name: Rc<str>,
    pub text: Rc<str>,
}

thread_local! {
    static SOURCES: RefCell<Vec<Rc<SourceFile>>> = const { RefCell::new(Vec::new()) };
}

/// Register a source text so error positions can show the offending line.
/// Returns the file id stored in every [`Pos`] produced from it.
pub fn register_source(name: &str, text: &str) -> u32 {
    SOURCES.with(|s| {
        let mut s = s.borrow_mut();
        s.push(Rc::new(SourceFile { name: name.into(), text: text.into() }));
        (s.len() - 1) as u32
    })
}

thread_local! {
    static ENGLISH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Also show a short English title under Hinglish errors (`adarsh --english`).
pub fn set_english(on: bool) {
    ENGLISH.with(|e| e.set(on));
}

fn source(file: u32) -> Option<Rc<SourceFile>> {
    SOURCES.with(|s| s.borrow().get(file as usize).cloned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Lexer / parser: the program text is not valid AdarshLang.
    Syntax,
    /// Static checks: undeclared names, wrong argument counts, `bas` outside a loop.
    Semantic,
    /// Errors while running (bad types, missing keys, division by zero, ...).
    Runtime,
    /// A value thrown with `chhoddo(...)` that nobody caught.
    Thrown,
    /// Not an error: something worth a look (unused variable, unreachable code).
    Warning,
    /// `band_karo(code)`: stop the program. Not catchable by `pakdo`.
    Exit(i32),
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub kind: ErrorKind,
    pub message: String,
    pub pos: Option<Pos>,
    /// The value passed to `chhoddo` (only for `ErrorKind::Thrown`).
    pub thrown: Option<Value>,
    /// Stable code such as `E001`, explained by `adarsh samjhao E001`.
    pub code: Option<&'static str>,
    /// Function calls the error passed through, innermost first.
    pub trace: Vec<(String, Pos)>,
}

/// Boxed so `Result<Value, AdarshError>` stays small: every interpreter frame
/// carries one, and smaller frames allow deeper recursion.
#[derive(Debug, Clone)]
pub struct AdarshError(Box<ErrorInfo>);

impl std::ops::Deref for AdarshError {
    type Target = ErrorInfo;
    fn deref(&self) -> &ErrorInfo {
        &self.0
    }
}

impl std::ops::DerefMut for AdarshError {
    fn deref_mut(&mut self) -> &mut ErrorInfo {
        &mut self.0
    }
}

impl AdarshError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        AdarshError(Box::new(ErrorInfo {
            kind,
            message: message.into(),
            pos: None,
            thrown: None,
            code: None,
            trace: Vec::new(),
        }))
    }
    pub fn syntax(message: impl Into<String>, pos: Pos) -> Self {
        AdarshError::new(ErrorKind::Syntax, message).at(pos)
    }
    pub fn semantic(message: impl Into<String>) -> Self {
        AdarshError::new(ErrorKind::Semantic, message)
    }
    pub fn runtime(message: impl Into<String>) -> Self {
        AdarshError::new(ErrorKind::Runtime, message)
    }
    pub fn thrown(value: Value) -> Self {
        let message = crate::value::display(&value);
        AdarshError(Box::new(ErrorInfo {
            kind: ErrorKind::Thrown,
            message,
            pos: None,
            thrown: Some(value),
            code: None,
            trace: Vec::new(),
        }))
    }

    /// Attach a position unless a more precise (inner) one is already set.
    pub fn at(mut self, pos: Pos) -> Self {
        if self.pos.is_none() {
            self.pos = Some(pos);
        }
        self
    }

    pub fn with_code(mut self, code: &'static str) -> Self {
        if self.code.is_none() {
            self.code = Some(code);
        }
        self
    }

    /// Can `pakdo` catch this?
    pub fn catchable(&self) -> bool {
        matches!(self.kind, ErrorKind::Runtime | ErrorKind::Thrown)
    }

    /// Hinglish label shown before the message.
    pub fn label(&self) -> &'static str {
        match self.kind {
            ErrorKind::Syntax => "Samjho",
            ErrorKind::Semantic => "Soch",
            ErrorKind::Runtime => "Galti",
            ErrorKind::Thrown => "Pakda nahi gaya",
            ErrorKind::Warning => "Dhyan do",
            ErrorKind::Exit(_) => "Band",
        }
    }

    /// Full multi-line rendering with file, line, column and a caret.
    pub fn render(&self) -> String {
        let code = self.code.map(|c| format!(" [{c}]")).unwrap_or_default();
        let mut out = format!("{}{code}: {}", self.label(), self.message);
        if ENGLISH.with(|e| e.get()) {
            let title = self.code.and_then(crate::explain::find).map(|x| x.title_en).unwrap_or(match self.kind {
                ErrorKind::Syntax => "Syntax error",
                ErrorKind::Semantic => "Error found before running",
                ErrorKind::Runtime => "Runtime error",
                ErrorKind::Thrown => "Uncaught error",
                ErrorKind::Warning => "Warning",
                ErrorKind::Exit(_) => "Exit",
            });
            out.push_str(&format!("\n  (English: {title})"));
        }
        let Some(pos) = self.pos else { return out };
        let Some(src) = source(pos.file) else { return out };
        out.push_str(&format!("\n  --> {}:{}:{}", src.name, pos.line, pos.col));
        if let Some(line_text) = src.text.lines().nth(pos.line.saturating_sub(1) as usize) {
            let num = pos.line.to_string();
            let pad = " ".repeat(num.len());
            let caret_pad: String = line_text
                .chars()
                .take(pos.col.saturating_sub(1) as usize)
                .map(|c| if c == '\t' { '\t' } else { ' ' })
                .collect();
            out.push_str(&format!("\n {pad} |\n {num} | {line_text}\n {pad} | {caret_pad}^"));
        }
        if !self.trace.is_empty() {
            out.push_str("\n  Kaise pahunche (call chain):");
            for (name, p) in self.trace.iter().take(8) {
                let file = source(p.file).map(|s| s.name.to_string()).unwrap_or_default();
                out.push_str(&format!("\n    '{name}' ko call kiya: {file}:{}:{}", p.line, p.col));
            }
            if self.trace.len() > 8 {
                out.push_str(&format!("\n    ... aur {} calls", self.trace.len() - 8));
            }
        }
        if let Some(c) = self.code {
            if self.kind != ErrorKind::Warning {
                out.push_str(&format!("\n  Aur jaankari: adarsh samjhao {c}"));
            }
        }
        out
    }

    /// The value a `pakdo ... chhoddo (err)` handler receives.
    pub fn catch_value(&self) -> Value {
        match &self.thrown {
            Some(v) => v.clone(),
            None => Value::str(&self.message),
        }
    }
}

impl fmt::Display for AdarshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AdarshError {}
