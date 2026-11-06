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
| `TESTCLEAN1` | done | `develop` 41de4e5c | Docs delta FB-39. UN-01 for the old folders. |
| `SKILLS1` | done | `develop` 41de4e5c | Full gate 2215 tests. |
| `KITIDS1` | done | `develop` 41de4e5c | FB-23, FB-24 open; docs delta FB-38. |
| `BASERULES1-M` | done | `develop` c804f8af | Full gate 2235 tests. FB-40, FB-41 open. |
| `U1.2` (`U1_STATS`) | slice 1 done | `develop` c804f8af | Pipeline and pool view, goldens byte-identical; FB-42, FB-43, FB-45. Slices 2-6 wait for READY. |
| `U2.2` (`U2_CLASS`) | slice 1 staged | `staging` | Two share vectors; every class identical at levels 1-10; FB-44, FB-46, FB-47. |
| `A4.2` (`INTENT2`) | audited | - | FB-16 to FB-22 closed; design updating INTENT2. |
| `U3.2` (`U3_GEAR`) | audited | - | Feasible, but the premise is partly stale; about 2M + 1M-L + 1L + 1S; FB-31 to FB-37. |
| `U4.2` (`U4_PASSIVES`) | audited | - | Feasible as a typed view plus compatibility loader, M-L in five slices; FB-25 to FB-30. |

## 2. Feedback Items (Append-Only)

Each entry uses: `### FB-nn: <short title>` with fields `Spec`, `Kind` (`feasibility` | `conflict` | `cost` | `sim` | `docs-delta`), `Status` (`open` | `answered` | `closed`), `Finding`, and `Recommendation`.

FB-01 to FB-59 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-60.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-04 are closed and moved to the archive (2026-11-07). New notes start at UN-05.
