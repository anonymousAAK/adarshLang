//! WebAssembly exports for the browser playground (no wasm-bindgen needed).
//!
//! JS protocol: `alloc(n)` → write UTF-8 source and input into memory →
//! `run(src_ptr, src_len, input_ptr, input_len)` → read `result_ptr()` /
//! `result_len()`: output, then a `\u{0}` byte, then the rendered error (if any),
//! then `\u{0}` and the turtle drawing as JSON.
//!
//! The page supplies these imports in the `env` module:
//! `host_now() -> f64` (ms since 1970), `host_tz() -> i32` (minutes east of UTC),
//! `host_sleep(ms: f64)`, and `host_write(ptr, len)` for streaming output.

use std::cell::RefCell;
use std::rc::Rc;

#[link(wasm_import_module = "env")]
extern "C" {
    fn host_now() -> f64;
    fn host_tz() -> i32;
    fn host_sleep(ms: f64);
    fn host_write(ptr: *const u8, len: usize);
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

/// # Safety
/// The pointers must reference `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn run(src: *const u8, src_len: usize, input: *const u8, input_len: usize) {
    let source = text(src, src_len);
    let input: Vec<String> = text(input, input_len).lines().map(String::from).collect();
    let result = crate::run_full(crate::RunOptions {
        name: "main.aak".into(),
        source,
        loader: Rc::new(crate::NoLoader),
        input,
        stream: Some(Box::new(StreamOut)),
        ..Default::default()
    });
    let mut bytes = result.output.into_bytes();
    bytes.push(0);
    if let Some(e) = result.error {
        bytes.extend_from_slice(e.as_bytes());
    }
    bytes.push(0);
    bytes.extend_from_slice(result.turtle_json.as_bytes());
    RESULT.with(|r| *r.borrow_mut() = bytes);
}

#[no_mangle]
pub extern "C" fn result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn result_len() -> usize {
    RESULT.with(|r| r.borrow().len())
}
