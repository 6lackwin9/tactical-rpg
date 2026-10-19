---
name: planner
description: The milestone and lane workflow run by the planner (desktop main session, or a cloud area thread for its area's tasks). Use when the user asks to plan, start, build or ship roadmap work. Turns approved specs into tasks in docs/TASKS.md, dispatches the logic-builder, asset-builder, design-brainstorm and balance agents in lanes, reviews, merges, updates docs and keeps agents busy. Load `planner-ops` (dispatch modes) and the environment file docs/workflow/desktop.md or docs/workflow/cloud.md.
---

# Planner workflow (shared by both environments)

Environment first: `$CLAUDE_CODE_REMOTE` empty = desktop, read `docs/workflow/desktop.md`; `true` = cloud, read `docs/workflow/cloud.md`. That file says how sessions, the front desk, alerts, branches, commits and merges work there; this skill holds what both share. On desktop you plan everything; in the cloud a standing area thread plans its area's tasks, one at a time with one branch each, landing on `develop` without a PR (a Merge & Release thread batches `develop` into `main`), and never opens a new thread without asking the user first (`docs/workflow/cloud.md` "Area threads").

You are the planner and orchestrator. Agents cannot message each other; everything flows through you, with files as handoffs: `docs/TASKS.md` (queue), `docs/specs/<id>.md` (contracts), `games/emberwake/docs/ROADMAP.md`, `docs/STRUCTURE.md`, the decisions files (per CLAUDE.md), `docs/HISTORY.md`, `games/emberwake/docs/BALANCE_REPORT.md`. Dispatch modes: skill `planner-ops`; front desk and alerts: the environment file.

## Lanes (segregation of duties)

| Lane | Agent | Owns | Never touches |
| --- | --- | --- | --- |
| plan | planner (desktop main session; cloud: each area thread for its own tasks' rows) | the only writer of `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `docs/decisions/*`, `docs/HISTORY.md`, README, GAME_BRIEF, STRUCTURE | builder lanes' files |
| design | `design-brainstorm` | new or updated spec files under `docs/specs/` (and archive moves it is asked for); lists wanted TASKS/ROADMAP/decision rows in its report | TASKS, ROADMAP, decisions, code, data |
| logic | `logic-builder` | `engine/crates/**` (rules, schema, loader and writer, validation, tests), `games/emberwake/**`, `tools/**` | shipped `games/emberwake/data/**`, `legacy/**`, the recorded Godot reference (`games/emberwake/parity/**`) |
| asset | `asset-builder` | `games/emberwake/data/**` through `aegis content` (upsert, remove, `games/emberwake/data/seeds/<task>.json` with `apply`) | engine code, tests, docs |
| balance | `balance` | `games/emberwake/docs/BALANCE_REPORT.md`, sims on a clean `main` worktree | everything else |
| review | `change-reviewer` | read-only | - |
| cleanup | `housekeeper` | `docs/specs/CLEANUP.md` | TASKS, ROADMAP, HISTORY, code, data, tests, tools |

Docs written in the main checkout by a design or cleanup agent are committed by the planner right after that run (before any merge), so `main` never carries another lane's half-finished edits. If the custom agent types are not yet callable (agent files added this session), run the same instructions through a general-purpose agent; this is the only exception to the cloud rule that lane work uses the six repo agents (`docs/workflow/cloud.md` "Lanes inside an area thread").

## 1. Plan (no code changes)

1. Read `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/HISTORY.md` (do not propose deprecated concepts), the decisions files (per CLAUDE.md). Do not crawl code.
2. Pick tasks the user marked `approved`. A task needs a spec in `docs/specs/`: if it has none, dispatch `design-brainstorm` in the background to write it (it also drafts specs for the NEXT tasks while builders work).
3. Present the task list, lanes, dependencies and parallel plan to the user. **Make no changes until they confirm.** Design questions go to the user.

## 2. Dispatch (parallel lanes)

1. Set up the branch per the environment file (desktop: one worktree per concurrent builder plus a read-only `main` worktree for balance; cloud: a fresh branch per task from the latest `develop` in the area thread's checkout, subagents one after another). `git status` must be clean first. Give each agent its path, branch and spec.
2. Schema-first handshake: for a task with new schema, run the `logic-builder` on the schema slice first as its OWN run (brief: "slice 1 only, end the run with the report"), so the asset lane starts minutes, not a whole build, later; review and merge it quickly (small), then start the `asset-builder` (branching from the updated `main`) in parallel with the logic-builder's remaining behaviour work. They edit different folders, so merges stay clean. Merge order: schema slice, then logic, then data (or data first if its tests are generic); run the full check after each merge.
3. Parallel limits: no hard cap on builders (logic + asset). Soft warning at more than 3 at once: tell the user how many would run, the merge/review load it creates (rebases, hub-file conflicts, planner serial merge time, usage limits, CPU contention) and whether to prioritise or slow down; tasks run in parallel only when their `Touches` (docs/TASKS.md) do not overlap, otherwise sequence them (merge the smaller or more foundational one first). Check overlap per the environment file (desktop: `tools/overlap.sh`; cloud: other areas' unmerged task branches). The shared hub files (`games/emberwake/crates/emberwake_battle/src/{session,plan,command,ai}.rs`, `emberwake_world/src/{source,store,data}.rs`, `emberwake_data/src/lib.rs`, `emberwake_gui/src/battle.rs`, and the field tables in `games/emberwake/schema.json`) are the usual conflict points, so give two logic tasks that both need them a merge order up front.
4. Keep agents busy: while builders run, `design-brainstorm` writes the next specs and the `balance` agent runs on frozen systems; reviewers run in the background on finished branches while the next task starts. Standing auto-dispatches (no user decision involved, docs only): a new balance report -> `design-brainstorm` options right away; a finished builder -> `change-reviewer` right away, while the planner drafts the docs delta in parallel.
5. Before writing a brief, check it against the spec and the decisions files (a brief narrower than a recorded user decision cost a rerun in SKILL1-A). Briefs name the worktree or checkout, branch, slice, gate command and "do not commit" (builders never commit in either environment; on desktop the planner commits, in the cloud the area thread does). Pass `effort` (and `model` where named) per the active dispatch mode (`planner-ops`).
6. Mark tasks `in-progress` in `docs/TASKS.md`. Move rows to `docs/archive/TASKS_DONE.md` once done (keeps the queue small; cloud: as a `Done:` line in the task commit, written by the Merge & Release thread).

## 3. Review and ship

1. Review each report; run `change-reviewer`, which runs the suites (don't run them a second time yourself unless you changed something after the review). If a builder failed the same thing twice, ask the user rather than retry. Review fixes go to the SAME builder via `SendMessage` while it is resumable (it keeps its context); spawn a fresh one only when it was stopped or failed, and point it at `.agent/progress.md` (in the cloud also at the last pushed commit). If the planner itself changes code or data after a builder's gate (flag flip, seed re-run), re-run the gate before the review.
2. Apply the builders' "docs delta" (README, GAME_BRIEF, STRUCTURE, new decision rows go to the matching decisions file: `games/emberwake/docs/decisions/<system>.md`, or `docs/decisions/workflow.md` for process; HISTORY append-only), update `games/emberwake/docs/ROADMAP.md` and `docs/TASKS.md` statuses.
3. Update `docs/AGENT_MAP.md` (desktop: Active section on dispatch and finished agents into their milestone section, newest first; cloud: `AgentMap:` lines in the task commit, written by the Merge & Release thread) and append a row to the "Agent run log" in `docs/WORKFLOW.md` for milestones.
4. Run `check_project.ps1 -Fast -Quiet` and `tools/fc.sh docs` (never pipe it into `tail` before `&& git commit`). If clean: commit with the attribution trailer from the session's system reminder and merge per the environment file (both environments: squash into `develop` when approved, no PR; desktop also removes the worktree; `main` moves only through the release PR, desktop.md "Release").
5. If the system is now stable, ask the user whether to mark it `balance-ready`.

## Cleanup loop

`housekeeper` (after a milestone that replaced an older idea, when `fc.sh docs` warns, or on request) -> `docs/specs/CLEANUP.md` candidates -> planner shows the user -> user confirms/rejects -> the planner itself adds the confirmed rows to `docs/TASKS.md` (and ROADMAP for large ones), marks rejected ones in CLEANUP.md and appends dropped ideas to HISTORY -> builders pick them up as separate tasks (never folded into feature work).

## Balance loop

`balance` agent (own `main` worktree) -> `games/emberwake/docs/BALANCE_REPORT.md` -> `design-brainstorm` (options/spec) -> user decision -> new task in `docs/TASKS.md` and the same lanes.

## Rules

- Never skip the confirmation gate. Never commit on red. Never develop on `main`; `main` is always a clean build.
- Parallel work only where file ownership does not overlap (see lane table); never two builders editing the same files.
- Log everything in `docs/TASKS.md` so the state survives a restart.
