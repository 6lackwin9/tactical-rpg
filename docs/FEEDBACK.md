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
| READY row 20 (R3-W1) | next | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-108 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-109. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-109: P1.2-S1 + P2.2-S1 (row 18) landed: terrain `walkable` / `stops_forced_move` are row data; `move_after_blocks` in the schema; R2 Wave 1 done

- **Spec**: `docs/READY.md` row 18; `docs/sessions/P_SPACE.md`; FB-40b, FB-51a,b, FB-70
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Schema terrain fields `walkable` (default true) and `stops_forced_move` (default false); a row without them loads today's rule (deep_water, lava, wall, tree unwalkable; stops = not walkable), not the schema default; `grid.rs`, `forced.rs` and the terrain / hold-tile checks read the row; `move_cost` was already read from the row. A forced move cannot enter a tile that stops forced moves and collides before it (extends today's unwalkable rule to walkable tiles; no row sets that yet). Schema ruleset field `move_after_blocks` (int, default 3, 1..4; loading clamps like every schema number). The writer stores these three keys only when an entry carries them (a re-save of the keyless frozen kit stays byte-identical). Data via seed `games/emberwake/data/seeds/p12s1.json` (aegis content apply, asset lane): explicit values on all 17 terrain rows and `move_after_blocks: 3` on both rulesets, equal to today's behaviour. Goldens: `rules_contract_golden` re-recorded (named; `move_after_blocks` moves from row-only to `int default=3 range=1..4 needs=round_intent`); `checks_golden` named but a re-record gives no diff (no check message changed); 0 battle/sim golden diffs. Landed after rows 15 and 17 (merge: schema.json rebuilt with all three rows' fields; schema-range count 13). Tests: `rpgkit_battle/tests/terrain_rows.rs` (4), `emberwake_battle/tests/terrain_rows.rs` (2), `rpgkit_content/tests/terrain_fields.rs` (5). With rows 14-19 on `develop`, R2 Wave 1 is complete.
- **Recommendation**: GAME_BRIEF terrain / forced movement: "`walkable` and `stops_forced_move` are terrain row fields (a row without them reads the old rule); a forced move collides before a tile that stops forced moves"; GAME_BRIEF rulesets: `move_after_blocks` is a schema number (1..4). STRUCTURE: `terrain.json` `walkable`, `stops_forced_move`; tests above. PORT_NOTES `grid.rs`: `BLOCKED` is only the missing-key fallback. HISTORY: "P1.2-S1 / P2.2-S1: terrain walkability as row data, `move_after_blocks` in the schema; re-recorded on purpose: `rules_contract_golden`; `checks_golden` unchanged". Confirm when a row first sets `stops_forced_move` on a walkable tile that "collides before it" is the rule (a mud tile that ends a push on it would be a different rule). Cleanup later: the writer's given-only rule for `move_after_blocks` is no longer needed now that both rows carry it; the terrain fallback lives in both `source.rs` and `grid.rs` (a shared helper would stop drift).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
