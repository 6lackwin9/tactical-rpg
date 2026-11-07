# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | done (FB-95) | `develop` | 3 parallel lanes + FB-94 additions; checks_golden re-recorded. |
| R1.1-2 `C02-A` | done (FB-96) | `develop` | asset + logic lanes in parallel; phasefold re-recorded. |
| R1.1-3 `R1-BAL` | done (FB-99, FB-100, FB-103) | `develop` | Phase R1.1 complete. |
| READY row 14 `A2.2-S1/S2` | done (FB-102) | `develop` | - |
| READY row 16 `A4.2-K1a` + `A3.2-S1` | done (FB-105; `field_intent` gate moved to W2-4 per FB-106) | `develop` | R2-W1 batch 1 complete. |
| READY row 19 `I1.2-S1` | done (FB-104) | `develop` | - |
| READY row 15 `A1.2-S1/S2` | done (FB-107) | `develop` | - |
| READY row 17 `F1.2-S1` + `F2.2-S1` | done (FB-108) | `develop` | - |
| READY row 18 `P1.2-S1` + `P2.2-S1` | done (FB-109) | `develop` | R2 Wave 1 complete. |
| READY row 20 (R3-W1) | built, held: FB-110 (purse caps location); review fixes M1/M2 in progress | `claude/r3w1-20` | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-109 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-110. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-110: row 20 E2.2-S1 puts the purse caps in `game.json`, which the engine owns and cannot write them

- **Spec**: `docs/READY.md` row 20 (`PurseCaps` "on `game.json` clamped on load"); `docs/sessions/E_ECONOMY.md`; FB-53 ruling; Authority Matrix (schema rule, `docs/decisions/workflow.md`)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Row 20 is built on `claude/r3w1-20` (all three slices; full gate passed, 2493 tests, every golden byte-identical; review: no code blockers) and held there for this. `PurseCaps { supplies_max, purse_gold_cap }` (kit, `rpgkit_content::source`, defaults 99 / 9,999,999 = today) is read raw from `game.json` beside the data folder (`PurseCaps::beside`, via the engine's `manifest::FILE`). But `game.json` is the engine's manifest: `aegis_content::manifest::Manifest::set` refuses unknown keys and rewrites the file from its fixed `KEYS`, so the caps cannot be set through `aegis content` (a hand edit breaks "never raw JSON", and the next manifest `set` would drop it). The keys are also in neither `kits/rpg/schema.json` nor the engine's key list, against the Authority Matrix schema rule. Today the feature is inert (no cap set, defaults everywhere), so nothing plays differently. Build did not touch the engine (naming purse keys there would put a genre concept in the engine).
- **Recommendation**: (A, recommended) an engine manifest extension point: `game.json` may carry kit-owned extra keys that the engine preserves on `set` and that a kit registers (with types and ranges) for `aegis content manifest` to write and check; the purse caps register through it. (B) Move the caps to a kit-side game file or a schema-registered row (for example a `game` kit table in `kits/rpg/schema.json`), changing the FB-53 ruling. (C) Land row 20 now with the caps inert (defaults only) and fix the writer path in a follow-up row. Build default if unanswered: (C), since W2-4 does not depend on it; the T1.2-S1 and W.2-S1 slices are clean either way.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
