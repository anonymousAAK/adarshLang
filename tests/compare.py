#!/usr/bin/env python3
"""Differential test: run each .aak program through the Python reference
implementation and the Rust binary, and compare stdout + error.

Usage: python3 tests/compare.py [path/to/adarsh] [files...]
Programs run with their own directory as cwd (so `lao` imports resolve).
Lines containing `random_number` / `current_time` output are skipped by marking
programs with a `# nondeterministic` comment: only the error/exit is compared.
"""
import glob, io, os, subprocess, sys, contextlib, traceback

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
sys.setrecursionlimit(100000)

def run_python(path):
    from adarsh_lang import adarshlang_compile_and_run
    import adarsh_lang.runtime as rt
    src = open(path, encoding='utf-8').read()
    buf = io.StringIO()
    err = None
    cwd = os.getcwd()
    os.chdir(os.path.dirname(path))
    try:
        with contextlib.redirect_stdout(buf):
            adarshlang_compile_and_run(src)
    except rt.AdarshUserException:
        err = 'AdarshUserException: User exception'
    except Exception as e:  # noqa: BLE001
        name = type(e).__name__
        if name == 'RuntimeError':
            name = 'RuntimeError'
        err = f'{name}: {e}'
    finally:
        os.chdir(cwd)
    return buf.getvalue(), err

def run_rust(binary, path):
    p = subprocess.run([binary, os.path.basename(path)], cwd=os.path.dirname(path),
                       capture_output=True, text=True, timeout=60)
    err = p.stderr.strip() or None
    return p.stdout, err

def main():
    binary = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, 'adarsh-rs/target/release/adarsh')
    files = sys.argv[2:] or sorted(glob.glob(os.path.join(ROOT, '*.aak')) +
                                   glob.glob(os.path.join(ROOT, 'tests/conformance/*.aak')))
    failed = 0
    for f in files:
        f = os.path.abspath(f)
        src = open(f, encoding='utf-8').read()
        nondet = 'random_number' in src or 'current_time' in src
        py_out, py_err = run_python(f)
        rs_out, rs_err = run_rust(binary, f)
        ok = (py_err == rs_err) and (nondet or py_out == rs_out)
        if nondet and ok:
            ok = len(py_out.splitlines()) == len(rs_out.splitlines())
        rel = os.path.relpath(f, ROOT)
        print(f"{'PASS' if ok else 'FAIL'}  {rel}{'  (nondeterministic: output not compared)' if nondet else ''}")
        if not ok:
            failed += 1
            if py_err != rs_err:
                print(f"    python error: {py_err}\n    rust   error: {rs_err}")
            pl, rl = py_out.splitlines(), rs_out.splitlines()
            for i in range(max(len(pl), len(rl))):
                a = pl[i] if i < len(pl) else '<missing>'
                b = rl[i] if i < len(rl) else '<missing>'
                if a != b:
                    print(f"    line {i+1}: python={a!r}\n    {' '*len(str(i+1))}       rust  ={b!r}")
                    break
    print(f"\n{len(files) - failed}/{len(files)} passed")
    sys.exit(1 if failed else 0)

if __name__ == '__main__':
    main()
