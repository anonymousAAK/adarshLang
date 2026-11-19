# Changelog

## 0.3.0

### Language
- Ranges `1..10` (inclusive; `5..1` counts down), `ke_liye (badlo i, x in xs)` for index + value (key + value for dicts).
- `pakka` constants; `PI` and `E` built in.
- Inline `agar (c) a warna b`; chained comparisons `0 < x < 10`; `in`; `x hai type`; `?.` and `??`; `//` floor division.
- Dot-calls for every builtin and function: `naam.upper()`, `xs.sort(by: f)`, `xs.map(x => x * 2)`.
- Slices `xs[a:b:step]` on lists, strings and tuples.
- `karo { } jabtak (c);` do-while; labelled loops with `bas label;` / `aage_badho label;`.
- `badlo a = 1, b = 2;` and destructuring `badlo [a, b, ...rest] = xs;` / `badlo { naam, umar } = p;`.
- Raw strings `r"..."`, multi-line `"""..."""`, block comments `#[ ]#`, `##` doc comments.
- Lambdas `(a, b) => a + b` / `x => x * x`; spread `f(...xs)` / `[...a, ...b]`; named arguments `f(naam: "A")`.
- `aakhir` (finally); `chuno` patterns: ranges, types, list shapes, `agar` guards; typed catches `chhoddo (e: Type)`.
- Generators with `dete_raho`; `ruko(sec)`, `band_karo(code)`.
- Tuples, sets (`set`, `union`, `intersection`, `difference`), enums `vikalp Rang { Laal, Hara };`.
- `dhacha` field defaults, inheritance `dhacha Student : Vyakti { }`, inherited methods, `shabd()` for custom printing, operator methods (`jodo`, `ghatao`, `guna`, `bhaag`, `barabar`, `chhota`).
- Optional type annotations: `badlo umar: number = 24;`, `kaam f(x: number) -> number`.
- Devanagari spellings of the keywords (`दिखाओ`, `बदलो`, `अगर`, ...).

### Standard library
- Math (`sqrt`, trig, `log`, `exp`, `gcd`, `lcm`, `is_prime`, `round(x, ank)`), strings (`starts_with`, `pad_left`, `title`, `format`, ...), lists (`sum`, `min`/`max` with `by:`, `any`, `all`, `zip`, `unique`, `flatten`, `group_by`, `insert`), random (`random_int`, `random_choice`, `shuffle`).
- `{value:spec}` formatting in strings: `.2`, `>8`, `03`, `,`, `%`.
- Indian formats: `lakh_format`, `rupaye`, `shabdon_mein` (numbers in Hindi words).
- Dates: `aaj`, `abhi`, `tareekh`, `din_ka_naam`, `tareekh_jodo`, `din_ka_fark`.
- JSON (`json_banao`, `json_padho`), files (`file_padho`, `file_likho`, `file_jodo`, `file_hai`), `internet_se`, terminal colours `rang_mein`, `padho_number`.
- Turtle graphics with `kachhua` (SVG in the CLI, canvas in the playground).

### Learning
- "Did you mean?" suggestions for misspelt names, fields, types and parameters.
- Warnings: unused variables, unreachable code, variables hiding builtins.
- Error codes with `adarsh samjhao E001`, `--english` titles, and the call chain for runtime errors.
- `jaancho` plus `adarsh test` (runs `test_*` functions in `*_test.aak` files).
- `adarsh python file.aak` shows the same program in Python.

### Performance
- Simple functions run on a new bytecode VM; everything else on a faster tree-walker (cached variable slots, vector scopes, interned names, integer fast paths).
- Compared with the interpreter just before this work, on the programs in `benchmarks/`: recursive calls 3.8x faster, a tight loop 3.6x, a sieve 2x, list and dict work 1.25x, top-level loops 1.6x.
- `ADARSH_NO_VM=1` turns the VM off; the test suite checks both engines give identical results.
- "Did you mean" suggestions no longer change from run to run.

### Tooling
- `adarsh fmt` formatter, `adarsh docs` (Markdown from `##` comments), `adarsh debug` step-through debugger, `adarsh web` single-file HTML export.
- Packages: `adarsh add` / `remove` / `install` with an `adarsh.json` manifest; `lao "name"` finds packages in `aak_modules/`.
- Language server (`adarsh lsp`): diagnostics, hover, completion, go-to-definition, formatting, document symbols. The VS Code extension uses it.
- Jupyter kernel in `editors/jupyter` (cells share state, turtle drawings show as images).

## 0.2.0
- Rust becomes the official implementation; Hinglish errors with line and column; interpolation; records with methods; `aur`/`ya`/`nahin`; WebAssembly playground; CLI `build`; VS Code extension; CI and releases.

## 0.1.0
- Original Python implementation (now in `legacy/python`).
