# Tactical RPG (Aegis engine, Rust)

The game runs on Aegis, its own Rust engine in `engine/` (since 2026-10-19). The Godot 4.7 build it replaced is frozen in
`legacy/` (`legacy/LEGACY.md`): never edit it; it is the parity reference the engine is checked against, and the `legacy`
branch resumes Godot work only if the user asks.

Environment: `$CLAUDE_CODE_REMOTE` empty = **desktop** (read `docs/workflow/desktop.md`), `true` = **cloud** (read `docs/workflow/cloud.md`). Both share everything in this file; what differs (sessions, branches, commits, merges, gate time, IDE rule) and how to switch: `docs/workflow/README.md`.

Read these instead of crawling the code:

- `docs/GAME_BRIEF.md`: layers, core rules/numbers, where to look for X
- `docs/ROADMAP.md`: systems, statuses (proposed/approved/in-progress/done/balance-ready), milestone order
- `docs/STRUCTURE.md`: where code goes (engine crates, data, tools)
- `docs/DECISIONS.md`: index of settled decisions (do not re-litigate); rows live in `docs/decisions/<system>.md` (read only the system you work on), superseded rows in `docs/decisions/archive.md`
- `docs/preferences/`: Tushar's design, UI and asset preferences that are not formal decisions (read the one for your area)
- `docs/HISTORY.md`: deprecated/tried ideas with reasons (append-only; never re-propose without new reasons)
- `docs/BALANCE_REPORT.md`: latest balance runs and open issues
- `docs/TASKS.md`: task queue (lanes, dependencies, status); specs in `docs/specs/`
- `docs/AGENT_MAP.md`: dashboard of active agents on top, finished runs grouped by milestone (planner-maintained; cloud: written by the Merge & Release thread)
- `docs/WORKFLOW.md`: tools and how to iterate; `docs/workflow/`: desktop vs cloud
- `docs/EFFICIENCY.md`: measured slow steps, fixes and cheap-path rules (update it when something is slow or floods context)
- `engine/README.md` (engine layout, CLI, GUI, agent link), `engine/docs/PORT_NOTES.md` (which Godot file each engine module ports, parity status)
- Skills `engine-checklist` (short edit checklist), `known-issues` (consult BEFORE debugging), `rust-conventions` (engine code patterns)

## Roles and lanes (agents cannot message each other; the planner routes via the docs above)

- **Planner** (skills `planner`, `planner-ops`; the writer of TASKS, ROADMAP, decisions and HISTORY): task queue (`docs/TASKS.md`), dispatch in lanes, user confirmation gate, review, commit/push, README/docs. Desktop: the main session for everything; cloud: standing area threads (Gameplay, UI/UX, Assets & Audio, Balance, Design, Platform & Workflow), each working its area's tasks one at a time on its own branch, squash-merged into `develop` without a PR; a Merge & Release thread batches `develop` into `main` with one PR; no new thread without asking the user first (the project chat is the front desk).
- `design-brainstorm` agent (design lane): options and specs in `docs/specs/`; lists wanted roadmap and task rows in its report (the planner writes them); works ahead while builders run.
- `logic-builder` agent (logic lane): engine code (`engine/crates/**`), schema, validation, tests and tools on the task's branch (desktop: a feature worktree); never edits shipped data; never commits (the planner or cloud thread does).
- `asset-builder` agent (asset lane): fills `games/emberwake/data/` through `aegis content` (upsert / remove / apply a seed) against a merged schema; no code, no tests.
- `balance` agent (balance lane): sims on a clean `main` checkout (desktop: its own worktree) on `balance-ready` systems; writes `docs/BALANCE_REPORT.md`.
- `change-reviewer` agent: read-only pre-commit review.
- `housekeeper` agent (cleanup lane): finds legacy/redundant code, dead paths and bloat; proposes trims in `docs/specs/CLEANUP.md`; the planner adds confirmed ones to `docs/TASKS.md`/`docs/ROADMAP.md`; never implements (builders do, as separate tasks).

## Standing rules

Strict static typing (Rust; the build has no compiler warnings, clippy is advisory); README is updated with every change (builders report a docs delta, the planner applies it); commit/push only when `check_project.ps1 -Fast -Quiet` (desktop) or `tools/fc.sh gate` (cloud; same steps: build every crate, run every test) is clean and `tools/fc.sh docs` is clean; on desktop also the IDE Problems tab has 0 errors/warnings (markdownlint compliant; see `.agents/rules/engine_workflow.md` Section 6); `main` is always a clean build (features on branches); edit content only through `aegis content` (`tools/fc.sh content`), never raw JSON; never edit `legacy/`; no sed line-range edits; docs history is append-only (mark deprecated with reason, never delete).

Godot parity (user 2026-10-19, a benchmark): the engine's parity suites replay the frozen Godot build step for step (battle traces, world battles, journeys, saves, the data loader and writer). They measure the engine against Godot, show what is missing, and keep the design work reusable without Godot; they are not a contract. An unintended divergence is a bug (fix the engine); an intended design change retires the parity checks it affects in the same change (named in the commit and a `docs/HISTORY.md` row). Never edit recorded traces or expectations to make a parity test pass.

## Feedback rule

Look up the failure in the known-issues table first; after two failed targeted fixes, stop and ask the user (state findings + options). Always ask before design/gameplay decisions. Add new recurring failure causes to the known-issues table.
