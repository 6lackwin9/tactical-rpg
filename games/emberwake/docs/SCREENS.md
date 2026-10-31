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
| 07 | Region map (exploration) | `region_map_view.gd` | `region.rs` `RegionMap` | done: 32 px tiles, full width under the HUD bar with a camera centred on the party (no clamp; wall tiles drawn past the edge, display only; PLAYUI1-M), markers with icons, badges and hover verbs, names on hover or always (setting `map_names`), a floating foldable "Here (N)" legend top right with marker and NPC service icons (shop, recruit, offer, turn-in; `explore::npc_services`), the event log ticker bottom left (L or a click: the last 12); the legend and the ticker are hidden while talking, click-to-walk, E interact, chests, lore, exits, fights; HUD bar (PLAYUI1-H): location name top left, gold, supplies and icon buttons Party (P), Journal (J), Stash (I, the Character popup's Gear tab), Help (?), Menu (Esc) top right, turned off while a conversation, shop, turn-in or lore page is open; props (PROPS1-L2): ground props over the tiles, then one feet-sorted pass of standing props, markers and the party (`props.rs`, `DepthList`), the party's ring again over a prop that hides it, then badges and names; a standing prop without art is a plain block on its footprint (fill, darker south face, edge), a ground prop without art draws nothing; props have no widget |
| 08 | Dialogue (bottom box) | `dialogue_panel.gd` | `RegionMap` `talk_layout` / `draw_talk` | done: a 400 px box (x text size, at most the canvas less 40) centred 6 px above the bottom, 104 px to 45% of the screen tall; 64 px portrait with the name and faction band under it; the NPC's line on top (scrolls past 4 lines), numbered replies below (1-9; scroll past 5; locked dim with their reason; Continue is a row); the camera centres on the party and the speaker above the box; no key hint line (PLAYUI1-D); the portrait lives (PORTRAITFX1-D: `portraits::Faces` on surface `dialogue`: breath, rim flicker, blinks, the sidecar's idle overlays and the state rows of the speaker's look, a party member's when one plays the speaker, e.g. `wounded`; the initial letter without a file); portrait, faction band, choices with lock reasons and rep / stat prefixes, services, turn-in picker (loot picks, recipient), recruits, fights; agent link `dialogue`, `dialogue.text`, `dialogue.choices`, `choice.<i>`, `dialogue.continue`, state `portraits` |
| 09 | Shop | `shop_panel.gd` | `shop.rs` `ShopPane` (drawn by `RegionMap`) | done (PLAYUI1-P): a three-column trade (shop, trade, stash), clicks only: a row adds 1, +5 / +10 add up to what is there ("X of N"), a trade line goes back whole, Total and Gold a -> b, Confirm via `shop::trade` (net gold, all or nothing), Clear, Close / Esc with lines asks "Discard this trade?"; 600x300 centred under the HUD bar over the dimmed map, growing with the text size, the dialogue box hidden while it is open; agent link `shop.row` / `shop.add5` / `shop.add10` / `stash.*` / `trade.*` / `shop.close` |
| 10 | World map (travel) | `world_map_view.gd` | `worldmap.rs` | done: backdrop, pins by map_pos (circle slots otherwise), routes with times, trip hover, travel with road fights, camp, enter; the region map's HUD bar (PLAYUI1-H) with Back before the name, Time in the side panel, Esc back to the region map |
| 11 | Game menu (hub) | `hub_model.gd`, `hub_view.gd` | `hub.rs` `Hub` | done: party box with HP, doors with the Godot refusal reasons, quit to menu |
| 12-15 | Character: Stats, Level up, Skills, Gear | `party_model.gd`, `stats_model.gd`, `level_up_model.gd`, `loadout_model.gd`, `gear_model.gd` | `party.rs` `Character` | done: active toggle, stat table and derived block, level-up draft with before / after and the permanent-spend question, loadout slots, gear and battle pouch drafts, discard questions; the Gear tab in three columns (GEARUI1: stats with this gear and the green / red change a hovered or dragged item would make, equipped slots with the pouch under them, the party stash; drag and drop both ways, clicks kept, item descriptions with their tokens filled; the stash is sorted by item name, Godot sorted by id; gear the class has no training in is refused as in Godot); the Skills tab on fixed slots (SKILLUI1-U: five class and three extra slots, drag and drop between the list and the slots, right click takes a skill out, Clear all, < > rank arrows with rank and AP hover texts) |
| 16 | Quest journal | `journal_model.gd`, `journal_view.gd` | `hub.rs` `Journal` | done: groups, resolved text, progress, current step, rewards, loot choices |
| 18 | Battle results | `battle_results_model.gd`, `battle_results_view.gd` | `hub.rs` `Results` | done: headline and note, XP rows with bars, gold / loot / quest lines, level-up prompt; battle report table not yet |
| 23 | Battle | `ui/combat/*`, `scenes/combat_arena.gd` | `battle.rs` | done: arena (catalog or world encounter) and world battles with the saved party at stored HP, HP write-back, runner settlement; enemy tells while planning (PORT-INTENT: kind chips and arrows on the grid, "Next:" on the hostile cards, Intent, revealed plan and forecast in the hover card; I toggles, the Show enemy tells setting); the timeline's masked enemy banner is not drawn (the engine's strip shows the selected hero only); units animate (SPRITE1-U): idle, walk while sliding to the new tile, the skill's attack clip, hurt or hurt_guarded, death (held, then the x marker), defend then idle_guarded; each stands on a flat ellipse in its side's colour (ally blue, foe red), the selected hero has a 1px gold outline around its sprite and a hovered unit a faint white one (no disc, no gold cell square); units face their target (FACE1): a unit turns toward the target or area aim of the action it resolves and toward the x of its step (a vertical step or a target in its column keeps the facing); while planning it shows its planned (hero: first targeted step) or intended (enemy tell, only while the tells are shown) target; a battle starts with the party facing right and the foes left; the frame and its outline are mirrored while facing left; animation speed and skip apply (skip ends in the same facing); bodiless units keep the letter panel |

## Dev tools (dev-only, never in the real game)

| # | Screen | Godot | Engine | Status |
| :--- | :--- | :--- | :--- | :--- |
| 19 | Content Forge (catalog viewer) | `ui/forge/*` | `forge.rs` | read-only viewer: kinds, entries, every field, engine summary, used by; no search, validation or editing yet |
| 20 | World Forge (world viewer) | `ui/world/*` | `forge.rs` | read-only viewer: world picker, overview (inheritance, connects, counts), merged entries with origin, used by |
| 21 | Gear & Skill Dojo | `ui/dojo/*`, `scripts/dojo/*` | `dojo.rs` on `emberwake_battle::dojo` | done (PORT-DOJO): the Godot dojo on a desk of engine windows (party with the carry bar, unit inspector with stat allocation, derived stats, gear with the pouch and quick-setup presets, stash with filters, stage, console, test log with breakdowns), drag and drop between the stash, the slots and the pouch, Godot's three layouts saved per player |
| 22 | Scene Runner | `ui/runner/*` | `runner.rs` | the engine's launcher (main menu Tools): arena on any encounter, forges, dojo, Exploration (RUNNER2: world and start pickers; fights Skip / Fled / Lost / Fight it), UI Gallery, data reload; the scene list scrolls; playtest recording not yet |
| - | UI Gallery | none (new) | `gallery.rs` | the UI one element at a time for review (UIGALLERY1), in four groups: Kit widgets (every kit widget in its states, usable); Art: Portraits (PORTRAITFX1-G: every portrait of the game and kit layers with its expression frames, drawn through the game's binding with the effect toggles, reactions, step events, one control per cue row, size, reduce motion, anchors and the expression picker; its sidecar summary); Screens (each screen shrunk into the pane on a sample debug game, dialogue and shop opened on the first person who offers them, the Mission board pane on a sample board (STORYUI1-U2), Open shows the screen itself); Battle panels (plan strip, party list, hero inspector, battlefield, hostile list, threat intel, status and log, each the battle screen's own panel enlarged), and the battle timeline on a live arena fight (all units' tracks by block, enemies masked while planning as "Plans in secret" with their tell, Reveal to show them, < > through past rounds, a block's snapshot of every unit, a segment's action detail) |

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

## Story screens (STORYUI1-U1, STORYUI1-U2)

- Story stage (`story_stage`): opens on a new game in a world with a start sequence, on a save made during one, and when a sequence
  starts from the maps (a dialogue choice, a campaign trigger). It plays the steps: vignettes full screen (Space, Enter or click to
  continue; Esc, right click or Skip to skip, asking first for an unseen scene), fights with the results popup after, then the region map
  at the `enter` location or the main menu at `end` `menu`. No game menu while it runs. Queued vignettes play over the region and world
  maps when nothing is open. PORTRAITFX1-D: a `line` card's portrait lives (breath from 96 device px, the Tallow Amber
  rim flicker, blinks, its sidecar's idle overlays; no state tints: the scene is staged); agent state `portraits`.
- Mission board pane (STORYUI1-U2, `board.rs`): the keeper's board reply opens it in the shop's place (600 x 300 at Normal,
  grown with the text, over the dimmed map, the dialogue box hidden): the board's name and keeper, the intro, the sections
  (Main, Jobs, Under way, Locked, Closed) on the left with each quest's name, node, deadline ("By day 14 (6 left)", red) and
  line, the picked quest's detail on the right (status, description, steps, rewards, loot picks, reputation, where, deadline),
  Accept (offered quests only; a refusal shows on the pane; `Quest taken: <name>.` in the log) and Close; Esc and right click
  close it back to the conversation. Agent link `board`, `board.row.<qid>`, `board.accept`, `board.close`, `board.errors`.
- Keeper badge: a keeper whose open board has work wears the quest badge on the map and in the Here legend ("New work on
  the board").
- World map: a node holding jobs of an opened board wears the quest badge at its pin's top left (`node.<qid>.job`) and its
  hover card lists `Job: <name>` per job.
- UI Gallery, Mission board entry (Screens): the pane on a sample board without a game (every section, deadlines, Accept moves
  an offered quest to Under way).
- Settings, Gameplay tab: Text speed (Slow, Normal, Fast, Instant) and Reduce motion (engine keys `text_speed`, `reduce_motion`).
- UI Gallery, Vignette player entry: a reel picker (the engine's sample, then every world's vignettes as written) in the game's card
  palette.
- Scene Runner, Exploration: a location other than the world's start drops the world's start sequence; the start location and the world map play it (STORYUI1-U1).

## Campaign screens (CAMPAIGN1-U slice 1)

- HUD bar in a campaign world: the gold and supplies counters take the world's purse names, icons and tips; the visible
  meters follow by `hud.order` as chips (an icon or a lamp in the band tone's colour, the short name, the value with its
  unit; agent link `hud.meter.<id>`); the hover card names the meter (or its label), the band, the next rule point and
  the description. The Ashlands bar is unchanged.
- Campaign notice (`campaign_notice`): one per meter warning, checkpoint warning or missed `go_on` checkpoint, shown one
  at a time by the region map, the world map and the story stage when nothing else is open; a title in the band tone's
  colour, the data's text, OK (`campaign_notice.ok`; Enter, Space, Esc or right click close it). Each event also logs a
  line.
- Asks before a run-ending step (the engine's `ask`, No as the default): a marker fight (No: the party stands on the tile
  it stepped from), a dialogue fight (the talk is closed; No fights nothing, the choice's other effects stay), a trip and
  Camp. A road or roaming fight that could end the run starts with a notice over the battle.
- Run over (`run_over`): replaces every screen once the run is over (warnings still waiting are dropped; they are in the
  log). The end card's vignette plays once, then a plain card: "The run is over", the end's text, and Load the act's
  start (`run_over.load_act`, off with the reason when there is no act-start save), Load a save (`run_over.load_save`,
  the load menu), Main menu (`run_over.main_menu`). Esc and right click do nothing; no slot or autosave is written
  while the run is over.

## Campaign screens (CAMPAIGN1-U slice 2)

- The campaign queue: with nothing open the region and world maps play its head after the notices: queued vignettes,
  then a queued battle (`Fight: <name>!`, with the notice over it when it could end the run), then on the region map a
  queued dialogue scene as the conversation (the world map leaves it for the region map).
- A battle started at zero upkeep: every party member has the purse's `start_ap` less AP in round 1 (not below 0); the
  log and the battle's opening line say so ("No Warded Tallow: -2 AP").
- Dialogue: a reply that rests asks first when the rest's days would end the run (No as the default; No keeps the talk).
- World map: Camp reads `Make camp (<cost> <upkeep>)` with the rest's days on its hover card ("Make camp (3 supplies)"
  in the Ashlands); the Travel hover card gives the trip in days and upkeep, then "Arrive on day 9, 34 days left.";
  a node's hover card lists its checkpoint lines ("Checkpoint: by day 14", red). Log lines name the purse (camp, wake,
  retreat, quest gold).
