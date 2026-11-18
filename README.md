# AdarshLang

AdarshLang is a Hinglish programming language: `badlo` declares a variable, `dikhao` prints, `agar` / `warna` branch, and `kaam` defines a function.
It's built for learning, demos and workshops. The interpreter is written in Rust and also runs in the browser through WebAssembly.

```aak
dhacha Vyakti { naam, umar };

kaam Vyakti.parichay() {
    wapas "Main {yeh.naam} hoon, umar {yeh.umar}.";
}

badlo log = [Vyakti { naam: "Adarsh", umar: 24 }, Vyakti { naam: "Priya", umar: 17 }];
ke_liye (badlo p in log) {
    agar (p.umar >= 18 aur p.naam != khali) {
        dikhao(p.parichay());
    } warna {
        dikhao("{p.naam} abhi chhota hai.");
    }
}
```

```text
Main Adarsh hoon, umar 24.
Priya abhi chhota hai.
```

## Try it

- **In the browser:** open the playground (`playground/`, deployed on Vercel). Nothing to install.
- **On your computer:** download `adarsh` for Windows, macOS or Linux from the [Releases](https://github.com/anonymousAAK/adarshLang/releases) page, or build it yourself:

```bash
cargo install --git https://github.com/anonymousAAK/adarshLang adarshlang
```

```bash
adarsh hello.aak                 # run a program
adarsh                           # interactive REPL (history, multi-line input)
adarsh check hello.aak           # find errors without running
adarsh test                      # run test_* functions in *_test.aak files
adarsh samjhao E011              # explain an error code
adarsh python hello.aak          # see the same program in Python
adarsh build hello.aak -o hello  # standalone executable; runs without AdarshLang installed
adarsh fmt hello.aak             # format code (--check to only report)
adarsh docs hello.aak            # Markdown docs from ## comments
adarsh debug hello.aak           # step through line by line
adarsh web hello.aak             # one HTML file that runs offline in a browser
adarsh add <git url> as ganit    # add a package to aak_modules/ (then: lao "ganit")
```

Editors: the VS Code extension in `editors/vscode` gets errors, hover, completion, go-to-definition and formatting from `adarsh lsp`. Jupyter notebooks: see `editors/jupyter`.

## The language at a glance

```aak
pakka GST = 0.18;
dhacha Cheez { naam, daam = 0 };
kaam Cheez.shabd() { wapas "{yeh.naam} ({rupaye(yeh.daam)})"; }

badlo tokri = [Cheez { naam: "Chai", daam: 20 }, Cheez { naam: "Samosa", daam: 15 }];
ke_liye (badlo i, c in tokri) { dikhao("{i + 1}. {c}"); }

badlo kul = tokri.map(c => c.daam).sum();
dikhao("Kul: {rupaye(kul * (1 + GST))}");
dikhao(agar (kul > 30) "bada order" warna "chhota order");
```

```text
1. Chai (₹20.00)
2. Samosa (₹15.00)
Kul: ₹41.30
bada order
```

| Area | What you get |
| --- | --- |
| Basics | `badlo` / `pakka` constants, numbers that never overflow, strings with `{interpolation:.2}`, raw `r"..."` and `"""multi-line"""` strings, Devanagari keywords |
| Control flow | `agar` / `warna`, inline `agar (c) a warna b`, `jabtak`, `karo ... jabtak`, `ginnati`, `ke_liye (badlo i, x in xs)`, ranges `1..10`, labelled `bas` / `aage_badho` |
| Matching | `chuno` with values, ranges (`case 90..100:`), types, list shapes (`case [pehla, ...rest]:`) and `agar` guards |
| Functions | defaults, named arguments, `baaki` / `...` varargs, spread, lambdas `x => x * 2`, closures, generators with `dete_raho`, dot-calls `xs.sort()` |
| Data | lists, dicts, tuples, sets, slices `xs[1:3]` / `[::-1]`, destructuring `badlo [a, b] = ...`, `in`, `?.` and `??` |
| Types | `dhacha` records with defaults, inheritance, methods, custom printing and operator methods; `vikalp` enums; `x hai number`; optional annotations `badlo umar: number` |
| Errors | `pakdo` / `chhoddo` / `aakhir`, typed catches, Hinglish messages with line, column, call chain, "did you mean?" and error codes |
| Library | 100+ builtins: math, strings, lists, sets, dates, JSON, files, internet, random, Indian number formats (`15,00,000`, `₹99.50`, "ek sau pachchees"), turtle graphics |

**Full reference: [docs/REFERENCE.md](docs/REFERENCE.md)**

## Project layout

| Path | What |
| --- | --- |
| `adarsh-rs/` | The interpreter (Rust): lexer → parser → semantic checks → interpreter, plus the CLI and REPL |
| `playground/` | Static web playground; the interpreter runs as WebAssembly in a Web Worker |
| `tests/conformance/` | Test programs with expected outputs (`cargo test` runs them) |
| `examples/` | Sample programs |
| `editors/vscode/` | VS Code extension: syntax highlighting and snippets for `.aak` |
| `legacy/python/` | The original Python implementation (frozen, for reference) |

## Development

```bash
cd adarsh-rs
cargo test --release                  # all tests
BLESS=1 cargo test --release          # after an intended change: rewrite expected outputs, then review the diff
cargo run --release -- ../examples/test.aak
../scripts/build-playground.sh        # rebuild playground/adarshlang.wasm
python3 ../scripts/fuzz.py --cases 4000   # throw thousands of broken programs at it; none may crash
```

CI (`.github/workflows/ci.yml`) runs tests on Windows, macOS and Linux, plus `rustfmt`, `clippy` and the WebAssembly build.
Pushing a tag such as `v0.2.0` builds release binaries for all three operating systems (`.github/workflows/release.yml`).
