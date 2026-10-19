#!/usr/bin/env bash
# Exports the Godot reference for the parity tests into engine/parity/out (about 4 s).
#   engine/parity/export.sh            then: cargo test -p emberwake_rules --test parity
# Godot binary: $GODOT, else godot on PATH, else the default Windows install. Records the Godot commit it measured.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
project="$(cd "$here/../../legacy" && pwd)"   # the frozen Godot reference (legacy/LEGACY.md)
GODOT="${GODOT:-$(command -v godot || echo "/c/Program Files/Godot/godot.exe")}"
win() { if command -v cygpath >/dev/null; then cygpath -m "$1"; else echo "$1"; fi; }
mkdir -p "$here/out"
# a fresh checkout has no class cache yet: import the project once
[ -d "$project/.godot" ] || "$GODOT" --headless --path "$project" --import > /dev/null 2>&1 || true
"$GODOT" --headless --path "$project" -s "$(win "$here/godot/export.gd")" -- out="$(win "$here/out")" "$@" 2>&1 | grep -E "^§|ERROR" | sed "s/^§ //"
sed -n 's/^Source commit: //p' "$project/LEGACY.md" > "$here/out/godot_commit.txt"
echo "godot reference: $(cut -c1-12 "$here/out/godot_commit.txt")"
