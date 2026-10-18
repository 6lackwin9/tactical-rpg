#!/usr/bin/env bash
# Records Godot balance-sim duels step by step into engine/parity/out/traces.json (trace.gd; the shipped catalog is
# only read, sim units live in a sandbox fork). Then: cargo test -p aegis_battle --test trace_parity
#   engine/parity/trace.sh level=5 seeds=2 [classes=warrior,mage] [preset=balanced] [jitter=on|off]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
project="$(cd "$here/../.." && pwd)"
GODOT="${GODOT:-$(command -v godot || echo "/c/Program Files/Godot/godot.exe")}"
win() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
mkdir -p "$here/out"
"$GODOT" --headless --path "$project" -s "$(win "$here/godot/trace.gd")" -- out="$(win "$here/out")" "$@" 2>&1 | grep -E "^§|ERROR" | sed "s/^§ //"
