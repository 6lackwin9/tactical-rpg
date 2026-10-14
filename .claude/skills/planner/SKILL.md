---
name: planner
description: The milestone workflow run by the MAIN session (not an agent). Use when the user asks to plan, start, build or ship roadmap work. Breaks approved roadmap items into milestones with change lists, waits for approval, dispatches the builder agent on a feature branch, reviews, summarises, commits, pushes and updates docs.
---

# Planner workflow (main session)

You are the planner and orchestrator. Agents cannot message each other; everything flows through you, with files as handoffs: `docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/BALANCE_REPORT.md`, `docs/DECISIONS.md`, `docs/HISTORY.md`.

## 1. Plan (no code changes)
1. Read `docs/ROADMAP.md`, `docs/STRUCTURE.md`, `docs/HISTORY.md` (do not propose deprecated concepts), `docs/DECISIONS.md`. Do not crawl code.
2. Take only items the user marked `approved` (or ask which `proposed` item to plan). Split into milestones of one reviewable change each.
3. For each milestone write: goal, files to create/modify (use paths from STRUCTURE.md; fill placeholder folders rather than inventing new ones), data changes (via `ContentCatalog.upsert`), tests to add (rule-derived, one suite per system), README/doc updates, risks, and the exact acceptance check (`tools/fc.sh all` clean).
4. **Stop and present the milestone list and change list to the user. Make no changes until they confirm.** Design questions go to the `design-brainstorm` agent or the user, never decided silently.

## 2. Build (after confirmation)
1. Create a feature branch `feat/<milestone-id>-<slug>` from clean `main` (`git status` must be clean first). `main` stays a clean build.
2. Launch the `builder` agent (isolation: worktree when work could overlap) with: the approved change list, the branch, and pointers to the docs. Builder implements and tests, then reports.
3. Mark the roadmap item `in-progress`.

## 3. Review and ship
1. Review the builder report; run `change-reviewer` (and `tools/fc.sh all`). If the builder failed the same thing twice, the answer is to ask the user, not to retry.
2. Summarise the implemented changes to the user (files, behaviour, tests, deviations).
3. Update README, `docs/ROADMAP.md` (status), `docs/STRUCTURE.md` (placeholder -> done), `docs/DECISIONS.md` / `docs/HISTORY.md` (anything replaced: append, never delete).
4. Run `check_project.ps1`. If clean: commit with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`, merge to `main` (fast-forward), push, delete the branch.
5. If the system is now stable, ask the user whether to mark it `balance-ready` so the `balance` agent may tune it.

## Balance loop
Balance findings go: `balance` agent -> `docs/BALANCE_REPORT.md` -> `design-brainstorm` agent (options) -> user decision -> new roadmap milestone (balance changes are code/data changes and follow this same workflow).

## Rules
- Never skip the confirmation gate. Never commit on red. Never work on `main` directly for features.
- Parallel work only for read-only tasks (sims, reviews, brainstorm), or builders in separate worktrees on separate systems.
