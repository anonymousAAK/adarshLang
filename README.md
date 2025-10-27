# AdarshLang

AdarshLang is a Hinglish-flavoured programming language built for fun demos and workshops. This repository ships a full compiler pipeline (lexer → parser → semantic analyzer → interpreter), a Flask playground, and sample `.aak` programs.

## Getting Started

```bash
python -m pip install -r requirements.txt
python -m adarsh_lang test.aak
```

* Run the interactive REPL with `python -m adarsh_lang --repl` and finish a snippet with a blank line.
* Launch the web runner locally by setting `FLASK_APP=app.py` and executing `flask run`.

## Language Features

| Feature | Syntax | Notes |
| --- | --- | --- |
| Variables | `badlo x = 42;` | Implicit declarations with `badlo` and later assignments. |
| Arithmetic & logic | `+ - * /`, comparisons, `aur`/`ya`/`nahin` | Uses standard Python semantics. |
| Booleans | `sahi_hai_be`, `jhuth` | Aliases of `True`/`False`. |
| Printing | `dikhao(expr);` | Sends values to stdout. |
| Conditionals | `agar (...) { ... } warna { ... }` | Supports chained `warna agar`. |
| While loops | `jabtak (condition) { ... }` | Break with `bas;`, continue with `aage_badho;`. |
| Counted loops | `ginnati (init; condition; update) { ... }` | Traditional C-style loop. |
| Foreach loops | `ke_liye (badlo value in items) { ... }` | Accepts either `in` or legacy colon separator. |
| Switch | `chuno (expr) { case ... warna_case ... }` | Multiple matches per case allowed. |
| Functions | `kaam naam(params) { ... }` | Default args, `baaki` varargs, anonymous `kaam (...)` expressions. |
| Returns | `wapas expr;` | `wapas;` is valid for early exit without a value. |
| Lists | `[1, 2, 3]` | Supports indexing, assignment, `push`, `pop`, `length`. |
| Dictionaries | `{ "key": value }` | Works with `rakho`, `nikalo`, and attribute access (`obj.field`). |
| Dhacha records | `dhacha Vyakti { naam, umar };` | Construct with `Vyakti { naam: "Adarsh", umar: 24 }`. |
| Exceptions | `pakdo { ... } chhoddo (err) { ... }` | Throw via `chhoddo(expr);`. |
| Modules | `lao "extras.aak";` | Loads and executes other source files once. |
| Builtins | `abs`, `floor`, `ceil`, `upper`, `lower`, `join`, `split`, `map`, `filter`, `reduce`, `random_number`, `current_time` | Register automatically in every runtime. |
| Records & collections helpers | `rakho`, `nikalo` | Provide dictionary/list convenience methods. |
| Functional helpers | `map`, `filter`, `reduce` | Accept user functions as arguments. |

## Project Layout

```
adarsh_lang/          # Package containing the compiler pipeline
├── ast.py            # AST node definitions
├── lexer.py          # Token class and lexer
├── parser.py         # Recursive-descent parser
├── semantics.py      # Symbol table + semantic analyzer
├── runtime.py        # Interpreter and builtin runtime helpers
├── pipeline.py       # Helper utilities + CLI entry point
└── __init__.py       # Public API for external callers
```

Use `python -m adarsh_lang` for the command-line interface or import compiler pieces directly from the `adarsh_lang` package.

## Example Program

```aak
lao "extras.aak";

dhacha Vyakti { naam, umar };

kaam banakar(naam, umar = 18) {
    wapas Vyakti { naam: naam, umar: umar };
}

badlo hero = banakar("Adarsh", 24);
badlo nums = [1, 2, 3];

ke_liye (badlo value in nums) {
    dikhao(value);
}
```

## Deployment Notes

* Package and ship the `adarsh_lang` directory; the module exposes both programmatic helpers and the CLI entry point (`python -m adarsh_lang`).
* The Flask app (`app.py`) embeds a quick tutorial and can be deployed directly to serverless hosts such as Vercel (see `vercel.json`).
* Include sample `.aak` programs in deployments so the feature list remains demonstrable.

## Contributing

1. Fork the repository and create a feature branch.
2. Run the test commands listed in [`TEST_README.md`](TEST_README.md) before opening a pull request.
3. Submit a PR with a summary of language/runtime changes and updated docs.
