#!/usr/bin/env bash
# Planner landing helper (desktop): rebase a task worktree onto origin/develop, run the gate and the docs check, then
# push the branch tip to develop (fast-forward), retrying the push after a fetch + rebase when develop moved.
#   bash land.sh <worktree path> <name>
# Prints "<name>: <gate summary>" and "<name> PUSHED <hash> <subject>", or "<name> REBASE CONFLICT" / "GATE FAILED" / "DOCS FAILED".
# Remove the worktree and branch ONLY after reading a PUSHED line (2026-10-25: a cleanup after a conflict deleted the
# branch mid-rebase; the commit was recovered from its hash). Never mask this script's exit code in a wrapper.
# Docs-only rebase conflicts: python resolve_docs.py <file.md> ..., then git add + GIT_EDITOR=true git rebase --continue.
wt="$1"; name="$2"; log="${TMPDIR:-/tmp}/land_${name}.log"
cd "$wt" || exit 1
git fetch -q origin && git rebase -q origin/develop || { echo "$name REBASE CONFLICT"; exit 1; }
t0=$(date +%s); powershell -NoProfile -File check_project.ps1 -Fast -Quiet > "$log" 2>&1; rc=$?; gate_s=$(( $(date +%s) - t0 ))
grep -E "FAILED|panicked" "$log" | grep -v "^test result" | head -12 | sed "s/^/$name: /"
grep -E "=== \[" "$log" | tail -1 | sed "s/^/$name: /"
echo "$name: gate ${gate_s}s (other builders running: $(tasklist //FI "IMAGENAME eq cargo.exe" 2>/dev/null | grep -c cargo))"
[ $rc -eq 0 ] || { echo "$name GATE FAILED $rc"; exit 1; }
bash tools/fc.sh docs > "$log.docs" 2>&1 || { tail -5 "$log.docs"; echo "$name DOCS FAILED"; exit 1; }
for i in 1 2 3; do
  if git push -q origin HEAD:develop; then echo "$name PUSHED $(git log --oneline -1 | cut -c1-40)"; exit 0; fi
  git fetch -q origin && git rebase -q origin/develop || { echo "$name REBASE CONFLICT (retry)"; exit 1; }
done
echo "$name PUSH FAILED"; exit 1
