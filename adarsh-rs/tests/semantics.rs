//! Targeted behaviour tests.

use adarshlang::run_to_string;

fn out(src: &str) -> String {
    match run_to_string(src) {
        Ok(s) => s,
        Err(e) => format!("{}: {}", e.label(), e.message),
    }
}

fn big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(512 << 20).spawn(f).unwrap().join().unwrap()
}

#[test]
fn logical_keywords_and_symbols() {
    assert_eq!(
        out("dikhao(sahi_hai_be aur jhuth); dikhao(jhuth ya sahi_hai_be); dikhao(nahin jhuth);"),
        "jhuth\nsahi_hai_be\nsahi_hai_be\n"
    );
    assert_eq!(out("dikhao(sahi_hai_be && jhuth || sahi_hai_be);"), "sahi_hai_be\n");
}

#[test]
fn logical_operators_short_circuit() {
    let src = "badlo n = 0; kaam bump() { n += 1; wapas sahi_hai_be; } badlo a = jhuth aur bump(); badlo b = sahi_hai_be ya bump(); dikhao(n);";
    assert_eq!(out(src), "0\n");
}

#[test]
fn functions_are_hoisted() {
    assert_eq!(out("dikhao(f(2)); kaam f(x) { wapas g(x) + 1; } kaam g(x) { wapas x * 10; }"), "21\n");
}

#[test]
fn nested_named_functions() {
    assert_eq!(out("kaam a() { kaam b() { wapas 7; } wapas b(); } dikhao(a());"), "7\n");
}

#[test]
fn while_ending_in_continue_keeps_going() {
    assert_eq!(out("badlo i = 0; jabtak (i < 3) { i += 1; aage_badho; } dikhao(\"after\");"), "after\n");
}

#[test]
fn interpolation() {
    assert_eq!(out("badlo n = \"Adarsh\"; dikhao(\"Hi {n}, {1 + 1} {[1, \"a\"]}\");"), "Hi Adarsh, 2 [1, \"a\"]\n");
    assert_eq!(out("dikhao(\"\\{x\\}\");"), "{x}\n");
}

#[test]
fn number_literals() {
    assert_eq!(out("dikhao(.5); dikhao(1e3); dikhao(1_000); dikhao(2.5e-1);"), "0.5\n1000.0\n1000\n0.25\n");
}

#[test]
fn pakdo_catches_runtime_errors() {
    assert_eq!(out("pakdo { dikhao(1 / 0); } chhoddo (e) { dikhao(e); }"), "Zero se divide nahi kar sakte.\n");
}

#[test]
fn records_and_methods() {
    let src = "dhacha P { x }; kaam P.dbl() { wapas yeh.x * 2; } badlo p = P { x: 4 }; dikhao(p.dbl()); dikhao(p);";
    assert_eq!(out(src), "8\nP { x: 4 }\n");
}

#[test]
fn printing_uses_adarshlang_words() {
    assert_eq!(out("dikhao([sahi_hai_be, khali, \"s\"]);"), "[sahi_hai_be, khali, \"s\"]\n");
}

#[test]
fn big_integers_and_floats() {
    assert_eq!(
        out("dikhao(2 ** 100); dikhao(0.1 + 0.2); dikhao(10 / 2);"),
        "1267650600228229401496703205376\n0.30000000000000004\n5.0\n"
    );
}

#[test]
fn runaway_recursion_is_an_error() {
    let r = big_stack(|| out("kaam f(n) { wapas f(n + 1); } f(0);"));
    assert!(r.starts_with("Galti: Recursion bahut gehri"), "{r}");
}

#[test]
fn errors_have_positions() {
    let err = adarshlang::run_to_string("badlo x = 1;\ndikhao(x + \"a\");").unwrap_err();
    let pos = err.pos.unwrap();
    assert_eq!((pos.line, pos.col), (2, 10));
    assert!(err.render().contains("dikhao(x + \"a\");"));
}

#[test]
fn semantic_errors() {
    assert!(out("dikhao(y);").starts_with("Soch: 'y' naam ka variable nahi mila"));
    assert!(out("bas;").starts_with("Soch: 'bas' sirf loop"));
    assert!(out("kaam f(a) { wapas a; } f(1, 2);").starts_with("Soch: 'f' ko 2 arguments diye"));
}
