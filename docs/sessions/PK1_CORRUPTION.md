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
| Cap at 100, set to 50 after a crisis (user 2026-11-10; was 75, CANON Q-62) | the row's `max`; rule effect `meter_set` (UM5) |
| Bracket effects | `bands` with `effects` (`stat`, `resist`, `stagger_taken`, `skill_potency`, `status_potency`, `crit`) and `statuses` (UM4) |
| Crisis at round end at 100 | a meter `triggers` row `on: round_end` with `when.meter` (held by every holder) |
| Eldritch attacks that add Strain to the target | a skill `riders` row with the effect `meter {meter, delta, to: other}` (a skill's own `triggers` do not take `hit_dealt`, `FB-167`) |
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

A `triggers` row on the Strain meter: `on: round_end`, `when: {meter: {strain: ">= 100"}}`, effects `[roll_table (PR5-1, equal weights), meter_set strain 50]`:

1. **Unchecked Spasm**: a full-power strike with the unit's weapon on the nearest unit, ally or foe (ties: the lowest unit id), then `stun` next round. **New** effect `strike_nearest` (section 3.3).
2. **Biological Collapse**: 0 AP next round (**new** `ap` key `set: 0` for the next round, section 3.4; a unit that pays no AP, such as an enemy, gets a 1-round `stun` instead, `FB-167` Q5), `bleed` 3 rounds, armour rating 0 for the rest of the battle (`expose` at a magnitude that zeroes the rating, `rounds` 99).
3. **Flesh Breach**: +1 Calcification stage, **taking hold at once** (user 2026-11-10): the new stage's effects apply to the unit from the next round on (the crisis rolls at round end) for the rest of the battle (PK2 gives each stage a slotless passive, so the battle grants that passive now), and the **new** effect `mark` records `{unit, mark: "calcify", value: 1}` in the battle result so PK2's track saves the stage when the battle settles (section 3.5). A Flesh Breach into **Stage V** turns the unit hostile on the spot (the lore's "turns hostile" fate, through the `PR5-2` phase effect `side` with `hostile_all`, reused as a trigger effect: the same mechanic as the corrupted founder at Act 3 node 20, the Silent Ossuary, CANON 3.6); it is lost when the battle settles.

## 3. New kit hooks for the Strain layer

1. **Band effect `heal_taken`** (pct, added to `BAND_EFFECTS`): scales healing the holder receives (all heal sources, after the heal formula, rounded down, min 0).
2. **Recoil**: the trigger effect `damage` gains `pct_of_hit` (an int percent of the damage the triggering hit dealt; `to: self`); the Aberrant Edge band holds a `hit_dealt` trigger with it. Recoil cannot down the unit below 1 HP (CANON is silent; Design default, so a crisis is the only self-kill path).
3. **`strike_nearest`** trigger effect `{power: 1.0, then_status, then_rounds}`: a basic weapon strike resolved immediately at round end on the nearest other unit of any side, no AP, no plan; logged as `[Spasm]`.
4. **`ap` with `set`**: `{type: ap, set: 0, rounds: 1}` sets the unit's AP to 0 for its next round's planning (the planner shows why).
5. **`mark`** trigger effect `{mark, value}`: appends to the battle result's per-unit marks (a list the world settle reads); PK1 records and shows it in the results screen; PK2 consumes `calcify`. With PK2, a `calcify` mark also grants the next stage's passive for the rest of the battle (and at Stage V applies `side: hostile_all`), so the stage bites in the fight.
6. **Hazards that add Strain**: arena hazard `effect` and the ground hazard row (flat fields on `HazardDef`, not an `EffectSpec`, `FB-167`) gain `meter {meter, delta}` (applied only to holders of that meter).
7. **`unmark` item effect (the in-battle undo of a Flesh Breach, user 2026-11-10)**: `{mark: "calcify"}` removes the target's latest `calcify` mark of this battle, so the stage is **not saved** when the battle settles, while the stage's effects stay until the battle ends (a temporary boost). On a unit that a Stage V breach turned hostile, it also **calms it**: the hostility ends at the next round start (the unit returns to its side, keeping its previous stage). The item may target such a unit although it is hostile (it was an ally at the battle's start). Carried by **Quenched Cinder Salt** (game data, CANON 2.5, Q-43, Q-69): a rare one-charge pouch item (loot, costly at the Slab, a few per act). **Also a skill** (user 2026-11-10): the **priest's (Heretic Acolyte's) keystone** carries a powerful in-combat skill with the same `unmark` effect on an ally, gated so it cannot be used every time: default **once per battle** (trigger `limit: battle` style, a skill field `uses_per_battle`) and a high cost (default 3 AP and 4 blocks; numbers by the balance lane), so a priest build is how a cell keeps its grafted units from calcifying. It is the priest's **Inoculated** keystone (user 2026-11-10: Cold Steel keystones are tied to weapons for every class, Inoculated keystones to corruption).

## 4. Channel (ACTS23 5.1, all user rulings)

### 4.1 Rule and fields

- Rule `channel` (requires `round_intent` and `unit_meters`; off in `classic`; on in `timeline`, inert while no unit holds the channel meter, so Act 1 is unchanged; `channel` joins the shipped `timeline` row in the PK1 data slice, `FB-167` Q3). It **retires the old Channel path** (`FB-167` Q2): `channel` leaves the ruleset `intents` list (`ruleset.rs:115`) and skills can no longer set `round_intent: channel` (no data used either; one `rules_contract_golden` re-record and a HISTORY row).
- Ruleset numbers (tunable): `channel_meter` (`"strain"`), `channel_power` (1.25), `channel_cost` (`[15, 25, 35]`: by AP 1 / 2 / 3+), `marrow_actives` (default limit when nothing else sets it; PK2 replaces it with one per installed graft).
- Skill field `channel` (optional object): `none: true` (no marrow version), `only: true` (a marrow-exclusive skill: graft skills, Inoculated keystones, the marrow basic), `cost` (overrides the derived Strain cost), `effects` (extra trigger effects on the marrow version), `power` (overrides `channel_power`).

### 4.2 Who can Channel and what a marrow version is

- A unit can Channel when it holds the `channel_meter` (a Strain carrier). Others never see the opener.
- **Eligible skills**: single intent (the `IR21` derived list has one entry), not neutral (Move, Rest and items never), not `channel.none`. Dual-intent skills are the Cold Steel heavies and have no marrow version (user cap).
- **Marrow version** of an eligible skill: same target, shape, blocks and AP; power x `channel_power`; `armor_piercing` on attacks; pays `channel_cost[AP]` (or `channel.cost`) of the channel meter at its start; adds `channel.effects`; costs no weapon durability (PK2).
- **What a unit has in a Channel round**: its `channel.only` skills (graft skills and an Inoculated keystone, PK2; in PK1 test data, granted by a test passive), the two Channel basics (Feed, Vent: listed in the `strain` row's `skills`, which open with no slot while their cost is met, UM6; `FB-167` Q8 retires a separate `channel_basics` field), the class role basic's marrow version (if single intent), and the marrow versions of the deck actives the player marked as marrow (deck field `marrow`, at most `marrow_actives` of them, single intent only).

### 4.3 The Channel round

1. **Channel opener, a toggle** (user 2026-11-10): a neutral command beside Move, Rest and Fill, **0 AP, 0 blocks**, only for a unit that can Channel. It names no target and sets a **`channel` flag on the round** (`FB-167` Q1 (a): not an intent value; cleared at each round start; the first skill's row stays the round's intent; entering a Channel round counts as an intent change, so intent-bound buffs end; tells show Channel while the flag is set); clicking it again returns the round to Cold Steel. **The toggle never locks; it converts the plan** (user 2026-11-10, replacing the earlier "declare first, then locked" rule): the round is a Cold Steel round or an Inoculated round, every round starts in Cold Steel, and the toggle can be flipped any time while planning. Flipping it keeps every planned **Move** where it is and swaps each planned skill to its other version (Cold Steel <-> marrow, same target and anchor); a skill with no counterpart in the new mode (a dual-intent heavy or a technique going into Channel, a graft skill or Channel basic going back to Cold Steel) is dropped, and the auto-fill refills the freed blocks for the new mode (Rest in Cold Steel, Feed in Channel). The mode lasts that round only.
2. After the opener, every intent row offers **only marrow versions** (section 4.2); Cold Steel versions are grey with the reason "Channel round".
3. The **first skill after the opener fixes the round's one target** (TG1); later picks must use that target, as in any round (the row each comes from still sets its anchor).
4. Follow-ups and Fill: marrow versions only, in the **Channel order**: `channel.only` skills, then marked actives, then the class basic; what cannot fit is Feed (the Channel round's Rest, item 5).
5. **The Channel basics: Feed and Vent** (user 2026-11-10; Surge and Hold dropped the same day: marrow versions are already empowered, and Hold could not hold a bracket, since any Channel round skips decay anyway): free (no slot), only in Channel rounds, only for Strain carriers; together they let a unit balance its Strain:
   - **Feed is the Channel round's Rest**: in a Channel round Rest is replaced by Feed, so it is the fill in both modes: a Cold Steel Rest gives AP, a Channel Feed gives a small heal at a Strain price. Per block: +5 Strain and a self-heal of 4% of max HP (the weaker one; first-pass defaults, kept small because it fills every empty block). The Acute Resonance band's healing -25% (from 25 Strain) applies to Feed too, so Feed gets weaker as Strain climbs while it keeps pushing Strain up: it is best at low Strain and something to avoid high up (an accepted trade-off, user 2026-11-10).
   - **Vent** (the stronger one, down): Stand self, 1 block, -20 Strain, costs 15% of max HP; **usable only while Strain is 50 or more when it starts** (an unlock condition, checked at the Vent's start; user 2026-11-10, replacing the once-per-round limit and an earlier floor at 50): a Vent from 55 goes to 35, and below 50 no further Vent is allowed, so it pulls a unit out of the danger brackets without letting it purge to the bottom. Several Vents in one round are fine while each starts at 50 or more. **Vent never knocks a unit out** (user 2026-11-10): its HP cost uses the existing HP-cost rule (HOOKS1 HC1, `command::hp_cost_problem`: refused and greyed when current HP is at or below the cost), so a unit without the HP to pay cannot Vent; the planner shows both reasons (Strain below 50, not enough HP). The HP cost may go higher after the balance run.
   Ruleset fields `channel_rest` (`feed` here: Rest becomes the named skill in a Channel round; kit-generic, another game may pick `normal`) and the `strain` row's `skills` (Feed and Vent; Vent carries a meter gate `strain >= 50` checked at its start: kit field `requires_meter` on a skill; the planner greys it with the reason when the planned Strain at that block is below 50; its HP cost needs a percent form of `hp_cost`, `hp_cost_pct` of max HP, through the same HC1 check). Numbers are first-pass defaults for the balance lane.
6. Tells: the plan strip, the unit card and enemy plan badges show the Channel intent from the opener on; the threat intel lists marrow versions as their own names (the skill name with a marrow mark, game term).
   **Visual mode indicator** (user 2026-11-10): an Inoculated round must be unmistakable both **on the unit** (an effect or a transformation look while the toggle is on) and **on the timeline** (marrow blocks look clearly different from Cold Steel blocks). PK1 ships a placeholder (a tint on the unit and the strip blocks); the final look is designed with the game UI layout (the Post-R4 UI polish track and the Artwork & Mock UI session), not here. **Placement** (user 2026-11-10): for PK1 the toggle stays a 0 AP, 0 block button in the neutral command list; the final UI will likely move it out of the command list into a dedicated mode control (declare Cold Steel or Inoculated there, and the command list follows), designed with the same UI layout work.
7. A round not opened with the opener is Cold Steel; no marrow version is offered.
8. Strain bookkeeping: each marrow action pays at its start; decay skips the round (`unless_gained`, and a Channel round always counts as gained, item 5).

### 4.4 AI

Strain-carrying units (aberrants, and AI-driven heroes) get one AI row: open with the opener when a marrow version scores at least `channel_ai_edge` (tunable, default 1.2) times the best Cold Steel option and the unit's Strain after the round would stay below 100; otherwise plan Cold Steel. Enemies pay no AP but pay Strain, so their crises are real.

## 5. Game data (asset lane, after the kit slices)

- `strain` row (section 2) with the crisis trigger.
- The two Channel basics (section 4.3 item 5): **Feed** and **Vent** (user 2026-11-10; they replace the Design default of a 1 AP Assault strike; Surge and Hold were dropped), each `channel.only`, with the default numbers there.
- A test passive `inoculated_test` granting the Strain meter (floor 0) and `channel.only` test skills, for the Dojo and sims until PK2's grafts exist.
- **Quenched Cinder Salt** (pouch item with `unmark`, rare: drop and price rows by the balance lane).
- One aberrant test enemy with Strain and a Strain-adding hit, and a Dojo preset "Channel duel".

## 6. Slices (proposed; Build's feasibility check may reorder)

| Slice | Scope | Acceptance |
| --- | --- | --- |
| `PK1-S1` | Section 3 hooks (`heal_taken`, `pct_of_hit`, `strike_nearest`, `ap set`, `mark` and `unmark`, hazard `meter`) | unit tests per hook; **0 golden diffs** (no data uses them) |
| `PK1-S2` | Section 4 kit: rule `channel` and fields, eligibility and marrow versions, the opener toggle, Channel Rest (`channel_rest`), planner rows and Fill order, deck `marrow` and the limit, tells, AI row | unit tests: the toggle works only for holders, at any point of planning, keeps Moves, swaps versions, drops skills without a counterpart and refills the freed blocks; a Channel round skips decay; Rest becomes Feed in a Channel round; Vent is refused when Strain is below 50 at its start and may take Strain below 50; Feed and Vent change Strain and HP by their amounts; rows offer only marrow versions after it; a dual-intent skill never gets one; cost by AP; the target rule; `rules_contract_golden` re-recorded (new ruleset fields), **0 battle and sim golden diffs** |
| `PK1-S3` | Section 5 data and the Dojo preset; balance run (Q-67: crisis rate per battle, Inoculated dominance in short fights, Flesh-Tension as a trap) | data checks clean; a `sim` report in `FEEDBACK.md`; goldens unchanged unless a fixture reads the new rows (named) |

### 6.1 Balance watch for `PK1-S3` (user 2026-11-10)

- **Is Vent worth it?** Measure how often a unit Vents to dodge a crisis (and its Calcification risk) only to end the battle barely alive or wounded; the HP cost is tuned against that trade, not on its own.
- **Is Vent too easy?** If venting makes Strain trivial, bring back a limit (once per round, or a rising HP cost per Vent in the battle).
- **Candidate variant** if a bigger brake is wanted: a two-block Vent that removes about 30 Strain (same 50 unlock, higher HP cost).
- **Feed under Acute Resonance**: whether the -25% healing makes Feed too weak (knobs: exempt Feed, or raise its base heal).

## 7. Fixed vs tunable (`aegis rules --contract`)

Tunable: `channel_power`, `channel_cost`, `marrow_actives`, `channel_ai_edge`, `channel_rest`, the Channel basics' numbers, the Strain row's bands, decay and crisis rows (data). Locked: the 0 AP / 0 block converting toggle, the single-intent eligibility, the all-marrow round.

## 8. Defaults Design chose (user may revisit; logged in `docs/DESIGN_REVIEW.md`)

- Recoil cannot down the unit (min 1 HP).
- Spasm targets the nearest unit of any side, ties to the lowest id.
- Biological Collapse's armour 0 is `expose` for 99 rounds.
- The Channel AI edge 1.2 and the "stay below 100" guard.
- ~~The marrow basic is an Assault skill at 1 AP~~: replaced by the user's pick: the Channel basics Feed and Vent (section 4.3 item 5; Surge and Hold dropped). User 2026-11-10: the other defaults above accepted.
