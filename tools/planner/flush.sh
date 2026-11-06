#!/usr/bin/env bash
# Planner batch push (desktop; decision "Batched gate", user 2026-10-28): runs the FULL gate once on `staging` and
# pushes the batch of staged tasks to develop.
#   bash flush.sh [repo root, default C:/dev/tactical-rpg]
# Rebases `staging` onto origin/develop (planner docs commits that went straight to develop), runs
# check_project.ps1 -Fast -Quiet and tools/fc.sh docs, then pushes staging to develop (fast-forward).
# Prints "FLUSHED <n> commits <hash>" or "REBASE CONFLICT" / "GATE FAILED" / "DOCS FAILED" / "NOTHING TO FLUSH".
# A failed gate: bisect the batch (git -C <stage> log origin/develop..staging), fix on the culprit's branch.
root="${1:-C:/dev/tactical-rpg}"; stage="$(dirname "$root")/tactical-rpg-stage"; log="${TMPDIR:-/tmp}/flush.log"
cd "$stage" || exit 1
git fetch -q origin
git rebase -q origin/develop || { echo "REBASE CONFLICT"; exit 1; }
n=$(git rev-list --count origin/develop..HEAD) || { echo "GIT FAILED (rev-list), nothing flushed"; exit 1; }
[ -n "$n" ] || { echo "GIT FAILED (no count), nothing flushed"; exit 1; }
[ "$n" -gt 0 ] || { echo "NOTHING TO FLUSH"; exit 0; }
source "$(dirname "${BASH_SOURCE[0]}")/heavy.sh"; heavy_start flush
t0=$(date +%s)
powershell -NoProfile -File check_project.ps1 -Fast -Quiet > "$log" 2>&1; rc=$?
grep -E "FAILED|panicked" "$log" | grep -v "^test result" | head -12
grep -E "=== \[" "$log" | tail -1
gate_s=$(( $(date +%s) - t0 ))
echo "full gate ${gate_s}s for $n commits"
printf "%s	%s	%s	%s
" "$(date '+%Y-%m-%d %H:%M')" "$gate_s" "$n" "$rc" >> "${AEGIS_PLANNER_LOG:-${TEMP:-/tmp}}/gate_times.tsv"
[ $rc -eq 0 ] || { echo "GATE FAILED $rc"; exit 1; }
bash tools/fc.sh docs > "$log.docs" 2>&1 || { tail -5 "$log.docs"; echo "DOCS FAILED"; exit 1; }
# Decision "Efficiency pass": a docs or art commit that reached develop during the gate does not re-run it.
git fetch -q origin
moved=$(git diff --name-only HEAD...origin/develop)
if [ -n "$moved" ]; then
  code=$(echo "$moved" | grep -vE '\.md$|^art_src/' | head -3)
  if [ -n "$code" ]; then echo "PUSH HELD: develop moved with code: $code"; exit 1; fi
  git rebase -q origin/develop || { echo "REBASE CONFLICT"; exit 1; }
  echo "rebased over docs and art only, gate kept"
fi
git push -q origin HEAD:develop || { echo "PUSH FAILED (develop moved: rerun)"; exit 1; }
echo "FLUSHED $n commits $(git log --oneline -1 | cut -c1-40)"
