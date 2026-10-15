# Tactical RPG (Godot 4.7, GDScript)

Read these instead of crawling the code:

- `docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/ROADMAP.md`: systems, statuses (proposed/approved/in-progress/done/balance-ready), milestone order
- `docs/STRUCTURE.md`: where code goes, including reserved placeholder folders
- `docs/DECISIONS.md`: settled decisions (do not re-litigate)
- `docs/HISTORY.md`: deprecated/tried ideas with reasons (append-only; never re-propose without new reasons)
- `docs/BALANCE_REPORT.md`: latest balance runs and open issues
- `docs/TASKS.md`: task queue (lanes, dependencies, status); specs in `docs/specs/`
- `docs/AGENT_MAP.md`: dashboard of active agents on top, finished runs grouped by milestone (planner-maintained)
- `docs/WORKFLOW.md`: tools and how to iterate
- `docs/EFFICIENCY.md`: measured slow steps, fixes and cheap-path rules (update it when something is slow or floods context)
- Skill `godot-debug-checklist`: known issues table; consult BEFORE debugging

## Roles and lanes (agents cannot message each other; the main session routes via the docs above)

- Main session = **planner** (skill `planner`): task queue (`docs/TASKS.md`), dispatch in parallel lanes, user confirmation gate, review, commit/push, README/docs.
- `design-brainstorm` agent (design lane): options, specs in `docs/specs/`, roadmap proposals; writes only `docs/`; works ahead while builders run.
- `logic-builder` agent (logic lane): engine, schema, validation, tests in a feature worktree; never edits shipped data; never commits.
- `asset-builder` agent (asset lane): fills `data/` through `ContentCatalog.upsert` against a merged schema; no code, no tests.
- `balance` agent (balance lane): sims in its own clean `main` worktree on `balance-ready` systems; writes `docs/BALANCE_REPORT.md`.
- `change-reviewer` agent: read-only pre-commit review.

## Standing rules

Strict static typing; update README on every change; commit/push only when `check_project.ps1 -Fast -Quiet` is clean (32 s; the plain script takes ~3 min) and IDE Problems tab has 0 errors/warnings (markdownlint compliant; see `.agents/rules/godot_workflow.md` Section 6); `main` is always a clean build (features on branches); edit content through `ContentCatalog.upsert`, never raw JSON; no sed line-range edits; docs history is append-only (mark deprecated with reason, never delete).

## Feedback rule

Look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. Add new recurring failure causes to the known-issues table.
