# ENCOUNTER2 tasks: tests, parity plan, task split, data, open questions

Status: proposed (2026-10-25); the rules are in [ENCOUNTER2.md](ENCOUNTER2.md) (section and rule ids such as H3 refer
to it)

## 1. Tests (rule-derived; scratch data folders, never the shipped data)

| Task | Test | Expectation from |
| --- | --- | --- |
| T | an encounter with `rows`; one with 8 rows or a bad glyph | terrain equals the rows; one error each (R0) |
| H | round 1 confirm before `stay_pick` | refused; after the pick, accepted; a second pick refused (H1) |
| H | a hero steps onto an exit with the holder off the tile | the step fails, the unit stops before the exit (H3) |
| H | the holder plans Withdraw | refused with the H3 text |
| H | holder on the tile, the other three withdraw | won at that block end, `stayed` = [holder], others `escaped` (H4) |
| H | holder downed, `fallback` any, a second hero takes the tile | gate open again; `stayed` = [second hero] (H2) |
| H | the same with `fallback` none | the gate stays shut; no exit step succeeds |
| H | last round ends with no verdict, `timeout_outcome` lost / default | lost / fled (S2) |
| H | an `unkillable` unit takes lethal damage twice | HP 1, one event cell (S1) |
| H | every classic catalog encounter run to its end | the same outcome as before the change (S2 refactor) |
| HA | `sim` with defaults on a data folder holding `kit_*` and `stay_behind` encounters | they are skipped; named they run (S4) |
| HA | autoplay on a hold encounter | the holder reaches the tile and defends; the others withdraw once it is open (H6) |
| W | a won hold battle with `flag` and `then: leave_party` | the flag holds the member id; the member is in `party.away`, not in `active_ids`; a save round trip keeps it (W2, W3) |
| K | `kill` with two refs | every placed unit of them at HP 0 through the journal; fates Killed; one event cell (P1) |
| K | `kill` with `within` 2 | only units within 2 tiles of `by` die |
| K | two phases in one `group`, the second's trigger met first | it fires; the first drops (P2) |
| K | checks: kill naming a hero, a protected guest, 7 refs | one error each |
| K | key loot on a ref killed by a kill effect | paid on a win (REWARD1 unchanged) |
| X | `swap` | `into` on the old tile at `hp_pct`; old fate Transformed pays nothing; slay retargets; `hp_below` on `into` fires (P3, P5) |
| F | `side` rogue with one enemy left | that enemy and the rogue fight each other; rout needs both out (P4) |
| F | no rogue in a fight | `foes_of` slices equal today's (trace_parity also proves it) |
| L | the first hero falls | rises at `stand_hp_pct` with the stand statuses at the block end; a second fall stays down; `stood` reported (L3-L5) |
| L | `exits_open` after_stand | exit steps fail before the stand, succeed after |
| R | an encounter without `rooms` | host state JSON byte-equal to before (R10) |
| R | two rooms, a door step with `transit_blocks` 1 | off the board for one block, then on the first free entry tile; all taken: waits (R5) |
| R | a ranged unit with range 6 next to a door | cannot target a unit in the other room (R5) |
| R | rout with a foe left in another room | not won until it is out (R8) |
| RA | an enemy with no foe in its room, `guard` false / true | takes the door toward the nearest foe / stays (R6) |
| D | `tools/fc.sh content check` | clean |

## 2. Parity plan (the Ashlands "classic" stays byte-identical)

- Every new field defaults to off; with the defaults the code takes today's paths. No recorded trace, golden or
  expectation is edited; no parity check is retired.
- `trace_parity`, `world_traces`, `journey`, `phasefold` (golden), `saves`, `checks_parity`, `sim_parity` and the
  host `replay` run unchanged after each task. The risky refactors land first and alone: S2 (the timeout verdict
  moves into the session) in H, the precomputed foe lists in F, the room-rectangle grid (R11) as R's first commit.
- `data_source` compares Godot's schema whole: T and H strip their new encounter keys (`rows`, `unkillable`,
  `timeout_outcome`, `stay_behind`; R adds `transit_blocks`, `rooms`, `objective_room`) through `AFTER_GODOT`,
  named in the commit and a HISTORY row each.
- Godot data holds no new effect, trigger or field, so the content checks' reports stay the same.
- Saves: a save without `party.away` reads it empty; if W needs a format step, it is a new version with a migration
  test, and every Godot save still loads.

## 3. Tasks (split by layer and lane; slice = needed before the vertical slice)

| Id | Title | Layer | Lane | Depends | Slice | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| ENCOUNTER2-T | Authored terrain rows on encounters (R0) | kit | logic | - | yes | `emberwake_battle/src/{grid,session}.rs`, `emberwake_content/src/check/encounter.rs`, `schema.json`, `emberwake_world/tests/data_source.rs` |
| ENCOUNTER2-H | Hold-a-tile escape, `unkillable`, timeout verdict in the session, party fates, `stay_pick` request (S1-S3, H1-H5) | kit | logic | ENCOUNTER2-T (same files) | yes | `emberwake_battle/src/{encounter,session,world,plan,log}.rs`, `emberwake_battle/tests/stay_behind.rs` (new), `emberwake_host/src/{lib,cli}.rs`, `emberwake_battle/src/sim.rs`, `check/encounter.rs`, `schema.json`, `data_source.rs` |
| ENCOUNTER2-HA | Party AI for hold escapes and the sim's default list (H6, S4) | kit | logic | ENCOUNTER2-H | yes | `emberwake_battle/src/{ai,session}.rs`, `emberwake_sim/src/args.rs` |
| ENCOUNTER2-W | The world reads stay-behind results (W1-W4) | kit | logic | ENCOUNTER2-H | yes | `emberwake_world/src/{battle,state,save,explore,dialogue,travel}.rs`, `emberwake_world/tests/rewards.rs`, `emberwake_gui/src/{battle,hub}.rs` (settle, results line) |
| ENCOUNTER2-U | Hold screen: the pick panel, gate and tile marks, objective line (H1, H7) | game | logic | ENCOUNTER2-H | yes | `emberwake_gui/src/battle.rs` (after STAGE1-L if it is in flight) |
| ENCOUNTER2-K | Phase `kill` effect and phase groups (P1, P2, P5) | kit | logic | ENCOUNTER2-H (same files) | yes (M2 Culling Timer) | `emberwake_battle/src/{encounter,session,log}.rs`, `check/encounter.rs` |
| ENCOUNTER2-D | Kit samples `kit_winch_escape`, `kit_culling` (section 4.1) | game | asset | ENCOUNTER2-T, -H, -K | yes | `games/emberwake/data/catalog/encounters.json` |
| ENCOUNTER2-L | Impossible fight and last stand (3.2) | kit | logic | ENCOUNTER2-HA | later | as H, plus `intent.rs`, `emberwake_gui/src/battle.rs` (L1 marks) |
| ENCOUNTER2-X | Phase `swap` and the fate Transformed (P3) | kit | logic | ENCOUNTER2-K | later (Act 3) | `emberwake_battle/src/{encounter,session}.rs`, `emberwake_world/src/battle.rs`, `check/encounter.rs` |
| ENCOUNTER2-F | Phase `side`: hostile to all (P4) | kit | logic | ENCOUNTER2-K | later (Act 3) | `emberwake_battle/src/{world,ai,session,intent}.rs`, `emberwake_gui/src/battle.rs` (rogue colour) |
| ENCOUNTER2-R | Rooms core: data, grid rooms, doors, transit, deployment, verdict, host JSON (R1-R5, R7, R8, R10, R11) | kit | logic | ENCOUNTER2-T | later | `emberwake_battle/src/{grid,world,session,plan,path,encounter}.rs` (maybe `room.rs`), `check/encounter.rs`, `schema.json`, `emberwake_host/src/lib.rs` |
| ENCOUNTER2-RA | Room AI: pursue, guard, doors in party presets (R6) | kit | logic | ENCOUNTER2-R | later | `emberwake_battle/src/ai.rs` |
| ENCOUNTER2-RU | Room screen: tabs, follow, dimmed strip rows, letters on badges (R9) | game | logic | ENCOUNTER2-R, STAGE1-L, RULESET1's strip task | later | `emberwake_gui/src/battle.rs` |
| ENCOUNTER2-D2 | Later samples `kit_last_stand`, `kit_betrayal`, `kit_two_rooms` | game | asset | ENCOUNTER2-L, -X, -F, -RA | later | `data/catalog/encounters.json` |

T, H and K share `session.rs`, `encounter.rs` and `check/encounter.rs`: run them in a row (T, H, K). Rooms are later:
no slice arena needs them if Act 1 arenas fit 12 x 9 (Q-60 (a)); the Crucible fits one board (section 4.2; EQ4).

## 4. Data (asset lane, `tools/fc.sh content upsert encounters @file.json`, then `content check`)

### 4.1 Kit samples (ENCOUNTER2-D; ids `kit_*` keep them out of default sims, S4)

Shared rows for `kit_winch_escape` (the Crucible's board):

```text
y0 ############    y3 #.........L.    y6 #..r..#....#
y1 #.....#....#    y4 #...........    y7 #.....#..o.#
y2 #..r..#....#    y5 #.........L.    y8 ############
```

| Key | `kit_winch_escape` | `kit_culling` |
| --- | --- | --- |
| `name` | Winch Escape (kit sample) | Culling Timer (kit sample) |
| `zone`, `rows` | `ruined_courtyard`, the rows above | `stone_arena`, none |
| `exits` | `[[11, 3], [11, 4], [11, 5]]` | `[[11, 4]]` |
| `max_rounds`, `rng_seed` | 8, 1337 | 6, 1337 |
| `party` | warrior, ranger, rogue, priest | warrior, ranger |
| `enemies` | goblin_warlord, goblin_brawler x2, goblin_archer x2 | goblin_berserker, goblin_brawler, goblin_shaman |
| `guests`, `protect` | none | dojo_warrior, dojo_ranger (the captives); none protected |
| `objective`, `objective_count` | escape, 3 | rout, 1 |
| `objective_text` | "Open the gate: one holds the winch, three get out." | "Free the captives before they are culled." |
| `unkillable`, `timeout_outcome` | [goblin_warlord], lost | [], fled |
| `stay_behind` | mode hold, tiles `[[10, 1]]`, tiles_label "the winch", fallback any | {} |
| `phases` | none | `cull_1`: round 3, kill [dojo_warrior] by goblin_berserker, line "The first captive falls."; `cull_2`: round 5, kill [dojo_ranger], line "Another falls." |

### 4.2 The Crucible (sketch for the slice's sunk_ward data task; not this spec's asset task)

World encounter `crucible_slag_gate`: the board, exits, objective, `unkillable` and `timeout_outcome` of
`kit_winch_escape`; party = the four deployed founders; enemies Nemeth, 2 Inquisitorial Purifiers, 2 Castellan
Footmen, 2 Watch Crossbowmen (ids from the cast registry's roster task); `unkillable` [Nemeth's ref]; `stay_behind`
adds `flag` `crucible_stayed`, `then` leave_party, `result_line` "`{name}` held the winch."; `max_rounds` 8;
numbers (levels, waves) from the balance lane.

### 4.3 Node 20 (sketch, Act 3; one variant per founder, EQ7)

`ossuary_<founder>`: enemies Nemeth, 2 guards, the corrupted founder; phases `break` (group `break`, `hp_below`
corrupted founder 50 with `or_round` 4: line, kill [Nemeth, guard] by the founder, swap founder into
`monstrosity_<founder>` at 100) and `break_early` (group `break`, `hp_below` Nemeth 40, the same effects), `frenzy`
(`hp_below` the monstrosity 30: enrage 20); `key_loot` the Warded Cassock on Nemeth; objective rout.

## 5. Docs delta (for the planner)

- `games/emberwake/docs/GAME_BRIEF.md` encounter section: stay-behind, `unkillable`, `timeout_outcome`, the new
  phase effects and groups, `rows`, rooms (each when its task lands).
- `docs/STRUCTURE.md`: `emberwake_battle` row (rooms, maybe `room.rs`), `emberwake_world` row (`party.away`).
- `engine/docs/authoring/encounters.md` (new, game-agnostic): stay-behind and phase scripting as data, rooms; the
  promote-generic-methods practice.
- CANON 2.1 rows Arenas and Stay-behind point to this spec; `decisions/lore.md` gets the Q-75a row (planner).
- ROADMAP row wanted: ENCOUNTER2, proposed, no dependency: encounter structures (stay-behind objectives: the hold
  escape now, the last stand later; phase kill, swap, side and groups; authored rows; connected rooms later); spec
  `docs/specs/ENCOUNTER2.md`.

## 6. Open questions (recommended option first; Q-75a is decided: (a), section 3.1)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| EQ1 | Where the holder is picked | (a) a pick panel on the battle screen before round 1, board visible (H1); (b) a dialogue choice per founder before the battle (needs a "member deployed" choice condition and a flag the battle reads) | (a): no per-founder data, the player sees the winch |
| EQ2 | A founder downed (not the holder) when the others escape | (a) carried out, wounded (Act 1 rule); (b) left behind and captured too | (a): the Crucible's one loss stays the chosen one |
| EQ3 | How Nemeth survives the Crucible | (a) `unkillable` (stays at 1 HP); (b) he withdraws at low HP (a new phase `withdraw` effect) | (a): one rule, also serves Q-74; (b) reads better if playtest finds 1 HP odd |
| EQ4 | Rooms in the slice | (a) no: the Crucible fits one 12 x 9 board (4.1); (b) the watchpost yard and the Slag Gate as two rooms | (a): rooms stay later, the slice keeps its scope |
| EQ5 | Door transit cost | (a) 1 block off the board; (b) 0 (a door is one tile step) | (a): doors are a commitment the other room can answer |
| EQ6 | Room screen | (a) tabs now, a minimap later; (b) both at once | (a) |
| EQ7 | Node 20 per founder | (a) six encounter variants chosen by `crucible_stayed` (data only); (b) a roster ref that reads a flag (kit hook) | (a) until a second story needs (b) |
| EQ8 | M2's captives with the 2-guest cap (`encounter.rs:20`) | (a) two captives, culled at rounds 3 and 5 (data only); (b) raise the cap to 4 (party panel work) | (a) for the slice |
| EQ9 | Units the founder kills: rewards | (a) as killed, any killer (REWARD1 2.2); (b) a `rewards: false` key on `kill` | (a): one rule |

## 7. Canon versus code (found while surveying)

1. CANON 2.1 "escape and hold exist": true, but the last-round "fled" verdict lives in the GUI, host and sim, not the
   session (`gui/battle.rs:716`, `host/lib.rs:203`, `sim.rs:208`), and the result names no hero (S2, S3).
2. CANON 3.6 "or earlier if Nemeth falls low": a phase has one trigger, and the checks reject an `hp_below` unit
   outside `enemies` (`check/encounter.rs:301`), so the monstrosity cannot be a trigger unit (P2, P5).
3. CANON 3.6 "the Cassock drops where he dies": loot is paid at settle, on a win only (key loot); no item lies on the
   field, and fleeing after the break pays nothing.
4. ENC-EVT-02's Culling Timer meets the 2-guest cap and the 4-phase cap (EQ8).
5. CANON 2.4 "captured" exists in no crate; W3's `party.away` is the stand-in.
6. "The Crucible requires four deployed" (CANON 3.2): no encounter field sets a minimum party; `PARTY_MAX` 4 only
   caps it (a small follow-up for the slice data task).
7. Zone maps are code (decision S12) while the slice is data only: R0 resolves it.
