//! Runs every program in `../tests/conformance/` and compares against the golden
//! output recorded from the Python reference implementation
//! (regenerate with `python3 tests/gen_expected.py`).

use adarshlang::{FsLoader, SharedBuf};
use std::path::PathBuf;
use std::rc::Rc;

fn run_program(path: &PathBuf) -> String {
    let src = std::fs::read_to_string(path).unwrap();
    let buf = SharedBuf::default();
    let result = adarshlang::run(&src, Rc::new(FsLoader), Box::new(buf.clone()));
    let mut out = String::from_utf8(buf.0.borrow().clone()).unwrap();
    if let Err(e) = result {
        out.push_str(&format!("ERROR: {}: {}\n", e.kind_name(), e));
    }
    out
}

#[test]
fn conformance_suite_matches_python_reference() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/conformance");
    let mut failures = Vec::new();
    let mut count = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.extension().and_then(|e| e.to_str()) != Some("aak") {
            continue;
        }
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let expected_path = dir.join("expected").join(format!("{name}.out"));
        let expected = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("missing golden file {}", expected_path.display()));
        count += 1;
        // run on a big stack: deep AdarshLang recursion recurses natively
        let p = path.clone();
        let actual = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || run_program(&p))
            .unwrap()
            .join()
            .unwrap();
        if actual != expected {
            failures.push(format!("{name}\n--- expected\n{expected}--- actual\n{actual}"));
        }
    }
    assert!(count > 0, "no conformance programs found");
    assert!(failures.is_empty(), "{} of {count} programs differ:\n{}", failures.len(), failures.join("\n"));
}
