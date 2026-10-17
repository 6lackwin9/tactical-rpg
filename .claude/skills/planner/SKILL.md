---
name: planner
description: The milestone and lane workflow run by the MAIN session (not an agent). Use when the user asks to plan, start, build or ship roadmap work. Turns approved specs into tasks in docs/TASKS.md, dispatches the logic-builder, asset-builder, design-brainstorm and balance agents in parallel lanes (separate git worktrees), reviews, merges, updates docs and keeps agents busy. Load `planner-ops` too (front desk, alerts, dispatch modes).
---

# Planner workflow (main session)

You are the planner and orchestrator. Agents cannot message each other; everything flows through you, with files as handoffs: `docs/TASKS.md` (queue), `docs/specs/<id>.md` (contracts), `docs/ROADMAP.md`, `docs/STRUCTURE.md`, the decisions files (per CLAUDE.md), `docs/HISTORY.md`, `docs/BALANCE_REPORT.md`. Front desk, alerts and dispatch modes: skill `planner-ops`.

## Lanes (segregation of duties)

| Lane | Agent | Owns | Never touches |
| --- | --- | --- | --- |
| plan | main session | the only writer of `docs/TASKS.md`, `docs/ROADMAP.md`, `docs/decisions/*`, `docs/HISTORY.md`, README, GAME_BRIEF, STRUCTURE | builder lanes' files |
| design | `design-brainstorm` | new or updated spec files under `docs/specs/` (and archive moves it is asked for); lists wanted TASKS/ROADMAP/decision rows in its report | TASKS, ROADMAP, decisions, code, data |
| logic | `logic-builder` | `scripts/**`, `tools/**`, schema/validation, tests | shipped `data/**` |
| asset | `asset-builder` | `data/catalog/**`, `data/worlds/**`, `data/seeds/<task>.gd` via the catalogs' `upsert` | scripts, tests, docs |
| balance | `balance` | `docs/BALANCE_REPORT.md`, sims on a clean `main` worktree | everything else |
| review | `change-reviewer` | read-only | - |
| cleanup | `housekeeper` | `docs/specs/CLEANUP.md` | TASKS, ROADMAP, HISTORY, code, data, tests, tools |

Docs written in the main checkout by a design or cleanup agent are committed by the planner right after that run (before any merge), so `main` never carries another lane's half-finished edits. If the custom agent types are not yet callable (agent files added this session), run the same instructions through a general-purpose agent.

## 1. Plan (no code changes)

1. Read `docs/TASKS.md`, `docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/HISTORY.md` (do not propose deprecated concepts), the decisions files (per CLAUDE.md). Do not crawl code.
2. Pick tasks the user marked `approved`. A task needs a spec in `docs/specs/`: if it has none, dispatch `design-brainstorm` in the background to write it (it also drafts specs for the NEXT tasks while builders work).
3. Present the task list, lanes, dependencies and parallel plan to the user. **Make no changes until they confirm.** Design questions go to the user.

## 2. Dispatch (parallel lanes)

1. Create worktrees for parallel lanes from clean `main` (`git worktree add ../<repo>-<id> -b feat/<id>-<slug>`); use one worktree per concurrent builder and one read-only `main` worktree for the balance agent. `git status` must be clean first. Give each agent its worktree path, branch and spec.
2. Schema-first handshake: for a task with new schema, run the `logic-builder` on the schema slice first as its OWN run (brief: "slice 1 only, end the run with the report"), so the asset lane starts minutes, not a whole build, later; review and merge it quickly (small), then start the `asset-builder` (branching from the updated `main`) in parallel with the logic-builder's remaining behaviour work. They edit different folders, so merges stay clean. Merge order: schema slice, then logic, then data (or data first if its tests are generic); run the full check after each merge.
3. Parallel limits: no hard cap on builders (logic + asset). Soft warning at more than 3 at once: tell the user how many would run, the merge/review load it creates (rebases, hub-file conflicts, planner serial merge time, usage limits, CPU contention) and whether to prioritise or slow down; tasks run in parallel only when their `Touches` (docs/TASKS.md) do not overlap, otherwise sequence them (merge the smaller or more foundational one first). Run `tools/overlap.sh` while worktrees are active and before every merge. The shared hub files (`content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `skill_def.gd`, `combatant.gd`, `suite_registry.gd`; item/class schema edits land in `item_fields.gd` / `class_unit_fields.gd` and validators in `scripts/content/validators/`) are the usual conflict points, so give two logic tasks that both need them a merge order up front.
4. Keep agents busy: while builders run, `design-brainstorm` writes the next specs and the `balance` agent runs on frozen systems; reviewers run in the background on finished branches while the next task starts. Standing auto-dispatches (no user decision involved, docs only): a new balance report -> `design-brainstorm` options right away; a finished builder -> `change-reviewer` right away, while the planner drafts the docs delta in parallel.
5. Before writing a brief, check it against the spec and the decisions files (a brief narrower than a recorded user decision cost a rerun in SKILL1-A). Briefs name the worktree, branch, slice, gate command and "do not commit". Pass `effort` (and `model` where named) per the active dispatch mode (`planner-ops`).
6. Mark tasks `in-progress` in `docs/TASKS.md`. Move rows to `docs/archive/TASKS_DONE.md` once done (keeps the queue small).

## 3. Review and ship

1. Review each report; run `change-reviewer`, which runs the suites (don't run them a second time yourself unless you changed something after the review). If a builder failed the same thing twice, ask the user rather than retry. Review fixes go to the SAME builder via `SendMessage` while it is resumable (it keeps its context); spawn a fresh one only when it was stopped or failed, and point it at `.agent/progress.md`. If the planner itself changes code or data after a builder's gate (flag flip, seed re-run), re-run the gate before the review.
2. Apply the builders' "docs delta" (README, GAME_BRIEF, STRUCTURE, new decision rows go to the matching `docs/decisions/<system>.md`; HISTORY append-only), update `docs/ROADMAP.md` and `docs/TASKS.md` statuses.
3. Update `docs/AGENT_MAP.md` (Active section on dispatch; finished agents into their milestone section, newest first) and append a row to the "Agent run log" in `docs/WORKFLOW.md` for milestones.
4. Run `check_project.ps1 -Fast -Quiet` and `tools/fc.sh docs` (markdown lint; the IDE panel only lints open files; never pipe it into `tail` before `&& git commit`). If clean: commit with the attribution trailer from the session's system reminder, merge to `main` (fast-forward), push, remove the worktree and branch with `powershell -NoProfile -File tools/wt_remove.ps1 ../ngp-<id> feat/<branch>`.
5. If the system is now stable, ask the user whether to mark it `balance-ready`.

## Cleanup loop

`housekeeper` (after a milestone that replaced an older idea, when `fc.sh docs` warns, or on request) -> `docs/specs/CLEANUP.md` candidates -> planner shows the user -> user confirms/rejects -> the planner itself adds the confirmed rows to `docs/TASKS.md` (and ROADMAP for large ones), marks rejected ones in CLEANUP.md and appends dropped ideas to HISTORY -> builders pick them up as separate tasks (never folded into feature work).

## Balance loop

`balance` agent (own `main` worktree) -> `docs/BALANCE_REPORT.md` -> `design-brainstorm` (options/spec) -> user decision -> new task in `docs/TASKS.md` and the same lanes.

## Rules

- Never skip the confirmation gate. Never commit on red. Never develop on `main`; `main` is always a clean build.
- Parallel work only where file ownership does not overlap (see lane table); never two builders editing the same files.
- Log everything in `docs/TASKS.md` so the state survives a restart.
