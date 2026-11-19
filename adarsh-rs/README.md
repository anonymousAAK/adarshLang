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
| `src/lexer.rs` | Tokens, including string interpolation, raw and triple-quoted strings, Devanagari keywords and `##` doc comments |
| `src/parser.rs` | Recursive-descent parser. Hoists `kaam`/`dhacha`/`vikalp` definitions to the top of each block and interns every name |
| `src/semantics.rs` | Static checks before running: undeclared names (with "did you mean"), argument counts, `pakka`, types, labels, warnings |
| `src/interp.rs` | Tree-walking interpreter: scopes, closures, records and methods, patterns, generators, `pakdo` |
| `src/vm.rs` | Bytecode compiler and stack machine for simple functions (see Performance) |
| `src/value.rs` | Values: arbitrary-precision integers, tuples, sets, enums, printing, equality, ordering, arithmetic |
| `src/builtins.rs` | The 107 builtin functions |
| `src/stdlib.rs` | Formatting, Indian number formats, dates, JSON, colours, turtle graphics, host access (files, internet, clock) |
| `src/error.rs` / `src/explain.rs` | Hinglish errors with codes, file:line:col, a caret and the call chain; `adarsh samjhao` texts |
| `src/loader.rs` | `lao` module loading: filesystem and `aak_modules/`, in-memory (bundled executables), or none |
| `src/fmt.rs`, `src/docs.rs`, `src/topython.rs` | `adarsh fmt`, `adarsh docs`, `adarsh python` |
| `src/lsp.rs` | `adarsh lsp`, the language server |
| `src/webexport.rs` | `adarsh web`: one HTML page with the WebAssembly build inside |
| `src/main.rs` | The CLI and REPL |
| `src/wasm.rs` | WebAssembly exports used by the playground |

## Performance

Three things keep it fast:

- **Cached variable slots.** Names are interned when parsing, and scopes keep variables in a small vector compared by pointer; only big scopes such as the global one (which holds every builtin) use a hash index. Each variable use in the syntax tree remembers where it found the name last time and checks that hint on every use, so a stale hint costs one slow lookup, never a wrong value.
- **A bytecode VM for simple functions.** On its first call, a `kaam` that only uses the common subset (locals, numbers, strings, operators, `agar`, `jabtak`, `ke_liye`, `wapas`, `bas`/`aage_badho`, `dikhao`, lists, dicts, indexing, fields, calls and dot-calls) is compiled to bytecode. Its locals live on a shared value stack, and bytecode calling bytecode passes arguments in place. Every other function (lambdas, nested functions, methods, generators, `pakdo`, `chuno`, typed locals, ...) and all top-level code stays on the tree-walker, so each feature has exactly one implementation.
- **Fast paths** for small-integer arithmetic and comparisons.

The VM must give exactly the tree-walker's output, errors and call chains: the conformance suite runs every program both ways. `ADARSH_NO_VM=1` turns the VM off; `ADARSH_VM_LOG=1` prints which functions it compiled. Step-through debugging always uses the tree-walker.

`../benchmarks/run.sh` times the benchmark programs. Native compilation is not implemented.

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

- Top-level code and functions outside the VM's subset run on the tree-walker (about 1.6x faster than before the performance work, but slower than bytecode).
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
