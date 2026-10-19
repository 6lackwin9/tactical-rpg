# OBJ1: Battle objectives and deployment variants (the centre ring returns)

Status: proposed (spec 2026-10-18; approved direction: decision world-campaign "DR1 design review answers (user 2026-10-18)" (all recommendations), review `docs/specs/DESIGN_REVIEW1.md` 4.1 and 5.5)
Depends on: BOSS1-L (shares the encounter and session files and reuses `is_reserved`); RECRUIT1-A before OBJ1-A (same encounter rows)
Lane: both (OBJ1-L schema-first, then OBJ1-A data, then OBJ1-B balance probe; OBJ1-U on the UI/UX thread after COMBAT-UI2-L1)
Touches: L: new `scripts/combat/session/encounter_objectives.gd` (parse, validate, verdict), `scripts/combat/session/{combat_encounter,catalog_encounter,combat_session}.gd`, `scripts/exploration/{game_encounter,battle_bridge}.gd`, `scripts/combat/arena/{combat_grid,block_snapshot}.gd` (deployment table, `objective_state`), `scripts/content/{content_schema,content_catalog}.gd`, `scripts/world/{world_schema,world_catalog}.gd`, `scripts/playtest/playtest_record.gd` (end result from the verdict), `tools/sim/sim_runner.gd` (a win is `outcome == "won"`), `scripts/ui/world/world_preview.gd`, new `scripts/tests/objective_test_suite.gd`. U: `scripts/ui/combat/combat_grid_view.gd` (ring and tile markers), planning header, `scripts/ui/combat/combat_enemy_card_panel.gd (was unit_inspector_ui.gd, COMBAT-UI3)` (guest badge), `scripts/ui/menus/battle_results_model.gd` (objective line), `combat_ui_framework_test_suite`. A: `data/worlds/ashlands/{encounters,characters,routes}.json`, `data/worlds/iron_reaches/routes.json`, one seed `data/seeds/obj1.gd`

## Goal

Not every fight is "kill everything". An encounter can say what wins it (rout, hold the ring, escape, slay the chief), whom the party must keep alive (a guest), and how the two sides start (side against side, ambushed from behind, or holding the centre). The damage formula, the AI and the 12x9 grid do not change. In the Ashlands three fights use it: the Courtyard is held on the ring, the steel wagon is ambushed with its carter to protect, and the finale ends when Grakka falls.

## What exists (cited)

- Encounter fields: zone, arena CLOSED or OPEN, exits (default [[11, 4]]), max_rounds, party, enemies, the wave fields (`content_schema.gd`, `world_schema.gd` `_encounter_fields`); validators `ContentCatalog._encounter_errors` and `WorldCatalog._encounter_errors`.
- Deployment (GRID12): party front x 5, back x 4, enemies front x 7, back x 8, rows 4, 3, 5, 2, 6, 1, 7; leftovers and waves take the free tile nearest the arena middle on their own half (`combat_grid.gd` `deploy_combatants`, `catalog_encounter.gd` `_place_leftovers`). `CombatGrid.centre_distance` treats the middle as between columns 5 and 6 on row 4.
- End of battle: `resolver.is_combat_resolved` (no active ally or no active enemy); `is_active` is alive and not withdrawn; a unit on an exit tile withdraws. `BattleBridge.outcome`: won (over, a party unit stands), lost (over, every party unit down), fled (over with the party withdrawn alive, or the round limit with both sides standing, decision X1).
- `GameEncounter.build` replaces the authored `party` with the saved active members; the authored `party` list only serves the Forge and tests.
- `BattleBridge.apply` gives XP for every listed enemy and wave unit on `won`; lost wakes the party (no further cost, DR1 (2)); fled changes nothing.

## How this avoids the dropped 9x9 centre ring

HISTORY "9x9 square combat grid with a marked centre ring" (GRID12) was dropped because a square grid was not needed, a standard encounter reads better side against side, and the centre tile needed no marker. OBJ1 keeps the 12x9 grid and the left-against-right standard deployment for every encounter that does not ask otherwise. The ring is not a grid feature: it is the default tile list of a `hold` objective, drawn only in a battle that has that objective, as an objective marker. The region-map `overlay_poi` icon keeps its job.

## 1. Schema contract (schema-first: yes)

### 1.1 New encounter fields (catalog `encounters` and world `encounters`)

| Field | Type | Default | Allowed / validation (message) |
| --- | --- | --- | --- |
| objective | enum | rout | rout, hold, escape, slay |
| objective_tiles | json | [] | [[x, y], ...] on the grid, walkable in the zone, not exit tiles ("Objective tile %s is off the grid, blocked or an exit."); hold only; empty = the ring [[5, 4], [6, 4]] |
| objective_rounds | int | 3 | 1..`max_rounds`; hold only ("Hold rounds must be 1..%d.") |
| objective_unit | string (roster ref) | "" | slay: required, an id in `enemies` ("Slay needs a unit from enemies.") |
| objective_count | int | 1 | escape: 1..4, heroes that must leave by an exit |
| objective_text | string | "" | 0..80 chars; empty = the generated text of rule 9 |
| deployment | enum | standard | standard, ambush, centre |
| guests | refs | [] | catalog: units with side party; world: characters with role party or npc whose base unit has side party; at most 2 ("An encounter holds at most 2 guests.") |
| protect | refs | [] | each id is also in `guests` ("Protect '%s' is not a guest.") |

Escape needs at least one exit tile (CLOSED arena) or an OPEN arena ("Escape needs an exit tile."). `party.size() <= 4` stays; guests are counted apart. The shared validator `EncounterObjectives.problems(entry, ref_ok: Callable) -> Array[String]` is called from both catalog validators.

### 1.2 Deployment table (`CombatGrid.DEPLOYMENTS`, data in code beside the GRID12 constants)

| deployment | party front, back | enemy group A front, back | enemy group B front, back | split |
| --- | --- | --- | --- | --- |
| standard | 5, 4 | 7, 8 | - | all enemies in A (today, unchanged) |
| ambush | 5, 4 | 7, 8 | 2, 1 | roster order: even index to A, odd to B |
| centre | 5, 6 | 8, 9 | 3, 2 | as ambush |

Within a group, melee takes the front column first and ranged the back column first (as `_deploy_side`); rows follow `DEPLOY_ROWS`. When both columns are full, a unit takes the next column away from the middle (A towards x 11, B towards x 0, party in centre outwards from x 5 and x 6). Guests deploy after the heroes with the same party rule. In ambush and centre, leftovers, the wave and BOSS1 phase waves alternate A, B, A... and take the free walkable, non-exit tile nearest (group front column, row 4) by grid distance, ties by `DEPLOY_ROWS` order. Front-to-front gaps stay 2 tiles, as in GRID12.

### 1.3 Runtime state

`EncounterObjectives` keeps `held` (int, rounds counted for hold). `BlockSnapshot.objective_state` copies it; `_restore_snapshot` restores it. `CombatSession.outcome: String` ("" while the battle goes on, then won, lost or fled) is set when the battle ends.

## 2. Rules (logic lane)

1. **Heroes and guests.** Heroes are the party units (the saved active members in a game). Guests are extra units on the party side, built from `guests` in `build()` after the heroes. Guests are allies for the planner, the AI and INTENT1 tells; they gain no XP, are not party members and are never written back.
2. **Who plans a guest (Q2 A).** The player plans guests like heroes. Autoplay drafts them with the Balanced tactic and never saves a tactic for them.
3. **Verdict.** After each block (after round-end effects, after BOSS1's `on_block_end`), `CombatEncounter.verdict(round, block, end_of_round) -> String` (new virtual; default = today's rule) returns "", won, lost or fled. The session ends the battle on a non-empty verdict and stores it in `outcome`; `BattleBridge.outcome` returns `session.outcome` when set, else today's round-limit rule (X1 unchanged). Checks, first match wins:
   - a. Every hero is down, or any `protect` guest is down: lost.
   - b. Every enemy that is not reserved (BOSS1 rule 4) is down or withdrawn: won (every objective can be won by a rout).
   - c. The objective's own rule (2.4 to 2.7).
   - d. No hero is active on the field and at least one withdrew (today's "party left"): fled.
4. **rout.** No extra rule: rule 3 b is today's win.
5. **hold.** Only who stands on the objective tiles at the end of each round (after the last block) matters (user 2026-10-18 02:10). If at least one active hero or guest stands on an objective tile and no active enemy does, `held` += 1. If at least one active enemy stands on one and no hero or guest does, `held` -= 1 (never below 0). If nobody stands on the ring, or both sides do, `held` does not change (both sides: planner default). Rounds count in total, not in a row. When `held` reaches `objective_rounds`: won. A hold win on the last round beats the round-limit fled.
6. **escape.** A hero or guest who steps on an exit tile withdraws (as today). Won when at least `objective_count` heroes have withdrawn, no hero is still active on the field, and every `protect` guest has withdrawn. If heroes are left down on the field the escape still counts (they wake with the party).
7. **slay.** Won when the objective unit (`CatalogEncounter.unit(id, 0)`) is down. If it withdraws through an exit: fled ("%s got away.").
8. **Rewards.** A won objective gives the full encounter XP (every listed enemy and wave unit, as today) and the quest hook; nothing else changes. Lost and fled cost only time and the objective (DR1 (2)).
9. **Generated text.** Shown in the round 1 intro and the planning header when `objective_text` is empty: rout "Defeat every enemy."; hold "Hold the ring for %d rounds, or defeat every enemy."; escape "Get %d out by the exit, or defeat every enemy."; slay "Defeat %s."; protect adds " Keep %s alive." (names joined with "and").
10. **No AI change.** `TacticalAI`, `PlanPresets` and AiProfile know nothing of objectives; the deployment and the tiles make the pressure. Enemies already step onto tiles in their way and flee by exits at their flee ratio.
11. **Sim and recordings.** The sim scores a win from `session.outcome == "won"` (a hold or escape win scores 1 like a KO win); the playtest record's end result comes from `outcome`. Combat results of recordings without objectives are unchanged.
12. **Viewers.** `world_preview.gd` shows `Objective: <text>` and `Deployment: <id>` lines.

Acceptance (tests, `objective_test_suite`):

- Every validation message in 1.1 fires for a bad entry; a full valid entry is clean in both catalogs; old entries (no new keys) normalise to rout, standard, no guests.
- Standard deployment places every unit on the same tiles as before (snapshot of today's positions for the shipped encounters).
- Ambush with 4 enemies: two on x 7 or 8, two on x 1 or 2; centre with a melee and a ranged hero puts them on (5, 4) and (6, 4).
- Hold 3: a hero on (5, 4) with no enemy on the ring for three round ends wins at the third; an enemy alone on (6, 4) at a round end lowers `held` by 1 (not below 0); an empty ring or both sides on it leaves `held` unchanged; standing on the ring mid-round but off it at the round end does not count; `reset_round` restores `held`.
- Escape 1: one hero on the exit with the others down is won; a protect guest left on the field is not.
- Slay: the unit down with others standing is won; the unit withdrawing is fled.
- A protect guest down with heroes standing is lost; guests get no XP and are not in `PartyState`.
- Every hero down while a guest stands is lost.
- The round limit with both sides standing stays fled (X1), except a hold win on that round end.
- A recorded rout battle replays identically before and after OBJ1-L.

## 3. UI hooks (OBJ1-U, UI/UX thread, layout C)

- Planning header: the objective text (rule 9) under the round title; hold adds "Held %d / %d".
- Grid: objective tiles drawn with a ring marker under the tokens (new asset key `overlay_objective`, placeholder via `tools/fc.sh export_placeholder`; until art exists `AssetRegistry` falls back to the `overlay_poi` texture); escape exits get the same marker in their colour; the slay target gets a crown chip; protect guests a shield chip.
- Hover card: guests show "Guest" and, if protected, "Must survive".
- Results screen: one line "Objective: held the Courtyard" style from the objective text; lost and fled keep today's text plus "The objective failed.".
- At 1280x720 in all four text sizes the header line and chips never cover the hover card.

## 4. Assets (OBJ1-A, after RECRUIT1-A)

Via `WorldCatalog.upsert`; numbers only from this table.

| Row | Change |
| --- | --- |
| courtyard_skirmish (ashlands) | objective hold, objective_tiles [] (the ring), objective_rounds 3, deployment standard (Q3 A), objective_text "Hold the Courtyard: keep the ring for 3 rounds, or drive them off." |
| wagon_raid (ashlands, new) | copy of ash_raiders; name "Raid on the Steel Wagon"; description "Raiders hit Orsa's wagon on the south road; Wenn must live to drive it home."; intro "The wagon lurches. Goblins come out of the ditch on both sides."; deployment ambush; guests [wenn_tallow]; protect [wenn_tallow]; objective rout; zone and the rest as ash_raiders |
| wenn_tallow (ashlands character, new) | name "Wenn Tallow", title "Orsa's Carter", role npc, base_unit ranger, faction "", home "", background "Orsa's nephew; drives the steel wagon and swears he can shoot." |
| south_road scripted `wagon_raid` (iron_reaches route) | encounter `ashlands:ash_raiders` becomes `ashlands:wagon_raid` |
| ashmaw_stand, ashmaw_stand_barred (ashlands) | objective slay, objective_unit grakka_ashmaw, objective_text "Break Grakka Ashmaw; the Horde scatters when he falls." |

Acceptance (asset, headless): `validate_all()` clean for both catalogs; the wagon raid starts with Wenn on the party side and two raiders behind the party; courtyard_skirmish shows the ring; a seeded ashmaw_stand win ends when Grakka falls with the wave still standing.

## 5. Balance follow-up (OBJ1-B, report only)

After OBJ1-A on main: solo `comp=warrior` vs courtyard_skirmish, the trio comps vs wagon_raid and both stands (with BOSS1-A if it landed). Targets: courtyard at the mob band (>= 90), wagon_raid at the line band (70-90), the stands at the boss band (30-50) (`docs/decisions/balance.md`). DR1 (3): light touch; if wagon_raid falls below 70 the asset fix is one raider fewer, not stats. World encounters and the sim: same note as BOSS1-B.

## Out of scope

Objective-aware AI or autoplay presets, timed objectives other than `max_rounds`, several objectives at once, objectives for enemies, moving or destructible objects (a guest is always a unit), high ground, new zones or map templates, injuries or any loss penalty (DR1 (2)), enemy morale and fleeing minions (MORALE1; slay ends the battle without them), the "surrounded on four sides" layout (centre covers both flanks), using `centre` in shipped data (available for ASHPACE1 and DESERT1).

## Overlap with other specs (files shared; run in sequence, not in parallel)

- BOSS1-L: `catalog_encounter.gd`, `combat_encounter.gd`, `combat_session.gd`, `block_snapshot.gd`, both schemas and catalogs, `battle_bridge.gd`, `world_preview.gd`. BOSS1-L first.
- INTENT1-L: `combat_session.gd`; guests get tells aimed at them like heroes (a protect fight is a reading fight).
- ITEMUSE1-L: `combat_session.gd`, `block_snapshot.gd`, `game_encounter.gd` (pouch write-back must skip guests), playtest recorder. Not in parallel.
- AUTO1: `combat_autoplay.gd` is not touched, but autoplay must draft guests (rule 2): check `CombatAutoplay` hero lists in review.
- RECRUIT1-L: `world_schema.gd`, `world_catalog.gd`; RECRUIT1-A: `courtyard_skirmish` row (solo Bryn), so OBJ1-A goes after it.
- CAMP1 data: the `south_road` scripted fight and both stands are CAMP1 rows; BOSS1-A edits the same stand rows (do them one after the other).

## Questions for the user

1. How are hold rounds counted?
   - (rec) A: in total: every round end with a hero on the ring and no enemy on it counts. Forgiving, easy to read as "Held 2 / 3".
   - Answered (user 2026-10-18 02:10): in total, but an enemy holding the ring at a round end lowers the count by 1; an empty ring changes nothing; only the round end counts.
   - B: in a row: an enemy on the ring resets the count to 0. Tenser, but one bad round can make the hold feel hopeless.
   - C: only the last round counts (be on the ring when time runs out). Simple, but the middle rounds stop mattering.
2. Who plans a guest such as Wenn the carter?
   - (rec) A: the player, like a hero (autoplay uses Balanced). No AI change, and the player can keep the guest safe on purpose.
   - B: the AI as an ally. More "escort" feel, but needs an ally-side AI draft and the guest can walk into danger.
   - C: the guest cannot act (stands still, can be hit). Simplest to read, but a unit with no turn feels odd in a planning game.
3. How does the solo Courtyard start?
   - (rec) A: standard deployment; Bryn starts on the ring and holds it for 3 rounds or wins by rout. A gentle first lesson for a hero alone.
   - B: centre deployment; one raider on each side. Shows the ring layout at once, but a solo hero flanked in the first fight is a big step up.
