# Screens to replicate

Goal (user, 2026-10-19): replicate every active Godot screen with its placeholder logic, function over form, so the
engine has a working reference for each screen type before the transition. Look and feel come after
(ENGINE_SPEC section 11).

Reference shots: `games/emberwake/docs/screens/` (1280x720, captured from the Godot build by
`games/emberwake/parity/godot/screen_tour.gd` on a sandbox save folder; re-run it after Godot UI changes). The engine's own
shots of the same screens are in `games/emberwake/docs/screens/engine/` (taken over the agent link).

The rules behind every player screen come from `emberwake_world`, checked against Godot by the journey replay
(`tests/journey.rs`: result, state and the views these screens read, after every action) and the save fixtures
(`tests/saves.rs`).

## Player screens

| # | Screen | Godot model / view | Engine | Status |
| :--- | :--- | :--- | :--- | :--- |
| 00 | Main menu | `main_menu_model.gd`, `main_screen_view.gd` | `menus.rs` `MainMenu` | done: Continue (newest save), New game, Load, Settings, Tools, Quit (asks when dirty) |
| 01 | New game (world cards) | `new_game_model.gd`, `new_game_view.gd` | `NewGameScreen` | done: playable worlds, party line, asks before replacing the autosave |
| 02, 17 | Load / save slots | `save_slots_model.gd`, `save_slots_view.gd` | `Slots` | done: autosave + 5 slots, peek rows; a click anywhere on a row saves (save mode, manual slots, asks before overwriting) or loads (load mode); lock and delete icons top right (lock on manual saves only, never the autosave; a locked save can't be overwritten or deleted, still loads; PLAYUI1-S); delete asks; agent link ids `slot.<n>`, `slot.<n>.lock`, `slot.<n>.delete`, `slots.back`; file import / export not yet (SAVE-IO must refuse a locked slot) |
| 03-06 | Settings (Sound, Display, Gameplay, Controls) | `settings_model.gd`, `settings_view.gd` | `SettingsScreen` | done as a draft-and-apply form saved to `%APPDATA%/Aegis/emberwake/settings.json`; the engine applies the display, text-size and sound settings (PORT-SETTINGS, PORT-AUDIO); the popup plays the draft volumes and a sample per volume slider |
| 07 | Region map (exploration) | `region_map_view.gd` | `region.rs` `RegionMap` | done: 32 px tiles, full width under the HUD bar with a camera centred on the party (no clamp; wall tiles drawn past the edge, display only; PLAYUI1-M), markers with icons, badges and hover verbs, names on hover or always (setting `map_names`), a floating foldable "Here (N)" legend top right with marker and NPC service icons (shop, recruit, offer, turn-in; `explore::npc_services`), the event log ticker bottom left (L or a click: the last 12); the legend and the ticker are hidden while talking, click-to-walk, E interact, chests, lore, exits, fights; HUD bar (PLAYUI1-H): location name top left, gold, supplies and icon buttons Party (P), Journal (J), Stash (I, the Character popup's Gear tab), Help (?), Menu (Esc) top right, turned off while a conversation, shop, turn-in or lore page is open |
| 08 | Dialogue (bottom box) | `dialogue_panel.gd` | `RegionMap` `talk_layout` / `draw_talk` | done: a 400 px box (x text size, at most the canvas less 40) centred 6 px above the bottom, 104 px to 45% of the screen tall; 64 px portrait with the name and faction band under it; the NPC's line on top (scrolls past 4 lines), numbered replies below (1-9; scroll past 5; locked dim with their reason; Continue is a row); the camera centres on the party and the speaker above the box; no key hint line (PLAYUI1-D); portrait, faction band, choices with lock reasons and rep / stat prefixes, services, turn-in picker (loot picks, recipient), recruits, fights; agent link `dialogue`, `dialogue.text`, `dialogue.choices`, `choice.<i>`, `dialogue.continue` |
| 09 | Shop | `shop_panel.gd` | `RegionMap::draw_shop` | done: buy / sell tabs, stock left, supplies x1 / x5, item detail line; centred under the HUD bar over the dimmed map, the dialogue box hidden while it is open (PLAYUI1-D); agent link `shop` |
| 10 | World map (travel) | `world_map_view.gd` | `worldmap.rs` | done: backdrop, pins by map_pos (circle slots otherwise), routes with times, trip hover, travel with road fights, camp, enter; the region map's HUD bar (PLAYUI1-H) with Back before the name, Time in the side panel, Esc back to the region map |
| 11 | Game menu (hub) | `hub_model.gd`, `hub_view.gd` | `hub.rs` `Hub` | done: party box with HP, doors with the Godot refusal reasons, quit to menu |
| 12-15 | Character: Stats, Level up, Skills, Gear | `party_model.gd`, `stats_model.gd`, `level_up_model.gd`, `loadout_model.gd`, `gear_model.gd` | `party.rs` `Character` | done: active toggle, stat table and derived block, level-up draft with before / after and the permanent-spend question, loadout slots, gear and battle pouch drafts, discard questions; the Gear tab in three columns (GEARUI1: stats with this gear and the green / red change a hovered or dragged item would make, equipped slots with the pouch under them, the party stash; drag and drop both ways, clicks kept, item descriptions with their tokens filled; the stash is sorted by item name, Godot sorted by id; gear the class has no training in is refused as in Godot); the Skills tab on fixed slots (SKILLUI1-U: five class and three extra slots, drag and drop between the list and the slots, right click takes a skill out, Clear all, < > rank arrows with rank and AP hover texts) |
| 16 | Quest journal | `journal_model.gd`, `journal_view.gd` | `hub.rs` `Journal` | done: groups, resolved text, progress, current step, rewards, loot choices |
| 18 | Battle results | `battle_results_model.gd`, `battle_results_view.gd` | `hub.rs` `Results` | done: headline and note, XP rows with bars, gold / loot / quest lines, level-up prompt; battle report table not yet |
| 23 | Battle | `ui/combat/*`, `scenes/combat_arena.gd` | `battle.rs` | done: arena (catalog or world encounter) and world battles with the saved party at stored HP, HP write-back, runner settlement; enemy tells while planning (PORT-INTENT: kind chips and arrows on the grid, "Next:" on the hostile cards, Intent, revealed plan and forecast in the hover card; I toggles, the Show enemy tells setting); the timeline's masked enemy banner is not drawn (the engine's strip shows the selected hero only); units animate (SPRITE1-U): idle, walk while sliding to the new tile, the skill's attack clip, hurt or hurt_guarded, death (held, then the x marker), defend then idle_guarded; each stands on a flat ellipse in its side's colour (ally blue, foe red), the selected hero has a 1px gold outline around its sprite and a hovered unit a faint white one (no disc, no gold cell square); animation speed and skip apply; bodiless units keep the letter panel |

## Dev tools (dev-only, never in the real game)

| # | Screen | Godot | Engine | Status |
| :--- | :--- | :--- | :--- | :--- |
| 19 | Content Forge (catalog viewer) | `ui/forge/*` | `forge.rs` | read-only viewer: kinds, entries, every field, engine summary, used by; no search, validation or editing yet |
| 20 | World Forge (world viewer) | `ui/world/*` | `forge.rs` | read-only viewer: world picker, overview (inheritance, connects, counts), merged entries with origin, used by |
| 21 | Gear & Skill Dojo | `ui/dojo/*`, `scripts/dojo/*` | `dojo.rs` on `emberwake_battle::dojo` | done (PORT-DOJO): the Godot dojo on a desk of engine windows (party with the carry bar, unit inspector with stat allocation, derived stats, gear with the pouch and quick-setup presets, stash with filters, stage, console, test log with breakdowns), drag and drop between the stash, the slots and the pouch, Godot's three layouts saved per player |
| 22 | Scene Runner | `ui/runner/*` | `runner.rs` | the engine's launcher (main menu Tools): arena on any encounter, forges, dojo, Exploration (RUNNER2: world and start pickers; fights Skip / Fled / Lost / Fight it), UI Gallery, data reload; the scene list scrolls; playtest recording not yet |
| - | UI Gallery | none (new) | `gallery.rs` | the UI one element at a time for review (UIGALLERY1), in three groups: Kit widgets (every kit widget in its states, usable); Screens (each screen shrunk into the pane on a sample debug game, dialogue and shop opened on the first person who offers them, Open shows the screen itself); Battle panels (plan strip, party list, hero inspector, battlefield, hostile list, threat intel, status and log, each the battle screen's own panel enlarged), and the battle timeline on a live arena fight (all units' tracks by block, enemies masked while planning as "Plans in secret" with their tell, Reveal to show them, < > through past rounds, a block's snapshot of every unit, a segment's action detail) |

## Known gaps (function over form)

- Godot's region tiles are drawn from its asset registry (green ground in the keep); the engine draws the
  `region_*` icon files, so colours differ. Marker icons are simple shapes until the sprite assets arrive.
- Battles in the world now play the full rules (objectives, phases, morale, guests, start statuses, world
  characters with named items and tuning) and battle items (pick a unit, then "Use `<item>` (`<n>` left)"; unused items
  go back to the saved pouch). The battle screen shows the objective and phase / morale events in the message bar;
  Godot's centred phase banner (BOSS1-U) is look-and-feel work for after the transition.
- Keyboard focus works on every screen (PORT-FOCUS: Tab / Shift+Tab, arrows except on the region map and the journal,
  Enter / Space); the look of the ring is not final, and the window toolbars and title bars take no focus yet.

## Order (done)

1. Data: the project's `games/emberwake/data/` folder through the engine's own loader (`emberwake_content::source`); the Godot exports
   (`parity/export.sh`) are only for parity checks.
2. Game state: GameState, NewGame, SaveGame (Godot saves load unchanged, every migration), party and progression rules.
3. Game loop rules: exploration, travel, dialogue, shop, quests, rest and camp, world battles, results; checked with
   journeys replayed in both engines (state compared after every action).
4. Screens on a shared widget kit (`ui.rs`, now `aegis_ui::ui`: buttons, tabs, cards, checkboxes, steppers, dropdowns, frames, rows,
   hover cards), menus first, then the play screens, then the dev tools.
