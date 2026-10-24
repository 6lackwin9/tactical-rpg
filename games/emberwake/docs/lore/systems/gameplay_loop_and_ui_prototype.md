# Gameplay loop and UI prototype

Status: canon for the Act 1 loop and the combat round ([CANON](../CANON.md) 2.1, 2.5-2.6, 3.2); proposal for the
prototype order. Related: [combat](combat_and_timeline.md), [screens](screen_architecture_and_separation_plan.md),
[safehouse economy](safehouse_economy_and_triage.md), [skill acquisition](skill_acquisition_and_loadout_systems.md).
Engine crates: `aegis_ui`, `emberwake_gui`, `emberwake_battle`, `emberwake_world`.

Two loops: the campaign loop between the hub and the district (days, Tallow, missions) and the combat loop inside a
fight (rounds of 6 blocks).

## 1. The campaign loop (Act 1)

| Step | What the player does | Screen | Layer |
| --- | --- | --- | --- |
| 1. The Sluice Undercroft | talk to Aline (Slab), Bram (Hearth) and Tess (board); rest in the Dry Hollows (1 day, 1 Warded Tallow, full HP); pick up to four and their gear, pouch and deck | region map (the Undercroft), Character | `[data]` NPCs and dialogue; `[kit hook]` facilities, bunks |
| 2. Tess's dead-drop board | take the next main mission (offered when the previous one is done) or optional board work | dialogue stand-in, later the mission board | `[kit hook]` mission board |
| 3. The district map | travel the Sunk Ward node graph: each route leg costs 1 day (stealth +1) and 1 Warded Tallow per day; road fights can trigger | world map | `[kit hook]` exists (travel clock, road fights); `[data]` nodes 00-07 |
| 4. The mission site | walk streets and interiors, meet patrols (roamer markers; no sight cones), loot chests, start the fight | region map | `[kit hook]` exists |
| 5. The fight | section 2 | battle | `[engine]` timeline, `[kit hook]` ruleset "timeline" |
| 6. The aftermath | XP, Scrap, loot and organs (REWARD1); downed founders become wounded; the board moves on | Results | `[kit hook]` exists; wounded `[kit hook]` new |
| 7. Back home | spend Scrap and materials at the Hearth, store organs for Act 2; the Doom Clock shows the days left | region map, facility | `[kit hook]` campaign meters |

Meters that run across the loop: the Doom Clock (42 days at Act 1 day 1, soft in Acts 1-2) and the Inquisition Alert
(0-100, no siege in Act 1). Battles cost no days. At 0 Warded Tallow every battle starts at -2 AP until restocked.
Act 2 adds Strain, Calcification, grafts, recruits, doctrine, permadeath and capture, sieges and the 8-bunk project
(CANON 3.3).

## 2. The combat loop (one round = 6 blocks) `[kit hook]` ruleset option "timeline"

The Ashlands keeps the "classic" ruleset; the Low Sleep worlds use "timeline" (CANON 2.1).

1. Read the enemy: enemy tells show while planning (INTENT1, exists: kind chips and arrows on the board, "Next:" on the
   hostile cards, the I key); windups to break are on the strip.
2. Plan: each deployed operative fills its 6 blocks with actions from its deck. AP is 10 at most, +4 per round; an
   action pays its AP when it starts (`ap_cost`, default = skill level). Speed tiers are relative windups: Snap 1
   block, Standard 2, Heavy 3-4, plus recovery blocks where a maneuver has them.
3. Resolve: all sides act at once, block by block; an action lands on its last block; same-block impacts follow the
   engine's initiative order (no mutual trades). A hit that empties a unit's poise breaks it: the action in windup and
   the unit's remaining blocks that round are cancelled.
4. Round end: statuses tick (bleed, poison, burn, regen; engine order); poise refills; Strain decays 10 if the unit
   used no Inoculated action, 10 more under Active Guard, and a unit at 100 rolls the 1d3 crisis (Strain and crisis
   from Act 2); then the next round's planning.

Cut from the old loop (CANON 2.1): the timeline scrub slider (the multi-track ghost scrubber), fixed tick anchors
("Snap T2, Heavy T5"), the d100 crisis, stances, evasion and accuracy rolls.

## 3. The battle HUD

Layout A (COMBAT-UI3): the planning strip on top, party cards and the selected hero on the left, hostile cards and the
enemy card on the right, the status bar and log at the bottom; the board in the middle is STAGE1 option B (tilted 12 x
9 board, 32 x 24 px cells, 40 px units). Details: [screens](screen_architecture_and_separation_plan.md) 5.

| Element | Today | Lore addition | Layer |
| --- | --- | --- | --- |
| Timeline strip | `aegis_ui::timeline` at the top: blocks, the selected hero's plan, dashed windups | impact badges, recovery and guard fills | `[engine]` |
| Hero cards | HP, AP | poise bar; Strain gauge from Act 2 (marks 25 / 50 / 75 / 100) | `[kit hook]` |
| Hostile cards | HP, "Next:" tells | poise bar | `[kit hook]` |
| Segment colours | strip defaults | windup, impact, recovery, guard and Inoculated edges in art-bible hues ([art](../specs/art_and_visual_style.md)) | `[data]` |

## 4. What the engine already gives

| Need | Existing piece | Reuse |
| --- | --- | --- |
| Block timeline | `aegis_ui::timeline` (`Timeline`, `Segment` with `dashed`), the battle strip, the UI Gallery's all-units viewer | add badges; no scrubber |
| Battle board | `battle.rs` `BattleScreen` with the STAGE1 stage, animated units (SPRITE1), facing (FACE1), tells (INTENT1) | add poise, the ruleset switch |
| Operative dossier | `party.rs` `Character` (Stats, Level up, Skills, Gear) | add poise, deck slots (passive and keystone slots are new) |
| Hub menu | `hub.rs` `Hub` (Party, Journal, Save, Load, Settings, Quit) | keep as the in-game menu; the safehouse is a region map |
| District map | `worldmap.rs` `WorldMap` + `emberwake_world::travel` | route times in days, Tallow as upkeep |
| Walkable places | `region.rs` `RegionMap` | the Undercroft and mission sites as locations |
| Combat sandbox | Scene Runner arena (`runner.rs`) and Gear Dojo (`dojo.rs`) | first combat prototype |

## 5. Prototype paths (proposal)

| Path | Steps | Gives |
| --- | --- | --- |
| A. Arena first | run ENC-PRO-0A (Kaelen against two Castellan Footmen) in the Scene Runner arena with the "timeline" ruleset; then ENC-PRO-0B and -0C | the feel of windups, breaks and guard fastest, no campaign plumbing |
| B. Vertical slice | prologue 0a-0c, the hearth scene, the Sunk Ward graph, Tess's board, Act 1 M1 | the whole Act 1 loop, slower to first play |

The old doc recommended A; the choice is the planner's (Open, see gaps).

## 6. Gaps

1. Prototype order: arena first (A) or the vertical slice (B).
2. Whether the first playable runs the "timeline" ruleset from day one or starts on "classic" while poise lands.
3. Where a mission site ends: does every mission walk a region map before the fight, or do some start the fight from
   the district map node.
