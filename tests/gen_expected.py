#!/usr/bin/env python3
"""Regenerate golden outputs from the Python reference implementation.

For every tests/conformance/NAME.aak writes tests/conformance/expected/NAME.out
containing stdout, followed by a final line `ERROR: <Kind>: <message>` if the
program raised. `cargo test` (adarsh-rs/tests/conformance.rs) checks against these.
"""
import glob, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from compare import ROOT, run_python

out_dir = os.path.join(ROOT, 'tests/conformance/expected')
os.makedirs(out_dir, exist_ok=True)
for f in sorted(glob.glob(os.path.join(ROOT, 'tests/conformance/*.aak'))):
    out, err = run_python(f)
    if err:
        out += f'ERROR: {err}\n'
    name = os.path.splitext(os.path.basename(f))[0]
    with open(os.path.join(out_dir, name + '.out'), 'w', encoding='utf-8') as h:
        h.write(out)
    print('wrote', name + '.out')
