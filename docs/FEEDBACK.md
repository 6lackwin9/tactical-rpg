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
| `PR7-76` (`ACT1-FIX2`) | done | `develop` (squash `PR7-76`) | Act 1 agent-run fixes PT-42..63 (Here list usable points and edge arrows, no repeat point answers, roads drawn at rest, vignette two-press, job places and locked reasons, ready quest in legend, Make camp reason, directional exits, walk resumes after a won fight, batched turn-ins, can't-wear badge, dialogue change log, opaque Journal, skill picks logged, Tithing at-0 line); seed `pr7_76_hub_cast` keys five hub NPC bodies; grate silt parked; shots `/mnt/project-files/pr7-76/`; goldens unchanged; `FB-388` |
| `PR7-78` (`ACT1-DATA1`) | done | `develop` (squash `PR7-78`) | q_a1_m4 already grants its promise (no change); `rec_level` on the eight flag-job quests, with a test; Crucible and Overlook already name the interlude in `closed_text` (no change); seed `pr7_78_quests`; goldens unchanged; `FB-390` |
| `ENGINE-34` (`BONEMILL-CHECK1`) | Report | `develop` | Bone-Mill won by hand on try 1 at Lv 7 (the auto plan still loses it); the Crucible lost 5 times at Lv 8 (1 auto, 4 hand), so the act's end was not reached; no debug; PT-66..84; `FB-392` |
| `PR7-81` (`DECOR1`) | done | `develop` (squash `PR7-81`) | Display-only encounter field `decor: [{art, x, y}]` (kit schema, typed reader, content check), drawn by the stage with `lights` on its art key; no rule reads it; goldens unchanged; `FB-396` |
| `PR7-84` (`ACT1-DATA2`) | done | `develop` (squash `PR7-84`) | `rec_level` 7, 7, 8 on the Bone-Mill, Vats and Crucible; the Crucible and Overlook lock hint names the Bone-Mill and Vats turn-ins; the winch drum gets a Here row; Dunstan's battle line is wordless; six `pr7_84_*` seeds; `FB-398` |
| `PR7-79` (`DIORAMA-DRESS1`) | done | `develop` (squash `PR7-79`) | Ten Act 1 Sunk Ward battle boards dressed through `decor` seeds (`cover/v3` lamps on blocks, edge `lantern_pole`s, edge props), no walkability change; fix: the arena launch path reads `<world>:<id>` so its board gets the world's stage set; shots `/mnt/project-files/pr7-79/`; goldens unchanged; `FB-400` |
| `PR7-86` (`DECOR-GEN1`) | done | `develop` (squash `PR7-86`) | Generated boards (no `rows`, no `decor`) draw 2 to 6 display-only decor pieces from the world manifest's `decor_pool` on their own seeded stream: cover keys on blocks or edges, props at the back only; seed `pr7_86_decor_pool`; goldens unchanged; `FB-402` |
| `PR7-85` (`XL-FIX2`) | done | `develop` (squash `PR7-85`) | "R 1/12" at Large and Extra large with the goal line clear; Stats and Gear rows grow with text size (second column when needed); every XL cut hovers to its full words; engine `Screen::audit_ignores` and cut `hover`/`bare_cut`; Normal shots unchanged; `FB-404` |
| `ENGINE-35` (`ACT1-END-RUN1`) | Report | `develop` | Crucible won by hand on try 3 at Lv 8 (no debug); Act 1's end reached at Tithing 19; Kaelen held the winch and was captured; PT-85..100; `FB-406` |
| `PR7-89` (`ACT1-FIX4`) | done | `develop` (squash `PR7-89`) | Captured hero keeps the dose; `[Wave]`, `[Hazard]` and foe `[Escape]` log lines; results list who fought and a captured hero's XP; Journal scrolls clear of Back; hover cards close at battle end; doubled words fixed; portcullis is a look point matched to its art; the post-act road starts at the Undercroft; goldens unchanged; `FB-408` |
| `PR7-91` (`ACT1-FIX5`) | done | `develop` (squash `PR7-91`) | The round banner hides once a battle has an outcome; blocker and forced-move lines name the terrain ("blocked by Lava", "the rubble"), a plain floor still "Floor"; goldens unchanged; `FB-410` |
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
| `PR7-77` (`DIORAMA5`) slice 0 | Built (tools lane) | `develop` | The content check and the layered-tree test accept ground sheets, accent decals, lip art and the exit path and gate props; R1 to R5 follow. FB-447 |
| `PR7-77` (`DIORAMA5`) R1 to R5 | Built (tools lane) | `develop` | The battle ground composed by the region composer from the sheets, hashed variant picks, accent decals, lip art, additive lamp light with `lights`, the exit path decal and gate, rubble on the zone floor. FB-449 |
| `PR7-77` (`DIORAMA5`) R6 | Built (tools lane) | `develop` | No grid lines at rest, the "Show full grid" setting (off by default), coherent 2 to 4 cell blobs, no edge pieces over the composed ground. FB-451 |
| `PR7-82` (`RETRY-SAVE-FIX1`) | Built (tools lane) | `develop` | A lost fight's retry count (`retry:<encounter ref>` in the save's `rolls`) passes the save check, so the autosave after a loss works again; the retry seed is unchanged. FB-453 |
| `ENGINE-33` (`XL-SWEEP1`) | Built (tools lane) | `develop` | Every scenario shot at Normal and Extra large text with a text audit per shot (`set=`, `audit=1`); four small fixes (plate HP, card name, boss name, world map side panel); the shop layout for Art/UI and three findings for Design. FB-455 |
| `PR7-80` (`WALLMASS1`) | Built (tools lane) | `develop` | A wall run shares one cap (one variant per run, a ground sheet's cap continuous when there is one, a back-row shade); top and bottom exits draw `exit_path_north` (flipped for the bottom). FB-457 |
| `PR7-83` (`ACT1-FIX3`) | Built in part (tools lane) | `develop` | PT-73, 75, 76, 79, 81, 84 fixed; PT-68 and PT-69 text fixed; PT-70 and PT-71 fixes move the sim golden, so held for Design with options; PT-67, 72, 74, 80 reported. FB-459 |
| `ENGINE-36` (`TOOLS-REPEATS2`) | Built (tools lane) | `develop` | `scenario shots save=` (a name or a folder path) shoots a map from a save with no scenario file; `seed-from --ids a,b,c`; `playtest start keep_saves=<dir>`. FB-461 |
| `PR7-87` (`FORECAST-FIX1`) | Built in part (tools lane) | `develop` | The forecast follows disengages, carried moves, pushes and stealth (PT-67, PT-69); every dropped later pick is said at pick time; PT-72's costs and "No path" not reproduced, reported. FB-463 |
| `ENGINE-37` (`LOOK-POINT1`) | Built (tools lane) | `develop` | A point's `use` may be `look` (listed in Here, answers with its line, sets nothing, never spent); the winch drum uses it and drops `sunk_ward.winch_seen` (seed `engine_37.json`). FB-465 |
| `ENGINE-38` (`LORE-RENAME1`) | Built (tools lane) | `develop` | `aegis content refs <kind> <id>` lists every reference; `aegis content rename <kind> <old> <new>` is a dry run, `--apply` rewrites data and seeds (only the matching strings) and moves the asset files, kept only when it adds no problem. FB-467 |
| `ENGINE-39` (`TARGET-PRUNE1`) | Built (tools lane) | `develop` | `tools/fc.sh prune [target=] [--dry]` keeps only the newest test binary per name in `target/*/deps` and removes the incremental folders; `tools/fc.sh gate` runs it first when under 6 GB is free (`FC_PRUNE_MB`). FB-469 |
| `ENGINE-40` (`ENGINE-GUIDE2`) | Built (tools lane) | `develop` | `engine/docs/authoring/battle_stage.md` (composed ground, stage sets and `stage.fx.json`, decor, painted sheets); scenario run keys and `shots diff` in `scenarios.md`; `content refs` / `rename` and look-only points; the sample game's field on the engine's ground composer. FB-471 |
| `ENGINE-41` (`STAGE-KIT1`) | Built (tools lane) | `develop` | The battle stage module (geometry, `stage.fx.json` reader, composed board, decor, wall-run caps, exit path and gate, draw passes) moved to `rpgkit_ui::stage`; the game re-exports it and gives its sizes through `StageSizes`. Fixture hash, goldens and shots unchanged. FB-473 |
| `ENGINE-42` (`KIT-SAMPLE1`) | Built (tools lane) | `develop` | `games/kit_sample/`: a two-a-side fight from its own catalog on the kit's battle stage (composed floor, one zone, a lit fire pit), engine and kit crates only; scenario in the gate; `engine/docs/authoring/kit_game.md`. FB-475 |
| `ENGINE-43` (`TERRAIN-PARTIAL1`) | Built (tools lane) | `develop` | A terrain table may list only the types it changes; the rest take the kit's code defaults with no problem and no warning; a listed row is still checked whole. FB-477 |
| `PR7-88` (`PLAN-FIX2`) | Built in part (tools lane) | `develop` | PT-86, PT-87 (ghost claims), PT-88, PT-89, PT-91 fixed with tests, goldens unchanged; PT-85 and PT-87's foe start tiles not done (findings in the item). FB-479 |
| `PR7-90` (`FORECAST-MOVE1`) | Built (tools lane) | `develop` | A hit forecast reads where the target stands in the block the hit lands, after every move booked before it (walks through friends, tiles freed earlier, foes stepping in, re-paths); three logged cases as tests, goldens unchanged. FB-481 |
| `ENGINE-45` (`FEEL-PROBE1`) | Built (tools lane) | `develop` | `aegis_ui::feel` times each input (arrival, handler done, frame start, presented) and every frame; `--feel` or the setting `feel` shows an overlay and writes `feel.log`; scenario step `feel` (p50, p95, worst; in the gate on `hello_walk`); report: battle planning frames 36 to 80 ms and two clicks over 100 ms, the rest under. FB-483 |
| `ENGINE-46` (`FEEL-FIX1`) | Built (tools lane) | `develop` | Stage floor layer cache (`Painter::cached`), panels composed once, per-frame forecast memo (`intent::memo`), Settings warmed before the first frame (`Context::warm`); `feel_battle` and `feel_settings` budgets (100 ms input, 33 ms frame; 2.5 x in dev) pass; planning frames 17 / 28 ms p50 / worst (were 38 / 80), Settings first draw 13 ms (was 57), `strip.block1` 55 to 59 ms (was 113; the guest ghost re-sent each click is reported). FB-485 |
| `ENGINE-47` (`GHOST-SETTLE1`) | Built (tools lane) | `develop` | The guest's ghost was not flipping (a draft settles on its own board; test added); `ally_moves` kept while nothing it reads changes (a run that fixes a step's tile is not kept, so plans stay bit for bit); `strip.block1` 43 to 46 ms against a 50 ms `feel` budget; goldens unchanged. FB-487 |
| `ENGINE-48` (`INPUT-REACH1`) | Built (tools lane) | `develop` | Scenario step `reach` (keys and pad reach, focus shown, back) in the gate on `reach_menus`; every screen walked; fixed a cutscene focus ring off the canvas and dead arrows on cutscenes and in hub talk, board and shop; party skill and gear slots mouse only, for Design; goldens unchanged. FB-489 |
| `PR7-92` (`INPUT-REACH2`) | Built (tools lane) | `develop` | Skill, gear, pouch and loadout slots pick (Enter), place or swap (Enter), drop the pick (Esc) and empty (Space, pad X) by key, hint line names the keys; the battle board is one Tab stop with an arrow tile cursor (ring 121 to 14); `reach_party` and `reach_battle` gate scenarios with `fail` on; goldens unchanged. FB-491 |
| `ENGINE-49` (`WALK-FEEL1`) | Built (tools lane) | `develop` | `feel` measures movement (held keys and d-pad, long frames, `motion`: first move, beat, overshoot, turns, camera); hub and Cages map meet every target; click to walk moved a frame late (fixed); `walk_feel` gate scenario with budgets; six feel choices for Design; goldens unchanged. FB-493 |
| `ENGINE-50` (`MAP-FRAME1`) | Built (tools lane) | `develop` | the region map's still layers kept and moved with the camera (engine `Painter::scroll_begin`), marker views read once per state: release median 18 to 6.8 ms, worst 51 to 19 ms on the hub; `frame_p50_ms` budget, `walk_feel` holds 16.7 / 33 ms; a key press after a long frame dated at most 25 ms back (`stall_ms` input tests it); shots diff clean; goldens unchanged. FB-495 |
| `ENGINE-51` (`SWITCH-FEEL1`) | Built (tools lane) | `develop` | every screen switch timed to its first settled frame (`feel` `switch`, `aegis_ui::switchfeel`, `Screen::transition`): all under target in release, no uncovered frame over 100 ms; fixes: double decodes, a fight's sheets decoded side by side, Save/Load warmed, the world map's background read ahead, the ground's noise side by side; gate scenarios `switch_feel`, `switch_feel_story`; goldens unchanged. FB-497 |
| `ENGINE-52` (`COMPOSE-PAR1`) | Built (tools lane) | `develop` | the ground composer's per-pixel pass and noise in bands of rows on up to 4 threads, the same bytes (tests on two maps, battle boards, an engine job); compose 40 to 26 ms alone; Continue (107 to 124 ms) and a fight with no intro (101 to 135 ms) miss 60 ms, reported with options; `switch_feel` budgets for both 200 ms; goldens unchanged. FB-499 |
| `ENGINE-53` (`SWITCH-AHEAD1`) | Built (tools lane) | `develop` | the composer's fields' blur, rims and contact lines on threads (same bytes); built ahead at idle (`Screen::warm_next`, `ground::keep`): the title's Continue map (ground composed on a thread, map drawn once unseen), a fight the party walks into or stands near and the fight after a vignette (started on a copy of the game, taken when not stale); Continue 19 to 22 ms, The Sweep 27 to 30 ms; test that a fight built ahead equals one built on the switch; `switch_feel` budgets 120 ms; goldens unchanged. FB-501 |
| `ENGINE-54` (`WARM-SPREAD1`) | Built (tools lane) | `develop` | build-ahead work on threads (decodes, grounds, fight starts; the kit's round plan `Send`), the rest drawn unseen in slices one a frame behind an empty canvas `bounds` after 300 ms still, none on a frame with input; no frame over 33 ms while warming (worst 22 ms by The Sweep on cold art), two cold slices of the fight 9 to 18 ms; Continue 19 ms, The Sweep 21 ms; new `walk_feel_fight`, `switch_feel` `title_warm`; goldens unchanged. FB-503 |
| `ENGINE-55` (`AUDIO-FEEL1`) | Built (tools lane) | `develop` | `feel` times each sound from its input or frame to its first audible sample, with the output's buffer and underruns; UI sounds 12 to 20 ms, hits within 6 ms of their frame, music 15 to 30 ms after its ask (was 96 ms on Continue), no underruns in release; music asked before the draw, decoding off the frame, a fixed 512-frame buffer; audio budgets on `feel` steps, new `audio_feel`; goldens unchanged. FB-504 |
| `ENGINE-56` (`AUDIO-FEEL2`) | Built (tools lane) | `develop` | menus answer by sound by rule (confirm, back, error; `Screen::ui_sound` overrides, the round controls and volume steps keep theirs); a soft map footstep with its own Footsteps volume under Effects (placeholder `map_step.wav`); battle music fades in over 0.4 s (`battle_music_fade_ms`); stings warmed with the battle screen; `audio_feel` gains a confirm, a back and a map step; goldens unchanged. FB-505 |
| `ENGINE-57` (`FEEL-RELEASE1`) | Built (tools lane) | `develop` | `switch_feel_story`'s release failure: a release build hides the developer surfaces, so New Game skipped the world picker; `dev=1` (dev tools builds only) shows them and every scenario launch passes it; `tools/fc.sh feel` runs every feel and reach scenario in release and prints one table (38 of 39 steps pass; `feel_battle` `battle_block` 51 to 74 ms against 50); goldens unchanged. FB-506 |

## 2. Feedback Items (Append-Only)

### FB-506: ENGINE-57 FEEL-RELEASE1 built; release scenario runs take the dev path, `tools/fc.sh feel` prints one table

- **Status**: built (tools lane), `develop`. Game GUI (`emberwake_gui` launch, menus, battle overlays), engine CLI
  (`aegis_cli` scenario launch), `tools/fc.sh feel`, `tools/planner/feel_table.py` with its tests, authoring guide
  `engine/docs/authoring/scenarios.md`; no data. **Every golden unchanged**.
- **Spec**: `docs/READY.md` row `ENGINE-57` (`FEEL-RELEASE1`); the approach agreed with Design in the thread.
- **1. The cause of `switch_feel_story`'s release failure**: the game shows its developer surfaces only in a debug build
  with the dev tools (`DEV_BUILD`, PR7-37 PT-01 and PT-14). A release build is a player build, so New Game starts the one
  player world at once (its opening vignette) where the gate's dev build opens the world picker the scenario clicks.
  **Fix**: a build with the dev tools shows them when launched with `dev=1` (`dev_build()`; `DEV_KEYS`, the F-keys, stay
  debug only), and the scenario driver passes `dev=1` on every launch, so one script takes one path in either profile.
  A shipped player build (no dev tools) ignores `dev=1`. Tests: `dev_1_shows_the_developer_surfaces_only_with_the_dev_tools`
  (lib; fails without the fix), `new_game_starts_the_one_world_unless_the_developer_surfaces_show` (menus, `at_once`),
  and the driver's launch keys (`aegis_cli` scenario). `switch_feel_story` now passes in release.
- **2. `tools/fc.sh feel [out=<dir>]`**: every scenario with a `feel` or `reach` step, each game (Emberwake's 10, `hello_walk`),
  no window, release build, release budgets; one table, one row per step: result, input, frame p50 and max, switch (own
  part), move, interface sound, hit, music, underruns, reach dead ends; `*` over the step's budget; exit 1 on a failed
  scenario. Not in the gate; a few minutes.
- **3. Reading it**: `engine/docs/authoring/scenarios.md` "Reading the feel table"; its "What a run owns" table gains the
  virtual sound device (ENGINE-55) and the developer surfaces.
- **First table** (this container): 38 of 39 steps pass their release budgets. **`feel_battle` `battle_block` misses**: the
  click on `strip.block1` takes 51 to 74 ms in its handler against `input_ms` 50 (4 runs); develop before this change
  measured 50 and 53 ms (one pass, one fail), so it sits on the line already. The gate's dev allowance (125 ms) hides it.
  A follow-up row could find what the block click does on the frame; not done here. Next highest: `switch_feel` menus
  and fight to results switches 62 to 65 ms (budget 150), `feel_battle` plan input 50.4 ms (budget 100), the
  `audio_feel` Continue input 59 ms (no input budget).
- **Docs delta**: none for the shared READMEs; `tools/fc.sh feel` is in `fc.sh`'s help.

### FB-505: ENGINE-56 AUDIO-FEEL2 built; menus answer by sound, map footsteps, battle music in 0.4 s, stings warmed

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui` screen, shell input and link, audio), game GUI
  (`emberwake_gui` battle, settings, region map, map view, defaults, tuning), kit asset `kits/rpg/assets/audio/sfx/
  map_step.wav` (placeholder), game `tuning.json` (`battle_music_fade_ms` 400), scenario `audio_feel`; no catalog data.
  **Every golden unchanged**. Shots diff: `screens_menus__settings_sound.png` (the new Footsteps row, intended) and
  `screens_menus__scene_runner.png` (its known artifact).
- **Spec**: `docs/READY.md` row `ENGINE-56` (`AUDIO-FEEL2`).
- **(4) Menu sounds by rule** (engine): the shell answers a press with `ui_confirm` when a button is pressed and taken
  (a click, Enter or Space on the focus, the agent link), `ui_error` on a disabled one, and `ui_back` when Esc, the pad's
  B or a right click closed a screen or an open choice list (a back that closes nothing is silent). Buttons are the kinds
  in `UI_SOUND_KINDS` (button, text button, icon button, tab, checkbox, choice field and option, section); sliders, rows,
  cards and board cells say nothing. A screen overrides per widget with `Screen::ui_sound` (the rule, none, or its own
  file): the battle's round controls keep their `ui_click` (`Action::clicks`), the Settings volumes' `<` and `>` keep
  their sample like the sliders. The files are the engine's (`engine/assets/audio/sfx`); a game's of the same name plays
  over them.
- **(3) Map footstep** (game): a soft step on each step of the party on the hub and region maps (`mapview::MAP_STEP`,
  Effects bus), at a new `Footsteps` volume under Effects (settings `footsteps`, default 60, a share of Effects; its row
  sits under Effects Volume on the Sound tab and previews the step). The file is a placeholder at its final path: the
  kit's `move_step.wav` low-passed at 45 % (Art/UI replaces it and adds its line to the kit's sfx README).
- **(2) Battle music fade**: a screen may name its music's fade-in (`Screen::music_fade_in`, `Audio::music_faded`); the
  battle names `battle_music_fade_ms` (tuning, 400). I read "from silence" as the battle track's own fade-in wherever it
  starts: The Sweep's track starts over the map's, so it now comes in over 0.4 s while the map's fades out over 1 s
  as before. Other tracks keep 2 s from silence and 1 s over another.
- **(5) Stings warmed**: a world battle screen decodes `combat_sting` and `victory_stinger` on a thread as it is made
  (`Audio::warm`; the map makes it ahead, unseen); the first combat sting in `switch_feel_story` played from the cache
  (0.08 ms on the ask, heard 24 ms after it).
- **Measured** (release, no window, 3 runs of `audio_feel`):
  - **Confirm**: 13 to 22 ms after the click (Continue, the game menu, its party screen); **back**: 16 to 21 ms after Esc.
  - **Map footstep**: within 3.5 to 5.5 ms of its frame; the first step of a key press is asked between frames and
    heard 21 ms after the key.
  - Round controls, hits and music as in FB-504.
  - Underruns: none in three runs. One release run had a single underrun in `battle_round` with a 6 ms buffer mix
    (the virtual device's thread preempted on this container); not seen again in three more.
- **Scenarios**: `audio_feel` gains `menu_confirm_back` (interface sounds 50 ms) and `map_step` (footsteps 16.7 ms off
  their frame), and `title_to_map` budgets Continue's confirm; README row.
- **Tests**: the shell's rule (confirm by click, Enter and the link, error on a disabled button, back only when
  something closed, nothing on a slider or where the screen plays its own); `warm`, `music_faded` and `play_gain`.
- **Found, not mine**: `switch_feel_story` fails in a release run at its step 4 (New Game opens a vignette, not the new
  game screen) on develop before this change too; it passes in the dev build the gate runs.
- **Docs delta (engine/README.md)**: in the audio bullet add "; since ENGINE-56 the shell answers presses by sound
  (`ui_confirm`, `ui_back`, `ui_error`; `Screen::ui_sound`, `UI_SOUND_KINDS`), a screen may name its music's fade-in
  (`Screen::music_fade_in`), `Audio::warm` decodes a sound ahead and `Audio::play_gain` plays under a bus's level".

### FB-504: ENGINE-55 AUDIO-FEEL1 built; interface sounds 12 to 20 ms, hits within 6 ms of their frame, no underruns

- **Status**: built (tools lane), `develop`. Engine only (`aegis_ui` `audio_out` and `audiofeel` new, `audio`, shell,
  headless feel replay, start; `aegis_cli` scenario feel budgets), scenarios `audio_feel` (new), `switch_feel` and
  `walk_feel_fight` (`underruns` 0); no data, no game code. **Every golden unchanged**. Shots
  diff: only `screens_menus__scene_runner.png` (its known artifact); `audio_feel` had one underrun in Continue's 1.5 s
  inside the diff's loaded run, so the dev allowance now counts a part of 2 s as one. One gate run failed `feel_battle`'s
  `battle_resolve` on `shotsexact`'s second run (one 97 ms frame, median 32 ms, no sound decode in it); it passed
  three times alone (worst 45 to 48 ms) and in the next full gate.
- **Spec**: `docs/READY.md` row `ENGINE-55` (`AUDIO-FEEL1`).
- **What `feel` now reports**: every `feel` reply has `audio`: each sound and track started, with when it was asked, when
  its first sample over 1/1024 was heard (a probe on the mixer's output), the time from its input (`after_input_ms`),
  from its ask (`after_ask_ms`) and, for an effect asked in a frame, from when that frame is shown (`beat_ms`), and the
  decode done on the ask; plus the output's buffer, underruns (a buffer mixed after it should have played) and the
  longest mix. With no window the sound plays into a virtual 48 kHz output on its own thread that asks for 512-frame
  buffers on time, so muted gate runs hear it too. A window's feel overlay adds the real device's buffer and underruns.
- **Measured** (release, no window, this 4-core container; before the fixes in brackets):
  - **UI click or confirm**: 12 to 20 ms after the click (under 50 ms).
  - **Hits and footsteps** (battle hits, `move_step`, defends): within 4.9 to 5.6 ms of their frame (one 11 ms seen);
    one frame is 16.7 ms.
  - **Music change**: 15 to 30 ms after its ask (Continue's map, the map to The Sweep, results to the map) [Continue's
    music 96 ms after the click: it was asked after the map's first draw].
  - **Decode on the frame**: about 1 ms for a track (the bytes read) [3.5 ms of decoder]; an effect 0.1 to 0.3 ms
    once cached [the Ogg stings 10 to 17 ms on their first play if decoded on the frame].
  - **Underruns**: none in release in `walk_feel_fight`, `switch_feel` or `audio_feel`; the longest buffer mix 2.5 ms
    against its 10.7 ms. In the gate's dev build one underrun shows now and then in a long replay (`switch_feel`'s menus,
    fight to results, and once in Continue's 1.5 s under the shots diff's load): a debug thread starved by the build's
    own load, not the mix.
- **Fixes**:
  - A screen's music is asked for right after its tick, before its draw (`Shell::follow_music`), not after it.
  - A track's decoder is made on a thread (`aegis-track`); an effect is decoded once and kept (a `.wav` on its first
    ask, any other format streamed on a thread while a copy is decoded for the next play).
  - The device is opened with our own `cpal` stream at a fixed 512-frame buffer (10.7 ms at 48 kHz) where the device
    allows it, else its default; the old default stream's buffer was unknown. No mixing work was on the audio thread
    beyond the mix itself.
- **Gate budgets**: a `feel` step's `budget` takes `ui_sound_ms`, `hit_beat_ms`, `music_ms`, `underruns` and `decode_ms`;
  one over, or one never measured, fails the step. In a dev build `decode_ms` counts 2.5 times and `underruns` allows one
  per 2 s of replay or part (`DEV_UNDERRUN_EVERY_MS`). `audio_feel`: music 50 ms, the confirm 50 ms, hits 16.7 ms, underruns 0;
  `switch_feel` and `walk_feel_fight`: underruns 0. Authoring guide `engine/docs/authoring/scenarios.md` "Sound".
- **Mix and feel choices for Design** (none changed here):
  1. Buffer 512 frames (10.7 ms) vs 256 (5.3 ms): 256 halves the wait, costs more wakeups and risks underruns on slow
     PCs. Recommend 512.
  2. Fades: a track from silence fades in over 2 s, a change crossfades over 1 s. The first notes of a short sting or a
     battle start are quiet for a moment; a shorter fade-in (0.3 to 0.5 s) for battle music may feel punchier.
  3. No footsteps on the map: `move_step` plays in battle only. A soft map step would help the walk feel.
  4. No menu or button sounds: the engine ships `ui_click`, `ui_confirm`, `ui_back` and `ui_error`, but only the battle's
     round controls and the volume sliders play them. A rule like "confirm on any button, back on Esc" would make menus
     feel answered.
  5. Stings (`combat_sting`, `victory_stinger`) now decode on a thread on their first play, so the first one is heard
     a frame or two later than a cached one; warming them with the screen would remove that.
  6. The dev-build underrun allowance (one per 2 s) vs none: release holds 0.
  7. On Windows a WASAPI device may refuse the fixed buffer and fall back to its default; the overlay shows the real
     buffer when Tushar plays with `feel=1`.
- **Docs delta (engine/README.md)**: in the audio bullet add "; since ENGINE-55 the output is our own `cpal` stream at a
  fixed 512-frame buffer (`aegis_ui::audio_out`; a virtual device with no window, so feel replays hear sound), effects
  decoded once and off the frame, and `feel` reports each sound from its input or frame to its first audible sample
  (`aegis_ui::audiofeel`, budgets `ui_sound_ms`, `hit_beat_ms`, `music_ms`, `underruns`, `decode_ms`)".

### FB-503: ENGINE-54 WARM-SPREAD1 built; no frame over 33 ms while warming, Continue 19 ms and The Sweep 21 ms

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui` art, ground composer, canvas and painter, shell), kit
  (`rpgkit_battle` plan, `rpgkit_ui` composed board), game GUI (`emberwake_gui` title, region map, story stage, battle
  start, map ground), scenarios `walk_feel_fight` (new) and `switch_feel`; no data. **Every golden unchanged**. Shots
  diff: only `screens_menus__scene_runner.png` (its known artifact) and the new scenario's shots; `feel_battle` failed
  once inside the diff's loaded dev run (a 105 ms frame) and passed twice alone.
- **Spec**: `docs/READY.md` row `ENGINE-54` (`WARM-SPREAD1`).
- **Build-ahead work on threads**:
  - Decodes: in a warm draw (`Art::defer`) a file not decoded yet is asked for and decoded on a thread (up to
    `DECODE_THREADS` at once); `Art::warm_poll` takes the finished ones in. Nothing built from a file left for later is
    cached (each cache checks `Art::left`).
  - Grounds: in a warm draw (`ground::set_deferring`) a ground not composed yet is composed on a thread
    (`ground::compose_later`, `ground::poll_later`): the map's ground and the fight's board ground. The map ground's
    ENGINE-53 path of its own (`compose_ahead`) is gone: the title's unseen map draw composes it the same way.
  - Fight starts: a fight the party walks toward or stands near, and the fight after a vignette, start on a thread
    (`battle::FightAhead::spawn`; the kit's round plan is now `Send`: `Arc` and atomics for its `Rc` and `Cell`s,
    same values).
- **The rest in slices**: the screen `warm_next` gives is drawn unseen one slice a frame (`Shell::warm_slice`): nothing
  can be drawn (an empty canvas `bounds`, which the screen's own clips cannot lift; shapes and glyph blits skip their
  pixels, a cached or scrolled layer is drawn whole for what it asks and keeps nothing), art made on the frame limited
  to 2 ms a slice (`WARM_ART_US`), the screen made on one slice and drawn from the next. A slice that left something
  waits for the threads it started before the next one; one that left nothing ends the warming.
- **When**: the title 300 ms after it shows (was its third frame); the map after the party has stood still 300 ms on a
  tile (was 10 frames; a fight already drawn unseen on this map is only started ahead, not drawn again); a vignette
  300 ms in. A frame after an input has no slice, and nothing waits on one.
- **Measured** (release, no window, 1280 x 720, this 4-core container):
  - **No frame over 33 ms while warming**: `walk_feel_fight` (new, cold art by The Sweep's marker, click walk
    included) worst 22 ms, median 5.3 ms; `walk_feel` worst 21 ms (click) and 17 ms (keys); the title while it makes
    Continue's map (`switch_feel`'s new `title_warm`) worst 19 ms, median 1 ms.
  - **Slices**: the fight's unseen draw once its art is in 3 to 4 ms; **two slices are over 8 ms**: the first draw on
    cold art, 16 to 18 ms (the battle's board asks for about 200 files), and the next, 9 to 14 ms (the bodies made from
    them). That work is the board's own first draw, not art the 2 ms limit counts; the frames stay under 33 ms
    with it. Splitting it further would need the battle's draw to stop part way; not done (a follow-up if wanted).
  - **Continue 18.6 to 21.5 ms; The Sweep 21 to 22 ms** (own part, both under 60 ms); in one `switch_feel` run of
    four The Sweep took 98 ms (two frames, worst 47 ms; under its 120 ms budget, not reproduced in three more runs).
- **Scenarios**: new `walk_feel_fight` (budgets of `walk_feel`); `switch_feel` gains `title_warm` (frames 33 ms).
  Scenarios README rows.
- **Docs delta (engine/README.md)**: in the screens / shell bullet, after the ENGINE-53 `Screen::warm_next` clause add
  "; since ENGINE-54 drawn in slices of one a frame, unseen behind an empty canvas `bounds`, once the screen has been
  still 300 ms and never on a frame with input: decodes (`Art::defer`, `Art::warm_poll`) and grounds
  (`ground::compose_later`) go to threads, and a slice that left something waits for them".

### FB-501: ENGINE-53 SWITCH-AHEAD1 built; Continue 19 to 22 ms and The Sweep 27 to 30 ms, both under 60 ms

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui` ground composer, `Screen::warm_next`, the shell), game
  GUI (`emberwake_gui` title, region map, story stage, battle start, map ground), the `switch_feel` scenario; no data.
  **Every golden unchanged** (the composer's pinned fixtures too). Shots diff: clean (only the known `screens_menus__scene_runner.png` artifact).
- **Spec**: `docs/READY.md` row `ENGINE-53` (`SWITCH-AHEAD1`), options (b) and (c).
- **(b) The composer's serial parts on threads**: the fields' blur (`grounds()` and the other fields, new `par_map`: one
  field a thread) and the sunken rims and contact lines (bands of rows, `by_rows`) run on up to `COMPOSE_THREADS`; each
  writes only its own field or row from what it reads, so the bytes are the one pass's (the ENGINE-52 tests, three
  grounds and two shipped maps on 1 / 2 / 4 threads, pass unchanged). The stone rim and the dressing (groups, placed
  patches, marks) stay on one thread: they read what their neighbours wrote. (Not timed apart: the switches below no longer compose on their frame.)
- **(c) Built ahead at idle**:
  - New engine hook `Screen::warm_next`: a screen may hand the shell a screen it may open next; the shell draws it once
    unseen at the start of a frame (after the tick), as it draws the warm list before the first frame (ENGINE-46), and
    drops what it asked for (navigation, notices). Asked of the top screen, then down the stack until one gives one.
  - Ground composed ahead: `ground::keep(key, img)` puts an image composed elsewhere in the composer's cache
    (`ground::held` tells); a stale one is never drawn, its key matching nothing.
  - **Continue**: on the title's third tick it reads the newest save (not entering it), starts its map ground on a
    thread (`emberwake_gui::ground::compose_ahead`, the same key and pixels as the map's own: test
    `a_ground_composed_ahead_is_the_one_the_map_draws`), keeps it when done, then draws the map once unseen
    (`region::Rehearsal`, on a game of its own swapped in for that draw; the log given back). Continue then finds every
    cache full. The title's `state.ahead` reads `waiting`, `composing`, `drawing` or `ready`. Continue also reads the
    newest slot the menu already found instead of parsing every slot again.
  - **A fight on the map**: the game keeps one `battle::FightAhead`: a world fight started on a copy of the game (the
    copy before and after, and what the start built). During a click walk, once per walk, its steps are tried on a copy
    (`explore::step`) and the fight they end in is started ahead; standing still 10 frames on a tile, the walk to the
    nearest encounter marker is tried the same way and the fight is also drawn once unseen (bodies, board ground, panels).
    `BattleScreen::world` takes it when the game, the fight and the runner are exactly those it was started from, and
    otherwise starts the fight as before (stale: dropped).
  - **A scripted fight**: while a story vignette shows (10 frames in, once a step), a fight step right after it is
    started ahead on a copy with the vignette's step done, and drawn once unseen.
  - Test (as asked): `transitions_tests::a_fight_built_ahead_equals_one_built_on_the_switch`: the same battle state, game
    and log after the start, the game untouched until taken, another runner or a changed game stale and started anew.
- **Measured** (release, no window, 1280 x 720, this 4-core container; `own_ms`):
  - **Title Continue to the map: 19 to 22 ms** (was 107 to 124; its first frame 17 to 18 ms).
  - **The Sweep from the map, no intro: 27 to 30 ms** (was 101 to 135; the fight's start 0.2 ms, its first draw 15 ms).
  - Story card to the stage to the fight (with its beat), dev build: 50 ms (was 94 in release).
  - No release first frame over 100 ms in `switch_feel`; every other switch as before (within noise).
  - **The cost moved to idle** (not a switch, listed as asked): the title draws the map once unseen in one frame of about
    50 ms after its ground is kept (about 0.3 s after it opens); a map draws a fight once unseen in one frame of about
    100 to 140 ms the first time the party stands still 10 frames near an encounter (cold art: decoding its bodies, its
    board ground, its panels; about 20 ms once the art is warm); a vignette the same once, 10 frames in. A click walk
    toward a fight adds about 12 to 20 ms to one walk frame (the fight started on the copy). None is a cue, and `walk_feel`
    stays within its budgets.
- **Budgets**: `switch_feel` waits for the title's `state.ahead` to read `ready` before Continue, and its Continue and
  The Sweep inputs go 200 -> 120 ms (300 in the gate's dev build).
- **Docs**: the scenarios README row. **Docs delta (engine/README.md)**: (1) in the ground composer's bullet, after the
  ENGINE-52 clause add "; its fields' blur, sunken rims and contact lines too, and `ground::keep` takes an image composed
  ahead (ENGINE-53)"; (2) in the screens / shell bullet add "`Screen::warm_next` hands the shell a screen to draw once
  unseen at the start of a frame, so a screen's first draw finds its caches full (ENGINE-53: the title's Continue map, a
  fight near the party, the fight after a vignette)".

### FB-499: ENGINE-52 COMPOSE-PAR1 built; the ground composer runs its rows on threads, same pixels; both switches miss 60 ms

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui` ground composer), kit (`rpgkit_ui` stage
  `compose_board`, for tests), game tests and the `switch_feel` budgets; no data. **Every golden unchanged** (the
  composer's pinned fixtures too). Shots diff: 128 unchanged, 1 changed
  (`screens_menus__scene_runner.png`, the known artifact); in that run `switch_feel` itself failed once (the party
  popup 411 ms against 375 in the dev build while the diff ran two builds' scenarios side by side; alone it is 110 to
  142 ms), so its budgets are load-sensitive on a shared machine.
- **Spec**: `docs/READY.md` row `ENGINE-52` (`COMPOSE-PAR1`).
- **Built**: the per-pixel pass and every noise pass (`add_noise`, so the patchy blobs too) run in bands of whole rows
  on up to `COMPOSE_THREADS` (4) threads (`ground::by_rows`; one on the web, one for a job under `PAR_MIN_PX` 16 K px a
  thread). Each pixel or noise row reads the shared fields and writes only itself, so the result is the one pass's
  bytes. `ground::with_compose_threads(n, ..)` caps it (tests). Tests: `aegis_ui` ground
  `a_job_is_the_same_on_one_thread_and_many` (three grounds with textures, wear, strays, tone, patchy and patches, square
  and tilted, 1 / 2 / 4 threads); `emberwake_gui` `a_maps_ground_is_the_same_on_one_thread_and_many` (Warden Keep and
  Junction 7, both views) and `a_battle_boards_ground_is_the_same_on_one_thread_and_many` (two Sunk Ward fights and the
  Ashlands courtyard, through the new kit `stage::compose_board`).
- **Measured** (release, no window, 1280 x 720, on this 4-core container):
  - The composer alone (Junction 7, 896 x 576 art px): 40 -> 26 ms (noise 12.8 -> 7, the per-pixel pass 14 -> 7; the
    fields' blur 3, the rims 2.5 and the dressing 5 to 7 stay on one thread). In the game the same compose took 42 to
    50 ms on 4 threads against 45 to 55 on one: the threads share the cores with the rest of the run.
  - **Continue to the region map: 107 to 124 ms to settled** (first frame 90 to 107 ms; about 45 of it the ground,
    about 40 the rest of the map's first draw: the tiles, walls, markers, legend and HUD). **Misses 60 ms.**
  - **The Sweep from the map, no intro: 101 to 135 ms** (own part; building the fight in the tick about 25 to 35 ms,
    its first draw 45 to 50). **Misses 60 ms.**
  - As asked, no cue added. Neither reaches 60 ms without moving most of a map's or fight's first draw off the input
    frame; listed below.
- **Still layers on the input frame**: not done. The region map's ground job is made in its first draw (`MapGround::sync`)
  and the ground is the first thing that draw paints, so starting it earlier on a thread overlaps only the 5 ms before
  it. A real gain needs the job made on the input (Continue's handler, a fight's start) and the map's other first-draw
  work moved beside it; that is a restructure of the region map's draw, not this row.
- **What would reach 60 ms** (for a Design call): (a) the region map's first frame drawn in two: the ground composed on a
  thread from the input while a plain floor shows one frame (a visible step, so a cue by another name); (b) the composer's
  remaining serial parts (fields, rims, dressing) on threads: about 10 ms more; (c) a fight built ahead while its marker
  is approached (the 25 to 35 ms tick). Each is a row of its own.
- **Budgets**: `switch_feel`'s Continue and The Sweep inputs 400 -> 200 ms (`own_ms`; 500 in the gate's dev build).
  Release runs of Continue occasionally show its first frame just over 100 ms (101 to 107), which a release run of
  `switch_feel` counts as an uncovered frame; the gate's dev run allows 250.
- **Fix found by the gate**: the switch probe counted a frame as settled under 33 ms in a dev build too, where a
  world map's frames take 34 to 54 ms, so the gate's second run of `switch_feel` read the world map as settling 1.5 s
  late. A switch now settles on a frame under 33 ms times the dev allowance (82.5 ms) in a dev build
  (`Switch::settled_frame_ms`); test `a_dev_build_settles_on_its_stretched_frame`; `scenarios.md` says so.
- **Docs**: the scenarios README row. **Docs delta (engine/README.md)**: in the ground composer's bullet add "its
  per-pixel pass and noise run in bands of rows on up to 4 threads, the same bytes (ENGINE-52,
  `ground::with_compose_threads`)".

### FB-497: ENGINE-51 SWITCH-FEEL1 built; every screen switch timed and within its target, no uncovered frame over 100 ms

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui` the new `switchfeel`, `Screen::transition`, `Art`
  read-ahead, the ground composer's noise; `aegis_cli` the `feel` step), game GUI (`emberwake_gui` battle, results,
  region map, world map, the warm list) and two new gate scenarios; no data. **Every golden unchanged**. Shots diff:
  128 unchanged, 1 changed (`screens_menus__scene_runner.png`, the known artifact).
- **Spec**: `docs/READY.md` row `ENGINE-51` (`SWITCH-FEEL1`).
- **The probe**: a `feel` step with `switch` true follows each key, click or pad input frame by frame until its new screen
  settles (`aegis_ui::switchfeel`): `first_ms` (input to the first frame of another screen, or the same at another stack
  depth, so a popup counts), `settled_ms` (to the first frame of the last screen it came to drawn in 33 ms: the loads are
  done; a story card that hands over to a stage and a fight is one chain), `own_ms` (from the old screen's last frame: a
  walk to an exit or the results' fade out is play, not loading), `long_frames` over 100 ms with the transition shown
  before each (new `Screen::transition`: the battle beat and the results' fade out report theirs). A script item's
  `switch_ms` fails the step when `own_ms` passes it, `to` names the screen it should reach (so a fight that opens on its
  own after the results is not counted in it), and `budget.uncovered_frames` limits long frames nothing covered. Tests:
  `aegis_ui` switchfeel (6), `aegis_cli` `a_feel_step_takes_switch_budgets`.
- **Measured** (release build, no window, 1280 x 720; own ms, worst frame ms; before -> after where a fix moved it):

  | Switch | Own | Worst frame |
  | --- | --- | --- |
  | Title Continue to the map | 85 to 107 | 84 to 90 |
  | Party screen, hub menu, its party, Settings, back | 17 to 62 | 46 |
  | Hub Save | 90 to 124 -> 17 | 107 -> 11 |
  | Title Settings, Load, New Game | 17 to 79 -> 17 | 62 -> 13 |
  | Region map to world map (after the walk) | 90 to 123 -> 25 | 102 -> 25 |
  | World map Enter to the map | 19 | 11 |
  | Region map to a fight, no intro (after the walk) | 113 to 146 -> 119 to 128 | 97 to 128 -> 75 |
  | Story card to stage to fight, with its beat | 81 to 119 -> 94 | 86 -> 61 |
  | Fight to results | 31 to 69 | 50 |
  | Results to the map (after the 500 ms fade out) | 17 to 20 | 36 |
  | Results to stage to the next story card | 42 | 26 |

  Every switch is under its target (menus and popups 150 ms, maps and fights 400 ms) and no frame over 100 ms is left
  uncovered in the release runs.
- **Fixes** (clear misses):
  - (1) Duplicated work: every body sheet and icon was decoded twice (once to read its clips or size, once to draw).
    `Art` keeps a small file's decoded pixels (`RAW_KEEP_PX`, a megapixel) so the second reads them.
  - (2) A fight's bodies at once: new engine `Art::load_bodies` decodes the sheets a screen opens on side by side
    (`decode_all`, up to `DECODE_THREADS` 4; one on the web); a world fight loads its units' bodies before its first
    frame (`battle/playback.rs` `load_bodies`). The fight's first frame 102 to 128 -> about 75 to 90 ms.
  - (3) Earlier, by warm: the Save and Load slots join the start's warm list (`Context::warm`, ENGINE-46): their slot
    panels' frames were a 50 ms first draw, a 107 ms frame opening Save.
  - (4) Earlier, by idle loading: new engine `Art::want_fit` starts reading a big picture on a thread for a screen that
    may open next (nothing on the web); the region map asks for the world map's background after its first frame
    (`worldmap::want_background`), so the world map opens in 25 ms instead of reading 36 ms and scaling 13 ms.
  - (5) The ground composer's four noise fields are made side by side (`ground::noise_fields`, the same values): the
    region map's ground compose 64 -> 43 ms, its noise 37 -> 16 ms.
- **Needs a loading cue or a later row (nothing over target now, listed as asked)**:
  - The title's Continue: the region map's first frame is 84 to 90 ms, of which composing its ground about 43 ms (its
    per-pixel pass 13, the noise 16) and its still layers' first full draw. Close to 100 ms on a slower machine. A cue
    (a short fade from the title) or a row parallelising the composer's per-pixel pass by rows (pure, so the same pixels)
    would hold it.
  - The region map to a fight with no intro: the fight's first frame 75 to 90 ms (building the fight about 25 ms in the
    tick, its stage ground about 11 ms). A fight that has the battle beat is covered by it; one without has nothing
    covering it. A cue is the same call as above.
  - The character popup (46 ms) and the results (50 ms) spend about 30 ms composing their framed panels on first open;
    within target, could join the warm list when they need no game.
- **Gate scenarios**: `switch_feel` (the save `saves/sweep_start`: Continue, the party screen, the hub menu, Save,
  Settings, the world map and back, The Sweep with no intro, results, back to the map) and `switch_feel_story` (a new
  game: Settings, Load, New Game, the story cards into the first fight and its beat, results, the next card), each input
  with `switch_ms` 150 (menus, popups) or 400 (maps, fights) and `uncovered_frames` 0; 2.5 times in the gate's dev build.
  Gate time about 80 s and 45 s.
- **Docs**: `engine/docs/authoring/scenarios.md` ("Input times": screen switches), the scenarios README rows.
- **Docs delta (engine/README.md, Feel probe bullet)**: after the ENGINE-50 clause add "; a `feel` step with `switch`
  times each screen switch to its first settled frame (ENGINE-51: `aegis_ui::switchfeel`, `Screen::transition` names a
  covering fade or beat; `switch_ms`, `to` and `uncovered_frames` budgets), and `Art::load_bodies` (sheets decoded side by
  side) and `Art::want_fit` (a picture read ahead on a thread) move loads off a switch's first frame".

### FB-495: ENGINE-50 MAP-FRAME1 built; the region map's frame cut to a third, a key press after a hitch takes no catch-up step

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui` painter and the headless `feel` replay; `aegis_cli` the
  `feel` step), game GUI (`emberwake_gui` region map) and the `walk_feel` scenario; no data. **Every golden unchanged**.
  Shots diff against `8277b246`: 128 unchanged, 1 changed (`screens_menus__scene_runner.png`, the known artifact).
- **Spec**: `docs/READY.md` row `ENGINE-50` (`MAP-FRAME1`).
- **Where the frame went** (release build, no window, 1280 x 720, the hub walked by `walk_feel`): about 18 ms a frame,
  of which the map's still layers (ground, tiles, walls, border, dressing, shadows, ground props) about 3.5 ms, the
  markers' pass about 5 ms and the "Here" legend about 5.5 ms; the last two asked `explore::marker_view` and
  `explore::npc_services` (quest scans) for every marker, twice a frame.
- **(1) Still layers drawn once**: a map's camera moves every frame while walking, so `Painter::cached` (keyed on the
  view) would redraw every frame. New engine `Painter::scroll_begin` / `scroll_end` with `Strip`: a layer keyed as
  `cached` plus an origin in whole physical px; the pixels kept from the last frame are put back moved by the origin's
  change and only the strips newly in view are left to draw (a key or view change draws it all). The region map
  (`RegionMap::floor`, the hub too) draws its still layers into those strips under their clip; `floor_key` holds the
  data, art generation, theme, location, view, zoom, tile sets, composed ground and the props shown (a hub's bunk states
  among them). Test: `aegis_ui` painter `a_scrolled_layer_draws_only_what_came_into_view` (at 1x and 2x: only the new
  columns and rows drawn, the same pixels as a full draw, unmoved all put back, a new key or a jump past it draws all).
- **(1b) Marker views per state**: `MarkerMemo` keeps each marker's view and services for one game state and data (all
  they read; compared each frame, so a step or any change reads them again). Same results, read once instead of twice a
  frame.
- **Frame times** (release, no window; median / p95 / worst ms):

  | Run | Before | After |
  | --- | --- | --- |
  | Hub, click walk | 18.0 / 26.7 / 29.0 | 6.8 / 12.6 / 19.0 |
  | Hub, keys and d-pad | 17.8 / 22.2 / 50.8 | 6.7 / 10.2 / 15.6 |
  | Crossroads Cages map, keys | 11.9 / 16.4 / 86.2 | 5.4 / 11.8 / 79.5 |

  The Cages run's worst frame is the map's first draw inside the replay (a cold start: the ground composed and the art
  read; 51 ms of it the first full draw of the still layers), not a walking frame. The gate's dev build on the hub:
  median 30 to 16.4 ms, worst 54 to 25 ms. Results in `/mnt/project-files/build-tools/map_frame/`.
- **Budgets**: the `feel` step's `budget` takes `frame_p50_ms` (the median frame; 2.5 times in a dev build like the
  others). `walk_feel` now holds `frame_p50_ms` 16.7 and `frame_ms` 33 on both its runs.
- **(2) A key press after a long frame**: a press stepped as of the last frame however long ago it was; after a hitch its
  glide jumped ahead and the held walk took a quick step to catch up (66 ms after the step, not a beat). Now the next
  tick dates it no more than `CLICK_STEP_BACK` (25 ms) back, as a click (`RegionMap::press_at`), and the next step comes
  a beat after that. Tests: `emberwake_gui` `a_key_press_after_a_long_frame_is_dated_now` (17, 25 and 250 ms after the
  frame); `walk_feel` presses during a 250 ms hitch through the new `feel` input `stall_ms` (the clock moves with no frame
  drawn, so the input arrives in the hitch): the first move is on the next frame and the beat stays even (without the
  fix the step fails with `cadence_spread_ms 66.67 over 17`). The held walk's own on-schedule steps (`step_time`) are
  unchanged: bounding those would slow walking at 30 frames a second.
- **Docs**: `engine/docs/authoring/scenarios.md` ("Input times": `frame_p50_ms`, `stall_ms`), the scenarios README row.
- **Docs delta (engine/README.md, Feel probe bullet)**: after "`Painter::cached` keeps a layer drawn once (ENGINE-46)"
  add ", and `Painter::scroll_begin` / `scroll_end` keep one that scrolls (ENGINE-50: the kept pixels move with the
  layer's origin and only the strips newly in view are drawn)"; after "a `feel` budget counts 2.5 times in a dev build"
  add ", `frame_p50_ms` holds the median frame".
- **Free-port flake**: not seen again (this row's gate passed first time).

### FB-493: ENGINE-49 WALK-FEEL1 built; walking measured on two maps, every target met after one fix, feel choices listed

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui`: new `walkfeel`, `Screen::motion`, the headless `feel`
  replay; `aegis_cli`: the `feel` step), game GUI (`emberwake_gui` region map) and one game scenario; no data. **Every
  golden unchanged**. Shots diff: see the last line.
- **Spec**: `docs/READY.md` row `ENGINE-49` (`WALK-FEEL1`).
- **What landed (the measure)**: the `feel` step takes held keys and d-pad directions (`press`, `release`, `pad_press`,
  `pad_release`, as a window's key down and up), `wait_ms`, `spike_ms` (one long frame), `step_ms` (the shot clock's
  frame step: 16.667 for 60 frames a second) and `motion` (the top screen's `Screen::motion` read after every frame:
  the mover's cell, where it is drawn, the camera and where it wants to be). `aegis_ui::walkfeel::report` reads first move,
  cadence, overshoot, turns, long frames and camera lag and jumps; movement budgets fail the step. Documented in
  `engine/docs/authoring/scenarios.md` ("Input times"). Tests: `aegis_ui` `walkfeel` (4 unit tests); `aegis_cli`
  `tests/scenario.rs` `a_feel_step_takes_movement_inputs_and_budgets`.
- **Gate**: `games/emberwake/scenarios/walk_feel.json` on the Sluice Undercroft (the hub, save `hub_start`), with budgets
  first move 17 ms, overshoot 0, cadence spread 17 ms, stutters 0, camera lag 1 tile, camera jump 0.5 tile a frame.
- **Report** (release build, no window, 60 frames a second on the shot clock, so the step times are exact; the hub and
  the Crossroads Cages map from `cages_start`; scenarios, every `feel_*.json` and shots in
  `/mnt/project-files/build-tools/walk_feel/`):

  | Measure | Target | Hub | Cages map |
  | --- | --- | --- | --- |
  | (a) Press to first visible move: keys, d-pad | next frame | 16.7 ms (1 frame) | 16.7 ms |
  | (a) Same, a tap (press and let go) | next frame | 16.7 ms | 16.7 ms |
  | (b) Beat while held | even | 200 ms, spread one frame (183 to 200) | same |
  | (b) Across a 250 ms frame | even | late step at 250 ms, then back on the beat (383, 583) | same |
  | (c) Steps after release | 0 | 0 in every release | 0 |
  | (d) Turn mid-walk (keys, d-pad) | answers, no drop | steps at once (16.7 ms); none dropped | same |
  | (d) Two keys at once (up, then left a frame later) | - | up steps, left steps a frame later; letting left go resumes up on the beat | same |
  | (e) Click to walk: first move | next frame | was 33 ms (2 frames), **now 16.7 ms** | (keys only) |
  | (e) A second click mid-walk | no stall | the walk turns at the next step (83 ms, the step in progress), on the beat | - |
  | (f) Camera lag | within a tile | 0 tiles (locked on the party) | 0 tiles |
  | (f) Camera jump in a frame | none | 0.09 tile a frame at most, at the edges too | 0.13 tile |
  | Input to drawn frame (feel probe) | - | 19 to 22 ms | 12 to 14 ms |
  | Frames over 33 ms | - | 1 a run (p50 18.6 ms, worst 39 ms) | 1 (p50 11.8 ms, worst 80 ms) |

- **Fix (no design call)**: **click to walk showed its first move a frame late**. A click walk took its first step on the
  next frame's tick, dated at that frame, so that frame drew the party not yet moving; a key press steps dated at the
  frame it came after. Now a walk ordered while the party stands takes its first step as of the frame the click came
  after (`RegionMap::tick_at`), when that frame is at most `CLICK_STEP_BACK` (25 ms) back; after a longer frame it is
  dated now as before (a first try with no bound dated it far back after a long frame and the walk ran to catch up: the
  shots diff caught it in `sweep_telegraph`). Test: `emberwake_gui`
  `region::sprite_tests::a_click_walk_moves_on_the_next_frame` (fails without the fix; also covers the long frame).
  A key press is dated at the last frame with no such bound (`RegionMap::key`); after a long hitch a held walk could take
  one quick catch-up step. Not changed here (the walk tests run on their own clock there); a candidate if the PC shows it.
- **For Design (feel choices, nothing changed)**:
  1. **Walk speed**: one tile every 200 ms (5 tiles a second); Shift runs (WALKRUN1). Keep, or another pace?
  2. **No acceleration**: full speed from the first step and a dead stop on release (no ease in or out of a walk).
     Keep, or a short ease?
  3. **A turn steps at once**: a new direction pressed mid-walk steps on the next frame, not after the step in progress,
     so a turn makes a short step (33 ms after the last one); two keys pressed together step twice within a frame or
     two. Keep (snappy), or queue the turn to the end of the step in progress (even beat)?
  4. **Hold to run**: running is Shift held; the pad has no run button. Give the pad one (a shoulder or a trigger), or a
     run after holding a direction a while?
  5. **Four directions only**: two keys held walk the latest pressed, no diagonal steps.
  6. **The camera is locked on the party** (no follow lag or look-ahead) and stops at the map's edges. Keep, or a soft
     follow?
- **Not in this row**: the hub's frames run about 18.6 ms at p50 in this container's software draw (over 16.7 ms for 60
  a second); one frame a run over 33 ms on each map. A frame-time row could follow if the PC shows the same.
- **Docs delta**: `engine/README.md` "No window (HEADLESS1)": the `feel` link request also takes `press` / `release` /
  `pad_press` / `pad_release` / `wait_ms` / `spike_ms`, `step_ms` and `motion` (ENGINE-49), and `aegis_ui::walkfeel`;
  the screen trait's list: `Screen::motion`. `docs/STRUCTURE.md`: none.
- **Shots diff** (every scenario, base `fd88cc64`): 129 shots, the one difference the Scene Runner's data folder path (the
  base ran from a scratch worktree).

### FB-491: PR7-92 INPUT-REACH2 built; slots pick, place and empty by key, the battle board one Tab stop

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui`: focus, shell, reach), game GUI (`emberwake_gui`:
  the party window, the battle screen) and two game scenarios; no data. **Every golden unchanged** (rules, sim, checks,
  rules contract, journey, timeline, PHASEFOLD). Shots diff: see the last line.
- **Spec**: `docs/READY.md` row `PR7-92` (`INPUT-REACH2`), from `FB-489` calls 1, 2 and 4.
- **Slots without a mouse (calls 1, 2)**, one rule for the skill slots (deck and classic), the gear slots, the pouch and
  the bunk loadout (the same Gear tab):
  - Enter on a filled slot picks it (outlined in gold); Enter on another slot then moves or swaps it there, Enter on a
    list row (owned skill, stash item) puts that row in the picked slot (a swap when filled), Enter on the list itself
    (the owned list, the stash) empties the picked slot, and Enter on the picked slot again drops the pick. Each place is
    the same move a drag makes (`accept_drop`), so the rules and refusals are the drag's.
  - Esc drops a pick (the window stays open); with no pick Esc closes as before.
  - Space (pad X) on a focused filled slot empties it (unslot or unequip, as a drag to the list).
  - The hint line (the window's detail box) names the keys while the focus is on a slot or a pick waits, from the
    player's bindings.
  - Every slot, empty ones too, and both lists are now in the keyboard ring. A mouse click is unchanged on a worn item
    and a pouch item (it takes it off); a click on a skill slot or an empty slot does what Enter does.
  - Engine: `Screen::press_focus(id, key)`, Enter or Space on a focused widget before the shell clicks it (true takes
    the key).
  - Tests: `emberwake_gui` `party::reach_slot_tests` `the_keys_pick_place_swap_and_empty_deck_slots` and
    `the_keys_pick_place_and_empty_gear_slots`.
- **The battle board is one Tab stop (call 4)**: `Screen::focus_group` (new, engine) names id prefixes of widgets that
  make one keyboard stop; the battle screen (story and Test Arena) gives its tiles and unit tokens. Tab and Shift+Tab land
  on the board once (on the tile last focused) and go on to the strip, panels and End turn; on the board the arrows move
  the tile cursor (the focus ring) among its tiles and tokens only, staying put at the edge; Enter presses the tile or
  token as a click. The ring of the Test Arena's plan went from 121 stops to 14. Engine: `focus::step_grouped`,
  `ring_grouped`, `in_group`; the shell keeps the member last focused. Tests: `aegis_ui` `reach` unit test
  `a_focus_group_is_one_tab_stop_and_the_arrows_walk_inside_it`; `emberwake_gui` `board_focus_tests`
  `the_board_is_one_tab_stop_and_the_arrows_walk_it`.
- **Reach**: the report reads the focus group (`report_grouped`, `device_reach_grouped`; `ring` holds the group once,
  `group` its prefixes). New gate scenarios with `fail` on: `reach_party` (stats, skills, gear, the bunk loadout) and
  `reach_battle` (the plan before and after a hero is picked, a round resolving). Release run, no window:

  | Screen | Mouse | Ring | Misses |
  | --- | --- | --- | --- |
  | Party: stats | 7 | 7 | none |
  | Party: skills | 22 | 22 (was 15) | none (was 7) |
  | Party: gear, and the bunk loadout | 17 | 17 (was 7) | none (was 10) |
  | Battle plan, and a hero picked | 121 | 14 (was 121) | none |
  | Battle resolve | 14 | 11 | none |

- **Call 3**: the region map is unchanged (d-pad walks, Y gives Tab).
- **Docs delta**: `engine/README.md`, the screen trait's list: add `Screen::focus_group` (a group of widgets that is one
  keyboard stop; `focus::step_grouped`) and `Screen::press_focus` (Enter or Space on the focus before its click); the
  reach line: the report reads the focus group. `docs/STRUCTURE.md`: none.
- **Shots diff** (every scenario, base `62e57a92`): 129 shots, the one difference the Scene Runner's data folder path (the
  base ran from a scratch worktree); no shot shows a pick or a moved ring.
- **Gate**: one run failed on `aegis_cli` `tests/scenario.rs` `each_start_gets_a_free_port_of_its_own_and_a_real_one_is_never_used`
  ("scratch is gone after"), code this row does not touch; it passed 5 runs of its own and the next full gate. A flake to
  watch (a scratch folder still there when checked); for the known-issues table if it comes back.

### FB-489: ENGINE-48 INPUT-REACH1 built; a `reach` step, every screen walked, three dead ends fixed, four for Design

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui`, `aegis_cli`), kit UI (`rpgkit_ui`) and game GUI code
  (`emberwake_gui`) plus one game scenario; no data. **Every golden unchanged** (rules, sim, checks, rules contract,
  journey, timeline, PHASEFOLD). `tools/fc.sh shots diff` over every scenario (base `16d0aa02`): 129 shots, the one
  difference the Scene Runner's data folder path (the base ran from a scratch worktree). The ring clamp moves no shot: no
  scenario shot shows a focus ring on a whole-canvas widget.
- **Spec**: `docs/READY.md` row `ENGINE-48` (`INPUT-REACH1`), from `FB-487`.
- **What landed**:
  - **`aegis_ui::reach`** (new, engine): pure functions over a screen's widget infos. The mouse set (every enabled widget
    a mouse can click, pick, drag, drop on or type into, less what a popup's backdrop covers), the Tab ring, what the
    arrows walk to from the opening focus (each move as the shell makes it), and what a pad reaches from its buttons in
    the player's `controls` (the ring when a button gives Tab, else its d-pad, and only where `Screen::arrow_focus` is
    true). Three unit tests.
  - **Link request `reach`** (no window, `shell/headless.rs`): the report above, plus a frame per ring widget with its
    focus shown, compared with a frame with none within `REACH_MARGIN_PX` (8) of its box: no change lists it under
    `invisible`, a box that changes between two plain frames under `unsure`. With `back` it presses Esc once and says what
    happened (`quits`, `closed`, `opened`, `swapped`, `changed`, `nothing`) and the top screen after.
  - **Scenario step `reach`** (`name`, `back`, `fail`, `allow`): writes `reach_<name>.json`, adds it under `perf`
    (`kind` `reach`) with `dead_ends`; `fail` fails the step on a dead end that `allow` does not name. Documented in
    `engine/docs/authoring/scenarios.md` ("Input reach").
  - **Gate scenario** `games/emberwake/scenarios/reach_menus.json` (`fail` on): the main menu, load slots, every settings
    tab and Controls; back closes the load slots and Controls; Esc on the main menu is allowed to do nothing.
  - Test `aegis_ui/tests/headless.rs` `the_reach_report_names_what_only_a_mouse_reaches` (a drop-only slot is named, a
    whole-canvas widget's focus shows, a back press reports).
- **Fixes (no design call)**, each with a test:
  1. **A whole-canvas widget's focus ring drew off the canvas** (the cutscene's `vignette.next` stage: focus invisible).
     `draw_ring` clamps the ring to the canvas. Covered by the headless test above.
  2. **The arrows did nothing on a cutscene** (`rpgkit_ui` vignette, `arrow_focus` false and no other use of them): now
     they move the focus. `rpgkit_ui/tests/vignette.rs` `the_arrows_reach_every_button_of_the_cutscene_bar`.
  3. **The arrows walked the party during a conversation, the notice board and a shop on the region map**, so a d-pad
     could not pick a reply or an item (the walk is held while those are open). `arrow_focus` is now true while one is
     open. `emberwake_gui` `map_tests::the_arrows_move_the_focus_in_a_conversation`.
- **Report** (release build, no window, 1280 x 720; scenarios and every `reach_*.json` in
  `/mnt/project-files/build-tools/input_reach/`, `summary.txt` the table's source). Mouse = widgets a mouse can use;
  ring = Tab stops. No screen showed an invisible focus after fix 1.

  | Screen | Mouse | Ring | Keys and pad miss | Back (Esc) |
  | --- | --- | --- | --- | --- |
  | Title (main menu) | 4 | 4 | none | nothing (root screen) |
  | Load slots | 1 | 1 | none | closes to the main menu |
  | Settings: Display, Sound, Gameplay, Battle | 26, 24, 13, 22 | same | none | not pressed per tab |
  | Settings: Controls | 21 | 21 | none | closes to the main menu |
  | Cutscene (story card) | 4 | 4 | none (after fixes 1, 2) | opens the skip question |
  | Skip question | 2 | 2 | none | closes to the cutscene |
  | Story battle, plan | 126 | 126 | none | not pressed |
  | Battle results | 2 | 2 | none | not pressed |
  | Test Arena, plan (and a unit picked) | 121 | 121 | none | not pressed |
  | Test Arena, resolve | 14 | 14 | none | not pressed |
  | Region map (hub) | 35 | 35 | d-pad alone: all 35 (the arrows walk; Y gives Tab) | not pressed |
  | Region map: talk, notice board, shop | 6, 11, 14 | same | none (after fix 3) | board and shop close |
  | Bunk card | 3 | 3 | none | closes to the region map |
  | Party: stats | 7 | 7 | none | not pressed |
  | Party: skills | 22 | 15 | 7 (below) | not pressed |
  | Party: gear, and the bunk loadout | 17 | 7 | 10 (below) | closes to the region map |
  | Game menu | 7 | 7 | none | closes to the region map |
  | World map | 27 | 27 | none | closes to the region map |

  Shops: the region shop above; no separate shop screen was reached in Act 1 play. The pad map is the default (A Enter,
  B Esc, X Space, Y Tab, LB/RB page, d-pad arrows), so the pad reaches what the keys reach.
- **For Design** (each needs a call):
  1. **Party skills: unslotting and reordering are mouse only.** The five slots (`skills.active.0..2`,
     `skills.passive.0..1`) are drag sources only, and `skills.keystone.0` and `skills.list` drop targets only. Slotting an
     owned skill by click works. Options: make a slot clickable (pick it, then Enter on a list row swaps; Enter on the
     slot again unslots), or a context action on the focused slot.
  2. **Party gear and the bunk loadout: unequipping is mouse only.** Ten drop-only slots (`gear.slot.main_hand`,
     `off_hand`, `head`, `body`, `accessory_1`, `accessory_2`, `gear.pouch.0..2`, `gear.stash`). Equipping from the list by
     click works. Same options as 1.
  3. **Region map on a pad: the d-pad walks, so markers, bunks and the HUD are reached only through Y (Tab).** By design
     today; confirm, or add a pad button that jumps between markers.
  4. **The battle plan's ring is about 120 stops** (about 108 of them board tiles), so Tab to the strip or the end turn
     button is long. Options: tiles out of the Tab ring (arrows only), or LB/RB to jump between board and panels.
  Esc on the title doing nothing is left as is (the root screen; no quit by Esc).
- **Docs delta**: `engine/README.md` "No window (HEADLESS1)": add link `reach` (keys and pad reach, focus shown, back;
  ENGINE-48) beside `feel`, and `aegis_ui::reach`. `docs/STRUCTURE.md`: none.

### FB-487: ENGINE-47 GHOST-SETTLE1 built; the guest's ghost was not flipping, ally_moves kept, strip.block1 under 50 ms

- **Status**: built (tools lane), `develop`. Kit battle code (`rpgkit_battle`), one kit test and the game's `feel_battle`
  scenario; no data. **No behaviour change**, so **every golden is unchanged** (rules, sim, checks, rules contract, journey,
  timeline, PHASEFOLD). `tools/fc.sh shots diff` over every scenario (base `16d0aa02`) is clean: 129 shots, the one
  difference being the Scene Runner's data folder path (the base ran from a scratch worktree).
- **Spec**: `docs/READY.md` row `ENGINE-47` (`GHOST-SETTLE1`), from `FB-485`.
- **Part 1, correcting FB-485: the guest's ghost does not flip.** On cages_start each draft was applied to a copy and
  drafted again; all 16 drafts in the run came back the same on the same board. A preset already drafts from the member's
  plan cleared (`ai::plan_round_intent` clears it first), and the battle screen already skips re-sending a member whose
  draft equals what it holds (`draft_ghost`'s `same`). The guest changed on every click because those clicks (`hero.self`,
  the strip block) changed the first member's plan, and the members after a changed one are redrafted by design (PL8,
  `reghost_after`). Nothing to change; the new test pins it: `rpgkit_battle/tests/intent_ai.rs`
  `a_tactic_draft_settles_on_the_board_it_left` drafts each member again under six tactics and finds the same intent,
  target, steps and planned tiles.
- **Part 2: `ally_moves` is worked out once while nothing it reads changes** (`Plan::ally_moves`, `moves_key`,
  `moves_memo`). It is kept with a hash of what it reads: every unit's steps in plan order (each field, the tiles they
  fixed included), the draft preferences, every round state, every unit's lock and where every unit stands. **The step
  tile is handled without moving it**: a run that fixes a step's tile (`PlanStep::cell`, set on first compile) changes the
  key, so that run is not kept and the next call works it out again exactly as before. Under a trial's blocked cells it is
  never kept. This keeps the planner's results bit for bit (no move of `cell` out of compile, which could have shifted the
  sim's plans and its goldens). Audit: with `RPGKIT_MOVES_CHECK=1` every kept value is checked against a fresh one; the
  suites of `rpgkit_battle`, `rpgkit_sim`, `rpgkit_host`, `emberwake_battle`, `emberwake_sim`, `emberwake_rules`,
  `emberwake_world` and `emberwake_content` pass that way. A first try keyed only each unit's first step and failed the
  audit (`tactics_map_to_intents`): the first step's compile reads later steps too, so every step is keyed.
- **Budget**: `feel_battle.json` has a step for the strip block alone (`battle_block`, `input_ms` 50, `frame_ms` 33); the
  rest of the planning script keeps 100 ms.
- **Before and after** (release build, no window, 1280 x 720, this cloud container; `/mnt/project-files/build-tools/ghost_settle/`):

  | Input | ENGINE-46 | Now |
  | --- | --- | --- |
  | `click strip.block1`, input to present | 55 to 59 ms | 43 to 46 ms |
  | First `key tab` | 61 to 64 ms | 45 to 46 ms |
  | `click hero.self` | 52 to 54 ms | 38 to 42 ms |
  | Planning frames, p50 / worst | 17 / 28 ms | 17 / 24 ms |
  | Resolve frames, p50 / worst | 14 / 20 ms | 14 / 20 ms |

  The block pick's margin under 50 ms is small on this container (46 ms worst of three runs); the gate's dev build judges
  it at 125 ms.
- **Docs delta**: none.

### FB-485: ENGINE-46 FEEL-FIX1 built; planning frames under 33 ms, Settings warmed, a stage floor cache

- **Status**: built (tools lane), `develop`. Engine (`aegis_ui`, `aegis_cli`), kit (`rpgkit_battle`) and game GUI code
  (`emberwake_gui`) plus two game scenarios; no data. **No behaviour change**: every cache gives the pixels and numbers
  the uncached path gives. **Every golden unchanged**; `tools/fc.sh shots diff` (every scenario, base `b4ad6a83`) is
  clean: 129 shots, the one difference the Scene Runner's data folder path (the base ran from a scratch worktree).
- **Spec**: `docs/READY.md` row `ENGINE-46` (`FEEL-FIX1`), the follow-ups of `FB-483`.
- **What landed**:
  - **Planning board draw cache** (part 1). `aegis_ui::painter::Snapshot` and `Painter::cached` (new, engine): a layer drawn
    once and put back (`Canvas::copy_rgba` / `put_rgba`) while its key and the view stay the same. The battle Stage's floor
    (tray, wings, backdrop, ground, lights, exit gates: passes 1 and 2) is one; its key hashes the art generation, the
    board's tiles and exits, the zone, set, look and decor, the camera shift, the grid lines and the colours. Marks, ghosts,
    hover and animation draw over it every frame. Panels: a `nine` panel at any scale is composed once and blitted
    (`Canvas::blend_rgba`, a row at a time), its cache bounded by count and bytes (`NINE_CACHE_MAX` 64, `NINE_CACHE_BYTES`
    64 MiB, replacing `REDUCED_MAX` 8).
  - **Block forecast cache** (part 2). `rpgkit_battle::intent::memo` (new, kit): inside a `memo::scope` over one world,
    round and plan, `planned_schedule`, `Plan::compile` per member and each forecast tile read are worked out once (they
    were recomputed about 1200 times a frame). The battle screen's draw opens a scope per frame. Caching applies only to the
    exact world, round and plan the scope borrows, so a changed plan always reads fresh.
  - **Settings warm-up** (part 3). `Context::warm` (new, engine, default none): screens the shell draws once, unseen, before
    the first frame (`Shell::warm`; no tick, no frame counted, audits paused, the canvas cleared after). Emberwake's list
    is the Settings screen and popup on each of the five tabs (`SettingsScreen::on_tab`): their panels, art, font face and
    glyphs are cached before the player opens them.
  - **Budgets**: `games/emberwake/scenarios/feel_battle.json` (cages_start: the plan picks, Tab, Self and the strip block,
    then a round resolving) and `feel_settings.json` (open, five tabs, back), each `feel` step with `budget` `input_ms` 100
    and `frame_ms` 33. A dev build (the gate's) judges each budget at 2.5 times (`aegis_cli::scenario::FEEL_DEV_ALLOWANCE`,
    `feel_misses`); the release run holds the real bar (`tools/fc.sh scenario feel_battle profile=release`). Both pass in
    release and in the gate. Doc: `engine/docs/authoring/scenarios.md` "Input times"; rows in the scenarios README.
  - Tests: `aegis_cli/tests/scenario.rs` `a_feel_budget_stretches_in_a_dev_build`; `aegis_ui/tests/theme.rs` (the panel
    cache stays bounded, now at `NINE_CACHE_MAX`). The goldens and the shot runs check that nothing drawn or forecast moved.
- **Before and after** (release build, no window, 1280 x 720, this cloud container; `/mnt/project-files/build-tools/feel_fix/`):

  | Run | Before (FB-483) | After |
  | --- | --- | --- |
  | Battle planning frames, p50 / worst | 37.7 / 79.9 ms, 37 of 44 over 33 ms | 17 to 18 / 27 to 28 ms, 0 over |
  | Battle resolve frames, p50 / worst | 20.1 / 34.3 ms | 13.5 to 14.3 / 18.6 to 21.6 ms |
  | `click strip.block1`, input to present | 112.7 ms | 55 to 59 ms |
  | First `key tab` in the fight | 84 ms | 61 to 64 ms |
  | Settings first draw, Controls tab first draw | 50 to 57 ms | 9 to 13 ms |
  | Battle plan, worst input to present | 112.7 ms | 58.5 ms |

- **Not under 50 ms: `strip.block1` (55 to 59 ms), for Design.** The click's handler takes about 37 ms, nearly all in
  re-drafting the ghosts of the members after the one changed (`reghost_after`: each later member's tactic preset planned
  on a copy, `ai::plan_party_intents`, 2 to 9 ms each). About 21 ms of it is one finding: **the guest (fifth member) on
  cages_start is re-sent on every plan click**: its drafted plan never equals the plan it was given, because the preset
  planned from plan A gives plan B and from B gives A. Its ghost rows and round target (a tile and Myself) alternate on each
  click; a player would see the guest's ghost flip. Making the preset draft settle (draft from the member's plan cleared,
  or keep the last draft while its inputs hold) would cut the click to about 35 ms but changes what the guest plans, so it
  is not in this row (behaviour unchanged here). The rest of the AI's cost is `Plan::blocked_cells` working out every
  ally's first move again for each route it tries (`ally_moves`, over half of a draft); a cache there would need the plan's
  steps to stop fixing their tiles while compiling (`PlanStep::cell`). Both are proposed as rows if Design wants them.
- **Docs delta**: `engine/README.md` "Feel probe (FEEL-PROBE1)": add "`Context::warm` draws a game's listed screens once
  before the first frame; `Painter::cached` keeps a layer drawn once (ENGINE-46); a `feel` budget counts 2.5 times in a dev
  build". `games/emberwake/docs/GAME_BRIEF.md`: none.

### FB-483: ENGINE-45 FEEL-PROBE1 built; input and frame times, and where the battle's time goes

- **Status**: built (tools lane), `develop`. Engine code (`aegis_ui`, `aegis_cli`), the sample game's scenario and an
  authoring doc; no game code or data. **No behaviour change**: the probe only reads the clock, and the overlay draws only
  when asked. **Every golden unchanged** (the gate passes; `hello_walk`'s new step comes after its last shot).
- **Spec**: `docs/READY.md` row `ENGINE-45` (`FEEL-PROBE1`).
- **What landed**:
  - `aegis_ui::feel` (new): `Probe` notes each input on arrival and when its handler is done, and closes it with the first
    frame that started after it: `to_state`, `to_frame` (the wait for the frame loop), `to_present`, the frame's own ms and
    the `tick` and `draw` the screens took in it. `report` gives p50, p95 and worst, every input and the flagged ones
    (input over 100 ms, frame over 33 ms).
  - The shell: the window notes key presses, clicks and wheel turns (`window_event`), pad events (the poll that read them)
    and each redraw up to `render()`; the no-window loop notes each frame drawn. On with `--feel` or `feel=1` on the command
    line, or the setting `"feel": true`: an overlay at the top left (last frame, worst of the last second, last input's
    input-to-present) and a line per input in `feel.log` beside `game.log` (`crashlog::dir`, `append_beside`).
  - Link request `feel` (no window): replays a script of `key`, `click`, `pad` and `frames` with an optional `times`, `gap`
    frames after each, and replies with the report.
  - Scenario step `feel` (`name`, `script`, `gap`, `budget` `input_ms` / `frame_ms`): `feel_<name>.json` in the bundle,
    under `perf` in `result.json` (`kind` `feel`). `games/hello/scenarios/hello_walk.json` ends with one, so it runs in
    the gate. Doc: `engine/docs/authoring/scenarios.md` "Input times".
  - Tests: `aegis_ui/tests/feel.rs` (5: an input closes with the first frame after it, two inputs in one frame, off notes
    nothing, the overlay text, the report and log flags), `aegis_ui/tests/headless.rs` `a_feel_script_replays_and_times_each_input`,
    `aegis_cli/tests/scenario.rs` (the step parses; eight refusals).
- **Report** (release build, no window, 1280 x 720, Normal text, this cloud container's CPU; nothing is presented, so a
  window adds the GPU copy, the present and vsync's wait). Scenario files and every result:
  `/mnt/project-files/build-tools/feel_probe/`. Overlay shot: `overlay_planning.png` there.

  | Run | Inputs | Input-to-present p50 / p95 / worst (ms) | Frame p50 / p95 / worst (ms) | Frames over 33 ms |
  | --- | --- | --- | --- | --- |
  | Hub walk (`hub_start`: arrows, two click-to-walks, E) | 23 | 19.2 / 29.3 / 29.5 | 18.4 / 27.8 / 31.8 | 0 of 222 |
  | Region walk (`cages_start`, crossroads: arrows) | 20 | 13.5 / 16.8 / 20.3 | 13.2 / 20.6 / 21.2 | 0 of 110 |
  | Battle plan (Cages: party picks, Tab, Esc, a block) | 11 | 43.9 / 112.7 / 112.7 | 37.7 / 65.3 / 79.9 | 37 of 44 |
  | Battle resolve (Confirm, then 120 frames) | 1 | 25.2 | 20.1 / 28.4 / 34.3 | 1 of 124 |
  | Main menu (arrows, New Game) | 7 | 1.3 / 13.4 / 13.4 | 1.2 / 3.7 / 12.5 | 0 of 33 |
  | Settings (open, five tabs, Esc) | 6 | 12.0 / 56.8 / 56.8 | 11.4 / 50.3 / 56.6 | 2 of 34 |

  Flagged, and where the time went:
  - **Battle planning frames, 36 to 80 ms**: almost all in the battle screen's `draw` (35 to 76 ms of each; `tick` under
    0.1 ms). The board, units and forecast marks are redrawn every frame while planning; resolve frames are about 20 ms.
    This is the main thing a hands-on playtest would feel (under 30 frames a second while planning).
  - **`click strip.block1`, 112.7 ms**: 64.6 ms in the click's handler (picking a block re-reads the plan and forecast),
    then a 48 ms frame.
  - **`key tab` (first in the fight), 84 ms**: 37.7 ms in the handler, then a 46 ms frame. The second Tab took 2 ms in the
    handler.
  - **First draw of Settings and of its Controls tab, 50 to 57 ms**: one frame each, in `draw`; the other tabs are 10 to
    13 ms. The first frame of a screen lays out its text.
  - A dev build is about 1.5 to 2 times slower (battle plan frames 43 to 76 ms, the hub walk's frames about 33 ms).
- **Follow-ups (not built; for Design)**: a draw cache for the planning board (draw the static stage once, redraw only the
  marks and hovered pieces); move the block pick's forecast re-read off the click (or cache it per plan change); warm the
  Settings text layout before its first frame. Each would land with a `feel` budget in a release scenario.
- **Docs delta**: `engine/README.md` "No window (HEADLESS1)": add the link request `feel` (a script of keys, clicks and pad
  buttons, timed; ENGINE-45) beside `frames` and `clip`; a line for the feel probe (`aegis_ui::feel`, `--feel`, overlay,
  `feel.log`).

### FB-481: PR7-90 FORECAST-MOVE1 built; a forecast reads the target's tile after every move booked before the hit

- **Status**: built (tools lane), `develop`. Kit battle code and one game test; no data. **Every golden unchanged**
  (rules, sim, checks, rules contract, journey, timeline, PHASEFOLD; the crate suites of `rpgkit_battle`, `rpgkit_sim`,
  `rpgkit_host`, `emberwake_battle`, `emberwake_sim`, `emberwake_world`, `emberwake_rules`, `emberwake_content`,
  `emberwake_gui` pass).
- **Spec**: `docs/READY.md` row `PR7-90` (`FORECAST-MOVE1`), PT-85 from `FB-406`.
- **Cause**: `intent::position_at` walked a moving unit with the move preview over the board as it stood at planning.
  A friend's tile stopped the walk (the real walk passes through it, ESCAPE-PLAN1 (2)); a tile a friend leaves earlier
  stayed blocked; a foe stepping in front earlier in the block did not stop it; a cut-short walk's next walk did not
  re-path.
- **What landed** (`kits/rpg/crates/rpgkit_battle/src/intent.rs`): the forecast walk now plays each booked walk
  against where every other unit stands at that walk's moment (earlier blocks, and earlier in the same block by
  initiative): it passes friends mid-route, takes tiles already left, stops before a unit standing there, stops on a
  sprung trap, steps back off a shared end tile, and re-paths a walk whose start moved, with the same budget as the
  round. Big units keep the old preview read.
- **Tests**: `kits/rpg/crates/rpgkit_battle/tests/forecast_move.rs`, one per logged case (try 2 R2 a walk through a
  friend, try 2 R4 a foe stepping in front, try 3 R1 a tile a friend leaves earlier). Each plays the round and checks
  the forecast tile against the tile the round leaves the unit on. All three fail on the old `intent.rs` and pass now.
  `emberwake_gui` `card_numbers_show_modifiers_and_the_riposte_chip` searched for a pick whose forecast showed a riposte;
  its old pick had the foe walk off before the hit (stepping the round confirmed it, so the old forecast was wrong). The
  search now takes only a pick whose hit lands from a tile next to the foe.
- **Replay**: I replayed run 3's inputs (failed lines left out) from `saves/crucible_entry` with `tools/fc.sh playtest
  run`. Today's build no longer reaches the logged boards: the PR7-88 fixes change the plans from R1, and the script
  stops at a refused popup in R3. So the three cases are rebuilt as fixed boards in the tests above rather than replayed.
- **Not done**: PT-87's foe start tiles (waits on the per-block claim rule, per Design).
- **Docs delta**: none.

### FB-479: PR7-88 PLAN-FIX2 built in part; five of the six run findings fixed, PT-85 and one PT-87 part left

- **Status**: built in part (tools lane), `develop`. Kit battle code and one game draw helper; no data. **Every golden
  unchanged** (rules, sim, checks, rules contract, journey, timeline, PHASEFOLD; the crate suites of `rpgkit_battle`,
  `rpgkit_sim`, `emberwake_battle`, `emberwake_sim`, `emberwake_world`, `emberwake_gui` pass). Every change applies
  only to a round with hand picks, which no golden plays.
- **Spec**: `docs/READY.md` row `PR7-88` (`PLAN-FIX2`), from `FB-406`.
- **How**: I replayed the run's own inputs (`run_log.jsonl`, failed lines left out) from `saves/crucible_entry` with
  `tools/fc.sh playtest run` on today's develop. Try 1 R2's refusal came back word for word. I traced it with a
  temporary state key, since removed.
- **What landed** (`kits/rpg/crates/rpgkit_battle` unless named):
  - **PT-88** (the root cause also covers the odd step costs): the planner's hazard penalty stays on a hero once its
    auto plan has been drafted (`plan.path_penalties`). A hand move then routed around the burning tiles: Maren's
    (3,5) to (7,7) took 8 tiles, not 6. That made it "too far", and every pick that rebuilt her was refused for it
    ("Blocked by Maren's plan"). Now a hero with hand picks walks the shortest route; the penalty only breaks ties
    (`Plan::pass_route`, `route_plain`). The replay now accepts Kaelen's Billhook Sunder there. Test
    `picks::a_hand_move_takes_the_shortest_route_past_the_planner_penalty`.
  - **PT-87, the goldens-safe part**: a later member's ghost plan (no picks, so the planner's, rebuilt after the pick
    by PL8) no longer claims tiles for a hero with hand picks. That member blocks only the tile it stands on
    (`Plan::blocked_for`). "Claimed by Vesper's move" and the 14-tile detour to the winch go. A hand-picked move still
    claims. Test `picks::a_ghost_plan_claims_no_tile_a_hand_pick_needs`.
  - **PT-86**: a crossbow's band starts at 2. The planner's AS2 kite walks to the farthest band tile, up to 4 tiles,
    and a hand pick used it too. For a hand pick (`Builder::hand`):
    - with the foe in the band (inside the foe's melee reach), there is no kite;
    - with the foe adjacent, the shortest step back that reaches;
    - out of reach, only the tiles with the shortest walk are ranked.

    The planner's own plans are unchanged. Test `picks::a_ranged_pick_walks_no_farther_than_it_must`.
    **Spec note for Design**: INPUT1 section 4's archer worked example used the old kite (2 tiles, "10 - 8 = 2, next
    6"). With one step back it reads "10 - 7 = 3, next 7"; `picks.rs` asserts the new numbers. Please update INPUT1.
  - **PT-91**: the fill after hand picks (`Builder::fill`'s new `picked`) works like this:
    - It places no skill that only re-grants a status, whatever the `regrant` switch. Vesper's Move here no longer
      fills 4 Ash-Cloud Fades; the test reproduces exactly that, `move, fade x4` without the fix.
    - It places no non-attack skill a hand pick already runs, such as a second Tourniquet.
    - Attacks may still follow up.

    The planner's whole-round fill is unchanged (the switch's golden ruling, FB-334, stays open). Test
    `auto_plan_fixes::the_fill_after_hand_picks_does_not_repeat_them`.
  - **PT-89**: a windup hit that breaks nothing now says why in the round summary: "no breaks (0 of 3 hits into a
    wind-up: 2 unbreakable, 1 not a breaking hit)", or "missed the break roll". It reads a new `StaggerHit::held`
    (P12) and the counts `RoundSummary::windup_unbreakable / windup_missed_roll / windup_not_breaking`. The log's
    goldened `Stagger` detail is unchanged. In the run, Decapitation is `unbreakable`, so the line would read "2
    unbreakable". `emberwake_gui` `draw::summary_and_prompt` drops the parenthesis whole, never at a comma inside
    it. Tests `poise::a_windup_hit_that_breaks_nothing_says_why` and an extra case in
    `visclar_d_tests::the_prompt_survives_the_longest_summary`.
- **Not done (please re-queue or rule)**:
  - **PT-85** (forecasts on a target that moves): not started. The three cases need the try 2 and try 3 boards. The
    replay method above reaches them, and `intent::hit_on` with `planned_schedule` is where the forecast reads the
    target's tile at the impact. The 415-against-277 gap in the same report is a crit (the card shows it apart).
  - **PT-87, a foe's start tile**: the planner's blocked set holds every foe's start tile (`blocked_for`), with no
    knowledge of when a foe leaves. Routing through it needs the foes' booked moves plus the per-block claim rule
    that stays with Tushar. Not changed.
  - **PT-88's general rule** ("a pick with no move is never refused by another hero's move"): the logged refusal came
    only from the detour above, now fixed. A real route clash still refuses under ESCAPE-PLAN1 (2) ("a pick never
    drops another member's hand picks"). Changing that rule is a decision.
  - **PT-91's AP strip** ("AP -3" while Vesper ended at 1/10): it followed from the 4 Fades. I have no separate
    repro, so no other change. "A filled Strike spent Ammo" was not looked at.
- **Docs delta**: none beyond the INPUT1 worked-example note above.

### FB-477: ENGINE-43 TERRAIN-PARTIAL1 built; partial terrain tables

- **Status**: built (tools lane), `develop`. Kit check change only; no data changed; every golden unchanged
  (`checks_golden` included: no shipped or golden data set carried the removed problem or warning).
- **Spec**: `docs/READY.md` row `ENGINE-43` (`TERRAIN-PARTIAL1`).
- **What landed** (`kits/rpg/crates/rpgkit_content`):
  - The content check no longer reports `terrain:missing` (a terrain file that lacks some built-in types), and the
    warnings no longer list one line per terrain when the file is absent. `View.terrain_file` and
    `simple::missing_terrain` are gone.
  - Nothing changed in the loader: `TerrainTable::from_catalog` already gave every type without a row its code
    defaults.
  - A listed row still runs `terrain_errors` whole (ranges, glyphs, hazards), so the writer refuses a bad one.
  - Test `tests/terrain_partial.rs`: a one-row table (floor) is written, checks and warns clean, keeps its own numbers,
    and the other 16 types match `TerrainTable::builtin()`; a listed row out of range is refused.
- **Docs delta**: `engine/docs/authoring/kit_game.md` and `games/kit_sample/data/README.md` now say terrain rows are
  optional per type (done here). For Design: any spec or brief line saying "all 17 terrain rows or none" is outdated.

### FB-475: ENGINE-42 KIT-SAMPLE1 built; the kit's sample game

- **Status**: built (tools lane), `develop`. New game folder, one kit change (below); Emberwake goldens unchanged.
- **Spec**: `docs/READY.md` row `ENGINE-42` (`KIT-SAMPLE1`).
- **What landed** (`games/kit_sample/`, made with `aegis new game kit_sample`, then given the kit):
  - `game.json` `"kit": "rpg"`, so the kit library is an asset layer. The crate depends on engine and kit crates only.
    Its CLI uses the kit's content kinds (`RpgContent`).
  - Data, all through `kit_sample_cli content upsert`: the 5 skills the kit requires plus `strike`, 2 classes
    (`ks_fighter`, `ks_shooter`), 4 class-derived units, and the encounter `ks_skirmish` (2 a side, zone
    `stone_arena`, decor `fire_pit` at (6, 0)). `content check` is clean apart from one warning per default terrain.
  - The fight (`battle.rs`) is the kit's `Session`, each round drafted by `auto_draft(AUTO_PRESET)` and resolved.
  - The board (`screens.rs` `draw_stage`) draws the kit stage passes: backdrop, composed ground, light, exit gate,
    face, then props, decor and bodies in feet order. It uses stage set `ks_stage`: `floor/sheet.png` is a 256 x 192
    crop of the kit library's floorboards, and `stage.fx.json` gives zone `stone_arena` a tint and a light spot and
    the `fire_pit` a light of radius 3. Bodies and the prop come from the kit library.
  - Scenario `kit_sample_fight`: menu, Start, board (shot), a round, to the end (shot), back. It passes, and the gate
    runs it in `scenarios_all` and `shotsexact`. 5 tests: folder and scenarios, the kit's content check and the art
    found, the fight ending the same way twice, the stage set's zone and light, and the host.
  - `engine/docs/authoring/kit_game.md` (new): starting a kit game from the sample. Indexed in the authoring README;
    `battle_stage.md` links to it.
  - Kit asset `kits/rpg/assets/ui/views/shop.view.json`: the kit library ships the shop layout, so every kit game's
    layout check wanted a shop fixture. This is the game's fixture with the names made neutral (Coin, The Counter,
    Traders); the game's own fixture still overrides it.
- **Kit change**: `rpgkit_content::source::xp_curve`: a missing `progression/xp_curve.json` now means the default
  curve with no problem. A file that does not parse or validate is still a problem. Before, a kit game without the
  file failed `content check`, and the file is no content kind, so there was no sanctioned way to write it. Emberwake
  ships the file, so nothing changes for it.
- **Findings** (not changed):
  - The class-reference message reads "unknown classe" (`singular` of `classes`). It is recorded in
    `checks_golden`, so fixing it needs a re-record.
  - Terrain rows are all or nothing (one row makes the other 16 required), so a small game keeps the code defaults
    and gets 17 warnings.
  - In this fight the party loses at the round limit. The sample shows the loop, not balance.
- **Shots**: `/mnt/project-files/build-tools/engine-42/` (board and end).
- **Docs delta**: `STRUCTURE.md`: a `games/kit_sample/` row beside `games/hello/` ("the RPG kit's sample game,
  ENGINE-42"). `README.md` / `GAME_BRIEF` games list: the same. `engine/README.md`: link `kit_game.md`.
  `CLAUDE.md` layers line could name `games/kit_sample/` as the kit's sample game, as it does for hello.

### FB-473: ENGINE-41 STAGE-KIT1 built; the battle stage drawing is the kit's

- **Status**: built (tools lane), `develop`. No behaviour change.
- **Spec**: `docs/READY.md` row `ENGINE-41` (`STAGE-KIT1`).
- **What moved**: the whole stage module, `emberwake_gui/src/stage.rs` and `stage/composed.rs`, to
  `kits/rpg/crates/rpgkit_ui/src/stage.rs` and `stage/composed.rs` (git moves). It already named no game; it now
  reads the kit crates directly (`rpgkit_battle`, `rpgkit_content`, `rpgkit_data`, new dependencies of `rpgkit_ui`
  with `indexmap`). That is the `stage.fx.json` reader (zones, spots, lights, lip, decal density), the composed
  battle ground, decor drawing, the wall-run caps, the exit path and gate, the geometry and the draw passes.
- **What the game keeps** (`emberwake_gui/src/stage.rs`): `pub use rpgkit_ui::stage::*`, and `impl StageSizes for
  Tuning`. `Geom::new(view, &impl StageSizes)` replaces the one reference to the game's `Tuning`, so the 36 call
  sites are unchanged. The kit also gives plain `Sizes` for a game with no tuning.
- **Tests**: 44 stage tests moved with the module and pass unchanged, `TODAYS_FRAME` among them. They draw at
  `SIZES` (40, 28, 5), the game's default tuning, and use a kit-local `gfx()` canvas. Five tests that read the
  game's own art, data or settings stayed in the game, unchanged: the shipped terrain rows, the two backdrop tests,
  the feet pass and the cover reveal. So did a copy of `painted_root`, which the map view's test uses. New game test
  `the_default_tuning_and_settings_match_the_kits_stage_tests` holds the two defaults the kit tests had asserted
  (Tuning sizes and the `battle_view` setting).
- **Shots**: `tools/fc.sh shots diff scenarios=all` against `dbb6e8bb`: 128 of 129 unchanged. The one change is
  the scene runner's developer panel, which prints the data folder's path (the base ran in a scratch worktree). No
  stage shot moved. Goldens unchanged.
- **Layers**: one comment example named a game set; it now uses a made-up set name.
- **Docs**: `engine/docs/authoring/battle_stage.md` now places the stage drawing in the kit.
- **Docs delta**: `STRUCTURE.md`: `rpgkit_ui` row gains `stage` (the battle stage, ENGINE-41); the `emberwake_gui`
  `stage.rs` row becomes "re-exports `rpgkit_ui::stage`, `StageSizes` for `Tuning`". `kits/rpg/README.md` UI line:
  add `stage`. `GAME_BRIEF` lines that point at `emberwake_gui/src/stage.rs` can point at `rpgkit_ui::stage`.
  `HISTORY.md`: none (a move).

### FB-471: ENGINE-40 ENGINE-GUIDE2 built; the authoring guide catches up, and hello shows the ground composer

- **Status**: built (tools lane), `develop`. Engine docs and the sample game only; no kit or game code changes.
- **Spec**: `docs/READY.md` row `ENGINE-40` (`ENGINE-GUIDE2`).
- **What landed** (`engine/docs/authoring/`):
  - New `battle_stage.md`: which layer owns each part; stage set files and sizes (`<terrain>/sheet.png`, `decal/`,
    `front_face/`, `lip/<terrain>/`, `cover/`, transition pieces); the composed ground; every `stage.fx.json` key
    (`lip`, `spots`, `lights`, `decal_density`, `zones.<zone>.set` / `tint` / `terrain`) with an example; encounter
    `decor` and world `decor_pool` with the generated-board rule and the checks; painted sheets (`foot`, `painted`).
  - `scenarios.md`: run-wide `set=`, `audit=1` and `save=` (a name or an absolute folder), a two-size sweep example,
    and a section on `tools/fc.sh shots diff`.
  - `naming_and_asset_briefs.md`: "Renaming an id" (`content refs`, `content rename`, `--apply`, the matching rule).
  - `region_grounds.md`: "Points the party can look at" (every point `use`, `look` among them).
  - The index (`README.md`) gains the new page and names the new sections.
- **Sample game** (`games/hello/`): the field's grass and a dirt path are now one picture from the engine's ground
  composer (`aegis_ui::ground`, textures from `ground::two_tone`, so no files), made once and cached by its key; the
  walls stay tiles on top and the walk reads only the walls. Test
  `the_field_ground_composes_the_same_picture_with_a_path_on_it`; `hello_walk` passes (its `field` and `walked`
  shots change: they show the path). README row "Ground".
- **Finding for Design**: the code that reads `stage.fx.json`, draws the composed battle ground, the lights and the
  decor lives in the game's GUI crate (`stage.rs`, `stage/composed.rs`), so a second game on the kit gets the files
  checked but not drawn. The page says so. A later row could move the stage look reader and the board compose into
  the kit's UI crate (`rpgkit_ui`); I have not queued or started it.
- **Docs delta**: `engine/README.md` could link `engine/docs/authoring/battle_stage.md` beside the region grounds
  line; `STRUCTURE.md` engine docs row: add `battle_stage.md`.

### FB-469: ENGINE-39 TARGET-PRUNE1 built; fc.sh prune and a low-disk prune before the gate

- **Status**: built (tools lane), `develop`. Tools only (`tools/fc.sh`, `tools/planner/prune_target.py`).
- **Spec**: `docs/READY.md` row `ENGINE-39` (`TARGET-PRUNE1`).
- **What landed**:
  - `tools/fc.sh prune [target=<dir>] [--dry]`: in each `<target>/<profile>/deps` it groups the executables named
    `<name>-<16 hex>` by name, keeps the newest (by modified time) and removes the rest; then it removes each
    `<target>/<profile>/incremental` folder (the gate builds with `CARGO_INCREMENTAL=0`). Libraries (`.rlib`,
    `.rmeta`, `.so`, `.d` and the like) and unhashed files stay. `--dry` lists the count and size and removes nothing.
  - `tools/fc.sh gate` runs it first when the target folder's disk has under `FC_PRUNE_MB` (default 6000) MB free, and
    says so in one `[gate]` line.
  - Test: `tools/planner/test_prune_target.py` (a fake target folder: the newest per name stays, libraries stay, a dry
    run removes nothing, a missing folder is nothing to do), run by `plannertests`.
- **On this cloud session**: the first run removed 45 stale binaries and folders, 1.16 GB.
- **Known-issues**: the cloud disk-full row now starts with `tools/fc.sh prune`.
- **Docs delta**: `docs/workflow/cloud.md` disk note could name `tools/fc.sh prune` (the gate runs it on its own when
  low). No README change needed (the verb is in `tools/fc.sh` help).

### FB-467: ENGINE-38 LORE-RENAME1 built; content refs and a dry-run-first rename

- **Status**: built (tools lane), `develop`. Engine only (`aegis_content::rename`, `aegis_cli` content commands).
- **Spec**: `docs/READY.md` row `ENGINE-38` (`LORE-RENAME1`).
- **What landed**:
  - `aegis content refs <kind> <id>` (`tools/fc.sh content refs ...`) prints every JSON string in the data folder
    (catalogs, worlds, seeds) that names the id, as `file:line: text`, then every asset file or folder under
    `<game>/assets` named by it.
  - A string names the id when it is the id, or when one of its `/` or `:` parts is the id or has the id as its stem
    (`rogue/<id>`, `portraits/<id>.png`, `<world>:<id>`). Prose and longer ids (`<id>_elder`) never match.
  - `aegis content rename <kind> <old> <new> [world=] [--apply]`. Without `--apply` it lists the files (with string
    counts) and the asset moves and writes nothing. With `--apply` it rewrites only the matching strings, so each file
    stays byte for byte otherwise, and moves the asset files or folders. Everything is put back if the game's
    `content check` then lists a problem it did not list before.
  - The kind's entry must exist and the new id must be free. `flag` is the kind with no entry.
- **On the real data**: `refs units tallow_dredger` lists 30 places in 13 files (data and seeds) plus
  `assets/units/rogue/tallow_dredger/`. An applied rename to `tallow_skimmer` on a copy of the game added no
  content-check problem and left no reference behind. The shipped data is not changed.
- **Limits**:
  - It matches by text across all kinds, so an id shared by two kinds (a unit and a character both called `x`) is
    renamed in both. The dry run shows each place, so check the list.
  - Ids inside Rust code, docs, scenarios or playtest recordings are not touched (`tools/fc.sh rename` covers code).
  - The cloud can't push moved LFS art; the PC push thread lands such a move.
- **Seam**: reads and probes go through `aegis_core::vfs`; its file writes and moves are native tooling, so
  `aegis_core/tests/fs_lint.rs` lists `rename.rs` among the writers and allows `rename` beside `write`.
- **Tests**: `engine/crates/aegis_content/tests/rename.rs` (fixture game: a unit referenced by an encounter, a quest, a
  seed, an art folder and a portrait path; the dry run writes nothing; refs lists every place; prose and longer ids
  stay; a rename that adds a problem is put back).
- **Docs delta**: for `engine/README.md` (content commands) and `docs/STRUCTURE.md` tools list: `aegis content refs`
  and `aegis content rename` (dry run first).

### FB-465: ENGINE-37 LOOK-POINT1 built; a look-only point, the winch drum off its flag

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `ENGINE-37` (`LOOK-POINT1`).
- **What landed**: a `point` marker may take `use: "look"` (`rpgkit_data::LOOK_USE`, in `POINT_USES`): it is listed in
  the Here list (any point with a use is), acting on it logs its line ("The gorge winch drum: Greased iron chain ...")
  and does nothing else, it sets no flag and is never spent, and it draws the Look glyph. The content check takes it
  like the other uses. `crucible_watchpost`'s `winch_post` now has `use: "look"` and no `sets_flags`, so
  `sunk_ward.winch_seen` (read by nothing) is gone. Seed `engine_37.json` writes the location's `markers` field.
- **My guess**: the row says "a marker kind". I added a point `use` rather than a new marker kind, because a point
  already has the label, note, Here row and glyph, so one value covers it. Say if a separate kind is wanted.
- **Tests**: kit `rpgkit_world/tests/points_of_interest.rs` `a_look_only_point_is_present_and_answers_with_its_line`;
  `rpgkit_content/tests/point_use.rs` covers every use; game `region.rs` `pt43_usable_points_join_the_here_list` (a
  look point is listed and acting on it sets no flag).
- **Docs delta**: `GAME_BRIEF` or the authoring notes, if they list point uses: add `look` (listed, answers, sets
  nothing).

### FB-463: PR7-87 FORECAST-FIX1 built in part; the forecast follows moves, pushes and stealth

- **Status**: built in part (tools lane), `develop`; goldens unchanged (`sim_golden`, the timeline forecast-vs-resolve
  check `hit_forecast_timeline` green).
- **Spec**: `docs/READY.md` row `00g3b` `PR7-87` (`FORECAST-FIX1`).
- **Correction to FB-459 (PT-67)**: a shaped blow's forecast already re-reads its shape on the target's tile
  (`intent::area_cells`, PT-17). The real gaps were in where the forecast puts units at the impact
  (`intent::position_at`), which only followed Walks, and that it never read stealth. Fixed both:
  - `position_at` now follows a carried Engage (as it starts), a Disengage (its landing tile) and a carried retreat
    (after its hit), each landing before the block read. Vesper's Hamstring Sever retreat was invisible to it.
  - A new fail `HitFail::Hidden` ("lost (stealth)"): an attack on a target, or a blow shaped on it, that holds stealth
    (now, or granted by its own action landing earlier) and has moved by the impact is forecast lost, as the resolver
    loses it (`intent::hidden_at`, matching `round_intent::hidden`).
- **PT-69**: a push or pull from another unit's attack locked on a unit, landing earlier, moves that unit in the
  forecast (`forced::forecast` steps), so a pushed foe's own blow is read from its new tile and its new target shows
  the incoming hit.
- **PT-72 (rest)**:
  - At pick time the kit already returns every later member's dropped pick, but the battle screen showed only the first.
    It now shows all of them, each named when there are several (`input::dropped_message`).
  - An earlier member's hand pick can't be voided: such a pick is refused with the reason. An auto plan that a pick
    breaks is redrafted at confirm. A sweep of every tile on a test board found no pick that breaks one. I added no
    second notice, since I couldn't test it.
  - The odd costs ((7,4) 4, (8,4) 9) and "No path" through a foe leaving in B1: not reproduced. The run log keeps only
    200 characters per read. Not done; `ENGINE-35` can capture the full board with `keep_saves=` and a `save=` shot.
- **Tests**: kit `hit_fail.rs` `a_stealthed_target_that_moves_is_forecast_lost`,
  `a_disengage_before_the_impact_moves_the_forecast_target`, `a_push_before_a_blow_moves_the_forecast_attacker`; game
  `battle/input.rs` `every_dropped_pick_is_said_at_pick_time`; `hit_forecast_timeline.rs` counts `Hidden` as a fail.
- **Docs delta**: none.

### FB-461: ENGINE-36 TOOLS-REPEATS2 built; shots from a save, seed-from for many ids, kept playtest saves

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `ENGINE-36` (`TOOLS-REPEATS2`).
- **What landed** (engine `aegis_cli`, tools):
  - (1) `scenario shots <name> save=<name or absolute folder>` launches from a save (a name under
    `scenarios/saves/`, or any folder, such as a kept playtest save), so a map or hub proof shot is one line:
    `tools/fc.sh scenario shots hub save=/abs/dir do=click:menu.continue,screen:region_map,shot:map`. A scenario
    file's `launch.save` takes an absolute folder too. It reuses the `do=` steps rather than adding a game-named step
    (the engine names no menu ids). Live check: `build-tools/engine-36/hub_from_save.png` (the hub from `hub_start`).
  - (2) `tools/fc.sh seed-from <kind> [<world>] --ids a,b,c [--set k=v]... [--fields a,b]` writes one step per id
    (same changes for each, in order; an empty or repeated id list is refused).
  - (3) `tools/fc.sh playtest start keep_saves=<dir>` keeps each `save <name>` also as `<dir>/<run>/<name>` and the
    saves as the game stops (`stop`, `quit`, the end of `run`) as `<dir>/<run>/last`. A cloud thread passes
    `/mnt/project-files/build-tools/playtest_saves/`; the engine names no path.
- **Fix to PR7-83**: its `forced.rs` comment named a hero, which the gate's layers step refuses (the PR7-83 gate stopped
  early, so I missed it); the comment now names no one. develop failed `layers` between the two landings.
- **My guess**: (1) read "a scenario step" as the `shots` keys plus `launch.save` taking a folder path, not a new step
  kind, since loading a save happens at launch. Say if a mid-scenario load step is wanted.
- **Tests**: `aegis_cli` `tests/scenario.rs` `a_save_folder_by_path_launches_shots_with_no_scenario_file`;
  `agentplay.rs` `keep_saves_copies_the_saves_beside_the_run` and the session file round trip;
  `tools/planner/test_tools_help.py` `test_ids_write_one_step_per_id_in_one_call`.
- **Docs delta**: `engine/docs/authoring/agent_playtest.md` updated here (the `keep_saves=` lines). For `engine/README.md`
  or `docs/STRUCTURE.md` (tools list), if they list the shots keys: add `save=` to `scenario shots`, `--ids` to
  `seed-from`, and `keep_saves=` to `playtest start`.

### FB-459: PR7-83 ACT1-FIX3 built in part; two rule fixes held because they move the sim golden

- **Status**: built in part (tools lane), `develop`; **needs Design** on PT-70, PT-71, PT-72 and PT-74 below.
- **Spec**: `docs/READY.md` row `00g3` `PR7-83` (`ACT1-FIX3`, `FB-392` findings).
- **Fixed** (goldens unchanged):
  - PT-73: a partial escape (some heroes out, fewer than needed, none left fighting) reads "Partly escaped" on the
    banner and the results, with the reason "2 of 3 got out; the escape needed 3." (kit `Session::partial_escape`,
    used by `end_reason`; game `hub::PARTLY_ESCAPED`). A retreat or a round-limit end is never partial.
  - PT-75: the boss bar's span shows HP carried over from a phase that ended by time (" +138"), and its hover says
    "This phase: left/span" and "Carried over: n HP from an earlier phase that ended by time, still to take."
  - PT-76: `tallow_dredger` short name "Tallow Dredger" (seed `pr7_83.json`, fields only).
  - PT-79: Enter at "Round over" only dismisses the summary (a second Enter confirms); a party slot click always
    selects (no toggle off); Escape steps back one layer at a time and stops once with nothing open before it leaves.
  - PT-81: planning a second exit move reads "{name} leaves by the exit earlier in this plan; nothing can follow."
  - PT-84: a pull or push of 0 tiles with nothing stopped logs nothing.
  - PT-68 (text): stealth hides a unit only from single-target attacks, and only in a round it moved
    (`round_intent::hidden`); Ash-Cloud Fade now says so ("for 2 rounds, a single-target attack cannot find you
    in a round you moved"; the old text also read "2 rounds rounds"). The hits in the run were area attacks (Heretic's
    Brand, Hammer Strike) or rounds Vesper did not move.
  - PT-69 (text): the Pulverizer is `unbreakable`, so `sure_break` cannot break its wind-up; Winch Sunder now says
    "breaks a windup outright (not an unbreakable one)".
- **Held for Design** (each tried, each moves `sim_golden`; reverted, per "goldens unchanged"):
  - PT-71: an arriving wave passing over the tiles the party's planned walks claim. Tried the whole path, then the goal
    only: both change `sim_golden` (the sim's waves land on claimed tiles). The kit helper
    `World::place_leftovers_avoiding` is in, tested and unwired. Options: (A) wire it, re-record `sim_golden` as the
    intended change (recommended: one line, low undo cost); (B) leave spawns as they are. On the "dropped move with no
    log line": every cut walk logs "[Move Cut Short] ..."; the run log keeps only 200 characters of each read, so I
    could not find the dropped line. `ENGINE-35` could capture the full log.
  - PT-70: a re-route that would end a block farther from the goal stays put (tried for every unit, then the party
    only: both change `sim_golden`). On open ground a cut walk already re-routes toward its goal (test below); the
    run's "wrong way" looks like the long way round a closed gap. Options: (A) party walks never end a block farther
    from the goal, re-record `sim_golden` (recommended); (B) keep the shortest route even when it starts away.
- **Reported, not changed**:
  - PT-72, the path-claim rule (no change made). Today a planned move claims every path tile for the round, so a
    1-tile gap passes one hero a round. Options: (A) keep; (B) a move claims each tile only in the block it walks it,
    so a friend may follow a block later (recommended: matches what players expect, friends already pass mid-block);
    (C) claim only block-end tiles. The rest of PT-72 (the "voided by a later pick" line at pick time, odd costs,
    "No path" through a foe leaving in B1) is not done: the pick already returns the dropped later picks
    (`Session::pick`), the game does not show them at pick time yet. A follow-up row.
  - PT-74: the auto plan is the sim's planner, so teaching it the escape objective changes `sim_golden` by
    construction. Not started; it needs an intended re-record.
  - PT-67: my reading (not proven by a test): Hammer Strike is a shaped Assault, so at impact its shape is re-read on
    its target's tile (`CmdKind::Area.target`); the forecast reads the cells fixed at planning, so a disengage looked
    safe ("-0") while the blow followed Vesper. Fix: the forecast follows the target the same way. Not done this row.
  - PT-69 (forecast part): a pushed skill's new target showing an incoming forecast is not done.
  - PT-80: data, not a rule. Tourniquet is 3 AP, 2 blocks and Cauterize (brimstone) 2 AP, 1 block on every target;
    both reach 2 tiles, so a far ally adds the walk to reach (my reading of the 8 AP and 4 blocks; not traced in a
    test). The "-3 blocks, -3 AP" rows after a Move pick show what that Move already spends.
- **Tests**: kit `escape_plan.rs` `an_escape_short_of_its_count_reads_as_partly_escaped`,
  `an_arrival_passes_over_a_claimed_tile`, `a_move_cut_short_reroutes_toward_its_goal`; game `hub.rs`
  `a_partial_escape_reads_partly_escaped`, `boss_ui1_tests.rs` (carried HP), `input1_tests.rs`
  (`a_fresh_escape_with_nothing_open_leaves_at_once`, `enter_at_round_over_only_dismisses_the_summary`, the
  Escape steps), `visclar_d_tests.rs` (the second Enter confirms).
- **Docs delta**: none.

### FB-457: PR7-80 WALLMASS1 built; wall runs as one mass, north and south exit paths

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `00g2` `PR7-80` (`WALLMASS1`).
- **What landed** (game `stage.rs`, engine `aegis_ui`):
  - (a) A run of blocks (side by side) shares one cap: the cap's tile variant is picked at the run's root (its
    topmost, then leftmost cell), so neighbours never change art at their seam (`stage::run_root`). When the wall
    terrain has a ground sheet (`<dir>/wall/sheet.png`, none ships yet), each cap is cut from the sheet at its place
    on the board, so the caps continue one picture (`stage::sheet_cap`); Art/UI can add one per set. Side lines and
    the top lip were already drawn only on the run's outline, and stay so. A run joined to the back row takes a
    darker shade (`OUTER_CAP_SHADE`, alpha 46) toward the back rows' look. No walkability change.
  - (b) Top and bottom exits draw `props/<set>/exit_path_north/v*` (the bottom one flipped top to bottom); the side
    exits keep `exit_path` (mirrored on the west); a set without the north art falls back to the east art
    (`stage::exit_look`). Engine: `Image::flipped` and `Art::image_flipped` (cached beside the mirrored copies).
- **My guess**: "the cap lowered toward the back rows' look" read as a darker tone on back-row runs, not a smaller
  lift (a lower lift moves every block's rect, decor on blocks and the depth sort). Say if a real lowering is wanted.
- **Golden**: `stage.rs` `TODAYS_FRAME` re-recorded `0xffa27f50b69d7596` -> `0x7898f381e249352d`, the one intended
  change (one variant per wall run, the back-row shade).
- **Tests** (`stage.rs`): `adjacent_walls_share_one_cap_and_a_lone_wall_keeps_its_outline` (one root and variant, no
  side line between neighbours, a lone wall keeps both, a sheet's caps continue across the edge, the back shade);
  `top_and_bottom_exits_take_the_north_art_flipped_for_the_south`.
- **Shots** (`fc.sh shots diff`, base `83c8e910`): 12 of 18 changed, all on the walls (one brick variant per run, the
  darker back band). Sheet `build-tools/pr7-80/shots_diff_sheet.png`, with two before | after crops beside it.
- **Docs delta**: `docs/HISTORY.md` row: "2026-11-22 PR7-80 WALLMASS1: the stage frame golden re-recorded for one tile
  variant per wall run and the back-row shade." `engine/README.md` `aegis_ui` line: add `Image::flipped`,
  `Art::image_flipped`.

### FB-455: ENGINE-33 XL-SWEEP1 built; the Extra large sweep, four fixes, findings

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `ENGINE-33` (`XL-SWEEP1`).
- **Sheet**: `build-tools/engine-33/xl_sweep_sheet.png` (fixes before and after, then the findings), with the per-shot
  audit lists `audit_xl.txt` and `audit_normal.txt` beside it.
- **How it ran** (engine and tools):
  - `aegis scenario run` (and `fc.sh scenario`, `shots-all`) take `set=<key>:<value>,...`: settings laid over every
    scenario's own (the driver's held settings and `mute` refused), so one run takes every screen at
    `set=text_scale:1.3`.
  - `audit=1`: after every shot the text audit of that frame is kept beside it as `<shot>.audit.json` (cut, edge,
    overlap, spill); `shots_all.py collect` copies them and writes `audit.txt`, the shots with a problem, most first.
  - The sweep: all 34 scenarios, 129 shots at 1280x720, at Normal and at Extra large, compared with `compare=` (the
    sheets of every shot side by side). Extra large raised the shots with a listed problem from 38 to 63.
- **Fixed here** (tests in `battle/tests/xl_battle_tests.rs` and `worldmap.rs`):
  - Battle plates (party and Hostiles): "hp/max" with its planned loss ran past the plate ("120/120 -120"); the
    maximum now drops first ("120 -120") only when the two would not fit (`plates::hp_fit`). Test
    `a_plate_drops_its_maximum_before_the_loss_runs_past_it`.
  - The docked unit card: a long name ran into the HP numbers under it ("Watch Crossbowman" over "136/136"); with
    no title line and a name line taller than its slot, the name stops before the numbers, and a cut name hovers to
    its full words. Names are now fitted in the display face they are drawn in. Test
    `the_card_name_stops_short_of_the_hp_numbers` (it fails without the fix).
  - The boss bar name: fitted in the display face, its box as tall as its line (it spilled at Extra large).
  - The world map side panel: "No open road to `<place>`." wraps instead of being cut, and wrapped lines step a whole
    line at larger text (8 at Normal, as before). Test `the_side_panel_lines_grow_with_the_text_size`.
  - At Normal no text moved (the 16 Normal shots that changed are the ground art from develop).
- **For Art/UI** (layout file `kits/rpg/assets/ui/layouts/shop.json`, not edited here): at Extra large "Confirm
  trade" is cut to "Confirm t..." and the shop row's "+10" button sits over the Trade column's edge. A wider Confirm
  button and a little more room for the row buttons would clear both.
- **For Design** (bigger than a layout fix; my guesses in brackets):
  - (a) The battle strip's "Round 1 of 12" is always cut to "Round 1..." at Extra large (the hover has the words) and
    its line touches the goal line under it. [Shorten to "R 1/12" at large sizes, or rely on the bottom bar's
    "Round 1/12".]
  - (b) The Character Stats and Gear stat rows are 12 units high; at Extra large the lines nearly touch and "Debuff
    resist %" meets its value. Readable, but tight. [Grow the row with the text size.]
  - (c) Ellipsis cuts by design remain: shop item names, "Party (24 points to spend)" on the hub menu, the save
    slot's place, skill lines on the Skills tab, the timeline's foe names. Most have a hover; a pass to check each
    hover is listed in `audit_xl.txt` under `cut`.
  - (d) Audit noise worth knowing: the board's tile widgets count as controls, so the battle log and the ally menu
    over the board show as spills; `layout_audit` already skips them, `text_audit` does not. [A later engine row: a
    screen may name boxes the audit ignores.]
- **Docs delta**: `engine/README.md` scenario line: add `[set=<key>:<value>,...] [audit=1]` to `aegis scenario run`
  (settings over every scenario; a text audit beside each shot). `STRUCTURE.md`: none.

### FB-453: PR7-82 RETRY-SAVE-FIX1 built; the autosave after a lost fight works again

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `00f3` `PR7-82` (`FB-392` PT-66).
- **Cause**: `PR7-75` keeps an encounter's lost tries in `gs.rolls` under `retry:<encounter ref>` (for example
  `retry:sunk_ward:enc_evt_03`), but the save's check took only a plain stream id as a `rolls` key, so every save after
  a lost or fled fight was refused.
- **What landed** (kit, `rpgkit_world`):
  - `state::roll_key_ok`: a `rolls` key is a stream id or `retry:<ref>`, the ref a qualified id `world:id` or a plain
    id; the check uses it, and its message names both forms. The key and the count stay as they were, so
    `retry_seed` gives the same seed for the same count, and an old save (with no retry keys) loads as before.
  - A fight with no encounter ref no longer writes a `retry:` key (it has no seed to vary, and the key was invalid).
- **Test**: `rpgkit_world/tests/world_fight.rs` `a_lost_fight_saves_and_reloads_with_its_retry_count` (a loss
  validates, saves, reloads with its count and rolls the same retry seed; malformed keys still refused).
- **Docs delta**: none (no README names the `rolls` keys).

### FB-451: PR7-77 DIORAMA5 R6 built; no grid at rest, coherent blobs, no pieces over the blend

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `00f2` `PR7-77` R6 (user 2026-11-22 08:45), with Art/UI's continuous cobbles (PR #95).
- **What landed**:
  - Grid: `tuning.json` `grid_alpha` 0 (the tuning now takes 0). The Settings row is "Show full grid" (same key
    `board_grid`), off by default; on, the full grid shows faint at rest (`stage::FULL_GRID_ALPHA` 34) and at
    `grid_alpha_planning` while planning. Off, no lines at rest or while planning: the cells that matter keep the marks
    they already had (the hover plate, the Move's tiles while it waits, the target and area plates), so no new outline
    was added.
  - Blobs: the composer's random patches (about a cell each) are off on the battle board; instead up to 3 placed
    patches 2 to 4 cells wide and 2 to 3 high, picked by the board's seed, each on one open non-sunken ground with a
    ring of that ground (`composed::blobs`, `BLOB_STRENGTH` 0.14). The composer's tone noise and edges were already
    sampled by art pixel over the whole board.
  - Sheets were already sampled by board pixel with a seeded offset (the composer's texture offset), so their seams do
    not sit on cell lines (unless the offset falls on a multiple of 64, one chance in 64 per axis).
  - No `over_*` transition piece draws when the ground is composed; a board without sheets keeps them.
- **Changed shots** (`fc.sh shots diff scenarios=diorama_zones,diorama_stage,boss_bar`, base `8c261fcc`): all 7, the grid
  gone and the edge pieces gone on the composed boards. `/mnt/project-files/build-tools/pr7-77-r6/`.
- **Guesses** (`[PROVISIONAL]`): (1) existing marks count as the outlines of the hovered, move and target cells; (2) a
  player who had the old Board Grid on keeps the key's saved value, so their full grid stays on.
- **Tests**: `battle_ui2c_tests` `the_board_grid_setting_turns_the_grid_lines_off` (off by default, 0 at rest, full
  grid stronger while planning), `stage.rs` `the_patches_are_a_few_coherent_blobs`.
- **Docs delta**: SCREEN_CONTRACT_BATTLE / the diorama spec: the grid off at rest, "Show full grid", blobs, no pieces
  over the composed ground. HISTORY row: "PR7-68 R6's faint grid at rest is replaced by no grid (PR7-77 R6, user
  2026-11-22: the grid kept the board from reading like the reference)".

### FB-449: PR7-77 DIORAMA5 R1 to R5 built; composed ground, decals, lips, lamp light, exit path and gate

- **Status**: built (tools lane), `develop`.
- **Spec**: `docs/READY.md` row `00f2` `PR7-77` (`DIORAMA5`), R1 as amended (the region ground composer), R2 to R5;
  Art/UI's placeholders from PR #94.
- **What landed**:
  - Engine (`aegis_ui`): the composer's `CELL_MAX` is 64 (it was 32), so the board composes at the stage art's own 64 x
    48 a cell; the region map keeps its own cap of 32 (`ground.rs` `REGION_CELL_MAX` in the game), so region maps are
    unchanged. `Art::pick_hashed` and `art::cell_hash` (a variant from an integer hash of the cell). `Canvas::add` and
    `Painter::glow` (an additive light pool with a smooth falloff).
  - (R1) `stage/composed.rs`: when any ground on the board has a `sheet.png` (override folder, then the zone's set,
    then the world's, then `common_stage`), the board's ground is one composer job: one layer per ground shown, its
    sheet as the texture (tinted as the zone), the commonest ground lowest and the others over it with worn edges,
    strays, a quiet tone and patches; sunken terrains keep to their cell edges (`clamp_water`) so the lips fit. A
    terrain without a sheet gives its first tile as its texture. The job is rebuilt only when the board, sets, zone look
    or art change; the composer's cache keeps the picture. Without any sheet the tiles draw as before, now picked by
    `pick_hashed` instead of (7x + 13y) mod n. Then accent decals: `<terrain>/decal/v*` on ground cells where
    `cell_hash` falls under `decal_density` (default 10 percent, 35 percent beside a block or a prop; the zone's
    `decal_density` over the file's). Transition pieces still draw over the result.
  - (R2) a sunken cell's lip from `lip/<terrain>/v*` (an override `<set>/<name>` reads `<set>/lip/<name>`, then the
    zone's set, then the world's), cut from its top to the band; the flat fill stays the fallback.
  - (R3) the lantern pools are additive glows at 150 at the centre (about twice the old rings); with any light the board
    darkens a little (`DARKEN`, alpha 34) so the pools read. New `stage.fx.json` key `lights`, top level or per zone
    (the zone's over the file's): `{"cover/v3": {"color": "#ffb060", "radius": 1.5}}`, a prop's `"<prop>/v1"` or bare
    `"<prop>"`; a matching cover or prop casts a pool on its cell and its neighbours (`stage::lamps`). Radius past 4
    cells is ignored.
  - (R4) Stage view, plain exits only (a hold escape's SHUT and OPEN signs stay): the `exit_path` decal on each exit
    cell at 60 percent with a faint 2.4 s pulse (static under Reduce motion), 92 percent when hovered or when an active
    hero stands one step from it; mirrored on a west exit, since the chevrons point east. The yellow plate and the ">"
    sign are gone where the art exists. An `exit_gate` stands beyond each run of exits on the top edge (on the
    backdrop's foot) and on the side edges (in the wing by the run's lowest cell); none on the bottom (no room under
    the front face).
  - (R5) rubble in a zone whose set has no `rubble` draws the zone set's floor (before the world's rubble).
  - Tools: `fc.sh shots diff` no longer marks a failed base run done in its cache (a full disk had cached an empty
    base, so every shot came out "new").
- **Golden**: the `stage.rs` fixture frame hash `TODAYS_FRAME` is re-recorded (`0x2a5309c84d603849` to
  `0xffa27f50b69d7596`) for the one intended change: the fixture's two grass variants are picked by the cell's hash
  (R1 c). HISTORY row: "PR7-77 (DIORAMA5 R1): the stage's tile variants are picked by an integer hash of the cell;
  the (7x + 13y) mod n lattice read as an alternating pattern (user 2026-11-22)".
- **Changed shots** (`fc.sh shots diff scenarios=diorama_zones,diorama_stage,boss_bar`, base `3c3873a9`): all 7
  changed; the floors are continuous with ragged joins, the window pools read, the exits show the path and gate.
  Sheet and shots: `/mnt/project-files/build-tools/pr7-77/`.
- **For Art/UI**: no `stage.fx.json` has `lights` yet, so no lamp casts a pool today; add e.g. `"lights":
  {"cover/v3": {"color": "#ffb060", "radius": 1.5}}` to the sets with a lamp post cover. The exit gate is drawn at
  32 x 36 logical (64 x 72 art). The decal and path have no north or south variant, so a top or bottom exit shows the
  east chevrons; a `v1` turned that way could be added later.
- **Guesses** (`[PROVISIONAL]`): (1) "a unit can step there" is an active hero one step away; (2) gates on the top and
  side edges only; (3) the darkening shows only when the set has spots or lamps; (4) the layer numbers (wear 0.12,
  tone 0.06, patches 0.12) are the region look, quieter; (5) the glow is a true add (no new blend mode for the board
  otherwise).
- **Tests**: `stage.rs`: `the_variant_pick_scatters_with_no_lattice`, `the_sheet_and_the_rubble_ground_follow_the_zone`,
  `a_ground_sheet_composes_one_continuous_floor`, `decals_fall_by_the_hash_density_and_gather_near_things`,
  `a_sunken_lip_draws_its_art_before_the_fill`, `a_lamp_prop_casts_its_pool`, `the_exit_reads_as_a_path_and_a_gate`;
  the lantern test updated for the darkening.
- **Docs delta**: `engine/README.md` (`aegis_ui`): `ground::CELL_MAX` 64; `Art::pick_hashed`, `art::cell_hash`;
  `Canvas::add`, `Painter::glow`. SCREEN_CONTRACT_BATTLE 8.1 (or the diorama spec): the composed ground, decals,
  `decal_density`, `lights`, lip art, exit path and gate. STRUCTURE: `emberwake_gui/src/stage/composed.rs`.

### FB-388: PR7-76 ACT1-FIX2 built; grate silt parked, guesses and data notes

- **Status**: built, `develop` (squash `PR7-76`). Goldens unchanged; two worldmap unit-test expectations changed as
  PT-45 intends (`roads_show_on_focus_only...`, `the_sunk_ward_map_shows_roads...`). Shots in `/mnt/project-files/pr7-76/`.
- **Parked, needs a call (grate silt)**: the validator refuses `mud` ground on a `~` (water) cell, so the west-basin silt
  banks cannot be ground-only. Option A: change `rows` and `ground` at x=9, y=7, 8, 10, 11 of `sunk_ward_runoff_grate`
  (walkable silt, a gameplay change; my reading of "west causeway row" is y=9, kept as water). Option B: a visual-only
  silt exemption in the validator (logic row). Nothing applied; the ground-only seed waits in the Build scratchpad.
- **Guesses marked [PROVISIONAL]**:
  - PT-54: a directional exit preselects a place only when the location has 2 or more world-map exits and the exit is
    not the generic `world_map` one; the open neighbour best lined up with the exit's direction from the map centre
    (cosine above 0) wins (`worldmap::exit_toward`).
  - PT-56: quests ready for the same person, with nothing to choose and no recipient, join one "Turn in: N quests"
    confirm.
  - PT-61: dialogue choices log "Paid N Scrap.", "Received N ...", "Handed over X xN.", "Received X xN." and newly ready
    quests.
  - PT-50: a ready quest stays in the legend as "`name` (ready)" with the turn-in icon.
  - PT-53: the Make camp reason sits in small dim text under the button.
  - PT-42: a countdown tooltip adds "At 0 days: `warning`" (or "the run ends."), plus the restart length.
  - PT-46: a press during typing shows the card, a press while held moves on, a press in the fade-out jumps to the next
    card shown in full; the one-step `press` is kept for tools.
- **Data notes to route**:
  - q_a1_m4 has no `rewards` though its `reward` text promises a poultice and five days of Warded Tallow: add them or
    change the text. Its pick `loot_orlovs_retribution` is a skill; it now logs "`name` (skill for `member`)".
  - The flag_job templates have no `rec_level`, so jobs show no level.
  - The Crucible and Overlook lock flag `interlude_done` is set by a sequence, so the unlock words fall back to the
    quest's `closed_text`.
- **Hub cast**: seed `pr7_76_hub_cast` sets `art` for Sabina, Berek, Talia, Ivo and Widow Varka to their existing unit
  folders; their NPC portraits in `assets/portraits/npcs/` are unchanged.
- **Docs delta**:
  - `kits/rpg/README.md`: rpgkit_world gains `board::flag_place`, `travel::legend_ready` and `person_place`,
    `Dialogue::turn_in_index`, `GameState::wear_refusal`, `Data::family_name`, `campaign::view::end_line`; rpgkit_ui
    `vignette::Player::press_next` is the two-press player model.
  - GAME_BRIEF, Where to look: "A walk broken off by a fight resumes after a win: `region.rs` `resume_walk` /
    `Game::fought`." and "A directional exit preselects the place that way: `worldmap::exit_toward`."
  - HISTORY: "AO-01(4) world map roads 'focus only, nothing at rest' replaced by open roads drawn at rest (PT-45,
    PR7-76)." and "Vignette press model V3 (one phase per press) replaced for players by two presses a card, with no
    press lost in the fade (PT-46, PR7-76); `press` kept for tools."

### FB-390: PR7-78 ACT1-DATA1 built; job levels guessed, lock text already names the interlude

- **Status**: built, `develop` (squash `PR7-78`). Seed `pr7_78_quests.json`; goldens unchanged.
- **(a)** No change needed; FB-388 misread it. The poultice is already in `q_a1_m4`'s template `a1_main_raid`
  (`sulfur_poultice` 1), and the five days of Warded Tallow are the `orlov_stores` chest at the Tallow Vats (`supplies`
  5, opens on `sunk_ward.m4_done`), as the text says ("wait in Orlov's stores"). A first cut that added 5 supplies at
  turn-in (doubling the Tallow) was caught in review and dropped.
- **(b)** `rec_level` is a world quest field, so it is set on the eight quests using an `a1_flag_job*` template, by the
  place that sets the flag. `[PROVISIONAL]` levels (only crossroads 4 and junction 7 at 5 are shipped bands; the rest
  follow story order): `q_a1_corvo` 3 (Corvo's gate), `q_a1_red_pen` 4 and `q_hook_rubbings` 4 (crossroads),
  `q_hook_lock_ciphers` 5 (the Grate, no shipped band), `q_hook_bellows` 5 (junction 7), `q_hook_wedges` 6 (Fourth
  Kiln), `q_hook_counter_still` 6 (Tallow Vats), `q_hook_overlook` 7 (Crucible watchpost). `q_a1_m3`, `q_a1_m4` and
  `q_a1_m5` still have no `rec_level` (not in the row).
- **(c)** No change and no gap: the only lock text field is the quest's `closed_text`, and both `q_a1_m5` and
  `q_hook_overlook` already say "Something under the Undercroft must be answered first.", which the board shows since
  PR7-76.
- **Test**: `emberwake_world/tests/board_rec_level.rs` `every_flag_job_on_a_board_has_a_level` (data-derived).
- **Docs delta**: none.

### FB-394: PR7-79 DIORAMA-DRESS1 blocked; battle boards have no data lever for lamps or props

- **Status**: blocked (schema gap); nothing changed.
- **Spec**: `docs/READY.md` row `00b7` `PR7-79` (`DIORAMA-DRESS1`).
- **Finding**: the encounter schema (`kits/rpg/schema.json`) has `rows`, `exits`, `objective_tiles` and the battle
  fields, but no `props`, `decor` or lamps (`maps.json` `props` is region maps only). A blocked cell's cover art is
  picked by position (`stage.rs` `shown_cover` / `cover_art`, `art.pick(.., x, y)`), so `cover/v3` cannot be chosen per
  cell; a battle prop comes only from a terrain's `prop` (`tree`, `rubble`), and terrain ids are a closed set
  (`rpgkit_battle` `grid.rs` `TERRAIN_IDS`). Any change would alter walkability, move cost or cover, which the row
  forbids, so all 24 Sunk Ward battle boards are listed, not changed. `stage.fx.json` `lights` already lights any
  `cover/v3` or `lantern_pole` that does show.
- **Options** (a logic row first):
  1. (recommended) an encounter field `decor: [{art, x, y}]`, display-only for the rules, drawn by the stage with
     `lights` matched on the art key; the content check refuses exits, deploy cells and objective tiles. Lamps and
     props both, no golden change.
  2. new terrain ids copying `floor`/`stone` stats with a `prop`: a larger `TERRAIN_IDS` and terrain-table golden
     re-records.
  3. a per-cell cover variant pick for blocked cells: lamps and cover density only, no props on walkable cells.
- **Docs delta**: Art/UI `code_rows.md` R3(b) assumes lamps can be placed on battle boards; true for region maps only.

### FB-392: ENGINE-34 BONEMILL-CHECK1 report; Bone-Mill won by hand on try 1, Crucible lost 5 times

- **Status**: report only. The Bone-Mill is beatable by hand at Lv 7 on the first try; the auto plan still loses it.
  The Crucible (`enc_evt_06`, Slag Gate) was lost five times (1 auto, 4 hand), so the act's end was **not reached**.
  No debug was used anywhere, and no save was edited to skip the Crucible.
- **Spec**: `docs/READY.md` row `ENGINE-34`; follows `FB-441`.
- **Run**: develop `61ce531b`, `tools/fc.sh playtest start` and `aegis playtest do`, 1280x720, no window, 08:52 to 10:14
  UTC. The ENGINE-32 saves were lost with that session, so the whole run was hand-replayed from a new game, following
  the old log. Folder `/mnt/project-files/build-tools/act1_agent_run2/`: `run_log.md` (and `.jsonl`), 59 shots named by
  log step, saves `before_kiln`, `before_boss`, `after_bonemill`, `before_crucible`. Fights other than the Bone-Mill and
  Crucible tries B to E ran on the auto-filled plan.
- **How far**: prologue, Grate, Cages, Sweep, turn-ins, Fourth Kiln (patrol, Bone-Mill), Vats (Elyse joins), Corvo,
  the interlude (Fissure fight, debate), Crucible Watchpost (Overlook, picket), then five Crucible losses.
- **Tithing**: 42 after the prologue, 32 at the Kiln, 23 after the Vats and Corvo, 19 at the Crucible entry; no final
  count, since the act's end was not reached.
- **Bone-Mill** (Overseer Grendel; Kaelen, Vesper, Maren, Hollis Lv 7 plus guest Dunstan on AI):
  - Try 1, hand: **won in 4 rounds**. R1 Hollis's Winch Sunder and the slag vents killed both Dredgers; R2 phase 1
    broke, Vesper downed by a 386 crit (PT-67); R3 Kaelen's Sunder exposed Grendel, Hollis's crit Sunder and the vents
    finished him; R4 the Enforcer fell. Tries 2 to 5 not needed.
  - Comparison, auto plan from `before_boss`: Fled at R6, Grendel 142/304 in phase 2.
- **Crucible** (Inquisitor-Prelate Nemeth, from `before_crucible`, party Lv 8):

  | Try | Plan | Holder | Result | What happened |
  | --- | --- | --- | --- | --- |
  | A | auto | Vesper | Fled R9 | Nemeth untouched, nobody out |
  | B | hand | Vesper | Fled R7 | Maren out R3, Hollis R4; Vesper downed R5, the gate shut |
  | C | hand (R4 on, auto) | Kaelen | Fled R9 | the R4 spawn ate Kaelen's move (PT-71); auto walked the holder off |
  | D | hand (R5 auto, a slip) | Vesper | Fled R6 | Kaelen and Maren out R4; Vesper downed R5; Hollis 1 tile short |
  | E | hand (R2 auto, PT-79) | Kaelen | gave up R3 | auto R2 left Vesper in melee, downed |

| PT | Sev | Area | Finding | Shots |
| --- | --- | --- | --- | --- |
| PT-66 | major | save | "Autosave failed: rolls: 'retry:sunk_ward:enc_evt_03' must be a stream id with a count >= 0." after every lost story fight (PR7-75's retry key); the post-loss state is never saved | steps 1242, 1851, 2279 |
| PT-67 | major | forecast | The boss blow read "Hammer Strike block 3: -0" for stealthed Vesper, who then took a 386 crit after disengaging one tile | log ~1000-1100 |
| PT-68 | major | stealth | Ash-Cloud Fade "readies stealth", yet the hero is hit that round and the next (Grendel's jabs; four Heretic's Brand hits on the winch holder, Brand also ends her guard) | 1907, 3171 |
| PT-69 | major | skills | Winch Sunder did not break the Pulverizer wind-up ("0 of 3 into wind-up"); it landed after a 2-tile push, on Maren with no incoming forecast | log ~1124 |
| PT-70 | major | movement | A move cut short re-routes the wrong way: Kaelen (5,4) to (4,2) then (3,4); Hollis (5,4) to (5,6) and to (4,4) | 2088 |
| PT-71 | major | Crucible | R4 reinforcements spawn on the open middle gap (6,4), (7,4) on claimed tiles; the planned move drops with no log line | 2541 |
| PT-72 | major | planning | A planned move claims every path tile, so a 1-tile gap carries one hero a round across blocks; auto ghost plans claim too; a later pick voids an earlier move, said only at confirm; odd path costs ((7,4) 4, (8,4) 9); "No path" through a foe leaving in B1 | log 2852-2894 |
| PT-73 | major | balance | Crucible not won in 5 tries at Lv 8: the suggested holder (232 HP) draws both Purifiers and a Footman, the gate shuts when the holder falls, and 2 of 3 out reads plain "Fled", like a wipe | 2273, 3171 |
| PT-74 | major | auto plan | The auto plan ignores the escape objective: walks the holder off the winch, kites Hollis away from the exits, keeps Vesper in melee; try A never touched Nemeth | 1844, 3247 |
| PT-75 | minor | UI | After the R4 time phase the boss bar reads "This phase 346/346, In all 484/692" while phase 1 still had 138; is it still owed? | 2154 |
| PT-76 | minor | UI | The Hostiles list names Tallow Dredgers "Skirmisher" | 846 |
| PT-77 | minor | board | No Lv on the Bone-Mill, Vats and Crucible main quests (`q_a1_m3..m5` have no `rec_level`) | 308, 1673 |
| PT-78 | minor | quests | The Crucible lock hint does not name the Bone-Mill and Vats turn-ins; the Wedges job unlocks at the Kiln but is taken only at the hub, 3 days away | 1673, 1301 |
| PT-79 | minor | input | Enter at "Round over" confirms the next round (cost try E); a party slot click toggles selection; a third Esc opens Settings | 3247 |
| PT-80 | minor | skills | Ally heals cost far more (Tourniquet 4 blocks/8 AP on an ally vs 2/3 on self; Cauterize 3/7 vs 1/2); after a Move pick rows show "-3 blocks, -3 AP" | 1054 |
| PT-81 | minor | UI | "Maren has already left the battle." shows while planning an exit | log 2460 |
| PT-82 | minor | lore | Dunstan's jaw is "wired shut", yet he speaks a battle line | 829 |
| PT-83 | minor | map | The Watchpost marker "The gorge winch drum" has no Here row | log 1727 |
| PT-84 | minor | log | "Bailiff Enforcer drags Kaelen 0 tiles" | log ~1150 |

- **Notes**: Vesper's max HP fell from 222 to 186 after being downed (an injury?); Maren's picks may have been empty at
  confirm in Crucible B R1 (low confidence).
- **FB-441 items re-checked as fixed**: PT-43, 44, 45, 46, 49, 51, 53, 55, 56, 60, 61, 63; PT-40 partly (the boss HP
  hover explains the phase span; the auto plan still loses the Bone-Mill).
- **Docs delta**: none.

### FB-396: PR7-81 DECOR1 built; decor key format for the asset lane, five guesses

- **Status**: built, `develop` (squash `PR7-81`). Goldens unchanged (sim, checks, rules contract, journey, resave).
- **Key format**: `decor` is a list of `{"art": KEY, "x": 0-11, "y": 0-8}` on a catalog or world encounter, omitted
  when empty. KEY is `cover/v<N>` (that cover variant of the board's set, zone set first, then the world's
  `stage_set`), `cover` (the cell's pick), `<prop>/v<N>` or `<prop>` (`props/common/<prop>/` first, then the other prop
  sets in name order). Catalog encounters (set "") cannot use cover keys. `stage.fx.json` `lights` match on the
  resolved key, so `cover/v3` and `lantern_pole` decor cast their pools.
- **Check** (`rpgkit_content` `check::encounter::decor_problems`): refuses a non-list, unknown keys, off-grid cells,
  exits, the cells the listed units deploy on, hold tiles and the escape objective's stay-behind tiles, two decor on one
  cell, and art the board's set does not have.
- **Guesses `[PROVISIONAL]`**:
  1. A cover-key decor that resolves to art on an inner block replaces that block's cover (drawing and light); prop
     decor on a block, and any decor on an outer wall's cap, draws raised on top of the block.
  2. Deploy cells are the cells the listed party, guests and enemies take; a world fight whose heroes come from the
     player's party may deploy onto decor, which units draw over.
  3. Props draw at their PNG size (`lantern_pole` is 64 x 112), so the asset lane may prefer `cover/v3` for lamps.
  4. Cover decor takes the zone tint for its terrain; props none, like terrain props.
  5. Decor under the boss band is skipped like blocks.
- **Tests**: `rpgkit_content/tests/decor.rs` (schema, key forms, writer round trip, board unchanged, refusals, world
  check, cover sets); `stage::tests` decor drawn on inner and outer blocks, a lamp decor's pool, walkability unchanged.
- **Docs delta**:
  - Kit README: "DECOR1 (PR7-81): encounter field `decor: [{art, x, y}]` (catalog and world, omitted when empty),
    display-only; `rpgkit_battle::encounter::decor_of` / `EncounterDef.decor`; `rpgkit_content::art::decor_art`,
    `decor_known(layers, key, cover_sets)`, `decor_cover_sets`, `decor_prop_dirs`, `stage_fx`;
    `check::encounter::decor_problems`; `World::deploy_cells`."
  - STRUCTURE: `kits/rpg/crates/rpgkit_content/tests/decor.rs` (DECOR1 tests).
  - GAME_BRIEF, Where to look: "Battle board decor (lamps, props, display-only): encounter `decor`, drawn by
    `emberwake_gui/src/stage.rs` `decor_items` / `draw_decor` / `lamps_with`; never changes walkability, cover or line
    of sight."

### FB-398: PR7-84 ACT1-DATA2 built; the winch drum row is a flag workaround

- **Status**: built, `develop` (squash `PR7-84`). Seeds `pr7_84_m3`, `_m4`, `_m5`, `_overlook`, `_winch`, `_dunstan`.
- **(PT-77)** `rec_level` `[PROVISIONAL]`: `q_a1_m3` (Bone-Mill) 7, `q_a1_m4` (Vats) 7, `q_a1_m5` (Crucible) 8, from
  the ENGINE-34 party's levels, one above the Kiln and Vats jobs (6) and the Overlook (7).
- **(PT-78)** `q_a1_m5` and `q_hook_overlook` share the lock `sunk_ward.interlude_done`, set by the interlude after
  `enc_evt_05`; its trigger `t_interlude` needs `m3_turned_in`, `m4_turned_in` and the two joined flags those fights
  set. Both `closed_text` now read "Turn in the Bone-Mill and Vats to Talia. The Undercroft stirs then.". The Wedges
  job at the hub (also PT-78) is not touched.
- **(PT-83)** The marker `winch_post` had no `use` and no `sets_flags`; the `use` values are rest, crypts and stash
  only. It now sets `sunk_ward.winch_seen` (read by nothing), so it is listed in Here. A look-only point would be a
  logic row (a new `use` or a listing rule), if wanted.
- **(PT-82)** Dunstan's round-1 line in `enc_evt_03` (`dlg_talk_m3_grendel`) now reads "Dunstan grinds his wired jaw
  and hammers the slag-ladle against the chute rail, twice, hard." in the plain narration style of his other lines.
- **Test**: `emberwake_world/tests/board_rec_level.rs` `every_flag_job_on_a_board_has_a_level` now also covers main
  raids.
- **Docs delta**: none.

### FB-400: PR7-79 DIORAMA-DRESS1 built; roam and patrol boards not dressed; arena path fix

- **Status**: built, `develop` (squash `PR7-79`). Seeds `pr7_79_alley`, `_rooftops`, `_canal`, `_foundry`, `_vats`,
  `_undercroft`, `_watchpost`; goldens unchanged (sim, checks).
- **Boards** (6 to 9 pieces each): `enc_pro_0a/0b/0c`, `enc_job_corvo_gate`, `enc_evt_01` to `enc_evt_05`,
  `enc_evt_06` and `enc_evt_06_overlook` (one layout). `cover/v3` lamps and `cover/v0`/`v1` on inner blocked cells where
  a board has them; `lantern_pole` only at edge columns, back rows or the top row; props (`barrel_stack`, `crates_tall`) at
  the edges too, kept off the fight lane (the check refuses exits, deploy cells and objectives); a first cut with poles
  in the fight lane was redone.
- **Listed, not changed**: the 12 roam, scav and patrol encounters have empty `rows` (generated boards), so decor has
  no fixed cells there. Dressing them needs a rule (decor by generator) if wanted.
- **Fix found on the way**: an arena launch with `scene=<world>:<id>` and no `ruleset` (`BattleScreen::sandbox`) read
  the whole string as the encounter id, so the board had no stage set and drew no cover art at all (blocks or decor).
  `sandbox()` now splits world and id like `arena_with` and `playback`. World fights were never affected. Test
  `battle/tests/decor_tests.rs` opens every shipped encounter with cover decor through both paths.
- **Guess**: `cover/v3` on a walkable cell draws (cover size) and is not refused by the check.
- **Shots**: `/mnt/project-files/pr7-79/` before and after for `enc_pro_0a`, `enc_evt_03`, `enc_evt_06`, plus
  `enc_evt_06_after_fix.png`.
- **Docs delta**: STRUCTURE: `emberwake_gui/src/battle/tests/decor_tests.rs` (shipped decor draws on both arena paths).
  GAME_BRIEF, if wanted: an arena launch `scene=<world>:<id>` now opens with that world's stage set, music and talks.

### FB-402: PR7-86 DECOR-GEN1 built; the pool is a world manifest field, props stay at the back

- **Status**: built, `develop` (squash `PR7-86`). Goldens unchanged (sim, checks, timeline, journey, phasefold, store,
  rules). Shots `/mnt/project-files/pr7-86/` (`enc_roam_watch` before and after).
- **Finding**: there is no board generator; the 13 Sunk Ward roam, scav and patrol fights have empty `rows`, so
  `Grid::apply_board` lays out a plain 12 x 9 floor. The rule runs at session start instead.
- **List**: a kit world manifest field `decor_pool` (strings, decor keys as in FB-396, omitted when empty, the nearest
  parent world's pool when a world has none), checked by `check::encounter::decor_pool_problems`. Seed
  `pr7_86_decor_pool` sets Sunk Ward's to `cover/v3`, `lantern_pole`, `barrel_stack`, `crates_tall`. `stage.fx.json`
  is untouched.
- **Rule** (`rpgkit_battle::encounter::generated_decor`, called in `Session::start_with`): only for an encounter with no
  `rows`, no `decor` and a pool; 2 to 6 pieces on a stream hashed from (battle seed, "decor", zone), so no other draw
  moves and a replay draws the same; cover keys on free blocked inner cells, else any edge cell; prop keys only on back
  cells (row 0, or columns 0 and 11 in rows 0 to 2), so nothing tall stands in the fight; never exits, start cells,
  hold or stay-behind tiles, nor two on one cell.
- **Guesses `[PROVISIONAL]`**: edge means the outer ring; `cover/v3` falls back to an edge floor cell (these boards have
  no inner blocks); later arrivals may land on decor and draw over it; "generated" means empty `rows`.
- **Tests**: `rpgkit_battle/tests/generated_decor.rs` (7), `rpgkit_content/tests/decor.rs`
  `a_world_checks_its_decor_pool`, `emberwake_world/tests/generated_decor.rs`, `emberwake_gui` `decor_tests`
  `generated_boards_draw_their_world_decor_through_both_arena_paths`.
- **Docs delta**:
  - Kit README: "DECOR-GEN1 (PR7-86): world manifest `decor_pool`; a generated board (no `rows`, no `decor`) draws 2 to
    6 display-only pieces at session start (`rpgkit_battle::encounter::generated_decor`, own seeded stream; cover keys
    on blocked inner cells else any edge cell; prop keys on back cells only, `is_back_cell`, `BACK_SIDE_ROWS` = 2);
    `EncounterDef.decor_pool` from `rpgkit_world::battle::decor_pool`; check `decor_pool_problems`."
  - STRUCTURE: `kits/rpg/crates/rpgkit_battle/tests/generated_decor.rs`,
    `games/emberwake/crates/emberwake_world/tests/generated_decor.rs`.
  - GAME_BRIEF, Where to look: "Decor on generated boards (roam, scav, patrol): the world manifest's `decor_pool`,
    placed by `rpgkit_battle::encounter::generated_decor` (props at the back only); display-only."
- **Tooling note**: the disk filled twice from stale test binaries in `target/debug/deps` (an `encounter.rs` change
  rebuilds every downstream test); a `tools/fc.sh` step that keeps only the newest binary per name would help (tools
  lane, if wanted).

### FB-404: PR7-85 XL-FIX2 built; FB-455's strip overlap and tile spills no longer showed

- **Status**: built, `develop` (squash `PR7-85`). Normal shots unchanged (`tools/fc.sh shots diff base=HEAD`: 128
  unchanged, `screens_menus__scene_runner` differs only by the temp data path). Shots `/mnt/project-files/pr7-85/`
  (Extra large and Large, plus `audit_xl_after.txt`).
- **Finding**: the strip overlap and tile spills FB-455 listed for (a) and (d) no longer show on today's develop; both
  are done as asked anyway.
- **What landed**:
  - (a) the battle strip title is "Round 1 of 12" at Small and Normal, "R 1/12" above (hovering to the long form); the
    goal line sits by the title's height in its display face.
  - (b) Character Stats and Gear rows step at least a line plus one unit apart at every size; a list that has no room
    takes a second column (`party.rs` `stat_rows_fit`); a label cut there hovers in full (none cut in today's data).
  - (c) the XL sweep found six cuts with no hover (the Stats passives line, the Skills source hint, shop item names);
    all hover now, and the sweep shows 0 (`audit.txt` marks NO HOVER).
  - (d) engine `Screen::audit_ignores()` (widget id prefixes the text audit does not treat as controls; the battle
    screen names `tile.` and `unit.`); each cut reports `hover`, the reply counts `bare_cut`; `panes::list_row` hovers a
    cut name when the caller gave no card.
- **Guesses `[PROVISIONAL]`**: the short form for any size above Normal; "grow" means a line plus one unit with a
  two-column fallback; unit tokens are ignored by the text audit like tiles. Caveat: at Normal a list of 12 or more
  rows would also take two columns (today's data stays in one).
- **Tests**: `aegis_ui/tests/text_audit.rs` (two new), `tests/panes.rs` (one new), `battle/tests/xl_battle_tests.rs`
  `the_strip_title_shortens_at_large_text_and_clears_the_goal`, `party.rs` `xl_rows_tests` (three).
- **Docs delta**:
  - engine/README.md, after the XL-SWEEP1 bullet: "**Audit ignores and cut hovers (XL-FIX2, PR7-85):**
    `Screen::audit_ignores()` names widget id prefixes the text audit does not take as controls (the battle screen
    names `tile.` and `unit.`); the layout audit is unchanged. Each audited cut says whether a widget at it hovers to its
    full words (`hover`), and `bare_cut` counts those that do not; `audit.txt` marks them NO HOVER. `panes::list_row`
    hovers a cut name when the caller gave none. `layout_audit::frame_report` takes the ignores."
  - engine/README.md, the link's `text_audit` line: "boxes in `Screen::audit_ignores` are not controls; each cut
    carries `hover` and the reply `bare_cut`".
  - GAME_BRIEF (optional): "XL-FIX2: the battle strip reads 'R 1/12' above Normal text; Stats and Gear rows keep a
    line apart at every size, taking a second column when needed."
  - EFFICIENCY: `fc.sh scenario` rebuilds the binaries first, so a "before" sweep must be taken before any edit;
    `shots diff` runs `cargo clean` on the workspace crates.

### FB-406: ENGINE-35 ACT1-END-RUN1 report; Crucible won by hand on try 3, act end reached at Tithing 19

- **Status**: report only. The Crucible (Inquisitor-Prelate Nemeth, escape with a winch holder) was **won by hand on
  try 3** in 7 rounds at party Lv 8. **No debug was used** and no save was edited. Try 1 ran rounds 8 to 12 on the auto
  plan after the holder fell, only to reach the loss screen for the PT-66 check; try 2 was given up at R6 by reloading
  the entry save, once 3 out could no longer happen. Everything else was hand-played.
- **Spec**: `docs/READY.md` row `ENGINE-35`; follows `FB-392`, with the logging asks of `FB-459` (PT-71) and `FB-463`.
- **Run**: develop `b24d2dee`, `tools/fc.sh playtest start save=` (the `before_crucible` save of act1_agent_run2, which
  loads on this build) `keep_saves=` `shots=off`, then `aegis playtest do`; 1280x720, no window, 17:23 to 18:17 UTC.
  Folder `/mnt/project-files/build-tools/act1_agent_run3/`: `run_log.md` (section 2 has every battle line in full from
  `state.log_groups`, per round, for all three tries), `run_log.jsonl`, 47 shots named by log step,
  `441_tree_odd_costs_A_R2.json` (the whole board at PT-88), `saveshot_ghost_claim_14.png` (a `save=` proof shot), and
  saves `crucible_entry`, `act1_end` (right after "Act 2 begins") and `slag_gate`.
- **How far**: Crucible won, the end vignettes `vg_m5_gate` and `vg_debate_3`, "Act 2 begins" at the Sluice Undercroft,
  the Crucible turned in to Talia, then travel to the Slag Gate.
- **Tithing**: 19 at the Crucible entry and **19 at the act's end** (the win ends the act; no day passes); 15 after the
  4-day trip to the Slag Gate (PT-97). Alert 12% to 30%.
- **Crucible tries** (all from `crucible_entry`; deterministic, R1 the same each time):

  | Try | Holder | Result | Rounds | What decided it |
  | --- | --- | --- | --- | --- |
  | 1 | Kaelen | Partly escaped, 1 of 3 out | 12 (hand R1-7, auto R8-12) | Vesper out R5; from R4 Footmen 3, 4 and Nemeth guarded all three column-6 gaps, so Maren and Hollis were cut short in R6; Decapitation crit Kaelen for 415 in R7 (forecast -277); two Pommel Strikes in its windup broke nothing (PT-89) |
  | 2 | Kaelen | given up R6 | 6 | Vesper out R5; Maren (89 HP) cut short at (5,3) when a Purifier stepped in front (Hollis's move took (4,4) first); Brands killed her in R5 |
  | 3 | Kaelen | **Victory** | 7 | Hollis killed Purifier 1 in R1 (Sunder 145 + Quarrel 99) and with Kaelen Footman 2 in R2; Kaelen on the winch R3 B2; Maren ran the north lane in R4 and got out R5; Vesper (stealth) R6; Hollis R7 at 7 HP |

  Results: "Kaelen held the winch. Kaelen was captured." His keepsake goes to the stash and Act 2 opens without him.

| PT | Sev | Area | Finding | Shots |
| --- | --- | --- | --- | --- |
| PT-85 | major | forecast | Hit forecasts on a target that moves disagree with the result: Decapitation on Hollis after his move showed -209 at (1,3), 5 tiles from Nemeth, and nothing at (2,4), 3 tiles (try 2 R2); Maren moving to (5,5) showed no incoming while a Footman and a Purifier targeted her, she took 151 (try 2 R4); Maren showed -138 and took 0 (try 3 R1) | 1415, 1763 |
| PT-86 | major | planning | A ranged pick on an adjacent foe adds an approach walk 4 tiles away from the fight and the exits (Winch Sunder walk to (3,1), try 1 R3; Barbed Quarrel walked Hollis from (5,7) to (1,7), away from his exit, try 3 R5) | 539 |
| PT-87 | major | planning | Foe start tiles and ghost plans still block routes (rest of PT-72): a foe's start tile counts as blocked though it leaves in B1, so the only gap reads "Too far: 12-18 tiles"; auto ghost plans still claim tiles ("Too far: 14 tiles" for Kaelen to the winch in R1, 8 after Clear; after Clear "Claimed by Vesper's move") | saveshot_ghost_claim_14 |
| PT-88 | major | planning | Kaelen's Billhook Sunder on an adjacent Footman (no move) refused: "Blocked by Maren's plan: Move here dropped: Too far: 8 tiles > 6"; Maren's costs from (3,5) ran above their step counts ((3,7) 4, (4,7) 5, (5,7) 6, (7,7) 8), and (7,7) had been accepted at 6 before the other picks | 312, 441_tree json |
| PT-89 | major | combat | Two Pommel Strikes landed in B1 and B2 of Nemeth's B1-3 Decapitation windup; the summary read "no breaks (0 of 3 hits into a wind-up)" with no reason; the blow then crit for 415 against a forecast of -277 | 1005 |
| PT-90 | major | story | After the win the log shows "Flag 'sunk_ward.sabina_dose_to' names no party member; nothing is installed." (guess: the dose went to Kaelen, now captured) | 2843 |
| PT-91 | minor | planning | Auto-fill after hand picks repeats them and spends more: Ash-Cloud Fade ran in 4 blocks, the strip said "AP -3" yet Vesper ended at 1/10 AP; a second Tourniquet healed 5; a filled Strike spent Ammo | 183 |
| PT-92 | minor | log | The R3 reinforcements (Footmen 3 and 4) arrive with no log line; wording "Nemeth is Burning by Floor" | log A R3 |
| PT-93 | minor | log | A Purifier at 16 HP walked out through the party's exit, logged like a hero ("safely WITHDREW through Exit!"); the Hostiles list keeps it at 16/200 | 1066 |
| PT-94 | minor | text | Double words: "exposes the target for 3 rounds rounds" (Billhook Sunder); "dash up to 3 tiles tiles" (Hamstring Sever) | run_log notes |
| PT-95 | minor | results | A win goes straight to the results with no banner (a loss shows one); Dunstan, not in the fight, is listed with empty stats and +35 XP; captured Kaelen gets no XP line | 2690 |
| PT-96 | minor | map | The Slag Gate's "The raised portcullis" marker has no Here row (like PT-83); its art shows the bars down while the text says it was lifted | 2904, 2911 |
| PT-97 | minor | flow | The act end moves the party to the Undercroft, but the new road starts at the Watchpost, so the Slag Gate costs 4 days (Tithing 19 to 15) and 4 supplies | 2895 |
| PT-98 | minor | UI | The journal's Back button sits on top of the last quest rows | 2850 |
| PT-99 | minor | UI | A unit hover tooltip from before the end stays over the battle-over banner and hides its title | 1063 |
| PT-100 | minor | lore | In `vg_debate_3` Sabina says "I watched" about a fight she was not at; Talia's "Eighteen, twenty-four, cowled to the eyes" reads oddly | run_log section 3 |

- **FB-392 items re-checked** (PR7-82, PR7-83 parts, PR7-84, PR7-87 on develop):
  - Fixed: PT-66 (no "Autosave failed" after a loss), PT-68 (attacks "lose" a stealthed hero who moves), PT-73 (results
    "Partly escaped", "1 of 3 got out; the escape needed 3."), PT-75 (boss bar "346/346 +330" with a carried-over
    hover), PT-77 (Crucible Lv 8 on the board), PT-83 (winch drum in Here).
  - Partly fixed: PT-72 (a pick that would void a move is refused with a reason; see PT-87, PT-88); PT-69 (Winch Sunder
    broke a Footman's Heavy Thrust; the Pulverizer not met); PT-79 (slot clicks always select; one Esc at "Round over"
    opened Settings); PT-80 (negative costs after a Move pick remain: "Ash-Cloud Fade -4 blocks, -7 AP").
  - Not reproduced: PT-70, PT-71 (every cut walk logged "[Move Cut Short]"), PT-81, PT-84.
  - Unchanged: PT-74 (on the auto plan Maren and Hollis were downed with no escape attempt).
  - Not re-checked (Bone-Mill and earlier boards not replayed): PT-67, PT-76, PT-78, PT-82.
- **Not done**: odd costs could not be kept as a mid-battle save (saves are the campaign slot); the tree file captures
  the R2 board. The Overlook and Wedges jobs remain open; Act 2 not played past the Slag Gate.
- **Docs delta**: none.

### FB-408: PR7-89 ACT1-FIX4 built; the win banner did not reproduce; three questions

- **Status**: built, `develop` (squash `PR7-89`). Goldens unchanged (sim, rules, contract, checks, journey, PHASEFOLD,
  timeline, resave). Seeds `pr7_89_text`, `pr7_89_portcullis`, `pr7_89_road`. Shots `/mnt/project-files/pr7-89/`.
- **PT-90** `[PROVISIONAL]`: the dose stays with the captured hero and returns with him. `dlg_int_dose` ends "The vial
  goes down a throat", and `vg_m5_gate` says Nemeth "took the stayer alive"; nothing passes it on. `install` on a flag
  naming a departed member grafts into the departed record (`rpgkit_world::dialogue::departed_from_flag`); the note
  reads "Kaelen is captured; Optic Graft stays with them and returns with them.". There is no rejoin yet.
- **PT-92**: reinforcements log "[Wave] `label`: A, B join the battle."; hazards log "[Hazard] X is caught in Burning
  Ground: Burning" (`command::terrain_label`, the terrain row's name or its id in words).
- **PT-93**: a foe leaving by an exit logs "[Escape] ... escaped through the exit." and leaves the Hostiles list
  (`Session::foe_escaped`); a surrendered foe stays listed.
- **PT-94**: the `{rounds}` and `{dash}` tokens already add their unit, so nine skill texts doubled it; the trailing
  words are dropped in data. No other doubled word found.
- **PT-95**: encounter XP goes to the members who fought (`progression::award_fielded_pct`; Dunstan's +35 came from the
  bench filling the captured hero's slot); a captured hero keeps his XP in his departed record and has a results row;
  a member not in the fight has none. **The win banner did not reproduce**: a won world fight plays the "Victory" beat
  for about 1.8 s, then opens the results; the run's wait swallowed it (`win_banner.png`).
- **PT-96**: the Slag Gate marker uses `use: "look"`; there is no raised-portcullis art, so the text now matches the
  lowered bars: "The Slag Gate portcullis", "The winch has been turned and the road is cleared. Beyond the
  chain-teeth, it climbs toward Upper Cinderreach.". Raised art would be an Art/UI request.
- **PT-97**: route `high_gorge` now runs from `sluice_undercroft` (where `seq_after_m5` leaves the party) to the Slag
  Gate; it no longer starts at the Watchpost. A second route from the Watchpost would be a design call.
- **PT-98**: the Journal list stays above Back and scrolls (selection kept in view, wheel, "more above/below" marks).
- **PT-99**: when the outcome panel shows, a veil over the board stops stale hover cards covering the banner.
- **Questions**:
  1. Should a win hold its Victory banner until a click, as a loss does (TRANSITIONS1 T2)?
  2. Should the round banner hide during the Victory beat? It draws over it now.
  3. Should blocker and forced-move lines also name the terrain instead of "Floor"? That changes log text.
- **Guess**: benched members no longer get XP from a fight they were not in. Fights settled without a battle (the default
  tally) still pay every active member; a member who died in the fight gets none; `install` on a flag naming a dead
  member installs nothing, with a note.
- **Docs delta**: GAME_BRIEF, combat log: "Reinforcements log `[Wave]`; terrain hazards `[Hazard] X is caught in
  Terrain: Status`; a foe leaving by an exit logs `[Escape]` and leaves the Hostiles list (`Session::foe_escaped`)."
  Rewards: "Encounter XP goes to the members who fought (`progression::award_fielded_pct`); a captured member keeps his
  XP in his departed record." Story effects: "`install` on a flag naming a departed member grafts into the departed
  record (`dialogue::departed_from_flag`)."

### FB-410: PR7-91 ACT1-FIX5 built; the banner hides for every outcome panel

- **Status**: built, `develop` (squash `PR7-91`). Goldens unchanged (none stores blocker or forced-move terrain text).
  Shot `/mnt/project-files/pr7-91/victory_banner_no_round_banner.png`.
- **What landed**:
  - (a) `playback.rs` `round_banner_now` returns nothing once the battle has an outcome: the Victory beat and every
    outcome panel. A new battle starts its own banner. A win keeps its 1.8 s beat and opens the results by itself.
  - (b) `rpgkit_battle::command::terrain_name` now uses `terrain_label` (the row's name, else its id in words), with
    "Floor" only for a plain floor; `forced.rs` `obstacle()` uses it, so pushes read "the rubble", "the lava".
- **Guess `[PROVISIONAL]`**: the banner hides for every outcome panel (Fled, Out of time too), since each covers the
  board; Victory and Defeat only is a one-line change.
- **Tests**: `rpgkit_battle/tests/terrain_rows.rs` `blocker_and_forced_move_lines_name_the_terrain`;
  `transitions_tests` `the_round_banner_hides_once_the_battle_is_over`; `story_battle_hub.json` checks
  `state.round_banner` is absent before the Victory shot.
- **Docs delta**: GAME_BRIEF, combat log: "Blocker and forced-move lines name the terrain by its display name
  ('blocked by Lava', 'hits the rubble'); a plain floor reads 'Floor'. The round banner hides once a battle has an
  outcome."
