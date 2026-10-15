#!/usr/bin/env bash
# Overlap check across active worktrees: lists files changed (committed vs main, modified or untracked) in more than one worktree,
# so the planner sees merge conflicts BEFORE merging. Usage: tools/overlap.sh   (run from any checkout of the repo)
# Exit code 1 when at least one file is touched by two or more worktrees.
cd "$(dirname "$0")/.." || exit 1
tmp="$(mktemp)"
count=0
while read -r key value; do
  case "$key" in
    worktree) path="$value" ;;
    branch)
      branch="${value#refs/heads/}"
      [ "$branch" = "main" ] && continue
      count=$((count + 1))
      {
        git -C "$path" diff --name-only main...HEAD 2>/dev/null
        git -C "$path" status --porcelain 2>/dev/null | awk '{print $NF}'
      } | sort -u | while read -r f; do [ -n "$f" ] && echo "$f|$branch"; done >> "$tmp"
      ;;
  esac
done < <(git worktree list --porcelain)
echo "[overlap] active feature worktrees: $count"
dups=$(cut -d'|' -f1 "$tmp" | sort | uniq -d)
if [ -z "$dups" ]; then
  echo "[overlap] no file is touched by more than one worktree"
  rm -f "$tmp"; exit 0
fi
echo "[overlap] files touched by more than one worktree (merge conflicts likely; merge the smaller change first):"
for f in $dups; do
  echo "  $f  <-  $(grep -F "$f|" "$tmp" | cut -d'|' -f2 | tr '\n' ' ')"
done
rm -f "$tmp"
exit 1
