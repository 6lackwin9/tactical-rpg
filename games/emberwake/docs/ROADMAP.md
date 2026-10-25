# Roadmap (agent-facing, high level)

Transition (2026-10-19): the game now runs on the Aegis engine (Rust, `engine/`); `done` systems were built in the Godot build and reached step-for-step parity in the engine (`engine/docs/PORT_NOTES.md`, exceptions in `games/emberwake/docs/SCREENS.md` "Known gaps" and the balance sim's unported modes). Engine milestones and the transition itself: `engine/docs/ENGINE_SPEC.md`.

Single source of truth for scope and status. Detailed per-system checklists remain in README "Roadmap". Do not delete entries: change the status and add the reason (see `docs/HISTORY.md` for deprecated ideas). Decided brainstorm options, answered questions and done plans (2026-10-15 brainstorm pass, plan v2, R1) are archived verbatim in `docs/archive/ROADMAP_2026-10.md` (CLN-11).

**Status values**: `done` (implemented, tested) | `balance-ready` (done and frozen: the balance agent may fine-tune it) | `in-progress` (a milestone is building it) | `approved` (milestone approved by the user, not started) | `proposed` (idea from brainstorm, not approved) | `placeholder` (folder reserved in `docs/STRUCTURE.md`, nothing built) | `deprecated` (kept for history; see HISTORY.md).

Only the user moves an item to `approved`. Only the planner (main session) sets `in-progress` and `done`. Only the user/planner set `balance-ready`.

## Systems

| # | Area | Status | Depends on | Notes |
| --- | --- | --- | --- | --- |
| S1 | Combat core: block resolver, planner, stances, AP, damage formula, statuses | balance-ready | - | `DamageCalculator` is the only formula |
| S2 | Classes, units, tiers, proficiency, passives, skill paths (six classes) | in-progress (M2c-1 roster, M2c-2a1 families/loadout, M2c-2a2 engine pieces, M2C2B content, SK2 level unlock, SKILL1 ranked skills, BAL2-L and BAL2-A done; BAL2-R rerun and BAL3 next, goal G3) | S1, S12 | Loses balance-ready until the rework is built and re-frozen |
| S3 | Gear and inventory: requirements, tiers, burden, carry, stash | done (gear budget guard added in M2d) | S1 | Gear budget per class is an open balance lever |
| S4 | Enemy AI (roles, presets, targeting) | done (M1: kiting, no idle turns, support heals, reckless profile) | S1 | Shaman/ranger AI wastes turns: candidate fix |
| S5 | Content catalog + Content Forge | done | - | Data pipeline for S2/S3/S6 |
| S6 | World engine + World Forge (lore layer, inheritance) | done | S5 | Characters, factions, items, quests as data |
| S7 | Party and progression: persistent unit state (spend, learned skills, passives), level-up UI, XP | done (M3 data model; level-up, party, loadout and gear screens in UI2-L) | S2 | |
| S8 | Quests and lore runtime: quest state, rewards granting passives/skills/items, flags | done (M4-L, M4-A, QUEST2: quest runtime, flag registry, world loot table, ready flag) | S6, S7 | Sources of passives |
| S9 | Region map and exploration: tiles, locations, encounters triggers, interactions | done (EXP1-L, EXP1-A; map fights still end in debug outcome buttons until BATTLE1, goal G1) | S6, S8 | README System 2; inputs below |
| S10 | World map and travel: routes, travel events, supplies | TRAV1 done; TRAV2 supplies approved (TRAV2-L, TRAV2-A) | S9 | README System 1; inputs below |
| S11 | NPCs and dialogue: dialogue trees, shops, factions reputation | in-progress (DLG1 dialogue and shops done; DLG1-F open; REP1 reputation approved) | S8 | README System 9; inputs below |
| S12 | Advanced combat: combos, terrain effects, status expansion, skill upgrades | in-progress (S12-A engine done in M1: ally-target support skills, gap-close, haste/bleed/taunt; riposte done in M2a; S12S status slice done; S12T-L terrain rules, S12T-A terrain rows, S12C-L combo engine and S12C-A combo data on develop) | S1, S2 | README System 6 |
| S13 | Save/load and game state, new-game flow, main menu | done (M3 schema and save I/O; UI1-L menus, 5 shared slots + autosave, SAVE-IO export / import) | S7 | |
| S14 | Audio, animation and asset pipeline, asset-load tests | in-progress (S14-L1, S14-L2 presenter, S14-L3 audio, S14-A1, S14-A2 done; MUSIC1 music by scene context, games/emberwake/docs/specs/MUSIC1.md) | - | README Test Suite 2 |
| S15 | Balance tooling and reports | done | S1, S2 | `tools/balance_sim.gd`, `games/emberwake/docs/BALANCE_REPORT.md` |
| S16 | Playtest tooling (debug only): recordings, notes, playback, headless replay | approved | S1 | Feedback loop desktop -> cloud; battles exact, overworld trail-only until PLAYTEST2. On the engine (PORT-PLAYTEST): arena battles record and replay; party battles of the world map wait for PLAYTEST2, so G1's "one full run replays headless" needs it too |
| LEVERS1 | Role-feel levers: brawler spin, priest Ward (DR2 4B) | approved (user 2026-10-18 14:26) | BAL5-L, BAL5-A | `games/emberwake/docs/specs/LEVERS1.md`; Bodyguard deferred, boss acts twice not built |
| DERIVE1 | Data-defined derived stats (an engine formula evaluator, kit derived-stat rows, resist checks); first row Composure, the corruption resist | proposed | - | `docs/specs/DERIVE1.md`; user 2026-10-25 (Q-47a) asked for the capability in the engine and kit; DQ1-DQ5 open |
| ENCOUNTER2 | Encounter structures: authored terrain, hold-a-tile escape, impossible fight and last stand, boss phase kill / swap / rogue side, linked rooms | proposed | - | `docs/specs/ENCOUNTER2.md`; user 2026-10-25 (Q-60, Q-75, Q-75a); slice needs T, H, HA, W, U, K, D; EQ1-EQ9 open |
| RULESET1 | The "timeline" combat ruleset (M1 feel prototype): ruleset world field and Dojo toggle, windups, recovery, chains, ap_cost, poise and breaks, Active Guard, pierce, reach 2, deck slot kinds, plan visibility; the classic ruleset byte-identical | proposed | DERIVE1 | `docs/specs/RULESET1.md`; user 2026-10-25 (combat group 1.1); TQ1-TQ11 open |
| CAMPAIGN1 | Campaign meters (Alert, the Doom Clock), checkpoints and bad ends, the run-over screen, purse and upkeep names, rests that cost days, defeat retreat, event triggers; off in worlds without a `campaign` field | in-progress | - | `docs/specs/CAMPAIGN1.md`; user 2026-10-25 (slice M0) |
| KIT1 | Kit extraction (kit crates, the Ashlands as the kit's sample) | in-progress | - | M0: genre rules into `kits/rpg/crates/rpgkit_*`, the `emberwake_*` crates become façades with game code and Emberwake's tests, the schema to the kit, a layer check in the gate; the Ashlands stay the kit's sample and parity world (classic); spec `docs/specs/KIT1.md`; KQ1-KQ8 open |
| STORYUI1 | Vignette player (engine), story sequences and the new-game flow, the mission board (kit) | in-progress | KIT1 | `docs/specs/STORYUI1.md`; user 2026-10-25 (slice M0) |
| ROSTER1 | The unit lifecycle (wounded, downed, captured, dead, retired), bunks apart from the party, recruits behind a story flag, keepsakes, temperament; hooks for bleed-out and Living Martyrs | in-progress | ENCOUNTER2, KIT1 | `docs/specs/ROSTER1.md`; user 2026-10-25 (slice M0) |
| HOOKS1 | Combat hooks for the slice's cast: forced movement, ground and arena hazards, triggers and reactions, unit meters, riders, HP costs, guard bypass, a capstone unlock, AI tactics, data zones, class names per world | in-progress | KIT1, RULESET1, ENCOUNTER2 | `docs/specs/HOOKS1.md`; user 2026-10-25 (Q-55, Q-56: no stand-ins) |
| CAST1 | The Act 1 combat cast as data: founders' kits by intent, new weapon families, the Act 1 bestiary, bosses, encounters, placeholder assets | in-progress | RULESET1, ENCOUNTER2, DERIVE1, HOOKS1 | `games/emberwake/docs/specs/CAST1.md`; user 2026-10-25 (Q-71, Q-74) |
| PORTRAITFX1 | Animated, reactive portraits: engine portrait effects (breath, flicker, blink, overlays, state tints, reactions) with a UI Gallery page, a kit cue table, per-portrait face sidecars, four surfaces (dialogue and vignettes, party and roster, the battle panels, the timeline), then 2x2 expression sheets for the 13 main characters | approved | FX1, STORYUI1, ART3 | `docs/specs/PORTRAITFX1.md` (+ _TASKS.md); user 2026-10-25 (A now, B for the main cast; PQ1-PQ5 as recommended) |

## Goals (user 2026-10-17, decision "Goal-based roadmap"; replaces the suggested build order)

Each goal is done when every check passes on `main`. Goals run in this order; G6 runs beside the others. The old 7-step build order is archived in `docs/archive/ROADMAP_2026-10.md` (HISTORY row "Suggested build order"). Review that led here: project file `roadmap/ROADMAP_REVIEW_2026-10-17.md`.

| Goal | Status | Finish line (all must pass) | Tasks |
| --- | --- | --- | --- |
| G1 Playable Ashlands loop (first test version) | in-progress | From New Game, a mob on the region map opens the combat arena with the four starting party members at their real levels and gear; a win returns to the same tile and the battle results screen shows real XP; a loss wakes the party at the last safe location; the Cinder Gate dialogue fight uses the same link; all three Ashlands quests can be finished and turned in without debug buttons; the browser build runs this loop; a playtest recording of one full run replays headless with no errors | BATTLE1 (thread "Link map fights to real battles"; its own TASKS row), 12x9 grid, PLAYTEST1-L, DLG1-F |
| G2 Feels right to play | approved | 12x9 grid merged with a before / after balance sim recorded; S12S statuses usable in battle and shown in the arena; every combat action plays a sound and an effect (S14-A1, S14-A2 merged; no placeholder warnings in the recorder's asset report); the user plays G1 once and the UI/UX notes are logged as tasks | S12S-L, S12S-A, S14-A1, S14-A2, S14-L1 commit 3, UI3-L, AUTO1-L (battle autoplay) |
| G3 A fair difficulty curve | approved | BAL2-R merged; BAL3 spec approved and built; party targets met: mob >= 90, line 70-90, wave 55-80, elite 50-75, boss 30-50 at L3 and L5; no class below 40% or above 60% in the L5 duel; no caster dies in round 1 in more than a quarter of party fights | BAL2-R, BAL3 |
| G4 A small campaign | approved | An Ashlands story spec approved; the Ashlands has a start, a middle and an ending quest; TRAV1 merged (travel between the Ashlands and the Iron Reaches on the world map); REP1 merged (one quest choice changes a shop price or opens a gated quest); a full New Game to ending playthrough is recorded and replays clean | CAMP1 (story spec, done: games/emberwake/docs/specs/CAMP1.md approved), CAMP1-A1, CAMP1-A2, CAMP1-B, CAMP1-P, TRAV1-L, TRAV1-A, REP1-L, REP1-A |
| G5 Deeper tactics | approved (after G1) | S12T terrain and S12C combos merged; a balance rerun after them still meets the G3 targets | S12T-L, S12T-A, S12C-L, S12C-A |
| G6 Clean house | approved (beside the others) | CLN-9, CLN-17, CLN-3b done; no test suite over 450 lines; `games/emberwake/docs/ROADMAP.md` statuses match `docs/TASKS.md` | CLN-9, CLN-17, CLN-3b, CLN-13 (parked) |

Balance rule: until G2 is done (grid and statuses in), balance work finds defects only; the tuning pass is G3. Until G5, only S1 is balance-ready.

## Inputs for placeholder systems (S8-S11; nothing approved)

Design options for S8-S11 from the 2026-10-15 brainstorm (P-S8 to P-S11 with recommendations) are in `docs/archive/ROADMAP_2026-10.md`; the settled parts are DECISIONS "Exploration encounters", "Travel supplies" and "Playable scope". The early design notes (`.context/GAME_DESIGN.md`, removed in CLN-4) add these phase-1 baselines, kept as input for the S9-S11 specs (the 10-block combat layer they sat on is deprecated, HISTORY row 1, so only the non-combat parts are kept):

- S10 travel: a node graph of regions (plains, mountains, forests, ruins, settlements) with a simple adjacency list; flat travel time per route (example 3 time units); a fixed encounter chance per travel leg, fought in the route's biome. Later additive rules: route weather, road hazards, supplies (reduced version only, DECISIONS "Travel supplies"), merchant caravans.
- S9 exploration: a 2D top-down grid per region with `Vector2i` step movement; tile zones SAFE (towns, sanctuaries: no rolls, rest, trade, dialogue, ambient NPCs with rumours, fixed special encounters) and DANGER (wilderness, dungeons, ruins: step counter triggers mob fights, visible roaming elites and bosses start a fight on touch or proximity); exit tiles on edges or portals open the world map. Later additive rules: bridges, locked gates, puzzle switches, loot chests, stealth and detection cones.
- S11 dialogue and S8 flags: a world-state key/value flag registry; static NPC prompts with branching choices; quest flags that unlock exits or trigger story events. Later additive rules: faction reputation meters, barter shops, timed world events.
- S12-C combo ideas (from `.context/COMBAT_SYSTEM.md`, concepts only, numbers belonged to the deprecated layer): flow moves (an action that ends in a different stance than its kind, so the next action skips the switch cost), a hold-stance action whose duration scales some skills, and disruptive hits that break the target's stance or interrupt a multi-block action. Check HISTORY before proposing any of them.

## Refactor backlog (separate from feature milestones; never part of a builder's feature task)

Done rows (R1) are archived in `docs/archive/ROADMAP_2026-10.md`.

| Id | Item | When | Rules |
| --- | --- | --- | --- |
| CLN-9 | obsolete since the transition 2026-10-19 (it split a Godot file, `content_catalog.gd`, now frozen in `legacy/`; same reason as the CLN-17 / CLN-18 closures, RESCOPE1); was approved (user 2026-10-17, DECISIONS "Cleanup pass 1 answers"): split `content_catalog.gd` (1384 lines) validators by kind into `scripts/content/rules/*` so ITEM1 and CLASS1 stop colliding on one hub file (docs/specs/CLEANUP.md#CLN-9) | after BAL2-L merges, last item of the logic cleanup batch, before ITEM1 and CLASS1 | Pure move; assertion count equal |
| CLN-10 | done 2026-10-17: `docs/DECISIONS.md` is an index; per-system files in `docs/decisions/`; superseded rows in `docs/decisions/archive.md` with "superseded by" pointers (docs/specs/CLEANUP.md#CLN-10) | done (docs only) | No row text changes; pointers in CLAUDE.md and agent files updated by the planner |
| CLN-13 | done 2026-10-19 (user, RESCOPE1): the README overview was written at the transition; the Godot feature log is `legacy/README.md` | - | Planner owns README |
| SPLIT1 | done (user approved 2026-10-19; stages 1-9 landed on develop 2026-10-19 and 2026-10-20): a game-agnostic engine core and the first game in `games/emberwake/` (docs/specs/SPLIT1.md; task rows SPLIT1-1..9) | before the PORT rows | move only: every stage keeps the gate and the parity suites green |

## BAL2 plan (done 2026-10-17; design-brainstorm 2026-10-17; details and options in `games/emberwake/docs/specs/BAL2_PROPOSAL.md`)

Answers the BAL1 open issues. Root causes found by probe: a PERC >= 18 range bonus turns melee units into "ranged" ones (priest L7, rogue L7, warrior L9 lose their melee power), a 4x durability spread (L5 EHP warrior 597 vs mage 146), zero CONS growth for mage/ranger/rogue, kits frozen at L5 by the default loadout rule, no L1 class skill for ranger/brawler, and a near-deterministic sim. Riposte is not the warrior lever (L5 warrior wins 100% with or without it).

| Milestone | Status | Goal | Lane split | Depends on | Balance-ready afterwards |
| --- | --- | --- | --- | --- | --- |
| BAL2-L | done | Sim reliability (continuous score, cap 10, smart_skills/loadout args, spend/distance/preset variants, stats and group modes; per-unit metrics, party-vs-encounter mode, role limit tests, composition sweep: games/emberwake/docs/specs/BAL2_SIM.md), PERC range fix, unit gear sentinel `none`, elite/boss tier values | 1 logic branch, sim commit first | SKILL1-L merged (content hub files) | none |
| BAL2-A | done | Data pass: growth (CONS for damage classes, warrior CONS2 STR2), warrior kite shield, ranger/brawler L1 path skill, unit loadouts, goblin_shaman gear and text | 1 asset branch (seed `games/emberwake/data/seeds/bal2.gd`) | BAL2-L sentinel commit | none |
| BAL2 | done (rerun 2026-10-17; clean rerun BAL2-R open, then BAL3, goal G3) | Rerun duel L1/3/5/7 on the continuous score, tiers, levels, group; one more small data round if a class is outside 40-60 | balance run on clean main | BAL2-L, BAL2-A | S2 and S4 (when every class is in bracket) |

## World systems baseline (approved: user accepted every recommendation, DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)")

Rules: `games/emberwake/docs/specs/S9_RULES.md` (exploration, EXP1), `S10_RULES.md` (travel, TRAV1 + TRAV2 supplies idea), `S11_RULES.md` (dialogue, shops, reputation: DLG1, REP1). Build specs (status `spec` in `docs/TASKS.md`, awaiting approval): `games/emberwake/docs/specs/EXP1_L.md` + `EXP1_A.md`, `DLG1_L.md` + `DLG1_A.md`, `TRAV1_L.md` + `TRAV1_A.md`, `REP1_L.md` + `REP1_A.md`. Progress is flags only; one shared battle bridge for exploration, road and dialogue fights; one seeded random stream per save.

| Milestone | Status | Depends on | Notes |
| --- | --- | --- | --- |
| EXP1 | done | M4-L | region maps, step-counter encounters, patrols, location entry hooks; save v3 |
| DLG1 | done | M4-L, EXP1-L | dialogue, service choices from resolved quests, shops, gold, `gone_flags`; save v4 |
| TRAV1 | TRAV1-L and TRAV1-A done (2026-10-17, develop) | EXP1-L | world map, routes, fog, road encounters, world clock; after DLG1 (shared hub files) and BATTLE1 (road fights use the battle link, goal G4); save v5 |
| REP1 | REP1-L and REP1-A done (2026-10-17, develop) | DLG1-L | reputation int flags, condition operators, price multipliers; no save bump; after TRAV1-L (shared hub files) |
| TRAV2 | approved (user 2026-10-17 23:25, card 'Both'; TRAV2-L, TRAV2-A) | TRAV1, DLG1 | supplies: one counter 0..99, a leg costs its route time, shops and chests refill, road chance x1.5 at 0, no fail state; save v6 |

EXP1, DLG1, TRAV1 and REP1 all edit `world_schema` and `world_catalog` (EXP1, DLG1 and TRAV1 also `game_state` and the save files): run the logic parts in sequence EXP1-L, DLG1-L, TRAV1-L, REP1-L, one save version bump each (v3, v4, v5; REP1 none). Each asset part may run beside the next logic part (data only).

## Core and presentation baseline (approved, design-brainstorm 2026-10-17; user answers in DECISIONS "Decision batch (user 2026-10-17)")

Rules and open questions: `games/emberwake/docs/specs/S12_RULES.md` (S12S status expansion, S12T terrain, S12C combos; skill upgrades covered by SKILL1 ranks and grades), `S14_RULES.md` (AssetCatalog manifest, presenter animation, SoundBank, asset-load suite), `UI_MENUS_RULES.md` (menus, save slots, new game, party screen with level-up, journal). Mechanics only add inputs to `DamageCalculator.compute`; numbers are data fields, no balance values proposed.

| Milestone | Status | Depends on | Notes |
| --- | --- | --- | --- |
| S12S | S12S-L and S12S-A done (2026-10-17) | ITEM1-L, CLASS1-L | status slice (goal G2), then S12T terrain and S12C combos in sequence (shared skill and plan files; goal G5, after G1) |
| S14-1 | done (S14-L1, S14-L2, S14-L3, S14-A1, S14-A2 done 2026-10-17) | - | AssetCatalog manifest, key resolution, placeholder icons, asset-load suite; then presenter (S14-2) and audio (S14-3) |
| ASSETORG2 | approved (user 2026-10-20, all eight recommendations) | ASSETORG1 (done) | assets grouped by reuse in three layers (engine baseline, RPG kit `kits/rpg/`, game named assets), fallback game, kit, engine, drawn shape; class bodies with variants, palettes, the token from the sprite, portraits by tier; spec `docs/specs/ASSETORG2.md`, rows ASSETORG2-E, -G, -D, -K, -B, -T |
| PLAYUI1 | approved (user 2026-10-22, all ten answers) | UIKIT1 | play-screen pass from the 2026-10-22 playtest: save slots, HUD bars, region map legend and log, NPC service icons, dialogue box, shop trade, note editor, icons, region sprites (spec games/emberwake/docs/specs/PLAYUI1.md) |
| PROPS1 | approved (user 2026-10-22, section 10 answers all A; spec games/emberwake/docs/specs/PROPS1.md, rows E, L1, L2, T, A1, A2) | PLAYUI1-M | buildings and props on region maps |
| SPRITE1 | done (SPRITE1-S2, A2, L, U done 2026-10-22; was approved by the user 2026-10-20) | ASSETORG2-T, ASSETORG2-E | animated battle tokens: one sheet per class body (SPRITE1-S2 spec update), then SPRITE1-A2, -L, -U |
| DIR4 | approved (user 2026-10-23, spec games/emberwake/docs/specs/DIR4.md) | FACE1, SPRITE2, WAR2H1 | four facing directions on the sprite rig: front and back views of idle, walk and the main attack; the engine's direction lookup; battle facing, then the region map; warrior pilot first, the rest after STAGE1's camera pick |
| STAGE1 | approved (user 2026-10-23, option B; spec games/emberwake/docs/specs/STAGE1.md) | DIR4-S, PROPS1-E, FACE1 | battle stage: a tilted 32x24 board with zone backdrops and a front face, upright props sorted with the units, contact shadows, 40 px units; Flat as a setting until the review |
| REWARD1 | in-progress (REWARD1-L, A and S done 2026-10-21; B next, after a release puts them on main; was approved by the user 2026-10-20, all eight recommendations) | S7, S8, S9, PURSE1 | battle rewards per defeated enemy: kill-weight purse shares, XP on every outcome, per-unit-type loot tables, key loot that needs the kill and a win, objective rewards; replaces the PURSE1 win-only rule; folds PURSE1-A and PURSE1-B (`games/emberwake/docs/specs/REWARD1.md`) |
| PORTRAIT1-A | covered by ART2 (2026-10-20: the eight heroes, the NPCs, the boss, the warchief and the named goblins generated and accepted; 12 generic bases) | ASSETORG2-G | the portraits land with ASSETORG2's layout |
| PALETTE1-A | proposed | ASSETORG2-G, SPRITE1-A2 | party palettes so characters of one class differ (minor adjustments on the class body) |
| ICONS1-A | folded into ART2 (2026-10-20: base icons per weapon family and skill kind come from the icon sheets) | ASSETORG2-K | kit base icons for the types without one yet (mace, orb, crossbow, material, quest) |
| PLAYTEST2 | idea | PLAYTEST1, UI1-L | overworld and dialogue exact replay |
| UI1 | done (UI1-L, UI1-A, UI2-L, UI3-L and SAVE-IO done 2026-10-17) | M4-L | menu router, main menu, new game, save slots; then UI2 party screen and level-up, UI3 journal |

## Playtest and delivery (2026-10-17, cloud threads)

| Milestone | Status | Notes |
| --- | --- | --- |
| WEB1 | done | Web (HTML5) build on every PR, GitHub Pages and private itch.io option (`docs/workflow/web.md`) |
| PLAYTEST1 | done (PLAYTEST1-L, release batch 2026-10-17) | Playtest recordings, notes, playback and headless replay; recorder logs clicks, hovers, UI open / close and asset loads (goal G1) |
| GRID1 (12x9 grid) | done (GRID12, release batch 2026-10-17) | 12x9 arena instead of a square grid, slimmer side panels, party left and enemies right; its thread records the decision and task row (goal G1) |
| BATTLE1 | done (BATTLE1-L, release batch 2026-10-17) | Map, dialogue and (with TRAV1) road fights open the real combat arena instead of the debug outcome buttons (goal G1); its thread owns the TASKS row |

## Open engine items (tracked)

- `ItemArt.icon_key` still reads the deprecated `shortsword` off-hand category (HISTORY row 23): cleanup candidate.
The resolved bullets (AI hooks, done in AI1) are archived in `docs/archive/ROADMAP_2026-10.md`.

- Disengage "once per round" and "after a MOVE" are not enforced/modelled (same as plain MOVE).
- Skill-tier picker is not rendered in the planner UI yet (grouping API exists). For party heroes this is replaced by the rank chosen on the Skills tab (SKILLUI1, approved 2026-10-20); it stays open for enemies only (no player-facing need).
