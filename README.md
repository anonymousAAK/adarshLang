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
adarsh build hello.aak -o hello  # standalone executable; runs without AdarshLang installed
```

## The language

### Basics

| | Syntax |
| --- | --- |
| Variable | `badlo x = 10;` (then `x = 11;`) |
| Print | `dikhao(x);` |
| Booleans / null | `sahi_hai_be`, `jhuth`, `khali` |
| Numbers | `42`, `3.14`, `.5`, `1e6`, `1_000_000`; integers never overflow |
| Strings | `"Namaste"` with escapes `\n \t \" \\ \{ \}` |
| Interpolation | `"Naam: {naam}, total: {a + b}"` |
| Comments | `# yeh comment hai` |
| Input | `badlo naam = padho("Aapka naam? ");` (returns `khali` at end of input) |

### Operators

| | |
| --- | --- |
| Arithmetic | `+ - * / % **` (`/` always gives a decimal; `%` follows the divisor's sign) |
| Comparison | `== != < > <= >=` |
| Logic | `aur`, `ya`, `nahin` (or `&&`, `||`, `!`), short-circuit |
| Compound | `+= -= *= /= %=` on variables, `list[i]` and `obj.field` |
| Strings / lists | `"ab" + "cd"`, `"ab" * 3`, `[1] + [2]`, `[0] * 5` |

### Control flow

```aak
agar (marks >= 90) { dikhao("A"); } warna agar (marks >= 75) { dikhao("B"); } warna { dikhao("C"); }

jabtak (n > 0) { n -= 1; }                         # while
ginnati (badlo i = 0; i < 5; i += 1) { dikhao(i); } # for
ke_liye (badlo x in [1, 2, 3]) { dikhao(x); }      # for-each over lists, strings, dict keys
bas;          # break
aage_badho;   # continue

chuno (din) {
    case "shanivaar", "ravivaar": dikhao("Chhutti!");
    case "somvaar":
        dikhao("Kaam shuru");
        dikhao("Chai lo");
    warna_case: dikhao("Normal din");
}
```

### Functions

```aak
kaam jodo(a, b = 10) { wapas a + b; }    # default arguments
kaam total(baaki nums) { wapas reduce(kaam (x, y) { wapas x + y; }, nums, 0); }  # varargs
badlo double = kaam (x) { wapas x * 2; }; # anonymous function / closure
dikhao(map(double, [1, 2, 3]));            # [2, 4, 6]
```

Functions can be called before the line that defines them, can be nested, and capture variables (closures).
A function without `wapas` returns `khali`.

### Data

```aak
badlo xs = [3, 1, 2];   xs[0] = 5;   push(xs, 4);   dikhao(xs[-1]);
badlo d = {"naam": "Adarsh", "umar": 24};   d["shahar"] = "Indore";   hatao(d, "umar");
dikhao(d.naam);         # dict keys can also be read as fields

dhacha Vyakti { naam, umar };             # record type
badlo p = Vyakti { naam: "Adarsh" };      # missing fields are khali
kaam Vyakti.badhao() { yeh.umar += 1; }   # method; `yeh` is the record
```

### Errors

```aak
pakdo {
    dikhao(10 / 0);
} chhoddo (err) {
    dikhao("Pakda: {err}");      # Pakda: Zero se divide nahi kar sakte.
}
chhoddo("Kuch galat hua");      # throw any value
```

`pakdo` catches values thrown with `chhoddo` as well as runtime errors.
Uncaught errors show the file, line and column:

```text
Galti: String mein sirf string jod sakte ho, yahan string + number hai. Interpolation "..{x}.." ya shabdme() use karo.
  --> main.aak:2:10
   |
 2 | dikhao(a + 1);
   |          ^
```

The label tells you when the error was found:
- `Samjho`: the syntax is wrong.
- `Soch`: checks before running found a problem.
- `Galti`: an error while running.
- `Pakda nahi gaya`: a `chhoddo` value nobody caught.

### Modules

```aak
lao "utils";   # runs utils.aak (relative to this file) once
```

### Builtins

| Area | Functions |
| --- | --- |
| Lists | `length push pop sort reverse slice contains find map filter reduce range join` |
| Dicts | `keys values rakho nikalo hatao contains` |
| Strings | `upper lower trim split replace find slice reverse length` |
| Numbers | `abs floor ceil round_val min_val max_val random_number` |
| Types | `prakar` (type name) · `shabdme` (to string) · `sankhya` (to number) |
| Other | `padho` (input) · `current_time` |

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
