# HOOKS1: combat and content hooks for the slice's cast

Status: proposed (spec 2026-10-25, from "Slice is data only" (Q-55, Q-56), "Class names per world" (Q-53), Q-54,
Q-61, Q-63, Q-74 in `games/emberwake/docs/decisions/lore.md`; the gaps are CAST1_GAPS section 1)
Depends on: KIT1-N (kit crates); RULESET1-S, -IN, -P, -PL, -AI; ENCOUNTER2-T, -H, -K
Lane: logic (schema first, then one hook per run), asset (kit data), design (the CAST1 rewrite)
Touches and companion: [HOOKS1_TASKS.md](HOOKS1_TASKS.md) (schema, data, tests, parity, tasks with Touches, the CAST1
rows that change, docs delta, open questions HQ1-HQ16)

History check (`docs/HISTORY.md`; decision S12, `games/emberwake/docs/decisions/combat.md`): S12 kept terrain static
("skill-left terrain later") and zone maps in code ("until S9 map data"); "Slice is data only" is the new reason. No
cut roll returns: nothing here draws a random number but the existing status potency roll. Act-then-move stays the
first-in-round jump (Q-54), so Ash-Cloud's jump is not built; a re-planning AI stays out (RULESET1 13, HQ2).

## 1. Goal

Every effect the slice's cast (CAST1: 0a-0c, the dilemma job, M3, M5) needs and no spec provides becomes a reusable
kit hook, off unless data names it, so the slice ships lore effects, not stand-ins. Battle hooks extend the timeline
rule register (RULESET1 3.2): classic never runs them, the Arena and the sim can switch them off. No engine change.

## 2. The gaps and where they land (hooks `[kit hook]`, their rows `[data]`; CAST1 rows: task file 6)

| Gap | Slice need | Hook | Sect. | Task |
| --- | --- | --- | --- | --- |
| G1 | the founders' class names | world `classes` overlay: name and path per world | 12 | C |
| G3 | Winch Sunder, both Slams, Billhook Hook, Shackle Drag, Buckler Jab; Braced Poise, Hypertrophy, Unyielding Authority | forced movement, collisions, immunity | 5 | F |
| G4 | Ash-Cloud Fade, Tripwire Snare, Grease Pot, Nozzle Sweep | ground hazards a skill or item leaves | 6 | G |
| G5 | Penance, Leverage, Phalanx, Iron, Spite, Triage, Dead-Eye, Tourniquet, Reflex, Agony, Intercepting Hook | events, triggers, riders | 8 | E, K |
| G14 | the Bile Phial; Slag Fury | per-unit battle meters (Strain's shape) | 9 | M |
| G19 | Grendel's Pulverizer | phase effect `deck` | 10 | U |
| G22 | the backdrops | zones as a catalog kind (layout: ENCOUNTER2-T `rows`) | 13 | Z |
| G24 | M3's slag vents | scheduled arena hazards | 7 | H |
| G27, G28 | Martyr's Rush; Decapitation, Heretic's Brand | `hp_cost`; `ignore_guard`, rider `end_guard` | 8.5 | K |
| G7 | Throat-Carver's execute and AP back | exploit `hp_below`; `kill` trigger | 8.6 | X |
| G29 | Footman, Flagellant, Bailiff, Purifier, Nemeth | AI tactics, target preferences | 11 | AI |
| G18 | Quell's valve (M2 = M2b) | once-only `unkillable`, trigger `spared`, effect `heal` | 10.2 | Q |
| G13, G21, G23, G30 | join level; the Amalgam; the Crucible's four; `loud`, `quiet` tags | covered: ROSTER1 BK5; BIGUNIT1 (later); ROSTER1 BK6; CAMPAIGN1 M3 battle inputs (CAMPAIGN1_TASKS 2) | - | - |

## 3. Survey (pre-KIT1 lines; `b/` `r/` = `kits/rpg/crates/rpgkit_{battle,rules}/src/`, `gui/` = `emberwake_gui/src/`)

| Piece | Where | Today |
| --- | --- | --- |
| Statuses, skills | `r/status.rs:37-53`, `:199-244`; `rpgkit_data/src/lib.rs:155-251`; `b/skill.rs:61-80`, `:239-260` | 15 code rows (debuffs roll potency, act next round); one status, grants, cleanses, `dash_tiles`, `jump_tiles`, `exploit`; areas NONE, PLUS, BURST, LINE, SWEEP; nothing moves another unit, leaves a hazard, costs HP or rides a hit |
| Terrain, hazards, cover, moves | `b/grid.rs:61-127`; `b/command.rs:697-771`; `b/resolver.rs:183-218`; `b/world.rs:262-287` | 17 static code types with catalog move cost, cover, one hazard status; rolls on enter (walk `:667`, dash `:609`, jump `:498`) and at round end; cover on ranged hits only; `move_unit` (an exit withdraws), `jump_unit`: only the mover moves |
| Zones | `b/grid.rs:130-173`; encounter `zone` (`options_from: zones`) | 8 code zones; STAGE1 keys the backdrop by zone id |
| Damage, guard | `r/damage.rs:152-221`; `b/resolver.rs:83-96`, `:153-172` | `include_guard` (`:161`) always on; guard + cover capped 0.9 (`:202`); guard reset each block, raised by a running Defend |
| Reactions, passives | `b/command.rs:19`, `:774-791`; `r/unit.rs:279-345`; `rpgkit_content/src/check/simple.rs:70-71` | riposte only; 10 static effect types (items 4), no condition, event or counter |
| Phases, hook points | `b/encounter.rs:219-279`, `:390-422`; `b/session.rs:105-112`, `:652-709`, `:903-1053` | triggers `hp_below`, `round`, `arrival`; effects line, wave, status, ai, enrage; `Roster::customize`; `step_next`, `on_block_start`, `on_action_resolved`, `on_block_end` |
| AI | `b/world.rs:24-42`; `b/ai.rs:595-619`; `b/plan.rs:312` | roles, NEAREST or WEAKEST, reckless, flee, defend; a hazard path penalty |
| Class names | `gui/hub.rs:18`, `gui/party.rs:1143`, `gui/dojo.rs:261` | the class row's `name`; no class key in the world manifest |

## 4. Shared rules `[kit hook]`

### 4.1 Rule ids (appended to the RULESET1 3.2 register; all extensions, off in classic)

| Rule id | Gates | Off (and classic) |
| --- | --- | --- |
| `forced_move` | skill `forced_move`, passive and item `forced_move`, trigger effect `move`, arena `push` | fields ignored |
| `ground_hazards` | skill and item `hazard`, the `hazards` kind | nothing placed |
| `arena_hazards` | encounter `arena_hazards` | nothing strikes |
| `triggers` | passive, item and skill `triggers`, skill `riders`, held guard | no event handled |
| `unit_meters` | passive `meter` effect, skill `meters`, meter skills | no meter held |
| `hp_cost` | skill `hp_cost` | no cost |
| `ignore_guard` | skill flag `ignore_guard` | guard applies |
| `ai_tactics` | unit `tactics`, target prefs `SOFTEST`, `TOUGHEST`, hazard-aware tiles | prefs read as ROLE |
| `execute` | skill `exploit` entries with `hp_below` (8.6) | the entry never matches |

The phase effect `deck` (10) rides RULESET1's `deck` rule. Classic worlds set no `classes` (12) and use code zones (13).

### 4.2 Classic and the timeline goldens stay byte-identical

- Every new branch reads its rule first; with a rule off no new random number, trace key, log line or metric appears.
- New fields default to off (`{}`, `[]`, 0, false, ""); catalog rows are appended only; no Ashlands row changes.
- The RULESET1-X fixture (`timeline_kit.json`) sets none of the new fields, so the timeline goldens do not move.
- Each hook logs one line per effect and adds timeline metric keys to `sim feel` (pushes, collisions, hazards placed
  and their damage, traps sprung, arena strikes, triggers fired, meter changes).

### 4.3 One landed hit, in order (extends RULESET1 P4 and P13)

Steps: (1) redirect (8.4), (2) damage, (3) status roll, (4) riposte, (5) stagger and break (P6-P14), (6) forced move
(5), (7) hazard placement (6), (8) riders (8.5), (9) events queued (8.1), run after the action resolves and the
resolver's combo pass (`b/resolver.rs:140`). Area hits run 1-8 per unit in `area_cells` order, then 9 once.

Area order (planner ruling 2026-10-27, HOOKS1-F): an area's hooks run all its hits, then all stagger, then all forced
moves, each in hit order; a unit is never hit twice by one area, and outcomes differ from a per-unit order only when a
pushed unit collides with another target of the same area.

## 5. Forced movement (G3) `[kit hook]`

- FM1. Skill field `forced_move` (family; schema in the task file 1.1): `kind` push or pull, `tiles` 1-3, `from`
  source (default) or center (the area's aim cell; a target standing on it uses the source), `when` always, moved or
  windup, `stop_moves`, `collide_mult`, `collide_stagger`, `collide_break`. It acts on each unit the skill damaged
  that is still alive, at step 6 of 4.3; `when` moved needs a move entry of the target landed this round, windup
  needs the target in windup (P5).
- FM2. Direction: push steps away from the origin, pull toward it, on the dominant axis of `target - origin`; a
  diagonal tie takes the axis of the attacker's facing (RULESET1 MV1), else x. A pull stops at distance 1.
- FM3. Steps run one tile at a time. A step is blocked by: off the grid, a tile that is not walkable, an exit (forced
  moves never withdraw a unit), or an active unit. The move ends at the first block.
- FM4. Collision (a step was blocked with tiles left): the moved unit takes a second hit at `collide_mult` x the
  attacker's power through its own armour and guard (`damage::compute`, no crit roll) and `collide_stagger` through
  P6; with `collide_break` it breaks as P14 does (not when unbreakable or steadfast). A unit that blocked it takes
  the same hit and stagger (any side, HQ11). Walls and blocked terrain take nothing (terrain change is G16).
- FM5. Passive and item effect `forced_move`: `immune` always or guarding (8.3) stops the move and its collision
  ("holds firm"); `bonus_tiles` 1-2 adds tiles to the holder's own pushes and pulls (cap 4).
- FM6. Each tile entered rolls its hazards as a walk does (`hazard_on_enter`, `b/command.rs:727`); a trap stops the
  move there (GH6). Root does not stop a forced move.
- FM7. A forced move is not the unit's move: it does not count as moved for stealth (GD5) or `when` moved, and it
  costs nothing. With `stop_moves` the target's Walk commands that have not landed this round are cancelled.
- FM8. The moved unit's scheduled actions stay: each lands only if still in reach at impact (`b/command.rs:233`); its
  planned move tiles are re-picked as MV6 says.
- FM9. The same step rules serve the trigger effect `move` (8.2) and an arena `push` (7), which give a direction and a
  flat `collide_damage` instead of an attacker. Log `[Push] Hollis drives Footman 2 tiles to (8, 4)`, `[Collision]
  Footman hits the wall: 9 damage`, `[Hold] Bailiff holds firm`.

## 6. Ground hazards (G4) `[kit hook]`, rows `[data]`

- GH1. Catalog kind `hazards` (`data/catalog/hazards.json`): a timed tile overlay with terrain-like numbers: `on`
  (enter, round_end, both), a status with potency, magnitude and rounds, flat `damage`, `cover_guard`, `move_cost`
  (added), `affects` (all, foes or friends of the placer), `trap`, `hidden`, `stop_moves`, `duration`, `tint`, `glyph`.
- GH2. Skill and item field `hazard`: `{id, shape, at, rounds}`. Shapes: SKILL (the skill's own area cells), NONE (one
  tile), PLUS, BURST, LINE, SWEEP (`b/skill.rs:239`); `at` target (the aim tile or the target unit's tile) or self;
  `rounds` 0 = the row's `duration`. Attack, Area and Support skills place at impact (step 7); a Defend places at its
  start, with its grants; an item at use. Only walkable tiles; a trap only on an unoccupied tile.
- GH3. An ATTACK with `power_mult` 0 and a `hazard` is a placement: it aims at a tile like an area opener (Assault's
  round target may be a tile, RULESET1_INTENT 2), deals no damage, rolls no status and raises no hit event.
- GH4. One hazard a tile (the newest replaces); a terrain hazard stays and both apply, terrain first. Placing on an
  occupied tile rolls nothing: the unit there meets it at round end or on re-entering.
- GH5. Effects: on enter (walk, dash, jump, forced move) and at round end, after the terrain's round-end roll
  (`b/resolver.rs:216`); `damage` is flat, through guard, not armour (HQ10); the status rolls its potency as terrain
  does; cover: `Grid::cover_guard` returns the larger of terrain and hazard, so ranged cover follows (Q-54: smoke gives
  cover); move cost adds to the terrain's for paths, the planner and the AI.
- GH6. A trap affects the first unit it `affects` that enters it: its effects apply, that move stops on the tile (the
  rest of the path is dropped), `stop_moves` cancels the unit's later Walks this round, and the trap is removed. A
  `hidden` hazard is not drawn for and not seen by the other side's AI.
- GH7. Duration: `rounds` N lasts until the end of round (placed + N - 1); it expires after the round-end rolls
  (log `[Hazard] Soot Smoke fades`). Instances store the placer (unit, side) for `affects` and `hidden`.
- GH8. AI: the path penalty (`AI_HAZARD_PENALTY`, `b/plan.rs:312`, `:1026`; `b/ai.rs:531`) counts a hazard that would
  harm the unit and that its side can see; PL4 counts it as a threat (11, AT3).

## 7. Scheduled arena hazards (G24) `[kit hook]`

- AH1. Encounter field `arena_hazards` (0-4 rows): `{id, name, cells | columns | rows, rounds, blocks, effect, line}`;
  exactly one of `cells`, `columns`, `rows`; `rounds` [] = every round; `blocks` 1-6. Both sides are struck; terrain
  does not change; a belt is a hazard with `push` every block (data).
- AH2. At the end of each listed block of a listed round, after the block's actions and before the phase checks
  and the verdict (`b/session.rs:943-957`), every active, placed, unreserved unit on a struck cell takes `effect` in
  registration order: damage (flat or `damage_pct` of max HP, through guard, not armour), the status roll, stagger
  (P6: a vent into a windup can break it), then `push` (FM9: `dir` east, west, north or south, `tiles`,
  `collide_damage`). One event cell with `line` (actor: the hazard's `name`).
  As built (user 2026-10-29, HOOKS1-U): the fight ends as soon as one side is wiped out; when the block's actions
  already wiped a side (every hero down, or no unreserved enemy standing) that block's strikes are skipped (no damage,
  no log cells, no random draws); with both sides standing they still strike before the verdict. Also as built: the
  strike cell is logged even with nobody on its cells; arena hits raise no trigger events and give no kill credit; a
  status from a last-block strike acts next round.
- AH3. Telegraph: the struck cells and blocks of the round are known at its start; the tells record gains `arena`
  (cells, blocks) under timeline; the battle screen marks them and the strip shows a hazard badge at the blocks; PL4
  counts a cell due this round as a threat (AT3).

## 8. Events, triggers and riders (G5, G27, G28) `[kit hook]`

### 8.1 Events

Handled after the action in raise order (only `ally_hit` runs at once, at step 1); effects run in list order and
raise no new event (depth 1); meter rules (9) run once on a crossing. Holders: passives (traits, granted, slotted
under `deck`), worn items, and the `triggers` of a skill while its command runs (start to impact), in registration
order, then list order. `limit`: round (default), battle or none.

| Event | Fires for the holder when | `other` |
| --- | --- | --- |
| `hit_taken` | a foe's attack hit dealt it damage | the attacker |
| `guarded_hit` | as `hit_taken`, while guarding (8.3) | the attacker |
| `ally_hit` | a friend within `range` is about to take an attack hit (before damage; the only redirect point) | the attacker |
| `hit_dealt` | its attack hit dealt damage | the target |
| `kill` | its hit took a foe to 0 HP | the downed foe |
| `ally_downed` | a friend within `range` fell | the friend |
| `broken`, `break_dealt` | it broke (P7); its hit broke a foe | the breaker; the broken foe |
| `evaded` | a locked attack on it found nothing (GD5) | the attacker |
| `heal_given`, `cleanse_given` | its heal landed; its cleanse removed at least one status | the target |
| `hp_below` | its HP share crosses from above `pct` to at or below (any cause; after each action and the round-end ticks) | - |
| `guard_start`, `round_start`, `round_end` | a Defend of it starts or a held guard begins; after the poise refill; after the round-end ticks | - |

### 8.2 Conditions and effects

Conditions (`when`, all must hold): `pct` (hp_below), `range` (ally events, default 1), `ranged`, `melee`, `skills`
(families of the event's skill), `other_hp_below` (the other unit's HP share before the event), `other_broken` (it
broke this round), `other_status`, `intent` (the holder's), `meter` (`{id, at_least}`).

| Effect | Keys | Rule (`to`: self default, `other`; riders default `other`) |
| --- | --- | --- |
| `ap` | `value` 1-4 | AP now, capped at max AP; party side only; it never re-plans the round |
| `grant` | `status`, `rounds` (-1 = the battle), `magnitude`, `until` "" or `guard_end` | a buff by `grant_now`; `guard_end` removes it when the unit stops guarding |
| `inflict` | `status`, `potency`, `rounds`, `magnitude` | a debuff by `try_inflict` (one draw) |
| `guard` | `mitigation` 0.1-0.9, `blocks` 0-6 (0 = rest of the round) | a held guard (8.3) |
| `redirect` | `pct` 1-100 | `ally_hit` only (8.4) |
| `move` | `kind`, `tiles` | a forced move of `to` away from or toward the holder (FM9) |
| `meter` | `meter`, `delta` -10..10 | 9 |
| `crit`, `resist` | `value`; `status`, `value` | crit chance points; a resist, both for the battle |
| `stagger`, `damage` | `value` | stagger through P6; flat damage through guard |
| `reveal` | `rounds` 1 | party-side holder: `plan_reveal` (RULESET1 V2) for the next round |
| `end_guard`, `stop_moves` | - | `to`'s running Defends and held guard end now (guard 0); its unlanded Walks this round are cancelled |

### 8.3 Held guard

The `guard` effect sets a held guard (mitigation, last block). `prepare_block` (`b/resolver.rs:83-96`) sets the unit's
guard to the larger of a running Defend's mitigation and the held guard. Guarding = a running Defend or a held guard:
it drives P6's guard factor, `guarded_hit`, and `immune: guarding`. A break (P7) and `end_guard` end it; it ends at
round end.

### 8.4 Redirect and intercept

When friend F of holder H is about to take an attack hit (H within `range` of F, H active, H not the attacker, F
alive): the hit is also computed against H (H's armour and guard, the same crit roll and factors). With `pct` below
100 only damage moves: H takes `pct`% of its result, F `100 - pct`% of its own (rounded, at least 1 when the share is
above 0); status, stagger and forced moves stay on F. With 100 (an intercept) the whole hit (damage, status roll,
stagger, forced move, events) lands on H. A redirect never takes H below 1 HP (the rest goes back to F). One redirect
a hit (CQ18 a): a running skill's triggers first, then passives and worn items, each in registration order; so
Kaelen's running Intercepting Hook takes the hit and Castellan's Penance only the hits it does not take (CAST1 C14).

### 8.5 Riders, HP cost, guard bypass

- RD1. Skill field `riders`: `[{when, effects}]`, run per unit the skill landed on (damaged by a hit, or the support
  target) at step 8; `other` is that unit.
- HC1. Skill field `hp_cost` (rank, 0-99): paid at the command's start after AP; HP becomes `max(1, hp - cost)`
  through `World::damage` (journaled; raises `hp_below`, no hit event). The planner and the AI refuse the skill when
  HP is at or below the cost (reason `HP 9 <= cost 10`). Log `[HP Cost] Flagellant bleeds 10 HP`.
- IG1. Skill family flag `ignore_guard`: its hits set `include_guard` false (`r/damage.rs:161`, `:201`: the guard part
  of the guard-and-cover term is 0, cover stays) and P6's guard factor is 1.0. The breakdown says "ignores guard".

### 8.6 Execute riders (G7) `[kit hook]`

- EX1. `exploit` entry `{"hp_below": pct, "power_mult": x}` (pct 1-99) matches when the target's HP share is at or
  below `pct` before the hit's damage (step 2); the best entry wins (`b/skill.rs:264`: x2.0 beats a staggered x1.5);
  `exploit_consume` removes nothing for it. Rule `execute` (4.1); off and in classic it never matches.
- EX2. Covered: the AP back is a passive trigger `kill`, `when.skills` [family]: `ap` 2 (as Dead-Eye); a flat bonus
  is a rider `other_hp_below`: `damage`; RQ9 reads the holder's HP. Not built: a lethal execute (M4 Talon; HQ15 b).

## 9. Unit meters (G14; the shape Strain reuses, Q-63) `[kit hook]`, rows `[data]`

- UM1. Catalog kind `unit_meters` (`data/catalog/unit_meters.json`): `min`, `max`, `start`, `show` (pips, bar,
  hidden), `decay`, `bands`, `rules`, `skills`. A unit holds a meter when a passive effect `{"type": "meter",
  "meter": id}` applies (optional `start`, `floor`; items: not in HOOKS1-S, whose items take only the `trigger` and
  `forced_move` effects of the task file's 1.2 table; a later row may add an item meter effect). Meters live in battle
  state only (no save key): each battle starts
  at the larger of `start` and the holder's `floor`.
- UM2. Changes: the trigger effect `meter`; the skill field `meters` (`[{meter, delta}]`, applied at the command's
  start after AP and HP cost); values clamp to floor..max. A negative skill delta needs a value at least its size,
  else the command fails unpaid; the planner refuses it (`Bile Phial 0 < 1`).
- UM3. Decay rows at round end, in order, `{delta, unless_gained, if_guarded}`: `unless_gained` skips the row when a
  skill raised the meter this round; `if_guarded` applies it only when the unit guarded this round.
- UM4. Bands (as CAMPAIGN1 M2: the last band whose `from` is at or below the value): a band's `effects` (passive
  grammar: stat, resist, stagger_taken, skill_potency, status_potency, crit) apply while in the band; its `statuses`
  are held (granted for 1 round at each round start and on entering, removed on leaving).
- UM5. Rules fire on a crossing as CAMPAIGN1 M5 (`at` a band id or number, `dir`, `once`); effects are the trigger
  effects plus `meter_set`. The crossing helper is shared (`rpgkit_rules::bands`, new; the first of HOOKS1-M and
  CAMPAIGN1-M to land writes it).
- UM6. Meter skills: the row's `skills` are usable by a holder while their `meters` cost is met, outside the deck (no
  slot, like basics for that unit); FU4's auto order lists them after the deck's actives of their intent.
- UM7. Shown on the unit card, inspect and the strip row (pips up to 10, else a bar).

| Strain need (CANON 2.2, Act 2) | Meter field |
| --- | --- |
| 0-100, brackets Dormant to Rupture with effects | `max` 100; `bands` with `effects` and `statuses` |
| gain by an Inoculated skill's `strain_cost` | skill `meters` `[{"meter": "strain", "delta": 20}]` |
| -10 at round end without an Inoculated action, a further -10 after Active Guard | `decay` `[{-10, unless_gained}, {-10, unless_gained, if_guarded}]` |
| the dormant floor from grafts; reset after battle | effect `meter` `floor`; battle-only state |
| crisis at 100, then set to 75 | `rules` at 100 up: the crisis effect (game code or a later `table` effect), `meter_set` 75 |

## 10. Capstone unlock (G19) and the cheat-death (G18)

### 10.1 Phase effect `deck` `[kit hook]`

- DU1. `{"type": "deck", "unit": ref, "add": [families], "order": {intent: [families]}}`: on fire each active unit of
  the ref adds `add` to its deck's actives and replaces the named intents' order lists. It applies from that unit's
  next planning, as the `ai` effect does (`b/encounter.rs:390-422`). DU2. Checks: `add` owned (path or
  `extra_skills`) and within `deck_slots`; `order` entries in the actives or that intent's basics; one per unit ref.
- As built (HOOKS1-U, user 2026-10-29 (b)): an added family always lands. On an automatic deck (no `active` list) the
  deck first takes its current active slots; when they are full each added family pushes out the last automatic
  family, keeping the slot count (authored actives keep their order and are never pushed out; an authored deck that is
  full stays a DU2 error). The content check warns (not an error) when an `add` on an automatic deck pushes families
  out, naming them; on an automatic deck DU2's `order` check also accepts the automatic actives the `add` keeps (as the
  run time does; planner 2026-10-29), while a family outside that fill and `add` stays an error; the check reads the
  automatic list from the battle's own build (`rpgkit_rules::unit::auto_actives`). An order-only effect
  leaves an automatic deck's actives automatic (a phase is not a player edit).

### 10.2 Cheat-death (G18; Quell is M2, so M2b: HOOKS1-Q, after the slice)

Encounter field `spare_once` (refs): S1's rule, once; phase trigger `{"kind": "spared", "unit"}` fires at the next
block start; phase effect `{"type": "heal", "unit", "pct"}` heals that share of max HP. Quell: `spare_once`
[`proctor_quell`], phase `spared`: line, heal 40, regen 3 rounds, enrage 15.

## 11. AI tactics (G29) `[kit hook]`

- AT1. Target preferences `SOFTEST` (lowest armour rating, then lowest HP, then nearest; unreachable this round last)
  and `TOUGHEST` (highest max HP among melee foes, then the rest; ties nearest), in `rank_targets` (`b/ai.rs:595`)
  and the RULESET1 AI table's row 5; a reachable taunter still ranks first.
- AT2. Unit field `tactics` and the phase `ai` key `tactics` (replaces the list): `brace_after_hit` (a row before
  AI table row 4: a friend within 2 tiles, or itself, lost HP to an attack hit last round, not reckless, a Guard
  skill usable: Guard, self or its `hold_line` tile); `hold_line` (PL4 for this unit ranks after threat: tiles next to
  a friend first, then fewer walkable free neighbours first); `flush_cover` (an area opener's aim scores +1 per foe hit
  on a tile with cover, `cover_for` from the aim's side).
- AT3. Hazard awareness for every unit (no tactic): PL4's threat count adds 1 for a visible harmful ground hazard
  on the tile and 1 for an arena hazard due on it this round.
- AT4. Not built: targeting a unit in windup (HQ2), raised tiles (G8), pack hunting (M4, a later tactic `pack`).

## 12. Class names per world (G1, Q-53) `[kit hook]`, names `[data]`

- CN1. World manifest field `classes` (json, {}): `{class_id: {name, description, skill_path}}`; per class id the
  nearest manifest in the lineage with that key wins (as `party_max`, `rpgkit_world/src/data.rs:165`).
- CN2. `Data::class_name(world, class_id)` and `class_description`: the overlay's, else the class row's. Screens with a
  world read them: the hub roster (`gui/hub.rs:18`), the party header (`gui/party.rs:1143`), battle inspect, results
  and recruit lines. Catalog screens (Dojo, Forge, Arena) keep the base names.
- CN3. `skill_path` present (an object; `{}` allowed) replaces the class path for every unit built in that world
  (members, world characters, encounter units: the world roster's `customize`, `b/session.rs:111`). No proficiency
  key: CAST1 CQ1 (a) drill passives stay (HQ6).

## 13. Zones as data (G22) `[kit hook]`, rows `[data]`

- ZN1. ENCOUNTER2-T's `rows` author the layout; missing is a backdrop id beyond the 8 code zones. Catalog kind
  `zones` (`data/catalog/zones.json`): `name`, `rows` (default 9 rows of floor), `backdrop` (asset key, default
  `backgrounds/battle/<id>.png`), `ground` (terrain id for the STAGE1 fallback gradient).
- ZN2. The encounter `zone` options are the 8 code ids, then the rows; a row may not reuse a code id (the classic
  zones stay code, parity).
- ZN3. Board: the encounter's `rows`, else the zone's (code zones unchanged, `b/grid.rs:164`). Backdrop: the row's
  `backdrop`; a missing file draws STAGE1's gradient (3.4) from `ground`. No per-zone tile sets (tiles are per terrain).

## 14. Out of scope

G2, G6, G8-G12, G15-G17, G20, G21, G25, G26 (not in the slice; riders can later carry a lethal execute and
second statuses); hazard overlap reactions (Volatile Catalyst); the Strain crisis table; Ash-Cloud's jump (Q-54); windup
targeting (HQ2); Content Forge forms for the new fields (raw JSON, as ENCOUNTER2).

## 15. As built (2026-10-29, CLEANUP.md CLN-24)

One line per landed task where the build settled something this spec does not state; the full notes are the task rows
in `docs/archive/TASKS_DONE.md`, the user's rulings the rows in `games/emberwake/docs/decisions/combat.md`.

- HOOKS1-GF: with the rule off the battle's catalog drops skill hazards at the start (`World::ignore_hazards_when_off`).
- HOOKS1-Z: data zones for encounters and road fights apply the board and hold tiles through `Grid::apply_board`.
- HOOKS1-E (user-confirmed): a redirect's leftover counts against the holder; ripostes raise no hit events; area holders
  take no redirect; the depth-1 rule. `ally_hit` non-hit effects (Intercepting Hook's pull) run after the action
  (user 2026-10-29, decision "RULESET1-U slice 1 and HOOKS1-E answers"; this line said "pending the user's pick").
- HOOKS1-K: decision "HOOKS1-K answers (user 2026-10-29)": the AI uses an HP-cost skill whenever it can pay; riders skip
  a user that died to a riposte.
- HOOKS1-X: an `exploit` entry `{hp_below, power_mult}` reads the target's HP share before the hit; the best entry wins
  and nothing is consumed.
- HOOKS1-M: decision "HOOKS1-M and deck answers (user 2026-10-29)"; meters share `rpgkit_rules::bands` with the
  campaign meters.
- HOOKS1-H: decision "HOOKS1-H, RULESET1-G and STATUSFX1-L answers (user 2026-10-29, all as recommended)".
- HOOKS1-C: members are built in their character's world (`UnitState::path_world`); screens name classes by the world
  being played.
- HOOKS1-U (building): decision "HOOKS1-U unlock on an automatic deck (user 2026-10-29)".
- HOOKS1-AI: AT3's threat (ground and arena) is under `ai_tactics` (the arena part also needs `arena_hazards`); hits
  are remembered per round in `World::hits` (attack hits that took HP); under round intent `flush_cover` picks a tile
  target for an area opener that still holds the first ranked target; SOFTEST and TOUGHEST outrank `reckless`.
- HOOKS1-UI (building; slice 1): UM7's pip cutoff is the steps from min to max (`max - min` above 10 reads as a bar) in
  the content check as on the battle screen; leaving a band removes a resist key the band created; the battle report
  calls an execute finish "executed"; the board marks the ground hazards the party's side sees and the arena's struck
  cells (overlay groups `hazards`, `arena`).
