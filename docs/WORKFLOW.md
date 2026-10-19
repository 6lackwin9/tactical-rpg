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
| Test numbers hard-coded, break on every data change | Derive expectations from rules or the Godot reference; print real values first (`aegis battle ... log`, `aegis serve`) |
| A rule change silently drifting from the released game | The Godot parity suites replay the frozen Godot build step for step; an intended divergence retires the check it breaks (CLAUDE.md "Godot parity") |
| Guessing at design/balance | Ask the user (options + trade-offs) after 2 failed fixes or any design choice |
| Markdown lint warnings (MD060, MD022, MD032, MD012) | Pad table pipes (`\| --- \|`), single blank lines around headings/lists, no consecutive blanks |

## Loop for a change

1. Read the brief; look up the relevant system in "Where to look for X".
2. Edit (Edit tool; data via `aegis content`, skill `patch-and-seed`).
3. `tools/fc.sh build`, then `tools/fc.sh test <filter>`; fix from the compact failure list.
4. Before commit: `check_project.ps1 -Fast -Quiet` (or `tools/fc.sh gate`) and `tools/fc.sh docs` clean, READMEs + docs updated; on desktop also IDE Problems tab 0 errors/warnings (markdownlint compliant). Desktop vs cloud steps: `docs/workflow/`.

## Agents and tools (built)

(Skills: engine-checklist, known-issues, rust-conventions, patch-and-seed, planner, planner-ops, pc-launch, patch-notes-write. Agents: design-brainstorm, logic-builder, asset-builder, balance, change-reviewer, housekeeper.)

| Tool | Use |
| --- | --- |
| `tools/fc.sh build / test / gate / clippy / docs / content / sim / parity` | Compact checks, data edits, the sim and the Godot reference (see the header of the script); `gate` is the commit gate in bash, `check_project.ps1` the same in PowerShell |
| `aegis` (shell `engine/crates/aegis_cli`, binary in `games/emberwake/crates/emberwake_host`; `cargo run -q -p emberwake_host -- ...` from the repo root) | `sim duel [level] [seeds]` (balance table, about a second per level), `battle <encounter> [seed=N] [log]`, `serve` (the battle command API over stdin / stdout), `link` (drive a running `emberwake`), `content` (edit `games/emberwake/data/`) |
| `games/emberwake/parity/*.sh` (`tools/fc.sh parity`) | Record the frozen Godot build's reference from `legacy/` (exports, battle and world traces, journeys) for the parity suites; needs Godot 4.7.2 |
| Agent `design-brainstorm` | Options + trade-offs + recommendation from the brief and sim numbers; never edits |
| Agent `change-reviewer` | Pre-commit review: the gate, code patterns, docs/README, known-issues match; read-only |
| Skills `engine-checklist`, `known-issues`, `patch-and-seed` | Edit checklist, known issues and safe editing |

The Godot build's sim (`tools/balance_sim.gd`, modes duel, tier, levels, group, party, role, sweep, campaign) stays in `legacy/`; only `duel` is ported (`aegis sim duel`, step for step against it).

Balance baseline (level 5, 2026-10-15): warrior wins every duel (100%), berserker 80%, brawler 53%, scout 47%, ranger 20%, shaman 0% (cannot beat a minion of itself: stalemates). Hero loses 1v3 minions and 1v1 bosses; elite = mirror (50%). These are open balance questions, not decisions.

## Iteration loop (agreed 2026-10-15)

1. **Brainstorm** (`design-brainstorm`): options for systems -> `games/emberwake/docs/ROADMAP.md` entries as `proposed`. User approves.
2. **Plan** (main session, skill `planner`): milestones + change lists; user confirms before any change.
3. **Build** (`builder` agent, feature branch `feat/<id>-<slug>`): implement + tests, report.
4. **Review/ship** (planner): review (+ `change-reviewer`), summary to user, README/docs, `check_project.ps1`, commit, merge to `main`, push.
5. **Balance** (`balance` agent, clean `main`, only `balance-ready` systems): `games/emberwake/docs/BALANCE_REPORT.md`.
6. **Discuss** (`design-brainstorm` + user): fixes or logic changes -> back to step 1. Dropped ideas go to `docs/HISTORY.md` with reasons.

Sims are parallel-safe (own sandbox per run); builders on separate systems use separate worktrees.

## Agent run log (append-only; the planner adds a row after each milestone)

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
