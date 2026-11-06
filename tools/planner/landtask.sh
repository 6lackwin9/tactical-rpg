#!/usr/bin/env bash
# landtask.sh <lane n> <task> <message file>: commit everything changed in the lane with the message file, land it on staging, print the result.
# Decision "Efficiency pass" (6). The planner still reviews first. The message file holds the subject, Review:, Gate: and trailer lines.
n="$1"; task="$2"; msg="$3"; S="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; wt="C:/dev/tactical-rpg-lane$n"
cd "$wt" || exit 1
git add -A && git commit -q -F "$msg" || { echo "COMMIT FAILED"; exit 1; }
git log --oneline -1 | cut -c1-70
bash "$S/landlane.sh" "$wt" "$task"
