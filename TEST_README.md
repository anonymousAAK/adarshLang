# Test Matrix

Run the following commands before deploying or submitting a pull request:

```bash
python -m py_compile adarsh_lang/*.py
python -m adarsh_lang test.aak
python -m adarsh_lang test2.aak
python -m adarsh_lang test_lists.aak
python -m adarsh_lang test_loops.aak
python -m adarsh_lang test_features.aak
```

* `python -m py_compile adarsh_lang/*.py` ensures that all package imports resolve correctly.
* Each `.aak` program exercises a different slice of the language (core syntax, loops, lists, advanced features) and should complete without raising a runtime error.
