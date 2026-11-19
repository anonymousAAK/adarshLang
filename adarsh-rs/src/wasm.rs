//! WebAssembly exports for the browser playground (no wasm-bindgen needed).
//!
//! JS protocol:
//! - `clear_files()` / `set_file(name, len, src, len)`: extra files that `lao` can load.
//! - `run(src, len, input, len, mode)`: mode bit 1 = record steps, bit 2 = ask the page
//!   for input (`host_read_line`) when the provided lines run out.
//! - `to_python(src, len)`: translate to Python.
//!
//! Results are read from `result_ptr()` / `result_len()` as `\0`-separated parts.
//! For `run`: output, rendered error, turtle JSON, steps JSON, warnings JSON.
//! For `to_python`: python source, rendered error.
//!
//! The page supplies these imports in the `env` module:
//! `host_now() -> f64` (ms since 1970), `host_tz() -> i32` (minutes east of UTC),
//! `host_sleep(ms: f64)`, `host_write(ptr, len)` for streaming output, and
//! `host_read_line(ptr, cap) -> i32` (bytes written, or -1 for end of input).

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::rc::Rc;

#[link(wasm_import_module = "env")]
extern "C" {
    fn host_now() -> f64;
    fn host_tz() -> i32;
    fn host_sleep(ms: f64);
    fn host_write(ptr: *const u8, len: usize);
    fn host_read_line(ptr: *mut u8, cap: usize) -> i32;
}

pub fn now() -> f64 {
    unsafe { host_now() / 1000.0 }
}

pub fn tz_offset_minutes() -> i32 {
    unsafe { host_tz() }
}

pub fn sleep(secs: f64) {
    unsafe { host_sleep(secs * 1000.0) }
}

fn read_line_from_page() -> Option<String> {
    let mut buf = vec![0u8; 8192];
    let n = unsafe { host_read_line(buf.as_mut_ptr(), buf.len()) };
    if n < 0 {
        return None;
    }
    buf.truncate(n as usize);
    Some(String::from_utf8_lossy(&buf).into_owned())
}

/// Output sink that streams each write to the page as it happens.
pub struct StreamOut;

impl std::io::Write for StreamOut {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe { host_write(buf.as_ptr(), buf.len()) };
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

thread_local! {
    static RESULT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static FILES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// # Safety
/// `ptr` must come from `alloc(len)`.
#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len));
}

unsafe fn text(ptr: *const u8, len: usize) -> String {
    String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len)).into_owned()
}

fn json_str(s: &str) -> String {
    let mut out = String::new();
    let _ = crate::stdlib::to_json(&crate::Value::Str(s.into()), false, 0, &mut out);
    out
}

#[no_mangle]
pub extern "C" fn clear_files() {
    FILES.with(|f| f.borrow_mut().clear());
}

/// # Safety
/// The pointers must reference `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn set_file(name: *const u8, name_len: usize, src: *const u8, src_len: usize) {
    let (n, s) = (text(name, name_len), text(src, src_len));
    FILES.with(|f| f.borrow_mut().insert(n, s));
}

/// # Safety
/// The pointers must reference `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn run(src: *const u8, src_len: usize, input: *const u8, input_len: usize, mode: u32) {
    let source = text(src, src_len);
    let mut lines: Vec<String> = text(input, input_len).lines().map(String::from).collect();
    let live_input = mode & 2 != 0;
    let files = FILES.with(|f| f.borrow().clone());
    let mut opts = crate::RunOptions {
        name: "main.aak".into(),
        source,
        loader: Rc::new(crate::MemLoader { files }),
        stream: Some(Box::new(StreamOut)),
        record_steps: if mode & 1 != 0 { 5000 } else { 0 },
        ..Default::default()
    };
    if live_input {
        // provided lines first, then ask the page
        lines.reverse();
        opts.input_fn = Some(Box::new(move || lines.pop().or_else(read_line_from_page)));
    } else {
        opts.input = lines;
    }
    let result = crate::run_full(opts);
    let mut out = result.output;
    out.push('\0');
    out.push_str(result.error.as_deref().unwrap_or(""));
    out.push('\0');
    out.push_str(&result.turtle_json);
    out.push('\0');
    out.push('[');
    for (i, s) in result.steps.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "{{\"l\":{},\"d\":{},\"o\":{},\"v\":[", s.line, s.depth, s.output_len);
        for (j, (k, v)) in s.vars.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            let _ = write!(out, "[{},{}]", json_str(k), json_str(v));
        }
        out.push_str("]}");
    }
    out.push(']');
    out.push('\0');
    out.push('[');
    for (i, w) in result.warnings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json_str(w));
    }
    out.push(']');
    RESULT.with(|r| *r.borrow_mut() = out.into_bytes());
}

/// # Safety
/// The pointer must reference `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn to_python(src: *const u8, src_len: usize) {
    let source = text(src, src_len);
    let files = FILES.with(|f| f.borrow().clone());
    let out = match crate::check("main.aak", &source, Rc::new(crate::MemLoader { files })) {
        Ok(program) => format!("{}\0", crate::topython::translate(&program)),
        Err(e) => format!("\0{}", e.render()),
    };
    RESULT.with(|r| *r.borrow_mut() = out.into_bytes());
}

#[no_mangle]
pub extern "C" fn result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn result_len() -> usize {
    RESULT.with(|r| r.borrow().len())
}
