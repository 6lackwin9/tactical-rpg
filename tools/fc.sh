#!/usr/bin/env bash
# Fast check: compact, failure-first output. Usage:
#   (every Godot command except parse-all/docs/gate first runs godot --headless --import when the class cache is missing or older than any .gd; one import under flock)
#   tools/fc.sh parse            parse-check changed files
#   tools/fc.sh parse-all        parse EVERY .gd in one process (~seconds; check_project.ps1 launches Godot once per file)
#   tools/fc.sh arena [pattern]  run the arena suite; print totals, FAIL/SCRIPT ERROR lines, and lines matching [pattern]
#   tools/fc.sh dojo [pattern]   run the dojo suite the same way
#   tools/fc.sh run <script.gd>  run a SceneTree script (scratch debugging), noise filtered
#   tools/fc.sh safe_delete <kind> <id> [--apply]   scan, then delete a SYSTEM asset (refuses while referenced)
#   tools/fc.sh export_placeholder <ns> <key> <variant> [team]   write a generated placeholder as PNG to its assets/ path (S14)
#   tools/fc.sh replay <file|id> [--battle=K] [--json] [--variant=<patch> --then=stop|recorded --write=<out>]
#        [--seeds=<a,b|a..b> --fallback=stop|<preset>] [--trail-report]   replay a playtest recording (tools/replay.gd, PLAYTEST1;
#        exit 0 match, 1 divergence or broken plan, 2 load or validation error)
#   tools/fc.sh gen_sfx <key> <wave> <hz_from> <hz_to> <ms> <envelope> <gain_db>   write a placeholder blip to assets/audio/sfx/<key>.wav (S14-L3)
#   tools/fc.sh docs            warn when an agent-facing doc passes 300 lines / 25 KB
#   tools/fc.sh all              parse + arena + dojo (same as check_project.ps1 without per-file parse of unchanged files)
#   tools/fc.sh gate             the commit gate, same steps as check_project.ps1 -Fast -Quiet (class cache, parse-all, arena, dojo); no PowerShell needed
#   Godot binary: $GODOT, else godot / godot4 on PATH, else the default Windows install
#   tools/fc.sh sim <mode> [level] [seeds] [key=value ...]   balance simulation (tools/balance_sim.gd, BAL2_SIM.md;
#        timeout ${SIM_TIMEOUT:-900} s, prints TIMEOUT and exits 1 on expiry; startup watchdog ${SIM_STARTUP_TIMEOUT:-120} s without output; script errors are shown)
#     modes: duel [level] | tier [level] | levels | group [level] | stats [level|all] | party [level] | role [level] | sweep [level]
#     keys:  cap=1..30 (10)  loadout=default|newest|role  spend=class|cons|primary|mixed|all
#            preset=<PlanPresets id>|both  distance=default|2..6 (role: 4)  score=both|hp|binary  quick  metrics=on|off  band=lo-hi (30-70)
#            comp=c1,..(<=4)  enc=<ids>|generated|all|world:[<world>/]<id>  elevel=+n|-n|catalog  elite=<class>  boss=<class>
#            role=tank|ranged|aoe|support|striker|all  class=<id>|all  dupes=on|off  sample=<n>  target=<key>:<lo>-<hi>  jitter=on|off (on)
#            world=<id> (bare enc= ids find that world's encounters)  size=<n> lead=<class> (sweep; size=3 lead=warrior = the 10 trios)
#            tier_foe=mirror|<class>  phase=on|off (generated boss phase, on)  s2=on|off (off); party / sweep default 10 seeds
#     jitter refusals on an unreachable target print PLAN SKIP and are skipped fights, not PLAN FAIL (BAL5-SIM K6)
#     full role/sweep runs take ~3 s per party fight: raise SIM_TIMEOUT or split by role= / level (one process each)
# Exit code 1 when anything failed.
cd "$(dirname "$0")/.." || exit 1
GODOT="${GODOT:-$(command -v godot || command -v godot4 || echo "/c/Program Files/Godot/godot.exe")}"
OUT="${TMPDIR:-/tmp}/fc/$(pwd | md5sum | cut -c1-8)"   # one log folder per worktree so parallel lanes never overwrite each other
mkdir -p "$OUT"
status=0

noise='^ERROR: .*leaked|^ERROR: Pages in use|^   at: (~|unref)|WARNING: A Thread|Please call wait_to_finish|^Godot Engine v|^$'

import_stale() {  # class cache missing, or any .gd newer than it
  local cache=.godot/global_script_class_cache.cfg
  [ ! -f "$cache" ] || [ -n "$(find scripts tools scenes data/seeds -name '*.gd' -newer "$cache" 2>/dev/null | head -1)" ]
}

do_import() {
  echo "[import] class cache stale: running godot --headless --import"
  timeout 300 "$GODOT" --headless --import > "$OUT/import.out" 2>&1
  local code=$?
  if [ $code -ne 0 ]; then echo "[import] FAIL (exit $code, full log: $OUT/import.out)"; return 1; fi
  touch .godot/global_script_class_cache.cfg
}

ensure_import() {  # one import for parallel runs: later callers wait on the lock, re-check, skip
  import_stale || return 0
  if command -v flock >/dev/null 2>&1; then
    ( flock 9; import_stale || exit 0; do_import ) 9>"$OUT/import.lock"
  else
    do_import
  fi || { status=1; return 1; }
}

godot_exit() {  # $1 = godot pipeline exit code, $2 = label, $3 = timeout seconds
  if [ "$1" -eq 124 ]; then echo "[$2] TIMEOUT after $3 s (raise SIM_TIMEOUT / REPLAY_TIMEOUT or split the run)"; status=1
  elif [ "$1" -ne 0 ]; then status=$1; fi  # keeps replay exit 2
}

sim() {  # raw output and watchdog flag are per process ($$), since parallel sims share $OUT
  local tmo=${SIM_TIMEOUT:-900} stmo=${SIM_STARTUP_TIMEOUT:-120} raw="$OUT/sim.$$.raw" flag="$OUT/sim.$$.watchdog" pid wd code
  rm -f "$flag"; : > "$raw"
  timeout "$tmo" "$GODOT" --headless --path . -s tools/balance_sim.gd -- "${@:1}" > "$raw" 2>&1 &
  pid=$!
  ( sleep "$stmo"; if ! grep -q '^§' "$raw" && kill -0 "$pid" 2>/dev/null; then touch "$flag"; kill "$pid" 2>/dev/null; fi ) &
  wd=$!
  wait "$pid"; code=$?
  kill "$wd" 2>/dev/null; wait "$wd" 2>/dev/null
  grep -E "^§|^SCRIPT ERROR|^   at: res://" "$raw" | sed "s/^§ //"
  if [ -f "$flag" ]; then echo "[sim] no output after $stmo s: likely a stale class cache or a script error; run tools/fc.sh parse-all"; status=1; rm -f "$flag" "$raw"; return; fi
  rm -f "$raw"
  godot_exit "$code" sim "$tmo"
}

parse() {
  local bad=0
  for f in $(git status --short -uall | grep -E '\.gd$' | grep -v '^ *D' | awk '{print $NF}'); do
    res=$(timeout 60 "$GODOT" --headless --path . --check-only -s "res://$f" 2>&1 | grep -E "SCRIPT ERROR|Parse Error|Compile Error" -A1 | head -6)
    if [ -n "$res" ]; then echo "== $f"; echo "$res"; bad=1; fi
  done
  [ $bad -eq 0 ] && echo "[parse] changed files OK" || status=1
}

suite() {  # $1 = scene, $2 = label, $3 = optional pattern
  timeout "${SUITE_TIMEOUT:-290}" "$GODOT" --headless "$1" > "$OUT/$2.out" 2>&1   # SUITE_TIMEOUT: slow containers (arena ~5.5 min on the cloud box 2026-10-17)
  local code=$? pass fail
  pass=$(grep -c "PASS" "$OUT/$2.out"); fail=$(grep "FAIL" "$OUT/$2.out" | grep -vc ": PASS")
  echo "[$2] PASS $pass  FAIL $fail   (full log: $OUT/$2.out)"
  [ $code -ne 0 ] && { echo "[$2] FAIL Godot exit code $code"; status=1; }
  grep -n "SCRIPT ERROR" -A1 "$OUT/$2.out" | head -20
  grep -n "FAIL" "$OUT/$2.out" | grep -v ": PASS" | head -30
  [ -n "$3" ] && grep -n -- "$3" "$OUT/$2.out" | head -30
  [ "$fail" -gt 0 ] && status=1
  grep -q "SCRIPT ERROR" "$OUT/$2.out" && status=1
}

docs_check() {  # warn when an agent-facing doc grows past ~300 lines or ~25 KB (split by domain behind an index)
  local f lines bytes warned=0
  for f in CLAUDE.md docs/*.md docs/workflow/*.md docs/specs/*.md; do
    [ -f "$f" ] || continue
    lines=$(wc -l < "$f"); bytes=$(wc -c < "$f")
    if [ "$lines" -gt 300 ] || [ "$bytes" -gt 25000 ]; then echo "[docs] WARN $f is $lines lines / $bytes bytes: split by domain behind an index"; warned=1; fi
  done
  [ $warned -eq 0 ] && echo "[docs] agent-facing docs within size limits (<=300 lines, <=25 KB)"
  # markdownlint subset (MD012/MD022/MD032/MD033) so lint warnings show up before the IDE panel does
  local md=() out
  for f in CLAUDE.md README.md docs/*.md docs/workflow/*.md docs/specs/*.md docs/decisions/*.md data/seeds/*.md .claude/agents/*.md .claude/skills/*/SKILL.md; do [ -f "$f" ] && md+=("$f"); done
  if out=$(awk -f tools/md_lint.awk "${md[@]}"); then echo "[docs] markdown lint clean (${#md[@]} files)"; else echo "$out" | head -20; echo "[docs] FAIL markdown lint: $(echo "$out" | wc -l) issue(s)"; return 1; fi
}

parse_all() {
  local res
  res=$(timeout 300 "$GODOT" --headless --path . -s tools/parse_all.gd 2>&1 | grep -E "^§|SCRIPT ERROR|Parse Error|Compile Error" -A1 | sed "s/^§ //")
  echo "$res"
  echo "$res" | grep -Eq "SCRIPT ERROR|Parse Error|Compile Error|FAIL" && status=1
}

gate() {  # check_project.ps1 -Fast -Quiet: stops at the first failing step
  if [ ! -x "$GODOT" ] && ! command -v "$GODOT" >/dev/null; then echo "[gate] FAIL Godot not found: set GODOT or put godot / godot4 on PATH"; status=1; return; fi
  if ! timeout 300 "$GODOT" --headless --import > "$OUT/editor.out" 2>&1; then
    echo "[gate] FAIL class cache / import scan (full log: $OUT/editor.out)"; status=1; return
  fi
  echo "[gate] class cache and imports up to date"
  parse_all; [ $status -ne 0 ] && return
  suite res://scenes/test_arena.tscn arena; [ $status -ne 0 ] && return
  suite res://scenes/gear_dojo.tscn dojo; [ $status -ne 0 ] && return
  echo "[gate] ALL CHECKS PASSED"
}

case "$1" in
  run|sim|replay|safe_delete|export_placeholder|gen_sfx|arena|dojo|all|parse)
    ensure_import || exit 1 ;;
esac

case "$1" in
  parse) parse ;;
  parse-all) parse_all ;;
  arena) suite res://scenes/test_arena.tscn arena "$2" ;;
  dojo) suite res://scenes/gear_dojo.tscn dojo "$2" ;;
  run) timeout 120 "$GODOT" --headless --path . -s "$2" 2>&1 | grep -Ev "$noise"; godot_exit "${PIPESTATUS[0]}" run 120 ;;
  sim) sim "${@:2}" ;;
  docs) docs_check ;;
  safe_delete) timeout 120 "$GODOT" --headless --path . -s tools/safe_delete.gd -- "${@:2}" 2>&1 | grep "^§" | sed "s/^§ //"; godot_exit "${PIPESTATUS[0]}" "$1" 120 ;;
  gate) gate ;;
  replay) timeout "${REPLAY_TIMEOUT:-600}" "$GODOT" --headless --path . -s tools/replay.gd -- "${@:2}" 2>&1 | grep -E "^§|^SCRIPT ERROR|^   at: res://" | sed "s/^§ //"; godot_exit "${PIPESTATUS[0]}" "$1" "${REPLAY_TIMEOUT:-600}" ;;
  export_placeholder) timeout 120 "$GODOT" --headless --path . -s tools/export_placeholder.gd -- "${@:2}" 2>&1 | grep "^§" | sed "s/^§ //"; godot_exit "${PIPESTATUS[0]}" "$1" 120 ;;
  gen_sfx) timeout 120 "$GODOT" --headless --path . -s tools/gen_placeholder_sfx.gd -- "${@:2}" 2>&1 | grep "^§" | sed "s/^§ //"; godot_exit "${PIPESTATUS[0]}" "$1" 120 ;;
  all) parse; docs_check; suite res://scenes/test_arena.tscn arena; suite res://scenes/gear_dojo.tscn dojo ;;
  *) sed -n '2,/^# Exit code/p' "$0" ;;   # the whole usage block, however many lines it grows to
esac
exit $status
