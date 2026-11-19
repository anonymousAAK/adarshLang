//! Runs every program in `../tests/conformance/` and compares stdout plus the
//! rendered error against `expected/NAME.out`. A `NAME.in` file, if present,
//! supplies lines for `padho()`.
//!
//! Each program runs twice, with and without the bytecode VM, and both runs must match.
//!
//! After an intended behaviour change, regenerate the golden files with
//! `BLESS=1 cargo test --release` and review the diff.

use adarshlang::FsLoader;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn run_program(path: &Path, vm: bool) -> String {
    adarshlang::vm::set_enabled_on_this_thread(vm);
    let src = std::fs::read_to_string(path).unwrap();
    let input = std::fs::read_to_string(path.with_extension("in"))
        .map(|s| s.lines().map(String::from).collect())
        .unwrap_or_default();
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let base = path.parent().unwrap().to_path_buf();
    let (mut out, err) = adarshlang::run_capture(&name, &src, Rc::new(FsLoader::new(base)), input);
    if let Some(e) = err {
        out.push_str(&e);
        out.push('\n');
    }
    out
}

#[test]
fn conformance_suite() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/conformance");
    let bless = std::env::var_os("BLESS").is_some();
    let mut entries: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).collect();
    entries.sort();
    let mut failures = Vec::new();
    let mut count = 0;
    for path in entries.into_iter().filter(|p| p.extension().is_some_and(|e| e == "aak")) {
        count += 1;
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let expected_path = dir.join("expected").join(format!("{name}.out"));
        // big stack: deep AdarshLang recursion recurses natively
        let run = |vm: bool| {
            let p = path.clone();
            std::thread::Builder::new()
                .stack_size(512 << 20)
                .spawn(move || run_program(&p, vm))
                .unwrap()
                .join()
                .unwrap()
        };
        let actual = run(true);
        // the tree-walker alone must give exactly the same output as with the bytecode VM
        let walker = run(false);
        if walker != actual {
            failures.push(format!("── {name}: VM and tree-walker differ\n--- VM\n{actual}--- tree-walker\n{walker}"));
        }
        if bless {
            std::fs::write(&expected_path, &actual).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("missing {}; run `BLESS=1 cargo test` to create it", expected_path.display()));
        if actual != expected {
            failures.push(format!("── {name}\n--- expected\n{expected}--- actual\n{actual}"));
        }
    }
    assert!(count > 0, "no conformance programs found");
    assert!(failures.is_empty(), "{} of {count} programs differ:\n{}", failures.len(), failures.join("\n"));
}
