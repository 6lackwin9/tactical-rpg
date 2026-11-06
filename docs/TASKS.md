# Task queue (reference map; active build queue is docs/READY.md)

**Two-session split (2026-11-07)**: The Build Session reads **only** docs/READY.md. The canonical 186-task master plan is docs/specs/MASTERPLAN1.md (MASTERPLAN1_ROWS.md maps every row below onto U1.1..X8). Full historical row descriptions are preserved in docs/archive/TASKS_PRE_MASTERPLAN.md, and completed/dropped rows are in docs/archive/TASKS_DONE.md.

| Id | Title | Layer | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CLN-49 | git rm the 20 executed seeds of finishe... | game | planner | - | approved | docs/specs/CLEANUP.md... | see CLEANUP.md CLN-49 |
| CLN-53 | Oversized docs | docs | design | CLN-30 | approved | docs/specs/CLEANUP.md... | see CLEANUP.md CLN-53 |
| CLN-71 | Relook the legacy debug screens with th... | engine + game | cleanup | - | idea | - | a CLEANUP.md section |
| SKILLTEXT2 | Skill descriptions print a zero value a... | game | asset o... | - | idea | - | games/emberwake/data/... |
| SIMKPI1 | Role sim: the striker's "prio down rnd"... | game | logic | PORT-SIM (done) | idea | - | kits/rpg/crates/rpgki... |
| ART2 | Art review of 2026-10-20 | kit + game | asset | SPRITE1, ASSETORG2 | waiting | games/emberwake/docs/... | tools/sprites/, tools... |
| ICONUI1 | Draw skill and item icons on the engine... | game | logic | ART2 | idea | games/emberwake/docs/... | games/emberwake/crate... |
| ARTTOOL1 | Port the asset generators (`tools/sprit`... | engine + ki... | logic | ASSETORG2-T | idea | docs/decisions/workfl... | tools/, a new tool crate |
| PLAYBACK2 | Round-by-round playback viewer | engine + game | logic | PORT-PLAYTEST (done) | idea | games/emberwake/docs/... | engine/crates/aegis_u... |
| DRAG1 | Drag and drop on the other screens wher... | game | design | UIKIT2 | approved | games/emberwake/docs/... | the screen pass; then... |
| REWARDXW1 | Cross-world road fights pay for their f... | game | logic | REWARD1-L | idea | REWARD1.md section 2.1 | kits/rpg/crates/rpgki... |
| LAYOUT1 | Layout mockups in the engine | engine + game | design | UIGALLERY1 | idea | docs/decisions/workfl... | engine/crates/aegis_u... |
| UIR-4 | UI review | game | logic | UIR-1 | approved | games/emberwake/docs/... | games/emberwake/crate... |
| QUESTTRACK1-U | Quest tracker | game | logic | - | approved | <https://claude.ai/ar... | games/emberwake/crate... |
| RCLOSE1-U | Right click closes settings and UI popu... | game | logic | UIR-2 | approved | user playtest 09:13 | games/emberwake/crate... |
| UIR-7 | UI review | engine + game | logic | UIR-1, UIR-6, PORT-... | approved | games/emberwake/docs/... | engine/crates/aegis_u... |
| CAMP1-P | New Game to campaign_done playthrough, ... | game | planner... | CAMP1-A2 | idea | games/emberwake/docs/... | a recorded engine pla... |
| PLAYTEST2 | Overworld and dialogue replay | game | logic | PORT-PLAYTEST | idea | games/emberwake/docs/... | rpgkit_world explore ... |
| LEVERS1-K | Kit capabilities from the dropped LEVER... | kit | logic | - | idea | `docs/specs/LEVERS1.md` | - |
| KEEP1 | Warden Keep upgrades (gold goal) | game | design | - | approved | games/emberwake/docs/... | new spec |
| DIFF1 | Difficulty options | game | design | - | approved | games/emberwake/docs/... | new spec |
| DEEDS1 | Deeds: mastery revived as a progression... | game | design | - | approved | games/emberwake/docs/... | new spec |
| RESPEC1 | Paid respec | game | design | - | approved | games/emberwake/docs/... | new spec |
| OFFHAND1 | Off-hand skill items (revives dual wiel... | game | design | - | approved | games/emberwake/docs/... | new spec |
| OBJ1-U | Objective header, ring and exit markers... | game | logic | VISCLAR1-B | approved | games/emberwake/docs/... | games/emberwake/crate... |
| MARKERS1 | One marker icon set | game | logic | PLAYUI1-I | idea | games/emberwake/docs/... | kits/rpg/crates/rpgki... |
| PROPS1-A3a | Prop prompts by size class | game (art) | art, th... | PROPS1-A2, EXPLOREV... | approved | this row; games/ember... | `games/emberwake/docs`... |
| PROPS1-A3b | Prop prompts by size class | game (art) | art, th... | PROPS1-A2, EXPLOREV... | approved | this row; games/ember... | `games/emberwake/docs`... |
| PIXEL4 | Tune the backdrop-aware rim on smooth (... | game (tools) | logic | PIXEL3 | approved | decision "Portrait ba... | `tools/portraits/pixe`... |
| MOB1 | Rank-and-file units (user 2026-10-27, d... | game | design,... | PIXEL4 | approved | this row, the decisions | `games/emberwake/docs`... |
| ALINE1-F | Fix the green streaks on Aline's dark s... | game (tools) | logic | PIXEL3 | approved | this row | `tools/portraits/pixe`... |
| PORTRAITFX1-XA | Expression frames for the 13 (B) | game (art +... | asset | PORTRAITFX1-X, PORT... | approved | docs/specs/PORTRAITFX... | `games/emberwake/docs`... |
| PORTRAITFX2-A | The second expression sheet (Determined... | game (art) | asset | PORTRAITFX2-S, PORT... | approved | games/emberwake/docs/... | `art_src/`, `games/em`... |
| LOOK1-M1F | Title backdrop fire layer | game | logic | LOOK1-M1, LOOK1-R | approved | games/emberwake/docs/... | emberwake_gui/src/men... |
| LOOK1-M2 | New game, load / save cards, settings r... | game | logic | LOOK1-M1, LOOK1-G2,... | approved | games/emberwake/docs/... | `emberwake_gui/src/{m`... |
| LOOK1-L2 | Region tile sets | game | logic | LOOK1-L1, EXPLOREVI... | approved | games/emberwake/docs/... | `emberwake_gui/src/{r`... |
| LOOK1-A | Art batches A1-A7 of LOOK1 section 6 (G... | game (art) | art | LOOK1-T, A1-A3, EXP... | approved | games/emberwake/docs/... | `art_src/`, `C:/dev/l`... |
| TURNFEEL1 | A turn from up or down into left or rig... | game | logic | WALKRUN1 | review | this row | `emberwake_gui/src/{m`... |
| LAYOUT720-Q | One `grid()` line per quadrant screen (... | game + engine | logic | LAYOUT720-E1 | approved | `docs/specs/LAYOUT720`... | those screens' files,... |
| LAYOUT720-G0 | The game declares `Fixed` and window si... | game | logic | LAYOUT720-E5 | approved | `docs/specs/LAYOUT720`... | `game.json`, `emberwa`... |
| LAYOUTDATA1-E2 | `menu_column_in`, `tab_rail_in`, `Ui | engine | logic | LAYOUT720-M0, LAYOU... | approved | `docs/specs/LAYOUTDAT`... | `aegis_ui/src/{panes,`... |
| LAYOUTDATA1-D1 | Tree: schema, walk, measure, reader, store | engine | logic | LAYOUTDATA1-E1 | approved | `docs/specs/LAYOUTDAT`... | new `aegis_layout/src`... |
| LAYOUTDATA1-D2 | Drawing, `LayoutHost`, contract, bare l... | engine | logic | LAYOUTDATA1-D1, LAY... | approved | `docs/specs/LAYOUTDAT`... | `aegis_ui/src/{layout`... |
| LAYOUTDATA1-M | Gallery "Layouts" and `MockHost` | engine + game | logic | LAYOUTDATA1-D2 | approved | `docs/specs/LAYOUTDAT`... | new `aegis_ui/src/lay`... |
| LAYOUTDATA1-G1 | `MainMenu` on a host | game | logic | LAYOUTDATA1-D2 | approved | `docs/specs/LAYOUTDAT`... | `menus.rs` (`MainMenu`... |
| LAYOUTDATA1-G2 | `main_menu.json` by the UI agent, the p... | game | ui | LAYOUTDATA1-G1, LAY... | approved | `docs/specs/LAYOUTDAT`... | `assets/ui/layouts/ma`... |
| LAYOUTDATA1-R | `ui-author` role note, `ui` lane | process | planner | LAYOUTDATA1-D1 | approved | `docs/specs/LAYOUTDAT`... | `.claude/agents/ui-au`... |
| LOOK1-M2-L | Menu rebuild layout files, mock first, ... | game | ui | LAYOUTDATA1-M, LAYO... | approved | `docs/specs/LAYOUTDAT`... | `assets/ui/layouts/*.`... |
| LOOK1-M2-B1 | Hosts: New game, Slots, Settings | game | logic | LAYOUTDATA1-G1 | approved | `docs/specs/LAYOUTDAT`... | `menus.rs` |
| LOOK1-M2-B2 | Hosts: Game menu, Run over, Campaign no... | game | logic | LAYOUTDATA1-G1 | approved | `docs/specs/LAYOUTDAT`... | `hub.rs`, `runover.rs` |
| LOOK1-M2-C | Stays code | game | logic | LAYOUTDATA1-E1, LAY... | approved | `docs/specs/LAYOUTDAT`... | `screens.rs`, `story.`... |
| FPSCAP2 | Region and world maps still draw every ... | engine | logic | FPSCAP1 | proposed | this row | `aegis_ui/src/shell.r`... |
| LINKGATE1 | The agent link is open in every build, ... | engine | logic | INTEG1-E1 | proposed | this row | `aegis_ui/src/{shell,`... |
| INTEG1-G | Region map `walk_to` | game | logic | INTEG1-E1 | approved | `docs/specs/AGENTROLE`... | `emberwake_gui/src/re`... |
| INTEG1-E2 | Frame counters, `perf` request and step... | engine | logic | INTEG1-E1 | approved | `docs/specs/AGENTROLE`... | `aegis_ui/src/{perf,s`... |
| INTEG1-E3 | `perf.jsonl`, `aegis scenario perf-repo`... | engine | logic | INTEG1-E2 | approved | `docs/specs/AGENTROLE`... | `aegis_cli/src/scenar`... |
| INTEG1-R | Role note, lane, worktree, report file | process | planner | INTEG1-E1 | approved | `docs/specs/AGENTROLE`... | `.claude/agents/integ`... |
| INTEG1-S1 | `warden_keep_smoke`, `tl_flank_fight` | game | integra... | INTEG1-E1, INTEG1-G... | approved | `docs/specs/AGENTROLE`... | `games/emberwake/scen`... |
| INTEG1-S2 | `sunk_ward_slice` | game | integra... | INTEG1-S1, STORYUI1... | approved | `docs/specs/AGENTROLE`... | `games/emberwake/scen`... |
| INTEG1-X | Cut-over briefs after the shadow | process | planner | INTEG1-S1 | approved | `docs/specs/AGENTROLE`... | `.claude/agents/*.md`... |
| SCOPE1 | Brief fixes needing no integration | process | planner | user answers | approved | `docs/specs/AGENTROLE`... | `.claude/agents/*.md`... |
| SKILLS1 | Scripts to `tools/planner/` | process | planner | - | approved | `docs/specs/AGENTROLE`... | `tools/planner/`, `.c`... |
| KITDONE1 | The measure in `milestone-status` | process | planner | - | approved | `docs/specs/AGENTROLE`... | `.claude/skills/miles`... |
| STATUSROWS1 | Status table as rows, goldens unchanged | kit | logic | N | approved | `docs/specs/AGENTROLE`... | `rpgkit_rules/src/sta`... |
| WEBPROBE1 | Probe a WebAssembly build of the engine | engine | logic | - | proposed | `docs/HISTORY.md` 202... | a throwaway branch an... |
| ARTSCALE1 | Check the remaining nearest-scaled art ... | engine | logic | SPRITEFIT1 | approved | decision "Fixed canva... | `aegis_ui` drawing |
| REGIONGROUND1-W | Tall walls | game | logic | REGIONGROUND1-L | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-S | Soft shadow, `pool`, `rim`, the `region`... | game | logic | REGIONGROUND1-W, RE... | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-A2 | Mark sheets and the Ashlands wall face,... | game (art) | art | REGIONGROUND1-T | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-R1 | Place the picks, the user plays the map... | game | asset, ... | REGIONGROUND1-D1, A... | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-D2 | Mark rows | game (data) | asset | REGIONGROUND1-R1 | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-A3 | Sunk Ward street textures, 9 images | game (art) | art | REGIONGROUND1-R1 | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-D3 | Sunk Ward ground rows and picks | game (data) | asset | REGIONGROUND1-A3, S... | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-C | Drop the old Ashlands `floor` and `shal`... | game | logic | REGIONGROUND1-D2 | approved | games/emberwake/docs/... | see the spec's slices |
| REGIONGROUND1-X | Connector pieces, only if a pair needs ... | engine, kit | logic | REGIONGROUND1-R1 | approved | games/emberwake/docs/... | see the spec's slices |
| REVEAL1 | Indoor reveal | game | design,... | REGIONGROUND1-R1 | approved | to spec | - |
| GROUNDTUNE1 | Ground tuner | engine + ga... | logic | REGIONGROUND1-L, LO... | approved | to spec | - |
| LOOK1-R | Place the picked art | game (data) | asset | LOOK1-K, LOOK1-T, S... | approved | games/emberwake/docs/... | `games/emberwake/asse`... |
| PUPPET1 | Layered portrait puppets (C) | engine + game | design | PORTRAITFX1-XA | idea | - | - |
| DIR4-R4 | Front and back views for the secondary ... | kit + game | asset | DIR4-R3 | idea | games/emberwake/docs/... | as DIR4-R3 |
| EXPLOREVIEW1-R | Review session | game | design ... | EXPLOREVIEW1-L | approved | games/emberwake/docs/... | `C:/dev/localgen/revi`... |
| FX1-A | Slice shakes | game (data) | asset | FX1-S, STORYUI1-A, ... | approved | games/emberwake/docs/... | `data/worlds/sunk_war`... |
| MASTERPLAN1 | The master plan of the game's systems a... | process | design | SILOS1 | approved | to spec | - |
| KITIDS1 | T4: no class id assumed in the kit | kit | logic | BASERULES1 | proposed | `docs/specs/SILOS1.md` | `rpgkit_rules/src/uni`... |
| WORLDFIGHT1 | T1: `battle::begin_world`, `battle::fin`... | kit + game | logic | VISCLAR1-D (overlap) | proposed | `docs/specs/SILOS1.md` | `rpgkit_world/src/bat`... |
| SERVICES1 | T2: `Services` out of `RegionMap` | game | logic | WORLDFIGHT1 | proposed | `docs/specs/SILOS1.md` | `emberwake_gui/src/{r`... |
| GALLERYWORLD1 | A world picker for the Gallery's sample... | game | logic | - | proposed | `docs/specs/SILOS1.md` | `emberwake_gui/src/ga`... |
| SHOPTEST1 | Scratch-world kit test for shop prices,... | kit | logic | - | proposed | `docs/specs/SILOS1.md` | `rpgkit_world/tests/s`... |
| KIT2-UI | Spec only (T3) | kit + game | design | Q2 | proposed | `docs/specs/SILOS1.md` | `docs/specs/KIT2_UI.md` |
| SILOCARD1 | Builder reports and spec template state... | process | planner | - | proposed | `docs/specs/SILOS1.md` | `.claude/agents/logic`... |
| SILOSCN1 | One short scenario per silo through `ru`... | game | integra... | INTEG1-R, GALLERYWO... | proposed | `docs/specs/SILOS1.md` | `games/emberwake/scen`... |
| BASERULES1 | The base rules pass before battle | kit | design | - | spec | `docs/specs/BASERULES`... | - |
| ATTACKLOCK1-O | Before the attack lock is switched on | kit | logic | ATTACKLOCK1 | proposed | this row | `rpgkit_battle/src/re`... |
| COMBATLIMIT1 | Limit test of combat alone | kit + game | balance | BASERULES1-F, SIMCO... | proposed | this row | `rpgkit_sim`, `BALANC`... |
| BASERULES1-DOM | Dominance audit, every area and not onl... | kit + game | balance | BASERULES1-M | proposed | `docs/specs/BASERULES`... | `BALANCE_REPORT.md` |
| BASERULES1-M | Register metadata, `check/ruleset.rs`, ... | kit | logic | ATTACKLOCK1 | spec | `docs/specs/BASERULES`... | `ruleset.rs`, `check/`... |
| BASERULES1-V | Values out of code into a `base_rules` ... | kit | logic | BASERULES1-M | superseded (U1/U2) | `docs/specs/BASERULES`... | `damage.rs`, `stats.r`... |
| BASERULES1-D | Stale rules | kit | logic | BASERULES1-M | superseded (U1/U2) | `docs/specs/BASERULES`... | `check/skill.rs`, `ch`... |
| BASERULES1-G | Guard term, `guard_model`, `guard_start`... | kit | logic | BASERULES1-M, STATU... | superseded (U1/U2) | `docs/specs/BASERULES`... | `resolver.rs`, `comma`... |
| BASERULES1-F | `damage_model` flat, `flat_*`, sim key,... | kit | logic | BASERULES1-M | superseded (U1/U2) | `docs/specs/BASERULES`... | `damage.rs`, `command`... |
| BASERULES1-P | Stats as data | kit | logic | BASERULES1-V | superseded (U1/U2) | `docs/specs/BASERULES`... | `rpgkit_data`, `stats`... |
| BASERULES1-PP | The proof | kit | logic | BASERULES1-P | superseded (U1/U2) | `docs/specs/BASERULES`... | tests only |
| BASERULES1-H | `heal_model`, `item_value`, item `grant`... | kit | logic | K3's start | superseded (U1/U2) | `docs/specs/BASERULES`... | `command.rs`, `rpgkit`... |
| BASERULES1-B | Kind `value_bands`, warnings, Forge-free | kit | logic | BASERULES1-M | superseded (U1/U2) | `docs/specs/BASERULES`... | `check/skill.rs`, schema |
| BASERULES1-A | Data: skill and item text free of stanc... | game | asset | BASERULES1-D | superseded (U1/U2) | `docs/specs/BASERULES`... | `data/catalog/` |
| BASERULES1-BN | Move rate and reference-hit bands, meas... | game | balance | BASERULES1-F, P or V | superseded (U1/U2) | `docs/specs/BASERULES`... | `BALANCE_REPORT.md` |
| BASERULES1-U | User session | kit + game | design | - | superseded (U1/U2) | `docs/specs/BASERULES`... | spec |
| STATUSPASS1 | A session with the user over all statuses | kit + game | design | - | proposed | to spec | - |
| SKILLPASS1 | A session with the user over all skills... | game | design | STATUSPASS1 | proposed | to spec | - |
| CAST1-A4 | Units: founders, enemies, bosses | game (data) | asset | CAST1-A2, CAST1-A3,... | in-progress | games/emberwake/docs/... | `data/catalog/units.j`... |
| DRILLSAVE1 | From the UNITDRILL1 review | kit | logic | UNITDRILL1 | proposed | this row | `rpgkit_world/src/sav`... |
| CAST1-A5 | `sunk_ward` characters, disciplines, lo... | game (data) | asset | CAST1-A4, SUNKWARD1... | approved | games/emberwake/docs/... | `data/worlds/sunk_war`... |
| CAST1-A6 | Encounters | game (data) | asset | CAST1-A5, ENCOUNTER... | approved | games/emberwake/docs/... | `data/worlds/sunk_war`... |
| CAST1-A6b | Encounters | game (data) | asset | CAST1-A6, HOOKS1-Q | spec | games/emberwake/docs/... | `data/worlds/sunk_war`... |
| CAST1-B | Act 1 feel and balance runs | game | balance | CAST1-A6, RULESET1-... | approved | games/emberwake/docs/... | `games/emberwake/docs`... |
| HOOKS1-Q | Cheat-death (`spare_once`, `spared` tri... | kit | logic | HOOKS1-U, ENCOUNTER2-H | spec | docs/specs/HOOKS1.md ... | schema; `kits/rpg/cra`... |
| BIGUNIT1 | Large units | engine + kit | design,... | - | idea | - | - |
| CAMPAIGN1-D | `sunk_ward` campaign rows (meters, chec... | game (data) | asset | CAMPAIGN1-S, SUNKWARD1 | in-progress | docs/specs/CAMPAIGN1.... | `data/worlds/sunk_war`... |
| CAMPAIGN1-P | Shop `supply_steps` (Act 3 prices) | kit | logic | CAMPAIGN1-S, KIT1-N | spec | docs/specs/CAMPAIGN1.... | `schema.json`, `kits/`... |
| CAMPAIGN1-DOC | Doctrine leans (Act 2) | kit | logic | CAMPAIGN1-M, ROSTER... | spec | docs/specs/CAMPAIGN1.... | `campaign/meter.rs`, ... |
| ROSTER1-D | `sunk_ward` roster rows | game (data) | asset | ROSTER1-L, SUNKWARD... | approved | docs/specs/ROSTER1.md... | `data/worlds/sunk_war`... |
| ROSTER1-T | Temperaments (slotless passive, default... | kit | logic | ROSTER1-L, RULESET1... | approved | docs/specs/ROSTER1.md... | `schema.json`; `kits/`... |
| ROSTER1-TA | Temperament samples | game (data) | asset | ROSTER1-T | approved | docs/specs/ROSTER1.md... | `data/catalog/{temper`... |
| ROSTER1-B | Bleed-out and revive (Act 2) | kit | logic | ENCOUNTER2-H, RULES... | spec | docs/specs/ROSTER1.md... | `kits/rpg/crates/rpgk`... |
| ROSTER1-BU | Bleed counter on screen | game | logic | ROSTER1-B | spec | docs/specs/ROSTER1.md... | `emberwake_gui/src/{b`... |
| ROSTER1-P | Pool, rescue, recruit templates, open-f... | kit | logic | ROSTER1-L | spec | docs/specs/ROSTER1.md... | `kits/rpg/crates/rpgk`... |
| ROSTER1-K | Procedural keepsake instances | kit | logic | ROSTER1-P | spec | docs/specs/ROSTER1.md... | `kits/rpg/crates/rpgk`... |
| ROSTER1-KD | Keepsake templates by class | game (data) | asset | ROSTER1-K | spec | docs/specs/ROSTER1.md... | `data/catalog/items.j`... |
| ROSTER1-M | Living Martyr fuse | kit | logic | ROSTER1-P, CAMPAIGN1-C | spec | docs/specs/ROSTER1.md... | `kits/rpg/crates/rpgk`... |
| ROSTER1-RU | Bunks screen | game | logic | ROSTER1-P | spec | docs/specs/ROSTER1.md... | `emberwake_gui/src/` ... |
| STORYUI1-A | Slice story data | game (data) | asset | STORYUI1-S, SUNKWAR... | approved | docs/specs/STORYUI1.m... | `data/worlds/sunk_war`... |
| STORYUI1-F | World Forge tabs for Vignettes, Sequenc... | game | logic | STORYUI1-S | proposed | docs/specs/STORYUI1.md | `games/emberwake/crat`... |
| ENCOUNTER2-L | Impossible fight and last stand (later ... | kit | logic | ENCOUNTER2-HA, KIT1-N | spec | docs/specs/ENCOUNTER2... | as H plus `intent.rs`... |
| ENCOUNTER2-X | Phase `swap`, fate Transformed (later) | kit | logic | ENCOUNTER2-K, KIT1-N | spec | docs/specs/ENCOUNTER2... | battle `encounter,ses`... |
| ENCOUNTER2-F | Phase `side | kit | logic | ENCOUNTER2-K, KIT1-N | spec | docs/specs/ENCOUNTER2... | battle `world,ai,sess`... |
| ENCOUNTER2-R | Rooms core | kit | logic | ENCOUNTER2-T, KIT1-N | spec | docs/specs/ENCOUNTER2... | battle `grid,world,se`... |
| ENCOUNTER2-RA | Room AI (later) | kit | logic | ENCOUNTER2-R, KIT1-N | spec | docs/specs/ENCOUNTER2... | `ai.rs` |
| ENCOUNTER2-RU | Room screen | game | logic | ENCOUNTER2-R, STAGE1-L | spec | docs/specs/ENCOUNTER2... | `emberwake_gui/src/ba`... |
| ENCOUNTER2-D2 | Later sample encounters (last stand, be... | game (data) | asset | ENCOUNTER2-L, ENCOU... | spec | docs/specs/ENCOUNTER2... | `encounters.json` |
| CLN-23 | Spec Status lines match their tasks (21... | game | design | CLN-P2 | approved | docs/specs/CLEANUP.md... | games/emberwake/docs/... |
| CLN-27 | GAME_BRIEF | game | design | - | approved | docs/specs/CLEANUP.md... | games/emberwake/docs/... |
| CLN-30 | Archive the 52 built Godot-era game spe... | game | design | CLN-23 | approved | docs/specs/CLEANUP.md... | games/emberwake/docs/... |
| CLN-33 | Docs status check in `fc.sh docs` (`too`... | engine | logic | CLN-P2, CLN-23 | approved | docs/specs/CLEANUP.md... | tools/docs_status.awk... |
| CLN-34 | `docs_needed.py` and skill task-landing | process | plan | CLN-33 | approved | docs/specs/CLEANUP.md... | .claude/skills/task-l... |
| VISCLAR1 | Battle visual clarity pass | game | design | RULESET1-U, HOOKS1-... | approved | games/emberwake/docs/... | the battle settings; ... |
| CROWDFIGHT1 | A crowded test fight of about 12 units,... | game (data) | asset | - | in-progress | this row | `data/catalog/encount`... |
| SKILLICONS1 | Own icons for the skill families the sl... | game | art | VISCLAR1-B | proposed | - | `games/emberwake/asse`... |
| INTENT2 | The intent framework | kit | design | - | spec | `docs/specs/INTENT2.md` | - |
| INTENT2-K1 | Rule `field_intent` | kit | logic | ATTACKLOCK1 | approved | `docs/specs/INTENT2.md` | `round_intent*.rs`, ``... |
| INTENT2-K2 | Rule `shape_anchors` | kit | logic | K1 | approved | `docs/specs/INTENT2.md` | `skill.rs`, `command.`... |
| INTENT2-K3 | Items: lists, `area`, `item_ap`, tile c... | kit | logic | K1 | approved | `docs/specs/INTENT2.md` | `command.rs`, `plan.r`... |
| INTENT2-AI | Row Z, ring row, row 2's self target, r... | kit | logic | K1, K2 | approved | `docs/specs/INTENT2.md` | `ai.rs` |
| INTENT2-A | Seed through `aegis content` | game | asset | K1-3 | approved | `docs/specs/INTENT2.md` | `data/catalog/` |
| INTENT2-U | All rows drawn and greyed (IR19), Field... | game | logic | K1-3, VISCLAR1-D | approved | `docs/specs/INTENT2.md` | `emberwake_gui/src/ba`... |
| INTENT2-B | Feel run, the measures of INTENT2_CASES... | game | balance | all above, SIMCOUNT1 | approved | `docs/specs/INTENT2.md` | `BALANCE_REPORT.md` |
| ATTACKLOCK1 | Decision "Attack lock at wind-up" | kit | logic | VISCLAR1-D, SIMCOUNT1 | slice 1 done (2... | this row | `rpgkit_battle/src/co`... |
| POISE1 | Revisit poise (user 2026-11-04) | game | design | VISCLAR1-B | proposed | to spec | - |
| HOLDART1 | Hold tiles and any interactable on the ... | game | asset | - | proposed | to spec | battle props |
| BATTLEMAP1 | Battle maps | kit + game | design | - | proposed | to spec | - |
| ENCOUNTER2-C | Mid-battle dialogue overlay (the talk b... | kit + game | design ... | ENCOUNTER2-U | spec | docs/specs/ENCOUNTER2... | to spec |
| TERMS1 | Terms table (decision "INPUT1 answers, ... | kit + game | design ... | - | spec | to spec | `kits/rpg/schema.json`... |
| TESTCLEAN1 | Tests leave their scratch folders in th... | engine | logic | - | proposed | this row | `aegis_ui/tests/`, te... |
| TOOLTESTS2 | `tools/sprites` suites reach no gate (``... | process | logic | - | proposed | - | `tools/fc.sh`, `check`... |
| TACTIC2 | Revisit the party and hero tactics unde... | kit + game | design | INPUT1-U | approved | to spec | - |
| ENCOUNTER2-CA | The prologue's scripted opening (`enc_p`... | game (data) | asset | ENCOUNTER2-C4, CAST... | approved | docs/specs/ENCOUNTER2... | `data/worlds/sunk_war`... |
