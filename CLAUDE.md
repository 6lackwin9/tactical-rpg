# Tactical RPG (Godot 4.7, GDScript)

Read these instead of crawling the code:

- `docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/ROADMAP.md`: systems, statuses (proposed/approved/in-progress/done/balance-ready), milestone order
- `docs/STRUCTURE.md`: where code goes, including reserved placeholder folders
- `docs/DECISIONS.md`: settled decisions (do not re-litigate)
- `docs/HISTORY.md`: deprecated/tried ideas with reasons (append-only; never re-propose without new reasons)
- `docs/BALANCE_REPORT.md`: latest balance runs and open issues
- `docs/WORKFLOW.md`: tools and how to iterate
- Skill `godot-debug-checklist`: known issues table; consult BEFORE debugging

## Roles (agents cannot message each other; the main session routes via the docs above)

- Main session = **planner** (skill `planner`): milestones, change lists, user confirmation gate, review, commit/push, README/docs.
- `design-brainstorm` agent: options, roadmap proposals, balance fixes (writes only docs/).
- `builder` agent: implements an approved change list on a feature branch `feat/<id>-<slug>`; never commits.
- `balance` agent: sims on clean `main`, only `balance-ready` systems, writes `docs/BALANCE_REPORT.md`.
- `change-reviewer` agent: read-only pre-commit review.

## Standing rules

Strict static typing; update README on every change; commit/push only when `check_project.ps1` is clean and IDE Problems tab has 0 errors/warnings (markdownlint compliant; see `.agents/rules/godot_workflow.md` Section 6); `main` is always a clean build (features on branches); edit content through `ContentCatalog.upsert`, never raw JSON; no sed line-range edits; docs history is append-only (mark deprecated with reason, never delete).

## Feedback rule

Look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. Add new recurring failure causes to the known-issues table.
