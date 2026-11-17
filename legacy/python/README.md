# Legacy Python implementation (frozen)

This is the original AdarshLang interpreter written in Python (v0.1), kept for reference.
**The official implementation is now the Rust one in [`../../adarsh-rs`](../../adarsh-rs).**

It no longer receives fixes or features, and behaves differently from the Rust version.
For example, it has no `aur`/`ya`/`nahin`, no string interpolation, and its `&&`/`||` don't lex.
The Flask playground (`app.py`) has been replaced by the static WebAssembly playground in [`../../playground`](../../playground).

```bash
python adarsh_lang_compiler.py ../../examples/test.aak
```
