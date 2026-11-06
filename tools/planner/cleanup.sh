#!/usr/bin/env bash
# Planner cleanup after a task is STAGED (decision "Batched gate"; skill task-landing section 3):
#   bash cleanup.sh <name> [<name> ...]
# A lane (lane1..lane3, decision "Builder efficiency" 2026-10-30): the worktree and its warm build cache stay; the lane is
# detached and its task branches (claude/<task>, claude/<task>-land) are deleted, ready for lane.sh.
# Any other name (a one-off task worktree): removes the worktree C:/dev/tactical-rpg-<name>, its branches and its build
# cache %TEMP%/aegis-target-tactical-rpg-<name> (2026-10-29: caches left by removed worktrees filled drive C:).
# Run it only after a STAGED line, never chained after stage.sh.
repo=C:/dev/tactical-rpg
tmp="${TEMP:-$LOCALAPPDATA/Temp}"
for name in "$@"; do
  case "$name" in stage|dev|plan|"") echo "refusing standing worktree '$name'"; continue;; esac
  wt="$repo-$name"
  if [[ "$name" =~ ^lane[1-3]$ ]]; then
    b=$(git -C "$wt" branch --show-current)
    [ -z "$(git -C "$wt" status --porcelain)" ] || { echo "$name NOT CLEAN, left as is"; continue; }
    git -C "$wt" checkout -q --detach && echo "$name: detached (cache kept)"
    for x in "$b" "${b%-land}" "$b-land"; do
      [ -n "$x" ] && git -C "$repo" rev-parse -q --verify "refs/heads/$x" >/dev/null && git -C "$repo" branch -D -q "$x" && echo "$name: branch $x deleted"
    done
    continue
  fi
  [ -d "$wt" ] && git -C "$repo" worktree remove --force "$wt" && echo "$name: worktree removed"
  for b in "claude/$name" "claude/$name-land"; do
    git -C "$repo" rev-parse -q --verify "refs/heads/$b" >/dev/null && git -C "$repo" branch -D -q "$b" && echo "$name: branch $b deleted"
  done
  cache="$tmp/aegis-target-tactical-rpg-$name"
  [ -d "$cache" ] && rm -rf "$cache" && echo "$name: build cache removed"
done
df -h /c | tail -1
