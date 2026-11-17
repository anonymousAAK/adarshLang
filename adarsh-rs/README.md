# AdarshLang in Rust

A native-code port of the AdarshLang compiler pipeline (lexer → parser → semantic
analyzer → interpreter). It produces the same output and the same error messages
as the original Python implementation in `../adarsh_lang/`, and runs about 15×
faster.

## Build and run

```bash
cargo build --release
./target/release/adarsh ../test.aak      # run a program
./target/release/adarsh                  # REPL (finish a snippet with a blank line)
```

## Tests

```bash
cargo test --release                     # golden-output suite + targeted tests
python3 ../tests/compare.py              # live diff against the Python implementation
python3 ../tests/gen_expected.py         # regenerate golden files from Python
```

`../tests/compare.py` runs every `*.aak` file in the repository root and
`../tests/conformance/` through both implementations. It compares stdout and
the final error (`Kind: message`).

## Layout

| File | Python counterpart | Role |
| --- | --- | --- |
| `src/lexer.rs` | `tokens.py`, `lexer.py` | Tokens, lexing |
| `src/ast.rs` | `ast.py` | AST as Rust enums |
| `src/parser.rs` | `parser.py` | Recursive-descent parser |
| `src/semantics.rs` | `semantics.py` | Static checks: declarations, arity, `bas` placement |
| `src/interp.rs` | `runtime.py` | Tree-walking interpreter, scopes, calls |
| `src/builtins.rs` | `runtime.py` `_builtin_*` | All 33 builtins |
| `src/value.rs` | Python objects | Values with Python semantics: printing, `==`, ordering, arithmetic |
| `src/loader.rs` | `os.path` in `lao` | Module loading (pluggable, so it can run with no filesystem) |
| `src/main.rs` | `pipeline.py` | CLI and REPL |

## Compatibility notes

The port copies the Python behaviour the language depended on without spelling
it out:

- Integers have arbitrary precision (`2 ** 100` is exact) and `int / int`
  division is correctly rounded.
- Floats print like Python's `repr`: `0.30000000000000004`, `1e+16`, `5.0`.
- Values inside lists and dicts print Python-style (`[1, True, None]`), while
  top-level `dikhao` prints `sahi_hai_be` / `jhuth` / `khali`.
- `True == 1 == 1.0`, so they share a dict key.
- `round_val` uses banker's rounding, and `%` takes the sign of the divisor.
- Type errors use Python's wording, e.g.
  `can only concatenate str (not "int") to str`.

## Intentional differences (bug fixes)

| Behaviour | Python | Rust |
| --- | --- | --- |
| `&&` / `\|\|` | Lexer error, `Unexpected character '&'` | Work, as the README documents |
| A `jabtak` loop whose last iteration ends in `aage_badho` | Silently stops the rest of the program | Continues normally |
| Very deep recursion | `RecursionError` after about 110 nested AdarshLang calls | Allows 10,000 nested calls, then `RecursionError` |
| Printing a function value | `<adarsh_lang.runtime.AdarshFunctionValue object at 0x...>` | `<kaam name>` |
| `random_number` | Python's Mersenne Twister | xorshift64* (different random sequence) |

## Next steps

1. **WebAssembly build**, so the playground runs `.aak` code in the browser
   (the loader and output sink are already pluggable).
2. **Speed:** resolve variables to slots at compile time instead of hashing names,
   then a bytecode VM.
3. **Native compilation** of `.aak` programs (Cranelift or LLVM backend).
