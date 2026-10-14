# Workflow: build, test, iterate

## Bottlenecks seen so far and the fix

| Bottleneck | Fix |
|---|---|
| Re-reading the codebase for context | `docs/GAME_BRIEF.md`, `ROADMAP.md`, `DECISIONS.md` (indexed from CLAUDE.md) |
| Full check takes minutes and floods output | `tools/fc.sh`: `parse` (changed files only), `arena`/`dojo` (failures first), `run <script>` (scratch debug) |
| Failures re-debugged from scratch | `godot-debug-checklist` skill known-issues table; add every new cause |
| Ad-hoc patch scripts truncating files / data edits by hand | `patch-and-seed` skill: Edit tool for text, `ContentCatalog.upsert` scripts for data |
| Stale IDE diagnostics after new `class_name` files | `tools/fc.sh parse` refreshes the class cache automatically |
| Test numbers hard-coded, break on every data change | Derive expectations from rules/formulas; print real values first |
| Guessing at design/balance | Ask the user (options + trade-offs) after 2 failed fixes or any design choice |

## Loop for a change

1. Read the brief; look up the relevant system in "Where to look for X".
2. Edit (Edit tool; data via `ContentCatalog.upsert` seed script run with `tools/fc.sh run`).
3. `tools/fc.sh parse`, then `tools/fc.sh arena` (or `dojo`); fix from the compact failure list.
4. Before commit: `check_project.ps1` clean, README + docs updated.


## Agents and tools (built)

| Tool | Use |
|---|---|
| `tools/fc.sh parse / arena / dojo / run / sim / all` | Compact checks and scratch scripts (see header of the script) |
| `tools/balance_sim.gd` (via `fc.sh sim duel\|tier\|levels [level]`) | Headless AI-vs-AI win-rate tables, sides swapped, 3 seeds. Duel at one level takes about 90 s |
| Agent `design-brainstorm` | Options + trade-offs + recommendation from the brief and sim numbers; never edits |
| Agent `change-reviewer` | Pre-commit review: fast checks, strict typing, docs/README, known-issues match; read-only |
| Skills `godot-debug-checklist`, `patch-and-seed` | Known issues and safe editing |

Balance baseline (level 5, 2026-10-15): warrior wins every duel (100%), berserker 80%, brawler 53%, scout 47%, ranger 20%, shaman 0% (cannot beat a minion of itself: stalemates). Hero loses 1v3 minions and 1v1 bosses; elite = mirror (50%). These are open balance questions, not decisions.
