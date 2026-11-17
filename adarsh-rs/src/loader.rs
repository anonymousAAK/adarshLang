//! Module loading for `lao "file.aak";`, pluggable so programs can run with
//! no filesystem (WebAssembly) or from files bundled into an executable.

use std::collections::HashMap;
use std::path::PathBuf;

pub trait ModuleLoader {
    /// Return the source of `path`, or `None` if it does not exist.
    fn load(&self, path: &str) -> Option<String>;
}

/// Loads modules from disk, relative to `base` (the main program's folder).
pub struct FsLoader {
    pub base: PathBuf,
}

impl FsLoader {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        FsLoader { base: base.into() }
    }
    /// Resolve relative to the current directory.
    pub fn cwd() -> Self {
        FsLoader { base: PathBuf::from(".") }
    }
}

impl ModuleLoader for FsLoader {
    fn load(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(self.base.join(path)).ok()
    }
}

/// A loader that never finds anything.
pub struct NoLoader;

impl ModuleLoader for NoLoader {
    fn load(&self, _path: &str) -> Option<String> {
        None
    }
}

/// Modules held in memory (bundled executables, the web playground).
#[derive(Default)]
pub struct MemLoader {
    pub files: HashMap<String, String>,
}

impl ModuleLoader for MemLoader {
    fn load(&self, path: &str) -> Option<String> {
        self.files.get(path).cloned()
    }
}

/// `lao "utils"` and `lao "utils.aak"` both load `utils.aak`.
pub fn module_path(name: &str) -> String {
    if name.ends_with(".aak") {
        name.to_string()
    } else {
        format!("{name}.aak")
    }
}
