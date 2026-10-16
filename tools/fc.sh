#!/usr/bin/env bash
# Fast check: compact, failure-first output. Usage:
#   tools/fc.sh parse            refresh class cache if .gd files were added, then parse-check changed files
#   tools/fc.sh parse-all        parse EVERY .gd in one process (~seconds; check_project.ps1 launches Godot once per file)
#   tools/fc.sh arena [pattern]  run the arena suite; print totals, FAIL/SCRIPT ERROR lines, and lines matching [pattern]
#   tools/fc.sh dojo [pattern]   run the dojo suite the same way
#   tools/fc.sh run <script.gd>  run a SceneTree script (scratch debugging), noise filtered
#   tools/fc.sh safe_delete <kind> <id> [--apply]   scan, then delete a SYSTEM asset (refuses while referenced)
#   tools/fc.sh docs            warn when an agent-facing doc passes 300 lines / 25 KB
#   tools/fc.sh all              parse + arena + dojo (same as check_project.ps1 without per-file parse of unchanged files)
#   tools/fc.sh sim <duel|tier|levels> [level]   balance simulation table (see tools/balance_sim.gd)
# Exit code 1 when anything failed.
cd "$(dirname "$0")/.." || exit 1
GODOT="${GODOT:-/c/Program Files/Godot/godot.exe}"
OUT="${TMPDIR:-/tmp}/fc/$(pwd | md5sum | cut -c1-8)"   # one log folder per worktree so parallel lanes never overwrite each other
mkdir -p "$OUT"
status=0

noise='^ERROR: .*leaked|^ERROR: Pages in use|^   at: (~|unref)|WARNING: A Thread|Please call wait_to_finish|^Godot Engine v|^$'

parse() {
  if [ -n "$(git status --short | grep -E '^\?\? .*\.gd$|^A .*\.gd$')" ]; then
    echo "[parse] new .gd files: refreshing class cache"
    timeout 150 "$GODOT" --headless --editor --quit-after 50 >/dev/null 2>&1
  fi
  local bad=0
  for f in $(git status --short | grep -E '\.gd$' | grep -v '^ *D' | awk '{print $NF}'); do
    res=$(timeout 60 "$GODOT" --headless --path . --check-only -s "res://$f" 2>&1 | grep -E "SCRIPT ERROR|Parse Error|Compile Error" -A1 | head -6)
    if [ -n "$res" ]; then echo "== $f"; echo "$res"; bad=1; fi
  done
  [ $bad -eq 0 ] && echo "[parse] changed files OK" || status=1
}

suite() {  # $1 = scene, $2 = label, $3 = optional pattern
  timeout 290 "$GODOT" --headless "$1" > "$OUT/$2.out" 2>&1
  local pass fail
  pass=$(grep -c "PASS" "$OUT/$2.out"); fail=$(grep "FAIL" "$OUT/$2.out" | grep -vc ": PASS")
  echo "[$2] PASS $pass  FAIL $fail   (full log: $OUT/$2.out)"
  grep -n "SCRIPT ERROR" -A1 "$OUT/$2.out" | head -20
  grep -n "FAIL" "$OUT/$2.out" | grep -v ": PASS" | head -30
  [ -n "$3" ] && grep -n -- "$3" "$OUT/$2.out" | head -30
  [ "$fail" -gt 0 ] && status=1
  grep -q "SCRIPT ERROR" "$OUT/$2.out" && status=1
}

docs_check() {  # warn when an agent-facing doc grows past ~300 lines or ~25 KB (split by domain behind an index)
  local f lines bytes warned=0
  for f in CLAUDE.md docs/*.md docs/specs/*.md; do
    [ -f "$f" ] || continue
    lines=$(wc -l < "$f"); bytes=$(wc -c < "$f")
    if [ "$lines" -gt 300 ] || [ "$bytes" -gt 25000 ]; then echo "[docs] WARN $f is $lines lines / $bytes bytes: split by domain behind an index"; warned=1; fi
  done
  [ $warned -eq 0 ] && echo "[docs] agent-facing docs within size limits (<=300 lines, <=25 KB)"
}

case "$1" in
  parse) parse ;;
  parse-all) res=$(timeout 300 "$GODOT" --headless --path . -s tools/parse_all.gd 2>&1 | grep -E "^§|SCRIPT ERROR|Parse Error|Compile Error" -A1 | sed "s/^§ //"); echo "$res"; echo "$res" | grep -Eq "SCRIPT ERROR|Parse Error|Compile Error|FAIL" && status=1 ;;
  arena) suite res://scenes/test_arena.tscn arena "$2" ;;
  dojo) suite res://scenes/gear_dojo.tscn dojo "$2" ;;
  run) timeout 120 "$GODOT" --headless --path . -s "$2" 2>&1 | grep -Ev "$noise" ;;
  sim) timeout 900 "$GODOT" --headless --path . -s tools/balance_sim.gd -- "${@:2}" 2>&1 | grep "^§" | sed "s/^§ //" ;;
  docs) docs_check ;;
  safe_delete) timeout 120 "$GODOT" --headless --path . -s tools/safe_delete.gd -- "${@:2}" 2>&1 | grep "^§" | sed "s/^§ //"; status=${PIPESTATUS[0]} ;;
  all) parse; docs_check; suite res://scenes/test_arena.tscn arena; suite res://scenes/gear_dojo.tscn dojo ;;
  *) sed -n '2,/^# Exit code/p' "$0" ;;   # the whole usage block, however many lines it grows to
esac
exit $status
