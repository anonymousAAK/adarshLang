//! Measures native stack used per AdarshLang call (for tuning recursion limits).
//! Each trial runs in a child process, because a stack overflow aborts the process.
const STACK: usize = 8 << 20;

fn trial(depth: usize) {
    let src = format!("kaam d(n) {{ agar (n == 0) {{ wapas 0; }} wapas 1 + d(n - 1); }} d({depth});");
    let ok = std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || adarshlang::run_to_string(&src).is_ok())
        .unwrap()
        .join()
        .unwrap_or(false);
    std::process::exit(if ok { 0 } else { 1 });
}

fn main() {
    if let Some(d) = std::env::args().nth(1) {
        trial(d.parse().unwrap());
    }
    let me = std::env::current_exe().unwrap();
    let (mut lo, mut hi) = (10usize, 9_000usize);
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        let ok = std::process::Command::new(&me)
            .arg(mid.to_string())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success();
        if ok {
            lo = mid
        } else {
            hi = mid
        }
    }
    println!("8 MB stack: max depth {lo}, ~{} bytes per call", STACK / lo);
}
