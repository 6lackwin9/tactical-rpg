---
name: build-session
description: Bootstrap and operate the Build Session planner in the two-session split (Build in C:\dev\tactical-rpg-plan, -stage, -lane1..3 vs Design in C:\dev\tactical-rpg-dev). Use when starting or resuming a Build Session (/build-session, /build-session force, "start build session", "start build session force") to sync origin/develop, claim or force-claim origin/build-lease, read docs/READY.md and docs/FEEDBACK.md, archive answered FB-nn items, and dispatch ready-to-build slices.
---

# Build Session Start & Workflow (`C:\dev\tactical-rpg-plan` or Cloud Container)

Operate as the **Build Session** (desktop: `C:\dev\tactical-rpg-plan`, `-stage`, `-lane1..3`; cloud when `$CLAUDE_CODE_REMOTE` is `true`: container checkout on `origin/develop`) under the two-session split (`CLAUDE.md`, `docs/WORKFLOW.md`, `docs/decisions/workflow.md`). Load `.claude/skills/planner/SKILL.md` and the environment file (`docs/workflow/desktop.md` on desktop, `docs/workflow/cloud.md` in the cloud).

## 1. Ownership & Boundaries

- **Sole writer of**: `docs/FEEDBACK.md` and `docs/archive/FEEDBACK_CLOSED.md` (plus crate-local READMEs where appropriate), and orchestrator of code/data builds and flushes onto `origin/develop`.
- **Never edit**: `docs/READY.md`, `docs/sessions/*.md`, `docs/specs/*`, `docs/TASKS.md`, `games/emberwake/docs/ROADMAP.md`, `games/emberwake/docs/MILESTONES.md`, `docs/DECISIONS.md`, `docs/decisions/*`, `docs/HISTORY.md`, or shared READMEs.

## 2. Startup Sequence (Run on Session Start — Supports `force` / `--force` Argument)

1. Fetch `origin` and claim the **sticky remote Build Session lease** (`origin/build-lease`) so two Build Sessions (cloud vs desktop) never collide:
   - **Normal start (`/build-session`)**:
     `git fetch -p origin`
     `python tools/planner/feedback_check.py lease claim --no-fetch`
     `git log -8 --oneline origin/develop`
     If `feedback_check.py lease claim` exits `1` (because `origin/build-lease` is owned by the other session or an unmerged `origin/claude/*` task branch / `building` row in `docs/FEEDBACK.md` was updated within the last 20 minutes), **stop immediately and ask the user** before touching any branch or file.
   - **Force-lease takeover (`/build-session force`, `/build-session --force`, or user says `"force lease"` / `"take over"`)**:
     `git fetch -p origin`
     `python tools/planner/feedback_check.py lease claim --force --no-fetch`
     `git log -8 --oneline origin/develop`
     Immediately claims `origin/build-lease` for this session (`desktop` or `cloud`) without prompting, overriding the previous session's lease so any sleeping session is blocked on auto-wake.
2. Read `docs/READY.md` and `docs/FEEDBACK.md`.
3. Compare `docs/FEEDBACK.md` against `docs/READY.md` Section 4:
   - Archive any answered `FB-nn` / `UN-nn` items whose `Follow-up` column in `READY.md` Section 4 is `none` or names an existing row in `READY.md` Section 1 or Section 3, moving them into `docs/archive/FEEDBACK_CLOSED.md` with `<disposition> (@<commit>)` so `docs/FEEDBACK.md` stays well under `25 KB`, and verify with `python tools/planner/feedback_check.py`.
4. Check which `ready-to-build` rows in `docs/READY.md` Section 1 are already on `origin/develop`, which are in progress in `-lane1..3` or `-stage`, and which are next in line.
5. Present the parallel lane dispatch plan for the next `ready-to-build` slices in `docs/READY.md` Section 1 and wait for user confirmation before changing code.

## 3. Continuous Queue Sync & Auto-Wake Lease Guard (Between Slices, On Resume, & Before Pushing)

The Design Session pushes `docs/READY.md` updates (answered `FB-nn` items, promoted waves, and `AUDIT-nn` bug-fix rows from `/audit-build`) directly to `origin/develop` while the Build Session is running, and the user may move the Build Session lease between `cloud` and `desktop` if a session hits a quota limit:

1. **Before resuming after any pause/auto-wake AND before every push/flush to `origin`**: Always run `python tools/planner/feedback_check.py lease check`. If `lease check` exits `1` because `origin/build-lease` was moved to another session (`desktop` vs `cloud`), **halt immediately without committing, merging, or pushing**.
2. **On every rebase/pull from `origin/develop`**: Whenever `git fetch origin develop` or `git pull --rebase origin develop` brings in a new Design commit that touches `docs/READY.md`, immediately re-read `docs/READY.md` Section 1 and Section 4.
3. **Archive newly answered feedback**: If the Design commit answered open `FB-nn` / `UN-nn` rows in `READY.md` Section 4, archive those answered items from `docs/FEEDBACK.md` into `docs/archive/FEEDBACK_CLOSED.md` (`python tools/planner/feedback_check.py`) on your next flush.
4. **Pick up newly queued slices automatically**: Before declaring a batch, wave, or Silo Release (`R1` / `R2` / `R3`) complete—or before ending a turn at a slice boundary—always `git fetch origin develop` and check `origin/develop:docs/READY.md` Section 1 for any new `ready-to-build` rows (such as `AUDIT-R1` or newly promoted wave rows) added since session start, and continue executing them in order before cutting the release PR or stopping.

## 4. Builder Authority Matrix (Allowed vs Forbidden)

The **Design Session is the sole governance authority**; the **Build Session is strictly implement-only as per spec**:

| Category | Allowed (Proceed & Build) | Forbidden (Stop on Item & Raise `Kind: limit-blocker`) |
| --- | --- | --- |
| **Task Selection** | Build rows in `docs/READY.md` Section 1 (`ready-to-build`) in order, plus explicit instructions in `READY.md` Section 4. | Building from `TASKS.md`, `READY.md` Section 3 (`Held`), or self-initiating unqueued features/cleanups. |
| **Spec & Formula Implementation** | Implement exact formulas, data structures, UI hooks, and call sites specified in `READY.md` and the linked `docs/sessions/*.md` spec. | Changing a formula, default value, ruleset gate (`classic` vs `timeline`), or skipping a consumer/UI call site ("done with caveats") because it moves numbers or is inconvenient. |
| **Schema & Data (`schema.json` & `source.rs`)** | Add new fields to *both* `source.rs`/`store.rs` and `kits/rpg/schema.json` as one unit. | "Row-only" JSON keys in `source.rs` that bypass `kits/rpg/schema.json` or content checks. |
| **Golden Fixtures** | Re-record **only** the behavioral goldens named in `READY.md`'s `Acceptance Test` column (plus purely mechanical schema/contract field-count updates in `checks_golden` / `rules_contract_golden` when adding a spec-mandated field). | Re-recording an unnamed behavioral golden (`rules_golden`, `timeline_golden`, `sim_golden`, `phasefold_golden`, `journey_golden`), OR inventing a code workaround to avoid moving a golden when the spec itself causes the move. |
| **Ambiguities & Edge Cases** | Internal Rust/engine plumbing choices (helper names, module split, test structure) with zero behavioral or schema impact. | Guessing gameplay/design intent when two specs disagree or when a spec clause breaks an existing invariant. |

- **Data-row blockers (`Kind: limit-blocker`, never `suggestion`)**: When a `Layer: data` row needs an unqueued engine/kit/game code hook (e.g., an effect type, trigger gate, UI guard, or shop discount), never ship a manual player self-report dialogue workaround or silently drop pure-data clauses—land the clean pure-data parts if separable and log the missing hook as `Kind: limit-blocker`.

## 5. Structured Feedback Protocol (`docs/FEEDBACK.md` Section 2)

Every `FB-nn` item in `docs/FEEDBACK.md` must use one of three structured `Kind` templates (or separate sub-blocks if a slice has both a `docs-delta` and a `suggestion`/`limit-blocker`):

1. **`Kind: limit-blocker`** (Builder hit a Forbidden boundary or constraint conflict and parked the blocked slice/clause):
   - `- **Spec**: <spec & READY slice>`
   - `- **Kind**: limit-blocker`
   - `- **Status**: open`
   - `- **Finding**: <exact limit/constraint hit + technical trace/assessment from code or tests>`
   - `- **Recommendation**: <Option A / Option B with trade-offs + Recommended default to unblock>`
2. **`Kind: suggestion`** (Slice built 100% as specified without workarounds; Builder is proposing a balance, feel, or follow-up improvement):
   - `- **Spec**: <spec & READY slice (@commit)>`
   - `- **Kind**: suggestion`
   - `- **Status**: open`
   - `- **Finding**: <what was observed in code, tests, or sims>`
   - `- **Recommendation**: <proposed follow-up task or tuning options with recommended default>`
3. **`Kind: docs-delta`** (Routine landing report for shared docs—no design decision required):
   - `- **Spec**: <spec & READY slice (@commit)>`
   - `- **Kind**: docs-delta`
   - `- **Status**: open`
   - `- **Finding**: <concise summary of what landed and test counts>`
   - `- **Recommendation**: <exact lines/bullets to apply to HISTORY.md, GAME_BRIEF.md, STRUCTURE.md, PORT_NOTES.md>`
