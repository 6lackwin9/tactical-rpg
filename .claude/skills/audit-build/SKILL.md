---
name: audit-build
description: Run an independent Design-Session audit on landed Build commits in origin/develop using the build-auditor subagent. Checks implementation fidelity against specs (Part A: bugs, missed call sites, AUDIT-nn fix rows) and evaluates big-picture system/decision health across the project (Part B).
---

# Independent Build Audit (`/audit-build`)

Use this skill in the **Design Session** (`C:\dev\tactical-rpg-dev`) to independently verify code, schema, data, and test commits landed by the Build Session on `origin/develop`.

## Workflow

1. **Identify Commit Range**:
   - Fetch `origin/develop`:
     `git -C C:\dev\tactical-rpg-dev fetch origin develop`
   - List Build commits landed since the last Design audit (`git -C C:\dev\tactical-rpg-dev log --oneline <last-design-sha>..origin/develop`).
2. **Sync Working Tree for Read-Only Inspection**:
   - Checkout `origin/develop` into `C:\dev\tactical-rpg-dev` via a temporary `GIT_INDEX_FILE` so the auditor can read the current source files directly without touching `.git/worktrees/tactical-rpg-dev/index`.
3. **Dispatch `build-auditor` Subagent**:
   - Invoke the `build-auditor` agent (`.claude/agents/build-auditor.md`) on the commit range and the corresponding session specs (`docs/sessions/*.md`) and `docs/READY.md` rows.
   - Ask for the two-part report:
     - **Part A (Spec & Implementation Audit)**: Call-site completeness across `rules`/`battle`/`world`/`content`/`sim`/`gui`, edge-case invariants, test coverage, and ready-to-queue `AUDIT-nn` rows for `docs/READY.md` Section 1.
     - **Part B (Project & System Health Critique)**: Outdated decisions/fields, cross-silo friction for upcoming blocks, and architectural/balance recommendations.
4. **Act on Findings**:
   - Present Part A and Part B to the user with recommended dispositions:
     - Queue urgent bugs or implementation misses as `AUDIT-nn` rows in `docs/READY.md` Section 1 (`ready-to-build`).
     - Queue post-release polish or architectural cleanups in `docs/READY.md` Section 3.
     - Retire or update outdated decisions/docs in `docs/DECISIONS.md`, `docs/HISTORY.md`, and `games/emberwake/docs/GAME_BRIEF.md`.
   - Restore `C:\dev\tactical-rpg-dev` to **Clean Pull-Ready Mode** after committing any `READY.md` / docs updates.
