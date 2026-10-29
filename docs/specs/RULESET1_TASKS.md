# RULESET1 tasks: schema, assets, tests, parity and the task split

Status: proposed, revised 2026-10-25 (intent rename); companion of [RULESET1.md](RULESET1.md) and
[RULESET1_INTENT.md](RULESET1_INTENT.md), whose rule ids it cites

## 1. Schema contract (schema-first: yes; R and S)

New rows in `kits/rpg/schema.json` join `AFTER_GODOT` (`emberwake_world/tests/data_source.rs:25`); defaults keep
today's behaviour.

### 1.1 World manifest (`world.world`, RULESET1-R)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `ruleset` | enum `""`, `classic`, `timeline` | `""` | `""` = the nearest parent's, else classic; `timeline` needs the `rulesets.timeline` row |
| `plan_reveal_flag` | string | `""` | a flag id (`[a-z][a-z0-9_]*`) or empty; read by V3 |

### 1.2 Ruleset rows (`content.rulesets`, new kind, `data/catalog/rulesets.json`, RULESET1-R)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | enum (key) | `timeline` | code-owned list: `timeline` (classic has no row) |
| `name` | string | "Timeline" | 1-24 characters |
| `rules` | strings | [] | subset of the register ids (index 3.2), no repeats |
| `intents` | enums | [`assault`, `guard`, `support`, `maneuver`] | subset of those four and `channel`, no repeats; at least one; needs `round_intent` in `rules` |
| `basics` | refs skills | [] | skills whose intent (IN4 default or `round_intent`) is neutral or in `intents`; required with `deck`; no repeats |
| `poise_stat` | string | "" | a `derived_stats` id; required when `rules` has `poise` |
| `stagger_windup_mult`, `_guard_mult`, `_repeat_mult` | floats | 2.0, 0.5, 0.5 | 1.0-4.0, 0.0-1.0, 0.0-1.0 |
| `flank_stagger_mult`, `flank_power_mult` | floats | 1.5, 1.0 | 1.0-3.0, 1.0-2.0 (TQ12 a: power stays 1.0, a balance knob) |
| `break_chance_scale`, `break_chance_cap` | float, int | 0.0, 50 | 0.0-1.0 (0 = off), 0-100 |
| `rest_ap_per_block`, `rest_ap_cap`, `rest_poise_per_block` | float, int, int | 0.5, 2, 5 | 0.0-2.0, 0-10, 0-50 (RS2, RS3) |
| `deck_slots` | json | {} | object with `active` 1-8, `passive` 0-4, `keystone` 0-2; required with `deck` |
| `origin` | enum | system | as every catalog kind |

### 1.3 Skills (`content.skills`, RULESET1-S)

| Key | Type | Default | Scope | Validation |
| --- | --- | --- | --- | --- |
| `round_intent` | enum `""`, `assault`, `guard`, `support`, `maneuver`, `channel`, `neutral` | "" | family | "" = by kind; a set value must be allowed for the kind (IN4 table); `channel` only when a ruleset row lists it |
| `intent_buff` | bool | false | family | DEFEND, SUPPORT, FOCUS; needs `grant_status` or `extra_grants` (IN6) |
| `ap_cost` | int | -1 | rank | -1 or 0-10; kinds ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE |
| `recovery_blocks` | int | 0 | rank | 0-2; kinds ATTACK, SUPPORT, FOCUS; `blocks + recovery_blocks <= 6`; read for enemy-side units only (T4) |
| `stagger` | int | 0 | rank | 0-100; ATTACK only |
| `windup_poise` | int | 0 | rank | 0-100; ATTACK, SUPPORT, FOCUS |
| `unbreakable` | bool | false | family | ATTACK, SUPPORT, FOCUS; warning when `windup_poise > 0` too |
| `sure_break` | bool | false | family | ATTACK; needs `stagger >= 1` on every rank |
| `armor_piercing` | bool | false | family | ATTACK |

`exploit` accepts `{"status": "flanked", "power_mult": x}` (MV3; timeline only, classic never matches it). Text
tokens `{stagger}`, `{recovery}`, `{ap}` (counted like `blocks`, `check/skill.rs:343`).
Carried moves reuse `dash_tiles` and `jump_tiles` (help text unchanged: parity); a Maneuver ATTACK with `dash_tiles`
0 warns (MV5).

### 1.4 Passives (`content.passives`, RULESET1-S)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `slot` | enum `""`, `passive`, `keystone` | `""` | `""` = source-granted, always on |
| effect `stagger_taken` | `{"type", "mult", "max_blocks"}` | - | `mult` 0.0-2.0; `max_blocks` 0-6 (0 = any action) |
| effect `foresight` | adds optional `range` | 0 | 0-8 (0 = today's rule) |

### 1.5 Units (`content.units`, RULESET1-S)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `deck` | json | {} | keys `active`, `passive`, `keystone`, `order`; `active`: owned families (path or `extra_skills`), no basic; `passive`/`keystone`: passives of that `slot`; counts within `deck_slots`; `order`: intent -> families from `active` or `basics` of that intent; no repeats anywhere |
| `poise` | int | -1 | -1 (derived), 0 (no bar), 1-200 (fixed Max Poise) |

### 1.6 Item families and items (RULESET1-S)

- `item_families.engine_rules` gains `reach`: WEAPON only, not with `always_ranged`, every hand's base range 1.
- `items.weapon_type` options gain `polearm` and `scourge` (TQ8).

### 1.7 Status table and formula input (code)

- `status.rs` rows 16 `steadfast` (buff, not resistable, 1 round, RefreshMaxDuration, cap 0, grant "support") and 17
  `stealth` (same, 2 rounds, display "Stealthed", grant "defend"); column `intent_bound` (true for `riposte` and
  `stealth` only; classic never reads it); `StatusState` gains the intent tag of a bound instance (IN6).
- DERIVE1-R gains the reserved formula input `gear_mitigation` (`Inventory::total_flat_mitigation`, `gear.rs:298`),
  rejected as a row id or constant name like `level`.

### 1.8 Runtime and save

- `EncounterDef`: `ruleset: Ruleset` (default classic), `plan_reveal: bool` (default false); not read from rows.
- `FightSpec`: `ruleset`, `rules`; sim args `ruleset=classic|timeline`, `rules=-id,+id` outside the Godot-parity help
  text (as `rewards`, `args.rs:53-55`); host `battle` command field `ruleset`.
- Battle unit state (timeline only): `round_intent: Option<RoundIntent>` (None at start), `facing`, `moved_this_round`.
- Plan: per unit `IntentChoice { intent, target (self, unit, cell), maneuver (engage, disengage, utility, none),
  opener (skill, item, none), manual }`; carried-move tiles in `Step.cell` (MV6); host requests `round_intent {unit,
  intent, target, maneuver, opener}`, `manual {unit, on}` (replays record them); the `intents` reply gains
  `round_intents` (V5, UI5).
- Party member `deck` (1.5's shape): save step N+1 adds `{}` (or it joins M0's new save format if that lands first).

## 2. Assets (RULESET1-A, asset lane)

Through `tools/fc.sh content upsert <kind> @<file>`, then `content check`; new rows appended, no Ashlands row changes
(their intents are kind defaults). Order: derived_stats, class, family, item, skills, passives, rulesets, units,
encounters, world field.

### 2.1 Ruleset row `timeline` (`name` "Timeline")

| Key | Value |
| --- | --- |
| `rules` | `round_intent`, `enemy_moves`, `deck`, `poise`, `ap_cost`, `pierce`, `reach`, `flank`, `plan_badges` |
| `intents` | `assault`, `guard`, `support`, `maneuver` |
| `basics` | `move`, `rest`, `strike`, `offhand_jab`, `active_guard`, `flank_cut` (TQ15 a; row in 2.4) |
| others | `poise_stat` `max_poise`; `deck_slots` `{"active": 3, "passive": 2, "keystone": 1}`; numbers at their 1.2 defaults except `break_chance_scale` 0.5 (TQ2) |

### 2.2 Derived row `max_poise` (DERIVE1 shape)

`name` "Max Poise", `label` "Poise", `description` "Refills each round. Stagger lowers it; at 0 the unit breaks and
loses its action and the rest of its round.", `formula` `CONS * per_cons + gear_mitigation * armour_factor`,
`constants` `{"per_cons": 2, "armour_factor": 1.0}` (TQ10), `min` 1, `max` 200, `rulesets` ["timeline"], `show_on`
["party", "dojo", "battle"], `order` 1.

### 2.3 Test class, family and item

| Kind | Id | Values |
| --- | --- | --- |
| class | `tl_drill` | name "Drill (test)"; base stats STR 13, DEX 10, INT 6, CONS 13, PERC 10, KNOW 10 (62); growth STR 1, CONS 1, DEX 1; proficiency sword, mace, dagger, polearm, shield, plate, leather at level 2, scaling 1.5; skill path `{"tl_thrust": 1}`; ai_role AGGRESSOR |
| family | `polearm` | WEAPON; hands `main_2h`; engine_rules `reach`; scaling_stats STR; damage_type pierce; base main_2h power 26, range 1, block_cost 1, burden 2, mitigation 0; bands as `sword` main_2h |
| item | `tl_billhook` | "Notched Billhook (test)"; family polearm; hand main_2h; scaling STR; rarity COMMON; weight 4.0; value 40; no modifiers |

### 2.4 Test skills (one rank each, tier 1, level 1, `learned_only` true except `tl_thrust` and the two basics)

Rec.: enemies only (T4). Engage: Hamstring Cut, the basic Flank Cut; Disengage: Slip Strike; Utility: the pouch
item `con_fire_bomb` (thrown, reach 3, damage 45).

| Id | Name | Kind | Intent | Blocks | Rec. | AP | Power | Stagger | Flags |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `active_guard` | Active Guard | DEFEND | guard | 2 | 0 | 1 | - | 0 | mitigation 0.5; `basic` false |
| `flank_cut` | Flank Cut | ATTACK | maneuver | 1 | 1 | 2 | 0.6 | 8 | `dash_tiles` 2; `basic` false |
| `tl_brace_counter` | Brace and Counter | DEFEND | guard | 2 | 0 | 2 | - | 0 | mitigation 0.3; grant `riposte` 1 round |
| `tl_fade` | Fade | DEFEND | guard | 1 | 0 | 1 | - | 0 | mitigation 0.2; grant `stealth` 2 rounds |
| `tl_field_dressing` | Field Dressing | SUPPORT | support | 2 | 0 | 2 | - | 0 | heal_base 8, heal_mult 0.3; ALLY_OR_SELF |
| `tl_hamstring_cut` | Hamstring Cut | ATTACK | maneuver | 1 | 1 | 2 | 0.8 | 10 | `dash_tiles` 3; exploit `flanked` x1.3 |
| `tl_slip_strike` | Slip Strike | ATTACK | maneuver | 2 | 1 | 2 | 0.9 | 8 | `dash_tiles` 2; status `slow` potency 60, 1 round |
| `tl_snap_jab` | Snap Jab | ATTACK | assault | 1 | 1 | 2 | 0.7 | 12 | - |
| `tl_thrust` | Thrust | ATTACK | assault | 2 | 1 | 2 | 1.0 | 8 | - |
| `tl_heavy_cleave` | Heavy Cleave | ATTACK | assault | 4 | 1 | 4 | 2.0 | 18 | - |
| `tl_shoulder_check` | Shoulder Check | ATTACK | assault | 1 | 0 | 1 | 0.5 | 10 | - |
| `tl_jaw_breaker` | Jaw-Breaker Drill | ATTACK | assault | 1 | 1 | 3 | 0.8 | 15 | sure_break |
| `tl_marrow_spike` | Marrow Spike | ATTACK | assault | 1 | 0 | 1 | 0.9 | 0 | armor_piercing |
| `tl_hook_thrust` | Hook Thrust | ATTACK | assault | 2 | 1 | 2 | 1.1 | 10 | - |
| `tl_snap_shot` | Snap Shot | ATTACK | assault | 1 | 1 | 2 | 0.6 | 6 | - |
| `tl_aimed_shot` | Aimed Shot | ATTACK | assault | 2 | 1 | 2 | 1.1 | 6 | - |
| `tl_overhead_smash` | Overhead Smash | ATTACK | assault | 3 | 1 | 3 | 1.8 | 15 | windup_poise 20 |
| `tl_crushing_judgment` | Crushing Judgment | ATTACK | assault | 4 | 1 | 4 | 2.2 | 20 | unbreakable |

Anchors (lore): Snap Jab = Pommel Strike; Heavy Cleave = Sunder and Pin; Jaw-Breaker Drill = Jaw-Breaker +1 AP;
Marrow Spike = Calcified Spike (no Strain); Crushing Judgment = the Bailiff Enforcer's. Descriptions: one sentence
naming intent, tier and role ("Assault, Snap: a quick jab that staggers."). Stagger = lore x 0.5 (index 5.6).
`round_intent` is written on every row (the three Maneuver attacks differ from the kind default).

### 2.5 Test passives

| Id | Name | `slot` | Effects |
| --- | --- | --- | --- |
| `tl_braced_poise` | Braced Poise | passive | `[{"type": "stat", "stat": "max_poise", "value": 20}]` |
| `tl_anvil_posture` | Anvil Posture | keystone | `[{"type": "stagger_taken", "mult": 0.0, "max_blocks": 1}]` |
| `tl_hawk_eye` | Hawk Eye | passive | `[{"type": "foresight", "range": 4}]` |

### 2.6 Test units (level 1, `stat_mode` derived, `spend` {}, `extra_skills` = the deck's actives not on the path)

Deck ids carry the `tl_` prefix (shortened here). Enemy `order`: assault = their actives as listed.

| Id | Side, tier | Class | Gear; pouch | `deck`: active; passive; order | `armor` | `poise` |
| --- | --- | --- | --- | --- | --- | --- |
| `tl_vanguard` | party | tl_drill | wpn_broadsword, shd_kite, arm_plate; con_heal_potion | heavy_cleave, shoulder_check, brace_counter; braced_poise; assault [heavy_cleave, shoulder_check, strike], guard [brace_counter, active_guard] | -1 | -1 |
| `tl_halberdier` | party | tl_drill | tl_billhook, arm_plate | hook_thrust, jaw_breaker, fade; -; assault [jaw_breaker, hook_thrust], guard [fade, active_guard] | -1 | -1 |
| `tl_spiker` | party | tl_drill | wpn_dagger, arm_leather; con_fire_bomb | marrow_spike, hamstring_cut, slip_strike; -; assault [marrow_spike, strike], maneuver [hamstring_cut, slip_strike, flank_cut] | -1 | -1 |
| `tl_archer` | party | ranger | wpn_shortbow, arm_leather; con_heal_potion | aimed_shot, snap_shot, field_dressing; hawk_eye; assault [aimed_shot, snap_shot], support [field_dressing] | -1 | -1 |
| `tl_footman` | enemy, standard | tl_drill | wpn_shortsword, arm_leather | snap_jab, thrust | 25 | -1 |
| `tl_enforcer` | enemy, elite | tl_drill | wpn_flanged_mace, arm_plate | overhead_smash, thrust | 35 | -1 |
| `tl_captain` | enemy, boss | tl_drill | wpn_claymore, arm_plate | crushing_judgment, thrust; keystone anvil_posture | 50 | 60 |
| `tl_crossbowman` | enemy, standard | ranger | wpn_shortbow, arm_leather | aimed_shot, snap_shot | -1 | -1 |

Dojo poise check: footman about 26 + 4 = 30, vanguard about 26 + 8 + 6 + 20 = 60.

### 2.7 Test encounters (catalog) and the world field

Shared: zone `stone_arena`, CLOSED, exits [[11, 4]], max_rounds 8, total_blocks 6, ap_regen 4, unit_ap 10, rng_seed
1337, objective rout, deployment standard, morale false, pace auto.

| Id | Party | Enemies | Judges |
| --- | --- | --- | --- |
| `tl_duel_windup` | tl_vanguard | tl_enforcer | a Snap timed into a Heavy (manual); follow-ups; Guard and riposte; Rest gain |
| `tl_duel_reach` | tl_halberdier | tl_footman | reach 2 against reach 1; the sure break; Fade and a locked attack lost |
| `tl_duel_armour` | tl_spiker | tl_captain | pierce against armour 50; an unbreakable windup; enemy recovery |
| `tl_flank` | tl_vanguard, tl_spiker | tl_footman, tl_crossbowman | Engage with a carried move, Disengage retreat, Utility fire bomb, kiting |
| `tl_skirmish` | the four party units | tl_footman twice, tl_enforcer, tl_crossbowman | the 4-on-4 loop, the AI table |

A roster that may not repeat an id gets `tl_footman_b`. `sunk_ward`: `ruleset` `timeline` once M0's skeleton exists.

## 3. The sim's feel mode (RULESET1-G)

`aegis sim feel [enc=a,b] [seeds=N] [rules=...]`: default the five `tl_` encounters, 10 seeds, timeline. Reports won %,
median rounds, breaks, windup breaks / hits, chance rolls and successes, sure breaks, guard blocks, pierce share, AP
per round, lock rate (broken two rounds running), intent share and changes, buffs dropped, follow-ups, Rest blocks
and AP, flanks, carried moves, kite steps, lost attacks, planner reasons, rounds whose intent had no active (OI1).

## 4. Tests (rule-derived)

### 4.1 Per task

| Task | Test | Expectation from |
| --- | --- | --- |
| R | scratch worlds: A `timeline`, B extends A with `""`, C unset; the three shipped worlds | timeline, timeline, classic, classic (3.1.1) |
| R | a timeline world without the `rulesets` row; each 1.2 violation | one content error each |
| S | each 1.3-1.6 violation (a DEFEND set `assault`, `channel` unlisted, an `order` entry of another intent); a Maneuver ATTACK with `dash_tiles` 0; `legacy/data` | one error each, field named; one warning; no new error |
| T | an enemy skill with blocks b, recovery r at start s; the same on a hero | enemy s..s+b+r-1, lands s+b-1; hero s..s+b-1 (T1, T4) |
| T | `ap_cost` -1 and set; an enemy round under `enemy_moves` | old formula, else `ap_cost + modifier` in 0..10 (T10); AP unchanged, no debt status (T11) |
| IN | an Attack under timeline; another intent's skill, another target, a plain move, Rest | no stance change, switch AP 0, matchup 1.0 (IN3); refused, refused, accepted, accepted (IN5, TG1, MM1) |
| IN | an intent change holding riposte, stealth, an `intent_buff` grant, haste (3 rounds), a debuff; no change | first three removed, the rest keep their rounds; none removed (IN2, IN6) |
| IN | three landed Rest blocks at 0.5 / 2 / 5; classic Rest | 1 AP at round end, 15 poise; classic 2 AP on landing (RS2-RS4) |
| IN | the target downed before the third command | it and later ones are Rest, unpaid (TG4) |
| IN | T engaged with E to its east; A west, north, north-east; T's target is A | flank, flank, no, no (MV2, MV3) |
| IN | Engage with Hamstring Cut (`dash_tiles` 3), foe 3 tiles away; the same foe under Assault | 3 tiles inside its block at its AP, plain move still free; an approach block and AP (MV4, MV5, AS1) |
| IN | Engage face-on in band; stored tile taken at run time; no flank tile left | moves to a flank tile; PL4 re-pick; move skipped, hit only in band (MV6) |
| IN | Disengage, Slip Strike last; an opener out of range | retreat after its hit within 2, in its blocks (MV8); per TQ22 (MV9) |
| IN | stealthed unit moved in b1, a locked attack lands in b3; an area on it; unit did not move | nothing dealt, AP spent; hit; hit (GD5) |
| PL | the four worked examples (intent 4) | their plans, AP left 3, 2, 0, 0 |
| PL | the same inputs twice | identical plans and reason tags (PL1, PL6) |
| PL | Engage with no flank tile in range; Disengage out of band | not offered, reason shown (MV7, MV8, PL7) |
| PL | a ranged unit, target 4 tiles away and threatening | last block reserved, the step after the attacks (AS3) |
| AI | each AI table row in a scratch fight | that intent and target (AI1) |
| P | random stagger s, every windup / guard / repeat / flank / passive combination | `roundi(s x product)` (P6) |
| P | a break in block k: a later same-block action, Rest blocks, a running guard, an unstarted action | all cancelled, guard 0 at once, unstarted AP unpaid (P7) |
| P | two breaking hits in a round; a hit the next round; `windup_poise`, `unbreakable`, `steadfast` | one break; stagger x repeat (P8, P10); poise rises and falls back, never below 1 (P11, P12) |
| P | chance scale 0 vs 0.5 on a seed | 0 draws vs one per qualifying hit; same seed, same result (P13) |
| P | `sure_break` into a windup, into an unbreakable windup | breaks; does not (P14) |
| P | `armor_piercing` hit | `compute` with rating 0; guard and cover still cut (A2) |
| K | a polearm wielder | band (1, 2); approach stops at 2; no riposte at 2; cover 0; `attack_range` 1 |
| V | a timeline round | tells with intent and target; `round_intents` has both sides; badges at live non-move impacts; `plan_reveal` shows all; foresight `range` 4 |
| D | units with and without `deck` and `order` | auto deck and FU4 order; slotted passives only; classic ignores `slot`; `basics` usable, `guard`, `switch_target` not |
| D | an old save | loads with `deck` {} (migration) |
| E | widget with marks and segment parts | drawn inside their blocks; specimen page; clicks unchanged |
| U, DU | battle and party screens at 1280x720, four text sizes | no text-audit fault; threat intel shows hero and foe intents; classic unchanged |
| G | sim help text; `feel` twice on a seed; the Dojo planner table | unchanged; identical; one plan per intent |

### 4.2 Classic unchanged and golden traces (RULESET1-X)

- Every parity suite passes untouched (section 5); each Ashlands encounter traced with classic and with every
  extension of 3.2 on (options off) gives identical traces, logs, metrics and RNG draw counts; classic stances, Rest,
  riposte removal and the dash and jump as the round's move stay byte-identical (`round_intent` off).
- `kits/rpg/crates/rpgkit_battle/tests/timeline_golden.rs` runs the five `tl_` encounters for seeds 1337, 7 and 42
  through `sim::fight` on a frozen kit copy (`tests/fixtures/timeline_kit.json`) against
  `tests/fixtures/timeline_golden.json`, recorded once when X lands; re-recorded only for an intended rule change
  named in the commit and a HISTORY row.

RULESET1-X as built (2026-10-29): both checks live in the game's façade crate,
`games/emberwake/crates/emberwake_battle/tests/timeline_golden.rs` with `tests/fixtures/{timeline_kit,timeline_golden}.json`
(the `tl_` fights are game data and the kit names no game). The frozen catalog is the loader's catalog output; each
fight keeps the encounter's party, enemies, round limit and board keys (as `feel`), preset `balanced`, no jitter, the
timeline row; a fight record is `sim::fight`'s trace plus outcome, rounds and plan failure; a failure prints the first
difference (round, step or the path inside log, metrics or report). Re-record: `RECORD=1 tools/fc.sh test --test
timeline_golden` (keeps the frozen catalog; delete it to re-freeze). The inert check plays the Ashlands' own encounters
for seeds 1 and 2 (preset `balanced`) under classic and under classic with every register extension on (HOOKS1's too)
and compares every step, the RNG draws after each step, events, log, metrics and report, with two allowances:
`plan_badges` (no data needed) appends columns to each tell, cut before the comparison, and an extension's own readout
(the `poise` metrics) may be added if it counts nothing; a run under the timeline row must differ (the check is live).

## 5. Parity plan

- No effect: `trace_parity`, `world_traces`, `journey` (classic, no new keys); `checks_parity`, `sim_parity`,
  phasefold golden (no Godot row names a new field; args outside the help).
- `data_source`: `AFTER_GODOT` rows; a `rulesets` kind strip as DERIVE1-S; the `weapon_type` and `engine_rules`
  option rows skip `options` like `HELP_CHANGED` (`:38`); HISTORY row.
- `saves`: members gain `deck`, stripped as v12's `reward_paid` (`saves.rs:40-43`); HISTORY row.
- `timeline_golden` (new): the engine-only reference (4.2).

## 6. Tasks

Paths: `b/` `r/` `w/` = `kits/rpg/crates/rpgkit_{battle,rules,world}/src/` (`rpgkit_*` there), `gui/` =
`emberwake_gui/src/`; `schema.json` in `kits/rpg/`; `data_source.rs`, `saves.rs` in `emberwake_world/tests/`.

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| RULESET1-R | Ruleset switch: world fields, `rulesets` kind, `Ruleset`, `EncounterDef.ruleset`, bridge, sim and host args; classic stances inert under `round_intent` (3, 1.1, 1.2, IN3) | kit | logic | - | `schema.json`; `rpgkit_data/src/lib.rs`; `rpgkit_content/src/{source,store,check/*}`; `r/ruleset.rs` (new); `b/{session,plan,skill,command,sim}.rs`; `w/{data,battle}.rs`; `gui/battle.rs`; `rpgkit_sim/src/args.rs`; `emberwake_host/src/cli.rs` (args); `data_source.rs` |
| RULESET1-S | Timeline fields and checks (1.3-1.7): skills, `exploit` `flanked`, passives, units, families, `steadfast`, `stealth`, `intent_bound` | kit | logic | R | `schema.json`; `rpgkit_data/src/lib.rs`; `rpgkit_content/src/check/{skill,simple,unit,item}.rs`; `r/status.rs`; `data_source.rs` |
| RULESET1-T | Enemy recovery and no enemy AP (`enemy_moves`), `ap_cost`, `ScheduledAction.impact` (4.2, 4.4) | kit | logic | S | `b/{resolver,plan,skill,intent,log}.rs`; tests |
| RULESET1-IN | Round intent rules (intent 2-6): persistence, intent-bound buffs, one target, the `add_step` filter, neutral moves, Rest, carried moves, after-moves, facing, flank, stealth, cover | kit | logic | T | `b/{round_intent (new),plan,resolver,command,skill,world}.rs`; `r/{unit,status,damage}.rs`; tests |
| RULESET1-PL | Auto planner: opener, tiles (carried-move tiles too), follow-up fill, after-moves, reason tags, manual fill, `deck.order` (intent 4, 7, 8) | kit | logic | IN | `b/{round_intent_plan (new),plan}.rs`; tests |
| RULESET1-P | Poise, stagger with the flank factor, breaks, chance, sure break, guard halving, pierce, metrics (5, 6, 7) | kit | logic | IN, DERIVE1-R | `r/{unit,damage,derived}.rs`; `b/{poise (new),resolver,command,session,metrics,log}.rs`; tests |
| RULESET1-K | Reach-2 melee (8) | kit | logic | S (after P: shared files) | `r/{unit,gear}.rs`; `b/{skill,command,plan,ai}.rs`; tests |
| RULESET1-V | Badges, the round intent in tells (`round_intents`, host reply), `plan_reveal`, foresight range, trace keys (10) | kit | logic | T | `b/{intent,session}.rs`; `r/unit.rs`; `w/battle.rs`; `rpgkit_host/src/lib.rs`; tests |
| RULESET1-AI | Enemy and party AI: the intent pick table, enemy order lists, AUTO1 mapping (intent 9) | kit | logic | PL, V | `b/{ai,world}.rs`; `gui/battle.rs` (tactics); tests |
| RULESET1-D | Deck rules: slot kinds, `basics`, order lists, slotted passives, member `deck`, save step (9 D1-D5) | kit + game | logic | S | `r/unit.rs`; `b/{skill,world}.rs`; `w/{state,save,progression}.rs`; `saves.rs`; tests |
| RULESET1-E | Timeline widget: marks (glyph, colour, tip at a block) and segment parts (windup, impact, recovery; hatching; links) | engine | logic | - | `engine/crates/aegis_ui/src/{timeline,specimens}.rs`; `aegis_ui/tests/kit_widgets.rs`; `engine/README.md` |
| RULESET1-U | Battle screen (intent 10, index 12): intent buttons, target, kinds, opener, Manual, reasons, after-moves, flank arc, stealth veil, intent in tells and threat intel, badges, BREAK, poise, forecasts, reach band | game | logic | E, P, K, V, PL | `gui/{battle,intent_text,hud}.rs` |
| RULESET1-DU | Deck buckets, Basics row and order editor (party screen, Dojo; 9 D6) | game | logic | D | `gui/{party,dojo}.rs` |

RULESET1-DU slice 1 as built (2026-10-29): the party screen's deck view; agent-link widget ids `skills.{active,passive,keystone}.<i>`, `skills.basic.<family>`, `skills.view.{owned,order}`, `skills.slot.<kind>.<id>`, `skills.owned.<kind>.<id>`, `skills.order.<intent>[.<k>]`, `skills.order.auto`; drag keys `deck.<kind>.<i>`, `deck.own.<kind>.<id>`, `deck.list`, `order.<intent>.<k>`; state `deck`, `deck_shown`, `deck_auto`, `deck_view`, `deck_order`, `basics`. The Dojo deck view follows RULESET1-G.

RULESET1-U slice 1 as built (2026-10-29): the round controls (intent UI1) sit in the selected-hero panel, not on a strip row per member (the 640 x 360 screen has one strip row, the selected hero's); the intent buttons two a row, keys 1-9 (one per intent in the ruleset's order); a not offered intent is disabled in auto mode (Manual may still pick it); the warning of an intent change is a red corner on the button and the tip line "Changing loses Riposte." (kit `round_intent::would_drop`); the Maneuver kind and the opener are menus (`hero.kind`, `hero.opener`: Auto, the order of the intent, the pouch items of the intent); Manual and Fill buttons. Auto mode: a board click on a unit or tile that fits the intent sets the round target (another ally is selected; anything else says why); Manual: the step menu, led by a "Round target" row for a new target. The strip's block tips add "Why: <reason tag>" (PL6) and the Rest gain on Rest blocks (kit `round_intent_plan::rest_preview`), Rest blocks are dim; the hero panel lists the plan without its Rest blocks and a "Rest: n AP, m poise" line. The ruleset chip of an Arena fight beyond classic is `bar.chip` at the bar's right end (`runner::chip_text`, the Scene Runner card's text). Agent-link state: `intents` {unit, offers [{intent, offered, reason}]} under the round intent, `ruleset.chip` in Arena fights. Slices left: (2) the foresight range measured at scheduling and dialogue `best_stat` through `build_in`; (3) board overlays (target ring, valid cells, move paths and after-move glyphs, flank arc, enemy facings, stealth veil, reach band); (4) tells and threat intel with intents, impact badges, full reveal; (5) poise bars, BREAK markers, wind-up, impact and recovery on the strip, forecasts, round summary; (6) the text audit at four sizes and polish.

RULESET1-U slice 2 as built (2026-10-29): V4's range is measured when the round's enemy plan is scheduled (`Session::measure_reveals` after every enemy scheduling: the round start and `replan_enemies`; kit `intent::range_reveals`); the tells and threat intel read the measured readers for the whole round, so units moving or a holder falling during the playback neither hides nor adds a range reveal (today's targeting rule, `readers_of`, stays live as before). Dialogue `best_stat` builds the active members with `progression::build_in` under the deck rule of `gs.world_id` (classic: `build` exactly). Session callers audited: every party built from saved members goes through `build_in` (the battle screen's world fight); the host, sim, CLI and Dojo build catalog or roster units, which `Session::start` builds with `build_unit_in` under the encounter's deck rule. An enemy that arrives during playback (a reinforcement) gets no range reveal until the next scheduling, and a holder who falls during playback stays a reader for that round (review, 2026-10-29).

RULESET1-U slice 3 as built (2026-10-29): the board overlays (intent UI3, K6) draw on the ground under the tokens, Flat after the tiles and Stage in the markers pass, only under the rule `round_intent` (classic keeps today's path line). While planning, for the selected hero: the reach band of its plain attack (kit `skill::attack_band`) around the tile its first attack strikes from (else where it stands), under Assault and Maneuver only; under Maneuver the target's front within 3 tiles (`round_intent::in_front`); a white ring on every unit a click would make the round target (`target_problem`; "valid cells lit" lights units, not every tile an area or Guard tile could take); the planned moves tile by tile (a plain move before the commands solid gold, a move carried by its skill dashed amber, a kite step or retreat after the commands dashed gold with an arrow head); the gold target ring. Always: a red chevron ahead of each enemy toward its facing (MV1) and a lavender veil under each stealthed unit, filled once it is hidden (GD5); the hover card adds "Hidden: ..." then. "Reach" on the hover card and the hero panel is the plain attack band under the rule `reach` ("Reach 1-2"), else `attack_range`. Agent-link state key `overlay` {target, valid, band, front, moves [{kind, tiles}], facings [{at, facing}], veils [{at, hidden}]}.

RULESET1-U slice 4 as built (2026-10-30): under the rule `plan_badges` (`Session::round_intents`, both sides, planning and playback) the hostile card's line leads with the round intent and target ("Assault: Warrior; Cut B1", `intent_text::round_card_line`); every hover card and the threat intel panel start with the unit's "Intent (kind): target" in the intent's colour; the intel panel shows the hovered unit of either side (its card or its token; widget `intel.unit`), else the first foe; during the playback an enemy's round record stands in for its tell, so a reveal and its readers stay listed (a reader who fell included). The shared intent glyph is a 10 px plate in the intent's command colour (Assault an attack's red, Guard a Defend's teal, Support green, Maneuver a Disengage's blue) with its initial: on the strip after the hero's name, at a party card's top right, before a hostile card's name, at a token's top right (Flat) or right of the tell chip (Stage); an enemy's glyph shows only while the tells show. V1: each hostile card gets a row of the round's blocks with a red mark at each hit's impact block and a gold one for other actions (widget `foe.<i>.badges`, a tip line per badge). V2: a revealed plan fills its actions on that row in their kind's colour (recovery dim), its tip says "Plan revealed" (or who reads it, or the enrage) and lists the plan, and the board draws each revealed enemy's walk dashed red (overlay `revealed_moves`). Review items: the overlay's move line starts at its entry's `pos_before`; the attack tile of the reach band is `strike_tile` (before a Disengage retreat); "Hidden" is gold on an enemy's card. The facing chevron turned pale (rgb 250, 226, 160): red read as part of the tells' arrows next to the glyphs (screenshots `slice4_chevron_red.png`, `slice4_chevron_pale.png`); the intent line on a card uses the intent's colour lifted toward white.

RULESET1-U slice 5 as built (2026-10-30): under the rule `poise`, a 2 px poise bar (pale steel blue; a windup bonus over the max shows a gold tip; red and empty once broken) under the HP bar of each token and party card, and "Poise v/max" (BROKEN this round, or broke last round) on the hover card; a BREAK plate over each unit broken this round (P7) and a BREAK mark (engine `timeline::draw_mark`) on the hero's cancelled strip span. The hero's strip spans take the timeline widget's parts (engine `timeline::draw_part`): a command that winds up (P5) over more than one block shades its wind-up blocks and brightens its impact block; a scheduled enemy-style recovery is hatched. Forecasts (index 12): each planned attack of the hero is forecast with kit `poise::forecast` against the enemies' scheduled actions on the bar its earlier planned hits leave (one break a round), flank from the tile it strikes from (`round_intent::flank_bonus_from`); a hit with no stagger, flank or pierce has none (the Ashlands skills carry no stagger); the strip tip lists "Forecast on X", "Stagger s x in wind-up 2 = t", the verdict ("Breaks: poise b -> 0", "Breaks: a sure break into its wind-up", "May break: c% (poise b -> a)", "Poise b -> a", "Already broken this round"), "Flank", "Armour pierced"; the intel panel adds `Your <skill> B<n>: ...` per hit on its unit. The round summary (kit `round_intent::round_summary`) leads the round-over message ("Round over: 1 break (1 of 2 hits into a wind-up), 1 flank, 3 follow-ups, Rest +1 AP"; follow-ups count each member's commands after its first, moves and Rest aside) and the state key `round_summary`. Every timeline overlay group has a flag (preference "Battle overlays and the clarity pass"): settings key `battle_overlays` {band, front, valid, veils, moves, target, facings, walks, glyphs, badges, poise, breaks, parts, forecasts, summary}, false hides a group (all on by default; slice 6 gives them display settings), state key `overlay_layers`. Review items: a revealed enemy's moves show only for the hovered enemy (its card or token) or while Alt is held; its walks are drawn as kit `command::walk_preview` walks them (stopping where `World::step_from` refuses a step, on an exit, on a trap), its carried Engage dash and its Disengage retreat or jump too (state `overlay.revealed_moves` [{unit, kind, tiles}]); the hostile card's glyph comes from the round records like the token's, so they agree during the playback; the Retreat-carry band and an unrevealed enemy's left-out walks are tested on real plans.

RULESET1-U slice 6 as built (2026-10-30, the last slice): the Settings screen gains a Battle tab (between Gameplay and Controls) listing the fifteen overlay groups (`battle::OVERLAY_GROUPS`: id, label, hover text) as checkboxes `settings.overlay.<id>`, saved under `battle_overlays`; `screens::default_settings` holds them all on, so Restore defaults turns every one back on (preference "Battle overlays and the clarity pass"; VISCLAR1 decides later what the game view keeps); the Controls list adds "Every revealed enemy move (timeline): Hold Alt". Decision "RULESET1-U slice 5 answers": under the round intent every round closes the text combat log with `[Round] <n>: <summary>` (kit `round_intent::summary_line` of `round_summary`, written as the round ends; classic logs none; the timeline golden traces hold the session's structured log, so they are unchanged), and the hovered enemy's revealed moves draw above the tokens (over a dark underlay; Alt still draws the others on the ground). The slice 5 review: the forecasts run through kit `poise::forecast_hits` (a sequence of `PlannedHit`s, each on the bar the earlier ones leave) and `Command::pierces` (now public). TEXTFIT1 at 0.85, 1.0, 1.15 and 1.3 at 1280x720: a strip span's first block widget spans the whole span (its label no longer spills; the later blocks keep their own hover cards), the hero panel's line pitch follows the text size (8 at Small and Normal, as before) so the derived-stat line no longer spills at 1.3, and the intent buttons fall back to one a row when a name would not fit two a row. Left as the preference allows: a one-block span's label shortens ("Off-hand Jab" to "Off-ha..."; the full name is on hover); the damage floats count as spills over the board's tile widgets in the audit (transient effects, not layout).

RULESET1-G slice 1 as built (2026-10-29): the Dojo. Kit `DojoSession`: `set_ruleset` rebuilds the heroes with `build_unit_in` (gear, pouch, spent points kept), `set_deck`, `deck_of`, `plan_view` / `plan_view_of` / `plans_by_intent` / `plan_openers` (a scratch `Session`: hero, dummy, optional second dummy engaging it), `set_plan_intent`, `place_plan`, `toggle_dummy_windup`, poise bars `hero_poise` / `dummy_poise`, `rest_blocks`; `poise::max_poise_of`. Game: windows `planner` and `deck`, layout "Timeline"; widget ids `dojo.plan.{intent.<id>,kind.<k>,opener,cell.<x>.<y>,piece.<p>,partner,row.<k>,intent_plan.<intent>}`, `dojo.deck.{<kind>.<i>,basics,owned.<kind>.<id>,order.<intent>.<k>,default,auto_order}`, `dojo.windup`; state (timeline only) `ruleset`, `deck`, `actives`, `poise`, `dummy_poise`, `dummy_windup`, `plan`. `DeckTerms::base` (user 2026-10-29): only an actives or order edit pins the automatic actives. Left: the Arena toggles and chip, the sim `feel` mode.

RULESET1-G slice 2 as built (2026-10-29): the Arena and `feel`. Scene Runner Arena card: widget ids
`runner.arena.{reveal,chip,rules,rule.<id>}`; the toggles list every register extension (HOOKS1's too) and may switch
one on over classic as `rules=` may (`Ruleset::adjust`); the chip and the toggles live in the Scene Runner card, not on
the battle screen; runner state gains `rules`, `rules_on`, `reveal`, `chip`; `BattleScreen::arena_with(cx, world,
encounter, ruleset, rules, plan_reveal)`; the battle state gains `ruleset` only beyond classic. `feel` (tasks 3) differs
in its inputs: `seeds=N` is also a key (the positional seeds win); the default encounters come from the game
(`SimGame.feel_encounters`, the kit names none); "guard blocks" are the blocks of landed Defend commands (hits into a
guard are counted apart); buffs dropped are read from the session's events (the party's); planner reasons are tallied by
kind (numbers dropped); a fight skipped for jitter is not counted. Dojo review fixes: `PlanView.ap_before` is the scratch
battle's round-1 AP; `set_active` drops the pending Rest blocks (documented); the GUI planner cache is keyed on a screen
revision and `state()["plan"]` is computed when stale (window closed too).

| RULESET1-G | Dojo toggle, planner view and readouts, Arena select and toggles, sim `feel` (index 12, tasks 3) | kit + game | logic | P, K, V, PL | `b/dojo.rs`; `gui/{dojo,runner}.rs`; `rpgkit_sim/src/{args,modes,feel (new)}.rs` |
| RULESET1-A | Test kit data and the `sunk_ward` field (2) | game | asset | R, S, DERIVE1-S | `data/catalog/{rulesets,derived_stats,classes,item_families,items,skills,passives,units,encounters}.json`; `data/worlds/sunk_ward/world.json` |
| RULESET1-X | Golden traces and the inert-extensions check (4.2) | kit | logic | A, G, AI | `rpgkit_battle/tests/{timeline_golden.rs,fixtures/*}` |
| RULESET1-B | Feel sim report: stagger and poise factors, chance, sure break, pierce at 25/35/50, Rest gain, flanks, carried moves, kiting AP, intent variety (OI1), lock rate | game | balance | X | `games/emberwake/docs/BALANCE_REPORT.md` |

R and S are schema-first. IN, PL, P, K share `plan.rs`, `command.rs`, `resolver.rs`: run them in that order; V may
run beside IN after T. Sequence ENCOUNTER2 (`schema.json`, `session.rs`) with R. Kit rows: kit crates (KIT1).

## 7. Docs delta (builders report, the planner applies)

- `GAME_BRIEF.md`: a Rulesets section. `docs/STRUCTURE.md`: `r/ruleset.rs`,
  `b/{round_intent,round_intent_plan,poise}.rs`, `rpgkit_sim/src/feel.rs`, `rulesets.json`, the fixtures.
- `engine/README.md` (widget marks and parts); `PORT_NOTES.md` (new modules port no Godot file); authoring notes
  "rulesets: options and extensions" and "a round-intent planner". Lore: `combat_and_timeline.md` 6 (reach 2);
  `rules_engine_architecture.md` 3.1, 4.2 (1.3's names). HISTORY: section 5's strips.
