# adarshlang (Rust crate)

The official AdarshLang implementation: library, `adarsh` CLI, and WebAssembly build for the playground.
The language itself is documented in the [main README](../README.md).

## Build

```bash
cargo build --release                       # target/release/adarsh
cargo test --release                        # all tests
../scripts/build-playground.sh              # WebAssembly → ../playground/adarshlang.wasm
```

## Pipeline

| File | Role |
| --- | --- |
| `src/lexer.rs` | Tokens. Handles string interpolation, number formats, and `aur`/`ya`/`nahin` |
| `src/parser.rs` | Recursive-descent parser. Hoists `kaam`/`dhacha` definitions to the top of each block |
| `src/semantics.rs` | Static checks before running: undeclared names, argument counts, `bas` outside loops, record fields |
| `src/interp.rs` | Tree-walking interpreter: scopes, closures, methods, short-circuit logic, `pakdo` |
| `src/value.rs` | Values: arbitrary-precision integers, printing, equality, ordering, arithmetic |
| `src/builtins.rs` | The 35 builtin functions |
| `src/error.rs` | Hinglish errors with file:line:col and a caret under the problem |
| `src/loader.rs` | `lao` module loading: filesystem, in-memory (bundled executables), or none (browser) |
| `src/main.rs` | CLI: `run`, `check`, `build`, REPL |
| `src/wasm.rs` | WebAssembly exports used by `playground/worker.js` |

## Embedding

```rust
let out = adarshlang::run_to_string("dikhao(\"Namaste {2 + 3}\");").unwrap();
assert_eq!(out, "Namaste 5\n");
```

`adarshlang::run_capture(name, source, loader, input_lines)` returns `(output, Option<rendered_error>)`, the same as the playground shows.

## Standalone executables

`adarsh build app.aak` copies the `adarsh` binary and appends the program, plus any modules it imports with a literal `lao "..."`.
When the result starts, it finds that payload at the end of its own file and runs it.

## Limits

- Recursion: 10,000 nested calls in the CLI, 300 in the browser (browsers give WebAssembly a small native stack).
  Going deeper raises a catchable error instead of crashing.
- The playground stops programs that run longer than 5 seconds.

## Differences from the original Python version (v0.1)

v0.2 made Rust the reference implementation and changed some behaviour on purpose:

- Lists and dicts print in AdarshLang style: `[sahi_hai_be, khali, "text"]`. Records print as `Vyakti { naam: "A" }`.
- `aur` / `ya` / `nahin` are keywords. `&&` / `||` work (they didn't lex in Python), and all of them short-circuit.
- `pakdo` also catches runtime errors. The error variable holds the message.
- A function without `wapas` returns `khali` (Python returned the last statement's value).
- Functions and `dhacha` can be used before their definition and can be nested.
- `{expr}` inside strings is interpolation (`\{` for a literal brace; an empty `{}` stays literal).
- Messages are Hinglish and point at the exact line and column.
