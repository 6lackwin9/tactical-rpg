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
| W2-4 (R2-W2) | building: slice 1 of 7 (`field_intent` rule) | worktree | Slices: field_intent; FB-107 text; K2 snap + retire SWEEP; INTENT2-A seed; A4.2 EffectSpec; K1b Field planner + UI; K3 items + INTENT2-U. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-110 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-111. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-111: row 20 (E2.2-S1, T1.2-S1, W.2-S1) landed: purse caps, dialogue `threshold`, `WorldRoster::new` + kit `finish_world`; R3 Wave 1 done

- **Spec**: `docs/READY.md` row 20; `docs/sessions/E_ECONOMY.md`, `T_STORY.md`, `W_WORLD.md`; FB-50a, FB-53a,b, FB-54a, FB-110
- **Kind**: docs-delta
- **Status**: open
- **Finding**: W.2-S1: `WorldRoster::new(d, world)` replaces the 21 struct literals; the after-fight dispatch by fight source (travel, dialogue, story / trigger, else explore) moved from `emberwake_gui` `battle::after_fight` into the kit as `rpgkit_world::battle::finish_world`, called by the GUI's three sites. T1.2-S1: dialogue `stat` conditions accept `threshold` as a pure alias of `min` (`dialogue::stat_threshold`; both keys on one condition is a check error; `gold` conditions do not take it); no schema change (conditions sit inside a JSON-typed field). E2.2-S1 (landed as built per FB-110): `PurseCaps { supplies_max, purse_gold_cap }` (kit `rpgkit_content::source`, defaults 99 / 9,999,999 = today) read from `game.json` (missing or invalid key reads the default; a bad value is a manifest problem); the party reads `PartyState::purse` (never saved); loading clamps gold and supplies above the caps (a save over 9,999,999 gold now loads clamped instead of failing `validate`; negatives still fail); a new game clamps its start purse; the checks bound start gold, start supplies (the inherited default 10 included) and chest supplies by the caps; the cap sentences, shop "left" value and HUD upkeep chip read the caps. With the defaults every message and number is unchanged. `SUPPLIES_START` and new `START_GOLD_MAX` (99999) live in `rpgkit_content::source`. Not touched (content-grant limits, not the purse cap): quest reward 5000 (`check/quest.rs`), chest gold 9999, the runtime quest reward bound (`GOLD_MAX`). 0 golden diffs (every golden). Tests: 24 new across `rpgkit_world/tests/{world_fight, dialogue_threshold, purse_caps}.rs` and `rpgkit_content/tests/{dialogue_conditions, purse_caps}.rs`. With row 20, R3 Wave 1 is complete.
- **Recommendation**: GAME_BRIEF economy: "purse caps are game-wide `PurseCaps` (`game.json` `supplies_max` / `purse_gold_cap`, defaults 99 / 9,999,999); a save over the caps loads clamped; world start values must fit the caps"; GAME_BRIEF story: "dialogue `stat` condition: `min` or its alias `threshold` (not both)". kit README / PORT_NOTES: `rpgkit_world::battle::finish_world` (was the GUI's `after_fight`), `WorldRoster::new`. STRUCTURE: the five test files. HISTORY: "row 20: hardcoded `SUPPLIES_MAX` / `GOLD_MAX` in state and checks replaced by `PurseCaps` (constants kept as defaults); no golden moved". Question: the E2 spec says the caps are shared by `check/{world,map,quest}.rs`; quest was not touched (its 5000 reward limit is a grant limit): intended?

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
