# Test Matrix

Run the following commands before deploying or submitting a pull request:

```bash
python -m py_compile adarsh_lang_compiler.py
python adarsh_lang_compiler.py test.aak
python adarsh_lang_compiler.py test2.aak
python adarsh_lang_compiler.py test_lists.aak
python adarsh_lang_compiler.py test_loops.aak
python adarsh_lang_compiler.py test_features.aak
```

* `python -m py_compile` ensures that all package imports resolve correctly.
* Each `.aak` program exercises a different slice of the language (core syntax, loops, lists, advanced features) and should complete without raising a runtime error.
