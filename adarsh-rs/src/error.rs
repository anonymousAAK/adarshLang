//! Error types. Each variant matches an exception class of the Python implementation
//! so messages and REPL prefixes stay identical.

use crate::value::Value;
use std::fmt;

/// Errors the Python implementation surfaced as built-in Python exceptions
/// (`TypeError`, `ZeroDivisionError`, ...). Kept separate so code that catches
/// specific kinds (e.g. compound index assignment) behaves the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostKind {
    TypeError,
    ValueError,
    ZeroDivisionError,
    OverflowError,
    IndexError,
    KeyError,
    RecursionError,
    MemoryError,
}

#[derive(Debug, Clone)]
pub enum AdarshError {
    /// Lexer failure (`RuntimeError` in the Python lexer).
    Lex(String),
    /// `AdarshParserError`
    Parse(String),
    /// `AdarshSemanticError`
    Semantic(String),
    /// `AdarshRuntimeError`
    Runtime(String),
    /// `AdarshUserException` raised by `chhoddo(expr);`
    User(Value),
    /// A host-level error (Python built-in exception in the reference implementation).
    Host(HostKind, String),
}

impl AdarshError {
    pub fn type_error(msg: impl Into<String>) -> Self {
        AdarshError::Host(HostKind::TypeError, msg.into())
    }
    pub fn value_error(msg: impl Into<String>) -> Self {
        AdarshError::Host(HostKind::ValueError, msg.into())
    }
    pub fn zero_div(msg: impl Into<String>) -> Self {
        AdarshError::Host(HostKind::ZeroDivisionError, msg.into())
    }
    pub fn overflow(msg: impl Into<String>) -> Self {
        AdarshError::Host(HostKind::OverflowError, msg.into())
    }
    pub fn runtime(msg: impl Into<String>) -> Self {
        AdarshError::Runtime(msg.into())
    }
    pub fn semantic(msg: impl Into<String>) -> Self {
        AdarshError::Semantic(msg.into())
    }

    /// Python exception class name, useful for diagnostics.
    pub fn kind_name(&self) -> &'static str {
        match self {
            AdarshError::Lex(_) => "RuntimeError",
            AdarshError::Parse(_) => "AdarshParserError",
            AdarshError::Semantic(_) => "AdarshSemanticError",
            AdarshError::Runtime(_) => "AdarshRuntimeError",
            AdarshError::User(_) => "AdarshUserException",
            AdarshError::Host(k, _) => match k {
                HostKind::TypeError => "TypeError",
                HostKind::ValueError => "ValueError",
                HostKind::ZeroDivisionError => "ZeroDivisionError",
                HostKind::OverflowError => "OverflowError",
                HostKind::IndexError => "IndexError",
                HostKind::KeyError => "KeyError",
                HostKind::RecursionError => "RecursionError",
                HostKind::MemoryError => "MemoryError",
            },
        }
    }
}

impl fmt::Display for AdarshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdarshError::Lex(m)
            | AdarshError::Parse(m)
            | AdarshError::Semantic(m)
            | AdarshError::Runtime(m)
            | AdarshError::Host(_, m) => f.write_str(m),
            AdarshError::User(_) => f.write_str("User exception"),
        }
    }
}

impl std::error::Error for AdarshError {}
