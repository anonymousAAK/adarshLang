from pathlib import Path
from typing import Dict, List

REPO_URL = "https://github.com/adardsh12/adarshLang"
README_REFERENCE_URL = "https://github.com/adardsh12/adarshLang#language-features"

FEATURES: List[Dict[str, str]] = [
    {
        "title": "Variables",
        "description": "Declare with `badlo` and reassign freely, e.g. `badlo x = 10;`.",
    },
    {
        "title": "Printing",
        "description": "Use `dikhao(expression);` to write output to stdout.",
    },
    {
        "title": "Booleans",
        "description": "`sahi_hai_be` (true) and `jhuth` (false) power comparisons and control flow.",
    },
    {
        "title": "Conditionals",
        "description": "Express decisions with `agar`, optional `warna agar`, and an ending `warna` block.",
    },
    {
        "title": "Loops",
        "description": "Iterate via `jabtak` (while), `ginnati` (C-style for), and `ke_liye` (foreach).",
    },
    {
        "title": "Switch Expressions",
        "description": "`chuno (expr) { case ... warna_case ... }` offers expressive multi-branch logic.",
    },
    {
        "title": "Functions",
        "description": "Define work with `kaam`, including defaults, `baaki` varargs, and anonymous lambdas.",
    },
    {
        "title": "Collections",
        "description": "Lists plus helpers like `push`, `pop`, and `length`, plus `dhacha` records for structure.",
    },
    {
        "title": "Exceptions",
        "description": "Wrap risky code with `pakdo { ... } chhoddo (err) { ... }` and raise via `chhoddo`.",
    },
    {
        "title": "Modules",
        "description": "Reuse code with `lao \"mera_module.aak\";` and share utilities across files.",
    },
    {
        "title": "Functional Helpers",
        "description": "Built-ins such as `map`, `filter`, `reduce`, `join`, and `split` simplify data transforms.",
    },
]

TUTORIAL_TRACKS: List[Dict[str, str]] = [
    {
        "id": "basics",
        "label": "Basics",
        "body": """
`badlo` introduces variables, while semicolons terminate statements. Strings use double quotes, and comments start with `//`.
""",
    },
    {
        "id": "control-flow",
        "label": "Control Flow",
        "body": """
Use `agar`/`warna` for branching, `jabtak` for while loops, and `ginnati`/`ke_liye` when you need counting or foreach behavior.
""",
    },
    {
        "id": "collections",
        "label": "Collections",
        "body": """
Lists are literal with `[1, 2, 3]` and support helpers such as `push(list, value)` and `length(list)`. `dhacha` creates record-like types.
""",
    },
    {
        "id": "errors",
        "label": "Errors",
        "body": """
Guard risky code inside `pakdo { ... } chhoddo (err) { ... }` and raise intentful errors with `chhoddo("message");`.
""",
    },
]

GETTING_STARTED_COMMANDS: List[str] = [
    "pip install -r requirements.txt",
    "python app.py",
]

GETTING_STARTED_TIPS: List[str] = [
    "Visit http://localhost:5000 and paste one of the sample programs.",
    "Edit the snippet in the in-browser editor and re-run instantly.",
    "Share `.aak` files with friends by dropping them in the repo and linking them from the Samples section.",
]

SAMPLE_PROGRAMS: List[Dict[str, object]] = [
    {
        "name": "FizzBuzz",
        "path": Path("test.aak"),
        "description": "Classic fizzbuzz showing `ginnati`, `%`, and conditional logic.",
    },
    {
        "name": "List Playground",
        "path": Path("test_lists.aak"),
        "description": "Play with list helpers, indexing, and printing results.",
    },
    {
        "name": "Loops & Breaks",
        "path": Path("test_loops.aak"),
        "description": "Demonstrates `jabtak`, `bas`, and `aage_badho` for control flow finesse.",
    },
    {
        "name": "Extras",
        "path": Path("extras.aak"),
        "description": "A grab bag of language constructs from the repo's examples.",
    },
]


def load_program_source(program: Dict[str, object]) -> str:
    path = program["path"]
    if isinstance(path, Path) and path.exists():
        return path.read_text(encoding="utf-8")
    return "// Sample program missing from deployment."


__all__ = [
    "FEATURES",
    "TUTORIAL_TRACKS",
    "GETTING_STARTED_COMMANDS",
    "GETTING_STARTED_TIPS",
    "SAMPLE_PROGRAMS",
    "load_program_source",
    "REPO_URL",
    "README_REFERENCE_URL",
]
