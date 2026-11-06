---
name: planner
description: The milestone and lane workflow run by the planner (desktop main session, or a cloud area thread for its area's tasks). Use when the user asks to plan, start, build or ship roadmap work. Turns approved specs into tasks in docs/TASKS.md, dispatches the logic-builder, asset-builder, design-brainstorm and balance agents in lanes, reviews, merges, updates docs and keeps agents busy. Load `planner-ops` (dispatch modes) and the environment file docs/workflow/desktop.md or docs/workflow/cloud.md.
---

# Planner workflow (shared by both environments)

**Two-session split (since 2026-11-07, decision "Two-session split: Design and Build")**:

- **Design Session** is the sole writer of `docs/READY.md`, specs (`docs/sessions/*`, `docs/specs/*`), `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `docs/decisions/*`, `games/emberwake/docs/decisions/*`, `docs/HISTORY.md`, `README.md`, `GAME_BRIEF.md`, and `STRUCTURE.md`.
- **Build Session** (this planner skill when running builds) reads **only** `docs/READY.md` (and the specs it names), builds only items marked `ready-to-build`, runs read-only code checks on items marked `feasibility-check`, and writes status, feasibility findings, spec questions, and docs deltas **exclusively to `docs/FEEDBACK.md`** (moving answered/closed items into `docs/archive/FEEDBACK_CLOSED.md` to keep `docs/FEEDBACK.md` under 25 KB, plus crate-local READMEs where appropriate). Do not edit `TASKS.md`, `ROADMAP.md`, decisions, `HISTORY.md`, or specs from the Build Session.

Environment first: `$CLAUDE_CODE_REMOTE` empty = desktop, read `docs/workflow/desktop.md`; `true` = cloud, read `docs/workflow/cloud.md`.

## Lanes (segregation of duties)

| Lane | Agent | Owns | Never touches |
| --- | --- | --- | --- |
| design-session | Design Session | `docs/READY.md`, specs, `docs/TASKS.md`, `ROADMAP.md`, decisions, `HISTORY.md`, shared READMEs, `GAME_BRIEF`, `STRUCTURE` | `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md`, builder lanes' code/data |
| build-plan | Build Session planner | `docs/FEEDBACK.md`, `docs/archive/FEEDBACK_CLOSED.md` (status, feasibility notes, docs deltas, closed archive), lane orchestration, commits & merges | `docs/READY.md`, specs, `TASKS.md`, `ROADMAP.md`, decisions, `HISTORY.md` |
| logic | `logic-builder` | `engine/crates/**` (rules, schema, loader and writer, validation, tests), `kits/rpg/crates/**` and `kits/rpg/schema.json` (the kit's genre rules), `games/emberwake/**`, `tools/**` | shipped `games/emberwake/data/**`, `legacy/**`, the golden fixtures (re-recorded only for an intended change) |
| asset | `asset-builder` | `games/emberwake/data/**` through `aegis content` (upsert, remove, `games/emberwake/data/seeds/<task>.json` with `apply`) | engine code, tests, docs |
| balance | `balance` | `games/emberwake/docs/BALANCE_REPORT.md` (or findings in `docs/FEEDBACK.md`), sims | everything else |
| review | `change-reviewer` | read-only | - |
| cleanup | `housekeeper` | `docs/specs/CLEANUP.md` | TASKS, ROADMAP, HISTORY, code, data, tests, tools |

Logic lane scope (decision "Logic lane scope", user 2026-10-27): dispatch `logic-builder` only for a new system or module, or a change across several crates or more than about five files, one slice per run; do smaller logic (fixes, review follow-ups, `tools/**` scripts, flag flips, single-crate changes) yourself in this session with the skills `engine-checklist`, `rust-conventions`, `patch-and-seed` and `known-issues`, in the task's worktree, and still run `change-reviewer` before the commit.

## 1. Plan (Build Session)

1. Read `docs/READY.md` and `docs/STRUCTURE.md`. Do not crawl `docs/TASKS.md` or historical specs.
2. For items marked `feasibility-check` in `docs/READY.md`, run a fast read-only code check against the target crates and log findings/cost/touchpoints as `FB-nn` items in `docs/FEEDBACK.md`.
3. Pick items marked `ready-to-build` in `docs/READY.md` in order. Present the lanes, dependencies and parallel plan to the user. **Make no code changes until they confirm.** Design questions go into `docs/FEEDBACK.md` (and chat summary).

## 2. Dispatch (parallel lanes)

1. Set up the branch per the environment file (desktop: one worktree per concurrent builder plus a read-only `main` worktree for balance; cloud: a fresh branch per task from the latest `develop` in the area thread's checkout, subagents one after another). `git status` must be clean first. Give each agent its path, branch and spec.
2. Schema-first handshake: for a task with new schema, run the `logic-builder` on the schema slice first as its OWN run (brief: "slice 1 only, end the run with the report"), so the asset lane starts minutes, not a whole build, later; review and merge it quickly (small), then start the `asset-builder` (branching from the updated `main`) in parallel with the logic-builder's remaining behaviour work. They edit different folders, so merges stay clean. Merge order: schema slice, then logic, then data (or data first if its tests are generic); run the full check after each merge.
3. Parallel limits: no hard cap on builders (logic + asset). Soft warning at more than 3 at once: tell the user how many would run, the merge/review load it creates (rebases, hub-file conflicts, planner serial merge time, usage limits, CPU contention) and whether to prioritise or slow down; tasks run in parallel only when their `Touches` (docs/TASKS.md) do not overlap, otherwise sequence them (merge the smaller or more foundational one first). Check overlap per the environment file (desktop: `tools/overlap.sh`; cloud: other areas' unmerged task branches). The shared hub files (`kits/rpg/crates/rpgkit_battle/src/{session,plan,command,ai}.rs`, `emberwake_world/src/{source,store,data}.rs`, `emberwake_data/src/lib.rs`, `emberwake_gui/src/battle.rs`, and the field tables in `kits/rpg/schema.json`) are the usual conflict points, so give two logic tasks that both need them a merge order up front.
4. Keep agents busy: while builders run, `design-brainstorm` writes the next specs and the `balance` agent runs on frozen systems; reviewers run in the background on finished branches while the next task starts. Standing auto-dispatches (no user decision involved, docs only): a new balance report -> `design-brainstorm` options right away; a finished builder -> `change-reviewer` right away, while the planner drafts the docs delta in parallel.
5. Before writing a brief, check it against the spec and the decisions files (a brief narrower than a recorded user decision cost a rerun in SKILL1-A). Briefs name the worktree or checkout, branch, slice, gate command and "do not commit" (builders never commit in either environment; on desktop the planner commits, in the cloud the area thread does). Pass `effort` (and `model` where named) per the active dispatch mode (`planner-ops`).
6. Update task status in `docs/FEEDBACK.md` section 1 (`Build Status`).

## 3. Review and ship

Landing checklist: skill `task-landing` (which docs a change needs, with `docs_needed.py`; the commit formats; stage, flush, cleanup and docs-conflict rules).

1. Review each report; run `change-reviewer`, which runs the suites (don't run them a second time yourself unless you changed something after the review). If a builder failed the same thing twice, ask the user rather than retry. Review fixes go to the SAME builder via `SendMessage` while it is resumable (it keeps its context); spawn a fresh one only when it was stopped or failed, and point it at `.agent/progress.md` (in the cloud also at the last pushed commit). If the planner itself changes code or data after a builder's gate (flag flip, seed re-run), re-run the gate before the review.
2. Record the builders' "docs delta" (lines for README, GAME_BRIEF, STRUCTURE, decisions, or HISTORY) as a `docs-delta` item in `docs/FEEDBACK.md` for the Design Session to apply, and update `docs/FEEDBACK.md` `Build Status`.
3. Record active/finished agent runs in `docs/FEEDBACK.md` `Build Status` (do not edit `docs/AGENT_MAP.md` or `docs/WORKFLOW.md` from the Build Session).
4. Run `check_project.ps1 -Fast -Quiet` (it includes `tools/fc.sh docs --all`; a docs-only commit: `tools/fc.sh docs`; never pipe either into `tail` before `&& git commit`). If clean: commit with the attribution trailer from the session's system reminder and merge per the environment file (both environments: squash into `develop` when approved, no PR; desktop also removes the worktree; `main` moves only through the release PR, desktop.md "Release").
5. If the system is now stable, ask the user whether to mark it `balance-ready`.

## Cleanup loop

`housekeeper` (after a milestone that replaced an older idea, when `fc.sh docs --all` warns, or on request) -> `docs/specs/CLEANUP.md` candidates -> planner shows the user -> user confirms/rejects -> the planner itself adds the confirmed rows to `docs/TASKS.md` (and ROADMAP for large ones), marks rejected ones in CLEANUP.md and appends dropped ideas to HISTORY -> builders pick them up as separate tasks (never folded into feature work).

## Balance loop

`balance` agent (own `main` worktree) -> `games/emberwake/docs/BALANCE_REPORT.md` -> `design-brainstorm` (options/spec) -> user decision -> new task in `docs/TASKS.md` and the same lanes.

## Rules

- Never skip the confirmation gate. Never commit on red. Never develop on `main`; `main` is always a clean build.
- Parallel work only where file ownership does not overlap (see lane table); never two builders editing the same files.
- Log everything in `docs/TASKS.md` so the state survives a restart.
