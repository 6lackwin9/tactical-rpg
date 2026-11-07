#!/usr/bin/env bash
# Builder's partial gate (decision "Builder efficiency", user 2026-10-30): the same checks stage.sh runs, on the
# uncommitted work in a task worktree. The full gate still runs on every flush to develop.
#   bash pgate.sh            (from the task worktree)
# 1. cargo check --workspace --all-targets (every crate and test compiles, no warnings);
# 2. the tests of the crates the task touches (vs its merge-base with `staging`, uncommitted and new files included);
# 3. the tools/sprites tests when tools/sprites changed; 4. the layer check; 5. tools/fc.sh docs.
# Prints "PARTIAL GATE PASSED" or "PARTIAL GATE FAILED: <step>"; the log path is printed at the end.
log="${TMPDIR:-/tmp}/pgate_$(basename "$PWD").log"; : > "$log"
if [ -z "${CARGO_TARGET_DIR:-}" ] && [ -f .git ]; then
  base=$(sed -n 's/^target-dir = "\(.*\)"/\1/p' .cargo/config.toml)
  export CARGO_TARGET_DIR="${base}-$(basename "$PWD")"
fi
mb=$(git merge-base staging HEAD)
files=$( { git diff --name-only "$mb"; git ls-files --others --exclude-standard; } | sort -u)
crates=$(echo "$files" | sed -n 's#^\(engine\|kits/rpg\|games/emberwake\)/crates/\([^/]*\)/.*#\2#p' | sort -u)
if echo "$files" | grep -q '^games/emberwake/data/'; then  # data changes: the game crates' tests read the shipped data
  crates=$(printf "%s\n" $crates $(ls games/emberwake/crates) | sort -u)
fi
# kit crates: their game façades (emberwake_<x> for rpgkit_<x>) keep the kit's parity tests (2026-11-01: a
# rpgkit_content check change passed the partial gate and failed emberwake_content checks_parity)
for c in $crates; do case $c in rpgkit_*) f=emberwake_${c#rpgkit_}; [ -d games/emberwake/crates/$f ] && crates="$crates $f";; esac; done
crates=$(printf "%s
" $crates | sort -u)
fail() { echo "PARTIAL GATE FAILED: $1"; grep -E "^(error|warning)|FAILED|panicked" "$log" | grep -v "^test result" | head -12; echo "log: $log"; exit 1; }
source "$(dirname "${BASH_SOURCE[0]}")/heavy.sh"; heavy_start pgate
t0=$(date +%s)
cargo check --workspace --all-targets --quiet >> "$log" 2>&1 || fail "cargo check"
grep -q "^warning" "$log" && fail "compiler warnings"
if [ -n "$crates" ]; then
  cargo test $(for c in $crates; do printf -- "-p %s " "$c"; done) --no-fail-fast >> "$log" 2>&1 || fail "tests of $crates"
fi
if echo "$files" | grep -q '^tools/sprites/'; then
  py="${LOCALAPPDATA:-}/Programs/Python/Python312/python.exe"
  for t in tools/sprites/test_*.py; do
    (cd tools/sprites && "$py" -m unittest -q "$(basename "$t" .py)") >> "$log" 2>&1 || fail "$t"
  done
fi
bash tools/fc.sh layers >> "$log" 2>&1 || fail "layers"
bash tools/fc.sh docs >> "$log" 2>&1 || fail "docs"
n=$(grep -o "test result: ok. [0-9]* passed" "$log" | awk '{s+=$4} END {print s+0}')
echo "PARTIAL GATE PASSED in $(( $(date +%s) - t0 ))s: crates ${crates:-none} ($n tests); log: $log"
