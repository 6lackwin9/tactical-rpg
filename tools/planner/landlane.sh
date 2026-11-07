#!/usr/bin/env bash
# landlane.sh <lane worktree> <task>: cherry-pick the lane's task commit onto claude/<task>-land from staging,
# resolve .md conflicts with resolve_docs.py and kits/rpg/schema.json with sch_merge.py, stop on any other conflict,
# then stage.sh; cleanup only on STAGED.
wt="$1"; task="$2"; S="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$wt" || exit 1
c=$(git rev-parse HEAD)
git checkout -q -B "claude/$task-land" staging || exit 1
if ! git cherry-pick "$c" >/dev/null 2>&1; then
  files=$(git diff --name-only --diff-filter=U); echo "conflicts: $files"
  for f in $files; do case $f in *.md) python $S/resolve_docs.py "$f" >/dev/null && git add "$f";;
    kits/rpg/schema.json) t=$(mktemp -d); git show :1:$f > $t/sch_base.json; git show :2:$f > $t/sch_ours.json; git show :3:$f > $t/sch_theirs.json
      python $S/sch_merge.py $t | grep -q "conflicts: \[\]" && cp $t/sch_merged.json $f && git add $f || { echo "SCHEMA CONFLICT"; exit 1; };;
    *) echo "NON-MD CONFLICT $f"; exit 1;; esac; done
  GIT_EDITOR=true git cherry-pick --continue >/dev/null || { echo "RESOLVE FAILED"; exit 1; }
  echo "resolved"
else echo "conflicts: "; fi
out=$(bash $S/stage.sh "$wt" "$task"); echo "$out"
case "$out" in *" STAGED "*) bash $S/cleanup.sh "$(basename "$wt" | sed 's/tactical-rpg-//')";; esac
