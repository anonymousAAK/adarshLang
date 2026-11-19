#!/usr/bin/env python3
"""Fuzz the `adarsh` interpreter with randomly broken programs.

Takes the programs in tests/conformance/ and examples/, applies random
edits (deleting text, inserting keywords and symbols, duplicating chunks)
and runs each result. Every run must finish with exit code 0 (ran) or 1
(reported an AdarshLang error). Anything else, such as a panic, an abort or
a stack overflow, is a crash: the program is kept in the output directory so
it can be reproduced.

Usage:
    python3 scripts/fuzz.py [--bin PATH] [--cases N] [--seed S] [--out DIR]

A timeout usually means the mutated program contains a real infinite loop.
Run `adarsh check FILE` on it: if that finishes, the time is spent running
the program, not in the parser.
"""

import argparse
import concurrent.futures as cf
import glob
import os
import random
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

FRAGMENTS = [
    '{', '}', '(', ')', '[', ']', ';', ',', '.', ':', '"', '\\', '{x}', '"{', '}"', '**', '%', '/',
    '0', '-1', '1e309', '.5', '9' * 40, 'agar', 'warna', 'jabtak', 'ginnati', 'ke_liye', 'chuno', 'case',
    'warna_case', 'kaam', 'wapas', 'badlo', 'dhacha', 'pakdo', 'chhoddo', 'lao', 'bas', 'aage_badho',
    'baaki', 'khali', 'yeh', 'aur', 'ya', 'nahin', 'in', '=', '+=', '==', 'dikhao(', 'push(', 'pop(',
    'hatao(', 'range(', 'slice(', 'sort(', 'map(', 'reduce(', 'padho(', 'sankhya(', '\n', ' ', 'é',
    '\U0001F600', '\t', '#',
    '..', '...', '?.', '??', '//', '=>', 'pakka', 'karo', 'hai', 'vikalp', 'dete_raho', 'aakhir', 'as',
    'r"', '"""', '#[', ']#', 'case [a, ...b]:', 'case 1..5:', ' agar ', 'x => x', '(a, b) => a', 'Rang.Laal',
    'set([1])', '(1,)', 'kachhua.aage(5);', 'json_padho(', 'tareekh(', 'format(', '{x:.2}', 'बदलो', 'दिखाओ(',
]


def mutate(rng, src):
    s = list(src)
    for _ in range(rng.randint(1, 6)):
        op = rng.random()
        i = rng.randint(0, max(0, len(s) - 1))
        if op < 0.35 and s:
            del s[i:i + rng.randint(1, 8)]
        elif op < 0.75:
            s[i:i] = list(rng.choice(FRAGMENTS))
        elif s:
            j = rng.randint(0, len(s) - 1)
            s[i:i] = s[j:j + rng.randint(1, 20)]
    return ''.join(s)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--bin', default=os.path.join(ROOT, 'adarsh-rs', 'target', 'release', 'adarsh'))
    ap.add_argument('--cases', type=int, default=3000)
    ap.add_argument('--seed', type=int, default=1)
    ap.add_argument('--timeout', type=float, default=3.0)
    ap.add_argument('--out', default=None, help='where to keep crashing programs (default: a temp dir)')
    args = ap.parse_args()

    if not os.path.exists(args.bin):
        sys.exit(f'{args.bin} not found; build it first: (cd adarsh-rs && cargo build --release)')
    out = args.out or tempfile.mkdtemp(prefix='adarsh-fuzz-')
    os.makedirs(out, exist_ok=True)
    files = glob.glob(os.path.join(ROOT, 'tests', 'conformance', '*.aak')) + glob.glob(os.path.join(ROOT, 'examples', '*.aak'))
    seeds = [open(f, encoding='utf-8').read() for f in sorted(files)]
    rng = random.Random(args.seed)
    programs = [mutate(rng, rng.choice(seeds)) for _ in range(args.cases)]

    def run(k):
        path = os.path.join(out, f'case{k}.aak')
        with open(path, 'w', encoding='utf-8') as f:
            f.write(programs[k])
        try:
            p = subprocess.run([args.bin, path], stdin=subprocess.DEVNULL, capture_output=True, timeout=args.timeout)
        except subprocess.TimeoutExpired:
            return 'timeout', path, ''
        err = p.stderr.decode('utf-8', 'replace')
        if p.returncode in (0, 1) and 'panicked' not in err:
            os.remove(path)
            return 'ok', path, ''
        return 'crash', path, f'exit code {p.returncode}: {err[-300:]}'

    counts, problems = {}, []
    with cf.ThreadPoolExecutor(os.cpu_count() or 4) as ex:
        for status, path, info in ex.map(run, range(args.cases)):
            counts[status] = counts.get(status, 0) + 1
            if status != 'ok':
                problems.append((status, path, info))

    print(f"{args.cases} programs: {counts.get('ok', 0)} ok, {counts.get('timeout', 0)} timeout, {counts.get('crash', 0)} crash")
    for status, path, info in problems[:20]:
        print(f'  {status}: {path} {info}')
    sys.exit(1 if counts.get('crash') else 0)


if __name__ == '__main__':
    main()
