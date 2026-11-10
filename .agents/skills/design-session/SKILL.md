---
name: design-session
description: Bootstrap and operate the Design Session in the two-session split (Design in C:\dev\tactical-rpg-dev vs Build in C:\dev\tactical-rpg-plan). Use when starting or resuming a Design Session (/design-session, "start design session") to sync origin/develop, review docs/READY.md and docs/FEEDBACK.md, score options against docs/PRINCIPLES.md, apply docs deltas, answer FB-nn / UN-nn items, and maintain READY.md and specs.
---

# Design Session Start & Workflow (`C:\dev\tactical-rpg-dev`)

Operate as the **Design Session** (`C:\dev\tactical-rpg-dev`, sole governance authority) under the two-session split (`CLAUDE.md`, `docs/WORKFLOW.md`, `docs/PRINCIPLES.md`, `docs/decisions/workflow.md`).

## 1. Ownership & Boundaries

- **Sole writer of**: `docs/READY.md`, `docs/PRINCIPLES.md`, `docs/DESIGN_REVIEW.md`, `docs/audits/*.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/specs/*`, `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `games/emberwake/docs/MILESTONES.md`, `games/emberwake/docs/lore/CANON.md`, `docs/DECISIONS.md`, `docs/decisions/*`, `docs/HISTORY.md`, `CLAUDE.md`, and shared READMEs (`README.md`, `engine/README.md`, `kits/rpg/README.md`, `docs/STRUCTURE.md`, `docs/WORKFLOW.md`, `games/emberwake/docs/GAME_BRIEF.md`, `engine/docs/PORT_NOTES.md`).
- **Never edit**: `docs/FEEDBACK.md` or `docs/archive/FEEDBACK_CLOSED.md` (owned exclusively by the Build Session so there are zero merge conflicts between sessions), or `assets/**/README.md` (owned by Art/UI).
- **Locked away from Artwork/UI Iteration**: Live UI prototyping, sprite generation, VN stage choreography, and character animation rigging (`tools/mock_ui/**`, `tools/mock_ui/anim_lab/**`, `art_src/vn_stage/**`, `games/emberwake/docs/specs/ART_UI_STUDIO.md`) are isolated to the dedicated **Artwork/UI Session (`/art-ui-session`)**. Do not run UI or animation iteration in the Design Session unless importing a signed-off UI/art decision into `docs/READY.md`.

## 2. Startup Sequence (Run on Session Start)

1. Sync `C:\dev\tactical-rpg-dev` with `origin/develop`:
   `git -C C:\dev\tactical-rpg-dev fetch origin`
   `git -C C:\dev\tactical-rpg-dev log --oneline -n 10 origin/develop`
2. Read `docs/READY.md`, `docs/FEEDBACK.md`, and `docs/PRINCIPLES.md` (from `origin/develop`), and run `python tools/planner/feedback_check.py --pending-ok`.
3. Triage open `FB-nn` (Section 2) and `UN-nn` (Section 3) items—plus any open `.1` spec or `CL-nn` canon gaps—using **`docs/PRINCIPLES.md` Section 0**:
   - **Autonomous Principle-Guided Rule (user 2026-11-10, `"Build feedback on the recommended option"` + `docs/PRINCIPLES.md`)**: Check `[LOCKED]` rules (`CANON.md`) and `DECISIONS.md` first, score candidate options against `P-ARCH`..`P-UI` in `docs/PRINCIPLES.md` (preferring lowest undo cost, reusable kit generality, and zero unintended golden drift), apply the winning option immediately as `[PROVISIONAL]` in `docs/READY.md` / specs / `CANON.md` without stalling Build, and log a row in `docs/DESIGN_REVIEW.md` citing the `P-*` IDs applied. Pause and ask first *only* on hard-stop exceptions (irreversible git/release actions, paid external API spend, breaking shipped release saves without migration, or `[LOCKED]` conflicts).
   - **`limit-blocker` (Highest Priority)**: Score `Option A` / `Option B` against `docs/PRINCIPLES.md`, unblock `docs/READY.md` Section 1/4 immediately, and log in `docs/DESIGN_REVIEW.md` (or ask interactively when the user is present on major trade-offs).
   - **`suggestion`**: Evaluate against `docs/PRINCIPLES.md` to accept into `READY.md` Section 1/3 or defer, recording autonomous calls in `docs/DESIGN_REVIEW.md`.
   - **`docs-delta`**: Apply the landed slice's shared-doc updates (`docs/HISTORY.md`, `docs/STRUCTURE.md`, `games/emberwake/docs/GAME_BRIEF.md`, `engine/docs/PORT_NOTES.md`, `engine/README.md`, `kits/rpg/README.md`) and record `accepted` in `docs/READY.md` Section 4.
   - **Art/UI requests** (`games/emberwake/docs/specs/ART_UI_REQUESTS.md`, decision `"Art/UI request outbox and ownership"`): queue or answer each **Outbox** row in `docs/READY.md` (READY Section 4 keeps the answer with the request id), then delete the row; put tasks for the Art/UI session in the **Inbox** (Art/UI deletes them once done).
4. When a Silo Release (`R1..R7`) or major wave lands on `origin/develop`, run `/audit-build` (`build-auditor` subagent) to verify implementation fidelity (Part A), benchmark against the **Builder Authority Matrix** (Part B), critique alignment with `docs/PRINCIPLES.md` & system health (Part C), and verify `CANON.md` alignment (Part D).
5. Run `python tools/planner/feedback_check.py` (must exit `0`) and `& "C:\Program Files\Git\bin\bash.exe" tools/fc.sh docs` before committing and pushing to `origin/develop`, then restore `C:\dev\tactical-rpg-dev` to Clean Pull-Ready Mode.
