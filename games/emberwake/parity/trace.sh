#!/usr/bin/env bash
# Records Godot balance-sim fights step by step into games/emberwake/parity/out/traces_<mode>_l<level>.json (trace.gd; the shipped catalog is
# only read, sim units live in a sandbox fork). Then: cargo test -p emberwake_battle --test trace_parity
#   games/emberwake/parity/trace.sh mode=duel level=5 seeds=2 [classes=warrior,mage] [preset=balanced] [jitter=on|off]
#   games/emberwake/parity/trace.sh mode=duel level=5 pouch=con_heal_potion,con_fire_bomb   (battle items: traces_duel_l5_items.json)
#   games/emberwake/parity/trace.sh mode=world seeds=2 preset=withdraw   (world battles: traces_world_withdraw.json)
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
project="$(cd "$here/../../../legacy" && pwd)"   # the frozen Godot reference (legacy/LEGACY.md)
GODOT="${GODOT:-$(command -v godot || echo "/c/Program Files/Godot/godot.exe")}"
win() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
mkdir -p "$here/out"
# a fresh checkout has no class cache yet: import the project once
[ -d "$project/.godot" ] || "$GODOT" --headless --path "$project" --import > /dev/null 2>&1 || true
"$GODOT" --headless --path "$project" -s "$(win "$here/godot/trace.gd")" -- out="$(win "$here/out")" "$@" 2>&1 | grep -E "^§|ERROR" | sed "s/^§ //"
