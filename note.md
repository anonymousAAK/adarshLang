Variable declarations and assignments (using badlo)
Arithmetic expressions (+, -, *, /, parentheses)
Comparison and Boolean operators (<, >, <=, >=, ==, !=, aur, ya, nahin)
agar-warna statements (including warna agar chains)
jabtak loops
dikhao statements
Simple function definitions (kaam) and returns (wapas)
List literals ([...]) and indexing (expr[expr])
Indexed assignment (expr[expr] = value)
Loop control with bas (break) and aage_badho (continue)
Built-in helpers such as length(expr) for sequences
List helpers push(list, value) and pop(list)
The compiler workflow is the same as before:

Lexical Analysis – Converts raw input into a stream of tokens.
Parsing – Builds an Abstract Syntax Tree (AST) using a recursive descent parser.
Semantic Analysis – Checks for correctness (declared variables, function signatures, etc.).
Code Generation / Execution – Directly interprets the AST in this toy version.