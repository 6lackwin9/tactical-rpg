# Combat hooks as data

Status: canon (HOOKS1-D, 2026-10-31). Game-agnostic: this page names no game. It is the authoring guide for the RPG
kit's combat hooks (`kits/rpg/`, spec `docs/specs/HOOKS1.md` with the field tables in `docs/specs/HOOKS1_TASKS.md`
section 1): forced moves, ground hazards, arena hazards, triggers and riders, HP costs and guard bypass, execute, unit
meters, data zones, the encounter phase effects `deck`, `kill` and `talk` with phase groups and the event triggers, scripted rounds, and AI tactics. A game uses them
by writing rows; it writes no code for them ([layer_rule.md](layer_rule.md): kit hook plus data).

## Rules every hook shares

- **Gated by a ruleset rule.** Each hook acts only when the battle's ruleset lists its rule id (the kit's rule
  register, `kits/rpg/crates/rpgkit_rules/src/ruleset.rs`): `forced_move`, `ground_hazards`, `arena_hazards`,
  `triggers`, `unit_meters`, `hp_cost`, `ignore_guard`, `ai_tactics`, `execute` (all extensions, HOOKS1 4.1); the
  phase effect `deck` rides the `deck` rule (the trigger `broken` the `poise` rule, scripted rounds the
  `round_intent` rule); the phase effects `kill` and `talk`, phase groups, the triggers `round` + `block` and `downed`, and data zones need no rule. A
  ruleset without the rule reads the fields as absent: no random draw, log line or metric key appears, so a classic
  ruleset and recorded goldens do not move (HOOKS1 4.2). With `ground_hazards` off the battle's catalog drops skill
  hazards at the start (`World::ignore_hazards_when_off`).
- **Off by default.** Every field defaults to off (`{}`, `[]`, 0, false, ""); a unit or skill that names nothing
  behaves as before.
- **One landed hit, in order** (HOOKS1 4.3): redirect, damage, status roll, riposte, stagger and break, forced move,
  hazard placement, riders, then the queued events after the action. An area runs all its hits, then all stagger,
  then all forced moves, each in hit order.
- **Write rows through the content tool**, never raw JSON: `tools/fc.sh content upsert <kind> @row.json` (or a seed
  through `content apply`), then `tools/fc.sh content check`. The checks live in `rpgkit_content/src/check/hooks.rs`
  (fields on skills, items, passives, units, encounters, manifests) and `check/hook_kinds.rs` (the
  kinds `hazards`, `unit_meters`, `zones`); the typed rows in `kits/rpg/crates/rpgkit_data/src/lib.rs` (`ForcedMove`,
  `HazardPlace`, `TriggerRow`, `Rider`, `MeterDelta`, `HazardDef`, `UnitMeterDef`, `ZoneDef`).
- **Measure with the sim.** `sim feel` adds a metrics key per hook (`forced`, `hazards`, `arena`, `triggers`,
  meters); use it to see a hook fire before a playtest.

The examples below use made-up ids; numbers are only illustrations.

## Forced moves (rule `forced_move`)

Code `kits/rpg/crates/rpgkit_battle/src/forced.rs`; spec HOOKS1 5 (FM1-FM9); tests `rpgkit_battle/tests/forced_move.rs`.

| Where | Field | Meaning |
| --- | --- | --- |
| skill (ATTACK, family) | `forced_move` | `kind` push or pull, `tiles` 1-3, `from` source or center (an area's aim cell), `when` always, moved or windup, `stop_moves`, `collide_mult` 0-3, `collide_stagger` 0-50, `collide_break` |
| passive or item effect | `{"type": "forced_move", "immune", "bonus_tiles"}` | `immune` always or guarding ("holds firm"); `bonus_tiles` 1-2 added to the holder's own pushes and pulls (cap 4) |

- It moves each unit the skill damaged and that is still alive, after stagger, along the dominant axis away from (push)
  or toward (pull) the origin; a pull stops at distance 1.
- A step blocked by the board edge, an unwalkable tile, an exit or a unit is a collision: the moved unit takes
  `collide_mult` x the attacker's power (no crit, no random draw) and `collide_stagger`; a unit it hit takes the same;
  `collide_break` breaks a windup. Walls take nothing.
- A forced move is not the unit's own move: it costs nothing, does not count as "moved", and tiles it enters roll their
  ground hazards (a trap stops it). `stop_moves` cancels the target's Walks still to land this round.
- The same step rule (`shove_unit`) serves the trigger effect `move` and an arena hazard's `push`, with a flat
  `collide_damage` instead of an attacker.

```json
{"forced_move": {"kind": "push", "tiles": 2, "collide_mult": 0.5, "collide_stagger": 6}}
```

## Ground hazards (rule `ground_hazards`)

Code `rpgkit_battle/src/ground.rs` (the overlay `Grid::ground`); spec HOOKS1 6 (GH1-GH8); tests
`rpgkit_battle/tests/ground_hazards.rs`.

A catalog row of kind `hazards` (`data/catalog/hazards.json`) is a timed tile overlay; a skill or consumable places it.

| Where | Field | Meaning |
| --- | --- | --- |
| `hazards` row | `on` | enter, round_end or both |
| `hazards` row | `status`, `potency`, `magnitude`, `status_rounds`; `damage` | the status roll; flat damage through guard, not armour |
| `hazards` row | `cover_guard` 0-0.5, `move_cost` 0-3 | cover (the larger of terrain and hazard); added path cost |
| `hazards` row | `affects` all, foes, friends (of the placer); `trap`; `hidden` (foes only); `stop_moves` | who it touches; a trap springs once and stops the move; a hidden hazard is unseen by the other side |
| `hazards` row | `duration` 1-5, `tint`, `glyph` | rounds; the board look |
| skill (ATTACK, DEFEND, SUPPORT) or consumable | `hazard` | `{id, shape, at, rounds}`: `shape` SKILL (the skill's own area), NONE, PLUS, BURST, LINE, RING, ARC; `at` target or self; `rounds` 0 = the row's `duration` |

- Attack, area and support skills place at impact, a Defend at its start, an item at use; only walkable tiles, a trap
  only on an empty tile. An ATTACK with `power_mult` 0 and a `hazard` is a pure placement: it aims at a tile, deals no
  damage and raises no hit event (under round intent it is a tile-target Assault opener).
- One hazard a tile (the newest replaces it); a terrain hazard stays under it and both apply, terrain first.
- It fades after the round-end rolls of round placed + `rounds` - 1. The AI pays a path penalty for a visible hazard
  that would harm the unit (`ground::ai_penalty`), and its tile ranking counts it as a threat.

```json
{"id": "caltrops", "name": "Caltrops", "on": "enter", "status": "slow", "potency": 80, "status_rounds": 1,
 "damage": 4, "affects": "foes", "trap": true, "hidden": true, "duration": 3, "tint": "#a0a0a0", "glyph": "x"}
```

```json
{"power_mult": 0.0, "hazard": {"id": "caltrops", "shape": "NONE", "at": "target", "rounds": 3}}
```

## Arena hazards (rule `arena_hazards`)

Code `rpgkit_battle/src/arena.rs`; spec HOOKS1 7 (AH1-AH3); tests `rpgkit_battle/tests/arena_hazards.rs`.

An encounter's `arena_hazards` (0-4 rows) strike fixed cells on a schedule: `{id, name, cells | columns | rows, rounds,
blocks, effect, line}`; exactly one of `cells`, `columns`, `rows`; `rounds` [] = every round; `blocks` 1-6.
`effect` holds at least one of `damage` 0-99, `damage_pct` 0-50 (of max HP), `stagger`, a status (`status`, `potency`,
`magnitude`, `status_rounds`) and `push` `{dir, tiles, collide_damage}` (`dir` east, west, north, south).

- At the end of each listed block of a listed round, after the block's actions and before the verdict, every active
  unit on a struck cell (either side) takes the effect: damage through guard, not armour, the status roll, stagger,
  then the push. One log cell with `line`.
- When the block's actions already wiped out a side, that block's strikes are skipped. Arena hits raise no trigger
  events and give no kill credit.
- The round's cells and blocks are told at its start (`Session::arena_tells`), so both the player and the AI's tile
  ranking can step off them. A conveyor is a hazard that pushes every block.

```json
{"arena_hazards": [{"id": "vents", "name": "Steam Vents", "rows": [4], "blocks": [3, 6],
  "effect": {"damage": 6, "stagger": 10}, "line": "The vents blast steam."}]}
```

## Triggers and riders (rule `triggers`)

Code `rpgkit_battle/src/trigger.rs` (held guard `rpgkit_rules/src/unit.rs` `HeldGuard`); spec HOOKS1 8.1-8.5; tests
`rpgkit_battle/tests/triggers.rs`, `riders.rs`.

A trigger row is `{on, when, effects, limit}`. Holders: a passive effect `{"type": "trigger", ...}` (traits, granted or
slotted passives), a worn item's effect of the same shape, and a skill's `triggers` (DEFEND, ATTACK, SUPPORT) while its
command runs (from its start to its impact).

| Part | Values |
| --- | --- |
| `on` (events) | `hit_taken`, `guarded_hit`, `ally_hit` (before the damage; the only redirect point), `hit_dealt`, `kill`, `ally_downed`, `broken`, `break_dealt`, `evaded`, `heal_given`, `cleanse_given`, `hp_below`, `guard_start`, `round_start`, `round_end` |
| `when` (all must hold) | `pct` (hp_below, required), `range` 1-6 (ally events, default 1), `ranged`, `melee`, `skills` (families of the event's skill), `other_hp_below`, `other_broken`, `other_status`, `intent`, `meter` `{id, at_least}` |
| `effects` (1-4, in order; `to` self or other) | `ap`, `grant`, `inflict`, `guard` (a held guard), `redirect` (`ally_hit` only), `move`, `meter`, `crit`, `resist`, `stagger`, `damage`, `reveal`, `end_guard`, `stop_moves` |
| `limit` | round (default), battle, none |

- Events run after the action in raise order and raise no new event (depth 1); only `ally_hit` runs at once.
- A redirect moves `pct`% of the damage to the holder (100 = the whole hit lands on it, an intercept), never below 1 HP
  for the holder; one redirect a hit, a running skill's trigger first, then passives and items in registration order.
  Its leftover counts against the holder; area hits are not redirected.
- **Riders**: a skill's `riders` `[{when, effects}]` run per unit the skill landed on (damaged, or the support
  target), after the forced move and the placement; `other` is that unit (the default `to`). Riders skip a user that
  died to a riposte.

```json
{"type": "trigger", "on": "ally_hit", "when": {"range": 1}, "effects": [{"type": "redirect", "pct": 35}],
 "limit": "round"}
```

```json
{"riders": [{"when": {"other_hp_below": 50}, "effects": [{"type": "inflict", "status": "bleed", "potency": 70,
  "rounds": 2, "magnitude": 3}]}]}
```

## HP cost and guard bypass (rules `hp_cost`, `ignore_guard`)

Spec HOOKS1 8.5 (HC1, IG1); tests `rpgkit_battle/tests/riders.rs`.

- Skill `hp_cost` (rank, 0-99): paid at the command's start after AP; HP becomes at least 1 (it raises `hp_below`, no
  hit event). The planner and the AI refuse the skill at or below the cost; the AI otherwise uses it whenever it can
  pay. The content check warns when the cost is above half the lowest owner's max HP.
- Skill flag `ignore_guard` (ATTACK): its hits skip the guard term (cover stays) and the stagger guard factor.

## Execute (rule `execute`)

Code `rpgkit_battle/src/skill.rs` (the exploit pick); spec HOOKS1 8.6 (EX1-EX2); tests `rpgkit_battle/tests/execute.rs`.

An `exploit` entry `{"hp_below": pct, "power_mult": x}` (pct 1-99) matches when the target's HP share is at or below
`pct` before the hit; the best matching entry wins and nothing is consumed. The estimate, the AI's combo order and the
log read it. An AP refund on a kill is a passive trigger `kill` with `when.skills`; a flat bonus is a rider with
`other_hp_below`. A lethal execute is not built.

```json
{"exploit": [{"hp_below": 35, "power_mult": 2.0}]}
```

## Unit meters (rule `unit_meters`)

Code `rpgkit_battle/src/meter.rs`, the band and crossing helper `rpgkit_rules/src/bands.rs` (shared with campaign
meters); spec HOOKS1 9 (UM1-UM7); tests `rpgkit_battle/tests/meters.rs`.

A catalog row of kind `unit_meters` (`data/catalog/unit_meters.json`) is a battle-only gauge (no save key):

| Field | Meaning |
| --- | --- |
| `min`, `max`, `start`; `show` pips, bar, hidden | range and start; pips up to a span (max - min) of 10, else a bar |
| `decay` | 0-3 round-end rows `{delta, unless_gained, if_guarded}` |
| `bands` | 0-8 `{id, name, from, effects, statuses}`: the last band whose `from` is at or below the value holds; `effects` use the passive grammar, `statuses` are held while in the band |
| `rules` | 0-6 `{id, at, dir, once, effects}` fired on a crossing (`at` a band id or a number); trigger effects plus `meter_set` |
| `skills` | meter skills: usable while their cost is met, with no deck slot |

- A unit holds a meter through a passive effect `{"type": "meter", "meter": id, "start", "floor"}`; each battle starts
  at the larger of `start` and the floor.
- Changes: a skill's `meters` `[{meter, delta}]` (paid at the command's start; an uncovered cost is refused by the
  planners and fails unpaid) and the trigger effect `meter`; values clamp to floor..max. Band stat effects are not
  re-derived mid-battle (max HP, speed and range stay as built). Leaving a band removes the resist keys it added.

```json
{"id": "rage", "name": "Rage", "short": "Rage", "min": 0, "max": 5, "start": 0, "show": "pips",
 "decay": [{"delta": -1, "unless_gained": true}],
 "bands": [{"id": "calm", "name": "Calm", "from": 0},
           {"id": "fury", "name": "Fury", "from": 3, "statuses": [{"status": "empower", "magnitude": 15}]}]}
```

## Data zones (no rule)

Code `rpgkit_battle/src/grid.rs` (`grid::data_zone`, `Grid::apply_board`), `rpgkit_content/src/art.rs`
(`zone_backdrop`); spec HOOKS1 13 (ZN1-ZN3); tests `rpgkit_content/tests/zones.rs`.

A `zones` row (`data/catalog/zones.json`) is a battle zone beyond the kit's code zones: `name`, `rows` (9 strings of
12 terrain glyphs; empty = floor), `backdrop` (asset key; default `backgrounds/battle/<id>.png`), `ground` (the terrain
the fallback gradient uses). A row may not reuse a code zone id. The board is the encounter's `rows`, else the code
zone, else the row's `rows`; an encounter's `zone` and a travel route's road-fight `zone` take either kind of id.

## Phase effects `deck` and `kill`, phase groups

Code `rpgkit_battle/src/encounter.rs` (`Phases::apply_decks`, `Phases::mark_fired`, `KILL_OVERRIDES_UNKILLABLE`),
`session.rs` (`phase_kill`), `world.rs` (`World::kill`); specs HOOKS1 10.1 (DU1-DU2), `docs/specs/ENCOUNTER2.md` 4 and
6; tests `rpgkit_battle/tests/deck_unlock.rs`, `phase_kill.rs`, `rpgkit_content/tests/phase_kill_checks.rs`.

- **`deck`** (rule `deck`): `{"type": "deck", "unit": ref, "add": [families], "order": {intent: [families]}}` adds
  `add` to each active unit of the ref's deck actives and replaces the named intents' order lists, from that unit's
  next planning (a phase fired mid-round: the next round), once per unit. On an automatic deck an added family always
  lands and pushes out the last automatic one (the content check warns, naming it); a full authored deck is an error.
  Use it for a boss capstone that unlocks in a later phase (the family must be owned: path or `extra_skills`).
- **`kill`**: `{"type": "kill", "units": [refs], "by": ref, "within": 0}` drops the refs' active units (never a hero)
  to 0 HP: one log cell, no kill credit, no trigger event; it overrides `unkillable`. `within` > 0 keeps only units
  that close to the `by` unit.
- **Phase key `group`**: when a phase fires, its unfired group-mates drop (two due at once: the first in list order).
  Use it for branches such as "the boss falls" versus "the timer runs out".

```json
{"id": "unleash", "name": "Unleash", "trigger": {"kind": "hp_below", "unit": "boss", "pct": 50}, "group": "end",
 "effects": [{"type": "deck", "unit": "boss", "add": ["earthbreaker"], "order": {"assault": ["earthbreaker"]}}]}
```

## Battle talk and event triggers

Code `rpgkit_battle/src/encounter.rs` (`TALK_EFFECT`, `TalkCue`, `Phases::mark_block`, `Phases::check_events`),
`session.rs` (`talks`, `units_named`); spec `docs/specs/ENCOUNTER2_C.md` 3.1, 3.2 and 5.2; tests
`rpgkit_battle/tests/battle_talk.rs`, `rpgkit_content/tests/battle_talk_checks.rs`, `rpgkit_host/tests/battle_talk.rs`.

- **`talk`**: `{"type": "talk", "dialogue": ref, "speaker": ref}` (at most one per phase, world encounters only: dialogue
  trees are world rows) adds a cue `{phase, dialogue, speaker, round, block}` to `Session::talks()` when its phase
  fires. No rule reads it: the battle plays exactly as without it, and sims and the host never wait. The game's battle
  screen opens the tree over the fight. Its tree may run only `set_flag` and `add_flag` effects.
- **`round` with `block`**: 0 (the default) fires at the round start, before planning (round 1 included); 1 to
  `total_blocks` at the start of that block.
- **`downed`**: `{"kind": "downed", "unit": name, "count": 1}` is due at the end of the block in which the count-th unit
  of the name is at 0 HP (any cause, phase kills included); **`broken`**: `{"kind": "broken", "unit": name}` once one of
  them breaks (rule `poise`; never without it). Both fire at the next block start, or the next round start after the
  last block; a phase due when the battle ends never fires. A name is an encounter ref (every unit built from it),
  `party` (any hero) or `party:<id>` (one hero).

```json
{"id": "first_down", "name": "First down", "trigger": {"kind": "downed", "unit": "footman", "count": 1},
 "effects": [{"type": "talk", "dialogue": "dlg_first_down", "speaker": "captain"}]}
```

## Scripted rounds (rule `round_intent`)

Code `rpgkit_battle/src/script.rs` (`Script`, `RoundScript`, `ScriptEntry`, `UnitName`, `plan_entries`), `session.rs`
(`round_script`, `round_locked`, `pick_on`, `script_party`); checks `rpgkit_content/src/check/encounter.rs`
(`script_problems`, `script_warnings`); spec `docs/specs/ENCOUNTER2_C.md` section 4 and 5.1; tests
`rpgkit_battle/tests/script_round.rs`, `rpgkit_content/tests/battle_script_checks.rs`, `rpgkit_host/tests/script_round.rs`.

An encounter's `script` plans both sides for one to three given rounds, for a tutorial or an opening scene. It is data
fed to the picks a player makes, not a second combat layer: AP, ranges, moves, Maneuver positions and windups all
apply, and a script never forces a hit, a miss or a roll.

- **A round script**: `{"round": n, "lock": true, "others": "auto", "banner": "", "entries": [...]}` (1-16 entries,
  one per unit). At the round start (after its phases) the enemies' entries are planned in list order, then the AI
  plans the other enemies; then the party's entries, then the members with no entry (`others`: `auto` their auto plan,
  `rest` Rest all round). `lock` true confirms the plan at once (the host refuses planning requests with "Round n is
  scripted."); `lock` false is a guided round (the enemies fixed, the party's entries become first picks the player
  may change).
- **An entry**: `{"unit": name, "target": "self", "picks": [...], "fill": "auto"}`. A name is `ref` (the first unit
  built from the encounter ref), `ref#n` (the n-th), `party:<id>` (a hero by its key) or `party#n` (the n-th hero). The
  target is `"self"`, a name or a tile `[x, y]`. Picks (1-4): `{"skill": family}` (the unit's own rank; optional
  `kind` engage, disengage, utility), `{"item": id}` (party only), `"rest"`, `{"move": [x, y]}` (first only),
  `{"step_after": [x, y]}`. `fill` `auto` lets the planner fill the blocks after the picks; `rest` leaves them Rest.
- **Fallbacks**: an entry whose unit or unit target is not on the field, whose skill the unit lacks or whose picks are
  refused logs one `Script` cell ("Script: name reason; it plays its own round.") and its unit plays its own round (the
  AI, or the hero's auto plan); the round stays locked. A ruleset without `round_intent` ignores the script with one
  cell. The metrics count `scripted_rounds` and `script_fallbacks` (in the sim's JSON only when not zero).

```json
{"script": [{"round": 1, "lock": true, "others": "auto", "banner": "", "entries": [
  {"unit": "brute", "target": "party:captain", "picks": [{"skill": "heavy_swing"}], "fill": "auto"},
  {"unit": "party:captain", "target": "brute", "picks": [{"skill": "quick_jab"}], "fill": "auto"}]}]}
```

## AI tactics (rule `ai_tactics`)

Code `rpgkit_battle/src/ai.rs` (`rank_by_build`, the `brace` row, `hold_tile`, `flush_pick`), `world.rs`
(`UnitTactic`, `World::hits`), `round_intent.rs` (`hold_rank`, the hazard threat); spec HOOKS1 11 (AT1-AT4); tests
`rpgkit_battle/tests/tactics.rs`.

| Field | Values |
| --- | --- |
| unit `target_pref` | adds `SOFTEST` (lowest armour, then HP, then nearest) and `TOUGHEST` (melee foes by max HP); a reachable taunter still ranks first; both outrank `reckless` |
| unit `tactics`; phase `ai` key `tactics` (replaces the list) | `brace_after_hit` (Guard when it or a friend within 2 tiles lost HP to an attack hit last round), `hold_line` (tiles next to a friend first, then fewer free neighbours), `flush_cover` (an area aim scores foes in cover; under round intent a tile-target area opener) |

Every unit's tile ranking also counts a visible harmful ground hazard and an arena hazard due on the tile as threats.
Without the rule the preferences read as `ROLE` and every plan is as before.

```json
{"target_pref": "SOFTEST", "tactics": ["brace_after_hit", "hold_line"]}
```

## Class names and paths per world (no rule)

Code `kits/rpg/crates/rpgkit_world/src/data.rs` (`class_name`, `cat_in`), `progression.rs` (`build_in`, `build_played`,
`build_with`); spec HOOKS1 12 (CN1-CN3) and its section 15.

A world manifest's `classes` `{class_id: {name, description, skill_path}}` renames a class in that world (the nearest
manifest in the world lineage with the key wins) and, when `skill_path` is present (`{}` allowed), replaces its skill
path for every unit built there. A party member is built with the class path of the world being played; a saved
loadout, deck or rank stays valid if its class owns the family in any world, and a build in a world that lacks it
falls back to the default loadout. Catalog screens keep the base names.

```json
{"classes": {"warrior": {"name": "Shieldbearer", "description": "A city guard's drill.", "skill_path": {}}}}
```

## Not built yet

A cheat-death (`spare_once`, the phase trigger `spared`, the phase effect `heal`: HOOKS1 10.2, task HOOKS1-Q),
targeting a unit in its windup, teleports on an event, terrain changed by a skill, overlapping hazards that react, a
lethal execute and a pack-hunting tactic (HOOKS1 14, AT4). A game that needs one asks for a kit hook first.

ROLLTABLE1 (2026-11-10): `roll_table` is also a trigger effect (`rows` of `{weight, effects}`, one weighted pick on the seeded battle RNG) and a meter-rule effect.

PK1-S1 (2026-11-10): trigger effects also `strike_nearest`, `mark`, `ap` (`set`, `rounds`) and `damage` `pct_of_hit`; a `hazards` row and an arena `effect` take `meter` {meter, delta} (a holder's meter change); unit-meter bands take `heal_taken` (bands only); meter-rule effects take the new trigger effects. On `ally_hit` a `roll_table` cannot hold a `guard` (PR5-16).

PK1-S2a (2026-11-10): skill fields `channel` {none, only, cost, power, effects}, `requires_meter` {meter, min}, `hp_cost_pct`; rule `channel` with ruleset fields `channel_meter`, `channel_power`, `channel_cost`, `marrow_actives`, `channel_rest`; a meter row's `skills` may raise the meter when `channel.only`.

PK1-S2b (2026-11-10): the game term `marrow_mark` names a marrow version ("Name (Marrow)").

HEALPCT1 (2026-11-10): skill field `heal_pct` (SUPPORT; a percent of the target's max HP added after the heal formula; `heal_taken` applies); ruleset field `channel_ai_edge`.
