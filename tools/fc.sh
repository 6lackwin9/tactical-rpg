#!/usr/bin/env bash
# Fast check for the Aegis engine (transition step 2, 2026-10-19): compact, failure-first output. Usage:
#   tools/fc.sh gate [auto]      the commit gate, same steps as check_project.ps1 -Fast -Quiet: build every crate, every test, the layer check,
#                                docs --all, tooltests
#                                (`auto` is accepted and runs everything)
#   tools/fc.sh layers           engine and kit crates name no game (any folder under games/) and no manifest has a path into games/;
#                                kit src, schema.json and kit.json name no lore word (world ids, games/<id>/lore_words.txt)
#   tools/fc.sh build            cargo build --workspace --all-targets (a compiler warning fails it)
#   tools/fc.sh test [filter]    cargo test --workspace, optional test-name filter
#   tools/fc.sh clippy           cargo clippy --workspace --all-targets (advisory: warnings do not fail)
#   tools/fc.sh docs [--all]     warn when an agent-facing doc passes 300 lines / 25 KB; markdown lint. Files changed against the merge base
#                                with origin/develop, staged or untracked; --all (the gate and the flush): every file
#   tools/fc.sh tooltests        the tools/tiles tests, time limit FC_TOOLTESTS_LIMIT (180 s); a visible SKIP when no Python has Pillow
#   tools/fc.sh content <args>   edit games/emberwake/data/ through the engine's writer: check | upsert | remove | world | apply | manifest (aegis content)
#   tools/fc.sh sim <args>       balance sim (aegis sim <mode> [level] [seeds] [key=value ...]; sim help: the keys)
#   tools/fc.sh replay <file> [...] replay a playtest recording (aegis replay: battle=, until=, json, variant=, write=, seeds=, fallback=)
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
  echo "$out" | grep -E "^test result|FAILED|panicked|^error|checked|diverge|compared|os error" | grep -v " 0 passed; 0 failed"
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

docs_check() {  # warn when an agent-facing doc grows past ~300 lines or ~25 KB (split by domain behind an index); markdown lint.
  # Every check looks at one file at a time. $1 = --all: every file. Else only *.md changed against the merge base with
  # origin/develop, staged or untracked (no origin/develop: every file). One wc and one awk for all files: a wc per file
  # forked 2-3 processes per file and was nearly all of the 80 s (2026-11-02, docs/EFFICIENCY.md).
  local all=$1 f base lines bytes warned=0 sized=() md=() total=0 out
  local -A changed=()
  if [ "$all" != "--all" ]; then
    if base=$(git merge-base HEAD origin/develop 2>/dev/null); then
      while IFS= read -r -d '' f; do changed[$f]=1; done < <(git diff --name-only -z "$base" -- '*.md'; git diff --cached --name-only -z -- '*.md'; git ls-files --others --exclude-standard -z -- '*.md')
    else
      echo "[docs] no origin/develop to compare with: checking every file"; all=--all
    fi
  fi
  for f in CLAUDE.md docs/*.md docs/workflow/*.md docs/specs/*.md games/*/docs/*.md games/*/docs/specs/*.md games/*/docs/lore/*.md games/*/docs/lore/*/*.md; do
    [ -f "$f" ] || continue
    case "$f" in */lore/archive/*) continue ;; esac  # archived exports keep their size
    if [ "$all" = "--all" ] || [ -n "${changed[$f]:-}" ]; then sized+=("$f"); fi
  done
  if [ ${#sized[@]} -gt 0 ]; then
    while read -r lines bytes f; do
      [ "$f" = "total" ] && continue
      if [ "$lines" -gt 300 ] || [ "$bytes" -gt 25000 ]; then echo "[docs] WARN $f is $lines lines / $bytes bytes: split by domain behind an index"; warned=1; fi
    done < <(wc -lc "${sized[@]}")
  fi
  [ $warned -eq 0 ] && echo "[docs] agent-facing docs within size limits (<=300 lines, <=25 KB)"
  # markdownlint subset (MD012/MD022/MD032/MD033) so lint warnings show up before the IDE panel does
  for f in CLAUDE.md README.md PATCH_NOTES.md docs/*.md docs/workflow/*.md docs/specs/*.md docs/decisions/*.md games/*/docs/*.md games/*/docs/*/*.md games/*/docs/lore/*/*.md games/*/data/*/README.md \
    .claude/agents/*.md .claude/skills/*/SKILL.md engine/README.md engine/docs/*.md engine/docs/*/*.md kits/*/README.md kits/*/crates/*/README.md legacy/LEGACY.md; do
    [ -f "$f" ] || continue
    total=$((total + 1))
    if [ "$all" = "--all" ] || [ -n "${changed[$f]:-}" ]; then md+=("$f"); fi
  done
  [ "$all" = "--all" ] || echo "[docs] changed files only: ${#md[@]} of $total (tools/fc.sh docs --all checks every file)"
  if [ ${#md[@]} -eq 0 ]; then echo "[docs] markdown lint clean (0 files)"; return 0; fi  # awk with no file would read stdin
  if out=$(awk -f tools/md_lint.awk "${md[@]}"); then echo "[docs] markdown lint clean (${#md[@]} files)"; else echo "$out" | head -20; echo "[docs] FAIL markdown lint: $(echo "$out" | wc -l) issue(s)"; return 1; fi
}

tools_python() {  # prints a Python that imports Pillow, or nothing; the WindowsApps `python` aliases are stubs, never probed
  local c p
  for c in "${LOCALAPPDATA//\//}"/Programs/Python/Python3*/python.exe python3 python py; do
    p=$(command -v "$c") || continue
    case "$p" in *WindowsApps*) continue ;; esac
    if "$p" -c "import PIL" >/dev/null 2>&1; then echo "$p"; return 0; fi
  done
  return 1
}

kill_tree() {  # $1 = a background job started by tooltests. Windows: taskkill /T /F ends the native process and every descendant
  # (a shell `kill` would end one process and leave its children); elsewhere the job leads its own process group (setsid).
  if [ -r "/proc/$1/winpid" ]; then taskkill //T //F //PID "$(cat "/proc/$1/winpid")" >/dev/null 2>&1; else kill -KILL -- "-$1" 2>/dev/null; fi
  kill -KILL "$1" 2>/dev/null
}

tooltests() {  # the tools/tiles tests, with a time limit (FC_TOOLTESTS_LIMIT seconds); no Python with Pillow: a visible SKIP, not a failure
  local py limit=${FC_TOOLTESTS_LIMIT:-300} log pid code start=$SECONDS run=()
  if ! py=$(tools_python); then echo "[tooltests] SKIP: no Python with Pillow found, tools/tiles tests did not run"; return 0; fi
  log=$(mktemp)
  [ -r "/proc/$$/winpid" ] || run=(setsid)
  (cd tools/tiles && exec "${run[@]}" "$py" -m unittest test_tiles) > "$log" 2>&1 &
  pid=$!
  # shellcheck disable=SC2064  # the pid is known now
  trap "kill_tree $pid; rm -f '$log'; exit 130" INT TERM
  while kill -0 "$pid" 2>/dev/null; do
    if [ $((SECONDS - start)) -ge "$limit" ]; then
      kill_tree "$pid"; wait "$pid" 2>/dev/null; trap - INT TERM
      tail -n 20 "$log"; rm -f "$log"
      echo "[tooltests] FAIL: tools/tiles tests hung: time limit of $limit s reached, process tree killed"; status=1
      return
    fi
    sleep 0.2
  done
  wait "$pid"; code=$?; trap - INT TERM
  if [ $code -eq 0 ]; then
    echo "[tooltests] ok: $(grep -E '^Ran [0-9]+ test' "$log") (tools/tiles)"
  else
    tail -n 40 "$log"; echo "[tooltests] FAIL: tools/tiles tests, exit code $code"; status=1
  fi
  rm -f "$log"
}

layers_check() {  # KIT1-G (docs/specs/KIT1_TASKS.md 4): the engine and kit crates name no game and have no path into games/
  local id ids=() hits bad=0
  for id in games/*/; do [ -d "$id" ] && ids+=("$(basename "$id")"); done
  # G1: every folder name under games/ is a game id; a case-insensitive hit in an engine or kit file (tracked or new, not
  # ignored) fails. Whole word, with `_` as a separator so crate names (<id>_rules) and prefixed ids count as well.
  for id in "${ids[@]}"; do
    if hits=$(git grep --untracked -n -I -i -E -e "(^|[^[:alnum:]])$(printf '%s' "$id" | sed 's/[^[:alnum:]_-]/\\&/g')([^[:alnum:]]|\$)" \
      -- engine/crates 'kits/*/crates/*' 'kits/*/schema.json'); then  # a pathspec glob must match the whole path
      echo "$hits" | head -20 | awk -v g="$id" '{ split($0, f, ":"); t = $0; sub(/^[^:]*:[^:]*:/, "", t); c = index(tolower(t), tolower(g)); c = c > 40 ? c - 40 : 1
        printf "[layers] %s:%s names game \"%s\": %s\n", f[1], f[2], g, substr(t, c, 120) }'  # the text around the hit (long JSON lines)
      bad=1
    fi
  done
  # G2: no path dependency of an engine or kit manifest resolves into games/; none of an engine manifest into kits/
  hits=$(git ls-files --cached --others --exclude-standard -- 'engine/*Cargo.toml' 'kits/*Cargo.toml' | xargs -r awk '
    FNR == 1 { dir = FILENAME; sub(/\/?Cargo\.toml$/, "", dir) }
    match($0, /path *= *"[^"]*"/) {
      p = substr($0, RSTART, RLENGTH); sub(/^path *= *"/, "", p); sub(/"$/, "", p)
      n = split((p ~ /^\// ? p : dir "/" p), seg, "/"); k = 0
      for (i = 1; i <= n; i++) { if (seg[i] == "" || seg[i] == ".") continue; if (seg[i] == "..") { if (k > 0) k--; continue } out[++k] = seg[i] }
      r = ""; for (i = 1; i <= k; i++) r = r (i > 1 ? "/" : "") out[i]
      if (r ~ /^games(\/|$)/) printf "[layers] %s:%d path dependency into games/: %s\n", FILENAME, FNR, p
      else if (FILENAME ~ /^engine\// && r ~ /^kits(\/|$)/) printf "[layers] %s:%d engine path dependency into kits/: %s\n", FILENAME, FNR, p
    }')
  [ -n "$hits" ] && { echo "$hits"; bad=1; }
  # G3 (TERMS1 7): kit code names no lore word of a game: its world ids (folders under games/<id>/data/worlds/, any case,
  # whole word with `_` as a separator, as G1) and the lines of games/<id>/lore_words.txt (fixed phrases, whole word, exact
  # case; `#` comments). Scope: kit src, schema.json and kit.json; kit tests and READMEs are not scanned. One git grep per
  # game (a world id becomes [Aa][Ss].. so both lists share one case-sensitive regex).
  local w ws lw ex
  for id in "${ids[@]}"; do
    ws=$(for w in "games/$id/data/worlds"/*/; do [ -d "$w" ] && basename "$w"; done | awk '{ r = ""
      for (i = 1; i <= length($0); i++) { c = substr($0, i, 1); r = r (c ~ /[[:alpha:]]/ ? "[" toupper(c) tolower(c) "]" : (c ~ /[[:alnum:]_-]/ ? c : "\\" c)) }
      printf "%s%s", (NR > 1 ? "|" : ""), r }')
    lw=$([ -f "games/$id/lore_words.txt" ] && tr -d '\r' < "games/$id/lore_words.txt" | sed -e 's/#.*//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e '/^$/d' \
      | sed 's/[][\.*^$+?(){}|/]/\&/g' | paste -sd'|')
    ex=()
    [ -n "$ws" ] && ex+=(-e "(^|[^[:alnum:]])($ws)([^[:alnum:]]|\$)")
    [ -n "$lw" ] && ex+=(-e "(^|[^[:alnum:]_])($lw)([^[:alnum:]_]|\$)")
    [ ${#ex[@]} -eq 0 ] && continue
    if hits=$(git grep --untracked -n -I -E "${ex[@]}" -- 'kits/*/crates/*/src/*' 'kits/*/schema.json' 'kits/*/kit.json'); then
      echo "$hits" | head -20 | awk -v g="$id" -v ws="$ws" -v lw="$lw" '{ split($0, f, ":"); t = $0; sub(/^[^:]*:[^:]*:/, "", t)
        if (lw != "" && match(t, "(^|[^[:alnum:]_])(" lw ")([^[:alnum:]_]|$)")) x = substr(t, RSTART, RLENGTH)
        else if (ws != "" && match(t, "(^|[^[:alnum:]])(" ws ")([^[:alnum:]]|$)")) x = substr(t, RSTART, RLENGTH)
        else x = "?"
        gsub(/^[^[:alnum:]]+|[^[:alnum:]]+$/, "", x); c = index(t, x); c = c > 40 ? c - 40 : 1
        printf "[layers] %s:%s names lore word \"%s\" of game %s: %s\n", f[1], f[2], x, g, substr(t, c, 120) }'
      bad=1
    fi
  done
  if [ $bad -eq 0 ]; then echo "[layers] ok: engine and kit crates name no game (${ids[*]}) and have no path into games/; kit code names no lore word"; else echo "[layers] FAIL: engine and kit code must not name a game, kit code no lore word (CLAUDE.md Layers)"; status=1; fi
}

gate() {
  build; [ $status -ne 0 ] && return
  cargo_test; [ $status -ne 0 ] && return
  layers_check; [ $status -ne 0 ] && return
  # the whole docs check; the size warnings, which are many, as one count (tools/fc.sh docs --all lists them)
  local out
  out=$(docs_check --all) || status=1
  echo "$out" | grep -v '^\[docs\] WARN'
  [ "$(echo "$out" | grep -c '^\[docs\] WARN')" -eq 0 ] || echo "[docs] $(echo "$out" | grep -c '^\[docs\] WARN') size warning(s)"
  [ $status -ne 0 ] && return
  tooltests; [ $status -ne 0 ] && return
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
  docs)
    case "${2:-}" in
      ""|--all) docs_check "${2:-}" || status=1 ;;
      *) echo "usage: tools/fc.sh docs [--all]"; status=1 ;;
    esac ;;
  tooltests) tooltests ;;
  layers) layers_check ;;
  content) aegis content "${@:2}" || status=1 ;;
  sim) aegis sim "${@:2}" || status=1 ;;
  replay) aegis replay "${@:2}" || status=1 ;;
  parse|parse-all|arena|dojo|run|safe_delete|export_placeholder|gen_sfx|systems|all)
    echo "'$1' is a Godot command: the Godot build is frozen in legacy/ (see legacy/LEGACY.md; legacy/tools/fc.sh, or the legacy branch)."; status=1 ;;
  *) sed -n '2,16p' "$0"; status=1 ;;
esac
exit $status
