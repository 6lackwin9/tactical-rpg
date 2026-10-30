# ENCOUNTER2: encounter structures for the new campaign (stay-behind, node 20 phases, connected rooms)

Status: proposed (spec 2026-10-25, from the user decisions Q-60, Q-75, Q-75a (a), "Nemeth's fight, revised" and
"Slice is data only" in `games/emberwake/docs/decisions/lore.md`)
Depends on: none for the slice parts; STAGE1 for the room screen; RULESET1 (written in parallel) for windups,
recovery (enemies), round intents and follow-ups, poise, reach-2 and the intent tells, which this spec only references
Lane: logic first (schema and rules), then asset (kit samples); tasks, tests, parity plan and open questions:
[ENCOUNTER2_TASKS.md](ENCOUNTER2_TASKS.md)
Touches (by task, see the task file): under `kits/rpg/crates/`: `rpgkit_battle/src/{encounter,session,
world,grid,plan,ai,intent,log}.rs`, `rpgkit_content/src/check/encounter.rs`, `rpgkit_world/src/{battle,explore,
dialogue,travel}.rs`, `rpgkit_host/src/lib.rs` (KIT1-N), `rpgkit_sim/src/args.rs`; under `games/emberwake/crates/`:
`emberwake_gui/src/battle.rs`, `emberwake_host/src/cli.rs`; `kits/rpg/schema.json`;
`games/emberwake/data/catalog/encounters.json` (asset lane, `kit_*` samples)
Paths (KIT1_TASKS 5): `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`;
line numbers in kit files are pre-KIT1 (KIT1-M moved them byte-identical).

History: `docs/HISTORY.md` holds no row on rooms, stay-behind or phase kills. Decision S12 (2026-10-17,
`decisions/combat.md`) kept zone maps in code "until S9 map data"; the new reason to author terrain as data is
"Slice is data only" (2026-10-25): the Crucible needs a winch and a gate no code zone has (section 5.1).

## 1. Goal

Three kit hooks for encounters, each off unless an encounter names it, so the Ashlands "classic" runs byte-identical:
stay-behind objectives (a hold-a-tile escape and an impossible fight with a last stand) whose result reports who
stayed; three small phase effects (kill named units, swap a unit's body, switch a unit to "hostile to all") plus
phase groups, for node 20; and connected rooms: several 12 x 9 boards in one battle on one timeline, linked by doors.
The slice needs the hold escape (the Crucible), the kill effect (M2's Culling Timer) and authored terrain rows; rooms,
the last stand, swap and side switch come later (task file section 3).

## 2. Survey: encounters today (develop at c1d8411)

| Area | Where | What it does |
| --- | --- | --- |
| Board | `rpgkit_battle/src/grid.rs:8-9`, `:148-171` | One `Grid` per battle, 12 x 9 (`GRID_W`, `GRID_H`); terrain from 8 code zones (`zone_rows`, `grid.rs:133`); no authored layout |
| Exits | `grid.rs:220-226`, `world.rs:261-276` | Listed tiles (default `[[11, 4]]`, `session.rs:76-79`) or every edge when `arena` is OPEN; a step onto one withdraws the unit |
| Withdraw | `plan.rs:648-668`, `plan.rs:603` | The planner's Withdraw routes to the nearest exit; a plain move onto an exit is refused |
| Deployment | `world.rs:291-430` | Party columns 5/4, enemies 7/8 (ambush, centre variants); leftovers and waves take free tiles nearest the middle of their half |
| Objectives | `encounter.rs:22-134` | rout, hold, escape, slay; verdict order (`:58-110`): a protected guest dead = lost, every hero dead = lost, no active foe = won, the objective, then no hero active with someone withdrawn = fled; escape needs `objective_count` (1-4) out and no hero left active |
| Guests | `encounter.rs:20`, `session.rs:250-322` | At most 2 party-side guests; `protect` makes their death a loss |
| Phases | `encounter.rs:138-423`, `session.rs:561-709`, `:976-1006` | Triggers `hp_below` (unit, pct, or_round), `round`, `arrival`; effects line, wave, status, ai, enrage; fire at round start or block start; phase-wave units spawn at battle start and stay reserved until fired |
| Phase checks | `rpgkit_content/src/check/encounter.rs:137-160`, `:279-330` | At most 4 phases, 5 effects, 12 enemy-side units; an `hp_below` unit must be in `enemies` |
| Sides | `world.rs:121-127`, `:189-209` | Two lists (`allies`, `enemies`); `foes_of` and `friends_of` return slices; `ai.rs` calls them 22 times |
| Battle end | `session.rs:1056-1072`; `emberwake_gui/src/battle.rs:705-720`; `rpgkit_host/src/lib.rs:203`; `sim.rs:208` | `outcome` won, lost or fled; the "last round over = fled" rule is written three times outside the session |
| Results | `rpgkit_world/src/battle.rs:173-211`, `session.rs:165-189`, `gui/battle.rs:722-768` | `Tally` holds enemy fates only (killed, surrendered, fled, standing); the GUI writes hero HP back; no party-side result; no unit lifecycle (`captured` appears in no crate); the party is `members` + `active_ids` (`rpgkit_world/src/state.rs:293-294`) |
| Key loot | `rpgkit_world/src/battle.rs` (REWARD1 step 6) | Paid on a win when every unit of the ref is killed, by anyone |
| Saves, replays | `rpgkit_world/src/save.rs`; `emberwake_host/tests/replay.rs` | Battles are not saved mid-fight; recordings replay command-API requests |
| Screen | `emberwake_gui/src/battle.rs:37-41`, `:1396`, `:1563` | Board at (128, 30), 32 px cells; the strip panel on top; STAGE1 (approved, not on this branch) tilts it to 32 x 24 |
| Sim | `rpgkit_sim/src/args.rs:178` | The default encounter list is every catalog encounter |
| Dialogue | `rpgkit_world/src/dialogue.rs:178` | Choice conditions: flags and stats; none for "this member is deployed" |

## 3. Stay-behind objectives `[kit hook]`

Both are variants of the `escape` objective, set by one block `stay_behind` (section 6). Shared rules:

- S1. `unkillable` (encounter field, refs): damage never takes such a unit below 1 HP; the first time it would, the log
  gets an event cell "`{name}` will not fall here." (once per unit). Rout can then not end the fight. Also usable
  for Quell's cheat-death (Q-74) and any story boss that must survive.
- S2. `timeout_outcome` (encounter field, `fled` default, or `lost`): the verdict when the last round ends with none.
  The rule moves into `Session` (`Session::final_outcome`); the GUI, host and sim call it. Default `fled` is
  today's rule, so classic outcomes do not change.
- S3. Party fates in the result: `Session::party_fates()` gives each hero (not guests) one of `escaped` (withdrew
  through an exit), `stayed` (held, section 3.1), `stood` (made the last stand, 3.2), `downed`, `standing`.
- S4. The sim's default encounter list skips encounters with `stay_behind` or `rooms` and ids starting `kit_`
  (they are not balance fights); named encounters still run.

### 3.1 The hold-a-tile escape (`mode: "hold"`; the Crucible, Q-75a decided (a))

The player picks, before the fight, who holds the winch tile while the others escape (user 2026-10-25, Q-75a (a)).
The theme (user): the first loss is a choice; as losses pile up they become the fate of the downed.

- H1. The pick: before round 1 is planned, the battle screen asks "Who holds `{tiles_label}`?" over the deployed
  heroes (the board visible); the pick is a host request `stay_pick {unit}`, recorded for replays, allowed once,
  only while round 1 is in planning and nothing is confirmed. Planning cannot be confirmed without it. Sims and
  autoplay pick by `pick_default` (`highest_hp` default, or `first`). Encounter data never names a hero.
- H2. The gate: `open` while the holder is active and on a hold tile (positions at that moment). With `fallback:
  "any"` (default) and the holder downed, any active hero on a hold tile opens it; `"none"` keeps it shut.
- H3. A step onto an exit while the gate is shut fails like a blocked tile (the unit stops before it,
  `World::move_unit`). Planning still accepts Withdraw (the holder may reach the tile in the same round); the plan
  line warns "The gate is shut until `{holder}` holds `{tiles_label}`." The holder's own Withdraw is refused
  ("`{holder}` holds `{tiles_label}`.").
- H4. Won, at a block end: the gate is open, at least `objective_count` heroes have escaped, and every other hero is
  escaped, downed or holding. `stayed` = the heroes holding at that moment. Rules before it are unchanged (a dead
  protected guest, every hero dead); "no active foe" still wins first, with `stayed` empty (use S1 to prevent it).
- H5. Lost by S2 when `timeout_outcome` is `lost` (the Crucible: nobody avoids the choice by waiting).
- H6. Party AI (sim, autoplay, the Balanced draft): the holder drafts a route to the nearest free hold tile, then
  Defend while on it (under the timeline ruleset: the Guard intent with the hold tile as its target, RULESET1_INTENT
  GD1); the others draft Withdraw once the gate is open or the holder is within its move of a tile; otherwise their
  preset as today.
  As built (ENCOUNTER2-HA, 2026-10-29): the AI reads the "keeper", the holder while it is on the field; with the
  holder down and `fallback` any, the hero already on a hold tile, else the one nearest to a hold tile (party order on
  a tie) takes its part (with `fallback` none nobody does: everyone plays its preset). A keeper beyond one move
  walks as far as the round's move reaches, then Defends (step AI) or takes the advance with Guard (round intent); on
  its tile with no Guard offered it Rests and never leaves it. A hold tile taken by a foe gives no part (the preset
  fights for it). S4 applies to the catalog sweep `enc=all` (the sim's default `enc` is the generated fights).
- H7. Screen: hold tiles drawn with the objective ring and a winch marker; the gate's exits show "shut" (red) or
  "open" (green); the objective line adds "`{holder}` holds `{tiles_label}`."

### 3.2 The impossible fight and the last stand (`mode: "stand"`; a later mission, not the Crucible)

- L1. Signals before it starts (planner note): the briefing line (`objective_text`, or the generated "You cannot win
  here. Get `{count}` out by the exit."), a "No victory" marker on the objective pill, and the `unkillable` boss's tell
  carries the badge "Unstoppable" (RULESET1 owns the tell layout, which now leads with the round intent); inspect
  shows "Cannot be defeated here."
- L2. Exits: `exits_open` `always` (default: a clean retreat with nobody down leaves nobody behind) or `after_stand`.
- L3. The stand: at the end of the block in which the first hero falls (first by the HP journal's order), that hero
  rises at `max(1, roundi(max_hp × stand_hp_pct / 100))` HP, statuses cleared as on death, then `stand_statuses`
  granted (default `[{"status": "taunt", "rounds": 5}]`). Once per battle. The stander's Withdraw is refused.
- L4. Payoff (planner note): an event cell and banner with `stand_line` (default "`{name}` stands alone."), phases
  with the trigger `{"kind": "stand"}` fire at the next block start (lines, ai, status, enrage on the boss); the result
  reports `stood`, and `flag` names the stander for the world (a keepsake, a memorial).
- L5. Won when at least `objective_count` heroes escaped and every other hero is escaped, downed or the stander.
  If the stander falls again they stay downed and are still reported `stood`.
- L6. Data guidance (not a rule): at most one impossible fight per act.

### 3.3 What the world reads `[kit hook]` (world layer)

- W1. `Tally` gains `party: Vec<(member id, fate)>` from S3; `after_fight` results carry `party_fates`.
- W2. `stay_behind.flag` (a flag key): set to the member id that stayed or stood ("" when none) on every outcome.
- W3. `stay_behind.then`: `none` (default), `capture` or `kill` (ROSTER1 3.2, ROSTER1_TASKS 4): `capture` and
  `kill` call `roster::depart(member, captured | dead, cause stayed | stood)`. No `party.away` list and no save step
  of its own: departed members live in `party.departed` (ROSTER1-L). Heroes who did not stay settle by
  `roster::settle_party` with the act's rule row (Act 1: wounded).
- W4. The Results popup adds one line per stayed or stood hero ("Kaelen held the winch.", from the encounter's
  `result_line`, default "`{name}` stayed behind.").

## 4. Phase hooks for node 20 `[kit hook]`

Node 20 (CANON 3.6, "Nemeth's fight, revised"): the corrupted founder fights at Nemeth's side; on the break the
founder kills Nemeth and his guard, then fights the cell alone as a monstrosity. All four hooks are phase data.

- P1. `kill` effect `{"type": "kill", "units": [refs], "by": ref, "within": 0}`: at fire time every active, placed,
  unreserved unit of the refs drops to 0 HP through `World::damage` (journaled); `within` > 0 limits it to units at
  most that many tiles from the `by` unit. Fates are Killed (REWARD1 2.2: any killer), so key loot such as Nemeth's
  Warded Cassock pays on a win; the dead keep their tiles. One event cell: actor `by` (else the encounter), text
  "`{names}` fall." Morale counts the deaths as any death. Refs: enemies, phase-wave refs or guests; never a hero.
- P2. Phase `group` (string, optional): when a phase fires, every unfired phase with the same group drops. Node 20's
  break is two phases in one group: `hp_below` on the founder with `or_round`, and `hp_below` on Nemeth.
- P3. `swap` effect `{"type": "swap", "unit": ref, "into": ref, "hp_pct": 100}`: `into` spawns at battle start
  reserved (as phase waves do, counted in the 12); on fire the first active unit of `unit` leaves the grid with the
  new fate Transformed (pays nothing), the `into` unit takes its tile at `hp_pct` of its max HP and forms the morale
  group `phase:{id}`; a `slay` objective on `unit` moves to `into`. Footprint stays 1 cell (Q-79 is art only).
- P4. `side` effect `{"type": "side", "unit": ref, "to": "rogue"}`: the unit leaves its side for a third list
  `rogues`, hostile to both sides; rogues are friends of each other. `World` keeps precomputed foe lists, so with no
  rogue every `foes_of` slice is today's. Rout needs no active enemy and no active rogue; a rogue leaves its morale
  group; tells show rogues in their own colour. Node 20 needs it only if `within` leaves survivors.
- P5. Triggers: an `hp_below` unit may be an enemy, a phase-wave ref or a swap `into` ref.

As built (ENCOUNTER2-K, 2026-10-29): P1, P2 and P5 (the swap `into` part comes with X). A kill also takes an
`unkillable` unit down (user-confirmed 2026-10-29, decision "ENCOUNTER2-HA and -K answers"; one switch,
`encounter::KILL_OVERRIDES_UNKILLABLE`, copied to `Session::kill_past_unkillable`; off: the unit keeps 1 HP and its
SPARED cell follows the kill's); it credits no unit (the metrics count the HP as the struck units' `scripted` loss,
`BattleMetrics::scripted`, part of the HP balance) and raises no HOOKS1-E event (user-confirmed: `kill` and
`ally_downed` come only from actions). The kill's cell (subtype `Kill`) follows the phase's own cell; it reads
"`{name}` falls." for one unit, "A, B and C fall." for more; an effect that takes no HP logs nothing. `by` resolves
to the first placed, unreserved unit of its ref and `within` measures from it; with no such unit (user-confirmed) a
`within` > 0 kill takes nobody; the checks add "a kill within tiles needs a by unit", an unknown ref error and the
`by` ref error. When two phases of a group are due at once, the first in list order fires. Groups live in `Phases`
(not `PhaseDef`).

## 5. Connected rooms (Q-60) `[kit hook]`, later than the slice

Room uses the user named (2026-10-25, Q-22): the hub siege (each facility a room; a lost room costs its facility a
level), floors to climb, enemy waves or packs to defeat, and rooms to clear. The room core (R) carries all four; each
is encounter data (objectives and arrivals per room), not a new mode.

### 5.1 Authored terrain (slice) and the room data

- R0. `rows` (encounter field, optional): 9 strings of 12 glyphs from the terrain glyph set (`grid.rs:65`); when set
  they replace the zone's rows; `zone` still picks the backdrop and tiles. The slice's arenas use it.
- R1. `rooms` (optional, 2-4 entries): each a board `{id, name, zone, rows, open, exits, doors, party, enemies,
  guard}`. Absent or empty = today's single board from the top-level fields (byte-identical). With rooms, the
  top-level `zone` is the default for rooms without one and the top-level `exits`, `rows` and `arena` are ignored.
- R2. A door `{id, at: [x, y], to: room id, entry: [[x, y], ...]}` sits in its room; a passage both ways is two doors.
  `exits` stay the ways out of the battle (escape counts only them).

### 5.2 Rules

- R3. One timeline: every room resolves in the same rounds and blocks, the resolver's order unchanged.
- R4. Start: the party deploys in the room with `party: true` (exactly one) by today's deployment; each room's
  `enemies` (indices into `enemies`, as `morale_groups`) deploy there; unlisted enemies go to the party's room.
- R5. A door step (planner mode Door, like Withdraw): route to the door tile, then `transit_blocks` (encounter field,
  0-2, default 1) off the board (not targetable, still active); the unit appears at the next block start on the first
  free entry tile in list order; all taken: it waits a block; still blocked at the round end: the free walkable tile
  nearest the first entry tile. No attack, area or aura crosses rooms.
- R6. Enemy AI: a unit with an active foe in its room plays today's AI there. Otherwise, unless `guard` (room field,
  or the phase `ai` key `roam: "guard"`), it takes the door on the shortest room path to the nearest room holding an
  active foe (ties in door order). Party presets do the same for heroes.
- R7. Reinforcements: `wave` and arrival effects gain `room` (default the party's room) and `entry` tiles (default
  that room's enemy half, as today).
- R8. Objectives and the end are per encounter: rout counts every room; escape counts exits in any room; hold and
  stay-behind tiles name their room (`objective_room`, `stay_behind.room`). Phase trigger `enter_room` (first hero
  arrives) is reserved for a later task.
- R9. Screen: one room at a time at the STAGE1 size; room tabs above the board (name, counts per side, a red dot when
  a hero is threatened there, a badge when an arrival is due); planning follows the selected hero's room, playback
  the room of the resolving action (the tab switches). Strip rows of units in other rooms are dimmed with the room's
  letter; impact badges for threats in other rooms carry the letter (strip layout: RULESET1). A room minimap in the
  intel panel is later.
- R10. Host state JSON: each unit gains `room` and the grid becomes `rooms: [...]` only when the encounter has rooms;
  requests with a cell take an optional `room` (default the actor's). Single-board JSON is unchanged.
- R11. Implementation note (builder's choice, invariants binding): rooms as far-apart rectangles of one `Grid`
  (room `i` at x offset `i × 64`), so `V2` stays as it is, cells are unique, no range spans a gap, paths cannot cross,
  and room 0 at offset 0 is today's grid; room-local helpers replace `GRID_W` uses (`centre_distance`,
  `is_party_half`, deployment columns, host and GUI drawing).

## 6. Schema contract (`content.encounters` and `world.encounters`; schema-first: yes)

| Key | Type | Default | Validation (error text) |
| --- | --- | --- | --- |
| `rows` | json | [] | empty, or 9 strings of 12 known glyphs ("Rows are 9 strings of 12 terrain glyphs.") |
| `unkillable` | refs | [] | each in `enemies` or a phase-wave ref ("Unkillable unit '`{r}`' is not in this encounter.") |
| `timeout_outcome` | enum | "fled" | `fled`, `lost` |
| `stay_behind` | json | {} | empty, or the block below; only with `objective` `escape` ("Stay-behind needs the escape objective.") |
| `transit_blocks` | int | 1 | 0-2; only read with `rooms` |
| `rooms` | json | [] | empty, or 2-4 rooms below |
| `objective_room` | string | "" | a room id when `rooms` is set; "" = the party's room |

`stay_behind` keys:

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `mode` | enum | "hold" | `hold`, `stand` |
| `tiles` | [[x, y]] | [] | hold: 1-2 tiles, walkable, not an exit or door, on the board ("Hold tile `{c}` is off the grid, blocked or an exit."); stand: must be empty |
| `tiles_label` | string | "the line" | 1-24 characters ("the winch") |
| `room` | string | "" | a room id when `rooms` is set |
| `fallback` | enum | "any" | `any`, `none` (hold only) |
| `pick_default` | enum | "highest_hp" | `highest_hp`, `first` (hold only) |
| `exits_open` | enum | "always" | `always`, `after_stand` (stand only) |
| `stand_hp_pct` | int | 30 | 1-100 (stand only) |
| `stand_statuses` | json | [{"status": "taunt", "rounds": 5}] | statuses from the table, rounds 1-10, at most 3 |
| `stand_line`, `result_line` | string | see 3.2, 3.3 | at most 160 characters; `{name}` the only token |
| `flag` | string | "" | a flag key (world encounters only) |
| `then` | enum | "none" | `none`, `capture`, `kill` (world encounters only; ROSTER1 3.2) |

Room keys: `id` (`[a-z][a-z0-9_]{1,23}`, unique), `name` (1-16 characters, the tab), `zone` (enum as today, default
the top-level zone), `rows` (as above), `open` (bool, false), `exits` ([[x, y]], may be empty), `doors` (0-4),
`party` (bool; exactly one room true), `enemies` (indices into `enemies`, each in at most one room), `guard` (bool,
false). Door keys: `id` (unique in the encounter), `at` (walkable, not an exit), `to` (another room's id), `entry`
(1-4 tiles in the target room: walkable, not an exit or door). Errors also: "Room `{id}` has no way in or out." when no
door leads to or from it; "`{n}` rooms: at most 4."

Phase effects (inside `phases`, schema unchanged; checks in `check/encounter.rs`): `kill` keys `type, units, by,
within` (units 1-6 refs, `by` a ref or "", `within` 0-11; "Kill names hero '`{r}`'.", "Kill names the protected guest
'`{r}`'."); `swap` keys `type, unit, into, hp_pct` (`into` resolves to a unit and is in no other list, `hp_pct`
1-100); `side` keys `type, unit, to` (`to` = `rogue`); `wave` gains `room`, `entry`; `ai` gains `roam` (`pursue`,
`guard`). Phase key `group` (`[a-z0-9_]{1,24}`); trigger kinds gain `stand` (only with `stay_behind.mode` `stand`).
At most one swap and one side effect per unit ref. The 12-unit cap counts swap `into` units.

HOOKS1 encounter fields (cross-reference, HOOKS1-D 2026-10-31; the schema is HOOKS1_TASKS 1.2 and 1.6): `arena_hazards`
(0-4 scheduled strikes, rule `arena_hazards`, HOOKS1 7), the phase effect `deck` (rule `deck`, HOOKS1 10.1), the
unit and phase `ai` key `tactics` (HOOKS1 11) and a data `zones` row as the `zone` (HOOKS1 13); later `spare_once`,
the phase trigger `spared` and the phase effect `heal` (HOOKS1 10.2, HOOKS1-Q).

Content tool path: `tools/fc.sh content upsert encounters @file.json` (catalog) or with `world=<id>` (world
encounters), or a seed through `tools/fc.sh content apply`; then `tools/fc.sh content check`. The Content Forge edits
the new json fields as raw JSON (no form work in this spec).

## 7. Out of scope

RULESET1's rules (windups, enemy recovery, round intents and follow-ups, poise, armour, reach-2, deck slots, the
intent tells); the unit lifecycle hook (captured, dead, retired: ROSTER1); multi-cell units (Q-79 art only); line of
sight across rooms; more than 12 enemy-side units or 4 rooms; a dropped-item marker on the field; the sunk_ward world
itself (the slice's data tasks fill the Crucible from the sketch in the task file); the Content Forge forms for the
new fields.
