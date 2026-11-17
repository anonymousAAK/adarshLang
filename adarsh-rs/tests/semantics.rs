//! Targeted tests, including behaviour where the Rust port intentionally
//! fixes bugs in the Python reference implementation.

use adarshlang::run_to_string;

fn out(src: &str) -> String {
    run_to_string(src).unwrap_or_else(|e| format!("ERROR: {}: {e}", e.kind_name()))
}

#[test]
fn logical_operators_work() {
    // The Python lexer rejected '&&' and '||' even though they are documented.
    assert_eq!(out("dikhao(sahi_hai_be && jhuth); dikhao(jhuth || sahi_hai_be);"), "jhuth\nsahi_hai_be\n");
}

#[test]
fn logical_operators_evaluate_both_sides_like_python() {
    let src = "badlo n = 0; kaam bump() { n += 1; wapas sahi_hai_be; } badlo r = jhuth && bump(); dikhao(n);";
    assert_eq!(out(src), "1\n");
}

#[test]
fn while_ending_in_continue_does_not_stop_program() {
    // Python leaked a ContinueSignal out of the loop and silently stopped the program.
    let src = "badlo i = 0; jabtak (i < 3) { i += 1; aage_badho; } dikhao(\"after\");";
    assert_eq!(out(src), "after\n");
}

#[test]
fn big_integers_are_exact() {
    assert_eq!(out("dikhao(2 ** 100); dikhao(9223372036854775807 + 1);"), "1267650600228229401496703205376\n9223372036854775808\n");
}

#[test]
fn float_formatting_matches_python_repr() {
    assert_eq!(out("dikhao(0.1 + 0.2); dikhao(10.0 ** 16); dikhao(1 / 100000); dikhao(10 / 2);"),
               "0.30000000000000004\n1e+16\n1e-05\n5.0\n");
}

#[test]
fn recursion_limit_is_an_error_not_a_crash() {
    let src = "kaam f(n) { wapas f(n + 1); } f(0);";
    let r = std::thread::Builder::new().stack_size(512 << 20).spawn(move || out(src)).unwrap().join().unwrap();
    assert_eq!(r, "ERROR: RecursionError: maximum recursion depth exceeded");
}

#[test]
fn errors_match_python_messages() {
    assert_eq!(out("dikhao(\"a\" + 1);"), "ERROR: TypeError: can only concatenate str (not \"int\") to str");
    assert_eq!(out("dikhao(y);"), "ERROR: AdarshSemanticError: Variable 'y' not declared in AdarshLang scope.");
    assert_eq!(out("chhoddo(1);"), "ERROR: AdarshUserException: User exception");
}
