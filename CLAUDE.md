# Tactical RPG (Godot 4.7, GDScript)

Read these instead of crawling the code:

- `docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/ROADMAP.md`: systems, statuses (proposed/approved/in-progress/done/balance-ready), milestone order
- `docs/STRUCTURE.md`: where code goes, including reserved placeholder folders
- `docs/DECISIONS.md`: index of settled decisions (do not re-litigate); rows live in `docs/decisions/<system>.md` (read only the system you work on), superseded rows in `docs/decisions/archive.md`
- `docs/HISTORY.md`: deprecated/tried ideas with reasons (append-only; never re-propose without new reasons)
- `docs/BALANCE_REPORT.md`: latest balance runs and open issues
- `docs/TASKS.md`: task queue (lanes, dependencies, status); specs in `docs/specs/`
- `docs/AGENT_MAP.md`: dashboard of active agents on top, finished runs grouped by milestone (planner-maintained)
- `docs/WORKFLOW.md`: tools and how to iterate
- `docs/EFFICIENCY.md`: measured slow steps, fixes and cheap-path rules (update it when something is slow or floods context)
- Skills `godot-debug-checklist` (short edit checklist), `godot-known-issues` (consult BEFORE debugging), `gdscript-typing` (typing patterns)

## Roles and lanes (agents cannot message each other; the main session routes via the docs above)

- Main session = **planner** (skills `planner`, `planner-ops`; the only writer of TASKS, ROADMAP, decisions and HISTORY): task queue (`docs/TASKS.md`), dispatch in parallel lanes, user confirmation gate, review, commit/push, README/docs.
- `design-brainstorm` agent (design lane): options and specs in `docs/specs/`; lists wanted roadmap and task rows in its report (the planner writes them); works ahead while builders run.
- `logic-builder` agent (logic lane): engine, schema, validation, tests in a feature worktree; never edits shipped data; never commits.
- `asset-builder` agent (asset lane): fills `data/` through `ContentCatalog.upsert` against a merged schema; no code, no tests.
- `balance` agent (balance lane): sims in its own clean `main` worktree on `balance-ready` systems; writes `docs/BALANCE_REPORT.md`.
- `change-reviewer` agent: read-only pre-commit review.
- `housekeeper` agent (cleanup lane): finds legacy/redundant code, dead paths and bloat; proposes trims in `docs/specs/CLEANUP.md`; the planner adds confirmed ones to `docs/TASKS.md`/`docs/ROADMAP.md`; never implements (builders do, as separate tasks).

## Standing rules

Strict static typing; README is updated with every change (builders report a docs delta, the planner applies it); commit/push only when `check_project.ps1 -Fast -Quiet` is clean (32 s; the plain script takes ~3 min) and IDE Problems tab has 0 errors/warnings (markdownlint compliant; see `.agents/rules/godot_workflow.md` Section 6); `main` is always a clean build (features on branches); edit content through `ContentCatalog.upsert`, never raw JSON; no sed line-range edits; docs history is append-only (mark deprecated with reason, never delete).

## Feedback rule

Look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. Add new recurring failure causes to the known-issues table.
