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
| READY row 17 `F1.2-S1` + `F2.2-S1` | built; rebase + contract re-record next | `claude/f12-s1` | - |
| READY row 18 `P1.2-S1` + `P2.2-S1` | built; rebase, asset terrain seed, contract re-record after row 17 | `claude/p12-s1` | - |
| READY row 20 (R3-W1) | queued | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-106 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-107. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-107: A1.2-S1 + A1.2-S2 (row 15) landed: `item_ap` ruleset number (timeline 0 in data); `stances` flag on the token renderer

- **Spec**: `docs/READY.md` row 15; `docs/sessions/A1_VALUE.md`; FB-55
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Item use costs the ruleset's `item_ap` (`command::item_ap(w, c)` replaces the fixed `ITEM_AP = 1` in `plan.rs`, `round_intent_plan.rs` and `rpgkit_host` `items_of`); a unit that pays no AP pays nothing. `item_ap` is a schema field (rulesets, int 0..2, default 1) read by `value::int`; a row-only int re-saved as `0.0`, so it needed the schema entry (as row 18 did for `move_after_blocks`). The default is 1 under every ruleset (the review caught a first draft that defaulted round-intent rows to 0, a hidden gate); the shipped values are in data via seed `games/emberwake/data/seeds/a12s1.json` (aegis content apply, asset lane): `timeline` `item_ap: 0`; the writer's re-save also states `classic` `item_ap: 1` (the default, as the spec says). So timeline items now cost 0 AP (1 before); no golden plays an item under timeline (timeline_golden pouches are empty, journey pouches are empty, the one sim item run is classic), so the change is real but uncovered. `source::render` / `token_value` take `stances: bool`: off renders `{flow}` / `{disrupt}` empty; every caller passes `true` today, so shipped text and the resave are unchanged. Re-recorded (named): `rules_contract_golden` (+`tunable item_ap int default=1 range=0..2`, +`promoted ITEM_AP -> item_ap`). Tests: `rpgkit_battle/tests/item_ap.rs` (4), `rpgkit_content/tests/stance_tokens.rs` (3); `ruleset_switches.rs` replays `a12s1.json`; `rulesets.rs` schema-range count 11 -> 12. Known gaps: the intent Builder's `Pick::Item` AP refusal and host `items_of` `ap` have no direct test; the `FIELD_RULES` range on `item_ap` duplicates the schema range (loading clamps first, so it only flags raw rows).
- **Recommendation**: GAME_BRIEF items (ITEMUSE1 rule 3): "item use costs the ruleset's `item_ap` (classic 1, timeline 0); a unit that pays no AP pays nothing". HISTORY: "A1.2-S1/S2: timeline items cost 0 AP (`item_ap`, seed `a12s1`); `stances` flag on the token renderer; re-recorded on purpose: `rules_contract_golden`". STRUCTURE / kit README: schema field `item_ap`; `source::render(template, kind, row, rank, stances)`; tests `item_ap.rs`, `stance_tokens.rs`. Spec question (A1_VALUE acceptance 2): the flag exists, but nothing renders with `stances` off, so timeline still prints `{flow}` / `{disrupt}` on the two shipped skills (`skills.json` lines ~1343, ~1368), and with stances off their templates would read "... power, ." (a dangling comma). Which slice wires the ruleset into rendering and fixes those two templates?

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
