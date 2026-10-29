# Time and token efficiency log

Living file: measure, fix, record. Add a row whenever a step feels slow or a run floods the context. Re-measure after changes. Times are wall-clock on the dev machine.

## Aegis engine (since 2026-10-19)

| Step | Cost | Verdict |
| --- | --- | --- |
| `tools/fc.sh gate` / `check_project.ps1 -Fast -Quiet`, warm build (2026-10-19) | 27 s: build check plus every test (battle traces ~8 s, journeys ~7 s, world battles ~5 s, the rest under 1 s) | pre-commit default |
| Layer check (KIT1-G, in the gate) | about 0.7 s in bash (0.24 s of it one `git grep` per game id), 0.75 s in PowerShell | fine; note a git pathspec glob must match the whole path (`kits/*/crates/*`, not `kits/*/crates`) |
| First build of a checkout or worktree (debug, every crate) | a few minutes; `run.bat` release build of the GUI and CLI likewise | once per folder; each worktree has its own build folder (fc.sh) |
| Per-worktree build folders (`<target-dir>-<worktree>`, 2026-10-25) | 5-10 GB each; 33 stale ones filled C: (about 200 GB) | `tools/wt_remove.ps1` removes a worktree's folder with it; `-Targets -WhatIf` is the cheap disk audit (WTCLEAN1) |
| `tools/fc.sh docs` | 16 s (129 files) | fine |
| `aegis sim duel <level> 2` (120 fights, debug build) | 0.6 s | the Godot sim took ~90 s for the same table |
| `aegis sim <mode>` (release build), the 20 recorded parity runs (`emberwake_sim` test `sim_parity`) | about 6 s in all | Godot took about 25 minutes for the same runs (`games/emberwake/parity/sim.sh`) |
| `tools/fc.sh parity` (Godot 4.7.2 on `legacy/`) | export ~5 s, each trace mode about a minute or less, journeys ~4 min | only when the reference must be re-recorded (a fresh checkout, CI records the exports only) |
| Windows Smart App Control blocking freshly built binaries (2026-10-19) | random test exes and GUI launches refused (`os error 4551`); retries and relinks did not always clear it | turned off by the user; the gates still relink a blocked exe (skill `known-issues`) |
| Local image generation (LOCALGEN1, 2026-10-26; LCM Dreamshaper int8, 512 px, 6 steps) | GPU (Arc 140V) 0.8-1.0 s an image, first compile seconds; NPU 2.1 s an image, but its first compile is 3.5-16 min for each new size or LoRA (cached in `C:/dev/localgen/cache`); img2img on the GPU 3.5-4.5 s | default device GPU; NPU only for long batches at one fixed size, never for LoRA trials |

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
| Arena suite on the cloud container (2026-10-18, release #38) | ~9.5 min (929 tests), whole gate 10m34s, about double the day before; the 290 s default timed out the release gate | `tools/fc.sh` defaults to 900 s when `CLAUDE_CODE_REMOTE=true` (290 s on desktop); `gate.yml` sets 900. Done (GATE-SPLIT, user 2026-10-18): arena split into 8 systems; tasks gate their own (`fc.sh gate auto`), CI runs them in parallel. Per-suite times 2026-10-18: Playtest 89 s, CombatUiFramework 86 s, Bal2Rules 81 s, Planning 71 s (42% of 772 s); by system rules ~110 s, ai ~150 s, combat_ui 125 s, balance 128 s, world 66 s, party_save 16 s, content 71 s, menus 105 s. Next: trim those four suites |
| `tools/fc.sh parse` in a branch review on a clean tree (2026-10-18, MAPKEYS1-U) | checks only files `git status` lists, so after a commit it checks nothing | reviewers run `godot --headless --check-only` on `git diff --name-only origin/develop...HEAD` (`.gd` files) instead |
| Builder agent for M1 | ~23 min, ~300k tokens, 159 tool calls | the dominant cost; see below |
| Reviewer agent | ~2.5 min, ~30k tokens | cheap, keep |
| Brainstorm agent | ~1.5 min, ~56k tokens | cheap, keep |
| gate `godot --headless --import` step (S14-A1/A2) | ~2 s when nothing new, longer on a fresh checkout with new assets | fine; needed so CI loads committed PNG/WAV files |

### Bottlenecks found and fixes

1. **Per-file parse in check_project.ps1 (~155 of 188 s).** Fixed: `tools/parse_all.gd` + `-Fast` switch (and `fc.sh parse-all`).
2. **Test output floods context (~9k tokens per full run).** Fixed: `-Quiet` prints failures and a summary only; `fc.sh` prints failures first with a log path. Rule: never print full suite output; grep the log.
3. **Patch scripts that failed silently or truncated files** (find() == -1, `\n` in `"""` strings, shell heredocs with GDScript). Cost: several retry loops. Fixed: `patch-and-seed` skill (Edit tool for text, upsert for data), known-issues rows.
4. **Test numbers hard-coded to old data** forced repeated suite-by-suite fixes after each data change. Fix: derive expectations from rules (done for new suites); when data changes broadly, print real values once with a scratch script instead of iterating through full runs.
5. **Rediscovering context** (reading files to learn rules). Fixed: `CLAUDE.md` + `games/emberwake/docs/GAME_BRIEF.md`, STRUCTURE, DECISIONS, HISTORY. Keep them updated in the same change.
6. **Large builder tasks.** One agent doing engine + AI + data + tests is long and expensive. Mitigation: split milestones into slices of roughly one subsystem each (e.g. M1 could have been 1: ally targeting, 2: gap-close + statuses, 3: AI), brief the builder with exact file pointers from the brief instead of letting it explore, and ask it to report open questions early rather than at the end.
7. **Re-spawning agents for follow-ups** loses their context. Prefer `SendMessage` to continue the same agent for fixes to its own work.
8. **Waiting on long jobs in the foreground.** Run sims and full checks with `run_in_background` and do doc work meanwhile; parallelise independent sims (each has its own sandbox).
9. **Custom agents are not callable until the session restarts**; a fallback (general-purpose agent with the same instructions) worked. Restart after adding agent files to avoid the detour.
10. **Stale IDE diagnostics** after adding `class_name` files caused false alarms. `fc.sh parse` refreshes the class cache; trust it over the IDE panel.

### Working rules (cheap path first)

- Edit -> `tools/fc.sh parse` -> `tools/fc.sh arena` (or `dojo`) -> repeat. Run `check_project.ps1 -Fast -Quiet` only before a commit.
- Read with `grep -n` and line ranges, not whole files; use the brief to find the file first.
- Batch independent tool calls in one turn; background anything over ~30 s.
- After 2 failed fixes, ask the user instead of brute-forcing (token sink).

### Ideas not done yet

- `fc.sh suite <name>`: run a single test suite (arena takes 20 s, so low value now; revisit if it grows past ~60 s).
- Cache catalog loads in sims (minor).
- Cut `balance_sim` time by lowering tick/step overhead or fighting fewer seeds on obvious matchups (revisit when tuning).

### Agent reuse (2026-10-15)

Agent types are reused; instances are fresh per call. Fresh instance for reviews (independence), balance runs and new milestones; resume the same instance (SendMessage) for follow-ups to its own work; do tiny fixes in the main session. A long-lived builder's context grows and every follow-up re-reads it. Since 2026-10-27 (decision "Logic lane scope") the builder takes only large logic work, one slice per run; the planner does small logic inline with the skills. A chain of tasks (e.g. HOOKS1 E, K, X, M) gets a fresh builder per task, briefed with the previous builder's hand-off notes; only a small next task on the same files (HOOKS1-X after HOOKS1-K) continues the previous builder (user 2026-10-28: a continued builder re-sends its whole context, e.g. 245k tokens, on every call).

### Parallel lanes (2026-10-16)

Idle time while a single builder ran (4-23 min) was the largest remaining cost. Fixes: split the builder into `logic-builder` and `asset-builder` with disjoint file ownership; run `design-brainstorm` ahead of the builders writing specs; run `balance` continuously in its own `main` worktree; reviews in the background; `fc.sh` logs per worktree. Expected costs: same tokens, less wall time; one-time worktree import delay; merge discipline matters (schema first, data files only touched by the asset lane). Re-measure after the first parallel milestone and update this section.

### Doc housekeeping (2026-10-16)

- Done: superseded drafts go to `docs/archive/`; `tools/fc.sh docs` (also in `fc.sh all`) warns when an agent-facing doc passes 300 lines or 25 KB.
- Parked for the next heavy edit (user: pick up then, not now): (3) split `docs/DECISIONS.md` by domain behind a short index once it passes ~25 KB (18 KB at the time of writing; rows are long single lines; done 2026-10-17 in CLN-10 at 36 KB: index plus `docs/decisions/<system>.md`); (4) turn the 640-line README into an overview plus per-system files in `docs/features/` when it next needs a large update. Agents do not read the README, so the benefit is safer edits for the planner.

### Downtime pass (2026-10-17, after SKILL1)

Measured on SKILL1: the asset lane waited for the whole logic run (schema + UI) instead of the schema slice; a usage-limit stop and an API error each forced a fresh builder to re-derive context; a brief narrower than a recorded user decision cost a seed re-run; the review fix went to a fresh builder (140k) instead of the resumable one; markdown lint issues were found only in the IDE panel (it lints open files only), so 38 old issues had piled up; 12 finished worktrees made every git command print "Permission denied" noise.

Fixes: schema slice is its own logic-builder run that ends with the report; builders keep `.agent/progress.md` (gitignored) and resume from it; builders flag brief-vs-spec/DECISIONS contradictions in the first report line; the planner checks briefs against DECISIONS, sends review fixes to the same builder via SendMessage, re-runs the gate after its own edits, auto-dispatches the reviewer on a finished builder and the brainstorm agent on a new balance report; `tools/md_lint.awk` (MD012/MD022/MD032/MD033 subset, no node) runs inside `tools/fc.sh docs` and fails it; the commit trailer comes from the session's attribution reminder instead of a hard-coded model; worktrees are removed right after their merge.

### Parallel CPU contention (2026-10-17)

With 3 logic builders plus a reviewer running Godot at once, `check_project.ps1 -Fast -Quiet` took about 10 minutes instead of 32 s (EXP1-L report). Reviewers re-running the full check add another Godot process. Rule: above 3 concurrent builders, expect checks to slow sharply; prefer starting the next builder when one finishes, and let a reviewer trust a fresh builder check when the diff is a small schema slice.

## Lanes and the builder's partial gate (2026-10-30)

- Measured 2026-10-29/30: a builder in a fresh worktree spent 5-10 min compiling a cold build cache (10-16 GB each; 12
  stale caches filled drive C:) and 5-7 min on a full gate that the staging gate and the flush repeat.
- Fix (decision "Builder efficiency"): three persistent lane worktrees keep their path and warm cache (cargo keys
  workspace crates to the source path, so moving a cache to another path does not help); builders end with the partial
  gate `pgate.sh`; the full gate runs once per flush.
## Smaller build caches (BUILDPROF1, user 2026-10-30)
- `Cargo.toml` `[profile.dev]` keeps line tables only and third-party crates build without debug info: a lane cache with
  every test binary is 4.9 GB instead of 14 GB (compiled deps 11 to 3.5 GB, incremental 3.2 to 1.4 GB). A profile change
  rebuilds every cache once.

