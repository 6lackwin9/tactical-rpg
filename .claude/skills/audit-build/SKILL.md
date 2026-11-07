---
name: audit-build
description: Run an independent Design-Session audit over landed Build commits on origin/develop using the build-auditor agent. Surfaces implementation bugs/misses (AUDIT-nn rows for READY.md), unlogged Builder changes / control-plane drift, and big-picture system/decision critique.
---

# Independent Build Audit (`/audit-build`)

Use this skill in the **Design Session** (`C:\dev\tactical-rpg-dev`) after the Build Session lands a batch or Silo Release (`R1`, `R2`, `R3`) on `origin/develop`.

## Workflow

1. **Identify Commit Range**:
   - Run `git -C C:\dev\tactical-rpg-dev fetch origin` and `git -C C:\dev\tactical-rpg-dev log --oneline origin/develop -n 25` to identify the landed Build commits since the last audit.
2. **Dispatch `build-auditor`**:
   - Spawn the `build-auditor` subagent with the commit range, the corresponding `docs/READY.md` rows, and the governing session specs (`docs/sessions/*.md`).
   - Require `build-auditor` to inspect the actual Rust/JSON diffs (`git show`, `read_file`, `grep_search`) and return:
     - **Part A (Implementation Audit)**: Bugs, missed call sites, edge-case gaps, and concrete `AUDIT-nn` rows for `docs/READY.md` Section 1.
     - **Part B (Unlogged Builder Changes & Control-Plane Drift)**: Any self-decided code/schema/data changes not logged in `docs/FEEDBACK.md`, unnamed golden re-records (e.g. `phasefold_golden` or `checks_golden`), raw-key workarounds bypassing `schema.json`, or ownership boundary slips, plus control-plane fixes.
     - **Part C (Project & System Critique)**: Outdated decisions, cross-silo friction, split state, and design/balance recommendations.
3. **Present & Queue**:
   - Present Part A (`AUDIT-nn` bug fixes), Part B (Unlogged Changes & Control-Plane Drift), and Part C (Big-Picture Critique) to the user.
   - Add approved `AUDIT-nn` rows to `docs/READY.md` Section 1 (`ready-to-build`) and any control-plane/doc updates to the relevant skills or specs, commit, push to `origin/develop`, and restore Clean Pull-Ready Mode.
