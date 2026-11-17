//! WebAssembly exports for the browser playground (no wasm-bindgen needed).
//!
//! JS protocol: `alloc(n)` → write UTF-8 source and input into memory →
//! `run(src_ptr, src_len, input_ptr, input_len)` → read `result_ptr()` /
//! `result_len()`: output, then a `\u{0}` byte, then the rendered error (if any).

use std::cell::RefCell;
use std::rc::Rc;

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
    let (out, err) = crate::run_capture("main.aak", &source, Rc::new(crate::NoLoader), input);
    let mut bytes = out.into_bytes();
    bytes.push(0);
    if let Some(e) = err {
        bytes.extend_from_slice(e.as_bytes());
    }
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
