# RULESET1 tasks: schema, assets, tests, parity and the task split

Status: proposed (companion of [RULESET1.md](RULESET1.md); rule ids T1, P1, K1, D1, V1 and so on are its)

## 1. Schema contract (schema-first: yes; RULESET1-R and RULESET1-S)

New schema rows join `AFTER_GODOT` (`emberwake_world/tests/data_source.rs:25`); every default keeps today's behaviour.

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
| `rules` | strings | [] | subset of the register ids (spec 3.2), no repeats |
| `poise_stat` | string | "" | a `derived_stats` id; required when `rules` has `poise` |
| `stagger_windup_mult` | float | 2.0 | 1.0-4.0 |
| `stagger_guard_mult` | float | 0.5 | 0.0-1.0 |
| `stagger_repeat_mult` | float | 0.5 | 0.0-1.0 |
| `break_chance_scale` | float | 0.0 | 0.0-1.0 (0 = off) |
| `break_chance_cap` | int | 50 | 0-100 |
| `deck_base` | refs skills | [] | each a skill with kind MOVE or DEFEND; required with `deck` |
| `deck_system` | refs skills | [] | no overlap with `deck_base` |
| `deck_slots` | json | {} | object with `active` 1-8, `passive` 0-4, `keystone` 0-2; required with `deck` |
| `origin` | enum | system | as every catalog kind |

### 1.3 Skills (`content.skills`, RULESET1-S)

| Key | Type | Default | Scope | Validation |
| --- | --- | --- | --- | --- |
| `ap_cost` | int | -1 | rank | -1 or 0-10; kinds ATTACK, DEFEND, FOCUS, SUPPORT, DISENGAGE |
| `recovery_blocks` | int | 0 | rank | 0-2; kinds ATTACK, SUPPORT, FOCUS; `blocks + recovery_blocks <= 6` |
| `follow_ups` | refs (family heads) | [] | family | at most 4; not the skill's own family; every rank needs `recovery_blocks >= 1` |
| `stagger` | int | 0 | rank | 0-100; ATTACK only |
| `windup_poise` | int | 0 | rank | 0-100; ATTACK, SUPPORT, FOCUS |
| `unbreakable` | bool | false | family | ATTACK, SUPPORT, FOCUS; warning when `windup_poise > 0` too |
| `sure_break` | bool | false | family | ATTACK; needs `stagger >= 1` on every rank |
| `armor_piercing` | bool | false | family | ATTACK |

Skill text: template tokens `{stagger}`, `{recovery}` and `{ap}` (counted like `blocks`, `check/skill.rs:343`).

### 1.4 Passives (`content.passives`, RULESET1-S)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `slot` | enum `""`, `passive`, `keystone` | `""` | `""` = source-granted, always on |
| effect `stagger_taken` | `{"type", "mult", "max_blocks"}` | - | `mult` 0.0-2.0; `max_blocks` 0-6 (0 = any action) |
| effect `foresight` | adds optional `range` | 0 | 0-8 (0 = today's rule) |

### 1.5 Units (`content.units`, RULESET1-S)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `deck` | json | {} | keys `active`, `passive`, `keystone` only; lists of ids; `active` names owned families (class path or `extra_skills`), no basic, no repeats; `passive`/`keystone` name passives whose `slot` matches; counts within the timeline row's `deck_slots` |
| `poise` | int | -1 | -1 (derived), 0 (no bar), 1-200 (fixed Max Poise) |

### 1.6 Item families and items (RULESET1-S)

- `item_families.engine_rules` gains `reach`: WEAPON only, not with `always_ranged`, every hand's base range 1.
- `items.weapon_type` options gain `polearm` and `scourge` (TQ8); classes may train them like any type.

### 1.7 Status table and formula input (code)

- `status.rs` gains row 16 `steadfast`: buff, not resistable, 1 round, RefreshMaxDuration, cap 0, display "Steadfast",
  grant "support" (skills grant it with `grant_status` or `extra_grants`).
- DERIVE1-R gains the reserved formula input `gear_mitigation` (`Inventory::total_flat_mitigation`, `gear.rs:298`),
  rejected as a row id or constant name like `level`.

### 1.8 Runtime and save

- `EncounterDef`: `ruleset: Ruleset` (default classic), `plan_reveal: bool` (default false); not read from rows.
- `FightSpec`: `ruleset`, `rules` (overrides); sim args `ruleset=classic|timeline`, `rules=-id,+id`, outside the help
  text (the help lines are Godot parity, as `rewards`, `args.rs:53-55`); host `battle` command field `ruleset`.
- Party member state: `deck` (same shape as 1.5); save migration step N+1 adds `{}` (or the field joins the new
  campaign's save format if M0 lands that first).

## 2. Assets (RULESET1-A, asset lane)

All through `tools/fc.sh content upsert <kind> @<file>`, then `tools/fc.sh content check`. New rows only, appended; no
Ashlands row changes. Upsert order: rulesets row needs `active_guard` (skills) and `max_poise` (derived_stats), so:
derived_stats, class, family, item, skills, passives, rulesets, units, encounters, world field.

### 2.1 Ruleset row `timeline`

| Key | Value |
| --- | --- |
| `name` | Timeline |
| `rules` | `no_stances`, `deck`, `poise`, `recovery`, `chains`, `ap_cost`, `pierce`, `reach`, `plan_badges` |
| `poise_stat` | `max_poise` |
| multipliers | windup 2.0, guard 0.5, repeat 0.5 |
| break chance | scale 0.5, cap 50 (TQ2) |
| `deck_base` | `move`, `active_guard` |
| `deck_system` | `rest`, `switch_target`, `strike` (TQ3; drop `strike` if the user picks b) |
| `deck_slots` | `{"active": 3, "passive": 2, "keystone": 1}` |

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

| Id | Name | Kind | Blocks | Rec. | AP | Power | Stagger | Flags | Anchor (lore) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `active_guard` | Active Guard | DEFEND | 2 | 0 | 1 | - | 0 | mitigation 0.5 | CANON Active Guard |
| `tl_snap_jab` | Snap Jab | ATTACK | 1 | 0 | 2 | 0.7 | 12 | - | Pommel Strike (2 AP, stagger 25) |
| `tl_thrust` | Thrust | ATTACK | 2 | 0 | 2 | 1.0 | 8 | - | a Standard swing |
| `tl_heavy_cleave` | Heavy Cleave | ATTACK | 4 | 1 | 4 | 2.0 | 18 | follow_ups `tl_shoulder_check` | Sunder and Pin (4 AP, Heavy 4 +1) |
| `tl_shoulder_check` | Shoulder Check | ATTACK | 1 | 0 | 1 | 0.5 | 10 | - | the chain's follow-up |
| `tl_jaw_breaker` | Jaw-Breaker Drill | ATTACK | 1 | 0 | 3 | 0.8 | 15 | sure_break | Jaw-Breaker (2 AP, stagger 30), +1 AP |
| `tl_marrow_spike` | Marrow Spike | ATTACK | 1 | 0 | 1 | 0.9 | 0 | armor_piercing | Calcified Spike (1 AP; no Strain in M1) |
| `tl_hook_thrust` | Hook Thrust | ATTACK | 2 | 0 | 2 | 1.1 | 10 | - | a billhook thrust (reach from the family) |
| `tl_snap_shot` | Snap Shot | ATTACK | 1 | 0 | 2 | 0.6 | 6 | - | a ranged Snap |
| `tl_aimed_shot` | Aimed Shot | ATTACK | 2 | 0 | 2 | 1.1 | 6 | - | a ranged Standard |
| `tl_overhead_smash` | Overhead Smash | ATTACK | 3 | 1 | 3 | 1.8 | 15 | windup_poise 20 | an enemy Heavy |
| `tl_crushing_judgment` | Crushing Judgment | ATTACK | 4 | 1 | 4 | 2.2 | 20 | unbreakable | Bailiff Enforcer's (Heavy 4 +1) |

Descriptions: one plain sentence each naming tier and role ("Snap: a quick jab that staggers."). The stagger values
are the lore's x 0.5 (spec 5.7).

### 2.5 Test passives

| Id | Name | `slot` | Effects |
| --- | --- | --- | --- |
| `tl_braced_poise` | Braced Poise | passive | `[{"type": "stat", "stat": "max_poise", "value": 20}]` |
| `tl_anvil_posture` | Anvil Posture | keystone | `[{"type": "stagger_taken", "mult": 0.0, "max_blocks": 1}]` |
| `tl_hawk_eye` | Hawk Eye | passive | `[{"type": "foresight", "range": 4}]` |

### 2.6 Test units (level 1, `stat_mode` derived, `spend` {}, `extra_skills` = the deck's actives not on the path)

| Id | Side, tier | Class | Gear | `deck` | `armor` | `poise` |
| --- | --- | --- | --- | --- | --- | --- |
| `tl_vanguard` | party, hero | tl_drill | wpn_broadsword, shd_kite, arm_plate | active snap_jab, heavy_cleave, shoulder_check; passive braced_poise | -1 | -1 |
| `tl_halberdier` | party, hero | tl_drill | tl_billhook, arm_plate | active hook_thrust, jaw_breaker, snap_jab | -1 | -1 |
| `tl_spiker` | party, hero | tl_drill | wpn_dagger, arm_leather | active marrow_spike, snap_jab, thrust | -1 | -1 |
| `tl_archer` | party, hero | ranger | wpn_shortbow, arm_leather | active aimed_shot, snap_shot; passive hawk_eye | -1 | -1 |
| `tl_footman` | enemy, standard | tl_drill | wpn_shortsword, arm_leather | active thrust, snap_jab | 25 | -1 |
| `tl_enforcer` | enemy, elite | tl_drill | wpn_flanged_mace, arm_plate | active overhead_smash, thrust | 35 | -1 |
| `tl_captain` | enemy, boss | tl_drill | wpn_claymore, arm_plate | active crushing_judgment, thrust; keystone anvil_posture | 50 | 60 |
| `tl_crossbowman` | enemy, standard | ranger | wpn_shortbow, arm_leather | active aimed_shot, snap_shot | -1 | -1 |

(Skill ids in `deck` carry the `tl_` prefix; shortened above.) Expected poise for a check in the Dojo: footman about
26 + 4 = 30, vanguard about 26 + 8 + 6 + 20 = 60 (exact values follow the items' effective mitigation).

### 2.7 Test encounters (catalog) and the world field

Shared: zone `stone_arena`, arena CLOSED, exits [[11, 4]], max_rounds 8, total_blocks 6, ap_regen 4, unit_ap 10
(CANON's pool), rng_seed 1337, objective rout, deployment standard, morale false, pace auto.

| Id | Party | Enemies | Judges |
| --- | --- | --- | --- |
| `tl_duel_windup` | tl_vanguard | tl_enforcer | timing a Snap into a Heavy; the chain; Active Guard |
| `tl_duel_reach` | tl_halberdier | tl_footman | reach 2 against reach 1; the sure break |
| `tl_duel_armour` | tl_spiker | tl_captain | pierce against armour 50; an unbreakable windup |
| `tl_skirmish` | the four party units | tl_footman twice, tl_enforcer, tl_crossbowman | the 4-on-4 loop |

If a roster may not repeat an id, add `tl_footman_b` as a copy. World: `sunk_ward` manifest `ruleset` = `timeline`
once the M0 skeleton exists; `plan_reveal_flag` stays "" until M2's reconnaissance quest names its flag.

## 3. The sim's feel mode (RULESET1-G)

`aegis sim feel [enc=a,b] [seeds=N] [rules=...]`: default the four `tl_` encounters, 10 seeds, ruleset timeline. Per
encounter: won %, rounds (median), and per fight: breaks, windup breaks / windup hits, chance rolls and successes,
sure breaks, chains, guard blocks, pierce share of damage dealt, AP spent per round, lock rate (a unit broken in two
rounds in a row). `ruleset=` also works with every other mode; `world=sunk_ward` picks timeline by itself.

## 4. Tests (rule-derived; no number nobody printed)

### 4.1 Per task

| Task | Test | Expectation from |
| --- | --- | --- |
| R | scratch worlds: A `timeline`, B extends A with `""`, C unset; the three shipped worlds | timeline, timeline, classic, classic (3.1.1) |
| R | a world on timeline with no `rulesets` row; each 1.2 violation | one content error each |
| R | an Attack planned under timeline | no stance change, switch cost 0, matchup 1.0 in the breakdown |
| S | each 1.3-1.6 violation in a scratch catalog; `legacy/data` load | one error each, the field named; no new error |
| T | skill with blocks b, recovery r at start s | occupies s..s+b+r-1, lands at s+b-1; overlap refused (T1, T2) |
| T | the lead then a listed follow-up / an unlisted skill | starts at impact+1 / at impact+r+1; each AP at its own start (T6) |
| T | every catalog skill with `ap_cost` -1 | AP equals the old formula (T10); set: `ap_cost + modifier`, bounds 0..10 |
| P | random stagger s and every windup / guard / repeat / passive combination | `roundi(s x product)` (P6) |
| P | a break in block k with a later same-block action, a running guard and an unstarted action | all cancelled, guard 0 at once, unstarted AP unpaid (P7) |
| P | two breaking hits in one round; a hit the next round | one break; the next stagger x repeat (P8, P10) |
| P | `windup_poise`, `unbreakable`, `steadfast` | poise rises and falls back; never below 1 (P11, P12) |
| P | chance with scale 0 vs 0.5 on a seed | 0 draws vs one draw per qualifying hit; same seed, same result (P13) |
| P | `sure_break` into a windup, into an unbreakable windup | breaks; does not (P14) |
| P | `armor_piercing` hit | equals `compute` with rating 0; guard and cover still cut (A2) |
| K | a polearm wielder | band (1, 2); dash stops at distance 2 when a cell exists; no riposte at 2; cover 0; `attack_range` 1 |
| V | a timeline round | badges = impact blocks of every live non-move enemy action; with `plan_reveal` every plan shown; foresight `range` 4 reveals within 4 |
| D | units with and without `deck` | auto deck; slotted passive applies, unslotted does not; classic ignores `slot`; base moves present, `guard` absent |
| D | an old save | loads with `deck` {} (migration) |
| E | widget with marks and segment parts | drawn inside their blocks; specimen page shows them; clicks unchanged |
| U, DU | battle and party screens at 1280x720, four text sizes | no text-audit fault; classic screens draw as before |
| G | sim help text; `feel` on a seed twice | unchanged; identical output |

### 4.2 Classic unchanged

- Every parity suite passes untouched (section 5).
- Extensions are inert without data: each Ashlands encounter traced with classic and with every extension of 3.2 on
  (options off) gives identical traces, logs and metrics.
- Under classic the battle RNG draw count of a fight is unchanged (no break-chance draw).

### 4.3 Golden traces (RULESET1-X)

`emberwake_battle/tests/timeline_golden.rs` runs the four `tl_` encounters for seeds 1337, 7 and 42 through
`sim::fight` with timeline on a frozen copy of the test kit (`tests/fixtures/timeline_kit.json`, so data tuning does
not move it) and compares with `tests/fixtures/timeline_golden.json`, recorded once when X lands. Re-recording only
for an intended rule change, named in the commit and a HISTORY row (the phasefold golden's rule).

## 5. Parity plan

| Suite | Effect | Action |
| --- | --- | --- |
| `trace_parity`, `world_traces`, `journey` | none: classic by default, no new keys | none |
| `data_source` | new rows, kind `rulesets`, option lists of `weapon_type` and `engine_rules` | `AFTER_GODOT` rows; a kind strip as DERIVE1-S; the two option rows skip `options` like `HELP_CHANGED` (`:38`); HISTORY row |
| `saves` | members gain `deck` | strip it as v12's `reward_paid` (`saves.rs:40-43`); HISTORY row |
| `checks_parity`, `sim_parity`, phasefold golden | none: no Godot row names a new field; args outside the help | none |
| `timeline_golden` (new) | engine-only reference | 4.3 |

## 6. Tasks

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| RULESET1-R | Ruleset switch: world fields, `rulesets` kind, `Ruleset`, `EncounterDef.ruleset`, bridge, sim and host args, stances off (3, 1.1, 1.2) | kit | logic | - | `schema.json`; `emberwake_data`; `emberwake_content/src/{source,store,check/*}`; `emberwake_rules/src/ruleset.rs` (new); `emberwake_battle/src/{session,plan,skill,command,sim}.rs`; `emberwake_world/src/{data,battle}.rs`; `emberwake_gui/src/battle.rs` (bridge call); `emberwake_sim/src/args.rs`; `emberwake_host`; `data_source.rs` |
| RULESET1-S | Timeline fields: skills, passives, units, families, `steadfast`, template tokens, checks (1.3-1.7) | kit | logic | R | `schema.json`; `emberwake_data/src/lib.rs`; `emberwake_content/src/check/{skill,simple,unit,item}.rs`; `emberwake_rules/src/status.rs`; `data_source.rs` |
| RULESET1-T | Recovery, chains, `ap_cost`; `ScheduledAction.impact` (4) | kit | logic | S | `emberwake_battle/src/{resolver,plan,skill,ai,intent,log}.rs`; tests |
| RULESET1-P | Poise, stagger, breaks, chance, sure break, Active Guard halving, pierce, metrics (5, 6, 7) | kit | logic | T, DERIVE1-R | `emberwake_rules/src/{unit,damage,derived}.rs`; `emberwake_battle/src/{poise (new),resolver,command,session,metrics,log}.rs`; tests |
| RULESET1-K | Reach-2 melee (8) | kit | logic | S | `emberwake_rules/src/{unit,gear}.rs`; `emberwake_battle/src/{skill,command,plan,ai}.rs`; tests |
| RULESET1-V | Badges, `plan_reveal`, foresight range, timeline trace keys (10) | kit | logic | T | `emberwake_battle/src/{intent,session}.rs`; `emberwake_rules/src/unit.rs`; `emberwake_world/src/battle.rs`; tests |
| RULESET1-D | Deck rules: slot kinds, base and system moves, slotted passives, member `deck`, save step (9 D1-D5) | kit + game | logic | S | `emberwake_rules/src/unit.rs`; `emberwake_battle/src/{skill,world}.rs`; `emberwake_world/src/{state,save,progression}.rs`; `saves.rs`; tests |
| RULESET1-E | Timeline widget: marks (glyph, colour, tip at a block) and segment parts (windup, impact, recovery styles; hatching; links) | engine | logic | - | `engine/crates/aegis_ui/src/{timeline,specimens}.rs`; `aegis_ui/tests/kit_widgets.rs`; `engine/README.md` |
| RULESET1-U | Battle screen under timeline: strip, badges and plans, BREAK, poise bars, forecasts, chain picks, reach band, round summary (4.3 T8, 12) | game | logic | E, P, K, V | `emberwake_gui/src/{battle,intent_text,hud}.rs` |
| RULESET1-DU | Deck buckets on the party screen and in the Dojo (9 D6) | game | logic | D | `emberwake_gui/src/{party,dojo}.rs` |
| RULESET1-G | Dojo toggle and readouts, Arena select and toggles, sim `feel` mode (12, 3) | game | logic | P, K, V | `emberwake_battle/src/dojo.rs`; `emberwake_gui/src/{dojo,runner}.rs`; `emberwake_sim/src/{args,modes,feel (new)}.rs` |
| RULESET1-A | Test kit data and the `sunk_ward` field (2) | game | asset | R, S, DERIVE1-S | `data/catalog/{rulesets,derived_stats,classes,item_families,items,skills,passives,units,encounters}.json`; `data/worlds/sunk_ward/world.json` |
| RULESET1-X | Golden traces and the inert-extensions check (4.2, 4.3) | kit | logic | A, G | `emberwake_battle/tests/{timeline_golden.rs,fixtures/timeline_*.json}` |
| RULESET1-B | Feel sim report: stagger factor, poise factor, chance on/off, sure break, pierce at 25/35/50, chains, lock rate | game | balance | X | `games/emberwake/docs/BALANCE_REPORT.md` |

R and S are schema-first; T, K, V and D can follow S in the order the planner's Touches allow (T, K and P share
`plan.rs`, `skill.rs`, `command.rs`). ENCOUNTER2 also edits `schema.json` and `session.rs`: sequence it with R. If
KIT1 lands first, the kit rows move to its crates (same modules).

## 7. Docs delta (builders report, the planner applies)

- `games/emberwake/docs/GAME_BRIEF.md`: a Rulesets section (switch, register, numbers by file).
- `docs/STRUCTURE.md`: `emberwake_rules/src/ruleset.rs`, `emberwake_battle/src/poise.rs`, `emberwake_sim/src/feel.rs`,
  `data/catalog/rulesets.json`, the golden fixtures; `aegis_ui` timeline marks.
- `engine/README.md` (widget marks and parts); `engine/docs/PORT_NOTES.md` (the new modules port no Godot file).
- `engine/docs/authoring/`: a game-agnostic note "rulesets: options and extensions" (the generic method).
- Lore docs (planner): `combat_and_timeline.md` 6 (reach 2 is new, not "exists"); `rules_engine_architecture.md` 3.1
  and 4.2 (field names of 1.3; a repeated break halves stagger, per CANON).
- HISTORY rows for the parity strips of section 5; decision rows once the user answers TQ1-TQ11.

## 8. Code survey (2026-10-25; paths under `games/emberwake/crates/` unless named)

| Piece | Where | Today |
| --- | --- | --- |
| Scheduling | `emberwake_battle/src/resolver.rs:68-79` | `end = start + block_cost - 1`; refused past `total_blocks` or overlapping the unit's own live actions |
| Landing | `resolver.rs:97-99` | actions with `end == block` land, sorted by initiative (`godot_sort`) |
| Start | `resolver.rs:104-124` | AP and stance switch paid at `start` (debt allowed), stance entered, `on_start` (dash, chase) |
| Cancel | `resolver.rs:126-148` | an inactive unit's actions are set `cancelled` (the only cancel today) |
| Initiative | `command.rs:159-161` | `priority x 1000 + speed`; Defend priority 1, Target 2, others 0 (`plan.rs:204-243`) |
| Guard | `resolver.rs:84-95`, `damage.rs:202` | a running Defend raises `guard` each block; `min(GUARD_CAP 0.9, guard + cover)` |
| Stance break | `command.rs:516` | `disrupt_break` knocks Offensive/Defensive to Tactical and ends the running guard |
| Armour | `damage.rs:11,122-138` | rating = gear mitigation + STR/4 (or unit `armor`) + fortify - expose; cut `r / (r + 50)` |
| AP cost | `skill.rs:149-156` | `clamp(level, 1, 5) + ap_cost_modifier`, clamped 1..5; Move 1 per tile; Rest/Target/Status 0 |
| AP pool | `session.rs:90`, `unit.rs:10` | encounters start units at `unit_ap` (default 12); `BASE_AP` 10 only for blank units |
| Reach | `skill.rs:223-236`, `command.rs:244,260,538-549`, `world.rs:182` | `attack_range > 1` means ranged: band (2, max(3, range)), cover on, no riposte, Sniper AI; melee (1, 1); skills may set `reach_min/max` |
| Riposte | `command.rs:774-783` | melee only, distance exactly 1 |
| Cover | `command.rs:697-717` | ranged hits only: the target tile's or a blocking neighbour's `cover_guard` |
| Statuses | `emberwake_rules/src/status.rs:37-53` | fifteen rows; `stagger` blocks 1 and 4, `stun` 1 and 2 (`status.rs:185-191`); AP debt 3 / 5 |
| Tells | `intent.rs:136-146`, `unit.rs:341` | each enemy's first action to everyone; the whole plan when a `foresight` holder is targeted or the enemy is enraged |
| Loadout | `stats.rs:10-11`, `unit.rs:386-400` | 5 class + 3 extra families; basics always active, never slotted |
| Passives | `unit.rs:279-343` | ten effect types incl. `stat` and `foresight`; always on |
| Skill row | `emberwake_data/src/lib.rs:155-245` | `blocks` 1-6 (schema), `level` 1-5 (schema label "Level (= AP cost)"), reach, `disrupt`, `exploit` |
| Families | `check/item.rs:26` | closed `engine_rules`: focus, always_ranged, shield, spell_source |
| Encounter | `session.rs:44-66` | `EncounterDef` from the row; no ruleset; `Session::start` has 17 call sites (sim, GUI, host, tests) |
| World manifest | `emberwake_world/src/data.rs:166` | lineage fields like `party_max` (-1 = the parent's) |
| Dojo | `emberwake_battle/src/dojo.rs:1-27` | a gear sandbox without the resolver: actions one at a time, stance shift (`:631`), dummy guard (`:675`) |
| Arena | `emberwake_gui/src/runner.rs:124-127` | the Scene Runner's Combat Arena plays any catalog or world encounter |
| Sim | `emberwake_battle/src/sim.rs:20-31`, `emberwake_sim/src/args.rs:36-55` | `FightSpec`; `key=value` args; args outside the Godot help text exist (`rewards`) |
| Timeline widget | `engine/crates/aegis_ui/src/timeline.rs:20-58` | tracks of segments (fill, dashed, tip); no marks, no segment parts |
