# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to R2 rows (R1 release, R1.1, R2 Waves 1-2, R2 release) | released (`main` @98ab973a, @3cc73f2b) | `main` | Rows moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10, size). |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (`3353e62b`) | `develop` | - |
| R4 release | released: `main` @32ae1ada ([#58](https://github.com/6lackwin9/tactical-rpg/pull/58), merge commit; tree = develop @ad903f92, which adds a whitespace fix for an MD012 lint error from 92a7282b) | `main` | Ready queue empty; next: Section 3 `R3-STAGE-SEED`, `POST-R4-CLEANUP` when Design queues them. |
| WEBREL1 feasibility | done: FB-147 (read-only audit, no code) | - | Waiting on Design: E1 split (Option A) and slice order. |
| PR4-1 `R3-STAGE-SIM` | done: part 1 (`stage=N`, `55ef14d8`), part 2 (AUDIT-07, `sim_golden` run 11 re-recorded, FB-154) | `develop` | - |
| PR4-2 `POST-R4-CLEANUP` | done: phase A (`feadda4d`), phase B re-freeze (`c49194b2`, FB-152; FB-153 suggestion); item 2 stays held | `develop` | - |
| PR4-3 `WEBREL1-E1a` | done (FB-155): wasm32 check clean, `aegis_core::clock`, clock lint | `develop` | - |
| PR4-4 `WEBREL1-E1b` | done (FB-156; FB-157 suggestion): `aegis_core::vfs`, `aegis_save::store`, wasm `cfg`, fs lint | `develop` | - |
| PR4-8 `SKILLCHECK-FOLD` | done (FB-158 suggestion): fold error when nothing else is reported; ranks normalised before the fold | `develop` | - |
| PR4-5 `WEBREL1-E2` | done (FB-159; FB-160 suggestion): `shell::start_web`, async WebGL2 renderer, DPR, audio on first gesture; `web_min` passes in headless Chromium | `develop` | - |
| PR4-6 `WEBREL1-G1` | done (FB-161; FB-162 suggestion): the game runs in a browser (web entry, bundle, `dev_tools`, `fc.sh web`, smoke 40/40, WEBREL1 6.2) | `develop` | - |
| PR4-7 `WEBREL1-P1` | done (FB-163; FB-164 suggestion): advisory `web` CI job, `web.yml` itch.io deploy; GitHub-side acceptance observable at the next release | `develop` | - |
| PR4-9 `WEBOPT1` | done (FB-166): pinned Binaryen version_125 on CI, `wasm-opt` version guard in `fc.sh web`; first real `wasm-opt` run at the next release PR | `develop` | Post-R4 queue PR4-1..PR4-9 complete (@381b8746). Watch at the next release PR: the `web` job (`web-build`, pinned `wasm-opt` sizes, smoke on the optimised build). |
| PR5-1 `ROLLTABLE1` | done (FB-168; FB-169 suggestion): trigger effect `roll_table` (weighted rows on the seeded battle RNG), content check, schema help; 0 golden diffs | `develop` | Post-R4 queue: PR5-1, PR5-2, ACTS23-R5 feasibility (FB-167)
| PR5-2 `PHASEFX1` | done (FB-170; FB-171 questions): encounter phase effects `side` (hostile_all, party, enemy) and `swap` (keep_pct, full), content checks; 0 golden diffs | `develop` | PR5-1, PR5-2 done; FB-167 feasibility open

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-166 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-167. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-167: ACTS23-R5 feasibility (PK1, PK2, PK3)

- **Spec**: `docs/specs/ACTS23.md` sections 2, 4, 5.1, 5.2; `docs/sessions/PK1_CORRUPTION.md`; READY Section 2 row 8
- **Kind**: feasibility
- **Status**: open
- **Finding**: All of R5 is buildable. PK1 can be sliced now and holds at 0 battle, sim or phasefold diffs: the only golden it moves is `rules_contract_golden` (new rule and numbers). PK2's item instances are the one L+ item, because about 300 `stash` / `equipment` sites expect plain ids (`state.rs:61`, `:951`). PK2 and PK3 have no `.1` yet, so their slices wait on Design. Two notes on PK1 section 1, neither needing code: a skill's own `triggers` do not take `hit_dealt` (`SKILL_TRIGGER_EVENTS`, `rpgkit_data/src/lib.rs:1818`), so a hit that adds Strain is a skill `riders` row with `meter {to: other}` (`trigger.rs:531`); ground hazards use flat fields, not an `EffectSpec` (`HazardDef`, `lib.rs:1991`). ACTS23 section 2 line numbers have drifted by about 11 lines.

| Package | Hook | Code sites | Cost | Golden risk | Flag-able at 0 diffs |
| --- | --- | --- | --- | --- | --- |
| PK1 | `strain` row and bands (data) | `unit_meters.json`; `meter.rs:131` (start), `:359` (pay), `:410` (decay) | S (asset) | none; phasefold reads shipped data but no fight holds `strain` | yes (no holder) |
| PK1 | 3.1 `heal_taken` band effect | `BAND_EFFECTS` `lib.rs:1862`; `grants.rs:81` `Target`, `band_ok`; `world.rs:374` `heal` | S | checks_golden only if it joins the passive effect list (the "(use stat, ...)" message) | yes, as a band-only effect |
| PK1 | 3.2 to 3.5: `pct_of_hit`, `strike_nearest`, `ap set`, `mark` | `TRIGGER_EFFECTS` `lib.rs:1829`; `trigger.rs:742` (`ap`, party side only), `:813` (`damage`); `mark` into `HeroEnd` (`roster.rs:86`), `Tally` (`rpgkit_world/src/battle.rs:246`) | M (`strike_nearest`: a resolver strike outside the plan) | none | yes (no data) |
| PK1 | 3.6 hazard `meter` | `ARENA_EFFECT_KEYS` `lib.rs:1875`, `arena.rs:225`; `ground.rs:370` | S | none (kinds not in checks_golden) | yes |
| PK1 | 4 Channel opener, marrow versions, planner, Fill, tells, AI | `round_intent.rs:25`, `:126` (`Pick`), `:221` (`target_problem`), `:272` (`set_intent`); `round_intent_plan.rs:496`, `:691`, `:919` (fill), `:2131` (menu rows); `skill.rs:375` (version read as Field's); `meter.rs:334` / `:347` / `:359` (cost); `ai.rs`; `intent.rs` (tells); `REGISTER` `ruleset.rs:83`; GUI `emberwake_gui/src/battle/` | L (kit) + M (game GUI) | `rules_contract_golden` re-record; timeline, sim, phasefold 0 (opener only for holders) | yes |
| PK2 | (d) unit-scope campaign track | refusal at `check/campaign.rs:287`; `campaign/meter.rs:59`; `UnitState` `state.rs:48` (an additive saved key); stage passives at `roster.rs:66` `field_unit`; `depart` with life `lost` `roster.rs:389`; `mark` settle at `:292` | M-L | none (journey uses frozen data; additive key) | yes (no data) |
| PK2 | (e) graft body slots: a skill and a meter floor | `LOADOUT_SLOTS` `lib.rs:26`; `gear_ok` `grants.rs:153`; `unit.rs:610`, `:639`; floors combine by max at `meter.rs:143-147` | M | checks_golden if grafts add an item type or slot value, or change the gear "(use ...)" list | yes, with a separate `body` field and a body-only effect list |
| PK2 | (f) recipe service | `SERVICES` `explore.rs:315`; purse, stash; member effects | M | none | yes |
| PK2 | weapon techniques per family | `apply_family_bonuses` `unit.rs:566`; `level_of` `stats.rs:222`; held like `gear_skills`; Cold Steel gate needs PK1's Channel round | M | none | yes |
| PK2 | weapon instances and durability | `state.rs:61`, `:951`, `:755`; `gear.rs:277`; `shop.rs`; loot `battle.rs:328`; `save.rs:18` (VERSION 13, `STEPS` empty); battle cost and dulling in `skill.rs`; return through `HeroEnd` | L+ (world L, battle M) | journey_golden if the `equipment` or `stash` shape changes or VERSION goes up | yes if instances are additive and act-gated (question 6) |
| PK3 | bleed-out and revive | `bleed_out_rounds` `lib.rs:1339` (unread); cause `bled_out` already exists (`lib.rs:1263`); `world.rs:374` (no heal on a downed unit), `:487` (a downed unit's cell is free) | M | none (no golden passes a rule) | yes |
| PK3 | recruit generator (ROSTER1-P) | `roster.rs:523` (`pool` = 0); `recruit` effect; `UnitState` has no `name` field | M-L | none | yes |
| PK3 | procedural keepsakes | `keepsakes_by_class` / `keepsake_name` `lib.rs:1393-1412` (unread); `depart` `roster.rs:408-415` | S-M | none | yes |

Dependencies: PR5-1 `roll_table` (the crisis); PR5-2 is not in R5.

- **Spec questions**:
  1. **What is a Channel round?** PK1 4.3 sets `RoundIntent::Channel`, but an intent persists across rounds (IN2), a change ends intent-bound buffs (IN6, `round_intent.rs:283`), and the target kind, deck order, follow-ups and AI rows are all read per intent. Options: (a) a `channel` flag on the round state, cleared at round start; the first skill's row stays the intent, and entering a Channel round counts as an intent change, so bound buffs end; (b) the intent value itself. **Recommend (a)**: tells show Channel when the flag is set.
  2. **Retire the old Channel path?** Today a ruleset can list `channel` in `intents` (`ruleset.rs:115`) and a skill can set `round_intent: channel` (`check/skill.rs:1008`). No data uses either. **Recommend retiring both** in favour of rule `channel`, with a HISTORY row and one contract re-record.
  3. **Timeline default**: ACTS23 5.1 says "off in timeline until Act 2 data", while PK1 4.1 says "on in timeline, inert". **Recommend PK1's wording**, with `channel` added to the shipped `timeline` row in the PK1 data slice.
  4. **"First pick"**: `Pick::MoveHere` usually comes first. **Recommend "before the first skill or item pick"**, so Move and Rest stay neutral.
  5. **Biological Collapse on enemies**: `ap set 0` does nothing to enemies, which pay no AP and get no `ap` effects (`trigger.rs:743`). **Recommend a 1-round `stun` for units that pay no AP.**
  6. **Graft floors**: floors combine by max, so 4 grafts at +10 give 10, not 40, and the Rock Salt flush (-10, Q-43) needs a per-member offset. **Recommend an additive body-only `meter_floor` grant plus an offset on the unit track.**
  7. **Durability switched on by act**: one campaign world (Q-A2 (a)) keeps one ruleset, so a ruleset flag cannot turn on at Act 2. Options: (a) manifest `durability.from_act` plus kit rule `durability`; instances are created at the act line and on later acquisitions, so Act 1 saves are untouched and VERSION stays; (b) the spec's VERSION bump with a migration. **Recommend (a).**
  8. **`channel_basics`** repeats the meter row's `skills`, which already open with no slot (UM6, `meter.rs:157`). **Recommend the `strain` row's `skills`.**
  9. **PK2 and PK3 have no `.1` yet.** Open items: technique lists, recipe rows, recruit stat ranges, quirks and names, revive when the downed cell is taken, and naming procedural keepsakes (the stash holds ids only). **Recommend PK2 `.1` and PK3 `.1` before slicing them.**
- **Recommendation**:
  - `PR5-3` (kit, M): PK1 hooks 3.1 to 3.6; 0 diffs.
  - `PR5-4` (kit, L): rule `channel`, skill field `channel`, eligibility, the marrow version in resolve and cost, the opener and planner rows, the Fill order, deck `marrow` and its limit; `rules_contract_golden` only.
  - `PR5-5` (kit, M): Channel tells and the AI row; 0 diffs.
  - `PR5-6` (game, M): GUI opener and greyed rows; 0 diffs.
  - `PR5-7` (asset, S-M): the `strain` row, the marrow basic, test passive and enemy, the Dojo preset, and `channel` in `timeline`; phasefold is the canary; then a balance run.
  - `PR5-8` (kit, M-L): unit track and the `mark` settle; 0 diffs.
  - `PR5-9` (kit, M): body slots and the additive floor; 0 diffs.
  - `PR5-10` (kit, M): recipe service; 0 diffs.
  - `PR5-11` (kit, M): bleed-out and revive; 0 diffs.
  - `PR5-12` (kit, M): weapon techniques, after PR5-4; 0 diffs.
  - `PR5-13` (kit, L, splittable): weapon instances, then durability; 0 diffs under question 7 (a).
  - `PR5-14` (kit, M-L): recruit generator; `PR5-15` (kit, S-M): procedural keepsakes; both 0 diffs.
  - PR5-3 to PR5-7 can start now; PR5-8 onward wait for PK2 `.1` and PK3 `.1`.

### FB-168: PR5-1 `ROLLTABLE1` landed (`roll_table` trigger effect)

- **Spec**: READY Section 1 row PR5-1; `docs/specs/ACTS23.md` PK1 hook (a) (the `PR5-1` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `rpgkit_data::TRIGGER_EFFECTS` gains `roll_table` (`rows` of `{weight, effects}`; `ROLL_TABLE`, `ROLL_ROW_KEYS`, `ROLL_WEIGHT = (1, 1000)`). Every trigger path ends in `trigger::run_effects` (skill, passive and item triggers, riders, deferred `ally_hit` effects, meter `rules` crossings), so one branch at its top covers all of them: `trigger::roll_row` draws once from the seeded battle stream (`randi_range(1, total)`) only when a table fires (no draw for no rows or a total of 1) and runs the picked row's effects through `run_effects` with the same event, holder and `rules` depth (each effect keeps its own `to`). Content check `roll_table_errors` (`rpgkit_content/src/check/hooks.rs`): rows a non-empty list; row keys `weight` and `effects` only; `weight` a whole number 1..1000; 1..4 effects, each through `effect_errors` (`meter_set` allowed on meter rules); a nested `roll_table` or `redirect` refused; `to` on the table is an unknown key. Schema help on `skills.triggers` and `unit_meters.rules`. Tests: `rpgkit_battle/tests/roll_table.rs` (5: same seed same row, weights followed, draws only when a table fires, row effects with their own `to`, meter-rule crossing) and `rpgkit_content/tests/roll_table_checks.rs` (4, 19 error cases one error each). Gate 2757 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer clean (3 low, below).
- **Recommendation**: `kits/rpg/README.md` (after the HOOKS1-M line): "ROLLTABLE1: the trigger effect `roll_table` (`rows` of `{weight 1-1000, effects}`) picks one row by weight on the seeded battle RNG (`trigger::roll_row`, one draw only when a table fires) and runs its effects as the trigger's own, in skill, passive, item and rider triggers and meter rules; no table or redirect inside a row." `engine/docs/authoring/combat_hooks.md`: add `roll_table` to the trigger effects list (:124) and the meter-rule effects (:179). GAME_BRIEF trigger effects: add `roll_table` (ACTS23 PK1 hook (a); no shipped data uses it yet). No HISTORY row (no golden re-recorded); STRUCTURE unchanged.

### FB-169: PR5-1 readings beyond the row text (for confirmation)

- **Spec**: READY Section 1 row PR5-1 (@ the `PR5-1` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (1) the row says `weight: int >= 1`; the check caps it at 1000 (kit checks state a range; the sum cannot overflow). (2) A nested `redirect` is refused: a redirect claims the hit before triggers run, so in a table it would do nothing. (3) No row-count limit (the row gives none). (4) The roll writes no log line of its own; the picked row's effects log as usual. (5) Review L1: on an `ally_hit` trigger a top-level `guard` shapes the hit, but a `guard` inside a table rolls at the end of the action (with the trigger's other non-guard effects), after the damage; the check allows it and no data uses it (PK1 crisis rows fire on `round_end`). (6) Process: `feedback_check.py archive` split FB-163 at a `## ` heading inside its fenced `~~~markdown` block, leaving the tail in FEEDBACK; restored by hand into FEEDBACK_CLOSED (this push).
- **Recommendation**: keep (1)-(4) as built. For (5), Option A (recommended): the check refuses `guard` inside a `roll_table` on an `ally_hit` row ("put guard at the top level"), a small follow-up row; Option B: document "on `ally_hit` a table rolls at the end of the action" in the help. Optional: a row cap (e.g. 8) for editor tables. For (6), a small `tools/planner` fix row: the archive splitter skips headings inside fenced blocks.

### FB-170: PR5-2 `PHASEFX1` landed (encounter phase effects `side` and `swap`)

- **Spec**: READY Section 1 row PR5-2; `docs/specs/ACTS23.md` PK6 hooks (l), (m) (the `PR5-2` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `side` {units 1..6, to: `hostile_all` / `party` / `enemy`}: a `hostile_all` rogue stays in `w.enemies` (the enemy AI plans it, the verdict and `a_side_is_wiped` count it as a foe) and `World.sides` (`Sides`, rebuilt by `World::set_side`) gives each enemy-side unit its own foe and friend lists while a rogue exists, read at the top of `foes_of` / `friends_of` (the ~70 callers keep their signatures; with no rogue the old path runs unchanged). `party` / `enemy` move the unit to the end of that side (party side = guest). A turned unit leaves its morale group; joining the party side drops `unkillable`. `swap` {unit, to, hp: `keep_pct` / `full`}: the `to` rows are built at battle start; at fire time every placed unit of the ref is replaced in place (same unit id, cell, side and encounter ref): HP `roundi(new_max * hp / max)` clamped 1..max or full, AP capped, statuses end, the new AI profile is the base, meters (`unit_meters`) and the default intent (`round_intent`) restart, poise refills, unlanded actions are cancelled at a block start, pouch skills join the catalog, the name is numbered if taken. `[Side]` / `[Swap]` log cells; metrics `swap_cell`; `Session::fates` walks the spawned enemy list. Content checks `side_errors` / `swap_errors` (units.side for a row's side; world encounters through `character_side`); the world sim maps a swap's `to`. Tests: `rpgkit_battle/tests/phase_side_swap.rs` (12) and `rpgkit_content/tests/phase_side_swap_checks.rs` (5). Gate 2774 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer, 2 medium (morale group, `unkillable` on swap) and 5 low fixed; 1 low (metrics) documented.
- **Recommendation**: `kits/rpg/README.md`: "PHASEFX1 (ACTS23 PK6 l, m): encounter phase effects `side` {units, to: hostile_all, party or enemy} (a `hostile_all` rogue stays on the enemy side, fights every other unit through `World.sides` and counts as a foe for the win objectives; a turned unit leaves its morale group; joining the party side drops unkillable) and `swap` {unit, to, hp: keep_pct or full} (in place: same unit id, cell, side and encounter ref; HP share roundi; statuses end; meters and default intent restart; new AI and deck from the next planning); fates follow the spawned enemy list; `[Side]` / `[Swap]` log cells; content checks; one cell per unit (2 x 2 is hook n)." STRUCTURE (rpgkit_battle row): "PHASEFX1: `encounter.rs` side/swap phase effects, `swap_hp`; `world.rs` `Sides`, `set_side`, `swap_unit`; `session.rs` `phase_side`, `phase_swap`; `meter.rs` `restart_unit`; tests `tests/phase_side_swap.rs`." GAME_BRIEF (encounters / boss phases): "Phase effects also `side` (turn units hostile to all or to a side) and `swap` (replace a unit in place, keeping its HP share or full HP)." No HISTORY row.

### FB-171: PR5-2 readings beyond the row text and four questions

- **Spec**: READY Section 1 row PR5-2 (@ the `PR5-2` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Built readings: (1) "the id of a side" is `party` or `enemy` (the battle has two sides; `neutral`, a valid `units.side`, is refused). (2) Heroes, reserved and off-board units are never turned or swapped (as `kill`); the checks also refuse a hero, a protected guest or the `slay` target in `side`. (3) A swapped unit keeps its encounter ref: later effects, triggers, fates, rewards, morale keys and `unkillable` follow the old ref; earlier phase `ai` effects on it still apply on the new base. (4) `swap` hits every placed unit of the ref (as `status` / `ai` / `deck`). (5) "Deck and AI from the next round": unlanded actions are cancelled at a block-start swap; the new deck and profile plan from the next planning (at block 0 the current round's, as the `deck` effect). (6) Trigger limits and facing carry over a swap. (7) Metrics keep start sides, so hits between a turned unit and its old side book as friendly fire (balance still holds; documented on `BattleMetrics`). (8) The static swap side check uses the roster side, even after an earlier `side` turn.
- **Recommendation**: keep (1)-(8) as built. Questions, each with the built default: (a) a unit turned mid-round keeps the actions it already booked, even against its new friends (default: keep; alternative: cancel its unlanded actions as a swap does); (b) accept `neutral` as a `side` target (default: no, two sides); (c) can a hero be turned (default: no, the player controls heroes); (d) node 20 Stage V hostility (PK1 `.1`) reuses `side: hostile_all`, so (a) decides whether a breaching unit's booked actions still land.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
