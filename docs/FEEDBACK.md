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
| `TESTCLEAN1` | queued (`ready-to-build`) | - | Listed in `docs/READY.md`. |
| `SKILLS1` | queued (`ready-to-build`) | - | Listed in `docs/READY.md`. |
| `KITIDS1` | queued (`ready-to-build`) | - | Listed in `docs/READY.md`. |
| `BASERULES1-M` | queued (`ready-to-build`) | - | Listed in `docs/READY.md`. |
| `U1.2` (`U1_STATS`) | queued (`feasibility-check`) | - | Read-only feasibility pass requested in `docs/READY.md`. |
| `U2.2` (`U2_CLASS`) | queued (`feasibility-check`) | - | Read-only feasibility pass requested in `docs/READY.md`. |
| `A4.2` (`INTENT2`) | queued (`feasibility-check`) | - | Read-only feasibility pass requested in `docs/READY.md`. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.
