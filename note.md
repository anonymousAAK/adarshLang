AdarshLang ab support karta hai:

- Variable declarations/assignments with `badlo`, arithmetic aur logical expressions.
- Booleans (`sahi_hai_be`, `jhuth`), conditionals `agar/warna/warna agar`.
- Teen tarah ke loops: `jabtak` (while), `ginnati` (counted for) aur `ke_liye (value in items)` foreach collections.
- Loop control `bas` (break) aur `aage_badho` (continue).
- `chuno` / `case` multi-branch switch expressions with optional `warna_case` default.
- Functions via `kaam` including default arguments, `baaki` varargs aur anonymous `kaam (...) { ... }` expressions.
- First-class functions: assign karo, pass karo, map/filter/reduce helpers ke saath use karo.
- List literals, indexing/assignment aur list helpers (`length`, `push`, `pop`).
- Dictionary/object literals `{ key: value }`, helpers `rakho`/`nikalo`, attribute access `obj.field`.
- `dhacha` definitions for typed records with brace construction syntax.
- Exception handling: `pakdo { ... } chhoddo (err) { ... }` try/catch plus `chhoddo(expr);` user throws.
- Imports using `lao "dusra_file.aak";` to share code modules.
- Extended builtin library: math (`abs`, `floor`, `ceil`), string helpers (`upper`, `lower`, `join`, `split`), list/functional helpers (`map`, `filter`, `reduce`), randomness/time (`random_number`, `current_time`) and more.
- Interactive REPL: `python adarsh_lang_compiler.py --repl` (ya bina arguments) for quick experimentation.

Compiler pipeline wahi hai: lexer -> parser -> semantic analyzer -> interpreter with runtime errors as `AdarshRuntimeError` aur user exceptions propagate hote hain.
