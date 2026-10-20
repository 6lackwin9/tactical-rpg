#!/usr/bin/env bash
# Records what the Godot content checks report (games/emberwake/parity/godot/checks.gd) into
# games/emberwake/parity/out/checks.json: validate_all / warnings_all of the catalog and the worlds, and the errors and
# warnings of generated broken entries. Then: cargo test -p emberwake_content --test checks_parity.
#   games/emberwake/parity/checks.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
project="$(cd "$here/../../../legacy" && pwd)"   # the frozen Godot reference (legacy/LEGACY.md)
GODOT="${GODOT:-$(command -v godot || echo "/c/Program Files/Godot/godot.exe")}"
mkdir -p "$here/out"
[ -d "$project/.godot" ] || "$GODOT" --headless --path "$project" --import > /dev/null 2>&1 || true
"$GODOT" --headless --path "$project" -s "$here/godot/checks.gd" -- "out=$here/out" 2>&1 | grep -E "^§|SCRIPT ERROR|ERROR:" || true
test -s "$here/out/checks.json"
