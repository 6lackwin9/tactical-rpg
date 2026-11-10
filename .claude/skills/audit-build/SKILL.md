---
name: audit-build
description: Run an independent Design-Session audit over landed Build commits on origin/develop using the build-auditor agent. Surfaces implementation bugs/misses (AUDIT-nn rows for READY.md), unlogged Builder changes / control-plane drift, big-picture system/decision critique, and Canon/Lore alignment against CANON.md.
---

# Independent Build Audit (`/audit-build`)

Use this skill in the **Design Session** (`C:\dev\tactical-rpg-dev` or cloud) after the Build Session lands a batch or Silo Release (`R1..R7`) on `origin/develop`. Unlike `/consistency-audit` (which scans the entire repository), `/audit-build` focuses strictly on the **recent landed Build commit range** since the last build audit.

## Workflow

1. **Identify Commit Range**:
   - Run `git fetch origin` and `git log --oneline origin/develop -n 25` to identify the landed Build commits since the last audit.
2. **Dispatch `build-auditor`**:
   - Spawn the `build-auditor` subagent with the commit range, the corresponding `docs/READY.md` rows, the governing session/feature specs (`docs/sessions/*.md`, `docs/specs/*.md`), and the canon blueprint (`games/emberwake/docs/lore/CANON.md`).
   - Require `build-auditor` to inspect the actual Rust/JSON/doc diffs (`git show`, `read_file`, `grep_search`) and return:
     - **Part A (Implementation Audit)**: Bugs, missed call sites, edge-case gaps, and concrete `AUDIT-nn` rows for `docs/READY.md` Section 1.
     - **Part B (Unlogged Builder Changes & Control-Plane Drift)**: Any self-decided code/schema/data changes not logged in `docs/FEEDBACK.md`, unnamed golden re-records (e.g. `phasefold_golden` or `checks_golden`), raw-key workarounds bypassing `schema.json`, or ownership boundary slips, plus control-plane fixes.
     - **Part C (Project & System Critique)**: Outdated decisions, cross-silo friction, split state, and design/balance recommendations.
     - **Part D (Canon & Lore Alignment Audit — `CANON.md`)**: Verification that touched data/UI/code obeys `CANON.md` `[LOCKED]` narrative & system invariants, introduces zero retired glossary terms (`CANON.md` Section 4), keeps `[PLACEHOLDER]` cast/visual flavour and `[PROVISIONAL]` gap defaults (`CL-01..CL-07`) 100% data/asset-driven without hardcoding or silent promotion to `[LOCKED]`, and never invents unapproved lore for `[MISSING]` slots (`CANON.md` Section 5.2).
3. **Present & Queue**:
   - Present Part A (`AUDIT-nn` bug fixes), Part B (Unlogged Changes & Control-Plane Drift), Part C (Big-Picture Critique), and Part D (Canon & Lore Alignment) to the user.
   - Add approved `AUDIT-nn` rows to `docs/READY.md` Section 1 (`ready-to-build`) and any control-plane/canon updates to the relevant skills or specs, commit, push to `origin/develop`, and restore Clean Pull-Ready Mode.
