# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to PR6-3, PR6-5 (excl. PR6-4; `FB-01`..`FB-281`, `UN-01`..`UN-12`) | released or done | `main` / `develop` (`f132eabc`) | Archived in `docs/archive/FEEDBACK_CLOSED.md` (moved 2026-11-17) |
| `PR6-4` (`PK3-RECRUITS-KEEPSAKES`) | done with parked clauses (`FB-282`, `FB-283`, `FB-284`) | `develop` (squash `PR6-4`) | Kit recruits/pool/crypts/fuse/keepsakes + `sunk_ward` seed `pr6_4.json`; temperaments, Martyr fuse data and the Retire/Last Stand prompt parked |
| `PR7-1` (`FIX-TEXT1`) | done | `develop` (squash `PR7-1`) | `sunk_ward` text fixes, seed `pr7_1.json`; new copy `[PROVISIONAL]` in `FB-286` |
| `WEBFIX1` (playtest, user 2026-11-17) | done | `develop` (squash `WEBFIX1`) | Web canvas fills page/fullscreen, web Fullscreen setting works; `FB-287` |
| `PR7-2` (`SPEC-FIX-N1N2`) | done | `develop` (squash `PR7-2`) | Sensory Rot on ranged hits; Cold Steel per-family rite + masterwork; `FB-288` |
| `PR7-9` (`LORE-DATA1`) | done | `develop` (squash `PR7-9`) | Seed `pr7_9.json`: Vanguard blurb and `notched_billhook` say one-handed polearm; `enc_evt_05` (the only `flag_skills` in `sunk_ward`) no longer grants `marrow_lash`; `dlg_int_dose` flags kept; test `sunk_ward_act1_marrow.rs`; PR5-44 Interlude grant test in `act1_story.rs` rewritten to assert no grant; 0 golden diffs |
| `PR7-11` (`MARTYR-FUSE1`) | done | `develop` (squash `PR7-11`) | Seed `pr7_11.json` (`martyr_last_stand`); `begin_fight` refuses on a spent fuse; Retire / Last Stand prompt `region::FusePrompt`; closes `FB-283`; `FB-295` |
| `PR7-12` (`BIGUNIT1`) | done | `develop` (squash `PR7-12`) | `footprint: 2` units fill a 2 x 2 square (BU1-BU15); seed `pr7_12.json` puts it on `calcified_amalgam`; `FB-296` |
| `PR6-6` (`PK4-CAMPAIGN-HOOKS1`) | done | `develop` (squash `PR6-6`) | Kit: data-configured doctrine, `recipe` trigger, hero strain/channel tallies, signed tracks, `keep_longer`, flag-armed checkpoints; `N4` mapping proposed; `FB-297` |
| `PR6-7` (`PK6-BATTLE-HOOKS1`) | done | `develop` (squash `PR6-7`) | Kit: region/encounter `start_meter`, hazard `when_meter` / `every_rounds`, ground meter-only fix, `$captive_unit` with `captive_fallback`, generic `requires_track`; `FB-298` |
| `PR6-8` (`PK5-HUB-FACILITIES1`) | done (UI feel to review) | `develop` (squash `PR6-8`) | Kit `facilities` kind + `facility` service + shop/recipe/rest hooks; `supply_price_mult_by_act` (seed `pr6_8.json`: `varka_trade` Act 3 `2.0x`); facility pane; placeholder icon; `FB-299` |
| `PR6-9` (`PK5-SIEGES1`) | done | `develop` (squash `PR6-9`) | Re-applied on develop after the user approved the re-record (decision card 2026-11-18): `checks_golden` case `ashlands encounters courtyard_skirmish` re-recorded (the objective enum message lists `retreat`); world kinds now 24 (sieges after endings, talks); `FB-300` |
| `PR6-10` (`PK6-STORY-PREP1`) | done | `develop` (squash `PR6-10`) | Kit `camp_prep` step, `retry_day_cost`, `echo_fallen`; `StoryCampPrep` screen; `FB-301` |
| `PR6-11` (`PK7-ENDINGS1`) | done | `develop` (squash `PR6-11`) | Kit `endings` kind, `end` kind `ending`, `campaign::ending` evaluator; GUI `EpilogueScreen`; `FB-302` |
| `PR6-14` (`CLASSKEY1`) | done | `develop` (squash `PR6-14`) | Class `keystone: {id, level}`; party units slot it at level 6 when their deck names none; seed `pr6_14.json`; `FB-303` |
| `PR7-13` (`BUNKS1`) | done (UI feel to review) | `develop` (squash `PR7-13`) | Hub bunks, bunk talks (`talks` kind, pair counter, templates), idle spots and passing the lead; seed `pr7_13.json`; six placeholders (AI-17); `FB-304` |
| `PR7-14` (`TRANSITIONS1`) | done (UI feel to review) | `develop` (squash `PR7-14`) | T1 battle beat, T2 victory beat, results over the field, hub arrival card; six AI-18 placeholders; scenario `story_battle_hub`; `FB-305` |
| `PR7-21` (`SETTINGS-MOCK1`) | done (UI to review) | `develop` (squash `PR7-21`) | Settings ported to the approved mock: tab rail, instant apply, staged Display Mode (Windowed, Borderless, Fullscreen) and Resolution with a 15 s keep-or-revert, close guard, decluttered in-game popup; review shots `/mnt/project-files/pr7-21/` beside the mock; `FB-308` |
| `PR7-22` (`AGENT-PLAYTEST1`) | done (tools lane) | `develop` (squash `PR7-22`) | `aegis playtest` start / do / run / stop / status (`tools/fc.sh playtest`), no-window by default; smoke script `games/emberwake/playtests/agent/smoke.play` plays a fresh save to the hub in the gate (~20 s); `FB-309` |
| `PR7-26` (`PLAYTEST-UI1`) | done (UI to review) | `develop` (squash `PR7-26`) | Story advance hint gone; battle Esc or Menu opens the in-game Settings popup; hub party list fits a full roster; portrait breath keeps the face aspect and the rim light eases; Mute in Background (on by default, user 2026-11-18); Battle tab Advanced heading; shots `/mnt/project-files/pr7-26/`; `FB-310` |
| `PR7-15` (`WEBFIX2`) | done (UI to review) | `develop` (squash `PR7-15`) | Battle, region and hub HUD panels draw a new `hud` frame role (F2 iron with oxblood top line; placeholder `ui/frames/hud.png`); Map Camera Zoom on Display too; web smoke plays the new opening at three sizes; web buffer capped at the GPU texture limit; `FB-312` |
| `PR7-24` (`HUB-POLISH1`) | done (UI to review) | `develop` (squash `PR7-24`) | Occupied bunks wear a name plate (class-initial sigil and name) on the 42 px bed; the bunk card is the dialogue box (portrait left, numbered choices, keys 1-9); shots `/mnt/project-files/pr7-24/`; `FB-314` |
| `PR7-23` (`HUB-LAYOUT1`) | done (UI to review) | `develop` (squash `PR7-23`) | Hub map rebuilt from the Art/UI eight-bunk layout (seed `pr7_23.json`, absorbs `PR7-19`): walls, 45 props, `bunk_1..8` (7 and 8 empty at cap 6), `idle_*` anchors; real prop art pushed from the PC; plates staggered on two heights so names read whole; `FB-316` |
| `PR7-25` (`STYLE-ENGINE1`) | done (UI to review) | `develop` (squash `PR7-25`) | S2 ramps: palettes may name `_lo` / `_hi` shades and the engine swaps them (variant bodies today; class bodies need a clips re-export); F2 portrait frame drawn in the one portrait widget on every screen; shots `/mnt/project-files/pr7-25/`; `FB-330` |
| `PR7-41` (`PLAYTEST-FIX3`) item 1 | done (blocker) | `develop` (squash `PR7-41-1`) | PT-15: a job's fight is offered only once a quest naming it is accepted; accepting after the fight was won marks it ready; stuck saves repaired on load with a notice; save `m3_stuck` fixture and scenario `job_fight_repair`; items 2-9 next; `FB-332` |
| `PR7-41` (`PLAYTEST-FIX3`) items 2 and 6 (part) | done; item 6 re-grant and retarget rules parked for Tushar (golden re-record) | `develop` (squash `PR7-41-2`) | PT-16: the auto-filled plan always confirms (a round a later draft broke is redrafted, at worst rests; hand-edited rounds never touched) and a refusal names hero, step and cause; PT-22 part: a friends-only cover guard needs a friend in range, Focus Weakest picks a foe it attacks, founders' third deck slot filled (seed `pr7_41_decks`); save `cages_start` and scenario `cages_auto_plan`; every golden unchanged; `FB-334` |
| `PR7-41` (`PLAYTEST-FIX3`) items 3-5 | done | `develop` (squash `PR7-41-2`) | PT-20: every forecast is per hit at the landing block with the target's guard then, "Targets you in Bn" names the landing block, unrevealed plans show only the told hit; PT-17: area marks are the tiles struck at impact, any revealed area marked; PT-21: fight 1 footmen `footman_curfew` (speed -3, seed `pr7_41_fight1`) so the Pommel Strike lands first; goldens unchanged; `FB-336` |
| `PR7-41` (`PLAYTEST-FIX3`) items 7-8 | done | `develop` (squash `PR7-41-3`) | PT-23: hero panel Ally button lists members the hero can use a skill or pouch item on and opens the skill menu there (as Ctrl+click); target refusals name the control; fight 1 hint names the click (seed `pr7_41_hints`); PT-28: under the deck rule members own their base deck passives from level 1 and the keystone from level 6 (kit `progression::deck_grants`), Skills tab says where they come from; goldens unchanged; `FB-338` |
| `PR7-41` (`PLAYTEST-FIX3`) item 9 | done (PT-35 partial, two points left as designed) | `develop` (squash `PR7-41-4`) | PT-25 captives saved on results; PT-26 capped heals "healed 7 of 52"; PT-27 AP debt shown as owed; PT-30 worn-copy refusal, shop worn/Scrap, key item uses, pouch legend; PT-32 Menu Retreat (fled-timeout fights), Restart/Step hidden when unavailable, Clear on auto stays cleared, strip off under intro talk; PT-34 text fixes (seed `pr7_41_text`); PT-35 Talia turn-in line, recipient prompt only when needed, quit confirm, slot labels; PT-36 bench/points notes and a real Alert option; shots `/mnt/project-files/pr7-41/item9/`; goldens unchanged; `FB-340` |
| `PR7-43` (`HUB-BUNKS2`) | done (UI to review; placeholder props) | `develop` (squash `PR7-43`) | Layout B (seed `pr7_43`): six niche bunks on the north wall 2.5 cells apart, three each side of the door, Timber Frames pair free-standing south; brass wall plates with class glyph and name (headboard for the south pair); class blankets, founder gear on pegs and keepsakes on beds; benched members sleep in their bunk (kit `bunks::sleepers`); provisional plank nooks with screens and a step; 33 placeholder PNGs; shots `/mnt/project-files/pr7-43/`; goldens unchanged; `FB-342` |
| `PR7-37` (`PLAYTEST-FIX2`) | done (item 4 partial; item 5 already done in PR7-41; item 9 checked, no fix needed) | `develop` (squash `PR7-37`) | New game skips the world picker outside dev builds; round-limit ends read "Out of time" with the jobs still open; hub points act (seed `pr7_37`, kit marker `use`); Varka's shop opens after Ivo is rescued; no empty `[]` tags; Character window aligned; world map "Next:" way hints, full job legend, "Opens after X", "New road open" notice; dev-only Settings rows hidden in player builds, Map Camera Zoom on Display only; web smoke and a release-profile `smoke_player.play` follow the player flow; shots `/mnt/project-files/pr7-37/`; goldens unchanged; `FB-344` |
| `PR7-41` (`PLAYTEST-FIX3`) item 6 switches | done (rules off by default; Tushar to rule, FB-334) | `develop` (squash `PR7-41-5`) | The parked re-grant and retarget rules land behind test-only switches `regrant` and `retarget` (`aegis_core::switches`, FB-343); off = today's behaviour, every golden unchanged; bisect results `/mnt/project-files/pr7-41/bisect/summary.md`; `FB-346` |
| `PR7-42` (`PACE1`) | done (sim bands unchanged; band misses predate it) | `develop` (squash `PR7-42`) | Act 1 patrols every 40-60 steps, at most 2 per location visit, a beaten patrol rests until the next day, repeat wins pay 75/50/25% XP then 0 (kit zone keys `visit_cap`, `rest_beaten`, `repeat_xp_pct`); quest `rec_level` on the board, The Cages (4) listed before The Sweep (5); the Candle-Maker sells Warded Tallow for 5 Scrap, Make camp needs 1 Tallow, the map shows the -2 AP penalty; fight 2 hint matches its objective and its unreachable exit tile is gone; seed `pr7_42`; goldens unchanged; `FB-348` |
| `PR7-36` (`STYLE-PASS1`) items 1-9 | done (items 4 and 9 partial; item 10 waits on PR #67) | `develop` (squash `PR7-36`) | Theme tokens: cyan, bright green and light blue retired for gold, amber, parchment, moss HP and cold-iron AP (`emberwake_gui::tokens`); popups on frame A (`panel.png`, placeholder), main panels ornate (`Ui::frame_in`); Alegreya SC gold titles; 32 px icon tiles on shop, stash, skills and gear rows; save slots over the dimmed title with portrait and day; one amber for chevron and chip; parchment move tiles and amber exit markers; world map label plates and gold party ring; engine shared blocks in `aegis_ui::panes` (shell, row, type steps, colour roles, tooltip, key hints); shots `/mnt/project-files/pr7-36/`; goldens unchanged; `FB-350` |
| `PR7-44` (`CONTROLS-UI1`) | done (default layout; Art/UI layout file optional) | `develop` (squash `PR7-44`) | Settings > Controls rebinds keys and pad buttons: click a row, press a key or button, Esc or right click cancels, a clash asks before it swaps, Restore defaults clears the settings key `controls`; engine `aegis_ui::controls` rebind model and `Screen::capture`; game view-model `settings_controls` drawn through the default layout; scenario `controls_rebind`; shots `/mnt/project-files/pr7-44/`; goldens unchanged; `FB-352` |
| `PR7-36` (`STYLE-PASS1`) item 10 | done except party token scale (ART-19, question) | `develop` (squash `PR7-36-2`) | Palette ids for Sabina, Varka and Berek (seed `pr7_36b`); region and hub points draw a glyph per kind, amber usable, iron used, oxblood locked exit; a 2 px intent line under battle skill icons; engine ground contact lines (`GroundLayer::contact`, off by default, on for the game's land layers); undercroft basin, hearth, hall, entry and hollows on `drain_brick` walls; shots `/mnt/project-files/pr7-36/item10/`; goldens unchanged; `FB-354` |
| `PR7-40` (`BATTLE-UI2`) part 1 | done (sections 1, 3-5, most of 2 and 6; the rest in parts 2-3) | `develop` (squash `PR7-40-1`) | Battle plates 176 px in 184 px panels with an 888 px board; foe panel with compact plates and groups, crown and boss gold; docked foe card from the live plan (per-hit poise pips, KO and break tags, after-KO, crit star); top bar round, goal and protect lines, AP pips and ruler; 4-slot status tabs; unit field `short_name` with 24 enemy short names (seed `pr7_40`, `[PROVISIONAL]`, for Tushar's review); battle view-model `battle_view`; shots `/mnt/project-files/pr7-40/`; goldens unchanged; `FB-356` |
| `PR7-40` (`BATTLE-UI2`) part 2 | done except section 7 (skill menu rows, question) | `develop` (squash `PR7-40-2`) | Gold rim on the latest pick, effect chips on strip tiles, tactic pennant buttons and the Auto button; foe groups open and close; the card stays on the planning hero's target; moss heal preview on plates (kit `intent::heal_on`); hazard icons with rounds left and dashed push and pull arrows; "Survive N rounds"; battle colours as theme tokens (Art/UI picks); shots `/mnt/project-files/pr7-40/part2/`; goldens unchanged; `FB-358` |
| `PR7-53` (`BUNKS-LAYOUT3`) with `PR7-50` (`BUNKS-UPGRADE-PROPS1`) | done (UI to review; placeholder art) | `develop` (squash `PR7-53`, one commit for both, Design 2026-11-19) | Take 8 bunk room: six niche beds 80 px apart on a dressed-stone wall with keystones, founder lunettes, one brass plate style, peg boards of gear, sleepers in the front idle pose under the blanket, stove between the tables, west barrels gone, the Act 2 pair sideways on the south wall; map prop key `until_bunk` holds a bunk's place until the bunk cap reaches it (timber, pallets, crates on the Act 2 cells); seeds `pr7_53`, `pr7_53_upgrade_props`; shots `/mnt/project-files/pr7-53/`; goldens unchanged; `FB-360` |
| `PR7-51` (`FOOTLOCKER1`) with `PR7-52` (`BUNKS-EMBLEM1`) | done (UI to review; placeholder footlocker art) | `develop` (squash `PR7-51`, one commit for both) | A footlocker at the foot of each active or benched member's bunk (none for a crypt member or an empty bunk); a click or E opens it and that member's loadout with the shared stash, and it closes with the screen; the hub stash chest is gone (seed `pr7_53z_footlocker`), the stash stays on I and the HUD; founder lunettes draw `ui/badges/founder_<id>.png`; the blanket folds over a sleeper's shoulders; every empty bunk wears a blank plate; shots `/mnt/project-files/pr7-51/`; goldens unchanged; `FB-362` |
| `PR7-40` (`BATTLE-UI2`) part 3 | done (row closed; two questions) | `develop` (squash `PR7-40-3`) | One row skill menu replaces the icon grid (contract 7): opens on an empty or picked strip block, a board target, or a ghost block by right click or Tab / pad north; title "Block N vs Target"; kit `pick_hits`, `next_pick_block`; proof-shot scenario `battle_board_marks` (heal, status, hazard, push); `game_colors` sorted; an open foe group listed once; section 8 and heal "+n" GUI tests; shots `/mnt/project-files/pr7-40/part3/`; goldens unchanged; `FB-364` |
| `PR7-58` (`FOOTLOCKER1B`) with `PR7-46` (`BUNKS-FIX1`) item 3 | done (PR7-46 closed: items 1-2 elsewhere) | `develop` (squash `PR7-58`, one commit for both) | E takes a footlocker ahead first, then what stands ahead, a footlocker underfoot last; the footlocker shows open for 0.25 s before its loadout and after it closes; Extra large plates drop the class glyph and cut a too-wide name with an ellipsis; one-click talk already worked, now proven by a test and scenario `one_click_talk` (state key `walk_to`); shots `/mnt/project-files/pr7-58/`; goldens unchanged; `FB-366` |
| `PR7-54` (`BOSS-UI1`) | done except BU8 (boss poise data waits for Tushar's golden approval) | `develop` (squash `PR7-54`) | Boss bar across the board top (contract 9, take 32): phase HP span and markers, per-phase names, one poise bar with ticks and "6/10", passive tags, icon-only tabs; planning-only intent rows (selected hero plus "Others", hover with every member, totals, "Party" line) on fixed cells; phase-break headers and greyed blocks; kit unit field `poise_pips` (no data); skill menus beside a tile keep clear of the card, foe panel and target; scenario `boss_bar`; shots `/mnt/project-files/pr7-54/`; goldens unchanged; `FB-368` |
| `PR7-57` (`STATUSVIS1-B`) | done | `develop` (squash `PR7-57`) | Statuses shown where they act (SV5-SV15): grey lock tiles with the status icon on the strip and in the full view (stun one tile, stagger single tiles; a locked block opens no menu); one planned-loss "-n" on party and foe plates with a hover of hits and round-end ticks; card numbers after all modifiers, tinted with an arrow, frameless breakdown hover; taunt badge over the sprite; riposte chip with no number; plate column priority; two passive sockets under the portrait; calm full timeline view with one target square per row; kit `status_view`; engine `aegis_ui` timeline slots; scenario `statusvis1b`; shots `/mnt/project-files/pr7-57/`; goldens unchanged; `FB-370` |
| `PR7-59` (`BOSS-UI1B`) + FB-370 riposte | done | `develop` (squash `PR7-59`) | Boss bar title is the unit's name and title (phase names only in the banner); optional kit phase key `form` (`<name>, <form>`, content check 1-40 characters, no data); the Stage board shifts down (at most 20 px) until the grown bar covers no unit or open tile; poise badges drawn from the kit files with the drawn fallback; riposte chip follows the riposte rule (`command::riposte_answers_at`); shots `/mnt/project-files/pr7-59/`; goldens unchanged; `FB-372` |
| `PR7-45` (`HITCALC1`) | done | `develop` (squash `PR7-45`) | One hit builder (`command::hit_terms`) shared by `Command::land` and `intent::hit_on`: cover by `cover_between`, both units' tiles, stances and guard at the landing block; impact failures forecast as such (dodged, out of reach, flank protected, taken by an ally); round-over hover reads "This round:"; seeds `act1story` and `cast1enc` archived (their stale `enc_pro_0a` copy undid PT-21); `refusal::Reason::NeedsTarget` replaces English matching; keystone header names its source; forecast-equals-result test (438 hits, 0 mismatches); goldens unchanged; `FB-374` |
| `PR7-47` (`POLISH-FIX4`) items 2-5 | done (item 1 moved to `PR7-60`) | `develop` (squash `PR7-47`) | The upkeep refusal names the world's item ("Not enough Warded Tallow to make camp."); the New road notice is kept in the save (`explore.roads_seen`) until the map shows it; the pawnbroker, lodging keeper and Skerrit (until vouched for) answer "What do you sell?" with a no-trade line (seed `pr7_47`); the battle Menu button fits at Extra large; goldens unchanged; `FB-376` |
| `PR7-48` (`STYLE-PASS1B`) | done | `develop` (squash `PR7-48`) | Amber ring on a click-to-walk's destination tile (the NPC's tile for a walk-and-talk) until arrival or a new order; battle heal float in the theme's moss (`tuning.json` `heal_color` retired); never stacked: `aegis_ui` `Screen::panel()` and `screen::drawn_screens`, a panel over a panel takes its place and Back returns to it; shots `/mnt/project-files/pr7-48/`; goldens unchanged; `FB-378` |
| `PR7-49` (`CONTROLS-UI1B`) | done | `develop` (squash `PR7-49`) | Every on-screen key hint reads the bound key (plus the pad button on hover titles) from `Controls`; a waiting rebind ends on the action's own input or after 5 s; `y`, `r` and the digits are fixed keys a rebind refuses, naming the screen; the controls view puts the key in `detail` and the pad button in `detail2`; question boxes answer `y` / `n` by the raw key; shots `/mnt/project-files/pr7-49/`; goldens unchanged; `FB-380` |
| `PR7-65` (`DIORAMA1B`) | done | `develop` (squash `PR7-65`) | Painted sheets: `clips.json` `foot` and `painted` (no palette swap); on the battle stage a body with a `foot` row stands 1.15 cells (32 px) tall per footprint cell, feet on the foot point (2 x 2 about 2.3 cells); seed `pr7_65` points the four founders and 15 Act 1 foes and bosses at their painted sheets; placeholder portrait copies keep every portrait unchanged; shots `/mnt/project-files/pr7-65/`; goldens unchanged; `FB-382` |
| `PR7-69` (`GRATE-GROUND1`) | done | `develop` (squash `PR7-69`) | Seed `pr7_69` retags 107 walkable mud cells of the Grate's north quay (`sunk_ward_runoff_grate`, rows 3-11) to flagstone; the silt bank, soot yard, catwalk and ring road unchanged; shots `/mnt/project-files/pr7-69/`; `FB-384` |
| `PR7-73` (`PAINTED-MAPS1`) | done | `develop` (squash `PR7-73`) | Hub and region maps scale a body with a `foot` row like the battle stage (1.15 of the map's tile width, feet on the tile's foot point); kit unit field `portrait` looked up before the body key; seed `pr7_73` keys the seven painted-body units to the files they already show (Art/UI #87 portraits kept); shots `/mnt/project-files/pr7-73/`; goldens unchanged; `FB-386` |
| `PR7-35` (`ASSETLIB1`) | done (tools lane) | `develop` (squash `PR7-35`) | Library folder `kits/rpg/library/` (README, empty) and `tools/assetlib/check_library.py` in the gate (`tools/fc.sh assetlib`); `FB-311` |
| `PR7-4` (`SCENARIO-GOLDENS1`) | part 1 done (tools lane); baselines await the user | `develop` (squash `PR7-4` part 1) | Scenario `shot` step `golden` (tolerance, pixels allowance, diff image), `record=1`, launch `save`; every scenario in the gate (`tools/fc.sh scenarios_all`); three screen scenarios (menus, hub, battle; 18 shots) run as review shots until the user approves the baselines (`/mnt/project-files/pr7-4/`); `FB-313` |
| `ENGINE-1` (`HELLO-GAME1`) | done (tools lane) | `develop` (squash `ENGINE-1`) | Sample game `games/hello/` on engine crates only: menu, walker sprite, one save slot, engine sounds, scenario `hello_walk` in the gate; programs `hello` and `hello_cli`; six RPG leaks listed; `FB-315` |
| `ENGINE-2` (`PERF-STEP1`) | done (tools lane) | `develop` (squash `ENGINE-2`) | Scenario `perf` step (no window: frame times, calibration, `budget` with a `no verdict` on a changing load); link request `perf`; `aegis_ui::perf` and `tools/fc.sh bench` (fill, panels, text); perf steps in both games' scenarios; `FB-317` |
| `PR7-38` (`UISPLIT1-E`) | done (tools lane) | `develop` (squash `PR7-38`) | `aegis_ui::screen_layout`: views and fixtures (`ui/views/<screen>.view.json`), layout files (`ui/layouts/<screen>.json`), 13 components, the default, the run-time loader with fallback and debug reload; `ui_layout check` in the gate (`tools/fc.sh layouts`) and `ui_layout shot`; UI Gallery entry Screen layouts; engine sample `layout_sample`; authoring page `engine/docs/authoring/screen_layouts.md`. FB-319 |
| `ENGINE-3` (`SHELL-SPLIT1`) | done (tools lane) | `develop` (squash `ENGINE-3`) | `aegis_ui/src/shell.rs` 4,423 to 2,904 lines (1,160 of code, the rest its tests): new modules `shell/input.rs`, `link.rs`, `settings.rs`, `window.rs`; code moved line for line, behaviour identical; native and wasm builds clean. FB-321 |
| `ENGINE-4` (`CI-HARDEN1`) | done in part (tools lane); web smoke as a required check waits for Design | `develop` (squash `ENGINE-4`) | Workspace deny set (`unsafe_code`, clippy `dbg_macro`, `todo`, `unimplemented`) in every crate; gate step `tools/fc.sh lints` (the deny set under clippy, `tools/deps/unused_deps.py` and its tests); advisory Windows job in `gate.yml` (build and test, MSVC). FB-323 |
| `ENGINE-6` (`NEW-GAME1`) | done (tools lane); which other authoring pages move to the kit waits for Design | `develop` (squash `ENGINE-6`) | `aegis new game <id> [title=] [root=]` (any game's CLI) writes `games/<id>/` on the engine alone from `engine/crates/aegis_cli/templates/new_game/`; gate step `tools/fc.sh newgame` makes one, builds, tests and plays its scenario, then removes it; `scenarios_all` runs every game with a `scenarios/` folder; API guide `engine/docs/API.md`; `combat_hooks.md` and `rule_library.md` moved to `kits/rpg/docs/authoring/` (pointer pages left). FB-325 |
| `ENGINE-7` (`INPUT-MAP1`) | done in the engine (tools lane); feel needs a user playtest with a pad; a rebinding screen is game UI | `develop` (squash `ENGINE-7`) | `aegis_ui::controls`: the control map (settings key `controls`, keys and pad buttons to `Key`s), applied to every key the shell hands a screen; Windows gamepad (`gilrs`) with d-pad and stick repeat and `Held::dir`; link requests `pad` and `controls`. FB-327 |
| `ENGINE-10` (`ROBUST1`) | done in the engine (tools lane); translating the game's own words is game work | `develop` (squash `ENGINE-10`) | `aegis_ui::strings` (string tables over the asset layers, `tr`, settings key `language`, a gate check in `fc.sh layouts`); `aegis_ui::crashlog` (`game.log` and crash reports beside the settings, windowed native runs); property tests for layout, pak and save migration. FB-329 |
| `ENGINE-5` (`ENGINE-PURITY-3`) | done (tools lane): parts 1 and 3 (part A), part 2 with `ENGINE-8` KU1 (part B, FB-339) | `develop` (squash `ENGINE-5` parts A and B) | Scenario `launch.screen` takes any screen id, the game's list `Game::launch_screens` (Emberwake `menu`, `arena`, `runner`) and its rules `Game::launch_problems` (ruleset only in the arena) checked by `scenario check`, `run`, `shots`; recordings pane words from `Recordings::labels` (engine default "SESSION"); timeline ruler label `Timeline::ruler`; trail plan edits read `subject` (alias `unit`); `bridge.rs` stays (names no RPG term). FB-331 |
| `ENGINE-11` (`GAME-START1`) | done (tools lane) | `develop` (squash `ENGINE-11`) | `aegis_ui::start::Start` (manifest, layers, saves, shots, settings, audio, options, the `Cx`) used by `emberwake_gui`, `hello` and the new-game template; `game.json` key `cli` (Emberwake `aegis`, else `<id>_cli`), read by `tools/fc.sh` for scenarios and the playtest smoke; the layers check exempts `hello`. FB-333 |
| `ENGINE-9` (`WORLD-PRIM1`) | done (tools lane) | `develop` (squash `ENGINE-9`) | New crate `aegis_grid` (cells, 4 and 8 neighbours, `search` uniform cost or A*, `reach`, `line`); `aegis_ui::world_camera` and `aegis_ui::tilemap`; the kit's battle routes (`rpgkit_battle::path` `find_path`, the 2 x 2 square path) run on `aegis_grid::search`, every golden unchanged; `games/hello` draws its field as a tile layer and walks to a clicked tile (link `walk_to`). FB-335 |
| `ENGINE-12` (`GATE-PARITY1`) | done in the cloud (tools lane); the PC run of `check_project.ps1` asked of the PC thread | `develop` (squash `ENGINE-12`) | `tools/gate_steps.txt` lists the gate's 13 steps; `tools/fc.sh gate` and `check_project.ps1` both run that list (the desktop gate gains repro, playtest smoke, scenarios, new game, layouts, lints, planner tests, asset library); `tools/fc.sh step <name>` runs one; the PowerShell copy of the layers check is gone. FB-337 |
| `ENGINE-8` (`KIT-UI1`) | KU1 done (tools lane, with `ENGINE-5` part 2); KU2-KU5 wait for screens split under `UISPLIT1` (the shop after `PR7-39`) | `develop` (squash `ENGINE-5` part B) | New kit crate `rpgkit_ui` (on `aegis_ui`, names no game): `portrait_fx`, `portrait_overlays`, `pips`, `vignette`, `portrait_page`. FB-339 |
| `ENGINE-13` (`SHADE-EXPORT1`) | done (tools lane) | `develop` (squash `ENGINE-13`) | `tools/sprites/export_units.py` lists a class body's `_lo` / `_hi` shades (the FB-330 patch); 20 class-body `clips.json` re-exported (shade slots only; sheets untouched); story vignette line-card portraits draw on the portrait well inside the F2 frame (`rpgkit_ui::portrait_fx::portrait_well`). FB-341 |
| `ENGINE-14` (`GOLDEN-BISECT1`) | done (tools lane) | `develop` (squash `ENGINE-14`) | `aegis_core::switches::on(name)` (test-only switches: env `AEGIS_SWITCHES`, debug builds only) and `tools/fc.sh golden-bisect <golden> <switch> [--keep DIR]` / `list`: records the golden with the switch off and on, lists the fights or runs that move, restores the fixtures. FB-343 |
| `ENGINE-15` (`TOOLS-HELP1`) | done (tools lane) | `develop` (squash `ENGINE-15`) | `tools/fc.sh golden-bisect repro <switch>` (the repro replays off and on), `tools/fc.sh schema-field` (a field into `kits/rpg/schema.json` byte for byte), `tools/fc.sh seed-from` (a seed from a stored entry with changes) and `aegis content get <kind> <id> [world=]`. FB-345 |
| `ENGINE-16` (`TOOLS-HELP2`) | done (tools lane) | `develop` (squash `ENGINE-16`) | `tools/fc.sh scenario shots-all out=<dir> [compare=<before>]` (every scenario's shots in one flat folder; changed, new and gone shots with pixel counts) and `tools/fc.sh rename <old> <new>` (an identifier across Rust and JSON; strings and comments left unless asked). FB-347 |
| `ENGINE-17` (`SHOT-CLOCK1`) | done (tools lane) | `develop` (squash `ENGINE-17`) | The shot clock: `aegis_core::clock::Instant` is stepped in every scenario run with no window (`clock=step`), time moving only by the frames the driver asks for; shot step `t`; new game seed from `clock::system_now`; gate step `shotsexact` (every scenario twice, 71 shots byte for byte). FB-349 |
| `ENGINE-18` (`LAYOUT-KIT2`) | done (tools lane) | `develop` (squash `ENGINE-18`) | `list_row` scrolls inside a short region (every row a widget, the focused row kept in view), no icon tiles without pictures, `detail2` second numbers column, `stat_block` `style: muted`, `heading` component with fixed `text`. FB-351 |
| `PR7-39` (`UISPLIT1-G`) | Stage A done (tools lane); Stage B waits for `ENGINE-19` | `develop` (squash `PR7-39` Stage A) | Shop view-model `shop_view.rs`, its actions through `ShopPane::act`, fixture `ui/views/shop.view.json`, tests; today's drawing kept. FB-353 |
| `ENGINE-19` (`LAYOUT-KIT3`) | done (tools lane) | `develop` (squash `ENGINE-19`) | `list_row` row buttons, selected rows, toned `detail`, a dimmed `then` section, view row ids (`row_ids`), and a `detail_line` component that follows the hot row. FB-355 |
| `ENGINE-8` (`KIT-UI1`) KU2 | shop done (tools lane); next screens wait for their UISPLIT1 split | `develop` (squash `ENGINE-8` KU2) | The shop and its view-model moved to `rpgkit_ui` (`shop`, `shop_view`, hosted by `ShopGame`); the game keeps its texts and tests. No behaviour change. FB-357 |
| `PR7-39` (`UISPLIT1-G`) Stage B | done (tools lane); the battle report half waits for `PR7-32` | `develop` (squash `PR7-39` Stage B) | The shop draws through the kit's layout `kits/rpg/assets/ui/layouts/shop.json`; new widget ids; no per-row hover cards. FB-359 |
| `PR7-39` (`UISPLIT1-G`) Stage B fix | done (tools lane) | `develop` (squash `PR7-39` Stage B fix) | The shop's layout matches the hand-drawn look: body-face numbers, thin column boxes, muted headings and title at the old inset, totals spaced as before (new layout styles). FB-361 |
| `PR7-34` (`AMBIENCE1`) | done (tools lane) | `develop` (squash `PR7-34`) | An ambience bus and loop channel in the engine beside the music; region, hub and road screens play `ambience/<key>.ogg` by the music's keys; a Ward Ambience slider on the Sound tab; placeholder loops in the kit. FB-363 |
| `ENGINE-12` PC gate fix | done (tools lane) | `develop` | The PC gate failed at step 4 (playtest): in the desktop's main checkout `tools/fc.sh` looked for binaries in `$PWD/target` while cargo built into `.cargo/config.toml`'s target-dir. `fc.sh` now exports that target-dir when `CARGO_TARGET_DIR` is unset (off Linux), as linked worktrees already did. The PC re-run is next. |
| `ENGINE-20` (`TOOLS-HELP3`) | done (tools lane) | `develop` (squash `ENGINE-20`) | Seed order check (gate step `seeds`), `seed-from --fields` with a `fields` upsert in the writer, `shots-all crop=` and before / after / diff sheets, toned text fields and the shop's total toned by sign again. FB-365 |
| `ENGINE-12` PC gate fix 2 | done (tools lane) | `develop` | The PC gate's step 10 (lints): `tools/deps/unused_deps.py` took a crate's name by splitting its path on `/`, but on Windows glob joins with a backslash, so the keep list never matched. It now uses `Path(crate).name`. In a sweep of the other tools, every other `split("/")` splits an asset key, not a path. `golden_bisect.py` now lists files as posix paths. |
| `ENGINE-12` (`GATE-PARITY1`) | closed: the PC gate passes | `develop` ec53813 | The PC thread ran `check_project.ps1` on develop ec53813: all 15 steps passed (3952 tests, exit 0). |
| `ENGINE-22` (`SEED-ARCHIVE1`) | done (tools lane) | `develop` (squash `ENGINE-22`) | 33 stale seeds moved to `data/seeds/archive/`, with a README row each. The 2 known overlaps are settled. `seed_order_known.txt` is empty. FB-367 |
| `ENGINE-22` fix | done (tools lane) | `develop` | Correction to FB-367: my edit to `cast1a3.json` (narrowing its `bile_coat` step) moved it last in the commit order, so after main Build's `pr7_40` landing the seeds step failed: 34 findings, all `cast1a3.json` against `pr7_41_text.json`. `cast1a3.json` is now restored as it was and archived, making 34 archived seeds. `tools/fc.sh seeds` reports 67 seeds and 0 findings. `seed_order.py`'s doc and the seeds README say an old seed is archived, never edited in place. develop was red on the seeds step between 33a6b49 and this fix. |
| `ENGINE-21` (`UISPLIT1-H`) board and save/load | Stage A done (tools lane); the character sheet waits for main Build's `PR7-45` | `develop` (squash `ENGINE-21` A1) | View-models, actions, fixtures and tests for the notice board and the save/load screens; today's drawing kept. FB-369 |
| `ENGINE-23` (`UISPLIT1-I`) recipe and facility | Stage A done (tools lane); the party loadout waits for main Build's `PR7-51`, the world map for `PR7-47` | `develop` (squash `ENGINE-23` A1) | View-models, actions, fixtures and tests for the recipe pane and the facility screen; today's drawing kept. FB-371 |
| `ENGINE-20` fix (CI seeds) | done (tools lane) | `develop` | The seeds step failed on CI since 18:13 UTC: the gate job's depth-1 checkout gave every seed the same commit time. The checks job now checks out with `fetch-depth: 0`, and `seed_order.py` fails with its reason when a seed's last commit is a shallow clone's boundary. FB-375 |
| `ENGINE-24` (`PASSIVE-ICON1`) | done (tools lane) | `develop` (squash `ENGINE-24`) | Kit field `icon` on passives with its derivation and content check A21, 16 placeholder icons, a seed giving the 97 passives their archetype, and two loadout passive icons under the portrait on party plates. FB-373 |
| `PR7-55` (`ENRAGE2`) + `PR7-56` (`PHASEBREAK1`) | done (tools lane); the battle screen parts are main Build's (`PR7-40` part 3, `PR7-54`) | `develop` (squash `PR7-55` + `PR7-56`) | Phase break (a hit stops at the phase threshold and ends the round; phase `vignette` cue); baseline enrage by tier from round 6 (ruleset fields, seed `enrage2.json`); `timeout_outcome: escaped` for random fights (no rewards, retreat allowed); Sunk Ward caps by tier; sim `pace=on`; one boss balance run; sim, PHASEFOLD and rules contract goldens re-recorded. FB-377 |
| `ENGINE-23` (`UISPLIT1-I`) party loadout | Stage A done (tools lane); the world map still waits for `PR7-47` | `develop` (squash `ENGINE-23` A2) | View-model, actions, fixture and tests for the party loadout (the Character window's Gear tab the footlockers open); today's drawing kept. FB-379 |
| `ENGINE-26` (`BALANCE-WHATIF1`) | done (tools lane); the data change waits for the user | `develop` (squash `ENGINE-26`) | What-if sim keys `foe_hp=`, `foes=`, `foe_level=` (feel mode, report only); random, elite and boss what-if runs in `BALANCE_REPORT.md`. FB-381 |
| `ENGINE-25` (`UISPLIT1-J`) forge, siege, epilogue | Stage A done for three screens (tools lane); the dojo next; the hub talk held for main Build's `PR7-46` | `develop` (squash `ENGINE-25` A1) | View-models, actions, fixtures and tests for the Content and World Forge viewers, the siege screen and the epilogue; today's drawing kept. FB-383 |
| `ENGINE-25` (`UISPLIT1-J`) dojo | Stage A done for the dojo (tools lane); four of five screens done, the hub talk held for main Build's `PR7-46` | `develop` (squash `ENGINE-25` A2) | View-model, actions, fixture and tests for the dojo; today's drawing kept. FB-385 |
| `ENGINE-25` (`UISPLIT1-J`) hub talk | Stage A done for all five screens (tools lane) | `develop` (squash `ENGINE-25` A3) | View-model, actions, fixture and tests for the talk box; today's drawing kept. FB-387 |
| `ENGINE-27` (`CRUCIBLE-CHECK1`) | Done, report only (tools lane) | `develop` | No rule bug: the escape needs all three non-holders out; at the authored level 1 a founder falls in round 1, and at level 4 the round-3 wave still blocks the exit lane (Overlook 75 %, base 0 %). FB-389 |
| `PR7-29` (`SLOTCARD1`) | Done (tools lane) | `develop` | The save and load screens draw six `slot_card`s in two columns (location picture, place, day, party portraits, time, lock and delete); one click acts as before; "The Ashlands · time" with no location. FB-391 |
| `PR7-28` (`TITLEFX1`) | Done (tools lane) | `develop` | Lantern glows and rising embers over the title backdrop at the spots of `backgrounds/menu/title.fx.json`; off under Reduce motion; scenarios `title_fx` and `title_fx_still`. FB-393 |
| `ENGINE-28` (`CAMPAIGN-SIM1`) | Done, report only (tools lane) | `develop` | `tools/fc.sh sim campaign world=<world>`: story fights in story order with the party each fields, XP and levels carried by the kit's rules, optional side fights; Sunk Ward founders reach M5 at level 7, where the Crucible is still 3 % (Overlook 9 %). FB-395 |
| `PR7-30` (`MENURAIL1`) | Done (tools lane) | `develop` | Game menu as a left rail plus party box; story title cards and the run-over card in the ornate frame with the title on a banner across the top edge (kit `rpgkit_ui::banner`); the Recipe pane ornate. FB-397 |
| `ENGINE-29` (`ESCAPE-PLAY1`) | Done, report only (tools lane) | `develop` | No escape bug in the driver; the level effect was the fixed start (jittered: Crucible 0-6 % at every level 3-8); two trial fixes were worse and reverted; options for the Crucible call. FB-399 |
| `ENGINE-30` (`AGENT-CRUCIBLE1`), `ENGINE-31` (`SIM-JITTER1`) | Done, report only (tools lane) | `develop` | The Crucible played by hand at level 7: base won in round 6 and Overlook in round 5 with no downs, every other try lost a founder; `objective_count` 2 would turn three losses into wins; sim start jitter is already the default (fixed starts move 7-16 of 24 fights). FB-401 |
| `PR7-60` (`ESCAPE-PLAN1`) | Built (tools lane); goldens parked | `develop` | Hand moves onto exits; a pick never drops another's (refused with the reason), hand moves pass through friends; the hold pick suggests the nearest hero and can change until round 1; a hero's Withdraw spends no AP it lacks; the foe card keeps off the winch and exits. `sim_golden`, `phasefold` and `timeline_golden` run the old moves behind a test-only switch until the user approves the re-record. FB-403 |
| `PR7-61` (`PLAN-RECURSE1`) | Built (tools lane) | `develop` | An Engage with an area skill that `affects` `all` as a unit's first step no longer recurses without end with friends active: while a first move is compiled, the area counts friends where they stand. No golden moved. FB-407 |
| `PR7-31` (`OBJUI1`) | Built (tools lane) | `develop` | Board chips for morale (state bars, the group letter with several groups), the leader's pennant, the slay crown and the guest shield (gold rim when it must survive); the words on the hover card and the foe card's portrait; the goal hovers to the objective and morale lines; the results read the objective line and a break's lines. No Enraged chip (Design's amendment). FB-409 |
| `PR7-33` (`FLOATPANELS1`) | Built (tools lane) | `develop` | The region's "Here (N)" legend, its event ticker and the battle log float (unlock, drag, minimise in place to a named button, place and lock kept in the settings) on the floating panel, now an engine module `aegis_ui::float_panel` with shared chrome. FB-411 |
| `PR7-62` (`INTERCEPT-LIMIT1`) | Built (tools lane) | `develop` | `trigger::intercept_of` walks the same `ally_hit` rows as the hit, in its order, with their use limits (counts now on `World::trigger_uses`): a spent intercept is no longer forecast. Kit tests for a spent limit; goldens unchanged. FB-413 |
| `ENGINE-21` (`UISPLIT1-H`) character sheet | Stage A built (tools lane) | `develop` | View-models `character` (the window), `stats`, `level_up`, `skills` and `deck`, with actions, fixtures and tests; the Gear tab stays the `loadout` view; today's drawing kept. FB-415 |
| `PR7-27` (`VNSTAGE1`) | Built (tools lane) | `develop` | The story stage: cards carry actors (full-body pictures, a code-drawn figure where none exists) who enter left and exit right, met ones pinned right; cameras Static, Scroll and Pan; dialogue and narration keep the scene's backdrop dimmed; the cutscene bar gains Auto and Log; born split (view `vignette` and its fixture). Seed `vnstage1` stages the meat market and the mortuary cart. FB-417 |
| `PR7-64` (`DIORAMA1`) | Built (tools lane) | `develop` | Sunk Ward names stage set `sunk_ward_stage` (seed `diorama1`); ground hazards draw their painted decal `props/hazards/<status>/v*` under the icon and rounds left (the tint without a file); row 0's ground cells draw the set's `wall_foot` strip. FB-419 |
| `PR7-66` (`VNSTAGE1B`) | Built (tools lane) | `develop` | The rest of Act 1's speaking scenes staged (seed `vnstage1b`): the three hub debates and the awakening fissure; data only, no new code. FB-421 |
| `PR7-67` (`DIORAMA2`) | Built (tools lane) | `develop` | The wall lip in the stage set's tone; the front face from `tilesets/<set>/front_face/<terrain>/`; the stage fills the 888 px panel with faded wings, inside the panel's rim; edge decals unclipped. FB-425 |
| `PR7-68` (`DIORAMA3`) | Built (tools lane) | `develop` | Amber light pools under the stage set's lantern spots and a soft vignette; a block inside the arena stands as the set's cover prop; the grid faint at rest, stronger while planning, with a Board Grid setting. FB-427 |
| `ENGINE-23` (`UISPLIT1-I`) world map | Stage A done (tools lane); ENGINE-23 complete | `develop` | View-model, actions, fixture and tests for the world map; today's drawing kept. FB-429 |
| `PR7-63` (`XL-BATTLE1`) | Built (tools lane) | `develop` | The battle screen at Extra large: the goal under the title, "Boss above" on its own row, cut texts hover to their words, tags and tabs centred; the boss bar's next blow back on screen. FB-431 |
| `PR7-70` (`PANELS2`) | Built (tools lane) | `develop` | Save/Load slots, forge, dojo and the services screen are panels; they already filled the screen, so nothing changes on screen. FB-433 |
| `PR7-32` (`BREPORTUI1`) | Built (tools lane) | `develop` | The results popup shows the battle report table and best moment; F2 panel sized to content, portraits, drop tiles, XP bars; PT-11 and PT-13 fixed. FB-435 |
| `PR7-71` (`CONTROLS-MIGRATE1`) | Built (tools lane) | `develop` | On load, an action on a fixed key goes back to its own key (swap undone), the settings are saved and a notice names it. FB-437 |
| `PR7-72` (`SHOT-DIFF1`) | Built (tools lane) | `develop` | `tools/fc.sh shots diff`: the scenarios a change can show, base shots cached per commit, `index.md` and a contact sheet. FB-439 |
| `ENGINE-32` (`ACT1-AGENT-RUN1`) | Report (tools lane) | `develop` | Agent run of Act 1: 9 quests turned in, then blocked at the Bone-Mill boss for the auto plan; PT-40..PT-65. FB-441 |
| `PR7-74` (`DIORAMA4`) | Built (tools lane) | `develop` | Zone stage sets, tint and terrain overrides (`stage.fx.json` keys for Art/UI), the backdrop band under the boss bar, see-through units in front of the focus, the boss-fight foe card off the party; S5 is an art crop. FB-443 |
| `PR7-75` (`ACT1-FIX1`) | Built (tools lane) | `develop` | The ending's reason in the results and the log, the boss HP hover and the Hostiles count, a fresh seed on a retry, the talk-first message, no sentinel durations, the guest portrait, the portrait key first. FB-445 |

## 2. Feedback Items (Append-Only)

### FB-445: PR7-75 ACT1-FIX1 built; ending reasons, boss HP, retry seed, talk message, durations, portraits

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `00g` `PR7-75` (`ACT1-FIX1`), from `FB-441`.
- **What landed**:
  - (PT-40, why it ended) the kit's `Session::end_reason()` (`rpgkit_battle`): the party's retreat ("The party
    retreated from the fight."), the round limit ("The round limit ran out (N rounds)."), the objective's own note,
    the last standing heroes leaving the field ("No hero was left fighting: Ash left the field."), every hero down;
    empty while the fight goes on. The results' objective lines and the round-over message ("Battle over: Fled. ...")
    carry it, and the log gets one "[End] ..." line. The run's Fled came from the last two standing heroes withdrawing
    through the exit, which the kit verdict reads as fled; the line now says so. The retreat log line is the kit's
    `RETREAT_LINE`.
  - (PT-40, boss HP) neither max is wrong: the bar shows the current phase's span (contract 9.1, 142/304) and the unit's
    own card showed the whole (142/608); the boss card already folds into the bar. The bar's HP read-out now has a hover
    with both ("This phase: 142/304", "In all: 142/608"; one line when they match) (`BossBar.whole`, `hp_tip`).
  - (PT-40, counter) "Hostiles N" counts the boss with the panel's foes.
  - (PT-41) a lost (or fled) world fight counts a try in the save (`rolls["retry:<encounter>"]`), and the next start
    mixes the count into the battle seed (`rpgkit_world` `retry_seed`; the base seed when there are no tries). Sims,
    replays and the journey keep their seeds: goldens unchanged.
  - (PT-47) when a talk is due at a round's start, the bar says "Round N: a talk first; the plan fills when it closes."
    and switches to the plan message once the plan fills after it.
  - (PT-48) a battle-long duration (the kit's `trigger::lasts_battle`, rounds above `BATTLE_ROUNDS - 100`) shows no
    number on the status tabs, the pending tabs and the taunt tip.
  - (PT-59) a guest's results row draws its catalog unit's portrait (`Report` rows keep the unit id, not written to the
    JSON; `Who::of_catalog`), the initial only without art.
  - (`FB-386` question 3) `portrait_chain_keyed` and `face_chain_keyed` put a unit's explicit `portrait` key first,
    above the unique tier's own files (face chain deduplicated).
- **Gate fix** (develop was red after PRs #90 and #91): the asset tree test (`region.rs` `layout_ok`) now allows a
  terrain override folder's own front face (`tilesets/<set>/<terrain>/front_face/v*.png`, the PR7-74 key the foundry
  and tallow sets now use) and, `[PROVISIONAL]`, a stage set's parked placeholder backdrop
  (`tilesets/<set>/backdrop_<name>.png`, High Bastion's ramparts) until its zone exists. For Design: add both to the
  ASSETORG2 tree (section 3, the stage set line), or ask Art/UI to park the backdrop under `backgrounds/battle/`.
- **Goldens**: none re-recorded; sim, rules, checks, journey and the other goldens pass unchanged.
- **Changed shots** (`fc.sh shots diff`, base `4974bcdf`): one changed, `story_battle_hub` results over the field, but
  the change is the results report that landed on `develop` after that base; a won rout has no reason line, so this
  row changes nothing visible there. Sheet: `/mnt/project-files/build-tools/pr7-75/shots_diff_sheet.png`.
- **Guesses** (`[PROVISIONAL]`): (1) the boss bar keeps the phase span and gains the hover rather than showing the whole
  (contract 9.1 stands); (2) any outcome but won counts as a try, fled included; (3) a sentinel shows no number rather
  than "until removed" (the tab's hover already names the status); (4) the reason for a fled that is neither a retreat
  nor heroes leaving is empty.
- **Tests**: `rpgkit_battle/tests/end_reason.rs` (6), `rpgkit_world/tests/world_fight.rs`
  (`a_retry_after_a_lost_try_draws_a_fresh_seed`), `rpgkit_content/tests/portrait_key.rs`, `boss_ui1_tests`
  (`the_counter_counts_the_boss_and_the_hp_hover_gives_both_numbers`), `statusvis1b_tests`
  (`a_battle_long_status_shows_no_sentinel_number`), `talk_screen_tests` (both messages), `hub`
  (`a_guest_row_shows_its_portrait`).
- **Docs delta**: `engine/README.md` none (no engine change). Kit README lines for Design: `rpgkit_battle` gains
  `Session::end_reason`, `RETREAT_LINE` and `trigger::lasts_battle`; `rpgkit_world` gains the retry count in the seed
  (`retry_seed`, save key `retry:<encounter>`). `HISTORY` none.

### FB-443: PR7-74 DIORAMA4 built; zone looks, backdrop band, see-through units, boss card docking

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `00f` `PR7-74` (`DIORAMA4`), with the Design relay of 2026-11-22 on Art/UI's zone sets
  (PR #88).
- **Key layout for Art/UI** (in `tilesets/<world set>/stage.fx.json`, which Art/UI owns; nothing of it is shipped by this
  row, so the game looks as before until the entries land):

  ```json
  "zones": {
   "tallow_vats": {
    "set": "sunk_ward_tallow_vats_stage",
    "tint": "#rrggbb",
    "terrain": {"lava": "sunk_ward_tallow_vats_stage/lava"},
    "spots": []
   }
  }
  ```

  - `set`: a stage set id. Its files are read before the world set's for the floor and every terrain tile, the wall
    face, the wall foot, cover props, the front face (`front_face/<terrain>`) and the transition pieces (the zone's
    pieces when it has any, else the world's). A set holding only `floor` and `stone` keeps the world set's walls and
    front face (then `common_stage`). Its files count at the stage art resolution like the world set's.
  - `tint`: "#rrggbb", multiplied over every tile, wall, cover prop, wall foot, piece and front face of the zone except
    terrains with an override.
  - `terrain.<terrain>`: a folder `<set>/<name>` under `tilesets/` (a bare `<name>` means the zone's set, else the
    world's), or `{"folder": ..., "tint": "#rrggbb"}`. Its `v*` replace that terrain's tiles in the zone; its
    `front_face/v*`, when present, the front face under it. The override's tint (none: untinted) replaces the zone's on
    that terrain. A path with `..` or an empty part is ignored.
  - Watchpost and the closed alleys: `tint` only. The keys are read once per world set and zone.
- **What landed**:
  - (S1) the keys above (`stage.rs` `ZoneLook`, `zone_look`, `zone_tile_art`); the engine gains
    `Painter::image_tinted` (an image multiplied by a colour). A row-0 wall's cap and cover are clipped at the board's
    top edge, so they no longer cover the backdrop's foot. While the boss bar shows, the backdrop band is drawn again
    from the bar's bottom down to the first row holding a walkable cell or a unit (at least 8 px), bottom-aligned, and
    the walls wholly inside that band are left out (`boss_band`, `draw_band`).
  - (S2) the terrain override, as above. With a demo entry (`set` and `lava` from PR #88, not committed) Orlov's
    floor shows the tallow and the zone floor with the world's walls.
  - (S3) a unit not in focus whose body overlaps a focus unit's and whose feet stand nearer draws at 115/255 (about 45
    percent): focus is the selected unit, the hovered one, the inspected one and the selected unit's round target
    (`stage::occluders`, `OCCLUDER_ALPHA`). 1.15 cells kept; the outline-through pass stays; static, so Reduce motion
    changes nothing.
  - (S4) in a boss fight the docked foe card counts the party's cells first: the side away from the party (the party's
    mean column) is tried first, then the other side; the compact card (header and totals, no rows) only when both full
    sides cover a party cell; ties go by kept-clear cells, then objective tiles (`card::dock`). Outside a boss fight the
    BU12 rule stands. (The gate's `statusvis1b` caught a first cut that went compact to spare the target cell.)
  - (S5) no clip in the code: a column-0 decal draws whole inside the stage clip, at rest and shaken (test). The cut
    in the Fable shot is in the art: `props/hazards/slow/v0.png` and `v2.png` and `poison/v1.png` carry a sliver of the
    next sprite at their left edge (columns 4 to 7), and `poison/v0.png`, `slow/v1.png` and `slow/v3.png` one at their
    right edge (columns 52 to 55). For Art/UI: re-crop those six.
- **Golden**: the `stage.rs` fixture frame hash `TODAYS_FRAME` is re-recorded (`0xe6c3440ef5f1669f` to
  `0x2a5309c84d603849`) for the one intended change, the row-0 wall clip (checked: with the clip off the old hash
  holds).
- **Changed shots** (`fc.sh shots diff scenarios=diorama_zones,diorama_stage,boss_bar`, base `8bc31c0b`): 6 changed, 1
  new; the boss shots gain the backdrop band under the bar and the units in front of Kaelen fade; the Sunk Ward shots
  change only along row 0's walls and in front of the selected unit. Sheet and shots:
  `/mnt/project-files/build-tools/pr7-74/` (`shots_diff_sheet.png`, `vats_boss_no_zone_entry.png`,
  `vats_boss_with_zone_set_demo.png`). New scenario `diorama_zones` (Orlov's fight in the tallow vats).
- **Guesses** (`[PROVISIONAL]`): (1) the band shows the backdrop's lower part where it fits rather than moving the board
  (the board has about 4 px to spare); (2) the tint multiplies, so it can darken and shift a colour but not lighten it;
  (3) the party side is the mean column of the party; (4) the compact card leaves out the rows that do not fit, with
  no "more" mark; (5) the demo shows the tallow override's front face falling back to the lava face colour, so a
  `front_face` folder in the tallow override would finish it (Art/UI).
- **Tests**: `stage.rs` (the key layout parsed, bad ids and paths left out, the file read; the zone's set before the
  world's, the tint, the override with its own tint, the wall face from the world's set; a row-0 wall changes nothing
  above the board; the band's extent and its walls; the occluders), `battle/card.rs` (the dock in and out of a boss
  fight), `battle/marks.rs` (a column-0 decal whole), `aegis_ui` `fx_loops.rs` (`image_tinted`).
- **Docs delta**: `HISTORY`: "2026-11-22 `PR7-74`: the `stage.rs` fixture frame hash re-recorded for the row-0 wall
  clip (DIORAMA4 S1)." `engine/README.md` (aegis_ui painter): "`Painter::image_tinted(img, r, tint, a)`: an image
  multiplied by a colour." `STRUCTURE`: none.

### FB-441: ENGINE-32 ACT1-AGENT-RUN1 report; Act 1 blocked at the Bone-Mill boss for the auto plan

- **Status**: report (tools lane), no code changes.
- **Spec**: `docs/READY.md` row `ENGINE-32` (`ACT1-AGENT-RUN1`).
- **Run**: `aegis playtest` (dev build, no window), a new Sunk Ward game, played through clicks and keys. I read
  every vignette card, confirmed the auto-filled plan each round, and clicked the board, the turn-ins, level-up, the
  gear, the shop and the travel; I made no debug jumps. Run log: `/mnt/project-files/build-tools/act1_agent_run/run_log.md` (4,870 commands), with the
  shots this table cites in the same folder. Saves are kept at the start of the hub, before the Cages, the Sweep, the
  Kiln, the Vats, and at the end.
- **How far**: the prologue (opening vignette and four fights), then the Sluice Lock-Ciphers, the Crossroads Cages,
  the Night of the Red Pen, the Sweep, the Seventh Ward Bellows, Liturgical Cipher Rubbings, the Tallow Vats (Elyse
  joins) and Corvo's Gate (toll), all turned in. The Bone-Mill Raid was lost three times (PT-40), so the Crucible
  (Locked) and the act's end were not reached. A player who plans the rounds by hand may get past it; this run used
  only the auto plan and AUTO.
- **Time per region** (wall clock, UTC 04:04 to 04:48): prologue to the hub 7 min; hub, board and level-up 3 min;
  Drowned Runoff Grate 2 min; Ash-Crawl Crossroads (Cages, Red Pen, a patrol) 6 min; Alleyway Junction 7 (the Sweep, a
  patrol) 4 min; the turn-ins at the hub 4 min; Alleyway again (shop, the Bellows, a patrol) 4 min; the Fourth Kiln
  (a patrol and the boss twice, then once more) 9 min; the Tallow Vats 3 min; Corvo's Gate 1 min. In game: Tithing 42
  to 17 days.
- **Slow frames**: none seen in this headless run; frame times need a windowed run.

| PT | Sev | Area | Finding | Shots (`NNN_*.png`) |
| --- | --- | --- | --- | --- |
| PT-40 | blocker | battle | The Bone-Mill boss (Overseer Grendel, phase 2) beats the party three times at Lv 7-8: twice on the auto-filled plan, once on AUTO. Each ends Fled at round 6-7 with two heroes standing and no line on why they fled. The boss card reads 142/608 while the boss bar reads 142/304, and the counter reads Hostiles 0 with the boss up. | 3819, 4191, 4865 |
| PT-41 | major | battle | A retry replays the same fight: new gear (Slag Plating, Gilded Pavise) and the same plans give the same damage per hero to the point (170/222/104/411/58) and the same Fled at round 6. A player who retries without replanning cannot get a different result. | 3831, 4191 |
| PT-42 | major | time | Tithing fell from 42 to 17 days by the Vats and Corvo. Each road and each rest costs a day, and a lost boss fight costs about 4 days for the trip home to rest. Nothing says what happens at 0 or how many days the rest of the act needs. | HUD in 4710 |
| PT-43 | major | region | Quest points (the lock tablet, the toll pen, the rubbings, the bellows cache) are not in the Here list, though NPCs and exits are. The bellows cache is about 25 tiles off screen, with no arrow pointing to it, so finding it means panning the whole map. I walked there by keys. | 2349, 2675, 3416 |
| PT-44 | major | region | After using a point (the rubbings, the toll pen), the log adds "You can't get there.". That line replaces the quest line in the one-line log bar, so the player sees an error where the quest moved. | 2675 |
| PT-45 | major | world map | No roads are drawn until a place is picked (roads [] at the start). A new player sees locked places with quest marks and no way to them. The Next hint shows only after clicking a locked place. | 2329, 2331 |
| PT-46 | minor | story | Each vignette card takes 2-4 Space presses: the first finishes the typing, and presses during the fade seem to be eaten. | 005..066 |
| PT-47 | minor | battle | A mid-fight talk opens with the round's plan empty (1 Idle .. 6 Idle) and Confirm disabled, while the message bar still says "Round 2: plan your party.". | 1839 |
| PT-48 | minor | battle | A downed foe lists "Hasted: 998 rounds" (a sentinel duration shown as a number). | 1839 |
| PT-49 | minor | board | Jobs show no Where line and no level, while the main quests show both. The Crucible reads "Locked: Locked" with no "Opens after" line, and so do the Overlook, Wedges and Counter-Still. | 2252, 3268 |
| PT-50 | minor | world map | A quest that is ready to turn in (Lock-Ciphers) drops off the Active quests legend. | 2359 |
| PT-51 | minor | region | Using the lock tablet logs its lore line twice. | 2359 |
| PT-52 | minor | story | The Night of the Red Pen resolves in one log line on a click (no scene, no fight), though the board sells it as a rescue. | 2675 |
| PT-53 | minor | world map | Make camp always shows disabled away from a safe place, and the reason ("Rest only at a safe place.") is only in its hover. | 3906 |
| PT-54 | minor | region | Exits named for a direction ("World map: the alley north", "the road north") all open the same world map at the current place. | 2736 |
| PT-55 | minor | region | After a patrol fight interrupts a walk to an exit, the walk does not resume (it takes two more clicks). | log |
| PT-56 | minor | turn-in | Each "We finished" opens its own confirm popup, even with nothing to choose (four popups for four quests). The "Quest complete: 60 XP, 20 Scrap." lines do not name the quest. | 3216 |
| PT-57 | minor | economy | Tallow fell from 10 to 1 in the first week (1 per road), so I had to make a shopping trip before the Bellows. The shop's "20 of 98" reads oddly. | 3355, 3358 |
| PT-58 | minor | region | Leaving the Fourth Kiln after the lost boss fight, a patrol jumps a party at 59/1/1/1 HP. There is no rest point there, so the walk out is another fight (Fled again). | log |
| PT-59 | minor | results | A guest row shows only its initial, though the guest has a portrait in the party list (Dunstan). This is my `PR7-32` drawing guests without art. | 3831 |
| PT-60 | minor | gear | Equipping an item the hero cannot use opens a notice with a raw id: "Vesper can't wear Marrow Scalpel: untrained in bone_saw (proficiency 0).". The stash list offers the item with no mark. | 3972 |
| PT-61 | minor | quest | Paying Corvo's toll completes Corvo's Gate with no log line (no "Quest updated", no Scrap line). Only the Journal shows it ready. | 4710 |
| PT-62 | minor | journal | The journal panel lets the region's Here list and log show through faintly behind the quest text. | 4710 |
| PT-63 | minor | turn-in | Low confidence: the Tallow Vats pick (`loot_orlovs_retribution`) is missing from its "Quest complete" line, and the stash then holds "Flensing Talon". The Sweep's pick (Gilded Pavise) was listed. | log |
| PT-64 | note | balance | The party reached Lv 7 after the first four quests, which are rated Lv 4-5, and won every fight up to the Bone-Mill boss in 1-8 rounds on the auto plan. | 2641 |
| PT-65 | note | flow | The Talia board, the Journal, level-up (Auto, then Confirm, then Yes per member), the turn-in rewards, the chandler shop and the region and world map travel all worked as a player would expect. | 2265, 4710, 3358 |

- **Next**: fixes come back as rows. PT-40 to PT-45 are the ones a player will hit first.

### FB-439: PR7-72 SHOT-DIFF1 built; `fc.sh shots diff`

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `PR7-72` (`SHOT-DIFF1`), from `FB-378` Repeats.
- **What landed** (tools `fc.sh`, `tools/planner/shots_diff.py` over `shots_all.py`):
  `tools/fc.sh shots diff [base=<sha>] [scenarios=a,b|all] [out=<dir>] [tolerance=0] [pixels=0] [limit=12]`. It picks
  the scenarios from the change since the base (committed or not, new files too): none when every changed path is a
  `.md` file or under `docs/`, `.claude/`, `.agents/`, `legacy/`, `tools/planner/` or `tools/mock_ui/` (then it writes
  an index saying 0 changed and runs nothing); only the changed scenario files when nothing else changed; else every
  Emberwake scenario. It runs them on the base in a scratch worktree (skipped for scenarios already in
  `target/shots_cache/<sha>/`), then on HEAD's working tree, and writes `index.md` (base, counts, changed with pixel
  counts, new, gone), `sheet.png` (before, after and diff of each changed shot at half size, most changed first, at
  most 12) and `compare/` (the full-size sheets) to `target/shots_diff/` or `out=`.
- **Proof**: against the commit before `PR7-32` with `scenarios=story_battle_hub,battle_report`: 1 changed (the results
  shot), 2 new (the `battle_report` shots), 6 unchanged, in 4 min 43 s with a cold cache. Sheet:
  `/mnt/project-files/build-tools/pr7-72_shots_diff_sheet.png` (it also shows the old Continue past the panel edge
  that `PR7-32` fixed). A docs-only commit (`e97dd714`) selects no scenario.
- **Guesses** (`[PROVISIONAL]`): (1) frame-exact by default (tolerance 0, pixels 0), since shots run on the shot clock;
  (2) the scenario choice is coarse (docs-only, scenario files only, else all); a finer map from code paths to
  scenarios can come later; (3) Emberwake scenarios only (the sample game has one scenario and no shots to review);
  (4) a scenario that fails still gives the shots it took, with its log path printed.
- **Finding** (worth a known-issues row): a second worktree on the same `CARGO_TARGET_DIR` makes cargo treat HEAD's
  workspace crates as built after the base build (cargo keys them by path relative to the workspace, and HEAD's files
  are older), so HEAD's build links the base's crates. The tool cleans the workspace crates (`cargo clean -p` each)
  after a base build; the dependencies stay shared. So "reuses the HEAD target dir" holds for the dependencies only.
- **Tests**: `tools/planner/test_shots_diff.py` (the choice of scenarios, one changed pixel gives one changed entry and
  a sheet, identical folders give none, `add` keeps earlier shots, a docs-only index reads 0 changed).
- **Docs delta**: `STRUCTURE` (tools): "`tools/fc.sh shots diff` (`tools/planner/shots_diff.py`, `SHOT-DIFF1`): the
  shots a change moves against a base commit, with `index.md` and a contact sheet; base shots cached in
  `target/shots_cache/<sha>/`. Builders use it for a FEEDBACK item's Changed shots line." The known-issues row is
  added (`.claude/skills/known-issues`).

### FB-437: PR7-71 CONTROLS-MIGRATE1 built; fixed-key bindings reset on load

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `PR7-71` (`CONTROLS-MIGRATE1`), `FB-380` gap.
- **What landed**: engine `aegis_ui::controls`: `Controls::release_fixed(fixed)` puts each action a fixed key gives
  back on its own key through `bind_key`'s swap (the existing clash rule: the key that held the action's own key gets
  its old action back), and the fixed key gives itself again; `controls::release_fixed(settings, fixed)` does it on
  the settings object and writes the map back only when something moved. Game `lib.rs`: when the settings load, it
  runs with `FIXED_KEYS`; when an action moved, the settings are saved again and the first screen's notice reads
  `Controls: <action> went back to its own key (that key is fixed).`
- **Guesses** (`[PROVISIONAL]`): (1) the player is told by a launch notice (the row did not say); (2) only keyboard
  bindings are checked: the fixed keys are keys, and a pad button giving `r` or a digit is not a binding on a fixed key.
- **Tests**: engine `tests/controls.rs::an_action_on_a_fixed_key_goes_back_to_its_own_key` (an old file binding Tab to
  `r` with its swap half, a lone digit entry, a map with no fixed key left untouched, no map nothing written).
- **Docs delta**: engine README, after the CONTROLS-UI1B line: "* **Fixed-key migration (CONTROLS-MIGRATE1, PR7-71):**
  `controls::release_fixed(settings, fixed)` (and `Controls::release_fixed`) puts an action a fixed key gives back on
  its own key; a game runs it when its settings load and saves when it returns any."

### FB-435: PR7-32 BREPORTUI1 built; the results show the battle report

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `PR7-32` (`BREPORTUI1`), `BREPORT1` rules 5-6, `ART-18`, `PT-11`, `PT-13`; also
  `PR7-48` item 4 (reward rows).
- **What landed** (game `battle.rs`, `battle/flow.rs`, `hub.rs`, `assets/ui/theme.json`): a world battle attaches the
  kit's live metrics when it starts; the settle builds the kit's `Report` and hands it to the results. The results
  panel is sized to its content and centred, with the ornate (F2) frame and a display-face title; a table lists the
  report's columns (kit rule: only columns with a non-zero number), one row per awarded or reporting member in party
  order then the guests, each with a 32 px portrait, its state, its XP line and an amber XP bar on a basalt track
  (new theme colour `xp_track`); each number is a widget with the kit's tip (a 0 reads "-"); the best-moment line
  follows; drops draw as item tiles with their counts merged, in place of the "Dropped:" line; Level up now and
  Continue sit at the bottom. PT-11: the slide-in over the battle stops 4 px short of the window edge. PT-13: Level
  up now pushes the character sheet over the results (it used to close them first), so Close comes back to the report.
  The state gains `report` (the kit's `Report::to_json`).
- **Provisional calls**: (1) the `dealt` column reads "Damage"; (2) when the heroes would overflow the window the rows
  go compact (no XP line, the bar beside the name); (3) a member with no award and no report row is left out of the
  table, so XP no longer shows for members who sat out; (4) the results shots in `story_battle_hub` change with the new
  layout.
- **Tests**: `hub::tests::the_results_show_the_battle_report_and_drop_tiles` (row order, numbers and tips, guest row,
  best line, drop tile, state, clean at Normal and Extra large), `level_up_now_keeps_the_results_under_the_character_sheet`,
  `the_sliding_results_stay_inside_the_window`; scenario `battle_report` (a recorded first fight: report in the state,
  best line, Level up now then Escape back to the results). Shot: `/mnt/project-files/build-tools/pr7-32_report.png`.
- **Docs delta**: `GAME_BRIEF` (results line): "The battle results show the kit's battle report: a per-hero table
  (damage, taken, healed, effects, combos; zero columns hidden) with tips, the best-moment line, drop tiles and XP
  bars (`PR7-32`)."
- **Next**: `PR7-39` Stage A for the report can now follow (its row names `PR7-32` as the prerequisite).

### FB-433: PR7-70 PANELS2 built; the four screens already drew alone

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `PR7-70` (`PANELS2`), `FB-378` question 2.
- **Finding**: Save/Load slots, the forge and the dojo were never drawn over the game menu: each is a full screen
  (`overlay()` false), so `drawn_screens` already drew only it and Back showed the game menu again. `FB-378`'s question
  read them as stacked; they were not. The shop is not a screen: it is a pane the region map (or the services screen,
  off the map) draws inside its own frame, so it cannot take the game menu's place.
- **What landed** (game `menus.rs`, `forge.rs`, `dojo.rs`, `talk.rs`): `panel()` true on `Slots`, `Forge`, `Dojo` and
  `ServiceScreen` (the services screen that hosts a shop off the map), so the panel rule names them; nothing draws
  differently today.
- **Tests**: `hub::tests::the_game_menu_then_slots_shows_only_the_slots_and_back_returns_to_the_menu` (Save from the
  game menu and the Load slots draw alone and Back shows the game menu; the forge, the dojo and the services screen are
  panels that fill the screen and draw alone over it). No shot changes. Engine goldens unchanged.
- **Docs delta**: `GAME_BRIEF.md` panel list (PR7-48 line): add "Save/Load slots, the forge, the dojo and the services
  screen (PR7-70)".

### FB-431: PR7-63 XL-BATTLE1 built; the boss bar's next blow was drawn off screen

- **Status**: built (tools lane), `develop`. Shots in project files `build-tools/pr7-63_xl_battle.png` (Extra large:
  the Vane fight on the Stage view above, the cage fight on Flat below).
- **Spec**: `docs/READY.md` row `PR7-63` (`XL-BATTLE1`), `FB-376` finding.
- **What landed** (game `battle/draw.rs`, `battle/boss_bar.rs`, `battle/plates.rs`, `battle/card.rs`), by layout, no
  smaller text; Normal keeps its places:
  - The strip's goal and protect lines stack by the line heights (`strip_lines`: the goal starts where the title's
    letters end, never above 11; the protect line two thirds of a line below, never under 6), so at Extra large
    "Rout them" no longer runs into "Round 1...". A cut round title hovers to the whole of it.
  - "Boss above" (the reinforcement tag) keeps its own cell: when it would meet "Hostiles N" it takes a row under it,
    as tall as its line, and the foe plates start under it.
  - The bottom bar's message hovers to its whole text when cut; the boss name, title and blow name hover when cut.
    Plates and guest plates already hovered to their names (`PR7-58`), so "Caged Uns..." and "Crossbow..." have theirs.
  - A status tab's letter (when its icon is missing) and the card's "after KO" tag centre their text when the line is
    taller than the tag.
  - **Bug (all sizes)**: `Painter::text` returns the end x, but the boss bar took it as a width, so the boss's next
    blow (its target, impact pips and loss) was drawn past the canvas's right edge (bone-mill raid: "K" and "-57" at
    x 644 to 704), and the boss title's room test always failed, so a boss title never showed beside the name. Both
    now use the width. The card's "+N" status count had the same mistake (it moved the column past the end).
- **Tests**: `xl_battle_tests.rs` `the_battle_reads_clean_at_normal_and_extra_large` (the Vane fight `enc_evt_01`, the
  cage fight `enc_evt_02` with two caged guests, the bone-mill raid `enc_evt_03` with a guest; Stage and Flat; Normal
  and Extra large: no text past the canvas, overlapping or spilling out of its control, every cut text with a hover
  holding its words; board cells and unit tokens are pictures, not controls, so letters over them are not spills),
  `the_strip_lines_follow_the_text_size`, `the_boss_above_tag_takes_its_own_row_when_it_would_meet_the_count`.
  `XL_SHOTS=<dir>` writes the frames with the game's art. Engine goldens unchanged.
- **Changed shots** (review only): Extra large battle shots; at Normal, boss fights whose boss has a next blow (its
  target, pips and loss now show) or a title with room.
- **Left**: with six full foe plates and the tag on its own row at Extra large, the last plate may reach the panel's
  foot (no shipped fight has both; the Vane fight has three foes).
- **Docs delta**: none.

### FB-429: ENGINE-23 UISPLIT1-I Stage A for the world map; the row is complete

- **Status**: built (tools lane), `develop`. With the party loadout, recipe pane and facility screen (`FB-371`,
  `FB-379`) every screen of the row now has its Stage A.
- **Spec**: `docs/READY.md` §3 `ENGINE-23` (`UISPLIT1-I`), ready since `PR7-47` landed.
- **World map** (`emberwake_gui::worldmap_view`, fixture `ui/views/world_map.view.json` from the shipped Sunk Ward hub
  with Tier 2 open and the Tallow Vats picked):
  - Fields: `title`; `here` (the party's place); `time`; `places` (a row per place the map shows, id its qid: name,
    region line as `sub`, its first hover line as `detail` toned `info` or `dim`, "Quest" as `detail2` in `gold` while
    it has a quest or job badge, its landmark picture (else its pin), the other hover lines as `note`, selected while
    picked, off while locked); `trip` (the Travel tip for the chosen way, or "No open road to X." in `foe`); `hint`
    (the next step toward a pick with no way); `ways` (a row per way, id its index, days and upkeep as `sub`);
    `quests` (the Legend's rows, id the place, on while the map shows it); `upkeep` (the Legend's 0-upkeep warning in
    `foe`); `log` (the last 16 lines).
  - Actions: `place`, `way`, `travel`, `enter`, `camp`, `mode`, `back`; `worldmap_act` maps them to `WAct` (an unknown
    place, a way past the last or an action that is off maps to nothing).
  - The agent link's world map state gains `layout_view` (its `view` stays the kit's travel view, as the siege screen).
  - The trip tip, way tip, mode button, camp button, place hover lines and Legend rows moved into `worldmap.rs`
    functions the drawing and the view share (`trip_text`, `way_note`, `mode_line`, `camp_line`, `place_lines`,
    `legend_rows`, `legend_about`, `upkeep_line`); the drawing is unchanged.
- **Tests**: `worldmap_view` `the_view_lists_the_places_at_rest` (fields, places as drawn, the party's place, locked
  places off, Travel off, clicks), `a_pick_lists_its_ways_and_the_trip` (ways as `state.ways`, the chosen one selected,
  Travel on, way clicks), `the_agent_link_has_the_view`, `the_fixture_has_the_view_shape`; an `#[ignore]` test rewrites
  the fixture. Engine goldens unchanged.
- **Guesses**: (1) The floating Legend's lock, minimize and drag stay drawing-only (the layout will own the panel), so
  the view has no actions for them. (2) The HUD bar's icon buttons are the shared HUD's, not the map's, so not in this
  view. (3) The debug layout editor is not part of the view.
- **Docs delta**: none. `HISTORY.md`: none.

### FB-427: PR7-68 DIORAMA3 built; the Fable review's R3, R5 and R6

- **Status**: built (tools lane), `develop`. Shots in project files `build-tools/pr7-68_canal.png` (cover props, pools)
  and `build-tools/pr7-68_alley.png` (planning grid, a lantern pool); scenario `diorama_stage`.
- **Spec**: `docs/READY.md` row `PR7-68` (`DIORAMA3`).
- **What landed** (game `stage.rs`, `battle/board.rs`, `tuning.rs`, `menus.rs`, `screens.rs`):
  - R3: `light_spots` reads the stage set's `stage.fx.json` spots (the zone's own `zones.<zone>.spots` first, else the
    top level; an `at` outside 0..1 is dropped). Each spot lights an amber pool on the back rows below its `u` (ten soft
    rings, about 46 x 18 px), then an 18 px vignette darkens the stage's top, bottom and sides. No file: no light.
  - R5: a wall block not joined to the grid's edge through other blocks (`outer_wall`) is cover: it draws the set's
    `tilesets/<set>/cover/v*` prop (64 x 72 art, drawn in the block's 32 x 36 box, feet on the cell's bottom, contact
    shadow kept); blocks of the outer wall, and every block in a set without `cover/`, keep their wall cap. Display
    only; the rules see the same block.
  - R6: the board grid uses `grid_alpha` at rest and the new tuning key `grid_alpha_planning` (default 72) while
    planning; the Gameplay settings tab has a Board Grid toggle (`board_grid`, on by default) that hides it.
- **Provisional calls** (small; Art/UI or Design can retune): pool size and ring alpha, vignette alpha 70, planning grid
  alpha 72 (`tuning.json` does not set it yet), outer-wall blocks keep their caps.
- **Tests**: `a_block_inside_the_arena_stands_as_cover` (outer wall vs free block, only the block's box changes),
  `the_lanterns_light_pools_under_their_spots` (zone override, default, invalid spot dropped, amber under the spot,
  nothing far from it), `the_board_grid_setting_turns_the_grid_lines_off` (frame differs, planning alpha above rest).
  Engine goldens unchanged.
- **Changed shots** (review only): Stage-view battle shots of sets with `stage.fx.json` spots or `cover/` art (Sunk
  Ward); planning shots show a slightly stronger grid.
- **Docs delta**: none.

### FB-425: PR7-67 DIORAMA2 built; the Fable review's R1, R2, R4 and R7

- **Status**: built (tools lane), `develop`. Before and after (left: before, right: after; alley above, canal below) in
  project files `build-tools/pr7-67_before_after.png`; scenario `diorama_stage`.
- **Spec**: `docs/READY.md` row 00d `PR7-67` (`DIORAMA2`); slice 0 (the content check and layout test) landed first
  (`FB-423`).
- **What landed** (game `stage.rs`, `battle/board.rs`):
  - R1: a wall cell cap's lip line is `lip_colour`: the stage set's `lip` ("#rrggbb") in its `stage.fx.json`, bronze
    `rgb(112, 88, 54)` when the file names none (Sunk Ward today), today's pale line with no set or a set without the
    file (so the empty-set frame test holds). Provisional: Art/UI can set `lip` in `stage.fx.json` to retune it.
  - R2: the front face under row 9 takes `tilesets/<set>/front_face/<top terrain>/v*` (64 x 24 art, drawn 32 x 12,
    picked by the column); a terrain without files keeps today's face and its light line.
  - R4: the stage's clip is the board panel inset 2 (`STAGE_CLIP` 100, 30, 440 x 286), so nothing of the stage covers
    the panel's rim; the old tray (whose rounded edge ran past the panel's bottom) is now that rect filled. New
    `draw_wings` before the passes: the backdrop repeated beside itself, the edge columns' ground (and row 0's wall
    foot) one cell further out, their front face and drop shadow, then a fade to the panel colour over the 28 px gutters.
  - R7: with the wider clip a decal on column 0 or the last column is not cut; a test checks the edge plate sits inside
    the clip with room and draws.
- **Tests**: `the_front_face_comes_from_the_stage_set_with_todays_fallback` (only the face band of the columns topped by
  that terrain changes), `the_wall_lip_takes_the_stage_sets_tone`, `the_wings_fill_the_gutters_and_fade_to_the_panel`
  (only the gutters, panel colour at the edge), `an_edge_column_decal_is_not_clipped`. `the_camera_shakes_the_board_only`
  now reads the hostile panel's own rect (its old fixed rect began inside the board panel, where the wings shake).
  Engine goldens unchanged.
- **Changed shots** (review only): every Stage-view battle shot (the wider stage), the Sunk Ward ones most.
- **Docs delta**: none.

### FB-423: develop's content check refused Art/UI's front-face files; fixed in the check (PR7-67 slice 0)

- **Status**: fixed (tools lane), `develop`.
- **What happened**: Art/UI's `70cbada1` added `tilesets/sunk_ward_stage/front_face/<terrain>/v*.png` (64 x 24) for
  `PR7-67` R2. Since `PR7-64` the Sunk Ward world names that stage set, so the content check (A14) measured them as stage
  tiles (64 x 48) and failed `tools/fc.sh content check` and the gate on develop from that merge on.
- **Fix**: kit `rpgkit_content` `check/assets.rs` takes a stage set's `front_face/` at the wall face's size (64 x 24)
  and its `cover/` at 64 x 72 (SW-20, coming), with cases in `look1_sets.rs`; the game's layered-tree test
  (`region.rs` `layout_ok`) takes `tilesets/<set>/front_face/<terrain>/v*.png` and `tilesets/<set>/stage.fx.json`
  (the test failed on both too); `cover/v*.png` already matched. The drawing itself (R2) lands with the rest of `PR7-67`.
- **For Design**: an Art/UI drop under a named stage set now meets the content check on its merge. A note in
  `ART_UI_REQUESTS` (or ASSETORG2's size table) that new stage-set folders need a size row in the check first would
  avoid a repeat.

### FB-421: PR7-66 VNSTAGE1B built; every Act 1 scene with speakers is staged

- **Status**: built (tools lane), `develop`. Data and a scenario only; no code changed. Shots in project files
  `build-tools/pr7-66_act1_stage.png` (scenario `vn_stage_act1`).
- **Spec**: `docs/READY.md` row 00c `PR7-66` (`VNSTAGE1B`).
- **What landed** (seed `vnstage1b.json`, `cards` field only, through `aegis content`): `vg_debate_1`,
  `vg_int_surge`, `vg_debate_2` and `vg_debate_3` (the Act 1 scenes with `line` cards; the prologue's two were staged
  in `PR7-27`). The rule, applied the same way to each scene (a script, so a revised scene can be restaged the same
  way):
  - party members (`role: party`) stand left, everyone else right, at most two per side (the stage's limit is four);
  - the opening picture card brings on the first two party speakers walking in and the first two others standing
    right;
  - a speaker not on stage takes the place of the one on their side who spoke longest ago (a party member walks in);
  - a narration (`text`) card walks the party out right, the others stay;
  - camera Static throughout (these scenes do not move: the hub and the fissure).
- **Not built**: no new stage action was needed. Berek, Talia, Hollis and Elyse have no full-body picture, so they
  stand as the code silhouette (placeholder-first); the Art/UI queue can add `portraits/full_body/<id>.png` and they
  show with no data change.
- **Tests**: the content check passes for every staged scene; scenario `vn_stage_act1` (9 shots, one or more per
  scene) plays each scene through from the UI Gallery. The prologue's own two scenes are in `vn_stage`; the first debate closes the
  prologue sequence, so a new game now plays it staged.
- **Docs delta**: none.

### FB-419: PR7-64 DIORAMA1 built; the Act 1 diorama battle stage (D1, D3, D4)

- **Status**: built (tools lane), `develop`. Visible: every Sunk Ward fight now draws on `sunk_ward_stage` (shot in
  project files `build-tools/pr7-64_diorama.png`, scenario `diorama_stage`).
- **Spec**: `docs/READY.md` row 00 `PR7-64` (`DIORAMA1`), `SCREEN_CONTRACT_BATTLE.md` 8.1.
- **What landed**:
  - D1: seed `diorama1.json` (a `world` step) sets `stage_set: "sunk_ward_stage"` on the Sunk Ward manifest. The
    content check knew the stage set's tiles (64 x 48) and wall face (64 x 24) only, and it refused the `wall_foot`
    strips (64 x 16), so kit `rpgkit_content` `check/assets.rs` now accepts a stage set's `wall_foot/` at 64 x 16 (a
    test case in `look1_sets.rs`).
  - D3 (game `battle/marks.rs`): a ground hazard whose status has files under `props/hazards/<status>/` draws one
    (picked by the tile, like the stage tiles) centred on its plate, at 2 art px per px, kept inside the plate; then
    today's icon and rounds left. The tint plate shows only when there is no decal (`root`, smoke and the rest today).
    The hidden-hazard fading stays (alpha 150). The agent state's ground marks carry `status`.
  - D4 (game `stage.rs`): row 0's ground cells draw `tilesets/<set>/wall_foot/v*` (the world's set first, then
    `common_stage`) over their top 8 logical px, after the tiles and before the lips and props; blocks and standing
    cells draw their own. A set without the files draws today's frame.
  - Dev link `debug_ground` (dev tools only, like `debug_status`): a test hazard of a status on a tile (`at` or the
    n-th free tile `free`), for proof shots.
- **Tests**: `a_hazard_decal_replaces_the_tint_and_the_tint_stays_without_one` (burn, poison and slow resolve their
  decals; two tints draw the same frame with a decal and different frames without one, both views),
  `the_wall_foot_draws_on_row_0_only` (only the top 8 px of row 0 change), `the_dev_link_puts_a_ground_hazard`, the
  content size case. Engine goldens unchanged; no golden shots.
- **Changed shots** (no baselines; review shots only): the Sunk Ward fights in `boss_bar`, `cages_auto_plan`,
  `fight1_pommel_hint`, `floatpanels_log`, `job_fight_repair`, `objui1_marks`, `story_battle_hub` and `vn_stage`.
- **For the coordinator**: ready for Fable's in-game UI consistency review. D5 (painted unit frames) is not in this row
  (`PR7-65`).
- **Docs delta**: none beyond the row (no new files or crates).

### FB-417: PR7-27 VNSTAGE1 built; the story stage, Auto and Log

- **Status**: built (tools lane), `develop`. Visible: the prologue's meat market and mortuary cart scenes now play on
  the stage (shots in project files `build-tools/pr7-27/`, scenario `vn_stage`).
- **Spec**: `docs/READY.md` row 1a `PR7-27` (`VNSTAGE1`), `ART_UI_STUDIO.md` §3.2 and screen 4, `ART-10`, `ART-11`.
- **What landed**:
  - Kit `rpgkit_ui::vn_stage` (new): a card's `actors` (`who`, `side` left or right, `move` stay, enter or exit) stand
    on the stage rectangle (the picture rect), feet on its floor. The party enters from off the left edge and exits past
    the right; a right-side actor under Scroll or Pan is pinned to the backdrop, so it comes in from off-frame right as
    the backdrop moves. Several on one side stand in slots 56 px apart toward their edge. Cameras: Static (the picture
    fitted, as before), Scroll (the backdrop slides 280 px, the party walks in place) and Pan (as Scroll, the party
    walks forward 60 px with it). The walk lasts `walk_ms` (default 1600, eased); reduce motion puts everyone at the
    end. The speaker draws last and full, the others slightly faded. Where no full-body picture exists (Hollis today)
    a code-drawn silhouette stands in (placeholder-first).
  - `ART-10`/`ART-11`: a line or text card keeps the scene's last picture, dimmed (alpha 102 of black, about 40 %),
    never plain black, where its own card's camera left it.
  - The cutscene bar: Auto (a card that waits moves on 1.8 s after its text) and Log (a panel of the lines so far;
    Next, a click or Back closes it), left of Skip.
  - Born split (UISPLIT1 US8): view `vignette` (fields `style`, `speaker`, `text`, `shown`, `backdrop`, `dimmed`,
    `camera`, `actors`, `auto`, `log_open`, `log`; actions `next`, `skip`, `auto`, `log`), `act_of`, the fixture
    `games/emberwake/assets/ui/views/vignette.view.json` with a shape test; the screen's state carries `view`.
  - Content: `cards` take `actors` (at most 4, none on a title card, each `who` a character), `camera` and `walk_ms`
    (0 to 6000), checked by `rpgkit_content`; `kits/rpg/schema.json` help updated. Kit `art::full_body_chain` and
    `story::stage_actor` resolve the name and pictures (the character's, then its class body's).
  - Data (seed `vnstage1.json`): `vg_meat_market` (Kaelen enters on a scroll, exits on the narration) and
    `vg_bridge_cart` (Vesper and Kaelen enter on a pan; Maren and Hollis stand right; all four exit on the last card).
- **Tests**: kit unit tests (enter, exit, slots, each camera, the backdrop covering the stage), kit vignette tests
  (actors read, the kept backdrop and its camera, Auto, the Log and the view), 12 bad content cases, game tests
  `a_staged_card_resolves_its_actors`, `the_story_stage_enters_exits_and_moves_its_camera` and the fixture shape;
  scenario `vn_stage` (8 shots, no golden). Engine goldens unchanged.
- **Next to the mock**: the mock's screen 4 fills the whole screen with the backdrop and draws large rigged actors
  with mood tags; the build keeps today's picture rect and dialogue box (one stage rectangle, `ART-11`) and static
  full-body pictures. A layout file can place the stage later (the view carries everything); the animated rig is not
  built.
- **Questions for Design** (provisional calls made): (1) the dim is a flat 40 % black; (2) the Log lists narration
  without a speaker line; (3) which other scenes to stage (only the two prologue scenes are staged so far).
- **Docs delta**: `STRUCTURE.md`, kit `rpgkit_ui`: add `src/vn_stage.rs` (the story stage: actors, cameras, the kept
  backdrop). `GAME_BRIEF.md` story: vignette cards may stage actors with a camera; the bar has Auto and Log.

### FB-415: ENGINE-21 character sheet Stage A built; the Character window's view-models

- **Status**: built (tools lane), `develop`. No visible change.
- **Spec**: `docs/READY.md` row `ENGINE-21` (`UISPLIT1-H`, US1, US6, US8), like the board and save/load (`FB-369`).
- **What landed** (game `emberwake_gui`, new `character_view.rs`; its doc comment lists every field and action):
  - `character`, the window: `title`, `member` ("Name  Lv N  +P"), `tabs` (the open one); actions `member_prev`,
    `member_next`, `tab`, `close`.
  - `stats`: the summary line, the wounded line, the In party flag and count, the bunks line, the core stat rows
    (total, base, growth and spent), the derived rows, the mastery and innate passives lines; action `toggle_active`.
  - `level_up`: points left, a row per stat with the draft (new total, "+N" in gold, `-` and `+` buttons), the
    derived rows with "old -> new" in gold; actions `remove`, `add`, `auto`, `revert`, `confirm`.
  - `skills` (classic): the class and extra slot rows (AP, rank in gold below the top, `<` and `>` buttons), the owned
    list with Slot, Slotted, Full or Worn; actions `slot`, `unslot`, `rank_down`, `rank_up`, `reset`, `use_default`,
    `clear_all`, `confirm`.
  - `deck` (the deck rule): per kind a head, a hint and the slot rows; the basics; the right column's tabs; the owned
    list; the order (a row per place, on from the second); actions `deck_slot`, `deck_unslot`, `right`, `order_up`,
    `order_auto`, `reset`, `use_default`, `confirm`.
  - The Gear tab stays the `loadout` view (`ENGINE-23`). `character_act` maps each view click to the window's own
    action, refusing a row that is not there or a click that does nothing; Stage B's layout draw will hand its clicks
    to it.
  - The agent link carries the window's view under `shell` and the open tab's under `view`.
  - `party.rs`: `skills_now` reads the Skills draft, or the saved skills before the tab loads them, without changing
    the screen (`load_skills` now uses it). The deck's skill line moved out of `draw_deck` into `deck_skill`,
    `intent_label` and `passive_text`, so the view and the drawing share it. Small readers were added for the views.
- **Fixtures**: `games/emberwake/assets/ui/views/{character,stats,level_up,skills,deck}.view.json`, written by the
  ignored test `write_character_fixtures` from the test bed world (classic, and the deck rule for `deck`).
- **Tests**: `character_view` +7 (the window, each tab's fields and clicks, the state keys, the fixtures' shapes). All
  `emberwake_gui` tests pass unchanged, so the drawing is the same.
- **Left**: the deck's follow-up editor and every drag stay with the drawing until Stage B. The portrait is not a view
  field (the screen draws it from the member's look, not a file path).
- **Docs delta**: `docs/STRUCTURE.md`: `emberwake_gui/src/character_view.rs` (the Character window's view-models,
  UISPLIT1-H Stage A) beside `loadout_view.rs`. `docs/HISTORY.md` none.

### FB-413: PR7-62 INTERCEPT-LIMIT1 built; a hit forecast no longer shows an intercept whose use is spent

- **Status**: built (tools lane), `develop`. No visible change beyond the corrected forecast.
- **Spec**: `docs/READY.md` row `0k9` (`PR7-62`), from `FB-374` "Left".
- **What landed**:
  - Kit `rpgkit_battle::trigger::Uses` (new): the trigger rows' `limit` counts (this round, this battle) moved from
    the resolver's `trigger::State` to the world (`World::trigger_uses`, no save key, like `skill_uses`), so a read-only
    forecast reads the counts the resolver checks. `battle_counts` and `battle_count` moved with them (the sim's
    `feel.rs` reads `s.w.trigger_uses`).
  - `trigger::watchers` (new, private): the `ally_hit` rows with uses left at a block, in the order the hit consults
    them (a running skill's rows first, then passives and worn items, each by unit). Both the hit (`guardians`) and its
    forecast (`intercept_of`) walk it, so their order and limits match by construction. The unit order was already the
    same (both by unit id); now it is one function.
- **Tests**: kit `hit_fail.rs` +2: a spent row is not forecast and the round agrees (full damage on the target, none on
  the holder); a first cut spends the guard's use, and a second foe's cut later in the round is forecast to land and
  does. Goldens unchanged (full gate clean).
- **Left**: uses an earlier hit of the same round will spend are not foreseen (the forecast reads the counts as they
  stand; the two tests read after the first cut lands). Inside one area action, the guard's per-action use count is not
  forecast per target either.
- **Docs delta**: `docs/STRUCTURE.md` none. `docs/HISTORY.md` none.

### FB-411: PR7-33 FLOATPANELS1 built; the region legend, the ticker and the battle log float like the world map's legend

- **Status**: built (tools lane), `develop`. UI to review.
- **Spec**: `docs/READY.md` row `1g` (`PR7-33`); `preferences/ui.md` "Floating panels" (user 2026-11-13), `P-UI-03`.
- **What landed**:
  - Engine `aegis_ui::float_panel` (new, game-agnostic): the world map's `FloatPanel` moved there unchanged (place in a
    free corner or the middle unlocked, drag while unlocked, minimise to a chip whose top right is the minimise
    button's, `saved`/`restore_saved`), plus `keep` (one frame's bookkeeping of a kept panel), `draw` (the shared
    chrome: frame, amber rim while unlocked, the lock and minimise buttons, or the named chip; a game frame drawn
    between the fill and the buttons) and `chip_size`. The world map re-exports it and draws its legend as before.
  - Game `emberwake_gui` `floats.rs` (new): the settings keys `region_legend`, `region_log`, `battle_log` and `frame`,
    which takes a kept place or the panel's default corner, locked, and saves a change.
  - Region map: the "Here (N)" legend is a floating panel at the top right (as before), its title row holding the
    lock and minimise buttons; minimised it is a "Here (N)" button in place (`map_legend_open` keeps that state as
    before). The ticker is a floating panel at the bottom left; minimised a "Log" button; its bottom edge stays put
    while L opens and closes it. Both keep the HUD frame. Widgets `legend`, `legend.lock`, `legend.min`,
    `legend.open`; `log.panel`, `log.panel.lock`, `log.panel.min`, `log.panel.open` (the ticker's `log` button stays).
  - Battle: Expand (or L) opens the combat log as a floating panel over the board, half its width plus 32, at its
    top left, locked; minimised a "Combat log" button; Close and L still hide it. The entry count moved to the label
    `log.lines` (the panel widget `log.panel` is now the floating body).
- **Tests**: engine `float_panel` (2), region `the_ticker_floats_drags_unlocked_and_minimizes_in_place`, battle
  `battle/tests/floatpanels_tests.rs`; three existing tests follow the new widgets (`legend` is a backdrop with
  `legend.min`/`legend.open` instead of a section toggle; `log.lines` carries the count). Scenario `floatpanels_log`
  (shots in the project files `tools-lane/pr733/`).
- **Docs delta**: `docs/STRUCTURE.md`: engine `aegis_ui/src/float_panel.rs`, game `emberwake_gui/src/floats.rs`;
  `engine/README.md` module list gains `float_panel`.

### FB-409: PR7-31 OBJUI1 built; morale, leader, slay and guest marks on the battle screen, the objective on the results

- **Status**: built (tools lane), `develop`. UI to review (placeholder glyphs, Art/UI may restyle).
- **Spec**: `docs/READY.md` row `1e` (`PR7-31`, amended 2026-11-20: no Enraged chip); `OBJ1.md` section 3 and
  `MORALE1.md` section 3 (archived specs).
- **What landed**:
  - Kit `rpgkit_battle::obj_view` (new): the morale state's word and flavour line (never a value), the group letter
    when a battle has several groups, the leader, the planning line on how they give up, the slay target, guests and
    must-survive guests, and the results lines. `Session::morale_leader` and `Session::slay_target` (new readers).
  - Battle screen (`emberwake_gui` `battle/objchips.rs`, new): a column of 8-unit chips at each standing unit's lower
    left on both boards (flat: the tile's corner; Stage: beside the sprite's feet), bottom up: the guest's shield,
    the morale state (three bars lit 3 steady, 2 shaken, 1 wavering, gold when fearless; the group letter instead with
    several groups), the leader's pennant, the slay crown. Each chip hovers to its words (widgets `board.guest.N`,
    `board.morale.N`, `board.leader.N`, `board.slay.N`; on Stage they sit above the unit widgets for the hover).
  - The unit hover card adds "Guest", "Must survive", "Slay target: defeat it to win", "Wavering, group B: eyes on
    the exit; runs once bloodied" and "Leader: its fall shakes every group". The docked foe card puts the morale chip
    and the pennant on its portrait's lower corners (`card.morale`, `card.leader`).
  - The top bar's goal hovers to the full objective line (rule 9) and the morale line ("They give up when Captain Vane
    falls or half of them are down; shaken ones run sooner."). Contract 2 has no room for a third strip line.
  - Results: `Objective met: <goal>` in gold on a win, "The objective failed." otherwise; after a win the break lines
    (rule 12). The results state gains `objective` after a battle only.
- **Tests**: `emberwake_gui` `battle/tests/objui1_tests.rs` (7 tests on the Sunk Ward sweep and bone-mill raid, the
  Ashlands wagon raid and Ashmaw stand); scenario `objui1_marks` (three shots, in the project files
  `tools-lane/pr731/`).
- **Docs delta**: `docs/STRUCTURE.md`: kit `rpgkit_battle/src/obj_view.rs` (objective and morale reads for the
  screen); `SCREEN_CONTRACT_BATTLE.md` has no line for these chips yet (Art/UI may add one with a restyle).

### FB-334: PR7-41 items 2 and 6 landed in part; two golden-moving rules parked

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-41` (2) and (6), systems playtest PT-16 and PT-22 (@squash `PR7-41-2`)
- **Built**: PT-16 in the kit session: each member's tactic is remembered; after every draft and at Confirm, any round still holding its auto draft that a later member's draft broke is drafted again (bounded passes, last resort `ai::rest_fallback`). Rounds the player picked or edited by hand (intent, target, steps, maneuver, channel) leave the auto set and are never redrafted; such a round that is broken is refused: `Plan refused: <hero>, step N (<skill>): <cause>.` PT-22: a guard whose only trigger is `ally_hit` is not opened or filled unless an active friend is in its trigger range (now or at the friend's planned end tile); Focus Weakest takes the weakest foe the round actually attacks (else the old pick); seed `games/emberwake/data/seeds/pr7_41_decks.json` fills each founder's third active slot (Kaelen billhook_sunder, Vesper throat_carver, Hollis heavy_winch_sunder, Dunstan foundry_shatter before Strike in the assault order; Elyse choke_gas in the field order).
- **Parked (needs Tushar, Design 6b7073c)**: `/mnt/project-files/pr7-41/pt22_regrant_retarget.patch` (applies on top): (a) auto-fill does not re-grant a 2+ round self status the unit holds (ends the Fade loop at 0 AP); (b) a hero whose target is down or out of reach turns on the nearest reachable foe (`[Retarget]`), enemies keep TG4 Rest. With it the timeline golden moves in 9 of 18 fights ((a) 6: `tl_duel_reach` 1337, `tl_flank` 1337/7, `tl_skirmish` 1337/7/42; (b) 8: `tl_flank` 1337/7, `tl_skirmish` 1337/7/42, `goblin_skirmish` 1337/7/42), sim golden run 21 (windup hits 8 to 9) and repro `agent-classic-and-timeline-autoplay`. Until then the Cages test keeps its no-double-grant check off (`REGRANT_RULE = false`).
- **Guesses**: third-slot skills and their order; Elyse filled too (same empty slot); the cover check uses current and planned end tiles; an edit the session refuses still counts as hand-set (refused at Confirm, never replaced).
- **Tests**: `rpgkit_battle/tests/auto_plan_fixes.rs` (4), `emberwake_world/tests/auto_plan_cages.rs`, `emberwake_world/tests/default_decks.rs`, `ruleset_switches.rs` replays the seed; scenario `cages_auto_plan.json` on save `scenarios/saves/cages_start/`. Not tested: the auto set cleared at round start (private `begin_round`).
- **Docs delta**: `GAME_BRIEF.md` (battle): "The auto-filled plan always confirms: an auto round a later member's draft broke is drafted again, at worst it rests; a hand-edited round is never redrafted, and a refusal names the hero, the step and the cause (PR7-41). A guard that only covers friends is auto-picked only with a friend in range (PR7-41)." `STRUCTURE.md` (scenarios): "`games/emberwake/scenarios/saves/cages_start/` playtest save (PT-16/22) used by `cages_auto_plan.json` and `emberwake_world/tests/auto_plan_cages.rs`." `HISTORY.md`: "2026-11-19 PR7-41 (2, 6 part): auto plans could fail Confirm with a developer message and cover guards were picked with no one to cover; fixed. No golden re-recorded; re-grant and retarget rules parked pending a golden ruling."
- **Repeats**: the builder bisected golden moves with temporary edits three times; a kept test-only switch or a `tools/fc.sh` golden-bisect helper would save it (tools lane idea).

### FB-346: PR7-41 item 6 rules behind switches; bisect list ready for Tushar

- **Status**: open
- **Spec**: `READY.md` §4 `FB-334`, `FB-343` (Design 2026-11-19: put the parked rules behind two switches) (@squash `PR7-41-5`)
- **Built**: rule (a) `regrant` in `kits/rpg/crates/rpgkit_battle/src/round_intent_plan.rs` (`regrants()`: auto-fill does not repeat a 2+ round self status the unit holds, and such skills open last); rule (b) `retarget` in `resolver.rs` with `Command::retarget` / `target_out_of_chase` in `command.rs` (a hero whose target is down or out of reach plus chase turns on the nearest visible reachable foe, log `[Retarget]`; enemies keep TG4 Rest). Both read `aegis_core::switches::on(..)`, so release builds and the goldens see today's behaviour. Their tests (`auto_plan_fixes.rs` `a_guard_round_grants_its_status_once`, `a_hero_whose_target_falls_retargets`; the Cages no-double-grant check) skip with a stderr note when the switch is off (setting the env inside a test would race the process-wide `OnceLock`) and pass with `AEGIS_SWITCHES=regrant,retarget`.
- **Bisect** (`/mnt/project-files/pr7-41/bisect/`): `regrant` moves 6 of 18 timeline fights (`tl_duel_reach` 1337; `tl_flank` 1337, 7; `tl_skirmish` 1337, 7, 42), no sim run, repro passes. `retarget` moves 8 timeline fights (`tl_flank` 1337, 7; `tl_skirmish` 1337, 7, 42; `goblin_skirmish` 1337, 7, 42), sim run `feel 5 1 enc=tl_duel_windup,tl_skirmish party=both`, and repro `agent-classic-and-timeline-autoplay`. Matches FB-334.
- **When Tushar rules**: per rule, drop the switch (keep or delete the rule), re-record the timeline and sim goldens and the repro, and add the HISTORY row then.
- **Repeats**: `golden-bisect` does not cover the repro; the repro was run by hand per switch. A `golden-bisect repro <switch>` mode would save that (tools lane idea).
- **Docs delta**: none until the ruling.

### FB-354: PR7-36 item 10 landed; party token scale needs a call

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-36` item (10), Art/UI `AI-20`, `AI-23` (PR #67 merged) (@squash `PR7-36-2`)
- **Built**: (a) seed `pr7_36b` sets `palette` for `sabina_kress`, `widow_varka`, `berek_dorn` (existing palette files); `warrior/base/palettes/berek_dorn.json` has no `_lo` / `_hi` shades, so his armour ramps wait on Art/UI. (b) `emberwake_gui/src/region.rs` draws an 8x8 glyph per point kind (exit, chest and stash, rest, vault, lore, look) on a dark plate, on the map and in the legend: amber while usable, iron once spent, oxblood for a locked exit; people, encounters and roamers unchanged. (c) a 2 px line in the intent colour along the bottom of strip blocks (`tips::draw_block_label` `line`; Rest none) and skill-menu cells. (d) engine `aegis_ui::ground` `GroundLayer::contact` (default off) and `CONTACT_DIM` 0.55: a 1 art px dark line on the upper ground where it meets a lower non-sunken ground; in the cache key and job-file keys; the game turns it on for every land layer, water keeps its rim. (e) the same seed switches `sunk_ward_undercroft` basin, hearth, hall, entry and hollows to `drain_brick`.
- **Not built**: party token scale (ART-19). Measured: tokens about 40 px tall beside bunks and braziers of 80-90 px at 1280 x 720.
- **Guesses**: glyph pictures, plate colour (`th().panel`), 2 px line height, dim 0.55, contact lines on outdoor maps too (the playtest names the Drowned Runoff Grate).
- **Tests**: `aegis_ui/tests/ground.rs` (+2), `strip_block_tests.rs` (+1), `region.rs` glyph tests (+3); `sunk_ward_maps.rs` hub expectations updated for the `drain_brick` switch (intended). Goldens unchanged (contact off by default keeps the ground pins). 39 of 74 scenario shots changed (ground lines, glyphs, NPC palettes, icon lines; `t1_battle_beat` is frame timing, FB-347).
- **Questions**: (1) Token scale: (A) `mapview::SPRITE` 24 to about 32 (non-integer pixel scale), (B) shrink tall props through `prop_fit`, (C) leave it? (2) Art/UI to add `_lo` / `_hi` armour shades to `berek_dorn.json`?
- **Review notes**: the ground cache key hashes one more byte, so every cached ground rebuilds once. Seeds `pr7_23`, `pr7_43` and `pr7_36b` each upsert the whole `sunk_ward_undercroft` entry, so re-applying an older seed would undo later wall changes. No test replays every seed.
- **Repeats**: before/after crops and diffs by inline PIL about 3 times; a `shots-all` crop option (tools lane idea).
- **Docs delta**: `engine/README.md` Ground composer: "`GroundLayer::contact` (PR7-36, ART-20): a 1 art px dark line (`CONTACT_DIM` 0.55) on the upper ground's side where it meets a lower non-sunken ground; default off; in the key and the job file (`contact`)." `GAME_BRIEF.md`: "Region and hub maps draw a glyph per point kind (`region.rs` `MapGlyph`), amber when usable, iron when used; land grounds draw contact lines (PR7-36)." `HISTORY.md`: "2026-11-19 The kit's green exit flag and white point ring replaced on Emberwake's maps by game-drawn glyphs (PR7-36 UX-28). No golden re-recorded."

### FB-374: PR7-45 hit calc landed; forecasts share the resolver's hit builder

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-45` (HITCALC1), `FB-336`, `FB-338` answers (@squash `PR7-45`)
- **Built**: (1) kit `command::{HitScene, HitTerms, hit_terms, cover_at}`: `Command::land` and the Attack and Area paths take their hit inputs from `hit_terms` (resolver byte-identical: same RNG order, cover, flank, guard; `rpgkit_rules` `HitInput.target_stance` falls back to the live stance); `round_intent::flank_bonus_at`; `intent::hit_on_adjusted` uses `hit_terms`; `intent::scene_at` reads both units' projected tiles, the guard and both stances at the impact; `position_at` skips moves already landed; `intent::HitFail { Dodged, OutOfReach, FlankProtected, Intercepted { by, pct } }` on `HitOn.fails`; `trigger::intercept_of`, a read-only mirror of `ally_hit` (shared helper; `ally_hit` unchanged). Foe card rows, planned-loss lines and the plan forecast name failures in words (`intent_text::fail_words`, e.g. "Thrust, dodged", "out of reach", "misses (flank protected)", "about 14 (40% taken by an ally)"). (2) Once the round is over (`Phase::Replay`), an enemy's hover heads its tell "This round: ..." and drops the stale forecast (`tips::ROUND_OVER_HEAD`). (3) Seeds `act1story` and `cast1enc` moved unchanged to `data/seeds/archive/` (never scanned or re-run), so their stale `enc_pro_0a` copy with `footman_patrol` can no longer undo PT-21; `pr7_41_fight1` owns the entry and `archive/act1fix1.json` never upserted it. Editing them in place was tried first: it moves both seeds last in the replay order, where they undo later seeds (`pr7_36b` palettes, `enrage2` caps, story locations), as `seed_order.py` warns; the shipped data is unchanged and the seeds step replays 71 seeds clean. (4) `refusal::{Reason::NeedsTarget, TargetNeed}` (same text) read by `intent_text::target_control`; the keystone header names its source via `rpgkit_world::progression::keystone_source` (class, deck or graft).
- **Guesses**: "this round's line" = the tell in the past tense without the forecast (the actual per-target result would need damage events the kit does not record); a planned clash onto one tile reads the target where it stands now (`intent::target_at`, the rule `counter_on` used); a chase or dash extra counts only when the target is out of band at the action's start.
- **Left**: `intercept_of` ignores a trigger row's use limit and walks units in id order, so a forecast can show an intercept the resolver skips once the limit is spent; `Plan::forecast_hit` rebuilds the planned schedule per entry per frame (performance only); `KeystoneSource::label()` holds English in the kit.
- **Tests**: `emberwake_battle/tests/hit_forecast_timeline.rs` (forecast before `execute_single` equals the struck result over the timeline fixtures: 438 hits, 93 failures, 0 mismatches); `rpgkit_battle/tests/hit_fail.rs` (5: one per kind plus the one-tile clash); `tells_tests::the_round_over_hover_reads_this_rounds_line` and fail-word asserts; `battle_events` +1; `ally_pick_tests` read the reason. No shot changed. Goldens unchanged.
- **Docs delta**: `STRUCTURE.md`: "`games/emberwake/crates/emberwake_battle/tests/hit_forecast_timeline.rs` forecast-equals-result over the timeline fixtures; `kits/rpg/crates/rpgkit_battle/tests/hit_fail.rs` the impact-failure forecasts." `kits/rpg/README.md`: "`command::hit_terms` (with `HitScene`, `HitTerms`, `cover_at`) builds a hit's inputs for both `Command::land` and `intent::hit_on`; `intent::{HitFail, scene_at, target_at}`; `trigger::intercept_of` (read-only mirror of `ally_hit`); `refusal::Reason::NeedsTarget` / `TargetNeed` (UIs read it instead of the text); rpgkit_world `progression::keystone_source` (class, deck, graft)." `GAME_BRIEF.md` (battle): "Forecasts and the resolver share one hit builder (HITCALC1): cover, both units' tiles and stances and the guard at the landing block; a hit that fails at its impact is forecast as such (dodged, out of reach, flank protected, taken by an ally); once the round is over an enemy's hover reads 'This round: ...'." `HISTORY.md`: "2026-11-20 PR7-45 HITCALC1: `intent::hit_on`'s own copy of the hit math retired for `command::hit_terms` shared with `Command::land`; English refusal matching in `intent_text::target_control` retired for `refusal::Reason::NeedsTarget`; the round-over hover's stale forecast lines retired; seeds `act1story` and `cast1enc` archived (their stale `enc_pro_0a` copy undid PT-21 when re-run). No golden re-recorded."

### FB-376: PR7-47 polish fixes landed; Extra large battle header still cut

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-47` (POLISH-FIX4) items 2-5, `FB-344`, `FB-348` answers (@squash `PR7-47`)
- **Built**: (2) kit `terms::not_enough_upkeep_to_camp(d, w)`: "Not enough {item} to make camp." with a renamed upkeep's full name (Sunk Ward: "Not enough Warded Tallow to make camp."), else the sentence word ("supplies"); `campaign::purse::rest_refusal` and `progression::can_camp` return `String`; `purse::REFUSE_UPKEEP` removed; no item name in kit code. (3) optional save key `explore.roads_seen` (`state::ROADS_SEEN_KEY`, sorted route ids; `explore_errors` refuses a damaged one); kit `travel::open_roads`, `roads_seen`, `note_new_roads` (with no saved list it only learns the open roads; unchanged writes nothing); the GUI's session-only `Game::roads_seen` removed; `enter_game` learns the roads only for a game with no list, before the clean copy; a road opened before a save is told after the load; no save version bump. (4) seed `pr7_47`: the pawnbroker, the lodging keeper and Skerrit get a "What do you sell?" choice to a `no_trade` node and back, like Varka's `stock_shut`; Skerrit's shows `unless sunk_ward.skerrit_open`. (5) the battle bar's Menu/Exit button is as wide as its label (at least 30); only Extra large widens it.
- **Guesses**: a renamed upkeep uses its full name, not the short word; the lines: pawnbroker "I lend, I don't sell. Every spoon on that shelf has an owner still coming back for it.", lodging keeper "Beds, not wares. And the beds are full, for faces like yours.", Skerrit "Sell? I make, and not for strangers. Somebody I trust vouches for you first."; the choice text and its place first; reaching the map counts as seen (a road told but not yet saved is told again after a load).
- **Finding (outside this row)**: the Extra large audit of the battle screen still finds the header "Round 1..." cut and over "Rout them", "Hostiles 3" over "Boss above", guest names cut ("Caged Uns..."), and boss and card names spilling. Suggest a follow-up row.
- **Tests**: kit `tests/road_notice.rs` (3: first look, told once, save round-trip, old save loads, damaged key refused); `terms.rs` `the_upkeep_refusal_names_the_worlds_item`; `emberwake_world/tests/no_trade_lines.rs` (every "What do you sell?" answers with a shop or a line); `menu_retreat_tests::the_menu_button_fits_at_extra_large_text` (fails on the old draw); `tallow_upkeep.rs` expects the named refusal. Shot `/mnt/project-files/pr7-47/battle_bar_xl_after.png`. No reference shot changed. Goldens unchanged.
- **Repeats**: a `seed-from` option to add one dialogue node and choice, and a `text_scale=` option for `scenario shots` (tools lane ideas).
- **Docs delta**: `kits/rpg/README.md`: "`terms::not_enough_upkeep_to_camp` (the campaign rest's refusal names the world's upkeep; `rest_refusal` and `can_camp` return String; `REFUSE_UPKEEP` removed); `travel::open_roads`, `roads_seen`, `note_new_roads`; optional save key `explore.roads_seen` (`state::ROADS_SEEN_KEY`, no version bump)." `GAME_BRIEF.md`: "At 0 Tallow Make camp says 'Not enough Warded Tallow to make camp.'; the New road notice is kept in the save until the map shows it; the pawnbroker, the lodging keeper and Skerrit (until vouched for) answer 'What do you sell?' with a no-trade line, like Varka; the battle bar's Menu button fits at Extra large (PR7-47)." `HISTORY.md`: "2026-11-21 PR7-47: the road notice's session-only memory (`Game::roads_seen`, PR7-37) replaced by the saved `explore.roads_seen` (a road opened before a save was never told); the generic 'Not enough upkeep to rest.' replaced by the world's upkeep name. No golden re-recorded."

### FB-378: PR7-48 style pass landed; two scope questions

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-48` (STYLE-PASS1B), `FB-350` items 1-2, `FB-366` (@squash `PR7-48`)
- **Built**: (A) `region.rs` `walk_dest`, set at the three walk orders (point, marker, floor click); `walk_ring()` while the walk has steps left; `draw_walk_ring` draws a flat amber ellipse (`tokens::walk_ring()` = the theme's amber `focus`; `WALK_RING = (0.75, 0.48, 0.22)`) on the floor before the sprites; agent state key `walk_ring` (`[x, y]` or none). Static, so no Reduce-motion variant. (B) the battle heal float and the support tracer use `tokens::moss()` (the `hp` moss); `Tuning.heal_color` and its `tuning.json` key removed (an old file with the key still loads); the Gate-state overlay help now says SHUT (oxblood) / OPEN (amber). (C) engine `aegis_ui`: `Screen::panel()` (default false) and `screen::drawn_screens`: an overlay panel with another panel above it stays in the stack but is not drawn; non-panel popups still draw over a panel; the base screen always draws; the shell's draw and frame-rate check use `drawn()`; input and tick stay top-only. Game panels: game menu (`hub`), Journal, battle results, Character, bunk card, in-game Settings.
- **Guesses**: the ring sits on the floor under sprites; the panel list above; the support greens in the intent rings and skill-kind tiles stay (the battle contract names "support green").
- **Questions**: (1) Should the support colour in intent rings and skill-kind tiles also move to moss? (2) Save/Load slots (opened from the game menu) are not a panel, so the menu still draws under them: count Slots (and the shop, forge, dojo) as panels too, or keep the rule to the menu screens?
- **Left**: no shot of the heal float (the colour is proven by a test); no before/after compare of every existing shot (disk); a test doc comment in `playback_tests.rs` sits above the new heal test instead of the tracer test it documents.
- **Tests**: `aegis_ui` shell `a_panel_over_a_panel_takes_its_place_and_back_returns_to_it`; `hub::tests::the_game_menu_then_party_shows_one_panel_and_back_returns_to_the_menu`; `playback_tests::a_heal_float_is_the_theme_moss`; `region::view_tests::a_click_to_walk_rings_its_tile_until_arrival_or_a_new_order` (Reduce motion off and on); `bunks::tests::style_pass1b_a_walk_and_talk_rings_the_npcs_tile_until_arrival`. Scenarios: `screens_hub` adds shots `game_menu`, `game_menu_party`; `one_click_talk` checks `walk_ring` and adds shot `sabina_walk_ring`. Goldens unchanged.
- **Repeats**: a before/after shots compare needs a second develop build (about 1.5 GB); a kept base-shots folder per develop commit would avoid it (tools lane idea).
- **Docs delta**: `engine/README.md` (`aegis_ui`): "`Screen::panel()` and `screen::drawn_screens`: never stacked; a panel opened over another panel takes its place on screen, the one under stays in the stack and shows again on Back; popups that are not panels still draw over a panel (PR7-48)." `GAME_BRIEF.md`: "A click-to-walk in the hub or a region rings its destination tile (an NPC's tile for a walk-and-talk) in amber until arrival or a new order (agent state `walk_ring`); the battle heal float is the theme's moss; the game menu, Journal, Character, results, bunk card and in-game Settings are panels, one shown at a time (PR7-48)." Scenarios README: `one_click_talk` "PR7-48: the walk rings the NPC's tile (shot `sabina_walk_ring`)"; `screens_hub` "plus the game menu and its Party in its place (`game_menu`, `game_menu_party`)". `HISTORY.md`: "2026-11-21 PR7-48: `tuning.json` `heal_color` (bright green) retired for the theme's moss; a panel opened over a panel no longer draws both. No golden re-recorded."

### FB-380: PR7-49 controls UI landed; stale fixed-key bindings are not migrated

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-49` (CONTROLS-UI1B), `FB-351`, `FB-352` answers (@squash `PR7-49`)
- **Built**: (1) engine `Controls::key_hint(action)` ("P") and `Controls::hint(action)` ("Esc / Pad B"); the game reads them through `controls_view::hints(settings)`: HUD button cards, region and world-map Help (`region::region_help`, `worldmap::world_help`), the region log card, the Character window's Q and E marks, Close on board, recipe and facility, siege Fight, the Auto-Confirm tip, the battle cards (Restart, Step, Menu, Exit, log, Next block, Fill, Self), the battle messages (move "Esc cancels", next round, tells), "Inspecting X (Esc clears)" (`battle/tips.rs::inspecting`), the Tab skill-menu line, and Settings "Keep changes [Enter]" / "Apply display [Enter]". (2) pressing the action's own key or button ends a waiting rebind with nothing written (`Outcome::Cancelled`); `Rebind.until` = `WAIT` (5 s) from the start, moved on any input, on the shot clock; `Rebind::expire` ends it (the swap question too); every button stays bindable. (3) engine `Fixed { key, label, screen }`; `Rebind::input(actions, fixed, settings, input)` refuses a fixed key (`refused_fixed`); the game's `FIXED_KEYS`: `y` (question boxes), `r` (battle screen), `1`-`9` (conversations; `1`-`4` the battle screen), e.g. "R stays fixed: the battle screen uses it (Restart the battle). Press another key, or Esc to cancel."; Journal and Undo still have no pad button. (4) controls rows: key in `detail`, pad button in `detail2`; a waiting row shows "Press a key or button..."; the default layout already draws two columns. Also: the question box (`screen::Ask`) answers `y` / `n` by the raw key, so moving Next block off N no longer changes how No is answered.
- **Guesses**: hover-card titles show key plus pad button, sentences show the key only; a buttonless action's second column reads "no pad button"; the own input ends as Cancelled (before, it wrote the same binding); any input, a refused one too, restarts the wait; fixed keys match the raw lower-case key; the world map's Menu button still shows no key; the HUD's drawn stand-in letters (P, J, I, shown only while an icon file is missing) stay; the fixed-key screen names' wording.
- **Left**: no migration for a settings file that bound an action onto `r` or a digit before this row (it keeps that binding until Reset); no test hovers a ghost block for the Tab line; Art/UI's `settings_controls.json` (AI-27) does not exist yet (the view is ready).
- **Tests**: engine `tests/controls.rs` +3 (hint follows the binding; own input or wait ends a rebind; fixed key refused); `shell.rs` question box answers raw `n` after a swap; `controls_view` +3 (own button and timer; fixed key names its screen; two columns); `hud.rs` `the_hud_and_help_name_the_bound_keys`; `input1_tests::the_battle_hints_follow_a_rebind` (cards, inspect message and panel, Step card); `the_controls_tab_rebinds_saves_and_resets` expects "Keep changes [K]". Scenario `controls_rebind`: `r` refused (shot `controls_fixed`, sizes 1.0 and 1.3), own `e` and own Pad B end a rebind, a 5200 ms pause ends one. All `controls_rebind` shots change (second column); hover cards on Esc read "(Esc / Pad B)". Goldens unchanged.
- **Docs delta**: `engine/README.md` (input row): "CONTROLS-UI1B (PR7-49): `Controls::{key_hint, hint}` (on-screen hints read the bound key, plus the pad button where there is one); `controls::{Fixed, WAIT}`; `Rebind::input(actions, fixed, settings, input)` refuses a fixed key (`refused_fixed`); a waiting rebind ends on the action's own input or after `WAIT` (5 s, `Rebind::{until, expire}`, shot clock); the question box (`screen::Ask`) answers `y` / `n` by the raw key." `STRUCTURE.md` (emberwake_gui row, optional): "`region::region_help` / `worldmap::world_help` (help lines from the control map)." `GAME_BRIEF.md`: "Every key hint (HUD, Help, battle cards) shows the bound key; a waiting rebind ends on the action's own button or after 5 s; Y, R and the digits are fixed keys a rebind refuses (PR7-49)." `HISTORY.md`: "2026-11-22 PR7-49: fixed key letters in on-screen hints (`REGION_HELP`, `WORLD_HELP`, HUD '(P)' etc.) replaced by hints from the control map; a rebind onto the action's own input no longer rewrites it. No golden re-recorded."

### FB-382: PR7-65 painted unit frames landed; three questions

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-65` (DIORAMA1B), contract `SCREEN_CONTRACT_BATTLE.md` 8.1 D5 (@squash `PR7-65`)
- **Built**: engine `aegis_core::assets::Clips` reads optional `foot` (the frame row the feet stand on) and `painted` (default false), with listed problems for a bad value; `aegis_ui::art::load_body` skips the palette swap for a painted sheet. Game `stage.rs` `PAINTED_CELLS = 1.15`, `Geom::sprite_px(n, frame, foot)` and `sprite_rect_n`: a body with a `foot` row is scaled so frame top to foot row is 1.15 x 32 px per footprint cell (about 36.8 px; 2 x 2 about 73.6 px), centred on the foot point (a 2 x 2 unit's centre foot point); `board.rs` `draw_stage` uses that square for the body, feet and see-through passes and the hover bounds. Sheets without `foot` and the Flat view draw exactly as before. Missing clips fall back to `idle` (now tested). Kit content check A4 skips a painted body. Seed `pr7_65` (19 `--fields art` unit steps): `founder_kaelen` `warrior/kaelen`, `founder_vesper` `rogue/vesper`, `founder_maren` `priest/maren`, `founder_hollis` `ranger/hollis`, and `castellan_footman`, `watch_crossbowman`, `tallow_dredger`, `ash_crawler` (`crawler/ash_crawler_painted`), `mortuary_flagellant`, `bailiff_enforcer`, `inquisitorial_purifier`, `calcified_husk`, `render_beast` (`beast/render_beast_painted`), `captain_vane`, `inquisitor_prelate_nemeth`, `proctor_quell`, `guild_master_orlov`, `overseer_grendel`, `calcified_amalgam` as the row lists.
- **Placeholders**: the portrait path follows the body key, so seven common-tier units would have changed portrait; byte copies of what each showed before now sit at `games/emberwake/assets/portraits/common/` (`beast_render_beast_painted`, `crawler_ash_crawler_painted`, `rogue_tallow_dredger`, `warrior_castellan_footman`, `rogue_mortuary_flagellant`, `warrior_calcified_husk`, `ranger_watch_crossbowman`). For Art/UI to replace or own.
- **Guesses**: "a cell" = the cell width (32 px; on the 24 px height units would be about 27.6 px, smaller than today's sprites); the scaling is keyed on `foot`, not on `painted` or frame size; the Flat view keeps the 28 px token square; exploration and hub maps show the painted sheets fitted in the token square (no foot-row scaling there); overhead chips and `Geom::chest` keep their token-based anchors (close enough at about 37 px).
- **Questions**: (1) Is 1.15 cells measured on the cell width (built) or height? (2) Should the exploration and hub maps also scale painted sheets by the foot row? (3) Should portraits be looked up by unit instead of body key, so painted bodies need no portrait copies?
- **Tests**: `aegis_ui/tests/painted_bodies.rs` (3: foot and painted parsing with errors; painted skips the swap; idle fallback); `stage.rs` (a 112 x 96 frame with foot 93 stands 1.15 cells on the foot point; a 224 x 192 2 x 2 frame stands 2.3 cells; old sheets and Flat keep the token square). `ruleset_switches.rs` and `big_units.rs` keep the shipped `art` when replaying old whole-unit seeds onto a scratch folder with no assets (test assumptions; no assertion weakened). 55 of 126 shots change (painted units in battle, hub and region scenarios). Goldens unchanged.
- **Docs delta**: `engine/README.md` (assets): "`clips.json` may give `foot` (the frame row the feet stand on) and `painted` (default false: a painted sheet takes no palette swap); `Clips::foot`, `Clips::painted` (DIORAMA1B)." `kits/rpg/README.md` (content checks): "A4 skips a painted body." `GAME_BRIEF.md` (battle stage): "A unit body with a `foot` row stands `PAINTED_CELLS` = 1.15 cells (32 px) tall per footprint cell, feet on the foot point; other sheets and the Flat view keep the token square; the founders and every Act 1 foe and boss use painted sheets (PR7-65)." `HISTORY.md`: "2026-11-22 PR7-65: the founders and Act 1 foes and bosses moved from shared class bodies (`warrior/2h_axe`, `rogue/base`, `brawler/base` and others) to painted sheets; the old bodies stay for other units and fixtures; common portrait placeholders copied to the painted body keys. No golden re-recorded."

### FB-384: PR7-69 Grate north quay retagged to flagstone (cells guessed)

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-69` (GRATE-GROUND1) (@squash `PR7-69`)
- **Built**: seed `pr7_69` (`seed-from maps sunk_ward_runoff_grate --fields ground`) replaces only that map's `ground` grid: every walkable `m` (mud) cell in rows 0-11 becomes `f` (flagstone), 107 cells: y3 x1-5, 7-10, 12-18, 22-24; y4 x1, 3, 5, 7-18, 22-24; y5 x1-5, 9-18, 22-24; y6 x5-6, 23-24; y7 x5-7, 24; y8-y10 x1-8, 23-24; y11 x1-7, 24-30. Unchanged: walls and water, the silt bank's footprint (x1-4, y6-7, kept mud as silt), the east soot yard, the catwalk, the cobble ring road (rows 12-18) and the mud borders.
- **Guess**: the Drive reference (`image_pipeline/flow/out/Region Maps`) is not reachable from the cloud, so "north quay" was read as all walkable mud north of the ring road, keeping the silt bank. If the reference keeps some mud, name the cells and a follow-up seed replaces the grid.
- **Shots**: `/mnt/project-files/pr7-69/grate_before.png`, `grate_after.png`: only the quay ground changes (brown mud to dark flagstone around the pool and tablets).
- **Checks**: `content check` data ok; seeds, test, scenarios, shots-exact clean.
- **Docs delta**: none.

### FB-386: PR7-73 painted bodies on the maps and portraits by unit landed; three questions

- **Status**: open
- **Spec**: `READY.md` §1 `PR7-73` (PAINTED-MAPS1), `FB-382` questions 2 and 3 (@squash `PR7-73`)
- **Built**: (a) `stage::painted_px(tile_w, n, frame, foot)` factored out of the battle stage (`Geom::sprite_px` calls it; the stage is unchanged); `mapview::body_foot(cell)` (centred, cell width / 8 above the bottom edge) and `mapview::body_rect(cell, clips)`: a body with a `foot` row stands `PAINTED_CELLS` (1.15) of the map's own tile width, foot row on the foot point; other sheets keep `sprite_rect`. `region.rs` (hub and region maps) uses it for the leader, people markers, hub members and `draw_sprite`; labels, badges, agent-link sprite rects and the roof check follow the drawn body; click areas stay the cell (EXPLOREVIEW1 3.5); the party still sorts by its tile's bottom edge. (b) kit unit field `portrait` (string, omit_default, max 64: a file stem under `portraits/common/`); `rpgkit_content::art` `portrait_key`, `keyed_portrait`, `portrait_chain_keyed`, `face_chain_keyed` (the old chains call them with an empty key); the GUI and story lookups pass the key; content check A5 warns when a key's file is missing and drops the body warning when it exists. Seed `pr7_73` (7 portrait-only steps): `render_beast` `beast_render_beast_painted`, `ash_crawler` `crawler_ash_crawler_painted`, `tallow_dredger` `rogue_tallow_dredger`, `castellan_footman` `warrior_castellan_footman`, `mortuary_flagellant` `rogue_mortuary_flagellant`, `calcified_husk` `warrior_calcified_husk`, `watch_crossbowman` `ranger_watch_crossbowman`. All files stay where they are (six are Art/UI #87's real portraits); nothing changes on screen today, the units are just decoupled from their body key.
- **Guesses**: lookup order: a unique tier's own file, then the unit key, then the body's common portrait, then the class base; a world character uses its base unit's key only when it wears the unit's body (own `art` empty).
- **Questions**: (1) Move the seven files to unit-named paths (e.g. `portraits/common/render_beast.png`; Art/UI would update `assets/**/README.md`)? (2) Keep the old pre-painted `beast_render_beast.png` and `crawler_ash_crawler.png`? (3) Should a unique-tier unit's key rank above its own tier file?
- **Tests**: `mapview::tests::a_painted_body_stands_1_15_tile_widths_on_its_foot_point` (both views, every zoom; no foot row keeps the token square); `region::sprite_tests::a_painted_leader_stands_on_the_tiles_foot_point`; `portraits::tests::a_units_portrait_key_comes_before_its_body_portrait`; kit `rpgkit_content/tests/portrait_key.rs` (2); `asset_checks::a_portrait_key_is_checked_and_stands_in_for_the_body_portrait`; `ruleset_switches.rs` keeps the shipped `portrait` when replaying old seeds. Hub and region shots with painted founders or foes move. Goldens unchanged.
- **Repeats**: proof shots of a region from a save need a throwaway scenario (`scenario shots` covers fights and menus); a multi-id `seed-from` would replace a loop (tools lane ideas).
- **Docs delta**: `STRUCTURE.md`: "`kits/rpg/crates/rpgkit_content/tests/portrait_key.rs`: a unit's portrait key and the portrait lookup order." `kits/rpg/README.md` (unit fields): "`portrait` (optional, PR7-73): the unit's own common portrait, a file stem under `portraits/common/`; lookup: a unique tier's file, then the key, then the body's common portrait, then `<class>_base`; a world character wearing the unit's body shows it too; check A5 checks the key's file." `GAME_BRIEF.md`: "Hub and region maps scale a body with a `foot` row like the battle stage (1.15 of the map's tile width, feet on the tile's foot point; `mapview::body_rect` over `stage::painted_px`); portraits look up the unit's `portrait` key before the body key (PR7-73)." `HISTORY.md`: "2026-11-22 PR7-73: portraits are no longer found only by body key; units can name their portrait (seed `pr7_73` for seven painted-body units); the files stay. No golden re-recorded."
