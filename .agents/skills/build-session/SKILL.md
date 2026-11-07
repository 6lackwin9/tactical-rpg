---
name: build-session
description: Bootstrap and operate the Build Session planner in the two-session split (Build in C:\dev\tactical-rpg-plan, -stage, -lane1..3 vs Design in C:\dev\tactical-rpg-dev). Use when starting or resuming a Build Session (/build-session, "start build session") to sync origin/develop, read docs/READY.md and docs/FEEDBACK.md, archive answered FB-nn items, and dispatch ready-to-build slices.
---

# Build Session Start & Workflow (`C:\dev\tactical-rpg-plan` or Cloud Container)

Operate as the **Build Session** (desktop: `C:\dev\tactical-rpg-plan`, `-stage`, `-lane1..3`; cloud when `$CLAUDE_CODE_REMOTE` is `true`: container checkout on `origin/develop`) under the two-session split (`CLAUDE.md`, `docs/WORKFLOW.md`). Load `.claude/skills/planner/SKILL.md` and the environment file (`docs/workflow/desktop.md` on desktop, `docs/workflow/cloud.md` in the cloud).

## 1. Ownership & Boundaries

- **Sole writer of**: `docs/FEEDBACK.md` and `docs/archive/FEEDBACK_CLOSED.md` (plus crate-local READMEs where appropriate), and orchestrator of code/data builds and flushes onto `origin/develop`.
- **Never edit**: `docs/READY.md`, `docs/sessions/*.md`, `docs/specs/*`, `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `games/emberwake/docs/MILESTONES.md`, `docs/DECISIONS.md`, `docs/HISTORY.md`, or shared READMEs (instead, log a `docs-delta` or `FB-nn` item in `docs/FEEDBACK.md` for the Design Session to apply).

## 2. Startup Sequence (Run on Session Start)

1. Fetch and checkout the latest `origin/develop` (desktop: in `C:\dev\tactical-rpg-plan` and check `git status` across `-stage` and `-lane1..3`; cloud: in the container checkout):
   `git fetch origin develop`
   `git log -8 --oneline origin/develop`
2. Read `docs/READY.md` and `docs/FEEDBACK.md`.
3. Compare `docs/FEEDBACK.md` against `docs/READY.md` Section 4 (`SKILLS1-FB`):
   - Run `python3 tools/planner/feedback_check.py --pending-ok` to verify all answered rows and list any open items waiting for Design, then run `python3 tools/planner/feedback_check.py archive` to move every answered `FB-nn` / `UN-nn` item into `docs/archive/FEEDBACK_CLOSED.md` with `closed: <disposition> (@<commit>)` so `docs/FEEDBACK.md` stays well under `25 KB`.
4. Check which `ready-to-build` rows in `docs/READY.md` Section 1 are already on `origin/develop`, which are in progress in `-lane1..3` or `-stage`, and which are next in line.
5. Present the parallel lane dispatch plan for the next `ready-to-build` slices in `docs/READY.md` Section 1 and wait for user confirmation before changing code.
