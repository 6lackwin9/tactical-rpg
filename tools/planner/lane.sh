#!/usr/bin/env bash
# Planner: start a task in a persistent lane worktree (decision "Builder efficiency", user 2026-10-30).
#   bash lane.sh <lane 1-3> <task>
# The lane worktree C:/dev/tactical-rpg-lane<n> keeps its path, so its build cache
# (%TEMP%/aegis-target-tactical-rpg-lane<n>) stays warm and builds stay incremental (cargo keys workspace crates to the
# source path). The task gets a fresh branch claude/<task> from `staging`. The lane must be clean (the previous task
# landed and was released with cleanup.sh). Prints the worktree path.
lane="lane$1"; task="$2"; repo=C:/dev/tactical-rpg; wt="$repo-$lane"
[ -n "$task" ] && [[ "$1" =~ ^[1-3]$ ]] || { echo "usage: lane.sh <1-3> <task>"; exit 1; }
if [ ! -d "$wt" ]; then
  git -C "$repo" worktree add -q --detach "$wt" staging || exit 1
fi
cd "$wt" || exit 1
[ -z "$(git status --porcelain)" ] || { echo "$lane NOT CLEAN: land or park its task first"; git status --short | head -5; exit 1; }
cur=$(git branch --show-current)
[ -z "$cur" ] || { echo "$lane still on $cur: run cleanup.sh $lane first"; exit 1; }
git checkout -q -B "claude/$task" staging || exit 1
rm -f .agent/progress.md
echo "$wt (branch claude/$task from $(git log --oneline -1 | cut -c1-60))"
