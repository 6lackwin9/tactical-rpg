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
| `SKILLS1` | staged | `staging` | Review passed; reaches `develop` with the next full gate. |
| `KITIDS1` | building | `claude/kitids1`, lane 2 | |
| `BASERULES1-M` | queued | - | Starts when a lane frees; see FB-01. |
| `U1.2` (`U1_STATS`) | audited, slice 1 queued | - | FB-04 to FB-09 closed. `U1.2-S1` follows `BASERULES1-M`. |
| `U2.2` (`U2_CLASS`) | audited, slice 1 queued | - | FB-10 to FB-15 closed. `U2.2-S1` follows `U1.2-S1`. |
| `A4.2` (`INTENT2`) | audited | - | FB-16 to FB-22 closed; design updating INTENT2. |
| `U3.2` (`U3_GEAR`) | auditing | - | Read-only audit running. |
| `U4.2` (`U4_PASSIVES`) | auditing | - | Read-only audit running. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

FB-01 to FB-22 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-23.
