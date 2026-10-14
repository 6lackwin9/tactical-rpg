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

## Planned (ask before building)

- Balance sim runner: headless AI-vs-AI over class x tier x level, reporting win rate and rounds-to-kill.
- Design/brainstorm agent (reads the brief, proposes options, asks) and review agent (typing + known-issues checks).
