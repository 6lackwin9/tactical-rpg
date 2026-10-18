# Tactical RPG (Godot 4.7, GDScript)

Environment: `$CLAUDE_CODE_REMOTE` empty = **desktop** (read `docs/workflow/desktop.md`), `true` = **cloud** (read `docs/workflow/cloud.md`). Both share everything in this file; what differs (sessions, branches, commits, merges, gate time, IDE rule) and how to switch: `docs/workflow/README.md`.

Read these instead of crawling the code:

- `docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/ROADMAP.md`: systems, statuses (proposed/approved/in-progress/done/balance-ready), milestone order
- `docs/STRUCTURE.md`: where code goes, including reserved placeholder folders
- `docs/DECISIONS.md`: index of settled decisions (do not re-litigate); rows live in `docs/decisions/<system>.md` (read only the system you work on), superseded rows in `docs/decisions/archive.md`
- `docs/preferences/`: Tushar's design, UI and asset preferences that are not formal decisions (read the one for your area)
- `docs/HISTORY.md`: deprecated/tried ideas with reasons (append-only; never re-propose without new reasons)
- `docs/BALANCE_REPORT.md`: latest balance runs and open issues
- `docs/TASKS.md`: task queue (lanes, dependencies, status); specs in `docs/specs/`
- `docs/AGENT_MAP.md`: dashboard of active agents on top, finished runs grouped by milestone (planner-maintained; cloud: written by the Merge & Release thread)
- `docs/WORKFLOW.md`: tools and how to iterate; `docs/workflow/`: desktop vs cloud
- `docs/EFFICIENCY.md`: measured slow steps, fixes and cheap-path rules (update it when something is slow or floods context)
- Skills `godot-debug-checklist` (short edit checklist), `godot-known-issues` (consult BEFORE debugging), `gdscript-typing` (typing patterns)

## Roles and lanes (agents cannot message each other; the planner routes via the docs above)

- **Planner** (skills `planner`, `planner-ops`; the writer of TASKS, ROADMAP, decisions and HISTORY): task queue (`docs/TASKS.md`), dispatch in lanes, user confirmation gate, review, commit/push, README/docs. Desktop: the main session for everything; cloud: standing area threads (Gameplay, UI/UX, Assets & Audio, Balance, Design, Platform & Workflow), each working its area's tasks one at a time on its own branch, squash-merged into `develop` without a PR; a Merge & Release thread batches `develop` into `main` with one PR; no new thread without asking the user first (the project chat is the front desk).
- `design-brainstorm` agent (design lane): options and specs in `docs/specs/`; lists wanted roadmap and task rows in its report (the planner writes them); works ahead while builders run.
- `logic-builder` agent (logic lane): engine, schema, validation, tests on the task's branch (desktop: a feature worktree); never edits shipped data; never commits (the planner or cloud thread does).
- `asset-builder` agent (asset lane): fills `data/` through `ContentCatalog.upsert` against a merged schema; no code, no tests.
- `balance` agent (balance lane): sims on a clean `main` checkout (desktop: its own worktree) on `balance-ready` systems; writes `docs/BALANCE_REPORT.md`.
- `change-reviewer` agent: read-only pre-commit review.
- `housekeeper` agent (cleanup lane): finds legacy/redundant code, dead paths and bloat; proposes trims in `docs/specs/CLEANUP.md`; the planner adds confirmed ones to `docs/TASKS.md`/`docs/ROADMAP.md`; never implements (builders do, as separate tasks).

## Standing rules

Strict static typing; README is updated with every change (builders report a docs delta, the planner applies it); commit/push only when `check_project.ps1 -Fast -Quiet` is clean (desktop ~32 s; cloud: `tools/fc.sh gate auto`, the per-system gate; the plain script takes ~3 min on desktop; `tools/fc.sh gate` runs the same steps without PowerShell) and `tools/fc.sh docs` is clean; on desktop also the IDE Problems tab has 0 errors/warnings (markdownlint compliant; see `.agents/rules/godot_workflow.md` Section 6); `main` is always a clean build (features on branches); edit content through `ContentCatalog.upsert`, never raw JSON; no sed line-range edits; docs history is append-only (mark deprecated with reason, never delete).

## Feedback rule

Look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. Add new recurring failure causes to the known-issues table.
