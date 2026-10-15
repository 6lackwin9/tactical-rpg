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

If the custom agent types are not yet callable (agent files added this session), run the same instructions through a general-purpose agent.

## 1. Plan (no code changes)

1. Read `docs/TASKS.md`, `docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/HISTORY.md` (do not propose deprecated concepts), `docs/DECISIONS.md`. Do not crawl code.
2. Pick tasks the user marked `approved`. A task needs a spec in `docs/specs/`: if it has none, dispatch `design-brainstorm` in the background to write it (it also drafts specs for the NEXT tasks while builders work).
3. Present the task list, lanes, dependencies and parallel plan to the user. **Make no changes until they confirm.** Design questions go to the user.

## 2. Dispatch (parallel lanes)

1. Create worktrees for parallel lanes from clean `main` (`git worktree add ../<repo>-<id> -b feat/<id>-<slug>`); use one worktree per concurrent builder and one read-only `main` worktree for the balance agent. `git status` must be clean first. Give each agent its worktree path, branch and spec.
2. Schema-first handshake: for a task with new schema, run the `logic-builder` on the schema slice first; review and merge it quickly (small), then start the `asset-builder` (branching from the updated `main`) in parallel with the logic-builder's remaining behaviour work. They edit different folders, so merges stay clean. Merge order: schema slice, then logic, then data (or data first if its tests are generic); run the full check after each merge.
3. Keep agents busy: while builders run, `design-brainstorm` writes the next specs and the `balance` agent runs on frozen systems; reviewers run in the background on finished branches while the next task starts. Independent tasks with no shared files run in separate worktrees at the same time.
4. Mark tasks `in-progress` in `docs/TASKS.md`.

## 3. Review and ship

1. Review each report; run `change-reviewer` (and `tools/fc.sh all`). If a builder failed the same thing twice, ask the user rather than retry.
2. Apply the builders' "docs delta" (README, GAME_BRIEF, STRUCTURE, DECISIONS / HISTORY append-only), update `docs/ROADMAP.md` and `docs/TASKS.md` statuses.
3. Append a row to the "Agent run log" in `docs/WORKFLOW.md` (task, agents, tokens/minutes, result).
4. Run `check_project.ps1 -Fast -Quiet`. If clean: commit with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`, merge to `main` (fast-forward), push, remove the worktree and branch.
5. If the system is now stable, ask the user whether to mark it `balance-ready`.

## Balance loop

`balance` agent (own `main` worktree) -> `docs/BALANCE_REPORT.md` -> `design-brainstorm` (options/spec) -> user decision -> new task in `docs/TASKS.md` and the same lanes.

## Rules

- Never skip the confirmation gate. Never commit on red. Never develop on `main`; `main` is always a clean build.
- Parallel work only where file ownership does not overlap (see lane table); never two builders editing the same files.
- Log everything in `docs/TASKS.md` so the state survives a restart.
