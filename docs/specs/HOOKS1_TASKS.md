# HOOKS1 tasks: schema, data, tests, parity, task split, CAST1 rows, questions

Status: proposed (2026-10-25); companion of [HOOKS1.md](HOOKS1.md) (rule ids and sections cited). Paths after KIT1:
`kb/`, `kr/`, `kc/`, `kw/` = `kits/rpg/crates/rpgkit_{battle,rules,content,world}/src/`, `kd` =
`kits/rpg/crates/rpgkit_data/src/lib.rs`, `schema` = `kits/rpg/schema.json`, `gui/` =
`games/emberwake/crates/emberwake_gui/src/`, `ds` = `games/emberwake/crates/emberwake_world/tests/data_source.rs`.

## 1. Schema contract (schema-first: yes, HOOKS1-S; every row joins `AFTER_GODOT` in `ds`)

### 1.1 Skills (`content.skills`)

| Key | Type | Default | Scope | Validation |
| --- | --- | --- | --- | --- |
| `forced_move` | json | {} | family | ATTACK; `kind` push/pull, `tiles` 1-3, `from` source/center (needs an area), `when` always/moved/windup, `stop_moves`, `collide_break` bool, `collide_mult` 0.0-3.0, `collide_stagger` 0-50 |
| `hazard` | json | {} | family | ATTACK, DEFEND, SUPPORT; `id` a `hazards` row, `shape` SKILL (needs an area)/NONE/PLUS/BURST/LINE/SWEEP, `at` target/self, `rounds` 0-5 |
| `triggers` | json | [] | family | DEFEND, ATTACK, SUPPORT; 0-3 rows (1.3), `on` one of `hit_taken`, `guarded_hit`, `ally_hit`, `evaded`, `broken` |
| `riders` | json | [] | family | 0-3 `{when, effects}` (1.3 keys; no `redirect`); ATTACK, SUPPORT |
| `meters` | json | [] | rank | 0-2 `{meter, delta}`; `meter` a `unit_meters` row; `delta` -100..100, not 0 |
| `hp_cost` | int | 0 | rank | 0-99; not MOVE, REST, TARGET, STATUS |
| `ignore_guard` | bool | false | family | ATTACK |
| `exploit` | json | [] | family | + entries `{hp_below 1-99, power_mult}` (range as a status entry's; no `status` key); timeline only, as `flanked` |

Warnings: a placement (ATTACK, `power_mult` 0, a `hazard`) with a status, stagger or `forced_move`; `hp_cost` above
half the lowest owner's max HP.

### 1.2 Items, passives, units, encounters, world, rulesets

| Where | Key | Type, default | Validation |
| --- | --- | --- | --- |
| items (consumables) | `hazard` | json {} | as 1.1 (`shape` SKILL not allowed) |
| item `effects` types | `trigger`, `forced_move` | - | as the passive types below (`check/simple.rs:71`) |
| passive effect | `forced_move` | `{type, immune, bonus_tiles}` | `immune` ""/always/guarding, `bonus_tiles` 0-2; at least one set |
| passive effect | `trigger` | `{type, on, when, effects, limit}` | 1.3 |
| passive effect | `meter` | `{type, meter, start, floor}` | `meter` resolves; `start` -1 (the row's) or in range; `floor` in min..max |
| units | `target_pref` | + `SOFTEST`, `TOUGHEST` | enum |
| units; phase `ai` | `tactics` | strings [] | subset of `brace_after_hit`, `hold_line`, `flush_cover`; no repeats |
| encounters | `arena_hazards` | json [] | 0-4 rows (1.6) |
| encounter phase effects | `deck` | `{type, unit, add, order}` | DU2; with `deck` in the world's ruleset |
| world manifest | `classes` | json {} | class id keys; `name` 1-32, `description` 0-240, `skill_path` absent or `{family: level 1-30}` |
| rulesets | `rules` | + 9 ids (with `execute`) | HOOKS1 4.1 |
| later (HOOKS1-Q) | `spare_once`; trigger `spared`; effect `heal` | refs []; `{kind, unit}`; `{type, unit, pct 1-100}` | refs in `enemies` or phase waves |

### 1.3 Trigger rows (passives, items, skills, riders)

| Key | Type, default | Validation |
| --- | --- | --- |
| `on` | enum | the events of HOOKS1 8.1 (not on riders) |
| `when` | object {} | 8.2 keys: `pct` 1-99 (`hp_below` only, required), `range` 1-6, `ranged`, `melee`, `other_broken` bool, `skills` resolve, `other_hp_below` 1-99, `other_status`, `intent`, `meter` `{id, at_least}` |
| `effects` | list, 1-4 | 8.2 types and ranges; `to` self/other; one `redirect`, only with `ally_hit`; `grant` buffs, `inflict` debuffs |
| `limit` | enum round | round, battle, none |

### 1.4 Kind `hazards` (catalog, new; `origin` as every kind)

| Key | Type, default | Validation |
| --- | --- | --- |
| `id`, `name`, `description` | id; "New Hazard"; text "" | name 1-24 |
| `on` | enum both | enter, round_end, both |
| `status`, `potency`, `magnitude`, `status_rounds` | "", 100, 0, 1 | a status row; 0-100; the status's range; 1-5 |
| `damage` | int 0 | 0-50 (flat, through guard) |
| `cover_guard`, `move_cost` | 0.0, 0 | 0.0-0.5; 0-3 (added) |
| `affects` | enum all | all, foes, friends (of the placer) |
| `trap`, `hidden`, `stop_moves` | false | `hidden` needs `affects` foes |
| `duration` | int 2 | 1-5 |
| `tint`, `glyph` | "#808080", "*" | `#rrggbb`; one character |

### 1.5 Kind `unit_meters` (catalog, new)

| Key | Type, default | Validation |
| --- | --- | --- |
| `id`, `name`, `short`, `description` | id; "New Meter"; ""; "" | name 1-24, short 1-8 |
| `min`, `max`, `start` | 0, 10, 0 | min < max <= 200; start in range |
| `show` | enum pips | pips (max at most 10), bar, hidden |
| `decay` | json [] | 0-3 `{delta, unless_gained, if_guarded}`; delta -50..50 |
| `bands` | json [] | 0-8 `{id, name, from, effects, statuses}`; first `from` = min, ascending; `effects` UM4 types; `statuses` buffs `[{status, magnitude}]` |
| `rules` | json [] | 0-6 `{id, at, dir, once, effects}`; `at` a band id or an int in range; effects as 1.3 plus `meter_set` `{meter, value}` |
| `skills` | refs [] | skills with a negative `meters` delta on this meter |

### 1.6 Kinds `zones` (catalog, new) and the `arena_hazards` row

| Where | Key | Type, default | Validation |
| --- | --- | --- | --- |
| zones | `id`, `name`, `description` | id; "New Zone"; "" | not one of the 8 code zone ids (ZN2) |
| zones | `rows` | json [] | empty (9 floor rows) or 9 strings of 12 glyphs (ENCOUNTER2 R0) |
| zones | `backdrop`, `ground` | "", "floor" | an asset key or "" (`backgrounds/battle/<id>.png`); a terrain id |
| arena hazard | `id`, `name`, `line` | id; 1-24; 0-160 | ids unique in the encounter |
| arena hazard | `cells` / `columns` / `rows` | [[x, y]] / ints / ints | exactly one set; on the board |
| arena hazard | `rounds`, `blocks` | [] (every round), [] | rounds 1-30; blocks 1-6, at least one |
| arena hazard | `effect` | json | at least one of `damage` 0-99, `damage_pct` 0-50, `stagger` 0-50, a status (`status`, `potency`, `magnitude`, `status_rounds`), `push` `{dir, tiles 1-3, collide_damage 0-50}` |

## 2. Data (HOOKS1-A; `tools/fc.sh content upsert <kind> @file.json`, then `content check`) FP

### 2.1 `hazards` rows

| Id | Name | `on` | Status (pot., mag., rounds) | `damage` | cover | move | `affects` | trap, hidden, stop | `duration` | tint, glyph | Used by |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `smoke` | Soot Smoke | both | none | 0 | 0.3 | 0 | all | no | 2 | #6b6b73, `s` | `ash_cloud_fade` |
| `tripwire` | Tripwire | enter | root (90, -, 1) | 15 | 0 | 0 | foes | yes, yes, yes | 3 | #c8b27a, `t` | `tripwire_snare` |
| `grease` | Scalding Grease | both | slow (80, -, 1) | 0 | 0 | 1 | all | no | 2 | #b38f2e, `g` | `scalding_grease_pot` |
| `phosphorus_fire` | Phosphorus Fire | round_end | burn (80, 5, 2) | 8 | 0 | 0 | all | no | 2 | #e8742a, `^` | `pyre_nozzle_sweep` |
| `pitch_fire` (M2b) | Pitch Fire | round_end | burn (70, 4, 2) | 6 | 0 | 0 | all | no | 2 | #d9531e, `^` | `phosphorus_pot` |
| `choke_gas` (M2b) | Choke-Gas | both | poison (80, 6, 3) | 0 | 0 | 0 | all | no | 2 | #7fa34a, `~` | `choke_gas` |

### 2.2 `unit_meters` rows

| Id | Name, short | min, max, start | `show` | `bands` | `skills` | Used by |
| --- | --- | --- | --- | --- | --- | --- |
| `bile_phial` | Bile Phial, Phial | 0, 3, 0 | pips | - | [`bile_coat`] | Maren's trait |
| `slag_fury` | Slag Fury, Fury | 0, 5, 0 | pips | `f0` 0 none; `f1` 1 to `f5` 5: `statuses` empower 5, 10, 15, 20, 25 | - | Dunstan's trait |

### 2.3 `zones` rows (no backdrop files in the slice: STAGE1's gradient; art in M4, asked first)

| Id | Name | `ground` | Used by (CAST1_ENCOUNTERS) |
| --- | --- | --- | --- |
| `alley` | Cinder Alley | stone | `enc_pro_0a`, `enc_job_corvo_gate`, `enc_evt_01`, `enc_evt_02` |
| `rooftops` | Market Rooftops | stone | `enc_pro_0b` |
| `canal` | Canal Bridge | bridge | `enc_pro_0c` |
| `foundry` | The Fourth Kiln | floor | `enc_evt_03` |
| `tallow_vats` | Tallow Vats | floor | `enc_evt_04` |
| `undercroft` | Sluice Undercroft | stone | `enc_evt_05` |
| `watchpost` | Crucible Watchpost | floor | `enc_evt_06` |

### 2.4 `sunk_ward` world manifest and the timeline ruleset row

- `classes` (HQ5 a): `warrior` Vanguard, `rogue` Gutter-Stalker, `priest` Heretic Acolyte, `ranger` Arbalestier,
  `brawler` Slag-Brute, `mage` Ichor Alchemist; each `"skill_path": {}`; descriptions from `lore/systems/class_*.md`.
- `rulesets.timeline.rules`: append `forced_move`, `ground_hazards`, `arena_hazards`, `triggers`, `unit_meters`,
  `hp_cost`, `ignore_guard`, `ai_tactics`, `execute`.

## 3. Tests (rule-derived; scratch data in the kit crates' `tests/`)

| Task | Test | Expectation from |
| --- | --- | --- |
| every logic task | classic parity; the `tl_` goldens with all nine rules on | unchanged (4.2) |
| S | each 1.1-1.6 violation; `legacy/data`; a classic world's `arena_hazards` | one error each; no new error; a warning |
| F | push 2 east of the source; a diagonal tie facing north; pull from 3 tiles | 2 tiles east; north axis; stops at distance 1 (FM2) |
| F | push 2 with a wall, a unit, an exit after 1 tile | collision hit and stagger; both units hit; stops before the exit (FM3, FM4) |
| F | `collide_break` into a windup / an unbreakable windup | breaks / does not (FM4) |
| F | `immune` always; `guarding` with / without a Defend; source `bonus_tiles` 1 | no move; no move / moves; 3 tiles (FM5) |
| F | across grease; `when` moved, walker / stander; `stop_moves`; a stealthed unit pushed | slow rolls; moves / not; Walk cancelled; still found (FM6, FM7) |
| G | each shape at target and self; a placement ATTACK | the cells, walkable only; no damage, no RNG draw (GH2, GH3) |
| G | a hazard on a hazard tile over terrain hazard; smoke, a ranged and a melee hit | replaced, terrain first; cover 0.3 ranged only (GH4, GH5) |
| G | `rounds` 2 placed in round 3 | there through round 4, gone in round 5 (GH7) |
| G | a tripwire met by the placer's ally, then a foe mid-path; enemy paths past a hidden trap, visible grease | ally passes; foe stops, 15 and a root roll, trap gone; no penalty / penalty (GH6, GH8) |
| E | each event; a trigger twice in a round, `limit` round / none; an effect that would raise an event | one call each; once / twice; no second call (8.1) |
| E | redirect 35; redirect 100; holder at 5 HP | holder 35% of its own result, ally 65%; the whole hit on the holder; holder stops at 1 (8.4) |
| E | one holder: passive redirect 35 and a running Guard's redirect 100; the Guard not running | the Guard takes the whole hit, the passive nothing; the passive redirects 35 (8.4, CQ18 a) |
| E | `guard` 0.5 blocks 0 in block 3, a hit in block 5 | guard 0.5, stagger x0.5, `guarded_hit` fires; gone next round (8.3) |
| E | `ap` 2 at 9 of 10 / on an enemy; `grant` until `guard_end`; `reveal`; `hp_below` 50 twice, `limit` battle | 10 / nothing; ends with the Defend; next round revealed; once (8.2) |
| K | `hp_cost` 10 at HP 25 / 10 / 8; `ignore_guard` on a guarding unit in cover | HP 15 / refused / refused; guard term 0, cover kept, stagger factor 1.0 (HC1, IG1) |
| K | rider `end_guard` on a guarding target; `when` `other_broken` | guard 0 at once; only on a target broken this round (RD1) |
| X | exploit [`stagger` x1.5, `hp_below` 35 x2.0]: a foe at 36% / 35% / 35% staggered; rule `execute` off | x1.0 / x2.0 / x2.0; x1.0, no new RNG draw (EX1) |
| M | `start` 1, `floor` 2; a skill delta -1 at 0 / 1 | starts at 2; refused (fails unpaid if forced) / value 0 (UM1, UM2) |
| M | decay `-10 unless_gained` after a gaining / a quiet round; a band with empower 10 entered and left | unchanged / -10; empower held in the band only (UM3, UM4) |
| M | rule `at` 3 up `once` over 2, 4, 2, 4; a meter skill, cost met / not | once; offered slotless / not (UM5, UM6) |
| H | vents on x 7-8 at b3, b6, a unit on (7, 4) in windup; `push` east 2 into a wall; a reserved unit there | struck at b3's end before the verdict, stagger x2, no armour; collision; untouched (AH2) |
| H | tells at round start | `arena` lists the cells and blocks; PL4 avoids them (AH3, AT3) |
| U | phase `deck` at block 4 of round 2; `add` of an unowned family; actives over `deck_slots` | round 3 uses the new order; one error each (DU1, DU2) |
| AI | `SOFTEST` vs armour 25 and 6; `TOUGHEST` vs a melee 230 HP and a ranged 250 HP | the armour-6 unit; the melee unit (AT1) |
| AI | `brace_after_hit`, friend hit last round / not; `hold_line`, a band tile next to a friend; `ai_tactics` off | Guard / the table; that tile; ROLE, no hazard threat (AT2, 4.1) |
| C | A `warrior` Vanguard, B extends A, C none; `skill_path` {} / absent | Vanguard, Vanguard, Warrior; own skills / class path (CN1-CN3) |
| Z | a data zone with / without encounter `rows`; a row named `stone_arena` | encounter rows / zone rows; one error (ZN2, ZN3) |
| UI | 1280 x 720, four text sizes, hazards, telegraph, push forecast, pips | no text-audit fault; classic unchanged |
| A | `tools/fc.sh content check` | clean |

## 4. Parity plan

- No effect: `trace_parity`, `world_traces`, `journey`, `phasefold`, `saves` (no new save key), `checks_parity`,
  `sim_parity`, host `replay`, RULESET1-X's `timeline_golden` (its frozen kit sets no HOOKS1 field).
- `ds`: S (and Q) strip the new keys and the three kinds through `AFTER_GODOT` (as DERIVE1-S), the `target_pref`
  options like `HELP_CHANGED`; one HISTORY row names them.

## 5. Tasks (status `spec`; spec `docs/specs/HOOKS1.md` (+ `_TASKS.md`))

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| HOOKS1-S | Schema, typed fields, checks for every field and kind (1.1-1.6), the 9 rule ids, parity strips | kit | logic | KIT1-N, RULESET1-S, ENCOUNTER2-K | `schema`; `kd`; `kc/{source,store}.rs`; `kc/check/{skill,simple,item,unit,encounter,world,mod}.rs`; `kr/ruleset.rs`; `ds` |
| HOOKS1-F | Forced movement: push, pull, collisions, immunity, bonus tiles, `stop_moves` (5) | kit | logic | HOOKS1-S, RULESET1-P | `kb/{forced (new),command,world,log,metrics}.rs`; `kr/unit.rs`; `rpgkit_battle/tests/forced_move.rs` |
| HOOKS1-G | Ground hazards: kind, overlay, placement, traps, cover, expiry, AI penalty (6) | kit | logic | HOOKS1-F | `kb/{ground (new),grid,command,resolver,plan,ai,skill,log,metrics}.rs`; `tests/ground_hazards.rs` |
| HOOKS1-Z | Zones as data: kind, `zone` options, board, backdrop key (13) | kit + game | logic | HOOKS1-G (grid), ENCOUNTER2-T | `kd`; `kc/source.rs`; `kb/{grid,session}.rs`; `gui/battle.rs` (backdrop key); `tests/zones.rs` |
| HOOKS1-E | Events, triggers, held guard, redirect and intercept (8.1-8.4) | kit | logic | HOOKS1-Z, RULESET1-V | `kb/{trigger (new),command,resolver,session,poise,log}.rs`; `kr/{unit,damage}.rs`; `tests/triggers.rs` |
| HOOKS1-K | Riders, `hp_cost`, `ignore_guard` (8.5) | kit | logic | HOOKS1-E | `kb/{trigger,command,resolver,round_intent_plan,ai}.rs`; `kr/damage.rs`; `tests/riders.rs` |
| HOOKS1-X | Execute riders: exploit `hp_below` in the match and the breakdown (8.6) | kit | logic | HOOKS1-K (same files) | `kb/{skill,command}.rs`; `kr/damage.rs`; `tests/execute.rs` |
| HOOKS1-M | Unit meters, bands helper, meter skills (9) | kit | logic | HOOKS1-X | `kb/{meter (new),trigger,session,skill,round_intent_plan}.rs`; `kr/{unit,bands (new)}.rs`; `tests/meters.rs` |
| HOOKS1-H | Scheduled arena hazards and their tells (7) | kit | logic | HOOKS1-M | `kb/{arena (new),session,intent,log,metrics}.rs`; `tests/arena_hazards.rs` |
| HOOKS1-U | Phase effect `deck` (10.1) | kit | logic | HOOKS1-H, RULESET1-D | `kb/{encounter,session}.rs`; `tests/deck_unlock.rs` |
| HOOKS1-AI | Target prefs, tactics, hazard-aware tiles (11) | kit | logic | HOOKS1-U, RULESET1-AI | `kb/{ai,round_intent_plan,world,encounter}.rs`; `tests/tactics.rs` |
| HOOKS1-C | Class names and paths per world (12) | kit + game | logic | HOOKS1-S | `kw/{data,battle,progression}.rs`; `gui/{hub,party}.rs`; `rpgkit_world/tests/class_overlay.rs` |
| HOOKS1-UI | Battle screen: hazard tiles, arena telegraph, push forecasts, meter pips, held guard, HP cost, inspect class | game | logic | HOOKS1-H, HOOKS1-C, RULESET1-U | `gui/{battle,intent_text,hud}.rs` |
| HOOKS1-A | Kit data: `hazards`, `unit_meters`, `zones`, `sunk_ward` `classes`, the timeline row's rules (2) | game (data) | asset | HOOKS1-S, SUNKWARD1 | `data/catalog/{hazards,unit_meters,zones,rulesets}.json`; `data/worlds/sunk_ward/world.json` |
| CAST1-R | CAST1 parts rewritten to the hooks (section 6), stand-in notes removed | game | design | HOOKS1 (spec) | `games/emberwake/docs/specs/CAST1*.md` |
| HOOKS1-Q | Cheat-death `spare_once`, trigger `spared`, effect `heal` (10.2; M2b) | kit | logic | HOOKS1-U, ENCOUNTER2-H | `schema`; `kb/{encounter,session}.rs`; `kc/check/encounter.rs`; `ds` |
| HOOKS1-D | Docs delta (7) | kit + game | planner | each task as it lands | the docs in 7 |

Order: S; then F, G, Z, E, K, X, M, H, U, AI in a row (shared `command.rs`, `resolver.rs`, `session.rs`, `grid.rs`;
one run each); C beside F; UI after H and C; A after S; Q after the slice. If S runs long, split 1.4-1.6 into S2.
The planner adds HOOKS1-S, HOOKS1-A and CAST1-R to the Depends of CAST1-A2, -A3, -A4, -A6, HOOKS1-Z to CAST1-A6, and
HOOKS1-AI and HOOKS1-UI to CAST1-B.

## 6. CAST1 rows that change (CAST1-R writes them; FP numbers)

| Part, row (stand-ins: CAST1) | Slice | With HOOKS1 |
| --- | --- | --- |
| Founders 2 `intercepting_hook` | yes | (HQ1 a) DEFEND guard, 2 blocks, AP 3, mitigation 0.3; trigger `ally_hit` range 1 melee: redirect 100, `move` pull 1 `to` other |
| Founders 2 `braced_poise` | yes | + `forced_move` immune guarding |
| Founders 2 `kinetic_leverage` | yes | trigger `guarded_hit`: `ap` 1 |
| Founders 2 `unbroken_phalanx` | yes | trigger `guard_start`: grant steadfast until `guard_end` (crit immunity: G11) |
| Founders 2 `castellans_penance` | yes | trigger `ally_hit` range 1: `guard` 0.5 for 1 block, then redirect 35 |
| Founders 3 `ash_cloud_fade` | yes | + `hazard` smoke BURST self 2 (no jump, Q-54) |
| Founders 3 `cinder_childs_reflex` | yes | trigger `evaded`: grant haste 1 round; (HQ16 a) + trigger `kill` `skills` [throat_carver]: `ap` 2 |
| Founders 3 `throat_carver` | yes | (HQ15 a) + exploit `hp_below` 35 x2.0 (EX1) |
| Founders 4 `tourniquet_tonic` | yes | + rider `other_broken`: `guard` 0.5, blocks 0 |
| Founders 4 `triage_efficiency` | yes | trigger `heal_given` `other_hp_below` 30: `ap` 1 |
| Founders 4 `copyist_of_st_lyras` | yes | effect `meter` bile_phial; trigger `cleanse_given`: `meter` +1, limit none |
| Founders 4 new `bile_coat` | yes | (HQ4 a) SUPPORT, 1 block, `ap_cost` 0, ALLY, `meters` [bile_phial -1], grant empower 15, 1 round |
| Founders 5 `tripwire_snare` | yes | (HQ12 a) placement: power 0, reach 1-3, `hazard` tripwire NONE target 3; Assault, tile target |
| Founders 5 `heavy_winch_sunder` | yes | + `forced_move` push 2, `collide_mult` 0.5, `collide_stagger` 6 |
| Founders 5 `dead_eye_discipline` | yes | trigger `kill` ranged: `ap` 2, `reveal` 1 (HQ7 a) |
| Founders 6 `shackle_pummel_slam` | yes | (HQ14 a) push 1, `collide_mult` 0.95, `collide_break`; no `sure_break`, AP 2 |
| Founders 6 `slag_slam_throw` | yes | single target, push 2, `collide_mult` 1.2, `collide_stagger` 12 |
| Founders 6 `hypertrophy` | yes | + `forced_move` `bonus_tiles` 1 |
| Founders 6 `unbroken_iron` | yes | trigger `hp_below` 30, limit battle: `ap` 2, grant fortify 8 for 2 rounds |
| Founders 6 `foundry_wired_spite` | yes | effect `meter` slag_fury; triggers `hp_below` 50 (battle): steadfast -1, resist stun 100; `hit_taken` (none): `meter` +1 |
| Founders 7 Elyse rows | M2b | `phosphorus_pot`, `choke_gas`: hazards pitch_fire, choke_gas SKILL 2; `catalytic_munitions`: trigger `hit_dealt` `other_status` burn, inflict expose 4 |
| Founders 1, titles | yes | class names from `classes` (2.4); `title` "" or an epithet ("Slag-Jaw") |
| Bestiary 3 `billhook_hook` | yes | `forced_move` pull 1, `when` moved, `stop_moves` |
| Bestiary 3 `iron_shackle_drag` | yes | + pull 2 |
| Bestiary 3 new `buckler_jab` | yes | Crossbowman: ATTACK assault, 1 block, rec 1, power 0.4, stagger 4, reach 1-1, push 1; in its deck, order assault [aimed_bolt, buckler_jab, strike] |
| Bestiary 3 `martyrs_rush` | yes | `hp_cost` 10 |
| Bestiary 3 `brand_of_the_heretic` | yes | rider `end_guard` |
| Bestiary 3 `pyre_nozzle_sweep`, `scalding_grease_pot` | yes | + `hazard` phosphorus_fire / grease SKILL 2 (Flammable: G17) |
| Bestiary 5 new `fanatical_agony` | yes | Flagellant trait: trigger `hp_below` 50 (battle): grant haste -1, `crit` 20 |
| Bestiary 2 AI knobs | yes | Footman `tactics` [brace_after_hit, hold_line]; Flagellant SOFTEST; Bailiff TOUGHEST; Purifier [flush_cover] |
| Bestiary 5 `unyielding_authority` | yes | + `forced_move` immune always |
| Bosses 3.3 Grendel | yes | deck active [hammer, jab]; phase `pulverizer` + `deck` add [pulverizer], order assault [pulverizer, hammer, jab]; phase `vents` removed |
| Bosses 2 `sanctified_decapitation`; Nemeth | yes | + `ignore_guard`; Nemeth TOUGHEST |
| Bosses 2 `pneumatic_hammer_strike` | yes | unchanged: the knock-down is the Staggered status (Q-56) |
| Bosses M2b | M2b | capstones by `deck`; Vane's Gilded Sunder push 1; Quell by HOOKS1-Q; Orlov's Cataclysm pull 1 |
| Gear 6 `vanes_gilded_pavise` | M2b | item trigger `guarded_hit`: `damage` 15, `stagger` 20 `to` other |
| Encounters 2-3 every row | yes | `zone` per 2.3 |
| Encounters 2 `enc_evt_03` | yes | `arena_hazards` `slag_vents`: columns [7, 8], blocks [3, 6], `damage_pct` 12, `stagger` 10, burn 60/3/1, line "The slag vents roar." |
| Encounters 2 `enc_pro_0b` | yes | (HQ1 a) text "Follow the smoke: hide, guard each other, slip away." |
| Gaps 1 G1, G3-G5, G7, G14, G19, G22, G24, G27-G30 | - | point to HOOKS1 (G7: 8.6); G18 to HOOKS1-Q (M2b) |

## 7. Docs delta (builders report, HOOKS1-D applies), ROADMAP and HISTORY rows

- `GAME_BRIEF.md`: a "Combat hooks (HOOKS1)" note. `docs/STRUCTURE.md`: `kb/{forced,ground,trigger,meter,arena}.rs`,
  `kr/bands.rs`, `data/catalog/{hazards,unit_meters,zones}.json`, the kit tests. `kits/rpg/README.md`: the hooks.
- RULESET1 3.2 (the 9 rule ids); RULESET1_INTENT 2 (a placement is a tile-target Assault opener), 7 PL4 (hazard
  threats, `hold_line`), 9 (the `brace_after_hit` row); ENCOUNTER2 6 (`arena_hazards`, `deck`, later `spare_once`);
  CAMPAIGN1 M5 (the shared crossing helper); CAST1 and CAST1_GAPS (section 6, CAST1-R).
- `engine/docs/authoring/combat_hooks.md` (new, game-agnostic): forced moves, hazards, triggers and meters as data.
- Decision S12 (`decisions/combat.md`): the static-terrain and code-zone clauses superseded (planner).

ROADMAP row (`proposed`): HOOKS1, Combat hooks for the slice's cast, depends on KIT1, RULESET1, ENCOUNTER2; spec
`docs/specs/HOOKS1.md`; HQ1-HQ16 open.

HISTORY rows (append):

| Date | Concept | Status | Replaced by | Why |
| --- | --- | --- | --- | --- |
| 2026-10-25 | Static terrain only; zone maps in code (decision S12) | replaced | HOOKS1 ground hazards and the `zones` kind (the 8 code zones stay) | "Slice is data only"; parity: `data_source` strips the new keys and kinds |
| 2026-10-25 | CAST1's stand-ins for G1, G3-G5, G14, G19, G22, G24, G27-G29 | replaced | the HOOKS1 hooks (CAST1-R) | Q-55, Q-56 |
| 2026-10-25 | The Crossbowman's lore AI "targets one in a windup" | not adopted (HQ2 a) | `target_pref` WEAKEST | enemies plan first; no re-planning AI (RULESET1 13) |

## 8. Open questions (the planner takes the recommendation while the user is away)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| HQ1 | Intercepting Hook's intent | (a) Guard: takes a melee hit meant for an adjacent ally, pulls the attacker 1 (lore; 0b teaches it); (b) Maneuver Engage as CAST1, plus a pull | a: the lore effect, Kaelen's identity from 0b |
| HQ2 | Targeting a unit in windup (Crossbowman lore) | (a) not built (enemies plan first); (b) a re-planning AI reading the party's plan | a: the tells contract holds (RULESET1 13) |
| HQ3 | Where per-unit meters live | (a) kind `unit_meters` (battle state), CAMPAIGN1's crossing helper shared; (b) CAMPAIGN1 `meters` with a `battle` scope | a: two lifecycles, one helper |
| HQ4 | Using a Bile Phial | (a) meter skill Bile Coat, slotless while a phial is held (0 AP Snap, empower 15, 1 round); (b) spent by her next Support on an ally | a: the lore's free Snap |
| HQ5 | Class paths in the Sunk Ward | (a) `skill_path` {} for the six; (b) keep the base paths (Ashlands skills ownable) | a: no Ashlands skill leaks in |
| HQ6 | Proficiencies in the class overlay | (a) none (CQ1 a drills stay); (b) overlay proficiencies | a: the drills work |
| HQ7 | Dead-Eye's reveal of the next round | (a) the `reveal` effect on RULESET1 V2; (b) drop it (Hawkeye's foresight) | a: small, the lore effect |
| HQ8 | M3 conveyor belts | (a) vents only, belts later as data; (b) belts now | a: one hazard lesson at a time |
| HQ9 | A push into lava or deep water | (a) a collision; (b) a fall (lethal or heavy) | a: no instant kills in Act 1 |
| HQ10 | Hazard damage vs armour | (a) through guard, not armour (as burn); (b) through armour | a: armoured units feel it |
| HQ11 | A unit that blocks a collision | (a) both take the hit (lore); (b) only the moved unit | a |
| HQ12 | Tripwire Snare's intent | (a) Assault, a tile-target placement; (b) Maneuver Utility (amends MV10) | a: no RULESET1 change |
| HQ13 | Quell's cheat-death (G18) | (a) HOOKS1-Q after the slice (M2 is M2b); (b) before it | a |
| HQ14 | Shackle Slam's break | (a) the wall-slam break replaces `sure_break`, AP 2 (lore); (b) keep both | a: rewards positioning |
| HQ15 | Throat-Carver's execute (G7) | (a) x2.0 power under 35% HP (EX1), the lethal form later (EX3); (b) a lethal rider now | a: HQ9, no instant kills in Act 1 |
| HQ16 | Where its 2 AP back lives | (a) a `kill` trigger on Vesper's trait (no new code); (b) a rider key `other_downed` on the skill | a: EX2 covers it |
