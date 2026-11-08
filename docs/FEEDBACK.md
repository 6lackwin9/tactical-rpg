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
| READY row 20 (R3-W1) | done (FB-111) | `develop` | R3 Wave 1 complete. |
| W2-4 (R2-W2) | slices 1, 2, 3, 5, 6 done (FB-121); slice 4 (`brace`, consumables, `tendril_flail` text; FB-119) next; slice 7 (K3 + INTENT2-U) after | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-120 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-121. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-121: W2-4 slice 6 (A4.2-K1b) landed: Field and shaped Assault rows, row-aware planner, Field UI

- **Spec**: `docs/READY.md` row W2-4 (K1b); `docs/specs/INTENT2.md` IR2-IR7, IR10, IR15, IR17, IR19, IR21-IR23; `docs/sessions/A4_EFFECTS.md` section 2; `docs/sessions/A3_SHAPES.md`; FB-106, FB-120
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Under rule `field_intent` (on in the shipped timeline; classic and round-intent-only play unchanged): `RoundIntent::Field` (tile target); skills get rows from `intent::allowed` (primary first; the auto order holds primaries only, `deck.order` any row, follow-ups share one row with their opener); the first pick's target picks the row (an enemy -> Assault, a tile -> Field, yourself -> Stand when allowed; `(Assault, tile)` is refused under the rule; `(Support, yourself)` stays for items until K3). Field round: one source tile for all blocks (the planner walks into the band; ARC / CONE need a neighbouring source), each family fills once a round, derived Field bonus (ARC 5 across, LINE 4 long, thrown reach +1, hazards +1 round, capped at 5; constants for now). Shaped Assault row (`PlanStep::area_on`): target-locked at impact (chases 1 tile when out of range at its start, misses if the target left reach, died or hid), `splash_mult` off the target, status on the target only, no hazard, foes only, `+assault_shape_blocks` wind-up capped at the round's blocks. Stand rings are self-anchored and cast only with an enemy in them. New row-only ruleset numbers `splash_mult` 0.5 (0.25..1.00, FB-120) and `assault_shape_blocks` 1 (0..2). Field `power` bonus x1.25 (`skill::FIELD_POWER_MULT`, FB-120) where the derived bonus is `power` (a Field skill with no hazard, not ARC / LINE, reach <= 1; no shipped skill yet). Content check: a dual Field / Assault skill must fit `blocks + recovery_blocks + assault_shape_blocks <= 6` (shipped skills land at 6). Deck orders and the saved-deck check read the derived rows and accept `field`. Data via seed `games/emberwake/data/seeds/w24k1b.json` (aegis content apply): 5 units' deck-order skills moved from `assault` to their derived row list (`founder_hollis` `tripwire_snare`, `tallow_dredger` `scalding_grease_pot`, `founder_elyse` `phosphorus_pot`, `inquisitorial_purifier` `pyre_nozzle_sweep` -> `field`; `calcified_amalgam` `tendril_flail` -> `guard`; appended, so `tallow_dredger`'s `assault` is now `[]` and it falls back to its automatic order). AI: a minimal Field row (a Field round when a source covers `area_threshold` foes, not two rounds running; hidden foes not counted); AI Guard rows open with the Defend skill under the rule; full AI rows are W2-5. UI: the battle menu draws a Field row from what the battle offers; a dual skill has a cell in each row (on an enemy, Assault locks on it and Field aims at its tile; on a tile, Field only); the arrow keys move a Field round's source one tile (refused, round unchanged, if it no longer fits); planned area tiles show in the row's colour (Field marks fixed, Assault on its target, Stand around the hero), hover previews them, the hover card names the row's bonus or the lock / splash / extra block; Field's colour is the theme's `check_on`; the Dojo planner lists Field and locks rows the opener lacks; the party deck order has a Field column on derived rows. Re-recorded (named): `rules_contract_golden` (+2 tunables, +2 promoted); per FB-120 (3) the frozen `timeline_kit.json` timeline row lists `field_intent`, re-recording `timeline_golden` (3 of 15 `tl_skirmish` fights from round 3: the Archer's Field Dressing now derives Support + Stand, so after one heal its Stand fill guards instead of healing again and resting; outcomes unchanged) and `sim_golden` (run 21: rules list, Support / Guard shares swap, +2 guard blocks); phasefold, checks, rules, journey and resave unchanged. No `tl_` deck holds an area skill yet, so the goldens exercise derived rows but no Field round. Balance (`sim feel 5 ruleset=timeline seeds=40`, shipped data, 0 plan failures): windup 65, reach 75, armour 65, flank 100, skirmish 87.5 (was 92.5; 4 capped, 1 lost), overall 78.5 % (H6: 79.5). Tests: `rpgkit_battle/tests/field_rows.rs` (21), `field_intent.rs` (+1), `rpgkit_content/tests/timeline_fields.rs` (+2), `rpgkit_world/tests/deck_rows.rs`, GUI `battle/tests/field_rows_tests.rs` (6+), Dojo and party tests; `ruleset_switches.rs` replays pre-W2-4 seeds with the five orders set aside.
- **Recommendation**: GAME_BRIEF intents / UI: the paragraph above, condensed. kit README (`kits/rpg/README.md:16`): `RoundIntent::Field`, the row-aware planner, `PlanStep::area_on`, `splash_mult` / `assault_shape_blocks`, `FieldBonus`. PORT_NOTES (`engine/docs/PORT_NOTES.md:38` still says `rpgkit_rules::intent` has no caller until K1b): it is now called by the planner. STRUCTURE: the new test files. HISTORY: "W2-4 slice 6 (A4.2-K1b): Field and shaped Assault rows, row-aware planner and Field UI under `field_intent`; re-recorded on purpose: `rules_contract_golden`; 5 deck orders moved to derived rows (seed `w24k1b`)". Spec question (deviation): the row and A4_EFFECTS say "WASD / source tile" for Field aiming; Build used the arrow keys, because A (Autoplay) and S (Self target) are battle keys; tile clicks still aim Field. Accept arrows, or reassign A / S during a Field round? Also confirm: AI Guard rows open with the Defend skill; an empty Stand ring is not cast; `tallow_dredger`'s empty `assault` list (drop the key instead?). Later cleanup: the GUI's tile ghost order and party deck-order helpers copy kit rules (move into the kit).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
