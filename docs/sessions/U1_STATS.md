# Session U1: stats and derived values

MASTERPLAN1 sub-block U1. Interactive design session with the user, 2026-11-06. Status: design closed (user 2026-11-06); build waits for the rest of Unit.

## The user's purpose for each derived value (2026-11-06)

1. **Max HP**: total health. Damage lowers it (min 0), heals raise it (max HP cap). At 0 the unit is knocked: a separate derived flag.
2. **Weapon power**: attack damage from the weapon; a formula by weapon, the scaling stat by weapon type; a fixed value from weapon and unit stat.
3. **Unarmed power**: damage with no weapon; rethink whether a separate value is needed.
4. **Speed**: today both initiative and move rate; consider splitting into move rate (how far a unit moves each move block) and initiative (which action takes precedence on a tick, the order of resolution).
5. **Armour**: mitigation with diminishing returns; survivability; helps classes specialise. Derives a mitigation value only, and conflicts with other mitigation sources.
6. **Crit chance**: adds crits to hits; variability and non-determinism; another damage amplifier.
7. **Resist**: counter to status effects; a chance on status application; reduces deterministic outcomes.
8. **Max poise**: the meter that holds command execution; adds or prevents interrupts to the command chain; needs simplifying so it is intuitive to play and track.
9. **Composure**: no behaviour today. See whether a morale system can use it, else remove; it should link to an anti-corruption mechanic.
10. **Ranged reach +1**: belongs to the skill, not the unit, as a unit can have melee and ranged abilities; rethink.
11. **Heal power**: strength of healing; belongs to the skill, not the unit; rethink.
12. **Unlock early**: unlocks skills sooner; more a passive than a derived value.
13. **Carry**: total weight allowed for the stash, a penalty for hoarding; adds little beyond inventory management.
14. **Item requirements**: minimum stats for a weapon's full power; a soft cap on when a weapon can be used; for proficient units the cost can drop, giving variance and preference in weapons.
15. **Dialogue check**: a threshold unlocks other options; flavour. Instead of a flat threshold, a roll plus bonus against it, to succeed or fail.

## The user's proposal

Treat every derived value as a meter: a value in a range (min, max), with behaviour on a threshold breach, on a change (+delta, -delta), or at a meter level.

## Decided (user 2026-11-06)

- **One derived class, two modes.** A **pool** holds a current value that moves in play, with behaviour on bands and on change (+/- delta): HP (knocked at 0 as a band effect), poise (5 pips as bands), AP. A **rating** is read when needed and changes only with its inputs (gear, buffs, level): initiative, move rate, armour, crit, resist, weapon power. Both: formula, clamp to (min, max), optional bands with effects. Reuses DERIVE1 rows and unit meters.
- **Speed splits** into move rate and initiative, both from DEX by default; a pack may separate them.
- **Unarmed folds in**: fists are an implicit weapon item, so no weapon is a weapon.
- **Moved out of U1**: ranged reach to the skill; heal value to the skill, with heal scaling as a base rule; unlock early to passives (U4); item requirements to gear (U3), proficiency lowering their cost; the dialogue check to story, as a roll plus bonus against a threshold; carry kept as an option, off by default.
- **Composure** parked as a named pool with no behaviour, a candidate for morale tied to anti-corruption, until its gameplay case.
- **The mitigation stack** is one rule covering armour, guard, cover and any others, settled in this session.
- **AP** counts as a U1 pool, the same shape as poise.
- **Baseline agreed** (user 2026-11-06): two mitigation layers, the stat table and the core list as the planner proposed them in the session, with the open questions there.

## Proposed by the user: a modular damage formula

Damage is a second-order effect. Its behaviour is a change to HP (delta HP), scaled by multipliers (mult x delta HP); the actual damage dealt comes from all the behaviours applied. Planner's shape, for the user's answer: fixed stages in the kit (base, amplifiers, mitigation layers, finish: rounding, minimum, clamp to the pool), each modifier a data entry naming its stage, its condition and its value source; the same pipeline for heals and for other pools (poise, AP); a trace of every modifier on every hit for forecasts and tests; the default pipeline reproduces today's formula exactly, rounding points included.

## Decided (user 2026-11-06, all recommendations)

- **Mitigation**: two named layers that multiply, a hit doing at least 1. Rating layer (who you are): gear, stat bonus, rating statuses such as fortify and expose, added into a rating, then the curve. Situational layer (what you are doing): Stand's guard status, cover, later stance-like sources, added, then capped; the cap is a parameter starting near 75 %, set by the limit test. Piercing skips the rating layer, ignore-guard the situational one. Every new mitigation source declares its layer.
- **Stats to values**: HP from CONS and STR; poise from CONS and gear; AP no stat; initiative and move rate from DEX, each with its own curve and cap; armour from gear and STR; **crit from PERC**; resist from INT; **status potency from KNOW**, against resist; weapon scaling by weapon family (STR, DEX or INT); heal scaling from KNOW as a base rule. All linear, no thresholds. CONS's weight is left to the limit test.
- **Core, never off**: HP, initiative, move rate, weapon power; poise, AP and heal scaling while those systems exist. Everything else has an off state.
- **Modular damage**: a pipeline of fixed stages in the kit, base, amplifiers, mitigation, finish (round, minimum 1, clamp to the pool); modifiers are data naming their stage, condition and value source; games cannot reorder stages; flat modifiers only in the base stage; the same pipeline for heals, poise and AP; every hit keeps a trace of its modifiers; the default pipeline reproduces today's formula exactly, both rounding points included.

## Design result, for the user's approval: what is fixed and what is a parameter

| Item | Fixed (kit behaviour) | Parameter (data), with today's value |
| --- | --- | --- |
| Derived class | Two modes, pool and rating; formula, clamp, bands, delta effects; the frozen formula grammar | Each value's formula weights, min, max and bands |
| Pools | HP, poise and AP exist as pools; knocked at HP 0 | Max HP weights (60 + CONS x 8 + STR x 2); poise weights (CONS x 2 + gear); AP pool 12, start 10, regen 4; poise bands (5 pips) |
| Ratings | Initiative, move rate, armour, crit, resist, potency, weapon power | Initiative weight (DEX); move rate curve and cap (today speed-based, cap 4); armour stat bonus (STR / 4); crit base and weight (5 % + PERC term), crit multiplier (1.5); resist weight (INT x 2.5); potency weight (KNOW); weapon scaling per family |
| Mitigation | Two layers, their order, minimum 1, skip flags | Armour curve constant (50); situational cap (near 75 %, today 0.9) |
| Damage pipeline | Four stages and their order; rounding points; flat only in base | Each modifier's value and condition |
| Stats | Behaviours read roles, never stat names | Stat ids, names and how many (6 today); which stat fills each role |

Off states, defined: crit off = no crits; resist off = statuses land at their potency; potency off = the skill's own potency; armour stat bonus off = gear only. Parked: composure (morale, anti-corruption), carry (an option, off by default).

Moved to other sub-blocks: ranged reach and heal value (skills, A1); item requirements (gear, U3); unlock early (passives, U4); dialogue rolls (story, T1).

## Adding a derived value without a logic change (user 2026-11-06)

A new derived value is a data row plus picks from existing behaviours: mode, formula in the frozen grammar, min and max, start; what moves it (triggers on existing events with a pool-change effect, pipeline effects aimed at that pool); what it does (bands granting existing statuses, which the AI already reads); modifiers that read it as a value source. The UI shows any pool by name, the trace lists its changes, and the level-3 tests already cover any pool's bands. Code is needed only for a new effect kind or a consumer that is neither a modifier nor a band, each one build task with a gameplay case.

**Composure is the U1 build's acceptance test**, as data only and off in this game: pool, `KNOW x 2 + CONS`, 0 to that, full each fight; -10 when an ally within 2 is knocked, +5 on rest; below half grants `weaken`, at 0 a shaken status. The user will revisit its behaviour when evaluating its gameplay, unlike the other values, which have a clear purpose now. The user approved the framework and the parameter list (2026-11-06): U1's design stage is closed. It is built with the other Unit sub-blocks once their design closes, as major builds hold until then.

## Feasibility resolutions (`FB-04` to `FB-09`, accepted 2026-11-07)

1. **Typed Pool facade (`FB-04`)**: `Unit.hp`, `Unit.ap`, and `poise` keep their typed struct fields so AI and planner callers do not churn; the `U1` Pool layer acts as a facade over them, while newly added data pools (`composure`) use the generic pool store.
2. **Pipeline stages, rounding, and Slice 1 exact port (`FB-05`)**: Allow `tier_power` as a multiplier in the base stage (flat adds remain base-only); keep fixed-stat and authored `power` overrides; leave flat hazard/push damage (`take_damage`) and ally intercepts on their paths until `A2.2`. **Slice 1 of `U1.2` is an exact port with all goldens byte-identical** before enabling any formula or cap changes.
3. **`Unit.guard` accumulator (`FB-06`)**: Keep `Unit.guard` as the numeric accumulator for Situational Mitigation (Layer 2), with guard statuses and skills feeding into it. Moving the situational cap from `0.90` to `0.75` lands as a separate, named balance commit.
4. **Speed split and attributable golden commits (`FB-07`)**: Burden and carry overload lower `Initiative` first (lowering `Move Rate` only if Initiative hits its floor of 1). Legacy fixed-stat `speed` maps to `initiative`. Land `PERC`->crit, `KNOW`->potency, and the Speed split as separate commits so each golden re-record is attributable with its own `HISTORY.md` row.
5. **Composure acceptance test scope (`FB-08`)**: Drop the unbuilt `corruption` pool reference from the `U1.2` proof test. Composure's max is the formula `KNOW x 2 + CONS` (clamped to `0..100`). `U1.2` includes formula-based pool max, percent-of-max band thresholds, the `within` distance filter on `ally_downed`, and widening/parameterizing the `-10..10` pool delta clamp.
6. **Numeric tier input (`FB-09`)**: Expose a data table of per-tier factors read by derived formulas as `tier_hp` and `tier_power`.
