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
    /// Look next to the importing file first, then in `aak_modules/` folders
    /// (installed with `adarsh add`) in this folder and every parent folder.
    /// `lao "pkg"` also finds `aak_modules/pkg/main.aak` and `aak_modules/pkg/pkg.aak`.
    fn load(&self, path: &str) -> Option<String> {
        if let Ok(s) = std::fs::read_to_string(self.base.join(path)) {
            return Some(s);
        }
        let stem = path.trim_end_matches(".aak");
        let mut dir = std::fs::canonicalize(&self.base).ok();
        while let Some(d) = dir {
            let m = d.join("aak_modules");
            for candidate in [m.join(path), m.join(stem).join("main.aak"), m.join(stem).join(path)] {
                if let Ok(s) = std::fs::read_to_string(&candidate) {
                    return Some(s);
                }
            }
            dir = d.parent().map(PathBuf::from);
        }
        None
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
