# Design session PK1: corruption in battle (Strain, crisis, Channel) (`PK1.1`, 2026-11-10)

Status: `.1` design written by the Design Session (2026-11-10) from the user's rulings in `docs/specs/ACTS23.md` sections 4 and 5.1 and CANON 2.2; Build's `ACTS23-R5` feasibility check (`docs/READY.md` Section 2 row 8) reviews it before slicing. Release `R5` with PK2 (Calcification, grafts, weapon techniques) and PK3 (roster lifecycle).
Layers: kit (hooks, all story-free: the kit says "meter", "channel", "marrow version"; the names Strain, Inoculated and marrow are game data), game (data rows, Dojo presets).
Extends: `S2` pools (unit meters), `A1` value and cost, `A4` effects, `F3` intent and planning, `I1` AI.

## 1. What already carries it (no new code)

Checked on `develop` @`53962283`:

| Need (CANON 2.2) | Existing hook |
| --- | --- |
| A per-unit meter 0-100 that lives in battle only, starts at the larger of `start` and the holder's floor, resets after battle | `rpgkit_battle/src/meter.rs` (UM1), `unit_meters` rows |
| Gain when an action starts | `skills.meters` `[{meter, delta}]`, paid at the command's start after AP (UM2) |
| Decay at round end: -10 unless the unit gained this round, a further -10 if it held Active Guard | `decay` rows with `unless_gained` and `if_guarded` (UM3) |
| Floor (grafts raise it, PK2) | a passive's `meter` grant `{meter, start, floor}` (`grants.rs:144`) |
| Cap at 100, set to 75 after a crisis | the row's `max`; rule effect `meter_set` (UM5) |
| Bracket effects | `bands` with `effects` (`stat`, `resist`, `stagger_taken`, `skill_potency`, `status_potency`, `crit`) and `statuses` (UM4) |
| Crisis at round end at 100 | a meter `triggers` row `on: round_end` with `when.meter` (held by every holder) |
| Eldritch attacks that add Strain to the target | a skill trigger `on: hit_dealt` with the effect `meter {meter, delta, to: other}` |
| Armour pierce on marrow attacks | skill field `armor_piercing` |
| Aberrant enemies carry Strain, humans never | the meter goes to whoever holds the passive; enemies pay no AP but do pay meters |

## 2. The Strain row (game data, `unit_meters.json`)

`strain`: min 0, max 100, start 0, show `bar`; `decay` `[-10 unless_gained]`, `[-10 if_guarded]`; bands:

| From | Band | Holds (statuses re-granted each round start) and effects |
| --- | --- | --- |
| 0 | Dormant | none |
| 25 | Acute Resonance | `crit` +15; status `haste` (move rate +1); **new** effect `heal_taken` -25 (section 3.1) |
| 50 | Flesh-Tension | status `empower` 25; status `fortify` 4 (armour); `stagger_taken` x 2 |
| 75 | Aberrant Edge | status `empower` 50; **recoil** 10% of damage dealt (a held trigger, section 3.2) |
| 100 | Rupture | the crisis at round end (section 2.1) |

### 2.1 The crisis (CANON 2.2: 1d3, equal odds, seeded, round end)

A `triggers` row on the Strain meter: `on: round_end`, `when: {meter: {strain: ">= 100"}}`, effects `[roll_table (PR5-1, equal weights), meter_set strain 75]`:

1. **Unchecked Spasm**: a full-power strike with the unit's weapon on the nearest unit, ally or foe (ties: the lowest unit id), then `stun` next round. **New** effect `strike_nearest` (section 3.3).
2. **Biological Collapse**: 0 AP next round (**new** `ap` key `set: 0` for the next round, section 3.4), `bleed` 3 rounds, armour rating 0 for the rest of the battle (`expose` at a magnitude that zeroes the rating, `rounds` 99).
3. **Flesh Breach**: +1 Calcification stage. Battle cannot change a campaign track, so a **new** effect `mark` records `{unit, mark: "calcify", value: 1}` in the battle result; PK2's unit track applies it when the battle settles (section 3.5).

## 3. New kit hooks for the Strain layer

1. **Band effect `heal_taken`** (pct, added to `BAND_EFFECTS`): scales healing the holder receives (all heal sources, after the heal formula, rounded down, min 0).
2. **Recoil**: the trigger effect `damage` gains `pct_of_hit` (an int percent of the damage the triggering hit dealt; `to: self`); the Aberrant Edge band holds a `hit_dealt` trigger with it. Recoil cannot down the unit below 1 HP (CANON is silent; Design default, so a crisis is the only self-kill path).
3. **`strike_nearest`** trigger effect `{power: 1.0, then_status, then_rounds}`: a basic weapon strike resolved immediately at round end on the nearest other unit of any side, no AP, no plan; logged as `[Spasm]`.
4. **`ap` with `set`**: `{type: ap, set: 0, rounds: 1}` sets the unit's AP to 0 for its next round's planning (the planner shows why).
5. **`mark`** trigger effect `{mark, value}`: appends to the battle result's per-unit marks (a list the world settle reads); PK1 only records and shows it in the results screen; PK2 consumes `calcify`.
6. **Hazards that add Strain**: arena hazard `effect` and ground hazard `EffectSpec` gain `meter {meter, delta}` (applied only to holders of that meter).

## 4. Channel (ACTS23 5.1, all user rulings)

### 4.1 Rule and fields

- Rule `channel` (requires `round_intent` and `unit_meters`; off in `classic`; on in `timeline`, inert while no unit holds the channel meter, so Act 1 is unchanged).
- Ruleset numbers (tunable): `channel_meter` (`"strain"`), `channel_power` (1.25), `channel_cost` (`[15, 25, 35]`: by AP 1 / 2 / 3+), `channel_basics` (the universal marrow basic id list, game data: one skill), `marrow_actives` (default limit when nothing else sets it; PK2 replaces it with one per installed graft).
- Skill field `channel` (optional object): `none: true` (no marrow version), `only: true` (a marrow-exclusive skill: graft skills, Inoculated keystones, the marrow basic), `cost` (overrides the derived Strain cost), `effects` (extra trigger effects on the marrow version), `power` (overrides `channel_power`).

### 4.2 Who can Channel and what a marrow version is

- A unit can Channel when it holds the `channel_meter` (a Strain carrier). Others never see the opener.
- **Eligible skills**: single intent (the `IR21` derived list has one entry), not neutral (Move, Rest and items never), not `channel.none`. Dual-intent skills are the Cold Steel heavies and have no marrow version (user cap).
- **Marrow version** of an eligible skill: same target, shape, blocks and AP; power x `channel_power`; `armor_piercing` on attacks; pays `channel_cost[AP]` (or `channel.cost`) of the channel meter at its start; adds `channel.effects`; costs no weapon durability (PK2).
- **What a unit has in a Channel round**: its `channel.only` skills (graft skills and an Inoculated keystone, PK2; in PK1 test data, granted by a test passive), the universal marrow basic (`channel_basics`, no slot), the class role basic's marrow version (if single intent), and the marrow versions of the deck actives the player marked as marrow (deck field `marrow`, at most `marrow_actives` of them, single intent only).

### 4.3 The Channel round

1. **Channel opener, a toggle** (user 2026-11-10): a neutral command beside Move, Rest and Fill, **0 AP, 0 blocks**, only for a unit that can Channel. It names no target and sets the round intent to `channel` (the existing `RoundIntent::Channel`); clicking it again returns the round to Cold Steel. It works **only while the unit has nothing planned this round**: once any block is on its timeline, the toggle is locked (it can no longer enter Channel, nor leave it).
2. After the opener, every intent row offers **only marrow versions** (section 4.2); Cold Steel versions are grey with the reason "Channel round".
3. The **first skill after the opener fixes the round's one target** (TG1); later picks must use that target, as in any round (the row each comes from still sets its anchor).
4. Follow-ups and Fill: marrow versions only, in the **Channel order**: `channel.only` skills, then marked actives, then the class basic, then the marrow basic; what cannot fit is Rest.
5. **Rest in a Channel round is "Hold the marrow"** (user 2026-11-10): a valid move, but not a normal Rest: it regains no AP (no Rest AP effect), and a Channel round counts as "gained" for decay even if it holds only Rest, so Strain does not decay at that round's end: the unit holds its bracket on purpose and stays visibly channelling. Ruleset field `channel_rest` (`hold`; kit-generic, other games may pick `normal`).
6. Tells: the plan strip, the unit card and enemy plan badges show the Channel intent from the opener on; the threat intel lists marrow versions as their own names (the skill name with a marrow mark, game term).
7. A round not opened with the opener is Cold Steel; no marrow version is offered.
8. Strain bookkeeping: each marrow action pays at its start; decay skips the round (`unless_gained`, and a Channel round always counts as gained, item 5).

### 4.4 AI

Strain-carrying units (aberrants, and AI-driven heroes) get one AI row: open with the opener when a marrow version scores at least `channel_ai_edge` (tunable, default 1.2) times the best Cold Steel option and the unit's Strain after the round would stay below 100; otherwise plan Cold Steel. Enemies pay no AP but pay Strain, so their crises are real.

## 5. Game data (asset lane, after the kit slices)

- `strain` row (section 2) with the crisis trigger.
- The universal marrow basic, **Marrow Surge** (user 2026-11-10; replaces the Design default of a 1 AP Assault strike): a **Stand self skill**, `channel.only`, that pays Strain to push the marrow: default 0 AP, 1 block, Strain cost 20, grants `empower` 25 until the end of the round, so the rest of the Channel round hits harder at a Strain price; works for every class.
- A test passive `inoculated_test` granting the Strain meter (floor 0) and `channel.only` test skills, for the Dojo and sims until PK2's grafts exist.
- One aberrant test enemy with Strain and a Strain-adding hit, and a Dojo preset "Channel duel".

## 6. Slices (proposed; Build's feasibility check may reorder)

| Slice | Scope | Acceptance |
| --- | --- | --- |
| `PK1-S1` | Section 3 hooks (`heal_taken`, `pct_of_hit`, `strike_nearest`, `ap set`, `mark`, hazard `meter`) | unit tests per hook; **0 golden diffs** (no data uses them) |
| `PK1-S2` | Section 4 kit: rule `channel` and fields, eligibility and marrow versions, the opener toggle, Channel Rest (`channel_rest`), planner rows and Fill order, deck `marrow` and the limit, tells, AI row | unit tests: the opener toggles only while nothing is planned and only for holders, and locks after the first block; a Channel Rest regains no AP and stops that round's decay; rows offer only marrow versions after it; a dual-intent skill never gets one; cost by AP; the target rule; `rules_contract_golden` re-recorded (new ruleset fields), **0 battle and sim golden diffs** |
| `PK1-S3` | Section 5 data and the Dojo preset; balance run (Q-67: crisis rate per battle, Inoculated dominance in short fights, Flesh-Tension as a trap) | data checks clean; a `sim` report in `FEEDBACK.md`; goldens unchanged unless a fixture reads the new rows (named) |

## 7. Fixed vs tunable (`aegis rules --contract`)

Tunable: `channel_power`, `channel_cost`, `marrow_actives`, `channel_ai_edge`, `channel_rest`, Marrow Surge's numbers, the Strain row's bands, decay and crisis rows (data). Locked: the opener's 0 AP / 0 block toggle that locks once a block is planned, the single-intent eligibility, the all-marrow round.

## 8. Defaults Design chose (user may revisit; logged in `docs/DESIGN_REVIEW.md`)

- Recoil cannot down the unit (min 1 HP).
- Spasm targets the nearest unit of any side, ties to the lowest id.
- Biological Collapse's armour 0 is `expose` for 99 rounds.
- The Channel AI edge 1.2 and the "stay below 100" guard.
- ~~The marrow basic is an Assault skill at 1 AP~~: replaced by the user's pick, Marrow Surge (section 5). User 2026-11-10: the other defaults above accepted.
