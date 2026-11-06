# Workflow: build, test, iterate

Since 2026-10-19 the game runs on the Aegis engine (Rust, `engine/`); the Godot-era version of this page is in
`legacy/docs/WORKFLOW.md`. The agent run log below is append-only and keeps its Godot-era rows.

## Bottlenecks seen so far and the fix

| Bottleneck | Fix |
| --- | --- |
| Re-reading the codebase for context | `games/emberwake/docs/GAME_BRIEF.md`, `ROADMAP.md`, `DECISIONS.md` (indexed from CLAUDE.md; index plus `games/emberwake/docs/decisions/<system>.md` or `docs/decisions/workflow.md`, read only the targeted system's file); `engine/docs/PORT_NOTES.md` maps each Godot file to its engine module |
| Full check floods output | `tools/fc.sh`: `build` (errors and warnings first), `test <filter>` (summary and failures only), `gate` (build + every test, ~30 s warm) |
| Failures re-debugged from scratch | `known-issues` skill table; add every new cause |
| Ad-hoc patch scripts truncating files / data edits by hand | `patch-and-seed` skill: Edit tool for text, `aegis content` (upsert, remove, JSON seeds) for data |
| Test numbers hard-coded, break on every data change | Derive expectations from rules or the engine goldens; print real values first (`aegis battle ... log`, `aegis serve`) |
| A rule change silently drifting from the released game | The engine goldens (rng_hash, rules_golden, sim_golden, checks_golden, journey, timeline, PHASEFOLD); an intended change re-records them, named in the commit and a HISTORY row (CLAUDE.md "Engine goldens") |
| Guessing at design/balance | Ask the user (options + trade-offs) after 2 failed fixes or any design choice |
| Markdown lint warnings (MD060, MD022, MD032, MD012) | Pad table pipes (`\| --- \|`), single blank lines around headings/lists, no consecutive blanks |

## Loop for a change

1. Read the brief; look up the relevant system in "Where to look for X".
2. Edit (Edit tool; data via `aegis content`, skill `patch-and-seed`).
3. `tools/fc.sh build`, then `tools/fc.sh test <filter>`; fix from the compact failure list.
4. Before commit: `check_project.ps1 -Fast -Quiet` (or `tools/fc.sh gate`; both include `tools/fc.sh docs --all`) clean, or for a docs-only change `tools/fc.sh docs` (files changed against the merge base with `origin/develop`, staged or untracked), READMEs + docs updated; on desktop also IDE Problems tab 0 errors/warnings (markdownlint compliant). Desktop vs cloud steps: `docs/workflow/`.

## Agents and tools (built)

(Skills: engine-checklist, known-issues, rust-conventions, patch-and-seed, planner, planner-ops, pc-launch, patch-notes-write, milestone-status, task-landing, game-design-library. Agents: design-brainstorm, logic-builder, asset-builder, balance, change-reviewer, housekeeper.)

| Tool | Use |
| --- | --- |
| `tools/fc.sh build / test / gate / clippy / docs / tooltests / layers / content / sim` | Compact checks, data edits and the sim (see the header of the script); `test` prints each panic's message on failure (SKILLS1); `gate` is the commit gate in bash, `check_project.ps1` the same in PowerShell; `layers` is the layer check alone (KIT1-G, KITIDS1, under 1 s): engine crates and kit crates (`kits/*/crates/**`, `kits/*/schema.json`) name no game (each folder under `games/`, any case, `_` as a separator, tracked or new files), kit `src/` quotes no class id from a game's `classes.json`, and no engine or kit `Cargo.toml` has a path into `games/` (engine none into `kits/`); G3 (TERMS1-G): kit `src/`, `schema.json` and `kit.json` name no lore word of a game (its world ids, any case, and the lines of `games/<id>/lore_words.txt`, exact case, whole word; kit tests and READMEs are not scanned); the gate and `check_project.ps1` step [3/5] run it; `docs` checks size and lint on changed files, `docs --all` on every file (gate step [4/5], the flush; under 1 s); `tooltests` runs `tools/tiles`' `test_tiles` with a Python that has Pillow (gate step [5/5], about 30 s; a visible SKIP when there is none, a failure after a time limit of 180 s, `FC_TOOLTESTS_LIMIT`, killing the whole process tree); planner helper scripts live in `tools/planner/` (`flush.sh` logs gate times to `$AEGIS_PLANNER_LOG`) |
| `aegis` (shell `engine/crates/aegis_cli`, binary in `games/emberwake/crates/emberwake_host`; `cargo run -q -p emberwake_host -- ...` from the repo root) | `sim <mode> [level] [seeds] [key=value ...]` (the balance sim: duel, tier, levels, group, stats, party, role, sweep, campaign; `sim help` lists the keys; a duel table in under a second; engine-only, not in the help: `feel` and CAMPAIGN1-SIM's fightless `deadlines [world=<id>] [losses=0..9]`, the best and worst day per campaign checkpoint and its slack), `battle <encounter> [seed=N] [log]`, `serve` (the battle command API over stdin / stdout), `link` (drive a running `emberwake`), `content` (edit `games/emberwake/data/`) |
| `games/emberwake/parity/*.sh` (`tools/fc.sh parity`) | Retired by CLN-42 (2026-11-01): the Godot recorders; the engine goldens replace the parity suites |
| `RECORD=1 tools/fc.sh test --test timeline_golden` | Re-records the timeline golden (RULESET1-X, `emberwake_battle/tests/fixtures/timeline_golden.json`) from the frozen catalog `timeline_kit.json` (delete that file first to re-freeze it from the shipped data); only for an intended rule change, named in the commit and a `docs/HISTORY.md` row; never edit the golden by hand |
| Agent `design-brainstorm` | Options + trade-offs + recommendation from the brief and sim numbers; never edits |
| Agent `change-reviewer` | Pre-commit review: the gate, code patterns, docs/README, known-issues match; read-only |
| Skills `engine-checklist`, `known-issues`, `patch-and-seed` | Edit checklist, known issues and safe editing |

The Godot build's sim (`tools/balance_sim.gd`, modes duel, tier, levels, group, stats, party, role, sweep, campaign) stays in `legacy/` as an archive; every mode is ported (`aegis sim <mode>`, crate `emberwake_sim`, PORT-SIM); since CLN-39 the test `emberwake_sim/tests/sim_golden.rs` compares those command lines with an engine-recorded golden on frozen data. The key `rewards=on` (REWARD1-S, off by default) adds what the defeated enemies pay: in `party` a line per encounter (defeated of roster with slain / surrendered / fled, XP per hero at the sim level, the purse share at the range mean, the expected drop value, key loot per fight), in `campaign` a line per run and the per-run means per steel choice.

Balance baseline (level 5, 2026-10-15): warrior wins every duel (100%), berserker 80%, brawler 53%, scout 47%, ranger 20%, shaman 0% (cannot beat a minion of itself: stalemates). Hero loses 1v3 minions and 1v1 bosses; elite = mirror (50%). These are open balance questions, not decisions.

## Iteration loop (agreed 2026-10-15)

Superseded (note 2026-10-29, CLEANUP.md CLN-26): steps 3 and 4 below (a `builder` agent on a `feat/<id>-<slug>` branch, merged to `main`) are the 2026-10-15 loop. Today a `logic-builder` or `asset-builder` works in a worktree `C:/dev/tactical-rpg-<task>` on branch `claude/<task>` branched from the local `staging` branch; a reviewed task lands on `staging` under a partial gate and `staging` is flushed to `develop` under one full gate (decision "Batched gate"); releases go `develop` -> `main` by PR. Steps: `docs/workflow/desktop.md` and the skill `.claude/skills/task-landing/SKILL.md`. The text below is kept unchanged.

1. **Brainstorm** (`design-brainstorm`): options for systems -> `games/emberwake/docs/ROADMAP.md` entries as `proposed`. User approves.
2. **Plan** (main session, skill `planner`): milestones + change lists; user confirms before any change.
3. **Build** (`builder` agent, feature branch `feat/<id>-<slug>`): implement + tests, report.
4. **Review/ship** (planner): review (+ `change-reviewer`), summary to user, README/docs, `check_project.ps1`, commit, merge to `main`, push.
5. **Balance** (`balance` agent, clean `main`, only `balance-ready` systems): `games/emberwake/docs/BALANCE_REPORT.md`.
6. **Discuss** (`design-brainstorm` + user): fixes or logic changes -> back to step 1. Dropped ideas go to `docs/HISTORY.md` with reasons.

Sims are parallel-safe (own sandbox per run); builders on separate systems use separate worktrees.

## Agent run log (append-only; the planner adds a row after each milestone)

Stopped on 2026-10-16 (note 2026-10-29, CLEANUP.md CLN-26): no rows were added after SKILL1. The record of finished tasks is `docs/archive/TASKS_DONE.md` plus `git log` (and the `milestone-status` skill per milestone); `docs/AGENT_MAP.md` keeps only the running agents. The rows below are kept unchanged.

Agent instances are fresh per call and idle once they report, so the agent list grows; this table is the readable record. Tokens and minutes are from the agent's own report.

| Milestone | Builder (tokens / min) | Reviewer (tokens / min) | Result |
| --- | --- | --- | --- |
| Roadmap brainstorm | general-purpose as brainstorm (56k / 1.5) | - | Proposed designs in ROADMAP |
| M1 combat extensions | 300k / 23 | 30k / 2.5 | merged `964ebb0` |
| M2a riposte | 128k / 9 | 33k / 2 | merged `dbedab2` |
| M2d gear budget | 66k / 8 | 24k / 2 | merged `3a68cb2` |
| M2b items and art | 68k / 4 | none (small data change) | merged `f654d78` |
| M2c-1 classes (+ skill draft by brainstorm 41k / 2) | 109k / 12 | 29k / 2 | merged `4db678c` |
| M2c-2a1 skill families | 126k / 7 | 37k / 2 | merged `17c852c` |
| M2c-2a2 engine pieces | 188k / 13 | 51k / 3 | merged `a23405f` |
| M2c-2b skill draft v2 and revision 3 (brainstorm) | - | - | draft only (51k / 3 and 47k / 2.5) |
| M2c-2 slice A foundation | 213k / 20 | 51k / 4 | merged (see git log: slice A foundation) |

Pattern: builders cost 65-300k tokens and 4-23 minutes (slices that touch many tests cost the most); reviewers cost 25-50k and about 2-3 minutes and have caught a real defect in most runs (stale status durations, rename hazard, unused-skill gaps).

## Parallel lanes (agreed 2026-10-16)

- Lanes: design (`design-brainstorm`), logic (`logic-builder`), asset (`asset-builder`), balance (`balance`), review (`change-reviewer`), cleanup (`housekeeper`: proposes trims, user confirms, builders implement); the main session plans and merges. Details and file ownership: skill `planner`; queue: `docs/TASKS.md`; contracts: `docs/specs/`.
- Parallel runs need separate git worktrees (one per concurrent builder, one read-only `main` worktree for balance); `tools/fc.sh` logs go to a per-worktree folder, sims use per-process sandboxes. The first Godot run in a new worktree imports the project (one-time delay).
- Schema-first handshake: the logic-builder merges the schema slice first; the asset-builder then fills data in parallel with the remaining logic work (different folders, clean merges).
- Builders end with a "docs delta" so the planner can update README, GAME_BRIEF, STRUCTURE and DECISIONS quickly.

| B1 rest to tactical (parallel pilot: logic-builder in a worktree + design-brainstorm writing C1/D1/M2C2B specs 131k / 7) | 47k / 10 | none (tiny change; fc.sh all re-run after rebase) | merged (see git log: B1) |

## Parallel safety (2026-10-16)

- Every task row in `docs/TASKS.md` and every spec lists its `Touches`; parallel only when they do not overlap. No hard builder cap (user 2026-10-17); above 3 at once the planner warns the user and asks whether to prioritise or slow down.
- `tools/overlap.sh` lists files edited in more than one active worktree (exit 1 when any). First run (C1-L vs D1-L): 8 shared scripts (`skill_def.gd`, `combatant.gd`, `content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `plan_entry.gd`, `status_effects.gd`, `dojo_session.gd`) plus `items.json` for C1-A1 vs C1-L: the "independent" judgement was optimistic, so expect small merge conflicts there. Lesson: shared hub files make most logic tasks overlap; sequence tasks that both change schema/validation, or merge the schema slices first.

| C1 orb/tome (parallel: C1-L, D1-L, C1-A1) | logic 155k / 14, asset 23k / 2 | 42k / 3 | merged |
| D1 weaken/riposte | logic 154k / 23 | 35k / 6 | merged |

Note (2026-10-16): the first three-way parallel run merged cleanly except for `items.json` (resolved by re-running the idempotent asset seed after taking main's file) and three small hunks in `plan_entry.gd`, `skill_def.gd`, `dojo_session.gd` (C1/D1 hooks overlapped, as `tools/overlap.sh` predicted). A worktree needs an editor class-cache scan after a rebase that brings in new `class_name` files.

| M2C2B skill content (asset 92k / 8 + test fix logic 75k / 6) | asset-builder then logic-builder | 56k / 1.5 | merged |
| AI1 (logic 205k / 20 incl. one resumed fix) | logic-builder | 36k / 1 | merged |

Throughput note (2026-10-16): R1, M2C2B-L, M3, AI1 and M2C2B ran as 3 parallel lanes; the planner's own merge work (rebase, registry conflicts, docs delta, cross-checks) is now the main serial cost. Parallel full-suite runs collide on fixed `user://` sandbox names (content tests): run the final suites one branch at a time, or make the sandbox names unique per process (follow-up task).

| BAL1 balance rerun | balance 66k / 22 | none (docs only) | report committed `80eeaf0` |
| SKILL1-L | logic-builder stopped by user (usage limit) mid-run | - | paused, partial work in ../ngp-skill1 |
| SKILL1 Forge skill tab | logic 105k / 2 (resume) + fixes 140k / 14, asset 111k / 11 | 83k / 3.5 | merged `611b667` |
