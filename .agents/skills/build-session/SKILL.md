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
3. Compare `docs/FEEDBACK.md` against `docs/READY.md` Section 4:
   - Archive any answered `FB-nn` / `UN-nn` items whose `Follow-up` column in `READY.md` Section 4 is `none` or names an existing row in `READY.md` Section 1 or Section 3, moving them into `docs/archive/FEEDBACK_CLOSED.md` with `<disposition> (@<commit>)` so `docs/FEEDBACK.md` stays well under `25 KB`, and verify with `python tools/planner/feedback_check.py`.
4. Check which `ready-to-build` rows in `docs/READY.md` Section 1 are already on `origin/develop`, which are in progress in `-lane1..3` or `-stage`, and which are next in line.
5. Present the parallel lane dispatch plan for the next `ready-to-build` slices in `docs/READY.md` Section 1 and wait for user confirmation before changing code.

## 3. Continuous Queue Sync (Between Slices & Before Stopping)

The Design Session pushes `docs/READY.md` updates (answered `FB-nn` items, promoted waves, and `AUDIT-nn` bug-fix rows from `/audit-build`) directly to `origin/develop` while the Build Session is running:

1. **On every rebase/pull from `origin/develop`**: Whenever `git fetch origin develop` or `git pull --rebase origin develop` brings in a new Design commit that touches `docs/READY.md`, immediately re-read `docs/READY.md` Section 1 and Section 4.
2. **Archive newly answered feedback**: If the Design commit answered open `FB-nn` / `UN-nn` rows in `READY.md` Section 4, archive those answered items from `docs/FEEDBACK.md` into `docs/archive/FEEDBACK_CLOSED.md` (`python tools/planner/feedback_check.py`) on your next flush.
3. **Pick up newly queued slices automatically**: Before declaring a batch, wave, or Silo Release (`R1` / `R2` / `R3`) complete—or before ending a turn at a slice boundary—always `git fetch origin develop` and check `origin/develop:docs/READY.md` Section 1 for any new `ready-to-build` rows (such as `AUDIT-R1` or newly promoted wave rows) added since session start, and continue executing them in order before cutting the release PR or stopping.

## 4. Governance Mandate: Implement-Only as Specified (Never Invent Workarounds for Limits)

The **Design Session is the sole governance authority**; the **Build Session is strictly implement-only as per spec**:

1. **Never work around a limit—raise it in `docs/FEEDBACK.md` and let Design unblock it**: If a clean implementation of the spec hits any limit, gate constraint, or conflict (for example: an `Acceptance Test` column omits a golden fixture like `checks_golden` or `phasefold_golden` that would move, a spec clause conflicts with an existing check or legacy ruleset, or a decision is ambiguous), **do not invent a workaround** (such as reading "row-only" JSON keys in `source.rs` without `kits/rpg/schema.json`, keeping shadow/dual legacy paths, re-recording an unauthorized golden on your own, or silently dropping part of the slice).
2. **Stop on the blocked item & log `FB-nn`**: Whenever you hit such a limit, log a `blocker` `FB-nn` item in `docs/FEEDBACK.md` stating the exact limit and what needs to be unblocked (e.g. authorizing a golden re-record, clarifying `classic` vs `timeline` gating, or adjusting the spec), leave that blocked slice or clause uncommitted/parked, and wait for the Design Session to rule in `docs/READY.md` (you may continue with independent non-blocked rows in `READY.md` if any exist).
3. **No shadow / dual legacy paths or silent deferrals**: Every consumer (battle, planner, AI, Dojo, Party UI, Content Forge, `kits/rpg/schema.json`, CLI/sim, save/load) must be wired to the canonical implementation specified in `READY.md`. Never mark a slice `done` with silent caveats or half-wired consumers.
