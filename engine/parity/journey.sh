#!/usr/bin/env bash
# Records seeded random game-loop journeys from the Godot reference into engine/parity/out/journeys.json (journey.gd;
# nothing is saved). Then: cargo test -p aegis_world --test journey
#   engine/parity/journey.sh [seeds=1,2] [steps=300]
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
project="$(cd "$here/../.." && pwd)"
GODOT="${GODOT:-$(command -v godot || echo "/c/Program Files/Godot/godot.exe")}"
win() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
mkdir -p "$here/out"
"$GODOT" --headless --path "$project" -s "$(win "$here/godot/journey.gd")" -- out="$(win "$here/out")" "$@" 2>&1 | grep -E "^§|ERROR|SCRIPT" | sed "s/^§ //"
