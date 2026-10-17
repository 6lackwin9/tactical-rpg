---
name: planner
description: The milestone and lane workflow run by the MAIN session (not an agent). Use when the user asks to plan, start, build or ship roadmap work. Turns approved specs into tasks in docs/TASKS.md, dispatches the logic-builder, asset-builder, design-brainstorm and balance agents in parallel lanes (separate git worktrees), reviews, merges, updates docs and keeps agents busy.
---

# Planner workflow (main session)

You are the planner and orchestrator. Agents cannot message each other; everything flows through you, with files as handoffs: `docs/TASKS.md` (queue), `docs/specs/<id>.md` (contracts), `docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/DECISIONS.md`, `docs/HISTORY.md`, `docs/BALANCE_REPORT.md`.

## Lanes (segregation of duties)

| Lane | Agent | Owns | Never touches |
| --- | --- | --- | --- |
| design | `design-brainstorm` | `docs/` (specs, proposals, task rows with status `spec`) | code, data |
| logic | `logic-builder` | `scripts/**`, `tools/**`, schema/validation, tests | shipped `data/**` |
| asset | `asset-builder` | `data/catalog/**`, `data/worlds/**` via `ContentCatalog.upsert` | scripts, tests, docs |
| balance | `balance` | `docs/BALANCE_REPORT.md`, sims on a clean `main` worktree | everything else |
| review | `change-reviewer` | read-only | - |
| cleanup | `housekeeper` | `docs/specs/CLEANUP.md`; after user confirmation `docs/TASKS.md`, `docs/ROADMAP.md`, `docs/HISTORY.md` rows | code, data, tests, tools (builders implement confirmed cleanup tasks) |

If the custom agent types are not yet callable (agent files added this session), run the same instructions through a general-purpose agent.

## Dispatch modes (user 2026-10-17)

The user switches mode with "mode standard", "mode economy" or "mode sprint" (typed in the front-desk session `new-game-project-09`, relayed here). Ack with the active mode on read. The mode persists until changed and applies to every later dispatch; agents already running are untouched. Pass `effort` explicitly on every Agent call per the table (this table is the explicit instruction for setting effort) and `model` where the table names one. Current mode: **standard** (record changes in `docs/AGENT_MAP.md` Active section).

| Agent | standard (default, quality baseline) | economy (fewest tokens) | sprint (least wall-clock time) |
| --- | --- | --- | --- |
| design-brainstorm | high | medium | medium |
| logic-builder | high for new systems, medium for fixes | medium; model sonnet for fixes and small tasks, opus only for new engine systems | medium |
| change-reviewer | medium | low, only at commit gates | medium |
| balance | medium (sonnet) | low (sonnet) | low (sonnet) |
| asset-builder | low (sonnet) | low (sonnet) | low (sonnet) |
| housekeeper | medium | low, only when asked | skip |

- Economy rules: run lanes serially unless tasks are independent and small; resume an existing agent via SendMessage instead of spawning a fresh one; give agents exact file paths and spec sections; no design-ahead work unless the user asks.
- Sprint rules: maximum parallel lanes in separate worktrees (soft warning above 3 still applies: CPU contention, see EFFICIENCY); design-ahead always on; `check_project.ps1 -Fast -Quiet` per task and the full check only before a milestone merge; batch review fixes into one builder pass; skip balance runs until the user asks.
- All modes keep the standing gates: user confirmation for design/gameplay decisions, change-reviewer before each commit, `check_project.ps1 -Fast -Quiet` clean, IDE Problems 0, README/docs updated. Modes change only effort, model and parallelism. This session's own model and effort are set by the user (`/model`, `/effort`).

## 1. Plan (no code changes)

1. Read `docs/TASKS.md`, `docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/HISTORY.md` (do not propose deprecated concepts), `docs/DECISIONS.md` (index) + `docs/decisions/<system>.md` (read only the targeted system; superseded rows in `docs/decisions/archive.md`). Do not crawl code.
2. Pick tasks the user marked `approved`. A task needs a spec in `docs/specs/`: if it has none, dispatch `design-brainstorm` in the background to write it (it also drafts specs for the NEXT tasks while builders work).
3. Present the task list, lanes, dependencies and parallel plan to the user. **Make no changes until they confirm.** Design questions go to the user.

## 2. Dispatch (parallel lanes)

1. Create worktrees for parallel lanes from clean `main` (`git worktree add ../<repo>-<id> -b feat/<id>-<slug>`); use one worktree per concurrent builder and one read-only `main` worktree for the balance agent. `git status` must be clean first. Give each agent its worktree path, branch and spec.
2. Schema-first handshake: for a task with new schema, run the `logic-builder` on the schema slice first as its OWN run (brief: "slice 1 only, end the run with the report"), so the asset lane starts minutes, not a whole build, later; review and merge it quickly (small), then start the `asset-builder` (branching from the updated `main`) in parallel with the logic-builder's remaining behaviour work. They edit different folders, so merges stay clean. Merge order: schema slice, then logic, then data (or data first if its tests are generic); run the full check after each merge.
3. Parallel limits: no hard cap on builders (logic + asset). Soft warning at more than 3 at once: tell the user how many would run, the merge/review load it creates (rebases, hub-file conflicts, planner serial merge time, usage limits) and whether to prioritise or slow down; tasks run in parallel only when their `Touches` (docs/TASKS.md) do not overlap, otherwise sequence them (merge the smaller or more foundational one first). Run `tools/overlap.sh` while worktrees are active and before every merge: it lists files edited in more than one worktree. The shared hub files (`content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `skill_def.gd`, `combatant.gd`, `suite_registry.gd`; item/class schema edits now land in `item_fields.gd` / `class_unit_fields.gd` and validators in `scripts/content/validators/`) are the usual conflict points, so give two logic tasks that both need them a merge order up front.
4. Keep agents busy: while builders run, `design-brainstorm` writes the next specs and the `balance` agent runs on frozen systems; reviewers run in the background on finished branches while the next task starts. Independent tasks with no shared files run in separate worktrees at the same time. Standing auto-dispatches (no user decision involved, docs only): a new balance report -> `design-brainstorm` options right away; a finished builder -> `change-reviewer` right away, while the planner drafts the docs delta in parallel.
5. Before writing a brief, check it against the spec and `docs/DECISIONS.md` (index) + `docs/decisions/<system>.md` (read only the targeted system; superseded rows in `docs/decisions/archive.md`) (a brief narrower than a recorded user decision cost a rerun in SKILL1-A). Briefs name the worktree, branch, slice, gate command and "do not commit".
6. Mark tasks `in-progress` in `docs/TASKS.md`. Move rows to `docs/archive/TASKS_DONE.md` once done (keeps the queue small).

## 3. Review and ship

1. Review each report; run `change-reviewer` (and `tools/fc.sh all`). If a builder failed the same thing twice, ask the user rather than retry. Review fixes go to the SAME builder via `SendMessage` while it is resumable (it keeps its context); spawn a fresh one only when it was stopped or failed, and point it at `.agent/progress.md`. If the planner itself changes code or data after a builder's gate (flag flip, seed re-run), re-run the gate before the review.
2. Apply the builders' "docs delta" (README, GAME_BRIEF, STRUCTURE, new decision rows go to the matching `docs/decisions/<system>.md`; HISTORY append-only), update `docs/ROADMAP.md` and `docs/TASKS.md` statuses.
3. Record cost: note the model passed to each dispatch (Agent `model` parameter or "inherited") and the main-session token counter delta in the "Cost tracking" table of `docs/AGENT_MAP.md`. Append a row to the "Agent run log" in `docs/WORKFLOW.md` and update `docs/AGENT_MAP.md` (move the finished agent from Active into its milestone section, creating the section at the TOP of the finished list: newest first; add new dispatches to Active when you launch them).
4. Run `check_project.ps1 -Fast -Quiet` and `tools/fc.sh docs` (markdown lint; the IDE panel only lints open files). If clean: commit with the attribution trailer from the session's system reminder (it follows the current model), merge to `main` (fast-forward), push, remove the worktree and branch with `powershell -NoProfile -File tools/wt_remove.ps1 ../ngp-<id> feat/<branch>` (plain `git worktree remove` fails on OneDrive).
5. If the system is now stable, ask the user whether to mark it `balance-ready`.

## Cleanup loop

`housekeeper` (after a milestone that replaced an older idea, when `fc.sh docs` warns, or on request) -> `docs/specs/CLEANUP.md` candidates -> planner shows the user -> user confirms/rejects -> planner resumes the housekeeper to add confirmed rows to `docs/TASKS.md` (and ROADMAP for large ones) -> builders pick them up as separate tasks (never folded into feature work).

## Balance loop

`balance` agent (own `main` worktree) -> `docs/BALANCE_REPORT.md` -> `design-brainstorm` (options/spec) -> user decision -> new task in `docs/TASKS.md` and the same lanes.

## Rules

- Never skip the confirmation gate. Never commit on red. Never develop on `main`; `main` is always a clean build.
- Parallel work only where file ownership does not overlap (see lane table); never two builders editing the same files.
- Log everything in `docs/TASKS.md` so the state survives a restart.
