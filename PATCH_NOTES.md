# Patch notes

Every change to the codebase, newest first, one section per day (UTC). "In the game" is written for players and testers; the lists below it are the commit subjects. Kept by the `patch-notes-write` skill: `python3 tools/patch_notes.py` prints the commits on `develop` since the marker below, and whoever runs it (Merge & Release at each release cut, or anyone asked for patch notes) writes the new section and moves the marker.

<!-- last: 260ad9f -->

## 2026-10-20

In the game:

- **Gear tab**: three columns, like the gear dojo: the hero's stats (green or red for the item you point at or drag), the equipped slots with the battle pouch under them, and the party stash. Drag from the stash onto a gear or pouch slot to equip, drag a worn item back to unequip, or click as before; gear the class cannot use is refused. Two-handed weapons now take the off hand too, so equipping one removes an off-hand item and the other way round.
- **Settings now work**: window mode (windowed, borderless, fullscreen), window size, vsync, frame cap and text size (Small to Extra large) apply as soon as you pick them; the Character popup and Settings tabs re-flow for bigger text. Dropdowns open as lists and volume controls are sliders you can drag.
- **Sound and music**: each screen has its music (menu, map, combat and a boss track for boss fights) with crossfades; battles play hit, crit, miss, heal, defend, move and defeat sounds; volume sliders play a sample as you move them.
- **Keyboard play**: every screen works with Tab / Shift+Tab, the arrow keys and Enter, with a visible focus ring; dropdown lists and sliders work from the keyboard too.
- **Enemy intent**: in combat planning, enemies show what they plan to do again (chips, arrows and a short line), as before the engine move.
- **Battle log and report**: the action log, combo and round-end lines, and the battle report with its metrics are back.
- **Character popup**: every row (stats, derived values, level-up rows) shows its details in the side pane when you point at it.
- **Test Arena**: an Exit button leaves a battle.
- **Saves and settings** now live in the game's own folder (`%APPDATA%/Aegis/emberwake`); your old saves and settings are copied there the first time you start.
- **For testers (Scene Runner)**: the Gear & Skill Dojo is back as a sandbox of movable, resizable windows; the UI Gallery shows panes on their own, starting with the battle timeline; Playtests lists your recorded battles with Play back, Take over at a round, notes and Copy to repo, plus the recording toggle; press **F8** anywhere while recording to leave a note with a screenshot and tags (mechanic, ui, ux, bug).

Every change that touched the game:

- PORT-PLAYTEST C (game): Scene Runner Playtests, battle playback and take-over (7696d7d)
- PORT-PLAYTEST C (engine): F8 notes and the recordings browser (2096500)
- ASSETORG2: asset layers and reuse groups (decision and spec), PORT-PLAYTEST B docs (408c1bc)
- PORT-PLAYTEST B (game): battle events in the playtest trail (4b745e5)
- PORT-PLAYTEST B (engine): the UI trail and its report (cd0f412)
- PORT-PLAYTEST A (game): arena battles are recorded and replay (a295018)
- PORT-PLAYTEST A (engine): playtest recordings and replay on the command API (23f49c4)
- PORT-CHECKS (game): the Forge's content checks, message for message (3a7cd5e)
- PORT-CHECKS (engine): blocking and warning hooks for the content writer (a9cd6c9)
- PORT-FOCUS (game): the screens on the kit's keyboard focus (814d137)
- PORT-FOCUS (engine): keyboard focus on every screen (ba41988)
- PORT-SIM (game): every balance sim mode on the engine (8dc993e)
- PORT-SIM (engine): Godot number formatting in aegis_core (55ae4a3)
- PORT-BREPORT (game): battle metrics, the battle report and the action log (1b56559)
- TIMELINE1, ROUNDHIST1 (game): round archive and the UI Gallery's battle timeline (ff62fa6)
- TIMELINE1 (engine): block timeline widget in the UI kit (0e97eed)
- PORT-INTENT (game): enemy intent tells (1fd2f71)
- PORT-INTENT (engine): Painter::arrow (588f4a9)
- PORT-DOJO (game): the gear dojo's sandbox on a desk of windows (236011b)
- PORT-DOJO (engine): scroll areas, nested clips and rich text in the UI kit (c19a9a4)
- UIKIT3 (engine): windows in the UI kit (ba4054a)
- GEAR2H1 (game): a two-handed weapon and the off hand exclude each other (1473492)
- GEARUI1 (game): Gear tab in three columns with drag and drop both ways (26cbd8c)
- UIKIT2 (engine): drag and drop in the UI kit (93ab0ea)
- ASSETORG1 (game): Emberwake art in a folder per thing (0c73dea)
- ASSETORG1 (engine): art loader reads one folder per thing (a655c52)
- PORT-AUDIO (game): Emberwake music, battle sounds and volume samples (61683e8)
- PORT-AUDIO (engine): sound and music in aegis_ui (a28d8f8)
- PORT-SETTINGS (game): settings screen on the engine's choices; layouts follow the text size (221e2cd)
- PORT-SETTINGS (engine): the engine owns and applies display and text-size settings (b402d14)
- UIKIT2: drag and drop is an engine row the screens borrow (user 2026-10-20) (bce945f)
- UIKIT1 (game): settings, Scene Runner, World Forge and turn-in use the select and slider (062dfcf)
- UIKIT1 (engine): a select with its choice list, a draggable slider, hover outline on info rows (ec9ae0e)
- PORT-DOJO: the Godot gear dojo's sandbox as a task row (user 2026-10-20) (54b658f)
- DOJO2-B: the gear dojo lists the unit's skills and pouch slots (d3fd76b)
- PARTY2-B: every row of the character popup fills the detail pane (a748ebc)
- ARENA-EXIT: an Exit button in Test Arena battles (7115038)
- DRAG1: drag and drop both ways on the gear tab, then on other screens (403c7cf)
- PLAYTEST1: user playtest of 2026-10-20 as task rows and decisions (1f865cf)
- SPLIT1-9: process by layer; the game's design docs move to games/emberwake/docs/ (d8415fd)
- SPLIT1-8: games/emberwake/ holds the game; game.json; saves and settings under the game id (70b9083)

Under the hood:

- Planner: PORT-PLAYTEST done; PLAYBACK2 and ARTTOOL1 rows; art tools decision (260ad9f)
- Planner: UIKIT3 engine windows before PORT-DOJO; the port goal decision (dc88260)
- CI: install ALSA on the Linux gate (rodio's cpal needs it) (f8ca4d9)
- Planner: approve SPRITE1 and PHASEFOLD; status notes on stale specs (1c8293c)
- Planner: SKILLUI1 spec and answers, RUNNER2 and UIGALLERY1 rows (21b5e5e)
- CI1: the gate fails on compiler warnings; gates stay on main only (60a9764)

## 2026-10-19

In the game:

- **The game moved to its own engine.** Emberwake now runs on Aegis, a Rust engine built for it, instead of Godot. Every screen was rebuilt (menus, region and world maps, dialogue, shop, game menu, character, journal, results and combat), your saves load as before, and battles play out step for step the same as in the Godot build. The interface is drawn smoothly at your screen's resolution, with hover cards, tactic dropdowns and a scrolling battle log; battle items and reinforcement waves work as before.

Every change that touched the game:

- SPLIT1-7: game crates renamed emberwake_* and moved to games/emberwake/crates/ (fcf201a)
- SPLIT1-6: engine command API protocol and CLI shell; Emberwake registers its commands (82927f5)
- SPLIT1-5: engine UI aegis_ui (drawing, widget kit, art, tuning, agent link, screen stack, shell) (ff7771a)
- SPLIT1-3: engine content layer aegis_content (file format, schema, world graph, writer hooks) (f891b71)
- SPLIT1-4: engine save framework aegis_save (slots, envelope, migration runner) (898ae6a)
- SPLIT1-2: engine core aegis_core (RNG, hash, sort, Variant helpers, field readers) (e85f6a6)
- SPLIT1-1: the Cargo workspace moves to the repo root (7a8903f)
- SPLIT1: a game-agnostic engine and the first game, Emberwake (spec approved) (908e671)
- SPRITE1-A: the pixel-art party sprites reach main (from develop) (3d19060)
- feat(transition): step 2b, the Godot project leaves the root (e3f4d71)
- feat(transition): step 2a, engine tooling replaces Godot's (1412c1f)
- feat(transition): step 1, frozen legacy copy of the Godot project (d8b8587)
- feat(engine): read data/ directly, verified against Godot's catalogs (2c84faf)
- feat(engine): battle items with Godot trace parity (cce2e84)
- feat(engine): encounter rules with world battle parity (f85caa3)
- feat(engine): every Godot screen on the engine, with save compatibility (3e8d911)
- feat(engine): aegis_world game loop with Godot journey parity (a81379d)
- feat(engine): Godot hash() port and world/game-data exports (5aa17fe)
- feat(engine): smooth UI layer, hover cards, tactic dropdowns, scrolling log (3f529ae)
- feat(engine): aegis-gui battle screen with agent link and live tuning (2c0b84a)
- feat(engine): command API (aegis_host) and aegis serve (1b2be79)
- feat(engine): aegis CLI, reinforcement waves, encounter traces (50a4f20)
- test(engine): group and team battle traces; replay every saved trace (7445497)
- feat(engine): battle sim port replays Godot duels step for step (3198f63)
- feat(engine): bit-exact Godot RandomNumberGenerator (PCG32) port (f89bb25)
- feat(engine): rules port with exact Godot parity for units and damage (337476d)
- fix(engine): keep engine files inside engine/, pin gnullvm toolchain (37c2243)
- feat(engine): achieve Godot combat feature parity in aegis_app and web tactical GUI (26068d7)
- feat(engine): implement aegis_app WGPU desktop presentation shell with Taffy UI and web player interface (69f16d5)
- feat(engine): implement aegis_game standalone game instance runner with autosave and catalog loading (cb75d1c)
- feat(engine): implement aegis_bench comparing world generation, UI layout, and combat simulation (3637709)
- feat(engine): align simulation unit stats with reference game catalog data (9ff0a7f)
- feat(engine): implement aegis_sim headless Monte Carlo balance simulation harness (25d3b3c)
- feat(engine): implement aegis_vm execution engine and deterministic combat simulation (13e10f6)
- feat(engine): scaffold Project Aegis Rust engine with Markdown-first architecture and gd2md migration bridge (e29cde2)

Under the hood:

- TIDY1: SPLIT1-1 row archived, SPLIT1 in progress, lint fixes (2f97441)
- REPO-MOVE1: the checkout moves out of OneDrive (user 2026-10-19) (e427afd)
- RESCOPE1: task queue re-scoped to the engine; engine gaps as PORT rows (7d0c977)
- WORKFLOW-DEV1: develop-first on desktop (user 2026-10-19) (0ec760f)
- fix(ci): parity tests on a fresh checkout (952f214)
- docs: Godot parity is a migration check (user 2026-10-19) (623a32d)
- docs(transition): step 3, the process switches to the engine (a81e6fd)
- docs(engine): reference shots of every Godot screen and the replication plan (e6ed039)
- chore(engine): retire Markdown-language crates and old placeholder sim (1c43bbc)
- docs(engine): record user decisions on parity, scope and Godot boundary (8f295d1)
- docs(engine): add agent-native engine spec, supersede Markdown-as-language proposal (0eddf1f)

## 2026-10-18

In the game:

- New combat screen layout. A slim strip at the top shows your selected hero's plan as coloured blocks, next to the tactics pickers, Autoplay, Clear and Confirm round. Your party is on the left with a panel for the selected hero; enemies are on the right with one combined enemy card showing their state, stats, next move and what your hero would deal and take. The battle log runs along the bottom (click it or press L to expand).
- Confirm round now asks first if a hero still has empty action blocks. You can turn this off in Settings > Gameplay > Warn about missing actions.
- Picking a Party or Hero tactic fills any empty blocks for you (a basic strike in reach, otherwise Rest). Enemy HP now sits on the same line as their name.
- In the Ashlands, the ash scouts and ash raiders now lose heart as the fight turns against them. Kill half of them, or the raiders' smokecaller, and the rest throw down their arms and surrender, which counts as a win.
- Talking to people now opens a bar across the bottom of the map with their portrait and numbered choices (press 1-9). Locked choices say why. The shop is now a popup with Buy and Sell tabs, and shows who in your party can use each item. Both follow the text size setting.
- **Map screens**: the region and world maps no longer show debug tools in the real game; the top bar shows the place name and your supplies, and chest notes name the items inside.
- **Balance**: Shield Wall now also fortifies the user (4/6/8 by rank) alongside its taunt, and the healing potion costs 30 in the Hest shop (was 15).
- **Character screen**: one Character popup with Stats, Level up, Skills and Gear tabs; questions and the Gear board no longer hide under other in-game popups.
- **Map movement**: walk with WASD or the arrow keys, hold a key to keep walking, and the party slides smoothly between tiles.
- **Game menu**: Esc on the map opens a menu over the paused map with Party, Journal, Save, Load, Settings and Quit; New Game and save/load fixes.
- **Healing between battles**: HP now carries over between fights, you regain some after each win, and rest places and Make camp (3 supplies) heal fully. Rest places added at Warden Keep and Slag Gate.
- **Enemy intent**: in combat planning, enemies show what they plan to do (chips, arrows and a short line), and Bryn's Warden's Eye passive reveals more; enemies no longer skip their turn when a packmate's plan blocks theirs, and an enemy with nothing to do shows "Holding".
- **Boss phases**: Grakka changes phase mid-fight (Crown of Nails), with a phase banner in combat.
- **Battle objectives**: fights can be about more than defeat: hold the courtyard, the wagon raid with Wenn Tallow, slay Grakka.
- **Recruiting**: Bryn and five story recruits join through the campaign, with the Toller muster and new priest and brawler units.
- **Battle pouch**: carry potions and bombs into battle (Gear screen) and use them as a planned step; the free class potions are gone; a Healing Draught now heals 90 (was 25), a Fire Bomb hits for 45 (was 20), and the Ashlands start with 100 gold (was 50).
- **Combat screen**: compact layout, a hover card for units instead of the inspector, tactic dropdowns and an Autoplay toggle on the timeline row.
- **Battle autoplay**: let the party fight on its own using tactics, with safety stops.
- **Maps**: full-screen region and world maps with marker icons, labels, a legend and hover info; click your own pin on the world map to enter; the world map opened over a region map draws again.
- **Travel supplies**: spent on the road, bought in shops (Hest) and found in chests.
- **Campaign**: the Ashlands campaign "The Long Dry"; quests keep tracking across connected worlds.
- **Music and sound**: music for menus, towns, danger, shops, travel and bosses; volume sliders play a sample; Mute.
- **Settings and controls**: Settings is a tabbed popup you can open during play; a Controls tab lists every key.
- **Battle flow**: a transition into combat and a victory popup with XP bars, rewards, minimise and replay.
- **Enemy AI**: taunters step in early, healers look after themselves, and area attacks tolerate a little friendly fire.
- **Art**: Ashlands placeholder art (characters, classes, icons, tiles, maps, backdrops) and a second pass for chips and tiles.
- **Scene Runner (testing)**: F11 or the Fullscreen button toggles fullscreen; Minimize and Exit buttons; no runner pill over the real game; a Note button for playtest notes.

Every change that touched the game:

- COMBAT-UI3-F: drafts fill empty blocks, hostile HP beside name, compact strip labels (ce3c381)
- PURSE1-L: encounter purses pay gold and loot on a win (save v10); no fight has a purse until PURSE1-A data (d0bcc9a)
- COMBAT-UI3: combat HUD layout A (planning strip, merged enemy card, hero panel) (d2d1d64)
- PURSE1-L (schema): encounter purse field and validation (02980d3)
- PACE1-L: two-act boss behaviour (enrage by HP or round, empower buff, enraged intents shown, world start buffs, Forge warning); no boss uses it until PACE1-A data (a1ca665)
- MORALE1-A: enemy morale on ash_scouts and ash_raiders (69c82e0)
- UIR-6: dialogue as a full-width bottom bar, shop as a popup, text scale (c2de328)
- PACE1-L (schema): boss enrage phase, empower status, encounter pace and start_status fields; no combat effect until the behaviour commit (39daec9)
- MORALE1-S: world sims carry the five morale fields (f70e79a)
- MAPDEV1-U: map debug tools only in debug scenes; play bars show place and supplies (7780fe0)
- MORALE1-L: enemy morale in battle (losses, breaks, flee bonus, surrender, Fearless) (f344531)
- BAL5-A: shield_wall fortify extra grant and potion price 30 (744f175)
- BAL5-L: extra_grants skill field (second grant on DEFEND/SUPPORT skills) (49a8a70)
- BAL5-S2 + POTLEDGER: support_attrition scenario and potion heals in the sim ledger (b9fcc80)
- STATCRED1-L: battle report credits each status to the unit that applied it (3fa4cea)
- UIR-3: Character popup with Stats, Level up, Skills and Gear tabs (3878540)
- BAL5-R1b: re-measure under the re-draft fix and BAL5-AI (a990fb7)
- MORALE1-L (schema): enemy morale encounter fields, validation and preview (6a473ad)
- CAMPLABEL1-S: campaign sim fall label names the fight actually lost (c40a7dd)
- ITEMPOW1-A: battle items worth using (potion 90, bomb 45, start gold 100) (02c6bf4)
- POUCH-TIDY: meaningful no-pouch test and stale pouch comments (8c1494c)
- INTENT1-U follow-up: enemies no longer skip their turn; "Holding" tell (f65f0bb)
- BAL5-AI: early taunt with walk-in, healer self-care, AoE friendly-fire tolerance (7ceddcb)
- UIR-2: game menu popup over the paused map (Party, Journal, Save, Load, Settings, Quit), new game, save/load fixes (52dda97)
- HEAL1-B: campaign check passes (arms 46, gate 95) (ddbb56c)
- MAPKEYS1-U: WASD, held-key walking, smooth map movement, label fix (126f56f)
- BAL5-R1: trio re-measure; T2, S1 and A2 triggers fire (f3d8ac2)
- BREPORT1-L: battle report data (shared BattleMetrics collector, combo signal, BattleReport, results-model API) (d0219fa)
- WMAP-BLANK: world map opened over the region map draws again (41d2282)
- BOSS1-U: boss phase banner in combat (c14582e)
- PILL-GAME: no Scene Runner pill over the real game (f75b545)
- HEAL1-S: campaign sim keys heal=rally, rest=, extra= (47ef3b4)
- HEAL1-L: between-battle healing rules (stored HP, rally, rest places, Make camp, defeat cost) (0832b03)
- BAL5-SIM + BAL5-PH: trio sim tooling and generated boss phase (c78b8ba)
- HEAL1-A: rest places at warden_keep and slag_gate (message for 85ac473) (9d270f6)
- INTENT1-U: enemy intent chips, arrows and tell lines in combat planning (8d4971d)
- ITEMUSE1-A2: remove the free class pouch potions (fda40cc)
- HEAL1-L (1/2 schema): stored HP, last rest place, save v9, location rest (44e0641)
- SONNET-TRIAL: decision row, Assets & Audio runs on Sonnet as a trial (user card 08:21 UTC) (a499b3f)
- UIR-1: shared screen frame, button width classes, selected style, No focus on destructive questions (3e4f6d3)
- CLN-3b: drop legacy skill keys and version 1 skills loading (6e6abf7)
- ORSA-QUEST (answer): iron_for_embers stays hidden until the Iron Reaches campaign (439bca6)
- MODE-STANDARD: dispatch mode set to standard (user 2026-10-18 07:53 UTC) (261f9a7)
- RECRUIT1-B, OBJ1-B, BOSS1-B: world-encounter probes; BAL4-ATTR v4 (2fd535c)
- ITEMUSE1: decision row, no free class potion (user 2026-10-18) (9fcbaf4)
- EXPWORLD: a child world survives a trip through a connected world (74f4748)
- QUESTX: quests keep tracking across connected worlds (2d9db38)
- ITEMUSE1-U: battle pouch section on the Gear screen (069ac28)
- COMBAT-UI2-L3: tactic dropdowns and Autoplay toggle in the timeline row (bb98f4f)
- ITEMUSE1-L: use pouch items in battle as a planner step (e5653ac)
- COMBAT-UI2-L2: unit hover card replaces the combat inspector (1c6fea4)
- BAL4-ATTR: campaign attrition sim and report (15850e0)
- ITEMUSE1-A: use_reach on the potion, focus elixir and fire bomb (3048364)
- ITEMUSE1-L (schema): hero battle pouch, save v8, use_reach, item plan step (ad83b98)
- ART2-A: Ashlands placeholder art pass 2 (intent and objective chips, ring tile, miss sound) (0218602)
- OBJ1-A: courtyard hold, wagon raid with Wenn Tallow, slay Grakka (bda1aa0)
- COMBAT-UI2-L1: combat layout C (compact columns) plus UIR-5 leftovers (7f66209)
- OBJ1-L: battle objectives runtime, deployments, guests and verdict (3abdb63)
- BOSS1-A: Crown of Nails phase on Grakka's stands (1303031)
- INTENT1-A: Warden's Eye passive on Bryn (4205bc1)
- MAPREAD1-L2: map marker icons, labels, legend, badges, world pins and hover (73f31fa)
- RECRUIT1-A: Bryn and five story recruits, Toller muster, priest/brawler units, trio sizing (40cd863)
- OBJ1-L (schema): encounter objective, deployment and guest fields (2c9bc80)
- BAL3-R + CAMP1-B: final BAL3 rerun, band proposal, Ashmaw finale sim (e9477d9)
- BOSS1-L: boss phases fire in battle (5c85e7e)
- MAPREAD1-A: Ashlands marker labels, world-map pins, 48x30 maps (092c1d6)
- ART1-A: Ashlands placeholder art (characters, classes, icons, tiles, maps, backdrops) (98d6da5)
- BOSS1-L (schema): encounter phases field and validation (1cb608e)
- MAPVIEW1: full-screen region and world maps (cf333ab)
- MAPREAD1-L1: marker labels and notes, marker_view, location map_pos, world-map node extras (bf162bb)
- INTENT1-L: enemy intent read-out and the foresight passive effect (a56dda7)
- OPTLAUNCH1: remove its TASKS row (landed in 467dba2) (0bda9bd)
- OPTLAUNCH1: run.bat opt opens a design option branch in its own folder (467dba2)
- RECRUIT1-L: recruiting, hero names and the runtime party cap (0d6a3ae)
- MODE-ECONOMY: dispatch mode set to economy (user 2026-10-18 02:35 UTC) (267dee7)
- DESIGN-REVIEW-CYCLE: recurring design review with up to 3 option branches and a commit ledger (d12440c)
- CONTROLS-HELP: Controls tab, Settings during play, volume fix and Mute (3a2ede5)
- RECRUIT1-L (schema): manifest party_max, party characters in start_party, recruit effect validation (795aca1)
- MUSIC2-A + MUSIC2-L: town, danger, shop, travel and boss music, wired by location kind (5a1f677)
- WORLDMAP-ENTER: click the party's own pin on the world map to enter its location (0bd96e3)
- SOUND-PREVIEW: volume sliders play a sample at the new level (bf53760)
- MUSIC1-A + MUSIC1-L: five generated music loops and a context-based music player (a836119)
- POI-FIX: region map shows how to use chests, people and lore, and a click works too (e143307)
- AUTO1-L (logic): battle autoplay loop, party and hero tactics, safety stops, save v7 (f09c933)
- NOTE-BUTTON: "✎ Note" in the in-scene pill opens the playtest note box (68f5ac3)
- BAL3-A: data levers (sweep_kick), elite power 1.0, sim tank loadout (a3b7c35)
- BATTLE-FLOW: combat transition, victory popup with XP bars, rewards, minimise and replay; COMBAT-UI2 approved (layout C) (608d45a)
- TRAV2-A: Ashlands supply sources and start supplies (ba415d6)
- TRAV2-L: travel supplies are spent on the road, bought in shops and found in chests (0cfe7f5)
- CAMP1-A1 + CAMP1-A2: Ashlands campaign "The Long Dry" (boss unit and story data) (793d456)
- SETTINGS-POPUP: Settings as a tabbed popup, window changes on select, right-click back fixed (5bf8f5e)

Under the hood (specs, decisions, handovers, workflow, tooling):

- RELEASE-LOGS: shared logs for the evening batch
- Wind-down handovers: UI/UX (c5d9544, 46133e3), Gameplay (f4133b6), Balance (7643049, d9d8dcb), Assets & Audio (6378713), Design (088910b)
- INTERACTIVE: new interactive mode; work-starting routines paused (e6582f7)
- BALANCE: trio sweep bands approved; walk-in taunt answer (a673f83, 8971de9)
- MORALE1-B: morale balance probe report (7bda7b5)
- PATCH-NOTES-BATCH: patch notes once per release, not per landing (f1db72d)
- USAGE: 15:59 usage report and delta (fb133e2)
- BALANCE-HANDOVER: PACE1-S unblocked (6e72b3b)
- MODE-ECONOMY: economy dispatch window 15:45-18:45 UTC; mode-trial rows updated (6fb06ac)
- LEVERS1: Shield Wall guard rule and default-kit slot decided (L3, A3) (e0f1ae5)
- GAMEPLAY-HANDOVER: add LEVERS1-L3 to the queue (b0fda75)
- BALANCE-HANDOVER: rotation after BAL5 close-out (233dd98)
- BAL5-R (probe): Shield Wall barely gets cast in sims (not in the default loadout; AI uses it only below 40% HP), so the tank gap went to Design (7740b0e)
- BAL5-R: final balance rerun; Shield Wall fortify moved nothing, remaining gaps listed in the balance report (d4ab083)
- ENGINE-HOLD: keep threads off the engine branch; branch cleanup skips it (15c3a03)
- UI-UX-HANDOVER: queue and notes for the next UI/UX session (8d12e98)
- PATCH-NOTES: MAPDEV1-U, Gameplay handover (5a11e86)
- GAMEPLAY-HANDOVER: rotation after MORALE1-L, STATCRED1-L, UIR-2 (c87d94d)
- PATCH-NOTES: MORALE1-L, LEVERS1 approval (c23d235)
- LEVERS1-B: point the S2 dependency at BAL5-S2 (b9fcc80) (6c01248)
- LEVERS1 approved: priest Ward, brawler spin; task rows routed (0525027)
- PATCH-NOTES: BAL5-A, MORALE1 spec rule 8 (b2ef357)
- MORALE1 spec: flee bonus never lowers an authored ratio (rule 8) (c75bb11)
- PATCH-NOTES: BAL5-L (26e90d7)
- PATCH-NOTES: BAL5-S2 + POTLEDGER (5e034c6)
- PATCH-NOTES: STATCRED1-L (f38bc19)
- PATCH-NOTES: UIR-3, LEVERS1 spec (c8073e9)
- LEVERS1 spec: brawler spin and priest Ward (DR2-LEVERS) (8e414c0)
- PATCH-NOTES: BAL5-R1b, PLATFORM-HANDOVER (d241191)
- PLATFORM-HANDOVER: rotation after GATE-SPLIT (context 278k) (d54daa9)
- PATCH-NOTES: GATE-SPLIT (a817f3c)
- GATE-SPLIT: per-system gates; area threads gate with `fc.sh gate auto`; CI runs the arena as 8 parallel jobs by system, so a red job names the broken system (bd932b2)
- PATCH-NOTES: MORALE1-L (schema), ASSETS-PREFS (0c3869c)
- ASSETS-PREFS: sprite animation notes in the world asset preferences (b7e5e1e)
- PATCH-NOTES: CAMPLABEL1-S (c5c9ddf)
- PATCH-NOTES: marker fix, PURSE1, DR3, release #39 (ff2247c)
- PURSE1: approved (purse all recs), decision and task rows (e1662da)
- DR3-ANSWERS: design review 3 approved (all recs) (bd6f885)
- DESIGN-LEDGER: design review 3 sent (reviewed at 8e2530a) (c337595)
- RELEASE: midday release to main, PR #39 at develop 8e2530a (e4f9289)
- PURSE1-SPEC: encounter purse, authored fight gold (proposed) (8e2530a)
- PATCH-NOTES: ITEMPOW1-A, FC-TIMEOUT, TRIAL-SONNET (a8dcf23)
- TRIAL-SONNET: restart the Sonnet vs Opus trial on Assets & Audio (e1bedb0)
- FC-TIMEOUT: fc.sh suite timeout 900 s in the cloud, 290 s on desktop (182f9bb)
- PATCH-NOTES: INTENT1-U follow-up, POUCH-TIDY, prefs files, GATE-TIMEOUT (922c8d0)
- GATE-TIMEOUT: CI gate allows 900 s per suite; the arena suite now takes ~10 min (a8f2444)
- PREFS-ASSETS-SPLIT: split asset preferences into content and world files (d80d9ed)
- DESIGN-PREFS: four preferences from today's Design answers (3a317ef)
- PREFS-FILES: design, UI and asset preference files in docs/preferences (462d2f2)
- PATCH-NOTES: PATCH_NOTES.md, its generator and the coordinator and assets cutbacks (bd9bff1)
- RELEASE-LOGS: shared logs for the 2026-10-18 midday batch (1c617f5)
- USAGE: 2026-10-18 11:45 usage report, delta and efficiency (2f01ec6)
- UI-UX-HANDOVER: queue and notes for the next UI/UX session (2b2f70e)
- RUNNER-WINDOW: fullscreen toggle, minimize and exit in the Scene Runner (5ddcea3)
- PACE1: Q3 B, start_status schema only, no reward data yet (ac51dec)
- PACE1: no party buff at the phase change; Empowered only as a story reward (597eedc)
- PACE1: approved (pace all recs), decisions and task rows (051bee9)
- PACE1-SPEC: fight pacing targets and the two-act boss (proposed) (bd77de4)
- USAGE-TRACKING fix: .gdignore so Godot skips the usage CSVs (12921f8)
- MORALE1: spec final, flee per state, surrender on Broken, Fearless bosses/elites/reckless (7e78b41)
- GAMEPLAY-HANDOVER: rotation after EXPWORLD, UIR-1, HEAL1-L (539862a)
- USAGE-TRACKING: usage reports, deltas and efficiency script in docs/metrics/usage (96da6e0)
- MORALE1-APPROVE: spec approved (all recs), refinements R1-R3 folded in for confirmation (cce2a14)
- TOKENS-PER-AGENT: Timing lines log tokens per agent kind and a task type (7abbb56)
- DR2-ANSWERS: design review 2 approved (all recs), UI choices are preferences (0eeb52a)
- MORALE1-SPEC: enemy morale spec (proposed, awaiting approval) (df535ca)
- BALANCE-HANDOVER: handover for a fresh Balance session (0331486)
- ASSETS-HANDOVER: queue HEAL1-A (unblocked by HEAL1-L schema) (d8b2d0e)
- ASSETS-HANDOVER: Assets & Audio rotation after ART2-A, ITEMUSE1-A, ITEMUSE1-A2 (c4a6f08)
- DESIGN-HANDOVER: queue, routines and the option-build slot rule (002c2b5)
- USAGE-CAPS: building slots, split dispatch mode, background sims, rotation by context size (e3f215b)
- HEAL1 (spec approved): defeat cost per the user, decision row, build rows approved (bbb1e21)
- HEAL1 (spec): between-battle healing spec from BAL4-ATTR v4, TASKS rows, waiting on the user (9794c49)
- BAL5 (spec approved) + Orsa quest answer: decision rows, BAL5 rows approved (2aac644)
- BREPORT1 (spec approved): decision row, TASKS rows approved (cf765b8)
- GAMEPLAY-HANDOVER: rotation after OBJ1-L, ITEMUSE1-L, QUESTX (73e2e64)
- UI-UX-HANDOVER: queue and notes for the next UI/UX session (71172b6)
- GAMEPLAY-HANDOVER: queue empty after OBJ1-L and ITEMUSE1-L, open class-potion card (b5a3c04)
- ASSETS-HANDOVER: new UI/UX session id (71fbeef)
- PLATFORM-HANDOVER: Platform & Workflow hands over to a fresh session (fe9aee2)
- UI-UX-HANDOVER: queue and notes for the next UI/UX session (7cf9c91)
- ASSETS-HANDOVER: Assets & Audio session rotation after INTENT1-A, BOSS1-A, OBJ1-A (1c50088)
- Assets & Audio handover (session rotation) (b337d5d)
- GAMEPLAY-HANDOVER: queue OBJ1-L then ITEMUSE1-L, follow-ups and landing notes (805a504)
- BAL5 (spec): lever options for the BAL3-R balance gaps, TASKS rows, review candidates (4a2b759)
- BREPORT1 (spec): battle report spec and TASKS rows, waiting on the user (94c3dc6)
- UI-UX-HANDOVER: queue and notes for the next UI/UX session (95b7cfc)
- ASSETS-HANDOVER: MUSIC2 landed; DR1, BOSS1 and OBJ1 -A rows queued (6d9bf54)
- docs: approve BOSS1 and OBJ1 (user all recs, hold count changed) and their build rows (519eda1)
- docs(specs): BOSS1 and OBJ1 specs (awaiting approval) and build rows (0adcd6c)
- docs: approve RECRUIT1, ITEMUSE1, INTENT1, MAPREAD1 (user all recs) and their build rows (53c7433)
- docs(specs): RECRUIT1, ITEMUSE1, INTENT1, MAPREAD1 specs (awaiting approval) and 11 build rows (46e09f9)
- ASSETS-HANDOVER: MUSIC1 landed; MUSIC2 proposed (23411fa)
- docs: design review answers DR1, review spec and 19 task rows (71765c2)
- GAMEPLAY-HANDOVER: TRAV2-L, AUTO1-L logic and POI-FIX landed; queue empty (e758867)
- docs(handover): design review 2026-10-18 delivered, awaiting Tushar's answers (8f805eb)
- ASSETS-HANDOVER: CAMP1-A1/A2 and TRAV2-A landed; queue empty (184f09c)
- AUTO1 (spec): battle autoplay with a tactic dropdown per timeline row (e666c30)
- DESIGN-PAUSE: option builds paused, Design proposes ideas for approval (4ad84c9)
