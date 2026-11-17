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
}

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub kind: ErrorKind,
    pub message: String,
    pub pos: Option<Pos>,
    /// The value passed to `chhoddo` (only for `ErrorKind::Thrown`).
    pub thrown: Option<Value>,
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
        AdarshError(Box::new(ErrorInfo { kind, message: message.into(), pos: None, thrown: None }))
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
        AdarshError(Box::new(ErrorInfo { kind: ErrorKind::Thrown, message, pos: None, thrown: Some(value) }))
    }

    /// Attach a position unless a more precise (inner) one is already set.
    pub fn at(mut self, pos: Pos) -> Self {
        if self.pos.is_none() {
            self.pos = Some(pos);
        }
        self
    }

    /// Hinglish label shown before the message.
    pub fn label(&self) -> &'static str {
        match self.kind {
            ErrorKind::Syntax => "Samjho",
            ErrorKind::Semantic => "Soch",
            ErrorKind::Runtime => "Galti",
            ErrorKind::Thrown => "Pakda nahi gaya",
        }
    }

    /// Full multi-line rendering with file, line, column and a caret.
    pub fn render(&self) -> String {
        let mut out = format!("{}: {}", self.label(), self.message);
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
