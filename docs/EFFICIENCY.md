# Time and token efficiency log

Living file: measure, fix, record. Add a row whenever a step feels slow or a run floods the context. Re-measure after changes. Times are wall-clock on the dev machine.

## Aegis engine (since 2026-10-19)

| Step | Cost | Verdict |
| --- | --- | --- |
| `tools/fc.sh gate` / `check_project.ps1 -Fast -Quiet`, warm build (2026-10-19) | 27 s: build check plus every test (battle traces ~8 s, journeys ~7 s, world battles ~5 s, the rest under 1 s) | pre-commit default |
| Layer check (KIT1-G, in the gate) | about 0.7 s in bash (0.24 s of it one `git grep` per game id), 0.75 s in PowerShell | fine; note a git pathspec glob must match the whole path (`kits/*/crates/*`, not `kits/*/crates`) |
| First build of a checkout or worktree (debug, every crate) | a few minutes; `run.bat` release build of the GUI and CLI likewise | once per folder; each worktree has its own build folder (fc.sh) |
| Per-worktree build folders (`<target-dir>-<worktree>`, 2026-10-25 / 2026-11-07) | 5-10 GB warm each, growing to 15-32 GB as Cargo accumulates old hashed `.exe` and `incremental/` dirs | `tools/wt_remove.ps1 -Targets [-WhatIf]` removes orphaned worktree folders (WTCLEAN1); `tools/wt_remove.ps1 -Trim [-WhatIf]` prunes superseded `.exe` and `incremental/` hashes older than 60m (keeping newest 2 + `.rlib` warm cache; freed 53.71 GB across 5 folders on 2026-11-07) |
| `tools/fc.sh docs` | 16 s (129 files); 80 s (272 files, 2026-11-02); now 0.7 s changed files, 0.9 s `--all` (EFF1) | fine |
| `aegis sim duel <level> 2` (120 fights, debug build) | 0.6 s | the Godot sim took ~90 s for the same table |
| `aegis sim <mode>` (release build), the 20 recorded parity runs (`emberwake_sim` test `sim_parity`; since CLN-39 `sim_golden`, 22 runs, about 16 s in the test profile, the old test took 22 s there) | about 6 s in all | Godot took about 25 minutes for the same runs (`games/emberwake/parity/sim.sh`) |
| `tools/fc.sh parity` (Godot 4.7.2 on `legacy/`) | export ~5 s, each trace mode about a minute or less, journeys ~4 min | only when the reference must be re-recorded (a fresh checkout, CI records the exports only) (retired 2026-11-01, CLN-42: `fc.sh parity` and the recorders are gone) |
| Full gate after CLN-42 (2026-11-01 flush) | about 5 min, 1335 tests | the Godot-only suites are gone; the engine goldens replace them |
| Windows Smart App Control blocking freshly built binaries (2026-10-19) | random test exes and GUI launches refused (`os error 4551`); retries and relinks did not always clear it | turned off by the user; the gates still relink a blocked exe (skill `known-issues`) |
| Battle screen ghost drafts (INPUT1 GH2 a, measured 2026-11-01 in the test profile on the two 4-member fights, `tl_skirmish` and `kit_winch_escape`) | one member's draft on copies (world clone 0.15-0.2 ms, the rest the tactic's AI draft) 1.2-1.6 ms; a round's planning start 5-7 ms; a pick redrafts the later members (up to 3 drafts, about 5 ms) | fine, not optimised (under one frame; the preview views are cached for the battle's revision) |
| Local image generation (LOCALGEN1, 2026-10-26; LCM Dreamshaper int8, 512 px, 6 steps) | GPU (Arc 140V) 0.8-1.0 s an image, first compile seconds; NPU 2.1 s an image, but its first compile is 3.5-16 min for each new size or LoRA (cached in `C:/dev/localgen/cache`); img2img on the GPU 3.5-4.5 s | default device GPU; NPU only for long batches at one fixed size, never for LoRA trials |

## Godot build (2026-10-15 to 2026-10-18, frozen in `legacy/`)

Archived in [EFFICIENCY_GODOT_2026-10.md](archive/EFFICIENCY_GODOT_2026-10.md).

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

## Efficiency pass (user 2026-11-02)

Measured on 2026-11-02: a builder's first pass 13-30 min, review 4-17 min, a fix round 4-15 min, usually 1-2 per task; full
gate under 2 min with idle lanes, 7.5 min with three lanes building; `fc.sh docs` 85 s under load. The biggest cost was
iterating on visuals through builders: the frame took 8 rounds and over 2 hours. Rules in decision "Efficiency pass".

Builder self-check, the findings reviews kept repeating:

- Inputs from files or the public API are bounded, refused rather than truncated or wrapped.
- A cache key holds everything the result depends on, by content; a resize or drag cannot grow a cache.
- A rule's test fails when the rule is broken: break the code once and see it fail (mutation).
- New paths run on real art or data, not only fixtures; say what was viewed, from the final files.
- Docs that describe what changed are updated, and specs get an as-built note where the build differs.
- Repeats: a temp script, a repeated tool call or a trial-and-error loop is fine the first time. Report each in the report's `Repeats:` line, so it can become a tool, skill or checklist line. Before writing one, check the list below; keep scratch files in the task's own subfolder of the scratchpad.

Docs check (EFF1, 2026-11-02): of the 80 s, the size loop's `wc` per file took 66-141 s when timed alone (2-3 forks per
file on Windows, where a fork is slow under load); one `wc -lc` on all files takes 0.13-0.19 s, the lint 0.5-0.8 s and the
list of changed files 0.3-0.6 s. `tools/fc.sh docs` now reads changed files only and `--all` every file; the gate runs
`--all`. On Windows a fork in a shell loop is the first thing to suspect in a slow script.

Tool tests (EFF1): `tools/fc.sh tooltests` takes about 30 s (120 tests) and joins the gate; 180 s is the limit because the
gate can take four times as long under load.

## Ground composer (REGIONGROUND1-E1, 2026-11-02)

Budget (spec `docs/specs/REGIONGROUND1_ENGINE.md` section 4.3): compose at most 100 ms, a copy at most 20 ms at 4K, in
release, for a 48 x 30, 8-layer job. Measured with `ground_compose --timing` (10 runs each, best to worst, while other
lanes built on the same machine):
| Form | Compose | Copy at 720p | Copy at 1080p | Copy at 4K | Blit of a 4K frame | Cache holds at 4K |
| --- | --- | --- | --- | --- | --- | --- |
| Square, 768 x 480 art px | 21-25 ms | 1.3-1.6 ms | 2.7-3.6 ms | 11-12 ms | 2.4-4.2 ms | 54.6 MB |
| Tilted, 768 x 360 art px | 16-20 ms | 1.0-1.1 ms | 2.0-2.9 ms | 8.1-12 ms | 2.5-3.6 ms | 42.4 MB |
The cheap path: an x blur runs once per cell row, not per pixel row, and a y blur is a few weights per cell row (a cell
row's pixels are alike), so the integers match full running sums at a fraction of the work. Only the noise a job reads is
made. Compose runs once on a map's first frame; a frame only blits what shows. `GroundKey::of` hashes every texture
pixel: 0.14 ms for that job, so it runs when the job changes, never each frame.

Patches and marks (REGIONGROUND1-E2, 2026-11-02), same budget, `ground_compose --timing` in release, 10 runs, best to worst,
every layer with `patches` .2 and 10 mark tries per 100 cells, other lanes building: square 38-50 ms on the many-borders job
(25-36 ms without them), 17-23 ms on one open ground with 16 placed patches; tilted 31-47 ms and 14-24 ms (17-23 ms without). A
job that asks for none reads no patch noise and no extra blur, so its cost and bytes do not move.

## Heavy tasks (user 2026-11-04)

The main session tracks each agent run's cost from its completion notice (tokens, tool calls, time). A run counts as heavy past about 200,000 tokens, 100 tool calls or 60 minutes; these lines are a starting point, to tune. For each heavy run, note why and what could reduce the effort: a narrower brief, a mock first, a tool or skill, a split task. Flag to the user when a cause repeats or looks fixable, so it can be streamlined. Time alone can mean machine load, not effort.

| Task (2026-11-02 to 11-04) | Agent | Tokens | Tool calls | Minutes | Likely cause |
| --- | --- | --- | --- | --- | --- |
| REGIONVIEW1 with zoom | logic | 485k | 146, plus 26 on fixes | 78, plus 25 | Scope grew mid-run (zoom), hand-written window measures |
| LAYOUT720-E3 | logic | 367k | 129 | 45 | Every widget against its old picture |
| LAYOUT720-E4 | logic | 328k | 73 | 47 | The same, for panes |
| LAYOUT720 spec | design | 316k | 55 | 32 | The direction changed 7 times during the run |
| LAYOUT720-E1 | logic | 282k | 100 | 30 | Shell and painter proof in real windows |
| DASH1 | logic | 261k | 74 | 90 | Sprite suites are slow; ran under load |
| LAYOUT720-M0 | logic | 250k | 76 | 72 | Hand-written T4 sweep in real windows |
| LOOK1-G2 | logic | 252k | 82 | 49 | About 700 uses migrated |
| VISCLAR1-C | logic | 375k | 81 | 56 | Five tasks in one run; hand-written timing probe |
| LAYOUT720-E5 | logic | 368k | 97 | 61 | Sweeps and proof in real windows |
| FPSCAP1 | logic | 206k | 61 | 50 | About 25 hand-written window measures |
| HEADLESS1 | logic | 252k, plus 54k on fixes | 51, plus 17 | 30, plus 14 | New launch mode with proof comparisons |
| INTENT2 spec | design | 361k, plus 91k on the revision | 61, plus 16 | 31, plus 10 | Every skill classified; seven additions from the user during the run |
| VISCLAR1-D slice 2 | logic | 371k, plus 142k on fixes | 107, plus 40 | 54, plus 29 | A kit rule and a new menu in one slice; two user answers added mid-run. Second visual slice over 500k with fixes |
| ATTACKLOCK1 slice 1 | logic | 234k | 69 | 25 | A rule with on and off sim runs, golden checks |
| BASERULES1 spec | design | 392k | 67 | 29 | Audit of the whole base rules and every option; five additions from the user during the run |
| ATTACKLOCK1 answers | logic | 84k | 25 | 18 | Fix pass on the same builder |
| VISCLAR1-D slice 1 | logic | 381k, plus 76k on fixes | 156, plus 34 | 54, plus 20 | Six features in one slice; a hand-written proof driver run 5 times, one batch blank |
| TURNFEEL1 | logic | 226k | 62 | 87 | Measured real windows by hand; scope grew mid-run |
| LAYOUTDATA1 spec | design | 224k | 44 | 19 | Long spec |
| AGENTROLES1 spec | design | 214k | 39 | 18 | Long spec, three additions during the run |

Repeating causes so far: hand-written real-window proof (AGENTROLES1's `aegis scenario` and the `real-window` skill target this), a scope added after the run started, and specs asked to cover much at once.

## Repeat findings (user 2026-11-04)

Anything done repeatedly, a temp script worth keeping or a trial-and-error loop is fine once, then gets tracked here and added to the builder checklist, to avoid token waste on repetitive tasks. The main session adds each `Repeats:` line from a report here, and acts on it: a checklist line, a skill, or a tool.

| Finding | Seen in | Fix | Status |
| --- | --- | --- | --- |
| Hand-written scripts driving a real window | 20 scripts in the lanes, 2,451 lines; most UI tasks | `aegis scenario`, skill `real-window` | INTEG1-E1, SKILLS1 |
| Scratch files overwriting each other in the shared scratchpad | VIEWKEYS1, TURNFEEL1, LAYOUTDATA1-E1 | Task subfolder (self-check above) | Done |
| `.agent/pgate.txt` left stale, gate output under other names, so reviewers read the wrong log | E3, E4, DASH1, CIPANES1, HOOKS1-A | The brief names one output; `pgate.sh` should write it itself | SKILLS1 |
| Content batches discover by check-and-fix loops that rows name things coming later, then hold them | CAST1-A2, CAST1-A4, CAMPAIGN1-D | Asset checklist: list what each row waits on before applying; hold those rows from the start | Done |
| Planner docs commits failed silently: chained commands went on after a script error, and two ran at once in `-plan` | 2026-11-04, twice | One docs commit at a time; stop on error; check the result. In skill `task-landing` under SKILLS1 | Part done |
| Scenario runs failing with "window minimized" at 1080p, intermittently | VISCLAR1-C | HEADLESS1: a run has no window unless asked; a blank shot is refused. Why a real window can open minimized is still unknown | Done for runs with no window |
| Hand-written proof shots and blank 73-byte PNGs | VISCLAR1-D, FPSCAP1 (`shots.py`, about 30 window runs) | `tools/fc.sh scenario shots <name> ...`, compare with `shot_compare`; `window=1` only for what needs a window | HEADLESS1 |
| Mutation scripts rewritten per task | Most logic tasks | Skill `mutation-check` | SKILLS1 |
| Comparison boards rewritten per task | LOOK1, REGIONGROUND1, SPRITEFIT1, VISCLAR1 | Skill `review-board` | SKILLS1 |
| `tools/fc.sh test` hides panic text, so a builder reruns raw `cargo` to read a failure | ATTACKLOCK1, twice | Show panic lines in its filter | Open |
| Python heredocs fail on quotes in the shell; agents fall back to writing a script file, and throwaway patch scripts pile up | INTENT2, BASERULES1, ATTACKLOCK1, VISCLAR1-D (`p1`-`p12`) | Agent note: use Edit or a script file from the start | Open |
| A planner script failing partway while the next command still lands the rest | FPSCAP1's docs, 2026-11-05 | `set -e` in landing commands too; do not chain a landing after a docs script | Done |
| An agent and the planner sharing `-plan`: an agent's checkout moved it between the planner's commit and push, and the commit stayed unpushed | INTENT2 revision, 2026-11-05 | No agent works in `-plan`; a spec revision takes a lane | Done |
| A user's design additions reaching a running spec agent one by one | INTENT2 seven, BASERULES1 five | Hold them for one revision pass when the run is past its first draft | Open |
| Tests leaving scratch folders in the temp dir | 8,181 folders by 2026-11-06 | TESTCLEAN1; the planner cleared them and four stale build caches (6 GB) | Open |
| A planner row insert matched a task id inside another row's dependency cell and split two rows (KITIDS1, BASERULES1-DOM) | 2026-11-05, found by MASTERPLAN1 | Insert before a line that starts with the id; check cell counts after each TASKS edit | Done |
| The `tools/fc.sh` gate filter hid a failing frame list | CI on the 2026-11-03 release | Filter widened; skill `ci-read` | Part done, SKILLS1 |

## Local CPU and heat (user 2026-11-04)

The machine is a low-power 8-core laptop chip (Core Ultra 7 258V). Each `cargo` and test run takes every core unless capped, and a window took about 90 % of a core before FPSCAP1. Since then a resting menu takes 1-2 % and a battle waiting for input 17-22 % (dev build, 720p); region and world maps still draw every frame (FPSCAP2). Decision "Local CPU and heat" sets the limits. Track the full gate's time per flush:

| Date | Gate time | Load |
| --- | --- | --- |
| 2026-11-03 | 299-360 s | Quiet |
| 2026-11-04 | 954 s | One lane running |
| 2026-11-04 | 1533 s | Builders and a re-landing |
| 2026-11-04 | see the last flush | Quiet |
| 2026-11-05 | 402 s | With the caps; reviewer only |
| 2026-11-05 | 487 s | With the caps; one builder running; 2,112 tests |

CPU1 (2026-11-04): each `cargo` run takes 4 build jobs and 4 test threads (`.cargo/config.toml`). Partial and full gates queue, one at a time, and run below normal priority; a builder's direct `cargo` command is capped but not queued or lowered. `flush.sh` logs each full gate to `gate_times.tsv` in the planner's script folder, copied to the table above. Run a gate in the background: a queued one can pass the shell's 10-minute limit.

Builder tip (PR4-2, 2026-11-10): dry-run a kit re-freeze in a scratch worktree (`RECORD=1` on `timeline_golden`) before re-recording, so the commit names exactly the goldens it moves.

Builder tip (PR4-4, 2026-11-10): total a long test log with `grep '^test result' | awk` instead of reading it.
