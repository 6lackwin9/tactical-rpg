# Workflow: build, test, iterate

## Bottlenecks seen so far and the fix

| Bottleneck | Fix |
| --- | --- |
| Re-reading the codebase for context | `docs/GAME_BRIEF.md`, `ROADMAP.md`, `DECISIONS.md` (indexed from CLAUDE.md) |
| Full check takes minutes and floods output | `tools/fc.sh`: `parse` (changed files only), `arena`/`dojo` (failures first), `run <script>` (scratch debug) |
| Failures re-debugged from scratch | `godot-debug-checklist` skill known-issues table; add every new cause |
| Ad-hoc patch scripts truncating files / data edits by hand | `patch-and-seed` skill: Edit tool for text, `ContentCatalog.upsert` scripts for data |
| Stale IDE diagnostics after new `class_name` files | `tools/fc.sh parse` refreshes the class cache automatically |
| Test numbers hard-coded, break on every data change | Derive expectations from rules/formulas; print real values first |
| Guessing at design/balance | Ask the user (options + trade-offs) after 2 failed fixes or any design choice |
| Markdown lint warnings (MD060, MD022, MD032, MD012) | Pad table pipes (`\| --- \|`), single blank lines around headings/lists, no consecutive blanks |

## Loop for a change

1. Read the brief; look up the relevant system in "Where to look for X".
2. Edit (Edit tool; data via `ContentCatalog.upsert` seed script run with `tools/fc.sh run`).
3. `tools/fc.sh parse`, then `tools/fc.sh arena` (or `dojo`); fix from the compact failure list.
4. Before commit: `check_project.ps1` clean, README + docs updated, IDE Problems tab 0 errors/warnings (markdownlint compliant).

## Agents and tools (built)

(Skills: godot-debug-checklist, patch-and-seed, planner. Agents: design-brainstorm, builder, balance, change-reviewer.)

| Tool | Use |
| --- | --- |
| `tools/fc.sh parse / arena / dojo / run / sim / all` | Compact checks and scratch scripts (see header of the script) |
| `tools/balance_sim.gd` (via `fc.sh sim duel\|tier\|levels [level]`) | Headless AI-vs-AI win-rate tables, sides swapped, 3 seeds. Duel at one level takes about 90 s |
| Agent `design-brainstorm` | Options + trade-offs + recommendation from the brief and sim numbers; never edits |
| Agent `change-reviewer` | Pre-commit review: fast checks, strict typing, docs/README, known-issues match; read-only |
| Skills `godot-debug-checklist`, `patch-and-seed` | Known issues and safe editing |

Balance baseline (level 5, 2026-10-15): warrior wins every duel (100%), berserker 80%, brawler 53%, scout 47%, ranger 20%, shaman 0% (cannot beat a minion of itself: stalemates). Hero loses 1v3 minions and 1v1 bosses; elite = mirror (50%). These are open balance questions, not decisions.

## Iteration loop (agreed 2026-10-15)

1. **Brainstorm** (`design-brainstorm`): options for systems -> `docs/ROADMAP.md` entries as `proposed`. User approves.
2. **Plan** (main session, skill `planner`): milestones + change lists; user confirms before any change.
3. **Build** (`builder` agent, feature branch `feat/<id>-<slug>`): implement + tests, report.
4. **Review/ship** (planner): review (+ `change-reviewer`), summary to user, README/docs, `check_project.ps1`, commit, merge to `main`, push.
5. **Balance** (`balance` agent, clean `main`, only `balance-ready` systems): `docs/BALANCE_REPORT.md`.
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
