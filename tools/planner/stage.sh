#!/usr/bin/env bash
# Planner staging helper (desktop; decision "Batched gate", user 2026-10-28): a task lands on the local branch
# `staging` after a PARTIAL gate; flush.sh later runs the full gate once and pushes the batch to develop.
#   bash stage.sh <task worktree> <name>
# 1. rebase the task branch onto `staging` (docs conflicts: resolve_docs.py, then git add + rebase --continue);
# 2. partial gate: type-check every crate and its tests (cargo check --workspace --all-targets: no linking), run the tests of the crates the
#    task touches (dirs under engine/crates, kits/rpg/crates, games/emberwake/crates), the layer and docs checks;
# 3. fast-forward `staging` (checked out in the sibling worktree <repo>-stage) to the task tip.
# Prints "<name> STAGED <hash> <subject>" or "<name> REBASE CONFLICT" / "PARTIAL GATE FAILED" / "DOCS FAILED".
# Remove the task worktree and branch only after a STAGED line. New task worktrees branch from `staging`.
wt="$1"; name="$2"; log="${TMPDIR:-/tmp}/stage_${name}.log"
stage="$(dirname "$wt")/tactical-rpg-stage"
cd "$wt" || exit 1
git rebase -q staging || { echo "$name REBASE CONFLICT"; exit 1; }
bad=$(git diff --name-only staging...HEAD | grep -E '(^|/)target(-[^/]*)?/' | head -3)
[ -z "$bad" ] || { echo "$name BUILD FOLDER COMMITTED: $bad (git rm -r --cached it)"; exit 1; }
if [ -z "${CARGO_TARGET_DIR:-}" ] && [ -f .git ]; then
  base=$(sed -n 's/^target-dir = "\(.*\)"/\1/p' .cargo/config.toml)
  export CARGO_TARGET_DIR="${base}-$(basename "$PWD")"
fi
crates=$(git diff --name-only staging...HEAD | sed -n 's#^\(engine\|kits/rpg\|games/emberwake\)/crates/\([^/]*\)/.*#\2#p' | sort -u)
if git diff --name-only staging...HEAD | grep -q '^games/emberwake/data/'; then  # data changes: the game crates read the data
  crates=$(printf "%s\n" $crates $(ls games/emberwake/crates) | sort -u)
fi
# kit crates: their game façades (emberwake_<x> for rpgkit_<x>) keep the kit's parity tests (2026-11-01: a
# rpgkit_content check change passed the partial gate and failed emberwake_content checks_parity)
for c in $crates; do case $c in rpgkit_*) f=emberwake_${c#rpgkit_}; [ -d games/emberwake/crates/$f ] && crates="$crates $f";; esac; done
crates=$(printf "%s
" $crates | sort -u)
source "$(dirname "${BASH_SOURCE[0]}")/heavy.sh"; heavy_start "stage $name"
t0=$(date +%s)
cargo check --workspace --all-targets --quiet > "$log" 2>&1; rc=$?
if [ $rc -eq 0 ] && [ -n "$crates" ]; then
  args=$(for c in $crates; do printf -- "-p %s " "$c"; done)
  cargo test $args --no-fail-fast >> "$log" 2>&1; rc=$?
fi
[ $rc -eq 0 ] && { bash tools/fc.sh layers >> "$log" 2>&1 || rc=1; }
# art tasks: the rig's own tests (2026-10-29: WINDUP1-R1/R2 staged as "docs only" with no test run)
if [ $rc -eq 0 ] && git diff --name-only staging...HEAD | grep -q '^tools/sprites/'; then
  py="${LOCALAPPDATA:-}/Programs/Python/Python312/python.exe"
  for t in tools/sprites/test_*.py; do
    (cd tools/sprites && "$py" -m unittest -q "$(basename "$t" .py)") >> "$log" 2>&1 || { echo "$name: $t FAILED"; rc=1; }
  done
  crates="${crates:+$crates }tools/sprites"
fi
gate_s=$(( $(date +%s) - t0 ))
tests=$(grep -o "test result: ok. [0-9]* passed" "$log" | awk '{s+=$4} END {print s+0}')
grep -E "^error|FAILED|panicked" "$log" | grep -v "^test result" | head -12 | sed "s/^/$name: /"
echo "$name: partial gate ${gate_s}s, crates: ${crates:-none (docs only)}, ${tests} tests"
[ $rc -eq 0 ] || { echo "$name PARTIAL GATE FAILED"; exit 1; }
bash tools/fc.sh docs > "$log.docs" 2>&1 || { tail -5 "$log.docs"; echo "$name DOCS FAILED"; exit 1; }
git -C "$stage" merge -q --ff-only "$(git rev-parse HEAD)" || { echo "$name STAGING NOT FAST-FORWARD"; exit 1; }
echo "$name STAGED $(git log --oneline -1 | cut -c1-40)"
