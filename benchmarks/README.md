# Benchmarks

```bash
./benchmarks/run.sh path/to/adarsh
```

| Program | What it exercises |
| --- | --- |
| `fib.aak` | Recursive function calls |
| `loop.aak` | A tight loop with arithmetic and local variables, inside a function |
| `sieve.aak` | List indexing and assignment |
| `lists.aak` | List building, dict counting and string interpolation |
| `toplevel_loop.aak` | The same loop as `loop.aak` at the top level, where the tree-walker runs it |

Best of three runs on the development machine, before and after the performance work in 0.3 (same language features in both):

| Program | Before | After | Speed-up |
| --- | --- | --- | --- |
| fib | 0.20 s | 0.05 s | 3.8x |
| loop | 0.96 s | 0.26 s | 3.6x |
| sieve | 0.61 s | 0.30 s | 2.0x |
| lists | 0.29 s | 0.24 s | 1.25x |
| toplevel_loop | 0.94 s | 0.59 s | 1.6x |

Wall-clock times are noisy on shared machines. For stable comparisons, count instructions with `valgrind --tool=callgrind`.
To compare the two engines, run with `ADARSH_NO_VM=1` (tree-walker only).
