#!/usr/bin/env bash
# Fast check for the Aegis engine (transition step 2, 2026-10-19): compact, failure-first output. Usage:
#   tools/fc.sh gate [auto]      the commit gate, same steps as check_project.ps1 -Fast -Quiet: build every crate, then every test
#                                (parity suites skip themselves when games/emberwake/parity/out is missing); `auto` is accepted and runs everything
#   tools/fc.sh build            cargo build --workspace --all-targets (a compiler warning fails it)
#   tools/fc.sh test [filter]    cargo test --workspace, optional test-name filter
#   tools/fc.sh clippy           cargo clippy --workspace --all-targets (advisory: warnings do not fail)
#   tools/fc.sh docs             warn when an agent-facing doc passes 300 lines / 25 KB; markdown lint
#   tools/fc.sh content <args>   edit games/emberwake/data/ through the engine's writer: check | upsert | remove | apply | manifest (aegis content)
#   tools/fc.sh sim <args>       balance sim (aegis sim <mode> [level] [seeds] [key=value ...]; sim help: the keys)
#   tools/fc.sh replay <file> [...] replay a playtest recording (aegis replay: battle=, until=, json, variant=, write=, seeds=, fallback=)
#   tools/fc.sh parity           re-record the Godot reference from legacy/ (export, traces, journeys, sim runs, content checks; needs Godot 4.7.2)
# The Godot-only commands (parse, parse-all, arena, dojo, run, safe_delete, export_placeholder, gen_sfx, systems) live on
# in legacy/tools/fc.sh for the frozen Godot copy (legacy/LEGACY.md).
# Build output: .cargo/config.toml (desktop) or $CARGO_TARGET_DIR. Exit code 1 when anything failed.
cd "$(dirname "$0")/.." || exit 1
status=0
# A linked worktree (.git is a file) builds into its own folder, so parallel lanes never share cargo's build lock or
# overwrite each other's binaries (.cargo/config.toml names one folder for every checkout).
if [ -z "${CARGO_TARGET_DIR:-}" ] && [ -f .git ]; then
  base=$(sed -n 's/^target-dir = "\(.*\)"/\1/p' .cargo/config.toml)
  export CARGO_TARGET_DIR="${base}-$(basename "$PWD")"
fi

# Windows Smart App Control sometimes blocks a freshly linked test binary (os error 4551): delete the blocked exe so the next
# run relinks it, up to 3 tries (engine/docs/PORT_NOTES.md).
cargo_test() {
  local out try
  for try in 1 2 3 4; do
    out=$(cargo test --workspace --no-fail-fast "$@" 2>&1)
    if echo "$out" | grep -q "os error 4551"; then
      echo "$out" | grep -o 'could not execute process `[^`]*\.exe`' | cut -d'`' -f2 | sort -u | while read -r exe; do rm -f "$exe"; done
      echo "[test] Smart App Control blocked a test binary; relinking (try $try)"
      continue
    fi
    break
  done
  echo "$out" | grep -E "^test result|FAILED|panicked|^error|match Godot|checked|diverge|catalog rows|compared|os error" | grep -v " 0 passed; 0 failed"
  echo "$out" | grep -qE "test result: FAILED|^error|os error 4551" && status=1
}

build() {  # every target (tests included); a compiler warning fails it (cargo replays cached warnings, CI1)
  local out
  if out=$(cargo build --workspace --all-targets 2>&1); then
    if echo "$out" | grep -qE "^warning"; then
      echo "$out" | grep -E "^warning" -A6 | head -60
      echo "[build] FAIL: compiler warnings"; status=1
    else
      echo "[build] ok"
    fi
  else
    echo "$out" | grep -E "^(error|warning)" -A8 | head -60
    echo "[build] FAIL"; status=1
  fi
}

docs_check() {  # warn when an agent-facing doc grows past ~300 lines or ~25 KB (split by domain behind an index)
  local f lines bytes warned=0
  for f in CLAUDE.md docs/*.md docs/workflow/*.md docs/specs/*.md games/*/docs/*.md games/*/docs/specs/*.md; do
    [ -f "$f" ] || continue
    lines=$(wc -l < "$f"); bytes=$(wc -c < "$f")
    if [ "$lines" -gt 300 ] || [ "$bytes" -gt 25000 ]; then echo "[docs] WARN $f is $lines lines / $bytes bytes: split by domain behind an index"; warned=1; fi
  done
  [ $warned -eq 0 ] && echo "[docs] agent-facing docs within size limits (<=300 lines, <=25 KB)"
  # markdownlint subset (MD012/MD022/MD032/MD033) so lint warnings show up before the IDE panel does
  local md=() out
  for f in CLAUDE.md README.md PATCH_NOTES.md docs/*.md docs/workflow/*.md docs/specs/*.md docs/decisions/*.md games/*/docs/*.md games/*/docs/*/*.md games/*/data/*/README.md \
    .claude/agents/*.md .claude/skills/*/SKILL.md engine/README.md engine/docs/*.md legacy/LEGACY.md; do [ -f "$f" ] && md+=("$f"); done
  if out=$(awk -f tools/md_lint.awk "${md[@]}"); then echo "[docs] markdown lint clean (${#md[@]} files)"; else echo "$out" | head -20; echo "[docs] FAIL markdown lint: $(echo "$out" | wc -l) issue(s)"; return 1; fi
}

gate() {
  build; [ $status -ne 0 ] && return
  cargo_test; [ $status -ne 0 ] && return
  echo "[gate] ALL CHECKS PASSED"
}

aegis() {  # the CLI, built on demand
  cargo run -q -p emberwake_host -- "$@"
}

case "$1" in
  gate) gate ;;
  build) build ;;
  test) cargo_test "${@:2}" ;;
  clippy) cargo clippy --workspace --all-targets 2>&1 | grep -E "^(warning|error)" -A4 | head -80 ;;
  docs) docs_check || status=1 ;;
  content) aegis content "${@:2}" || status=1 ;;
  sim) aegis sim "${@:2}" || status=1 ;;
  replay) aegis replay "${@:2}" || status=1 ;;
  parity)
    games/emberwake/parity/export.sh || status=1
    for m in duel; do for l in 1 5 10; do games/emberwake/parity/trace.sh mode=$m level=$l || status=1; done; done
    games/emberwake/parity/trace.sh mode=group level=3 && games/emberwake/parity/trace.sh mode=group level=7 || status=1
    games/emberwake/parity/trace.sh mode=teams level=3 || status=1
    games/emberwake/parity/trace.sh mode=teams level=5 seeds=3 || status=1   # the committed teams_l5 and encounters traces use 3 seeds
    games/emberwake/parity/trace.sh mode=teams level=7 || status=1
    games/emberwake/parity/trace.sh mode=encounters level=1 seeds=3 || status=1
    games/emberwake/parity/trace.sh mode=duel level=5 pouch=con_heal_potion,con_fire_bomb,con_focus_elixir || status=1
    games/emberwake/parity/trace.sh mode=teams level=5 seeds=3 pouch=con_heal_potion,con_fire_bomb,con_heal_potion || status=1
    games/emberwake/parity/trace.sh mode=group level=7 pouch=con_heal_potion,con_heal_potion,con_fire_bomb || status=1
    for p in balanced aggressive withdraw defensive; do games/emberwake/parity/trace.sh mode=world seeds=2 preset=$p || status=1; done
    games/emberwake/parity/journey.sh seeds=1,2,3,4 steps=700 || status=1
    games/emberwake/parity/sim.sh || status=1
    games/emberwake/parity/checks.sh || status=1
    ;;
  parse|parse-all|arena|dojo|run|safe_delete|export_placeholder|gen_sfx|systems|all)
    echo "'$1' is a Godot command: the Godot build is frozen in legacy/ (see legacy/LEGACY.md; legacy/tools/fc.sh, or the legacy branch)."; status=1 ;;
  *) sed -n '2,13p' "$0"; status=1 ;;
esac
exit $status
