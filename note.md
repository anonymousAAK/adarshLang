## AdarshLang Feature Guide

### Core Syntax
- **Variables:** Declare with `badlo`, reassign later by name.
- **Expressions:** Supports arithmetic, comparison, and logical operators (`aur`, `ya`, `nahin`).
- **Booleans:** `sahi_hai_be` (true) and `jhuth` (false).
- **Printing:** `dikhao(expr);` writes to stdout.
- **Conditionals:** `agar (...) { ... } warna { ... }` plus chained `warna agar` branches.

### Control Flow
- **Loops:**
  - `jabtak` while loops.
  - `ginnati (init; condition; update)` counted loops.
  - `ke_liye (badlo value in iterable)` foreach loops (also accepts the legacy colon separator).
- **Loop Control:** `bas;` (break) and `aage_badho;` (continue).
- **Switching:** `chuno (expr) { case ... warna_case ... }` with multiple match expressions per case.

### Functions & Exceptions
- **Functions:** `kaam naam(params) { ... }` with default values and `baaki` varargs. Anonymous `kaam (...) { ... }` works anywhere an expression is allowed.
- **Returns:** `wapas expr;` or `wapas;` for void returns.
- **Exceptions:** `pakdo { ... } chhoddo (err) { ... }` and user throws via `chhoddo(expr);`.

### Data Structures
- **Lists:** Literals (`[1, 2, 3]`), indexing, assignment, and helpers `length`, `push`, `pop`.
- **Dictionaries:** `{ "key": value }` with helpers `rakho`, `nikalo`, and dotted attribute-style access.
- **Dhacha Records:** `dhacha Vyakti { naam, umar };` and constructions like `Vyakti { naam: "Adi", umar: 21 }`.

### Modules & Builtins
- **Modules:** `lao "extras.aak";` loads other `.aak` files once per interpreter.
- **Builtins:** Math (`abs`, `floor`, `ceil`), strings (`upper`, `lower`, `join`, `split`), collection utilities (`map`, `filter`, `reduce`, `rakho`, `nikalo`), randomness/time (`random_number`, `current_time`).

### Deployment Tips
- Import compiler pieces directly from `adarsh_lang`; the legacy monolithic module has been retired in favour of the package entry point.
- The Flask web UI in `app.py` is deployment ready; set `FLASK_APP=app.py` locally or lean on the provided `vercel.json` for Vercel setups.
- Bundle `extras.aak` and sample programs (`test_*.aak`) with deployments so demonstrations cover every feature.
- Use the REPL (`python -m adarsh_lang --repl`) for quick health checks after deployment.

### Debugging Hints
- Run `python -m py_compile adarsh_lang/*.py` to validate imports and module wiring.
- Execute regression suites listed in `TEST_README.md` to confirm runtime behaviour before shipping updates.
