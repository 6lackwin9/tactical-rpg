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
| READY row 16 `A4.2-K1a` + `A3.2-S1` | next: FB-101 answered (4 ARC / 3 RING); rework on the parked branch | `origin/claude/a42-k1a` @e07054b0 | - |
| READY row 19 `I1.2-S1` | done (FB-104) | `develop` | - |
| READY rows 15, 17, 18, 20 (R2-W1 batch 2, R3-W1) | queued | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-103 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-104. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-104: I1.2-S1 (row 19) landed: ai.rs reads statuses through S1 table predicates; status table gains `rating` and `targeting`

- **Spec**: `docs/READY.md` row 19 `I1.2-S1`; `docs/sessions/I_AI.md` section 1 item 5; FB-52 ruling
- **Kind**: docs-delta
- **Status**: open
- **Finding**: The 8 status-id literals in `rpgkit_battle/src/ai.rs` are gone; the AI reads `pulls_targeting` (taunt: `targeting` primitive with `targeting: pull`, stealth excluded), `counters` (riposte: `reactive`), `heals_over_time` (regen: a buff `tick`), `stops_moving` (stun, root: `locks_movement`, a lock covering move blocks 1-2 or Move itself; stagger [1, 4] excluded), `raises_armor` / `lowers_armor` (fortify / expose: `modifies_rating(Armor)` buff / debuff) and `lowers_move_rate` (slow: `modifies_rating(MoveRate)` debuff; haste excluded, as slice 1 says). The buff/debuff qualifiers keep exact parity with the one-directional literals; slice 2's scorer covers the other directions. To back `modifies_rating` without re-listing ids, the compiled kit table `statuses.json` gains two columns on every row: `rating` (`attack_power` weaken/empower, `armor` fortify/expose, `move_rate` haste/slow, `guard` guard; on every `modifier` row and only there) and `targeting` (`pull` taunt, `hide` stealth; on every `targeting` row and only there), both parse-checked. `Book::granting` / `ally_buffs` take a predicate. 0 golden diffs. Test `rpgkit_battle/tests/ai_status_reads.rs` (2): each call-site read equals the old literal set over all 18 rows. Out of scope, still literal: `unit.rs` `move_per_block` names `haste` / `slow`. Process note: the H6 tuning commit `7d67fbe9` (FB-103) got a post-hoc read-only review here: data diff exactly the 5 FB-98 values, writer form, the `ruleset_switches.rs` replay is legitimate, no fixture moved; its gate line ran `fc.sh test` rather than the full gate (this row's full gate on top of it passed).
- **Recommendation**: Confirm the column names and values `rating` / `targeting` (now part of the compiled table). GAME_BRIEF statuses (line about row columns): add `rating` and `targeting`; "The AI names no status id (`I1.2-S1`); haste / weaken / empower stay unscored until `I1.2-S2`." STRUCTURE / `kits/rpg/README.md` S1 queries: add `has_targeting_pull`, `has_reactive_counter`, `has_tick`, `locks_movement`, `modifies_rating`, `is_buff`; the same lines name queries that do not exist in `status.rs` (`armor_delta`, `move_rate_delta`, `blocks_action`, `stops_move`, `draws_aggro`, `counters_melee`, `holds_poise`, `conceals`, `skill_power_mult`): replace them. STRUCTURE: test `ai_status_reads.rs`. HISTORY: "I1.2-S1: the 8 status-id literals in `ai.rs` became S1 table predicates (1-to-1, no golden re-record); the status table gained `rating` and `targeting` columns".

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
