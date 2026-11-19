#!/usr/bin/env bash
# Time each benchmark: ./benchmarks/run.sh [path/to/adarsh]
BIN=${1:-adarsh}
cd "$(dirname "$0")"
for f in *.aak; do
    start=$(date +%s.%N)
    out=$("$BIN" "$f" 2>&1 | tr '\n' ' ')
    end=$(date +%s.%N)
    printf '%-10s %6.2fs  %s\n' "${f%.aak}" "$(echo "$end - $start" | bc)" "$out"
done
