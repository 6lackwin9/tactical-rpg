#!/usr/bin/env bash
# Records the Godot balance sim (legacy/tools/balance_sim.gd) for a set of command lines into
# games/emberwake/parity/out/sim/<n>.txt: the first line is the command line, the rest what the sim printed (its "§"
# lines). Then: cargo test -p emberwake_sim --test sim_parity (the engine's sim prints the same lines).
#   games/emberwake/parity/sim.sh            every run of RUNS below
#   games/emberwake/parity/sim.sh 3 7        only runs 3 and 7
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
project="$(cd "$here/../../../legacy" && pwd)"   # the frozen Godot reference (legacy/LEGACY.md)
GODOT="${GODOT:-$(command -v godot || echo "/c/Program Files/Godot/godot.exe")}"
mkdir -p "$here/out/sim"
[ -d "$project/.godot" ] || "$GODOT" --headless --path "$project" --import > /dev/null 2>&1 || true
# One command line per run: every mode, the keys that change what a mode prints, and two argument errors.
RUNS=(
  "duel 3 1"
  "duel 5 1 preset=both score=hp band=40-60"
  "duel 5 1 metrics=on loadout=role distance=3 spend=mixed"
  "tier 5 1"
  "tier 3 1 tier_foe=warrior quick"
  "levels 5 1 score=binary"
  "group 5 1 jitter=off"
  "stats all spend=all"
  "stats 5 loadout=newest"
  "party 5 2 enc=mob,line,wave,elite,boss"
  "party 3 1 comp=warrior,priest enc=world:ash_patrol,ash_raiders elevel=+1 pouch=con_heal_potion"
  "party 5 1 comp=warrior,ranger,mage enc=all metrics=off"
  "sweep 3 1 size=2 enc=line metrics=off"
  "sweep 5 1 size=3 lead=warrior enc=mob,boss phase=off"
  "role 5 quick"
  "role 7 1 role=support class=priest"
  "campaign 5 1 comp=warrior,priest,mage heal=rally"
  "campaign 3 1 comp=warrior,ranger,rogue heal=25 rest=3 extra=4:ash_raiders"
  "duel 5 1 bad=1"
  "party 5 1 comp=warrior,knight"
)
pick=("$@")
for i in "${!RUNS[@]}"; do
  n=$((i + 1))
  if [ ${#pick[@]} -gt 0 ] && [[ ! " ${pick[*]} " =~ " $n " ]]; then continue; fi
  run="${RUNS[$i]}"
  # shellcheck disable=SC2086
  { echo "$run"; (cd "$project" && "$GODOT" --headless --path . -s tools/balance_sim.gd -- $run 2>&1) | grep -E "^§" | sed "s/^§ //"; } > "$here/out/sim/$n.txt"
  echo "run $n: $run ($(($(wc -l < "$here/out/sim/$n.txt") - 1)) lines)"
done
