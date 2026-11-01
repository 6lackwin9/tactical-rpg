# Screens to replicate

Goal (user, 2026-10-19): replicate every active Godot screen with its placeholder logic, function over form, so the
engine has a working reference for each screen type before the transition. Look and feel come after
(ENGINE_SPEC section 11).

Reference shots: `games/emberwake/docs/screens/` (1280x720, captured from the Godot build by
its screen tour script, retired with `parity/` in CLN-42; kept as the reference shots). The engine's own
shots of the same screens are in `games/emberwake/docs/screens/engine/` (taken over the agent link).

The rules behind every player screen come from `emberwake_world`, checked by the engine journey golden
(`tests/journey.rs`: result, state and the views these screens read, after every action) and the store resave
(`tests/store.rs`).

## Player screens

| # | Screen | Godot model / view | Engine | Status |
| :--- | :--- | :--- | :--- | :--- |
| 00 | Main menu | `main_menu_model.gd`, `main_screen_view.gd` | `menus.rs` `MainMenu` | done: Continue (newest save), New game, Load, Settings, Tools, Quit (asks when dirty) |
| 01 | New game (world cards) | `new_game_model.gd`, `new_game_view.gd` | `NewGameScreen` | done: playable worlds, party line, asks before replacing the autosave |
| 02, 17 | Load / save slots | `save_slots_model.gd`, `save_slots_view.gd` | `Slots` | done: autosave + 5 slots, peek rows; a click anywhere on a row saves (save mode, manual slots, asks before overwriting) or loads (load mode); lock and delete icons top right (lock on manual saves only, never the autosave; a locked save can't be overwritten or deleted, still loads; PLAYUI1-S); delete asks; agent link ids `slot.<n>`, `slot.<n>.lock`, `slot.<n>.delete`, `slots.back`; file import / export not yet (SAVE-IO must refuse a locked slot) |
| 03-06 | Settings (Sound, Display, Gameplay, Controls) | `settings_model.gd`, `settings_view.gd` | `SettingsScreen` | done as a draft-and-apply form saved to `%APPDATA%/Aegis/emberwake/settings.json`; the engine applies the display, text-size and sound settings (PORT-SETTINGS, PORT-AUDIO); the popup plays the draft volumes and a sample per volume slider |
| 07 | Region map (exploration) | `region_map_view.gd` | `region.rs` `RegionMap` | done: 32 px tiles, full width under the HUD bar with a camera centred on the party (no clamp; wall tiles drawn past the edge, display only; PLAYUI1-M), markers with icons, badges and hover verbs, names on hover or always (setting `map_names`), a floating foldable "Here (N)" legend top right with marker and NPC service icons (shop, recruit, offer, turn-in; `explore::npc_services`), the event log ticker bottom left (L or a click: the last 12); the legend and the ticker are hidden while talking, click-to-walk, E interact, chests, lore, exits, fights; HUD bar (PLAYUI1-H): location name top left, gold, supplies and icon buttons Party (P), Journal (J), Stash (I, the Character popup's Gear tab), Help (?), Menu (Esc) top right, turned off while a conversation, shop, turn-in or lore page is open; props (PROPS1-L2): ground props over the tiles, then one feet-sorted pass of standing props, markers and the party (`props.rs`, `DepthList`), the party's ring again over a prop that hides it, then badges and names; a standing prop without art is a plain block on its footprint (fill, darker south face, edge), a ground prop without art draws nothing; props have no widget |
| 08 | Dialogue (bottom box) | `dialogue_panel.gd` | `talk.rs` `talk_layout` / `draw_talk` (ENCOUNTER2-C2: shared with the battle talk), drawn by `RegionMap` | done: a 400 px box (x text size, at most the canvas less 40) centred 6 px above the bottom, 104 px to 45% of the screen tall; 64 px portrait with the name and faction band under it; the NPC's line on top (scrolls past 4 lines), numbered replies below (1-9; scroll past 5; locked dim with their reason; Continue is a row); the camera centres on the party and the speaker above the box; no key hint line (PLAYUI1-D); the portrait lives (PORTRAITFX1-D: `portraits::Faces` on surface `dialogue`: breath, rim flicker, blinks, the sidecar's idle overlays and the state rows of the speaker's look, a party member's when one plays the speaker, e.g. `wounded`; the initial letter without a file; PORTRAITFX1-X: the node's `expression` is the speaker's face for the node, its frame `faces/<id>/<expression>.png` when it exists, else the `neutral` frame, else the single portrait, over the cue rows' faces; the region map's and the battle talk's box alike; agent `talk` state `expression`); portrait, faction band, choices with lock reasons and rep / stat prefixes, services, turn-in picker (loot picks, recipient), recruits, fights; agent link `dialogue`, `dialogue.text`, `dialogue.choices`, `choice.<i>`, `dialogue.continue`, state `portraits` |
| 09 | Shop | `shop_panel.gd` | `shop.rs` `ShopPane` (drawn by `RegionMap`) | done (PLAYUI1-P): a three-column trade (shop, trade, stash), clicks only: a row adds 1, +5 / +10 add up to what is there ("X of N"), a trade line goes back whole, Total and Gold a -> b, Confirm via `shop::trade` (net gold, all or nothing), Clear, Close / Esc with lines asks "Discard this trade?"; 600x300 centred under the HUD bar over the dimmed map, growing with the text size, the dialogue box hidden while it is open; agent link `shop.row` / `shop.add5` / `shop.add10` / `stash.*` / `trade.*` / `shop.close` |
| 10 | World map (travel) | `world_map_view.gd` | `worldmap.rs` | done: backdrop, pins by map_pos (circle slots otherwise), routes with times, trip hover, travel with road fights, camp, enter; the region map's HUD bar (PLAYUI1-H) with Back before the name, Time in the side panel, Esc back to the region map |
| 11 | Game menu (hub) | `hub_model.gd`, `hub_view.gd` | `hub.rs` `Hub` | done: party box with HP (ROSTER1-U: a wounded member's field HP and red "Wounded" mark with a hover; the box grows with its widest line and takes two columns past four members; "Bunks used/cap" under it only in a world with a `roster` block; PORTRAITFX1-R: a 20 px portrait at each row's left, its three lines 22 px in, `portraits::Faces` on surface `roster` with the member's look, so a wounded member's portrait desaturates; the initial letter without a file), doors with the Godot refusal reasons, quit to menu; agent link `hub.portrait.<member id>`, state `portraits` |
| 12-15 | Character: Stats, Level up, Skills, Gear | `party_model.gd`, `stats_model.gd`, `level_up_model.gd`, `loadout_model.gd`, `gear_model.gd` | `party.rs` `Character` | done: the member's 48 px portrait at the frame's top-left, left of the member picker (PORTRAITFX1-R: `portraits::Faces` on surface `roster`: rim flicker and blinks, breath from 96 device px, the state rows of the member's look, e.g. `wounded`; Q / E draw the next member's; agent link `character.portrait`, state `portraits`); active toggle (ROSTER1-U: "Bunks used/cap" at its right only in a world with a `roster` block; a wounded member's "Wounded HP n / cut" after the level line, with a hover), stat table and derived block, level-up draft with before / after and the permanent-spend question, loadout slots, gear and battle pouch drafts, discard questions; the Gear tab in three columns (GEARUI1: stats with this gear and the green / red change a hovered or dragged item would make, equipped slots with the pouch under them, the party stash; drag and drop both ways, clicks kept, item descriptions with their tokens filled; the stash is sorted by item name, Godot sorted by id; gear the class has no training in is refused as in Godot); the Skills tab on fixed slots (SKILLUI1-U: five class and three extra slots, drag and drop between the list and the slots, right click takes a skill out, Clear all, < > rank arrows with rank and AP hover texts); under the deck rule the deck view (RULESET1-DU: buckets, Basics, the owned list or the order by intent) and, in the order view, a follow-up list per entry (INPUT1-L, FL3: a ">" toggle at the entry's right, "+" when a list is set; the open list across the right pane under the entry: "After `<skill>`:", "Use the `<Intent>` order", entries as chips (drag to reorder, right click takes one out, a "Drop here" end chip), "Add:" with the intent's skills; ids `skills.follow.<family>`, `.<k>`, `.end`, `skills.follow.add.<family>.<family2>`, `skills.follow.clear.<family>`; text audit clean at the four sizes) |
| 16 | Quest journal | `journal_model.gd`, `journal_view.gd` | `hub.rs` `Journal` | done: groups, resolved text, progress, current step, rewards, loot choices |
| 18 | Battle results | `battle_results_model.gd`, `battle_results_view.gd` | `hub.rs` `Results` | done: headline and note, the stay-behind lines (ENCOUNTER2 W4), the party lines (ROSTER1-U: "`{name}` was captured." / "died." per departure, "Keepsake: `{item}`, to the stash.", "Wounded until the next rest: ..."), XP rows with bars, gold / loot / quest lines, level-up prompt; battle report table not yet |
| 23 | Battle | `ui/combat/*`, `scenes/combat_arena.gd` | `battle.rs` | done: arena (catalog or world encounter) and world battles with the saved party at stored HP, HP write-back, runner settlement; enemy tells while planning (PORT-INTENT: kind chips and arrows on the grid, "Next:" on the hostile cards, Intent, revealed plan and forecast in the hover card; I toggles, the Show enemy tells setting); the timeline's masked enemy banner is not drawn (the engine's strip shows the selected hero only); units animate (SPRITE1-U): idle, walk while sliding to the new tile, the skill's attack clip, hurt or hurt_guarded, death (held, then the x marker), defend then idle_guarded; each stands on a flat ellipse in its side's colour (ally blue, foe red), the selected hero has a 1px gold outline around its sprite and a hovered unit a faint white one (no disc, no gold cell square); units face their target (FACE1): a unit turns toward the target or area aim of the action it resolves and toward the x of its step (a vertical step or a target in its column keeps the facing); while planning it shows its planned (hero: first targeted step) or intended (enemy tell, only while the tells are shown) target; a battle starts with the party facing right and the foes left; the frame and its outline are mirrored while facing left; animation speed and skip apply (skip ends in the same facing); bodiless units keep the letter panel; a phase talk opens the region map's talk box over a 35 % veil (ENCOUNTER2-C2, section "Battle talk" below); timeline battles take the one-flow input with selection and inspect (INPUT1-U: a right click inspects any unit and pins the threat intel panel; section "One-flow input" below) and the AP pip gauge beside the strip (INPUT1-AP); a calmer board: one quiet target arrow per unit while planning, bars for the units in focus, the first-move chip behind a setting (VISCLAR1-A, section "Calmer board" below); the full timeline view on key T over the grid panel, every unit's track with enemies masked to what the party knows (INPUT1-T, section "Full timeline view" below); playback aids: impact tracers, the strip's block cursor, speed 1x / 0.5x / block by block (P, N), the log by round and block, the open view pausing the playback (INPUT1-PB, section "Playback aids" below) |; hero and intel panel portraits (PORTRAITFX1-P: 32 px, reacting at the FX1 impacts; state `portraits`)

## Dev tools (dev-only, never in the real game)

| # | Screen | Godot | Engine | Status |
| :--- | :--- | :--- | :--- | :--- |
| 19 | Content Forge (catalog viewer) | `ui/forge/*` | `forge.rs` | read-only viewer: kinds, entries, every field, engine summary, used by; no search, validation or editing yet |
| 20 | World Forge (world viewer) | `ui/world/*` | `forge.rs` | read-only viewer: world picker, overview (inheritance, connects, counts), merged entries with origin, used by |
| 21 | Gear & Skill Dojo | `ui/dojo/*`, `scripts/dojo/*` | `dojo.rs` on `emberwake_battle::dojo` | done (PORT-DOJO): the Godot dojo on a desk of engine windows (party with the carry bar, unit inspector with stat allocation, derived stats, gear with the pouch and quick-setup presets, stash with filters, stage, console, test log with breakdowns), drag and drop between the stash, the slots and the pouch, Godot's three layouts saved per player |
| 22 | Scene Runner | `ui/runner/*` | `runner.rs` | the engine's launcher (main menu Tools): arena on any encounter, forges, dojo, Exploration (RUNNER2: world and start pickers; fights Skip / Fled / Lost / Fight it), UI Gallery, data reload; the scene list scrolls; playtest recording not yet |
| - | UI Gallery | none (new) | `gallery.rs` | the UI one element at a time for review (UIGALLERY1), in four groups: Kit widgets (every kit widget in its states, usable), Pip gauge (INPUT1-AP: the engine's pip gauge alone in its sample states with their readings, numbers on hover), Timeline pane (INPUT1-T: the engine timeline widget alone on its sample round, no game or battle: two sections, masked tracks with notes and marks, parts, a link, a ghost, hatching, the playhead, a selected block and track; the rows scroll); Art: Portraits (PORTRAITFX1-G: every portrait of the game and kit layers with its expression frames, drawn through the game's binding with the effect toggles, reactions, step events, one control per cue row, size, reduce motion, anchors and the expression picker; its sidecar summary); Screens (each screen shrunk into the pane on a sample debug game, dialogue and shop opened on the first person who offers them, the Mission board pane on a sample board (STORYUI1-U2), Battle talk: the Test Arena's battle with the sample world's first dialogue tree open over it (ENCOUNTER2-C2), Open shows the screen itself); Battle panels (plan strip, party list, hero inspector, battlefield, hostile list, threat intel, status and log, each the battle screen's own panel enlarged), and the battle timeline on a live arena fight (the battle's full timeline view through the same builder, INPUT1-T: Party and Enemies tracks by block, an unrevealed enemy masked while planning with its intent, target and tell, Reveal to show them, < > through past rounds, a block's snapshot of every unit, a segment's action detail) |; the UI one element at a time for review (UIGALLERY1), in four groups: Kit widgets (every kit widget in its states, usable), Pip gauge (INPUT1-AP: the engine's pip gauge alone in its sample states with their readings, numbers on hover); Art: Portraits (PORTRAITFX1-G: every portrait of the game and kit layers with its expression frames, drawn through the game's binding with the effect toggles, reactions, step events, one control per cue row, one box per expression-effect row (PORTRAITFX1-EX: holds that expression's tint, mark and shake), size, reduce motion, anchors and the expression picker; Battle panels (plan strip, party list, hero inspector, battlefield, hostile list, threat intel, status and log, each the battle screen's own panel enlarged), and the battle timeline on a live arena fight (all units' tracks by block, enemies masked while planning as "Plans in secret" with their tell, Reveal to show them, < > through past rounds, a block's snapshot of every unit, a segment's action detail) |

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
   are retired (CLN-42).
2. Game state: GameState, NewGame, SaveGame (saves below the current version are refused, CLN-47), party and progression rules.
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
  PORTRAITFX1-X: the card's `expression` picks its frame the same way (an expression frame without its own sidecar
  uses its base portrait's anchors); the agent state's portrait shows the expression.
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

## Battle talk (ENCOUNTER2-C2)

- A phase `talk` (`docs/specs/ENCOUNTER2_C.md` 3.3) opens over the battle once the impacts of the step that raised it
  have played: a 35 % black veil over the board and panels, the region map's talk box above it (`talk.rs`: 400 px
  times the text size, centred 6 px above the bottom, the living portrait, numbered replies, Continue), the speaker's
  token in the gold selection ring (the unit whose hero key, or world character, is the node's speaker). A round-1
  talk opens before round 1 is planned.
- While it is open the screen sends no request: no step, no planning, Confirm and the playback controls do nothing,
  autoplay's plan waits; every board click, battle key, Esc and right click is swallowed; hover cards stay. Keys 1-9
  pick a reply, Enter or Space continue.
- Replies apply their flag effects to the game in play (a world fight) or to a scratch game state of the Arena's world
  (dropped with the battle). Closing returns to planning, or the playback resumes after `impact_gap_ms`. A talk that
  cannot open is skipped with its reason in the message bar. The spoken lines stay out of the combat log.
- Agent link: `state.talk` `{dialogue, node, speaker, text, replies [{id, text, locked}], portraits, speaker_unit}`
  while one is open (absent otherwise); the box keeps the region map's widget ids (`dialogue`, `dialogue.text`,
  `dialogue.choices`, `choice.<i>`, `dialogue.continue`) plus `talk.backdrop`.

## Travel modes (CAMPAIGN1-TM)

- World map, in a campaign world whose manifest names `travel_modes`: under the title line the mode button
  (`world_map.mode`, "Travel: Stealth", the default first) cycles the modes in data order and logs "Travel mode:
  Forced march."; its hover card gives the mode's tip, its days a leg ("Each leg takes 1 day more.") and the ambush
  change ("Ambush chance x50%."). The Travel hover card starts with the mode's name ("Stealth: 4 days, 4 Tallow.");
  road labels show each leg's days in the mode. A trip whose forced-march Alert would end the run asks first (No as
  the default). Without modes there is no button and nothing changes (the Ashlands).

## One-flow input (INPUT1-U, timeline battles)

- Click a target while a hero plans: an enemy, an ally, an empty tile, or the hero itself (the hero panel's Self
  button, `hero.self`, or S). The click sets the round target at once (the gold ring) unless a pick holds one, then the
  skill menu opens at the click, titled "`<hero>` -> `<target>`": the hero's skills that could act on that kind of
  target in hotkey order ("1 Heavy Cleave", detail "4 blocks, 4 AP; Assault"; a Maneuver attack once per kind, "Maneuver
  (Engage)"), the pouch items of that aim, Rest; a tile adds Move here, "Move here, then" each Guard and self skill,
  the area skills and, after a skill pick, Step here after. A row that cannot be picked now is dim with its reason
  ("AP 3 < 4", "Out of reach: 3 > 2", "Flank protected"); a row whose intent change drops an intent-bound buff has a
  red corner and the tip "Changing loses Riposte.". An ally's menu ends with "Select `<ally>`" (with no usable row the
  click selects the ally).
- Picking a row appends the pick; the first skill or item pick sets the round intent and Maneuver kind; the menu stays
  in place and lists only what can still be picked (it closes when nothing is left). Hovering a row previews it on the
  strip (and, for a Maneuver Engage row, rings its target's position tiles: gold preferred, dim fallback). A right
  click, a click outside or Esc closes the menu and does nothing else. Keys 1-9 pick the listed row with that digit,
  or with the menu closed that hotkey skill on the round target. Z (or a right click with no menu) undoes the last
  pick, Clear removes them all.
- The planner fills the free blocks as ghost blocks (the engine's ghost look: lighter fill, dashed outline, dim
  label; the tip starts "Auto-filled"); Settings, Gameplay: "Auto-fill the rest" (off: the free blocks Rest). A hero
  with no pick shows its hero tactic's draft as its ghost (the Party / Hero tactic menus set it); after a target click,
  the auto plan on that target. A ghost that would change the intent and drop an intent-bound buff waits for Confirm
  (the panel warns "Changing loses ..."). The hero panel reads "Intent: `<Intent>` from `<first pick | tactic>`" (or
  "(carried over)"), the target, the picks and the planned commands (the ghost dim). The intent buttons, the kind and
  opener menus, Manual and Fill are gone (the host keeps `round_intent`, `manual` and `fill` for recordings, sims and
  the Dojo).
- Selection and inspect (INPUT1-U slice 3): with a hero selected and planning a click on an enemy or ally targets it;
  a click on the selected hero (its token or party card) selects nothing; with no hero selected, or outside planning,
  a click on an enemy inspects it and one on an ally selects it (a party card always selects). A right click on any
  unit (token or card, either side, the selected hero too) inspects it and changes nothing else; elsewhere it undoes
  the last pick; with the menu open it only closes the menu. The inspected unit pins the threat intel panel, headed
  "Inspecting `<name>` (Esc clears)", until another inspect, Esc or the next round (playback keeps it), and its
  target arrow shows beside the selected hero's (VISCLAR1-A; before it, its revealed moves showed as a hovered enemy's
  did). An enemy shows only what the party knows: its round intent and target,
  HP, poise, statuses, reach, facing ("Facing west"), Hidden, the tell's first action and its forecast only while the
  tells show (I), its plan only when revealed, the selected hero's planned hits on it; never an unrevealed plan's
  skills (the hover card's classic "Next:" line is left out). A party member adds its AP numbers ("AP 10 of 10",
  "Plan: 7 (picked 6, auto-filled 1)", "Left: 3", "Next round: +4 -> 7 (Rest +0)"), "Picks: ..." and one line a planned
  entry ("B2-5 Heavy Cleave on Wolf", ", auto-filled" on a ghost). Esc closes the menu, then the log, then clears the
  inspect, then selects nothing, then (Arena) leaves.
- AP gauge (INPUT1-AP): `strip.ap`, after the last block, shows the selected hero's AP as pips (12 at most; above 12 AP
  a pip is 2 or more AP, "1 pip = 2 AP" in the tip): full pips are AP kept, hollow ones AP the plan spends, a ghost
  fill next round's gain; hovering a row refused for AP turns the outlines red with "-1" after the pips; a debt shows
  "-2" before empty slots; the tip has the AP numbers. During playback the paid pips go hollow over 150 ms and the
  regained ones fill at the next planning (at once with skip animations or reduce motion).
- Agent link: `state.picks` [{unit, picks}], `state.deferred`, `state.player_target`, `state.inspect` (unit or null),
  `state.ap_gauge` (pips, red, label), the menu as `popup` (rows `popup.<k>`), `overlay.positions` while an Engage row
  is hovered; requests `pick`, `unpick`, `clear`, `autofill`.

## Calmer board (VISCLAR1-A, timeline battles)

The round intent's board only (classic and a timeline battle without `round_intent` keep today's board).

- Target arrows: each unit draws at most one dashed arrow (4 px on, 3 off, a 4 px head, alpha 90; party gold, enemy
  red; Flat 1 px wide, Stage 1.5 px on the ground under the tokens) from its tile to its round target (an enemy without
  a round intent: its tell's target); a unit or tile target ends there, a self target at the end of its move (no move:
  no arrow). A move leg bends it where the unit's moves before its first command end (its walks, then the carried move
  of its first command): a hero's always (group `moves`), an enemy's only when its plan is revealed (group `walks`);
  an unrevealed enemy's goes straight. By default only the selected hero's arrow and the inspected unit's show; hold
  Alt for every unit's; an enemy's only while the tells show (I). Arrows draw only while the party plans.
- They replace the red tell arrows, the gold planned-move lines (walk, carried, after-move) and the revealed enemy
  walks (also the hovered enemy's walk above the tokens). The destination plate of each planned move stays while
  planning (an after-move shows only as its plate); during playback no arrow, plate or area tell plate draws (the arena
  telegraph stays). The facing chevrons (pale yellow) and the gold target ring stay.
- Bars in focus: the HP and poise bars under the tokens draw for the selected hero, its round target, the unit under
  the mouse (token or card) and the inspected unit; hold Alt for every unit's. The party and hostile cards keep theirs.
- Settings, Battle: two new overlay groups, off by default: "Enemy first move" (`first_move`: the tell's kind chip on
  each enemy token, a debug view; the intent glyph and the hostile card's first-action line stay) and "All bars"
  (`all_bars`: every unit's token bars, today's look). The rows sit at a 14 px pitch (15 before INPUT1-PB's 25th group) so the debug row fits; a debug
  build adds "Debug" with "Party foresight off" (`settings.debug.party_foresight_off`, key
  `debug_party_foresight_off`): every plan a party foresight passive reveals (Hawk Eye) draws as unrevealed on the
  board, the hostile cards, the hover cards and the intel panel, and the arrows' move legs go straight; enrage and the
  world's plan reveal still show. Release builds have no row and never read the key. Controls: "Every bar and arrow
  (timeline): Hold Alt".
- Agent link: `overlay.arrows` [{unit, points [[x, y], ...], alpha}] as last drawn (an unrevealed enemy's has two
  points), `state.focus` [units] (the focus set as last drawn); `overlay.revealed_moves` is gone.

## Full timeline view (INPUT1-T, timeline battles)

A battle beyond classic only (classic keeps its screen; no `timeline_view` key in its state).

- Key T, planning and playback, opens the view over the grid panel's rect (126, 28, 388, 292): the engine timeline
  widget 4 px inside it on `timeline_view.rs`'s builder. T or Esc closes it (Esc first, before the menu, log, inspect
  and selection steps). While open the board under it takes no click (a backdrop over the panel); planning pauses
  nothing; T does nothing while a battle talk is open. During playback it pauses the playback at the end of the
  resolving block while open (PB5, section "Playback aids").
- The ruler of the round's blocks (the playhead during playback), "Party" (allies in roster order, guests too) above
  "Enemies" (the hostile list's order), one track per unit present or acting this round; a downed unit's lane hatched.
- A party track: the member's plan while planning (picks solid, the planner's ghost in the ghost look, wind-up,
  impact and recovery parts; a selected hero's hovered menu row or waiting ghost as the strip shows it), its committed
  actions during playback (BREAK marks, a cancelled span hatched); the label leads with the intent's letter ("A
  Vanguard"), its hover card the AP lines of the gauge's card; the selected hero's track is selected; a click on a
  party track (or its span) selects that hero, the view stays open.
- An enemy track: while planning an unrevealed enemy is masked with "`<Intent>`: `<target>`" ("Plans in secret"
  without a round intent) and, while the tells show, "; Next: `<skill>` `<blocks>`"; its impact badges as marks (red
  "!" a hit, gold "*" anything else). During playback an unrevealed enemy holds only its actions that have started; a
  revealed one (foresight, enrage, the battle's plan reveal) holds its whole plan, its card "Plan revealed (Read by
  ...)", "(enraged)" or "(plan reveal)". The debug Party foresight off masks foresight reveals again.
- Text: labels and notes shorten to fit at the four text sizes; their hover cards hold the full text. 16 tracks fit
  before the widget scrolls. Controls: "Full timeline view, open / close (timeline): T, Escape".
- Agent link: `timeline_view` (null: closed) {blocks, playhead, selected_track, sections [{title, tracks [{key, label,
  masked, hatched, segments [{key, start, end, label, ghost, hatched}], marks [{block, glyph}]}]}]}; widget ids
  `timeline.backdrop`, `timeline.block.<b>`, `timeline.track.u<unit>`, `timeline.seg.u<unit>.<k>`,
  `timeline.mark.u<unit>.<k>`.

## Playback aids (INPUT1-PB, timeline battles)

A battle beyond classic only (classic plays, paces and logs as before; no `playback` or `log_groups` key).

- Impact tracers: at each impact a straight line from the actor's token to each unit its action landed on (a hit or a
  stagger, each unit an area caught, a riposte back to the attacker; a support or item on another unit in the heal
  colour; an area's hazard to its tile), party gold, enemy red, 1 px Flat, 1.5 px Stage, above the tokens; a protected
  flank or a lost target draws it dashed; none for a self command. 400 ms at alpha 200 fading out (animation speed and
  0.5x apply; skip animations: when the step shows; reduce motion: no fade). Settings, Battle: group "Impact tracers"
  (`tracers`, on).
- Strip cursor: from a block's first step until the next block's, a 1 px gold frame around its column on the strip
  (its span edged gold); none from Confirm to the first step and once the round is over. The timeline view's
  playhead marks the same block.
- Playback speed: during playback the chips "1x", "0.5x", "Block" (and "Next block" in Block mode) sit where the
  tactic menus are; P cycles them, N plays the next block; setting `playback_speed` (default 1x; Settings, Battle:
  "Playback speed" at the overlays heading). 0.5x doubles the walks, clips, floats, tracers and the pauses between
  steps and rounds (skip animations plays it as 1x). Block: after each block the playback waits, "Block `<b>` of
  `<n>`: N for the next"; autoplay waits too, also before the next round.
- Combat log: the expanded log lists "Round `<r>`", then "Round start", "Block `<b>`" and "Round end" headers (dim, a
  1 px rule) with their lines 6 px in; during playback the compact log shows the resolving block's last line ("Block
  `<b>`: ...", the block's lines on its card).
- Timeline view (T) during playback: the playback pauses at the end of the resolving block while it is open ("paused
  while the timeline view is open"); closing it resumes; in Block mode N steps with it open.
- Text: the chips, the log's headers and lines pass the text audit at the four text sizes. Controls: "Playback speed:
  1x, 0.5x, block by block (timeline): P", "Next block (timeline): N".
- Agent link: `playback` {speed, waiting, block}, `overlay.tracers` [{from, to, alpha, dashed}], `log_groups`
  [{round, block, lines}]; widget ids `playback.speed.1`, `playback.speed.half`, `playback.speed.block`,
  `playback.next`, `settings.playback_speed`, `settings.overlay.tracers`.
