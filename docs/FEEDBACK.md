# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates `## 1. Build Status` and appends items to `## 2. Feedback Items` (feasibility findings, cost surprises, spec conflicts, sim results, and docs deltas for `README.md`, `STRUCTURE.md`, `GAME_BRIEF.md`, `HISTORY.md`).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| `ATTACKLOCK1` (slice 1) | built (off) | `develop` | Waiting on `ATTACKLOCK1-O` design sign-off. |
| `SIMCOUNT1` | done | `develop` | `sim feel counts=on` landed. |
| `VISCLAR1-D` | built | `develop` | Battle view parts 1 and 2 landed; waiting on user look to close M1. |
| `TESTCLEAN1` | building | `claude/testclean1`, lane 1 | One scratch helper removing its folder on drop; `aegis_ui` first. |
| `SKILLS1` | built, in review | `claude/skills1`, lane 3 | `tools/planner/` with relative paths and a README; `fc.sh test` prints each panic's message (checked on a real failing run). |
| `KITIDS1` | building | `claude/kitids1`, lane 2 | |
| `BASERULES1-M` | queued | - | Starts when a lane frees; see FB-01. |
| `U1.2` (`U1_STATS`) | auditing | - | Read-only audit running; findings come as FB items. |
| `U2.2` (`U2_CLASS`) | auditing | - | Read-only audit running; findings come as FB items. |
| `A4.2` (`INTENT2`) | auditing | - | Read-only audit running; findings come as FB items. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

### FB-01: BASERULES1-M points at the wrong sections

- **Spec**: `docs/READY.md` section 1 row 4; `docs/specs/BASERULES1.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: The row says to build from `BASERULES1.md` "sections 1-2 only". Those sections describe today's rules and their weaknesses. The register metadata, `check/ruleset.rs`, bundle tests and `aegis rules` with `LOCKED` and `--contract` are specified in `BASERULES1.md` section 6 ("How the engine enforces it") and its section 7 row BASERULES1-M, and in `BASERULES1_LIBRARY.md` (the option table, bundles, and section 7's authoring note).
- **Recommendation**: Point the row at `BASERULES1.md` sections 6-7 and `BASERULES1_LIBRARY.md`. Until answered, the build follows the row's own summary and acceptance test, which match section 6.

### FB-02: Planner skill still sends the build session to shared docs

- **Spec**: `.claude/skills/planner/SKILL.md`; `CLAUDE.md`
- **Kind**: conflict
- **Status**: open
- **Finding**: CLAUDE.md says the build session writes only `docs/FEEDBACK.md` (plus crate-local READMEs). The planner skill's step 3.3 still says to update `docs/AGENT_MAP.md` and append to the agent run log in `docs/WORKFLOW.md`; the `task-landing` skill and `docs_needed.py` ask the lander to update README, STRUCTURE, GAME_BRIEF and HISTORY. The build session will follow CLAUDE.md: agent runs and docs deltas go here.
- **Recommendation**: Drop or reword those steps in `planner` and `task-landing` for the build session, or name which shared files, if any, it may write.

### FB-03: The gate checks every doc, so a spec can block a build push

- **Spec**: `CLAUDE.md` standing rules; `tools/fc.sh docs`
- **Kind**: feasibility
- **Status**: open
- **Finding**: The full gate (`check_project.ps1`, and `flush.sh`) runs `tools/fc.sh docs --all`. A spec or shared doc that fails the lint blocks the build session's push to `develop`, and under the split the build session may not fix it.
- **Recommendation**: The design session runs `tools/fc.sh docs` before each commit; if a doc still breaks the gate, the build session files a feedback item and waits, rather than editing.
