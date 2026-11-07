# Archived 2026-11-07, superseded by Aegis Engine timings in docs/EFFICIENCY.md

## Godot build (2026-10-15 to 2026-10-18, frozen in `legacy/`)

The measurements and fixes below are the Godot build's; they stay as the record. Its tools (`fc.sh parse`, `arena`, `dojo`, `balance_sim.gd`) are in `legacy/tools/`.

### Measured steps

| Step | Cost | Verdict |
| --- | --- | --- |
| `tools/fc.sh parse` (changed files) | 0.5 s (auto class-cache refresh only when new .gd files appear) | fine |
| `tools/fc.sh parse-all` (every script, one process) | 3 s | use for "does everything compile" |
| `tools/fc.sh arena` | 20 s | fine |
| `tools/fc.sh dojo` | 2.5 s | fine |
| `check_project.ps1` (old default) | 188 s, ~170 output lines (~9k tokens of PASS lines) | **slow + noisy**: 130+ Godot launches (one per file) |
| `check_project.ps1 -Fast -Quiet` | 32 s, ~20 output lines | pre-commit default |
| `tools/fc.sh gate` / `check_project.ps1 -Fast -Quiet` on a Linux cloud container (2026-10-17) | ~3.5 min on a fresh clone, ~3 min with the class cache; parse steps take ~17 s, so the arena suite (inferred) dominates | slow but usable; worth measuring the arena suite there if cloud checks become routine |
| `check_project.ps1` (per-file parse) on a Linux cloud container | ~8 min | avoid; use `-Fast` |
| GitHub CI `checks` job on a release PR after KIT1 (2026-10-27, #49: every crate including the seven kit crates, every test) | 4 min 49 s | fine; the kit split did not slow the gate noticeably (cloud gate ~3-3.5 min on 2026-10-17 with fewer tests) |
| `tools/fc.sh sim duel 5` (3 seeds) | ~90 s; `tier` ~2 min; 10 seeds ~5 min | acceptable; run in background / in parallel (own sandboxes) |
| `tools/fc.sh sim` on a fresh cloud checkout (2026-10-17, BAL3-R0) | hung silently with empty output for ~40 min (stale class cache after a new `class_name`, e.g. `AssetRegistry`); cloud role runs: L3 19 min, L5 35 min, L7 40 min with 4 runs in parallel | fc.sh imports automatically when the class cache is stale (one import under flock for parallel runs); timeouts print TIMEOUT; sim has a 120 s startup watchdog |
| `tools/fc.sh gate` arena suite on the cloud container (2026-10-17, BAL3-S) | ~5.5 min, base commit included, so the default 290 s suite timeout fails there | set `SUITE_TIMEOUT=900` (seconds; default 290) when gating in the cloud |
| Arena suite on the cloud container (2026-10-18, release #38) | ~9.5 min (929 tests), whole gate 10m34s, about double the day before; the 290 s default timed out the release gate | `tools/fc.sh` defaults to 900 s when `CLAUDE_CODE_REMOTE=true` (290 s on desktop); `gate.yml` sets 900. Done (GATE-SPLIT, user 2026-10-18): arena split into 8 systems; tasks gate their own (`fc.sh gate auto`), CI runs them in parallel |
| `tools/fc.sh parse` in a branch review on a clean tree (2026-10-18, MAPKEYS1-U) | checks only files `git status` lists, so after a commit it checks nothing | reviewers run `godot --headless --check-only` on `git diff --name-only origin/develop...HEAD` (`.gd` files) instead |
| Builder agent for M1 | ~23 min, ~300k tokens, 159 tool calls | the dominant cost; see below |
| Reviewer agent | ~2.5 min, ~30k tokens | cheap, keep |
| Brainstorm agent | ~1.5 min, ~56k tokens | cheap, keep |
| gate `godot --headless --import` step (S14-A1/A2) | ~2 s when nothing new, longer on a fresh checkout with new assets | fine; needed so CI loads committed PNG/WAV files |
| Partial gates of engine-wide changes under three busy lanes (2026-11-02) | aegis_ui changes (LOOK1-ME, LOOK1-G1) pull in every emberwake crate: 722-1095 tests, 824-881 s; kit-only or tool changes 90-200 s; full gate at a flush 276-873 s (1383-1450 tests) | Reviewers skip re-running a gate whose log is newer than every changed file (run the targeted tests instead); keep at most two engine-touching lanes at once |
