//! Module loading for `lao "file.aak";`. Abstracted so the interpreter can run
//! without a filesystem (e.g. compiled to WebAssembly for the playground).

pub trait ModuleLoader {
    /// Return the source of `path`, or `None` if it does not exist.
    fn load(&self, path: &str) -> Option<String>;
}

/// Loads modules from the filesystem, relative to the current directory
/// (same as the Python implementation).
pub struct FsLoader;

impl ModuleLoader for FsLoader {
    fn load(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// A loader that never finds anything.
pub struct NoLoader;

impl ModuleLoader for NoLoader {
    fn load(&self, _path: &str) -> Option<String> {
        None
    }
}

pub fn module_path(name: &str) -> String {
    if name.ends_with(".aak") {
        name.to_string()
    } else {
        format!("{name}.aak")
    }
}
