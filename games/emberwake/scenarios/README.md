# Emberwake scenarios: notes for writing them

The format and the commands are in `engine/docs/authoring/scenarios.md`. These are facts about this game, measured 2026-11-04 (INTEG1-E1) in a real window and
2026-11-05 (HEADLESS1) with none. A run has no window by default; none here needs one. The gate runs them all (PR7-4); a baseline change for a golden shot
needs the user's approval (`engine/docs/authoring/scenarios.md` "Golden shots"). `saves/hub_start/` was made with the agent
playtest harness (`tools/fc.sh playtest`) after the prologue; make it again the same way when a save change breaks it.
`saves/m3_stuck/` is the agent systems playtest's save of 2026-11-18 (PT-15), kept as found: it must load through the repair.
`saves/cages_start/` is the same playtest's save at the Ash-Crawl Crossroads (PT-16), kept as found: its fight's first auto-filled plan was refused.
`saves/sweep_start/` is the same playtest's save at Alleyway Junction 7 before The Sweep (PT-17, PT-20), kept as found.
`saves/after_vats/` is the same playtest's save after The Tallow Vats, heroes at level 7 and 8 (PT-28), kept as found: it loads with the repair notice and Elyse's recruit talk.

## The scenarios

| File | Walks | Time, dev build |
| --- | --- | --- |
| `menu_opens` | The main menu to the new game screen and back | 0.8 s |
| `timeline_fight` | `tl_flank` on the Timeline ruleset, autoplay on screen to a win, with a clip | 13 s |
| `story_battle_hub` | A new Sunk Ward game: the opening vignette, the battle beat (TRANSITIONS1 T1), the first fight won by the link's `autoplay`, the victory beat and the results over the field (T2), then the prologue's vignettes and fights to the hub | 18 s |
| `battle_report` | A new game's first fight won by `autoplay`; the results show the battle report (`state.report`, the best-moment line), then Level up now opens the character sheet and Escape comes back to the report (`PR7-32`, PT-13) | 10 s |
| `screens_menus` | Review shots (PR7-4; golden once the user approves the baselines): main menu, new game, load, the five settings tabs, the Scene Runner | 3 s |
| `screens_hub` | Review shots from the save `saves/hub_start` (the hub right after the prologue): region map, talk box, notice board, shop, party (stats, skills, gear), world map; plus the game menu and its Party in its place (`game_menu`, `game_menu_party`, PR7-48) | 25 s |
| `screens_battle` | Review shot: `tl_flank` seed 7 at its start, planning | 4 s |
| `job_fight_repair` | PR7-41 (PT-15): the save `saves/m3_stuck` (The Bone-Mill Raid won before it was taken) loads with the repair notice and the job ready; travel to the hub, turn it in to Talia | 6 s |
| `cages_auto_plan` | PR7-41 (PT-16, PT-22): the save `saves/cages_start` enters The Crossroads Cages; Confirm takes the untouched auto-filled plan and the round plays | about 10 s |
| `sweep_telegraph` | PR7-41 (PT-17, PT-20): the save `saves/sweep_start` enters The Sweep; with Vesper selected the Footman's card names the block its Heavy Thrust lands; round 2 marks Gilded Sunder's tiles | about 25 s |
| `fight1_pommel_hint` | PR7-41 (PT-21): a new game's first fight; the untouched auto plan's Pommel Strike breaks a Heavy Thrust before it lands (the round 1 log) | about 25 s |
| `ally_support_menu` | PR7-41 (PT-23): the save `saves/cages_start` enters The Crossroads Cages; with Hollis selected the hero panel's Ally button lists the members he can use his pouch item on, and a row opens his skill menu on one | about 10 s |
| `party_keystone` | PR7-41 (PT-28): the save `saves/after_vats`; the party screen's Skills tab shows the deck passives and the keystone slotted, with where they come from | about 5 s |
| `battle_menu_retreat` | PR7-41 (PT-32, PT-25): the save `saves/cages_start` enters The Crossroads Cages; the strip is disabled under the intro talk, the bar has no Restart or Step while planning, Menu offers Retreat; Yes ends the fight as fled and the results count the captives | about 10 s |
| `hub_text_audit` | PR7-41 (PT-30, PT-34, PT-35, PT-36): the save `saves/after_vats`; the stash cards say what key items are for, the hub menu, the save slots' whole labels and Quit to main menu asking first | about 6 s |
| `feel_battle` | ENGINE-46 (FEEL-FIX1): the save `saves/cages_start` enters The Crossroads Cages; `feel` steps time the planning picks and Tab against 100 ms an input, the strip's block pick against 50 ms (ENGINE-47), then a round resolving; 33 ms a frame throughout (a dev build: 2.5 times each budget) | about 15 s |
| `feel_settings` | ENGINE-46: the settings screen opened, every tab, back, timed against the same budgets | 2 s |
| `reach_menus` | ENGINE-48 (INPUT-REACH1): the main menu, the load slots, every settings tab and Controls walked by keys and pad with `fail` on; back closes the load slots and Controls, and does nothing on the main menu (allowed: the root screen) | 3 s |
| `reach_party` | PR7-92 (INPUT-REACH2): the save `saves/hub_start`; the party window's stats, skills and gear tabs, then the bunk loadout from Maren's footlocker, walked by keys and pad with `fail` on (every slot picks, places and empties by key) | about 5 s |
| `reach_battle` | PR7-92: the Test Arena (`tl_flank`, seed 7): the battle plan before and after a hero is picked, and a round resolving, with `fail` on; the board is one Tab stop (`focus_group`) | about 4 s |
| `walk_feel` | ENGINE-49 (WALK-FEEL1): the save `saves/hub_start` in the Sluice Undercroft at 60 frames a second (`step_ms`): a click walk and a second click mid-walk, then held keys, taps, the d-pad, turns, two keys at once, a 250 ms frame, a press during a 250 ms hitch (`stall_ms`, ENGINE-50) and the walls and edges of the map, each timed by `feel` with `motion`; budgets: the first move on the next frame (17 ms), no step after a release, the beat even to a frame (so no quick step after the hitch), no stutter, the camera within a tile and no jump; frames (ENGINE-50 MAP-FRAME1): the median under 16.7 ms and none over 33 ms (2.5 times in the gate's dev build) | about 25 s |
| `switch_feel` | ENGINE-51 (SWITCH-FEEL1): the save `saves/sweep_start`, each screen switch timed by `feel` with `switch`: the title's Continue to the map, the party screen, the hub menu, its party, Save and Settings and back, a walk to the world map and Enter back, The Sweep from the map (no intro), the results and back to the map; budgets on each switch's own part: menus and popups 150 ms, maps and fights 400 ms, no frame over 100 ms that no transition covers (2.5 times in the gate's dev build) | about 80 s |
| `switch_feel_story` | ENGINE-51: a new game: the title's Settings, Load and New Game, Sunk Ward started, the story cards line by line into the first fight and its battle beat, the results and on to the next story card, with the same budgets | about 45 s |
| `controls_rebind` | PR7-44 (CONTROLS-UI1): Settings > Controls; a row click then `k` rebinds Walk up, Esc cancels, `z` asks to swap with Undo and Swap trades, Pad Y for Help is refused (no button to trade), Confirm and Back swap Pad A and Pad B, Reset to defaults drops the map; shots and the text audit of the tab and the question | about 3 s |
| `tallow_shop` | PR7-42 (PT-19): the save `saves/sweep_start`; the Candle-Maker's first choice opens his window selling Warded Tallow at 5 Scrap, and one is bought | about 5 s |
| `battle_board_marks` | PR7-40 part 3 (BATTLE-UI2 contract 6-8): `kit_winch_escape` on the Timeline ruleset, picks by the link; round 1 a Snare Wire on the Warlord's tile, round 2 a Shield Bash push and a Tend Wounds heal; shots of the heal "+n", the status chips, the hazard with its rounds left, the push to its landing tile and the row skill menu on a strip block | about 5 s |
| `boss_bar` | PR7-54 (BOSSUI1, contract 9): Captain Vane's fight (`scene=sunk_ward:enc_evt_01`, the arena's world encounter form) on the Timeline ruleset; the boss bar with its phase markers and passive tags, Kaelen's intent row while planning, the member hover with the Party line, the bar folded to 84 px once confirmed | about 10 s |
| `one_click_talk` | PR7-46 (PT-35): the save `saves/hub_start`; one click on Berek, then on Sabina, walks there and opens the talk; PR7-48: the walk rings the NPC's tile (shot `sabina_walk_ring`) | about 10 s |
| `vn_stage` | PR7-27 (VNSTAGE1): a new game's prologue on the story stage; Kaelen walks in left on the meat market's scrolling backdrop, the narration keeps it dimmed while he walks out right, the Log and Auto; the mortuary cart's pan with the party walking in, the captives pinned right (Hollis on the code figure until his full-body picture exists) and all four walking out | about 15 s |
| `vn_stage_act1` | PR7-66 (VNSTAGE1B): every Act 1 story scene with speakers on the story stage, played from the UI Gallery's vignette pane: the three hub debates and the awakening fissure interlude (the party walks in left and out on the narration, the council stands right) | about 40 s |
| `diorama_stage` | PR7-64 (DIORAMA1): Sunk Ward fights on the world's stage set `sunk_ward_stage`; the alley with burn, poison and slow hazards from the dev link (`debug_ground`: their painted decals) and a root one (no decal: the tint), then the canal fight | about 10 s |
| `diorama_zones` | PR7-74 (DIORAMA4): Guild-Master Orlov's fight in the tallow vats (`scene=sunk_ward:enc_evt_04`); the zone's look from `stage.fx.json` `zones.tallow_vats`, the backdrop band under the boss bar, the foe card off the party's cells, see-through units in front of the selected one | about 10 s |

The three screen scenarios also time frames (`perf`, ENGINE-2, numbers only, no budget): the main menu, the region map and
battle planning. Measured 2026-11-18 at 1280x720 in a dev build with no window: p50 4.6 ms, 29.5 ms and 20.2 ms.

A battle's `state.beat` is there only while a beat runs (`kind` `intro` or `victory`); wait for it to be absent before the fight takes input. A battle talk can open after the link's `autoplay` and holds the victory beat: Enter moves it on.

The main menu's screen id is `main_menu`.

## Fights

| How | What it does | Use it for |
| --- | --- | --- |
| `{"step": "key", "key": "a"}` | The battle's own autoplay: rounds play on screen, with their timing | Proving the fight on screen |
| `{"step": "link", "body": {"cmd": "autoplay"}}` | Resolves the whole battle in the host in one request: no playback, and the screen's message stays as it was | Reaching an outcome fast |

Wait for the end with `{"step": "wait", "until": {"path": "state.outcome", "ne": ""}, "timeout_ms": 240000}`: `state.outcome` is empty
until the fight ends. `tl_flank` on the Timeline ruleset took about 11 s on the key in a window, and 12 s with none at 15 fps, in a dev build.
A fight on one ruleset: launch with `{"screen": "arena", "scene": "tl_flank", "ruleset": "timeline"}` (`"encounter"` and `"rules"` are accepted as file load aliases; CLI launch and `shots` accept `encounter=` but refuse `rules=`). Any other `screen` with a `ruleset`, or an unknown ruleset, stops the game at launch.

## The Scene Runner's arena

| Fact | Detail |
| --- | --- |
| Screen id | `test_arena`, not `battle` |
| Ruleset | Pick it in the `runner.arena.ruleset` select. `screen=arena` with no `ruleset` opens the fight on the Classic rules, even for a `tl_` fight |
| Options | `<select id>.opt.<n>`, in list order, so an option's place moves when the data changes. A click on the select replies with the list |
| The end of a fight | `banner.leave` and `banner.restart`. No `battle_results` screen appeared |

## Comparing with a real window

Measured 2026-11-05 at 1280x720, dev build, `tl_flank` seed 7 on the Timeline ruleset.

| Compared | Window and no window |
| --- | --- |
| Main menu; fight start with `reduce_motion` | Byte for byte the same |
| Fight start with motion | 2,653 pixels within 1 px: idle clips and portraits at another moment. Two runs with no window also differ, by 1,100 |
| Fight end, 2 frames after the outcome | 9,344 pixels: the playback's last moments, shot a little apart in time |
| Fight end, 3 s later | 484 pixels: only the bodies' idle frame, faint behind the banner |
| Result | The same: `won` in 2 rounds, Vanguard HP 78/210, Spiker 210/210, all log lines, 33 requests, both checkpoints |
| Playback, key to outcome | 12.3 s against 10.9 s (about 11 % long; 12.1 s in an earlier run): timers fire on the next frame, at 15 fps |

So a shot shows the same state, not the same animation frame; to compare pixels, use `reduce_motion` and a screen at rest.

`fps=` starts at 12 because the battle's effects clock takes at most 100 wall ms a frame (`battle.rs`, `tick`): at 10 fps the frame gap is 100 ms plus overshoot, and each frame would lose the overshoot.
