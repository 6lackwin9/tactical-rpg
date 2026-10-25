# RULESET1 tasks: schema, assets, tests, parity and the task split

Status: proposed, revised 2026-10-25 (companion of [RULESET1.md](RULESET1.md), rule ids T, P, K, D, V, and
[RULESET1_STANCE.md](RULESET1_STANCE.md), rule ids ST, TG, FU, AS, GD, MV, SP, RS, PL, MM, AI)

## 1. Schema contract (schema-first: yes; R and S)

New schema rows join `AFTER_GODOT` (`emberwake_world/tests/data_source.rs:25`); defaults keep today's behaviour.

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
| `stances` | enums | [`assault`, `guard`, `support`, `maneuver`] | subset of those four and `channel`, no repeats; at least one; needs `round_stance` in `rules` |
| `basics` | refs skills | [] | skills whose stance (ST4 default or `round_stance`) is neutral or in `stances`; required with `deck`; no repeats |
| `poise_stat` | string | "" | a `derived_stats` id; required when `rules` has `poise` |
| `stagger_windup_mult`, `_guard_mult`, `_repeat_mult` | floats | 2.0, 0.5, 0.5 | 1.0-4.0, 0.0-1.0, 0.0-1.0 |
| `flank_stagger_mult`, `flank_power_mult` | floats | 1.5, 1.0 | 1.0-3.0, 1.0-2.0 (TQ12; b sets power 1.25) |
| `break_chance_scale`, `break_chance_cap` | float, int | 0.0, 50 | 0.0-1.0 (0 = off), 0-100 |
| `rest_ap_per_block`, `rest_ap_cap`, `rest_poise_per_block` | float, int, int | 0.5, 2, 5 | 0.0-2.0, 0-10, 0-50 (RS2, RS3) |
| `maneuver_move_ap`, `maneuver_move_speed` | floats | 0.5, 2.0 | 0.0-1.0, 1.0-3.0 (MV4) |
| `deck_slots` | json | {} | object with `active` 1-8, `passive` 0-4, `keystone` 0-2; required with `deck` |
| `origin` | enum | system | as every catalog kind (the first draft's `deck_base`, `deck_system`: gone) |

### 1.3 Skills (`content.skills`, RULESET1-S)

| Key | Type | Default | Scope | Validation |
| --- | --- | --- | --- | --- |
| `round_stance` | enum `""`, `assault`, `guard`, `support`, `maneuver`, `channel`, `neutral` | "" | family | "" = by kind; a set value must be allowed for the kind (ST4 table); `channel` only when a ruleset row lists it |
| `stance_buff` | bool | false | family | DEFEND, SUPPORT, FOCUS; needs `grant_status` or `extra_grants` (ST6) |
| `ap_cost` | int | -1 | rank | -1 or 0-10; kinds ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE |
| `recovery_blocks` | int | 0 | rank | 0-2; kinds ATTACK, SUPPORT, FOCUS; `blocks + recovery_blocks <= 6`; read for enemy-side units only (T4) |
| `stagger` | int | 0 | rank | 0-100; ATTACK only |
| `windup_poise` | int | 0 | rank | 0-100; ATTACK, SUPPORT, FOCUS |
| `unbreakable` | bool | false | family | ATTACK, SUPPORT, FOCUS; warning when `windup_poise > 0` too |
| `sure_break` | bool | false | family | ATTACK; needs `stagger >= 1` on every rank |
| `armor_piercing` | bool | false | family | ATTACK |

`exploit` accepts `{"status": "flanked", "power_mult": x}` (MV3; timeline only, classic never matches it). Text
tokens `{stagger}`, `{recovery}`, `{ap}` (counted like `blocks`, `check/skill.rs:343`). `follow_ups`: dropped, unbuilt.

### 1.4 Passives (`content.passives`, RULESET1-S)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `slot` | enum `""`, `passive`, `keystone` | `""` | `""` = source-granted, always on |
| effect `stagger_taken` | `{"type", "mult", "max_blocks"}` | - | `mult` 0.0-2.0; `max_blocks` 0-6 (0 = any action) |
| effect `foresight` | adds optional `range` | 0 | 0-8 (0 = today's rule) |

### 1.5 Units (`content.units`, RULESET1-S)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `deck` | json | {} | keys `active`, `passive`, `keystone`, `order`; `active`: owned families (path or `extra_skills`), no basic; `passive`/`keystone`: passives of that `slot`; counts within `deck_slots`; `order`: stance -> families from `active` or `basics` of that stance; no repeats anywhere |
| `poise` | int | -1 | -1 (derived), 0 (no bar), 1-200 (fixed Max Poise) |

### 1.6 Item families and items (RULESET1-S)

- `item_families.engine_rules` gains `reach`: WEAPON only, not with `always_ranged`, every hand's base range 1.
- `items.weapon_type` options gain `polearm` and `scourge` (TQ8); classes may train them like any type.

### 1.7 Status table and formula input (code)

- `status.rs` rows 16 `steadfast` (buff, not resistable, 1 round, RefreshMaxDuration, cap 0, grant "support") and 17
  `stealth` (same, 2 rounds, display "Stealthed", grant "defend"); column `stance_bound` (`riposte`, `stealth` true;
  classic never reads it); `StatusState` gains a per-status stance tag (ST6).
- DERIVE1-R gains the reserved formula input `gear_mitigation` (`Inventory::total_flat_mitigation`, `gear.rs:298`),
  rejected as a row id or constant name like `level`.

### 1.8 Runtime and save

- `EncounterDef`: `ruleset: Ruleset` (default classic), `plan_reveal: bool` (default false); not read from rows.
- `FightSpec`: `ruleset`, `rules`; sim args `ruleset=classic|timeline`, `rules=-id,+id` outside the Godot-parity help
  text (as `rewards`, `args.rs:53-55`); host `battle` command field `ruleset`.
- Battle unit state (timeline only): `round_stance` (none at battle start), `facing`, `moved_this_round`.
- Plan: per unit `RoundIntent { stance, target (self, unit, cell), kind (engage, disengage, utility, none), opener
  (skill, item, none), manual }`; host requests `stance {unit, stance, target, kind, opener}`, `manual {unit, on}`
  (replays record them).
- Party member `deck` (1.5's shape): save step N+1 adds `{}` (or it joins M0's new save format if that lands first).

## 2. Assets (RULESET1-A, asset lane)

Through `tools/fc.sh content upsert <kind> @<file>`, then `content check`; new rows appended, no Ashlands row changes
(their stances are kind defaults). Order: derived_stats, class, family, item, skills, passives, rulesets, units,
encounters, world field.

### 2.1 Ruleset row `timeline` (`name` "Timeline")

| Key | Value |
| --- | --- |
| `rules` | `round_stance`, `enemy_moves`, `deck`, `poise`, `ap_cost`, `pierce`, `reach`, `flank`, `plan_badges` |
| `stances` | `assault`, `guard`, `support`, `maneuver` |
| `basics` | `move`, `rest`, `strike`, `offhand_jab`, `active_guard`; TQ15 a adds `flank_cut` (ATTACK, maneuver, blocks 1, AP 2, power 0.6, stagger 8, `basic` false) |
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

### 2.4 Test skills (one rank each, tier 1, level 1, `learned_only` true except `tl_thrust`)

Rec. is read for enemies only (T4). Every stance has a skill; Engage example Flank Cut, Disengage Slip Strike, Utility
the existing pouch item `con_fire_bomb` (thrown, reach 3, damage 45).

| Id | Name | Kind | Stance | Blocks | Rec. | AP | Power | Stagger | Flags |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `active_guard` | Active Guard | DEFEND | guard | 2 | 0 | 1 | - | 0 | mitigation 0.5; `basic` false (a timeline basic by `basics`) |
| `tl_brace_counter` | Brace and Counter | DEFEND | guard | 2 | 0 | 2 | - | 0 | mitigation 0.3; grant `riposte` 1 round |
| `tl_fade` | Fade | DEFEND | guard | 1 | 0 | 1 | - | 0 | mitigation 0.2; grant `stealth` 2 rounds |
| `tl_field_dressing` | Field Dressing | SUPPORT | support | 2 | 0 | 2 | - | 0 | heal_base 8, heal_mult 0.3; ALLY_OR_SELF |
| `tl_flank_cut` | Flank Cut | ATTACK | maneuver | 1 | 1 | 2 | 0.8 | 10 | exploit `flanked` x1.3 |
| `tl_slip_strike` | Slip Strike | ATTACK | maneuver | 2 | 1 | 2 | 0.9 | 8 | status `slow` potency 60, 1 round |
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
naming stance, tier and role ("Assault, Snap: a quick jab that staggers."). Stagger = lore x 0.5 (index 5.6).
`round_stance` is written on every row (only the two Maneuver attacks differ from the kind default).

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
| `tl_spiker` | party | tl_drill | wpn_dagger, arm_leather; con_fire_bomb | marrow_spike, flank_cut, slip_strike; -; assault [marrow_spike, strike], maneuver [flank_cut, slip_strike] | -1 | -1 |
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
| `tl_flank` | tl_vanguard, tl_spiker | tl_footman, tl_crossbowman | Engage flank, Disengage, Utility fire bomb, kiting |
| `tl_skirmish` | the four party units | tl_footman twice, tl_enforcer, tl_crossbowman | the 4-on-4 loop, the AI table |

A roster that may not repeat an id gets `tl_footman_b`. `sunk_ward`: `ruleset` `timeline` once M0's skeleton exists.

## 3. The sim's feel mode (RULESET1-G)

`aegis sim feel [enc=a,b] [seeds=N] [rules=...]`: default the five `tl_` encounters, 10 seeds, timeline. Reports won %,
median rounds, breaks, windup breaks / hits, chance rolls and successes, sure breaks, guard blocks, pierce share, AP
per round, lock rate (broken two rounds running), stance share, follow-ups, Rest blocks and Rest AP, flanks, kite
steps, lost attacks, stance changes and buffs dropped, planner reasons, rounds whose stance had no active (OI1).

## 4. Tests (rule-derived)

### 4.1 Per task

| Task | Test | Expectation from |
| --- | --- | --- |
| R | scratch worlds: A `timeline`, B extends A with `""`, C unset; the three shipped worlds | timeline, timeline, classic, classic (3.1.1) |
| R | a timeline world with no `rulesets` row; each 1.2 violation | one content error each |
| S | each 1.3-1.6 violation (a DEFEND set `assault`, `channel` unlisted, an `order` entry of another stance); `legacy/data` | one error each, field named; no new error |
| T | an enemy skill with blocks b, recovery r at start s; the same on a hero | enemy s..s+b+r-1, lands s+b-1; hero s..s+b-1 (T1, T4) |
| T | catalog skills with `ap_cost` -1; set values | the old formula (T10); `ap_cost + modifier` in 0..10 |
| T | an enemy round under `enemy_moves` | AP unchanged, no debt status (T11) |
| ST | an Attack under timeline | no classic stance change, switch AP 0, matchup 1.0 (ST3) |
| ST | another stance's skill, another target, a plain move, Rest | refused, refused, accepted, accepted (ST5, TG1, MM1) |
| ST | a stance change holding riposte, stealth, a `stance_buff` grant, haste; no change | three removed, haste kept; nothing removed (ST2, ST6) |
| ST | three landed Rest blocks at 0.5 / 2 / 5; classic Rest | 1 AP at round end, 15 poise; classic 2 AP on landing (RS2-RS4) |
| ST | the target downed before the third command | it and later ones are Rest, unpaid (TG4) |
| ST | T engaged with E to its east; A west, north, north-east; T's target is A | flank, flank, no, no (MV2, MV3) |
| ST | a 4-tile move at 2 tiles a block | maneuver 2 AP, 1 block; standard 4 AP, 2 blocks (MV4) |
| ST | stealthed unit moved in b1, a locked attack lands in b3; an area on it; unit did not move | nothing dealt, AP spent; hit; hit (GD5) |
| PL | the three worked examples (stance 4) | their plans, AP left 3, 2, 0 |
| PL | the same inputs twice | identical plans and reason tags (PL1, PL6) |
| PL | Engage with no tile outside the front; Disengage out of band | not offered, reason shown (MV5, MV6, PL7) |
| PL | a ranged unit, target 4 tiles away and threatening | last block reserved, the step after the attacks (AS3) |
| AI | each AI table row in a scratch fight | that stance and target (AI1) |
| P | random stagger s, every windup / guard / repeat / flank / passive combination | `roundi(s x product)` (P6) |
| P | a break in block k: a later same-block action, Rest blocks, a running guard, an unstarted action | all cancelled, guard 0 at once, unstarted AP unpaid (P7) |
| P | two breaking hits in a round; a hit the next round | one break; stagger x repeat (P8, P10) |
| P | `windup_poise`, `unbreakable`, `steadfast` | poise rises and falls back; never below 1 (P11, P12) |
| P | chance scale 0 vs 0.5 on a seed | 0 draws vs one per qualifying hit; same seed, same result (P13) |
| P | `sure_break` into a windup, into an unbreakable windup | breaks; does not (P14) |
| P | `armor_piercing` hit | `compute` with rating 0; guard and cover still cut (A2) |
| K | a polearm wielder | band (1, 2); approach and Maneuver moves stop at 2; no riposte at 2; cover 0; `attack_range` 1 |
| V | a timeline round | tells with stance and target; badges at every live non-move impact; `plan_reveal` shows all; foresight `range` 4 |
| D | units with and without `deck` and `order` | auto deck and FU4 order; slotted passives only; classic ignores `slot`; `basics` usable, `guard`, `switch_target` not |
| D | an old save | loads with `deck` {} (migration) |
| E | widget with marks and segment parts | drawn inside their blocks; specimen page; clicks unchanged |
| U, DU | battle and party screens at 1280x720, four text sizes | no text-audit fault; classic screens as before |
| G | sim help text; `feel` twice on a seed; the Dojo planner table | unchanged; identical; one plan per stance |

### 4.2 Classic unchanged and golden traces (RULESET1-X)

- Every parity suite passes untouched (section 5); each Ashlands encounter traced with classic and with every
  extension of 3.2 on (options off) gives identical traces, logs, metrics and RNG draw counts; classic stances, switch
  AP, matchup, Rest (2 AP, Tactical) and riposte removal stay byte-identical (`round_stance` off).
- `emberwake_battle/tests/timeline_golden.rs` runs the five `tl_` encounters for seeds 1337, 7 and 42 through
  `sim::fight` on a frozen kit copy (`tests/fixtures/timeline_kit.json`) against `tests/fixtures/timeline_golden.json`,
  recorded once when X lands; re-recorded only for an intended rule change named in the commit and a HISTORY row.

## 5. Parity plan

| Suite | Effect | Action |
| --- | --- | --- |
| `trace_parity`, `world_traces`, `journey` | none (classic, no new keys) | none |
| `data_source` | new rows, kind `rulesets`, option lists of `weapon_type` and `engine_rules` | `AFTER_GODOT` rows; a kind strip as DERIVE1-S; the two option rows skip `options` like `HELP_CHANGED` (`:38`); HISTORY row |
| `saves` | members gain `deck` | strip it as v12's `reward_paid` (`saves.rs:40-43`); HISTORY row |
| `checks_parity`, `sim_parity`, phasefold golden | none: no Godot row names a new field; args outside the help | none |
| `timeline_golden` (new) | engine-only reference | 4.2 |

## 6. Tasks

Crate paths are under `games/emberwake/crates/` (`b/` = `emberwake_battle/src/`, `r/` = `emberwake_rules/src/`).

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| RULESET1-R | Ruleset switch: world fields, `rulesets` kind, `Ruleset`, `EncounterDef.ruleset`, bridge, sim and host args; classic stances inert under `round_stance` (3, 1.1, 1.2, ST3) | kit | logic | - | `schema.json`; `emberwake_data`; `emberwake_content/src/{source,store,check/*}`; `r/ruleset.rs` (new); `b/{session,plan,skill,command,sim}.rs`; `emberwake_world/src/{data,battle}.rs`; `emberwake_gui/src/battle.rs`; `emberwake_sim/src/args.rs`; `emberwake_host`; `data_source.rs` |
| RULESET1-S | Timeline fields: skills (`round_stance`, `stance_buff`, `recovery_blocks`, poise flags), `exploit` `flanked`, passives, units (`deck`, `order`, `poise`), families, `steadfast`, `stealth`, `stance_bound`, checks (1.3-1.7) | kit | logic | R | `schema.json`; `emberwake_data/src/lib.rs`; `emberwake_content/src/check/{skill,simple,unit,item}.rs`; `r/status.rs`; `data_source.rs` |
| RULESET1-T | Enemy recovery and no enemy AP (`enemy_moves`), `ap_cost`, `ScheduledAction.impact` (4.2, 4.4) | kit | logic | S | `b/{resolver,plan,skill,intent,log}.rs`; tests |
| RULESET1-ST | Round stance rules: stance and persistence, stance buffs, one target, stance filter in `add_step`, neutral moves, Rest, maneuver moves and after-moves, facing, engaged, flank, stealth, cover tiles (stance 2-6) | kit | logic | T | `b/{stance (new),plan,resolver,command,skill,world}.rs`; `r/{unit,status,damage}.rs`; tests |
| RULESET1-PL | Auto planner: opener, tiles, follow-up fill, after-moves, reason tags, manual fill, `deck.order` (stance 4, 7, 8) | kit | logic | ST | `b/{stance_plan (new),plan}.rs`; tests |
| RULESET1-P | Poise, stagger with the flank factor, breaks, chance, sure break, guard halving, pierce, metrics (5, 6, 7) | kit | logic | ST, DERIVE1-R | `r/{unit,damage,derived}.rs`; `b/{poise (new),resolver,command,session,metrics,log}.rs`; tests |
| RULESET1-K | Reach-2 melee (8) | kit | logic | S (after P: shared files) | `r/{unit,gear}.rs`; `b/{skill,command,plan,ai}.rs`; tests |
| RULESET1-V | Badges, stance in tells, `plan_reveal`, foresight range, timeline trace keys (10) | kit | logic | T | `b/{intent,session}.rs`; `r/unit.rs`; `emberwake_world/src/battle.rs`; tests |
| RULESET1-AI | Enemy and party AI: the stance pick table, enemy order lists, AUTO1 mapping (stance 9) | kit | logic | PL, V | `b/{ai,world}.rs`; `emberwake_gui/src/battle.rs` (tactics); tests |
| RULESET1-D | Deck rules: slot kinds, `basics`, order lists, slotted passives, member `deck`, save step (9 D1-D5) | kit + game | logic | S | `r/unit.rs`; `b/{skill,world}.rs`; `emberwake_world/src/{state,save,progression}.rs`; `saves.rs`; tests |
| RULESET1-E | Timeline widget: marks (glyph, colour, tip at a block) and segment parts (windup, impact, recovery; hatching; links) | engine | logic | - | `engine/crates/aegis_ui/src/{timeline,specimens}.rs`; `aegis_ui/tests/kit_widgets.rs`; `engine/README.md` |
| RULESET1-U | Battle screen: stance buttons, target pick, Maneuver kinds, opener, Manual, reasons, after-moves, flank arc, stealth veil, badges, BREAK, poise bars, forecasts, reach band, summary (stance 10, 11) | game | logic | E, P, K, V, PL | `emberwake_gui/src/{battle,intent_text,hud}.rs` |
| RULESET1-DU | Deck buckets, Basics row and order editor (party screen, Dojo; 9 D6) | game | logic | D | `emberwake_gui/src/{party,dojo}.rs` |
| RULESET1-G | Dojo toggle, planner view and readouts, Arena select and toggles, sim `feel` (stance 11, 3) | game | logic | P, K, V, PL | `b/dojo.rs`; `emberwake_gui/src/{dojo,runner}.rs`; `emberwake_sim/src/{args,modes,feel (new)}.rs` |
| RULESET1-A | Test kit data and the `sunk_ward` field (2) | game | asset | R, S, DERIVE1-S | `data/catalog/{rulesets,derived_stats,classes,item_families,items,skills,passives,units,encounters}.json`; `data/worlds/sunk_ward/world.json` |
| RULESET1-X | Golden traces and the inert-extensions check (4.2) | kit | logic | A, G, AI | `emberwake_battle/tests/{timeline_golden.rs,fixtures/*}` |
| RULESET1-B | Feel sim report: stagger and poise factors, chance, sure break, pierce at 25/35/50, Rest gain, flanks, kiting AP, stance variety (OI1), lock rate | game | balance | X | `games/emberwake/docs/BALANCE_REPORT.md` |

R and S are schema-first. ST, PL, P, K share `plan.rs`, `command.rs`, `resolver.rs`: run them in that order; V may
run beside ST after T. Sequence ENCOUNTER2 (`schema.json`, `session.rs`) with R. With KIT1 first, kit rows move there.

## 7. Docs delta (builders report, the planner applies)

- `GAME_BRIEF.md`: a Rulesets section. `docs/STRUCTURE.md`: `emberwake_rules/src/ruleset.rs`,
  `emberwake_battle/src/{stance,stance_plan,poise}.rs`, `emberwake_sim/src/feel.rs`, `rulesets.json`, the fixtures.
- `engine/README.md` (widget marks and parts); `PORT_NOTES.md` (new modules port no Godot file); `engine/docs/authoring/`
  notes "rulesets: options and extensions" and "a round-intent planner".
- CANON 2.1 "Cut" (planner): it still lists facing and backstab; a derived facing returns for flanks, stealth without
  cones. Lore: `combat_and_timeline.md` 6 (reach 2 is new); `rules_engine_architecture.md` 3.1, 4.2 (1.3's names).
- HISTORY rows: section 5's strips; dropped first-draft ideas (authored `follow_ups`, base slots, player recovery, 2 AP
  a Rest block under timeline). Code survey: [RULESET1_STANCE.md](RULESET1_STANCE.md) section 12.
