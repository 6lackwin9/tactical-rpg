# Screens: what the lore needs and what the engine has

Status: canon for the mapping onto the real screens and for the battle look (STAGE1 option B, timeline strip at the
top; [CANON](../CANON.md) 2.1); proposal for the new screens and the build order. Engine reference:
[SCREENS.md](../../SCREENS.md), [GAME_BRIEF.md](../../GAME_BRIEF.md). Related: [gameplay loop](gameplay_loop_and_ui_prototype.md),
[safehouse economy](safehouse_economy_and_triage.md).

Layer tags on screens: `[engine]` for the screen technology (`aegis_ui`: screen stack, widget kit, timeline widget,
windows, art, saves); `[kit hook]` for a screen shape any story in the genre could fill (today's play screens live in
`emberwake_gui` and count as kit shapes; kit code moves out when a second game shares it, STRUCTURE.md); `[data]` for
what this story fills in; `[game code]` for screens only Emberwake could use.

## 1. Exploration and combat stay separate `[engine]`

Screens sit on the engine's stack (`aegis_ui::screen::Screen`, navigation queued in `cx.nav` as `Nav::Push`, `Nav::Pop`
or `Nav::Reset`; screens never call each other). A fight pushes the battle screen over the map and pops it when done.

- Persistent state: `GameState` in the game context (roster, stash, purse, flags, quests, travel clock, saves through
  `aegis_save`).
- Transient state: the battle `emberwake_battle::session::Session` (plans, statuses, the round's blocks; poise and
  Strain will live here too).
- Handoff: the battle ends won / lost / fled; `emberwake_world::battle::settle` pays rewards by REWARD1 (XP, purse
  share, loot rolls, `key_loot`, objective rewards), HP is written back (HEAL1) and the Results popup opens. Organs
  arrive as loot; wounded, captured and Calcification notices are new lines `[kit hook]`.

## 2. Lore screens on real screens

| Lore name | Real screen (file, type) | Layer | Change the lore needs |
| --- | --- | --- | --- |
| District map (the Sunk Ward node graph) | world map, `worldmap.rs` `WorldMap` on `emberwake_world::travel` (pins, routes with times, road fights, camp, enter) | `[kit hook]` exists | `[data]` nodes 00-07, backdrop, route times in days; meters on the HUD bar `[kit hook]` |
| Walkable places: the Sluice Undercroft, mission streets, interiors | region map, `region.rs` `RegionMap` (square top-down tiles, 16 logical px; markers, NPCs, chests, exits, roamers, click-to-walk) | `[kit hook]` exists | `[data]` the Undercroft as a location with Aline, Bram and Tess as NPC markers; the map stays square top-down |
| Dialogue | the region map's bottom box (`talk_layout`, `draw_talk`; portrait, choices with lock reasons) | `[kit hook]` exists | `[data]` lines; stand-in for crafting and the board (section 3) |
| Black market (Madame Sallow), forge and slab trades | the shop's three-column trade, `shop.rs` `Trade` / `ShopPane` | `[kit hook]` exists | currency shown as Scrap (purse hook) |
| "Hub" menu (party, journal, save) | in-game menu, `hub.rs` `Hub` (doors Party, Journal, Save, Load, Settings, Quit, Return to map) | `[kit hook]` exists | none: this is not the safehouse |
| Operative dossier and deck | Character, `party.rs` `Character` (tabs Stats, Level up, Skills, Gear; Gear holds the pouch and the stash) | `[kit hook]` exists | Poise and armour poise in Stats; Composure, a derived stat (CANON 2.1; data-defined derived stats `[kit hook]`); Strain floor and Calcification stage (Act 2) `[kit hook]`; deck slots (section 4) |
| Quest journal | `hub.rs` `Journal` | `[kit hook]` exists | none |
| Battle results and triage | `hub.rs` `Results` (headline, XP rows, gold, loot and quest lines, level-up prompt) | `[kit hook]` exists | lifecycle and Calcification lines `[kit hook]` |
| Tactical battle | `battle.rs` `BattleScreen` on `aegis_ui::timeline` | `[engine]` widget, `[kit hook]` screen | section 5 |
| Main menu, new game, slots, settings | `menus.rs` `MainMenu`, `NewGameScreen`, `Slots`, `SettingsScreen` | `[kit hook]` exists | `[data]` title and art |
| Combat sandbox | arena from the Scene Runner (`runner.rs`) and the Gear Dojo (`dojo.rs`) | dev only | none |

Corrections to the old plan: `DistrictMap`, `PartyScreen`, `SafehouseHub`, `SurgeryScreen` and `ForgeScreen` do not
exist; `forge.rs` is the dev-only Content Forge viewer and is never a player screen; `battle_results_model.gd` is the
frozen Godot model, the engine's is `hub.rs` `Results`.

## 3. Missing screens

The slice gets no stand-ins (user 2026-10-25, Q-55, Q-56): every screen it needs is built as an engine or kit piece
in M0-M1, before the slice ([roadmap](../specs/full_game_systems_roadmap.md) section 2). The last column says when.

| Screen | What it shows | Layer | When |
| --- | --- | --- | --- |
| Bunks (the Dry Hollows) | 6 (later 8) bunks with each operative's state: ready, wounded, captured (memorial), Isolation Crypts; dismiss and retire | `[kit hook]` unit lifecycle | before the slice if the slice shows bunks (wounded, the Crucible capture); dismiss, retire and the Crypts from Act 2 |
| Facility (Tinker's Hearth, Apothecary's Slab) | keeper, level, projects (the bunk project), exchange recipes | `[kit hook]` facility + recipes | before the slice for the Hearth's forge if the slice sells forging; the bunk project from Act 2 |
| Slab patient view | the anatomical silhouette with four graft regions (Act 2) | `[game code]` | Act 2 |
| Mission board (Tess's dead-drop board) | the next main mission and optional board work | `[kit hook]` | before the slice |
| Vignette player | the opening vignette cards, the hearth scene card, act title cards, the Doom Clock's first appearance | `[kit hook]` (images and timed text on the engine painter) | before the slice |
| Endings | Ending A Preservation, B Ascendancy, C Red Martyrdom (Act 3, proposal) | `[kit hook]` vignette player, `[data]` cards | Act 3 |
| Extinguished | the failure screen: an Act 1 Breach (Q-13), no living operative (Act 2 on), or the Act 3 Doom Clock runs out | `[kit hook]` | before the slice (the Act 1 Breach) |
| Campaign meters | Doom Clock and Inquisition Alert on the map HUD bar, beside Scrap and Tallow, with a clear warning at each Alert threshold | `[kit hook]` | before the slice |

## 4. Character screen: the deck

CANON 2.1: 8 slots = 2 base moves (Move, Active Guard) + 3 class actives + 2 passives + 1 keystone. The Skills tab today
has five class and three extra slots (SKILLUI1-U) and holds skill families only: basics (strike, guard, move, rest,
switch_target, offhand_jab) are always owned and never slotted, and passives come from sources (world character,
faction, discipline, named item), never from slots. Passive and keystone slots are therefore a `[kit hook]`, not an
existing engine slot (CANON 2.1 says so). Decided (2026-10-25, user, Q-48): the deck works like today's skill slots,
bucketed by kind (base, active, passive, keystone); passive and keystone slots are a new kit slot type. Card faces
(proposal): speed tier badge in blocks, AP cost, Strain
cost (Act 2), poise damage, a pierce mark for Inoculated skills; Cold Steel and Inoculated cards in two palette hues
from the [art bible](../specs/art_and_visual_style.md).

## 5. The battle screen

- Board `[kit hook]`: STAGE1 option B ([STAGE1](../../specs/STAGE1.md), approved 2026-10-23): a tilted board of 12 x 9
  cells drawn 32 x 24 px, a zone backdrop above the far edge, a front face below the near edge, 40 px units, upright
  props sorted with the units by their feet, contact shadows. The old "32 x 32 isometric grid" and "32 x 32 flat grid"
  yield to it.
- Sprites `[data]`: "32 x 48 sprite" means the figure's bounds inside the rig's 48 x 48 frames.
- Timeline strip `[engine]`: at the top of the screen (COMBAT-UI3 layout A), the engine's `aegis_ui::timeline`. A round
  is 6 blocks; windups are dashed segments (`Segment.dashed`), impacts get badges on the existing strip, recovery blocks
  a muted segment, Active Guard its own fill. The multi-track ghost scrubber is cut (CANON 2.1); the strip shows the
  selected hero's plan, and the UI Gallery keeps the all-units timeline viewer.
- Enemy tells `[kit hook]` exist (INTENT1: kind chips and arrows on the board, "Next:" on hostile cards, the I key).
- Hero cards `[kit hook]`: HP and AP today; add a poise bar and, from Act 2, a Strain gauge with marks at 25, 50, 75
  and 100 (the bracket edges; the old 30 mark is retired). Colours from the art bible (the poise bar is Tallow Amber,
  hue 10).
- The Doom Clock and Alert are not on the battle screen (the strip owns the top); see gaps.

## 6. Build order (proposal, re-based on the 2026-10-25 milestones)

1. M0-M1, the kit screens the slice needs: the Mission board, the Vignette player, the campaign meters on the HUD bar
   with the Alert warnings, the Extinguished screen (the Act 1 Breach), Results lines for wounded and captured
   founders, the Character screen's deck with slot kinds, and Bunks and Facility as far as the slice uses them.
2. M1, combat sandbox: the battle screen with the "timeline" ruleset option, poise, chains and impact badges, run from
   the Dojo and the Scene Runner arena on ENC-PRO-0A (Kaelen against the two Castellan Footmen).
3. M2, the slice as data: the Undercroft as a region map with Aline, Bram and Tess; the Sunk Ward world map with day
   costs and Tallow upkeep; no dialogue stand-ins.
4. Later: the Slab patient view and the Bunks project (Act 2), Endings (Act 3).

## 7. Gaps

1. Deck slots: CANON 2.1 now states that the engine slots hold skill families only (section 4). Decided (2026-10-25,
   user, Q-48): slot kinds base, active, passive, keystone.
2. Battle meters: whether the Doom Clock and Alert appear anywhere in battle.
3. Vignette player scope: still cards with typewriter text only, or also simple pans and fades.
4. Who opens the facility screens: the keeper NPC on the region map, or a facility list on a hub screen.
5. How much of the Bunks and Facility screens the slice needs (the roadmap's beat 6 names the Hearth's forge and the
   bunks): with no stand-ins allowed, the answer sets M0-M1 scope. Options: (a) both screens in a slice cut (states,
   forge recipes); (b) the slice shows bunks only on the Character screen's roster and sells forging through the
   existing shop Trade. Recommendation: (b) if the shop Trade counts as an existing system, else (a).
