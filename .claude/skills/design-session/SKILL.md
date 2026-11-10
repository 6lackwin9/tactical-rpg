---
name: design-session
description: Bootstrap and operate the Design Session in the two-session split (Design in C:\dev\tactical-rpg-dev vs Build in C:\dev\tactical-rpg-plan). Use when starting or resuming a Design Session (/design-session, "start design session") to sync origin/develop, review docs/READY.md and docs/FEEDBACK.md, apply docs deltas, answer FB-nn / UN-nn items, and maintain READY.md and specs.
---

# Design Session Start & Workflow (`C:\dev\tactical-rpg-dev`)

Operate as the **Design Session** (`C:\dev\tactical-rpg-dev`, sole governance authority) under the two-session split (`CLAUDE.md`, `docs/WORKFLOW.md`, `docs/decisions/workflow.md`).

## 1. Ownership & Boundaries

- **Sole writer of**: `docs/READY.md`, `docs/specs/MASTERPLAN1.md`, `docs/sessions/*.md`, `docs/specs/*`, `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `games/emberwake/docs/MILESTONES.md`, `docs/DECISIONS.md`, `docs/decisions/*`, `docs/HISTORY.md`, `CLAUDE.md`, and shared READMEs (`README.md`, `engine/README.md`, `kits/rpg/README.md`, `docs/STRUCTURE.md`, `docs/WORKFLOW.md`, `games/emberwake/docs/GAME_BRIEF.md`, `engine/docs/PORT_NOTES.md`).
- **Never edit**: `docs/FEEDBACK.md` or `docs/archive/FEEDBACK_CLOSED.md` (owned exclusively by the Build Session so there are zero merge conflicts between sessions).
- **Locked away from Artwork/UI Iteration**: Live UI prototyping, sprite generation, VN stage choreography, and character animation rigging (`tools/mock_ui/**`, `tools/mock_ui/anim_lab/**`, `art_src/vn_stage/**`, `games/emberwake/docs/specs/ART_UI_STUDIO.md`) are isolated to the dedicated **Artwork/UI Session (`/art-ui-session`)**. Do not run UI or animation iteration in the Design Session unless importing a signed-off UI/art decision into `docs/READY.md`.

## 2. Startup Sequence (Run on Session Start)

1. Sync `C:\dev\tactical-rpg-dev` with `origin/develop`:
   `git -C C:\dev\tactical-rpg-dev fetch origin`
   `git -C C:\dev\tactical-rpg-dev log --oneline -n 10 origin/develop`
2. Read `docs/READY.md` and `docs/FEEDBACK.md` (from `origin/develop`), and run `python tools/planner/feedback_check.py --pending-ok`.
3. Triage open `FB-nn` (Section 2) and `UN-nn` (Section 3) items by their structured `Kind`:
   - **Standing rule (user 2026-11-10, decision "Build feedback on the recommended option")**: for `limit-blocker` and `suggestion` items alike, apply the recommended option without waiting and add a row to `docs/DESIGN_REVIEW.md` for the user to review; ask first only for steps that cannot be undone or items with no recommendation. The two bullets below describe what to record.
   - **`limit-blocker` (Highest Priority)**: Present the Builder's limit, technical assessment, and `Option A` / `Option B` recommendations to the user first so we can unblock `docs/READY.md` Section 1/4 immediately.
   - **`suggestion`**: Present the Builder's assessment and recommended default for user confirmation (to either accept into `READY.md` Section 1/3 or defer).
   - **`docs-delta`**: Apply the landed slice's shared-doc updates (`docs/HISTORY.md`, `docs/STRUCTURE.md`, `games/emberwake/docs/GAME_BRIEF.md`, `engine/docs/PORT_NOTES.md`, `engine/README.md`, `kits/rpg/README.md`) and record `accepted` in `docs/READY.md` Section 4.
3b. Triage the Art/UI outbox `games/emberwake/docs/specs/ART_UI_REQUESTS.md` (if it exists): queue each new request into `docs/READY.md` (Build work) or answer it (process), then mark the row done in READY Section 4 with the request id (decision "Art/UI request outbox and ownership").
4. When a Silo Release (`R1`, `R2`, `R3`) or major wave lands on `origin/develop`, run `/audit-build` (`build-auditor` subagent) to verify implementation fidelity (Part A), benchmark against the **Builder Authority Matrix** (Part B), and review big-picture system/decision health (Part C).
5. Run `python tools/planner/feedback_check.py` (must exit `0`) and `& "C:\Program Files\Git\bin\bash.exe" tools/fc.sh docs` before committing and pushing to `origin/develop`, then restore `C:\dev\tactical-rpg-dev` to Clean Pull-Ready Mode.
