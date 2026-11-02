# Patch notes

Every change to the codebase, newest first, one section per day (UTC). "In the game" is written for players and testers; the lists below it are the commit subjects. Kept by the `patch-notes-write` skill: `python3 tools/patch_notes.py` prints the commits on `develop` since the marker below, and whoever runs it (Merge & Release at each release cut, or anyone asked for patch notes) writes the new section and moves the marker.

<!-- last: 767ef4ed -->

## 2026-11-03

In the game:

- **Walking and running**: the party walks at a calmer pace on region maps, and holding Shift runs. The Animation speed setting scales both.
- **Dash**: humanoid units now have their own dash animation in battle, with dust, and speed lines from the side.
- **Testers' keys** (debug builds): F6 cycles the map zoom, F9 the window size, F11 the walk step.

Every change that touched the game:

- WALKRUN1: the region map walks at 200 ms a step and runs at 130 ms while Shift is held, in every build, each / anim_speed, on held-key and path walks; letting go walks from the next step; a step glides in the time it started with; F11 and the debug slider set the walk only; Run: Hold Shift joins the Maps keys (767ef4ed)
- LAYOUT720-E3: Metrics (Ui::m, Metrics::of, hair), Env, theme metrics (no key = today's value doubled, layout included), Ui::{screen, image_in, frame_body, corner_size}, every widget and draw_tip right on all grids, carets on Native fields right; a tip follows the focus on Native screens only; T1 unedited, T3 in tests/widgets_native.rs, shipped art zero pixels different (106c3221)
- VIEWKEYS2: debug-build F11 cycles the walk step 90, 110, 130, 160, 200 ms on the region map, applied at once and saved; the shell drops key repeats of F6, F9 and F11 (119796e5)
- DASH1: side dash, dash_down, dash_up on the 22 humanoid sheets (5 frames, 40-50-60-50-50 ms = DASH_MS, not looped; speed lines on the side view only, dust on all views); the side row sits before the wind-up rows; export_units check_dash; tools/sprites test_dash.py and dash_board.py; existing rows pixel-identical; no Rust change; seen in game right, left, up and down, warrior only (cbdbc077)
- VIEWKEYS1: debug-build hotkeys: F6 cycles the map zoom 1x, 1.5x, 2x on the region map; F9 cycles the window 1280x720, 1600x900, 1920x1080 in Windowed mode on the region and world maps, battle, game menu and main menu, in Borderless or Fullscreen only a note; each saves and shows a note. Walk speed: step = base / anim_speed, base 130 ms (was a fixed 90), a debug Walk step slider 60-240 ms; the walk linger follows the step; a release-build unused import warning fixed (0c459186)
- LAYOUT720-E1: View.g, GRID, Grid, Screen::grid() (default Old, exactly today's view); shell per-screen view, the mouse derived from the physical position when the view changes; ring, lists, ghost, scroll and drag thresholds in the top screen's units; ui_tree reports canvas and grid; s * g strokes and nine scale; View::nine_content; screen_in and region_in take the shown screen's grid. No screen moved, no golden edited; stage.rs wide_line deferred to the Stage's move (5c9855a5)
- SUNKWARD1: the sunk_ward world shell: manifest, region, 4 factions, the hub sluice_undercroft, nodes 01, 03, 04, 06, 5 routes (two marked placeholder links) through seed sunkward1.json; start_party empty until CAST1-A4, so the world stays hidden from New Game; encounters arrive with CAST1-A6 (dbb5cbbb)

Under the hood:

- Docs: kit done measure and UI authored by a design agent; LAYOUTDATA1 proposed (8d6acbb2)
- Docs: WALKRUN1 as built (41eab430)
- Docs: WALKRUN1, TURNFEEL1 rows; HOOKS1-A parked on its branch (a7cf7396)
- Docs: LAYOUT720-E3 and VIEWKEYS2 as built (162ec350)
- Docs: DASH1 as built; TOOLTESTS2 proposed (a0702993)
- Docs: decision "Engine goal: agents build, humans judge feel" (08b3c38a)
- Docs: human mockup and feel tools are worth keeping (user 2026-11-03) (23a3edab)
- Docs: the test for a debug feature (user 2026-11-03) (5f162a72)
- Docs: decision "No upkeep on unused debug tools"; CLN-71 idea (dcec0f5c)
- Docs: VIEWKEYS1 and WALK2 as built (864f041a)
- Docs: LAYOUT720-E1 as built; E3 adds shell.rs, Q clips the quadrant (9ecca2f1)
- Docs: NPCFACE1 row and preference: NPCs face down by default (e66691da)
- Docs: DISPLAY2 row: Borderless and Fullscreen as toggles (user 2026-11-03) (4921e478)
- Docs: SUNKWARD1 as built; CAST1-A4 warned about New Game and journeys (7ed59a06)
- Docs: VIEWKEYS1 row (246814a2)
- Docs: plan strip mock C picked (VISCLAR1-B, preferences/ui.md) (2e79c883)
- Docs: decision "Feel run answers"; TACTIC2 approved by role and class; SIMCOUNT1 row (981cb621)
- Docs: decision "Ashlands is a test bed"; backlog pass applied: classic balance rows dropped, rewrites and merges, OBJ1-U joins M2; LEVERS1-K idea (83e8fc35)

## 2026-11-02

In the game:

- **Title screen**: a new backdrop and logotype, with the crest's eye as the A in the name.
- **Game icon**: the crest is now the window, taskbar and .exe icon.
- **New look**: screens and battle use the new palette and fonts, with lighter secondary text that is easier to read.
- **Region map ground**: maps can mix ground types with natural edges, first on Warden Keep.
- **Region map camera**: the party glides between tiles and the camera follows smoothly; it stops close to the map's edge, and props stay inside the map.
- **Map zoom**: a Map zoom setting in Settings > Gameplay, 1x to 3x, zooms the whole scene.
- **Sprite size**: units now stay the right size at every resolution; at 1080p they used to draw too small.
- **Battle AP**: rounds start at 10 of 12 AP, the pip gauge groups AP by planned skill in its kind's colour, keys 1-4 select party members, and Shift+1-9 pick skills.
- **Frames**: panels and popups can use the new ornate frames, tunable in the Gallery's Frame tuner.

Every change that touched the game:

- LAYOUT720-E2: aegis_layout (no_std, no dependencies): Rect moved and re-exported by aegis_ui::canvas; relative layout by shared edges over any area (Part, resolve, Stack, pick, Rect::{pad, place, at, offset, cut_*, split_x, split_y, grid}); aegis_ui::shot_diff; authoring note reactive_layout.md. No screen moved, no caller changed; wasm32 not compiled (target not installed) (ec66d73c)
- Warden Keep: stores_crates moves to (38, 29) on the south wall, so its art stays inside the map in both views (user 2026-11-02: props stay inside the map; REGIONVIEW1's commit named this move, which lands here) (01472b61)
- REGIONVIEW1: the region map's camera, edges and zoom: the party glides between tiles on the step's schedule (a real 90 ms step), the camera follows in whole physical px; clamp with a margin of 1 cell, 2 rows of wall past the last walkable row; a map that fits centres; pans and the talk camera ease 260 ms, none under reduce motion; the outer ring and past the map are a flat border under everything; prop art past the map cut on its edge; arrows never stack; map zoom map_zoom, 1x to 3x in 0.25 steps, default 1x, the whole scene from its cell; engine ground.rs blit_scaled for maps past PHYSICAL_MAX; Warden Keep's crates moved inside the map (38, 29) (7ee7c2ae)
- SPRITEFIT1: a unit sprite fills its exact size at any scale: nearest at an exact multiple, between multiples nearest at the next whole number then a true area average (Image::fit_square, Image::reduce_box); region sprites now keep about 5.8 % of the window height at 720p, 1080p and 1440p (1080p drew them at about 4 %); battle draws as before; floor expectations retired in art.rs and stage_pieces.rs (decision "Fixed canvas, map zoom and exact sprite size") (d31f658b)
- LOOK1-G2: emberwake_gui reads the theme: 13 colour constants and chrome literals migrated, 46 game_colors by use with today's literal as fallback; 15 minor and 14 E1B roles set; new role grip; playing raised to 4.87:1; dim #9ca3af (user); mono ships the static JetBrains Mono Bold v2.304; contrast_pairs 40; story::palette() follows the theme (a69d32b8)
- ICON1: the game's window, taskbar and .exe icon from the crest, set per game: game.json `icon` (optional .png path over the layers); aegis_ui::icon averages and sharpens to at most 256 px (Windows: 32 px window, 64 px taskbar); a missing or bad file is a notice, not a crash; the .exe icon comes from emberwake_gui/build.rs with windres or rc.exe, no new crate; tools/game_icon.py writes the committed res/emberwake.ico (6 sizes; whole crest down to 32, arch at 24, eye at 16) (b7fb090d)
- LOOK1-FT: the frame tuner: an engine pane in the Gallery: frame role, panel size, corner size, tip size, line width and colour, padding and straddle on a live preview, a read-out of the values, Save and Revert in debug builds; new generic sidecar keys; gen_art.py keeps tuned keys unless the slices changed (1f47d05e)
- REGIONGROUND1-D1: the 14 ground rows and Warden Keep's ground layer (ash, packed dirt, flagstone, dirt road, mud, paved road, water), the 9 Ashlands textures; wear halved, strays off and patches halved after the approved scene (712e1ead)
- REGIONGROUND1-T2: board.py --mix and the pair sheets go through ground_compose (legend in the kit plan's order, patches included), the Python edge deleted; --wear scales the table's wear, --props draws props and units sorted by feet; the table follows the first map's softer values; tooltests limit 300 s (037adb5e)
- REGIONGROUND1-L: the region map draws a map's ground in both views: job and key only when map, view, sets, data or art change, a flat colour with no art, the location's tileset wired; Art::generation; Reload data opens a new Art (75c9266a)
- REGIONGROUND1-E2: aegis_ui::ground draws random patches (dropped whole near another ground or a blocked cell or past the cap, fainter as they grow), placed patches and random marks; E1 pins unchanged (81990a56)
- LOOK1-A8: the ornate frame kit: one corner mirrored to four, the kit tip's head at 2/3 lit from the corner's spikes, a red line the whole edge, corners following the window; ornate_full, the kit's edge tiled as a whole frame, unused so far; the arm keys removed (f8ba5bc5)
- EFF1: tools/fc.sh docs reads changed files only, --all reads every file (one wc call, 80 s to under 1 s); tools/fc.sh tooltests with a time limit and a visible skip; both gates gain docs --all and tooltests; CI installs Pillow (a332fd8e)
- REGIONGROUND1-K: the grounds kind, map keys ground, ground_legend, ground_patches and wall_storeys, a location's boundary and a world's material_family, checks G1-G10 and A16-A20, art lookups and a pure ground plan; maps.json re-saved through the writer with empty keys (ae422797)
- REGIONGROUND1-T3: flatten.py --colour sets a texture's mean to its ground's colour; groundcheck.py pair fails grounds too alike, a stones warning for built grounds; named colour caps for the dirt grounds and tallow; board.py --mix, a one-picture map (406d3f02)
- REGIONGROUND1 art: the test terrain and frame kit takes, and the second Ashlands ground batch (Drive queue, prompts beside them) (877856a1)
- EFFICIENCY: blank lines around a heading after a docs merge (0f7467b5)
- engine README: one theme line, 50 colour roles (a merge kept both counts) (1128cf11)
- LOOK1-R1: place the title art: title backdrop, title mark, crest, banner assets; the high-resolution ornate frame straddling its panel (arms per edge with limits, one red frame_line, content rects from the frame); image_steps and whole_steps removed (a6735545)
- REGIONGROUND1-E1: aegis_ui::ground: the ground composer (edges, wear, strays, bites, sunken rims, a map-wide tone), its cache and blit, soft_ellipse, the Tiles page's Ground switch, the ground_compose tool (adc1a394)
- REGIONGROUND1-T: tools/tiles flatten.py, groundcheck.py (broad tone, banding, direction, blotch, contrast, colour), cut.py --form ground and region_tall_face, ground boards, read_contrast, the Ground texture prompt template (37a5fa00)
- CLN-50: remove the Manual planning mode (RoundState.manual, set_manual) and the host's manual and fill requests; readers use picks; fill_rest stays; goldens unchanged (e32f81cb)
- CLN-52: emberwake_gui battle.rs split as a pure move: a 1,107-line battle.rs (screen and wiring), battle/{board,draw,flow,input,marks,overlays,playback,talks,tips,tokens}.rs and battle/tests/ (21 test modules); 412 tests before and after, only the test module paths change (23120f87)
- LOOK1-A4 art: re-rolled stage tiles, low contrast with no grid (prompts beside them) (47eb7494)
- LOOK1-E1B: the desk windows, timeline, recordings, vignette, note box and pip gauge read the theme: 14 new generic roles defaulting to the values drawn before; 99 frame hashes recorded before the first edit stay unchanged (6568257f)
- LOOK1-T2: tools/tiles/cut.py box reduce, --form for the stage and region tiles and faces, optional --colours; artqueue records and failures name the queue used (7017cbc1)
- LOOK1-A6 art: the crest letter A takes and the final title mark; decision "Title mark: crest letter A"; row LOOK1-R1 (0fcae468)
- LOOK1-A6 art: card banner takes and the crest-in-the-name concepts (Drive queue, prompts beside them) (eafcbb04)
- EXPLOREVIEW1-L: the region_view debug setting and the tilted region map (mapview geometry, squashed tiles, wall blocks in one feet-sorted pass, Developer select and a debug-build F7), square unchanged and forced outside the Scene Runner (899afcf0)
- LOOK1-M1: title and main menu: backdrop composed once and blitted, text-only menu rows (label-wide hit area, one active item, no arrow), the Continue peek on hover or a keyboard move, ui/logo_title.png with a text fallback (35d01659)
- LOOK1-L1: battle stage sets: tiles and the wall face from the world's stage_set, transition pieces on two-ground corners after the ground pass, clipped to the board; an empty set draws today's frame (5321586f)
- LOOK1-A6 art: backdrop B and logotype B with the scattered fire taken out (Drive queue, prompts beside them) (f28a18ec)
- LOOK1-A4 art: second stage tile chain, straight down (prompts beside them); decision: tiles must not show a grid when repeated; row LOOK1-T2 (3b28ece6)
- LOOK1-A6 art: title backdrop, logotype, crest and ornate corner takes (Drive queue, prompts beside them); Tile template, tiles prompted square and straight down (1ecd0255)
- PROPS1-A2 fix: the kit's sample prop takes move to props/sample_camp_black and props/sample_camp_tan (the layered tree's props/`<set>/<art>/v<n>.png;` one folder too deep failed the_assets_follow_the_layered_tree) (204dfd51)
- PROPS1-A2 (S1, S3): the first Gemini prop sheets cut into the kit and game (the user's black war tent set), both camp takes kept as kit samples; sheets.py props --reduce scale (ca9d43c6)
- LOOK1-E2: aegis_ui::autotile (corner masks, block edges) and the Gallery Tiles page (c7e5a959)
- LOOK1-T: tools/tiles (cut, seam, masks, compose, board): the scale-only tile cut, seam check and fix, deterministic transition pieces and review boards (8c477274)
- LOOK1-G1: the Tallow and Iron game theme (ui/theme.json), 14 code-generated frames, the picked fonts with their OFL licences (07b2e76a)
- LOOK1-ME: engine menu panes (aegis_ui::panes: Backdrop, MenuColumn, TabRail, SlotCard, themed ask) and the Gallery Kit widgets Panes page (a242f801)
- LOOK1-K: tile set binding (tileset on world locations, stage_set on the world, lookups, checks A13-A15, schema flag omit_default) (d6eab816)
- LOOK1-E1: the engine runtime theme (aegis_ui::theme: layered ui/theme.json, 35 colour roles with today's values as defaults, body/display/mono faces, layout values, nine-slice frames with sidecars), the Gallery Theme page (d198a191)
- INPUT2: playtest battle input: style D AP pips (fixed slots, a capsule per skill block by kind), battles start at 10 of 12 AP, keys 1-4 select members (skills on Shift+1-9), the ghost-click fix, F accepts the ghost, a ghost block click accepts up to it, Ctrl+click supports an ally (0a668f8d)
- DIR4-R1F: the front and back walk show their steps (visible foot lift and plant, an outward knee, a stronger bob) for every humanoid body (78f89618)
- WALK2: region map hold-to-walk steps at once and chains steps while a direction is held (engine Held.dir and Screen::hold_walk; the shell drops OS key repeats for hold-walk screens) (0734ad7c)
- CLN-46: screens and host read the ruleset rows, not is_classic() (Dojo rules picker, Arena lists the rows, host state always names the ruleset, Dojo reset by the deck rule, Rest preview by rest_at_round_end) (b654ad21)
- MAPNAV1-U: region map edge arrows, legend-row pan (a second click walks), the click-to-walk refusal, and switching the exploring leader (Q or a click on the leader; saved as explore.leader) (fab92d8c)
- PORTRAITFX2-FA: the four new face rows (determined, happy, sad, surprised with their motion) and seven face cue rows (battle start, round start, big skills, ambush, ally down, victory, unspent on the roster) (9c290349)
- FOUNDERPORT1-P and the founders' expression frames: six Gemini portraits on the Hearth backdrop and 48 aligned faces with sidecars (55a5fde4)
- ARTQUEUE5: the Drive queue is artqueue.py's default (artqueue.json; --backup for .image_pipeline), references copied into refs/, artqueue.py attach lists what to attach (2e686b46)
- PORTRAITFX2-F: the face list is game data (DEFAULT_FACES as kit defaults) and the battle and roster triggers for the new faces (d6feced1)
- PORTRAITFX2-T: sheets.py faces --cells a,b,c,d (any four snake_case face names) and --align-to IMAGE (every cell aligned to a given raw neutral cell) for the second expression sheet (4f993356)
- CLN-51: closed, superseded by INPUT1-U slice 3 (the battle screen's Deferred previews through the host's own round_intent_plan::preview_round and ai::plan_party_intents) (edfcd36e)
- FOUNDERPORT1 art: Hollis's graft fix (a new base with the graft eye unmistakable, now his pick) and his eight faces as single edits of it (6d4eb9c5)
- FOUNDERPORT1 / PORTRAITFX2-A art: both expression sheets for the six founders (Gemini web-app queue, the founders' picks attached as references) (70aa8de8)

Under the hood:

- Docs: INPUT1-MB feel run report; REGIONVIEW1, LAYOUT720-E2, PROPS1-A2 as built; VISCLAR1-B with the user's strip colours; STRUCTURE and LAYOUT720 notes (ada01bdd)
- Docs: CLEANUP: Later backlog pass, CLN-54 to CLN-70 proposed (05c62901)
- Docs: decision "Battle view answers"; DIR4-R1F closed; VISCLAR1-B, POISE1, HOLDART1, BATTLEMAP1 rows (74bebe80)
- Docs: decision "Priorities after the visual pass"; MILESTONES: the visuals the slice uses join M2, DIR4-R4 and ENCOUNTER2-CA to M2, TACTIC2 out of M1; SUNKWARD1's dependency loop removed; stale rows corrected (a39ccb56)
- Docs: ROADMAP: the LAYOUT720 row in the table's columns (ff71e815)
- Docs: LAYOUT720 approved (option B); decision "Engine layout is relative"; slice rows; ROADMAP and HISTORY rows (3d0b87ca)
- Docs: LAYOUT720 screen table: only the main menu is finalized (user 2026-11-02) (586ef31f)
- Docs: LAYOUT720: the confirmed relative model; images, minimum caps and space priority named as later work (930a10d3)
- Docs: LAYOUT720 spec, proposed (screen table, options, slices) (c9e277ae)
- Docs: SPRITEFIT1 as built; HISTORY row for the floor fit (cd895dc6)
- Docs: LOOK1-G2 as built; mono is the static Bold of v2.304 (f552c0f3)
- Docs: WEBPROBE1 not scheduled; PC build with hotkeys first, touch later (c910605d)
- Docs: LAYOUT720-S widened to reactive layout; WEBPROBE1 proposed (77f7babe)
- Docs: dim text is #9ca3af (user 2026-11-02) (1c983548)
- Docs: decision "Layout grid moves to 1280 x 720"; LAYOUT720-S and ARTSCALE1 rows (b944e827)
- Docs: ICON1 as built; STRUCTURE rows for the icon, the frame tuner and ground.rs (70675e9d)
- Docs: map zoom is an accessibility option, default 1x (decision "Fixed canvas, map zoom and exact sprite size") (602a3399)
- Docs: decision "Fixed canvas, map zoom and exact sprite size"; SPRITEFIT1 row; zoom in REGIONVIEW1; resolution rules in LOOK1-M2 (738ce0e9)
- Docs: LOOK1-FT and REGIONGROUND1-E2, -L, -T2, -D1 as built (c25b13fb)
- Docs: decision "Region camera and terrain use"; rows REGIONVIEW1 and ICON1 (e8bfbdda)
- Docs: decision "Compact frame for side-by-side panes"; LOOK1-M2 takes it (016f177a)
- Docs: a compact frame style for side-by-side panes (preferences) (2c1f5bfd)
- Docs: which screens take the full frame (preferences, LOOK1-M2) (a47942d6)
- Docs and art: decision "Ashlands ground set: first picks"; the re-rolled grounds (a2d77257)
- Docs: decision "Ornate frame kit", HISTORY row, LOOK1-FT re-scoped (7d155f07)
- Docs and art: LOOK1-R1, REGIONGROUND1-T, -E1 as built; ornate rule corrected; frame_line in LOOK1 3.2; test terrain and frame kit takes (9fe1de52)
- Docs and art: row LOOK1-A8, a new frame kit; the first A1 ground takes kept for reference (c8c61c54)
- Docs: decisions "Efficiency pass" and "Ground textures: pattern from art, colour from code"; EFFICIENCY section; rows REGIONGROUND1-T3, GROUNDTUNE1, EFF1 (df18a423)
- Docs: decision "Two kinds of wall"; material families and doorways in REGIONGROUND1_SCENE; row REVEAL1 (5c132d80)
- Docs: REGIONGROUND1 as built notes from the tools review: natural gain steps, two dirt hexes, calibration (07a0b813)
- Docs: region boundary walls are their own art (REGIONGROUND1_SCENE, decision) (eb4f167a)
- Docs: decision "Wall storeys and walls beside props"; REGIONGROUND1_SCENE takes it (f7b1ae0e)
- Docs: row LOOK1-FT, a frame tuner in the UI Gallery (5ee3d25f)
- Docs: decision "REGIONGROUND1 answers"; REGIONGROUND1 rows approved, ROADMAP row, HISTORY rows; PROPS1-A3 split; LOOK1-L2 narrowed; Tilted is the pick (5c030677)
- Docs: REGIONGROUND1 spec: region maps with several ground types, natural blends, tall walls and the terrain hierarchy (status spec, questions open) (b5b3e92d)
- Docs: decision "Ornate frame rules"; LOOK1-M2 takes the story card banner and alternate corners for wide panels (7dc12b40)
- Docs: CLN-50 as built; HISTORY row for the Manual planning mode (a8afce84)
- Docs: decisions "Art scaling by asset type" and "Ground blends: a generated edge with light wear" (ce76c0a0)
- Docs: decision "Tiles must connect naturally on all sides" (e2840326)
- Docs: decision "Region maps mix ground types; layout before tile sets"; CLN-52 as built (7ee78a16)
- Docs: LOOK1-E1B as built and its 14 roles in LOOK1 3.2; LOOK1-G2 takes their game values and two contrast notes (7790a6e6)
- Docs: decisions on the story card banner, EXPLOREVIEW1-L click areas and LOOK1-L1 corners; LOOK1-T2 as built; spec notes (7dbb500a)
- Docs: decision "Title backdrop and its fire layer"; row LOOK1-M1F (3703c5ff)
- Docs: decision "LOOK1-A6 picks: corners, crest, story card banner, title"; main menu preferences from the second M1 playtest (label-only hit area, one active item, peek stays while Continue is active) (851345a3)
- Docs: art preference: fire in generated art must look placed (user review of the title takes) (d1e1df71)
- Docs: decision "Prop sheets by size class; tile textures straight-down"; row PROPS1-A3 (a2c1dfb8)
- Docs: main menu preferences from the user's M1 review (text-only items, the title as an image, the peek waits for hover); LOOK1-A gains the logotype (02e6c8ff)
- Docs: decision "LOOK1-G1 points and the first prop picks"; PROPS1-A2 row updated (the black tent set for the game, both takes as kit samples, a scale-only cut) (7dabb97f)
- Docs: TASKS tidy after the LOOK1 landings (stale LOOK1-M and LOOK1-L rows gone, G1 row restored; ME and G1 follow-ups placed on M1, M2 and G2) (4fc4b759)
- Docs: EFFICIENCY row for engine-wide partial gates under three busy lanes (wind-down 2026-11-02) (683a308b)
- Docs: decision "LOOK1 fonts" (Alegreya SC Bold headers, JetBrains Mono Bold badges); LOOK1-F done (ae3379da)
- Docs: decision "LOOK1 and EXPLOREVIEW1 spec answers" (all as recommended) (2607f81c)
- LOOK1 revised to the answers (LOOK1_TASKS slice briefs) and the EXPLOREVIEW1 spec; LOOK1 rows split (F, G1/G2, M1/M2, L1/L2), EXPLOREVIEW1 -L/-R; HISTORY row for the options not adopted (dff405e2)
- Docs: decision "LOOK1 answers" (all as recommended); LOOK1 rows and ROADMAP approved; HISTORY (the navy palette replaced); assets-world preference (Sunk Ward tiles first) (11c29695)
- Docs: decision "Playtest 2026-11-02: AP gauge, party hotkeys, ghost fill"; rows INPUT2, WALK2; DIR4-R1F in-combat notes (b457e379)
- LOOK1 options spec (design lane): visual pass (theme, menus, Sunk Ward tile sets); ROADMAP row and LOOK1 task rows (spec, waiting for the user's answers) (6becfc14)
- Docs: ARTQUEUE5 done (45c31b1e)

## 2026-11-01

In the game:

- **Battle input (timeline)**: click a target (enemy, ally, empty tile, or your hero with the Self button or S), then pick a skill from the menu; the round's intent follows from the skill, the rest of the round fills in as ghost blocks, and Settings > Gameplay has "Auto-fill the rest". Keys 1-9 pick skills, Z undoes, Clear removes all picks; the intent buttons, Manual and Fill are gone.
- **Selection and inspect (timeline)**: right click any unit to inspect it (only what your side knows); Esc steps back menu, log, inspect, selection; clicking the selected hero selects nothing.
- **Maneuver skills**: Flank Cut, Hamstring Cut and Slip Strike dash to the target's flank or back and hit from there; a blocked flank makes the skill miss with its AP spent; every skill chases a target that steps 1 tile away.
- **Battle talks**: a dialogue can open mid-battle as an overlay; the fight pauses under a veil and the speaker gets a ring.
- **Portraits**: portraits breathe and blink on the dialogue box, story cards, the Character screen and the party list; a wounded hero's portrait is greyed.
- **Party and Results (new campaign worlds)**: wounded marks, a bunks chip, and Results lines for captured or fallen heroes, keepsakes and the wounded; a mission board pane with deadlines and job markers on the world map; travel modes (stealth, forced march) on the world map.

Every change that touched the game:

- ARTQUEUE1: our side of the Antigravity image queue (artqueue.py request / text / ingest / status) (9e19249a)
- CLN-49: remove the 20 executed seeds of finished tasks; the seeds README indexes them with their last commit (df0adbfa)
- CAMPAIGN1-TM2: a save naming a removed travel mode loads in the default mode with a note; one leg-days helper (cf075e15)
- INPUT1-U slice 3: selection and inspect (INPUT1-U done); the round-intent preview and the hotkey target rule move into the kit (98b1e455)
- PORTRAITFX1-X: the screens read the expression keys (dialogue nodes, line cards) through face_chain (fd53135e)
- TERMS1-L: the terms register, the content.terms kind and one resolver; kit texts read the world's names (50e08c6a)
- CLEANUP.md Pass 3 (CLN-36..53): retire Godot parity, classic as a ruleset row, other cleanup; questions for the user (51a3bc27)
- ARTPROMPT1 done: routes by asset type and the winning prompts in ART_PROMPT_TEMPLATES.md (after the Gemini, local and face-fix tests) (79ba432f)
- INPUT1-U slice 2: the battle screen's one-flow input (target then skill, ghost fill, Auto-fill, keys 1-9, tactic ghost); the old round controls removed (b0c195f9)
- known-issues: the kit parity tests live in the game façades; landings stop on code conflicts (2026-11-01) (9057c089)
- ENCOUNTER2-C2: the battle talk overlay (one shared talk box for the region map and the battle; the battle pauses while a talk is open) (f0fae089)
- PORTRAITFX1-A: sidecars for the 42 portraits (anchors, idle overlays) and the portrait cue rows (8535fe0f)
- PORTRAITFX1-R: animated member portraits on the Character screen and the game menu's party list (6fd4159f)
- INPUT1-U slice 1: the one-flow battle input in the kit, host and engine (picks, ghost fill, Auto-fill, one regular move, AP forecast, the ghost segment) (17b6c654)
- INPUT1-MA: Maneuver skill data on range and position; the timeline ruleset's melee_diagonal and flank_protected_cost (7e3a09d0)
- CAMPAIGN1-TM: travel modes (stealth by default, forced march) as a world manifest block (db94b5bd)
- INPUT1-M: the Maneuver rework (position flank or behind, one range rule with a 1-tile chase for every skill, flank protected, the dash event, melee_diagonal) (1e6580ca)
- ROSTER1-U: wounded marks, the bunks chip and the departure, keepsake and wounded lines on Results (27fc3e6e)
- ENCOUNTER2-C1: the battle talk phase effect, triggers round.block / downed / broken, Session::talks, checks (bb80e9fa)
- ENCOUNTER2-W: stay-behind results through the roster (then capture or kill, the flag, result lines); the party settle moves into the kit for every runner (8e8435a0)
- TERMS1-G: layer check G3, kit code names no lore word (world ids and `games/<id>/lore_words.txt`) (4e2fa8ee)
- PORTRAITFX1-D: animated portraits on the dialogue talk box and the vignette line cards (38e51a4b)
- STORYUI1-U2: the mission board pane, the board service glyph, world-map job markers, the UI Gallery's Mission board (ffd0f277)
- ROSTER1-L slice 4: bunks and the bench, join level, party_min, departed recruits refused, roster read keys, the over-cap load warning (ROSTER1-L done) (d6775264)
- CAMPAIGN1-SIM: aegis sim deadlines, best and worst finish day per checkpoint (7dafe18c)
- Founders: the Gemini takes become the portrait picks (Klein picks kept as `<id>_klein_pick);` decision "Founder portraits from the Gemini route"; FOUNDERPORT1-P updated, PIXEL4 superseded (7f0c09a8)
- FOUNDERPORT1 and PROPS1 art: the first Gemini web-app batch (open-ended founder takes, the buildings and Ashlands prop sheets) (e5f17332)
- CLN-48: the legacy wave_* fold shim goes (fold_legacy_wave, LEGACY_WAVE_KEYS, source::folded and fold_legacy_waves and their call sites) (fe3f51a7)
- PORTRAITFX2-M: motion profiles per face (engine Motion, kit fields on portrait_expressions, game binding); hurt, angry and afraid move as winces, heaves and shivers (91fc9bcc)
- PORTRAITFX1-TL: the timeline widget's icon square (engine) and unit portraits on the full timeline view's tracks, minimal tier (9dcddd6a)
- CLN-45: the switches bundled under round_intent become ruleset row fields (rest_ap, chase, root_stops_chase, switch_target_step, stances); melee_diagonal reads as set (f6e0fb5a)
- PORTRAITFX1-XA (7 of 13): expression frames and sidecars for tess, bram, aline, captain_vane, proctor_quell, guild_master_orlov and inquisitor_prelate_nemeth; the six face cue rows (dde61706)
- PORTRAITFX1-P: portraits on the battle hero and intel panels, reacting at FX1's impacts (5e40661c)
- PORTRAITFX1-XA tool slice: sheets.py faces --reduce scale (no palette pass, faction backdrop), --align (cells scaled and shifted onto the neutral figure) and --cell name=image (20811b47)
- known-issues: parallel agents and the shared shell cwd reset (1a11fff3)
- CLN-44: classic is a rulesets row (the closed ruleset lists go; a missing classic row falls back with a load note); the classic row shipped through aegis content (876964f2)
- ARTQUEUE4: the prompts-file reader takes each id's own section (the props file had sent each sheet the wrong block) (6f96564b)
- ENCOUNTER2-C4: the cutscene screen for scripted rounds (locked controls, banner, Skip sends resolve, auto-advance) and the game's tactics for the scripted-round draft (0eb227cc)
- CLN-43: docs and rules after the end of Godot parity (CLAUDE.md engine-golden rule, skills, agents, workflow docs, the session hook without Godot) (a78fdf41)
- ARTQUEUE3: artqueue.py ingest archives what it ingested (archive/`<date>/{output,completed});` archive sweeps leftovers; status counts the archive (d6b55864)
- PROPS1 art: first prop sheet takes from the Antigravity queue (kit props, a buildings take, and a kit-like take); row ARTQUEUE4 (the prompts-file reader mixed sections) (e4414a03)
- PORTRAITFX1-XA art: six single-cell edits of the neutral cell (Bram angry and afraid with the loupe, Aline angry, Nemeth angry and afraid, Quell neutral without the smoke) (9f5e8f06)
- TERMS1-U: the screens read the terms (Results, journal, shop, party sheet, region chest, HUD helpers; intent names through label everywhere) (2897a7cf)
- CLN-42: Godot switched off (the Godot-only parity suites, the parity/ recorders, fc.sh parity, the CI Godot steps and the export-only CLI keys and loaders go) (10f71aed)
- ENCOUNTER2-C3: scripted rounds (the encounter `script` field: planned at round start, locked or guided, per-entry fill, fallbacks, metrics, content checks) (2bae8eeb)
- CLN-47: saves below the current version are refused; the development migration steps and the Godot JSON text emulation go, the release step hook stays (f5159a37)
- PORTRAITFX1-XA art: expression sheets for the seven story faces (Antigravity queue, batch 2) and the prompts file (923e7cd9)
- INPUT1-L: per-opener follow-up lists in the deck (the list first, then the deck order) and their editor (1cda9103)
- CLN-41: journeys become an engine-owned random-walk golden (seeded walks through the whole game loop with a real save round trip) (c2b60b6d)
- CLN-40: the content-checks parity becomes an engine-owned golden (1,012 representative broken-entry cases on frozen data) (70accd92)
- INPUT1-PB: the playback aids (impact tracers, the strip's block cursor, 1x / 0.5x / Block speed, the log grouped by block, T holds playback for the timeline view) (52b17336)
- CLN-39: sim parity becomes an engine-owned golden (22 sim runs on frozen data); the sim help lists its engine-only modes and keys (b03fd894)
- ARTQUEUE2: artqueue.py shows the Antigravity quota pause (paused.json); the contract's quota rule documented (968dcf8c)
- PORTRAITFX1-EX: runtime expression effects (a face tint, a mark and a shake per expression) from a data table (80b9338c)
- INPUT1-T: the full timeline view (key T) and the UI Gallery's timeline pane, built by one shared builder (aead07b8)
- Row ARTQUEUE2: the image queue's quota pause (user 2026-11-01) (3675fc2a)
- CLN-37A: re-save the nine world files saved before newer fields existed; store.rs compares every shipped file (f58f40af)
- VISCLAR1-A: the calmer default board (one quiet target arrow per unit while planning, bars for the focus set, Alt shows all; the first-move chip behind a setting; debug-only Party foresight off) (2a9d4802)
- CLN-37: the writer check becomes "every shipped file re-saves byte for byte" (the Godot resave comparison and its AFTER_GODOT lists gone) (2adba00e)
- CLN-38: engine-owned RNG and hash golden (aegis_core) and the rules golden on a frozen catalog; emberwake_rules parity test gone (4f9771b1)
- INPUT1-AP: the AP pip gauge beside the strip (engine pips widget, the kit's structured AP shortfall) (afaf45b8)
- CLN-36: the host tests read the shipped data instead of the Godot export; one-off Godot-data checks and timeout_verdict deleted (6414b3ba)

Under the hood:

- Docs: today's rows dated 2026-11-01 (were 2026-11-02 by mistake); decision "Save upgrades at game version updates"; CLN-47 keeps the migration mechanism (5a63096b)
- Decision "Antigravity queue as the default image route" (user 2026-11-02); ART_PROMPT_TEMPLATES routes note (35e2612f)
- Decision "Godot parity retired and cleanup pass 3 answers" (user 2026-11-02, all as recommended); rows CLN-36..53 (6f7b8cc1)
- Decision "Antigravity image queue" (user 2026-11-02); row ARTQUEUE1 (fd7278a5)
- Decisions "Saves and data edits", "INPUT1-M rule choices and the classic ruleset", "ARTPROMPT1 routes" (user 2026-11-02); row CAMPAIGN1-TM2 (4efb30cc)
- INPUT1 spec: answers part 4 applied (one range rule with a 1-tile chase for every skill, the position attribute, the dash on the windup tick); rows INPUT1-S done, -M, -MA, -U, DASH1 updated (bb3ef615)
- Decision "INPUT1 answers, part 4" (user 2026-11-01: one range rule for every skill, position attribute for Maneuver, dash on the windup tick); rows DASH1, TACTIC2 (M1) (0288abcd)
- Decisions "ENCOUNTER2-C answers", "TERMS1 answers", "ROSTER1-L slice 4 answers" (user 2026-11-01, all as recommended); ENCOUNTER2-C and TERMS1 rows approved (d2d115f3)
- INPUT1 spec revised for the answers parts 1-3 (Maneuver rework, AP gauge, playback aids; Q10-Q19 open); specs ENCOUNTER2_C.md and TERMS1.md; task rows INPUT1-*, VISCLAR1-A, ENCOUNTER2-C1..CA, TERMS1-G/-L/-U (06062951)
- Docs: decision "Gemini web app as the primary image route" (Antigravity as the unattended backup); row ARTQUEUE5 (8ad6633e)
- Docs: decision "Gemini web app as a second image queue" (text-only requests; references stay on Antigravity) (84177a4c)
- Docs: ARTQUEUE4 done (a0ed7c27)
- Docs: decisions "PORTRAITFX2 answers" and "ENCOUNTER2-C4 answers"; PORTRAITFX2 rows (-S done, -M/-F/-A updated, new -FA and -T), ROADMAP row, two HISTORY rows (6232759a)
- PORTRAITFX2 spec (design lane): motion profiles per face, the face list as game data, triggers for Determined / Happy / Sad / Surprised, the second sheet; open questions Q1-Q10 for the user (a8dcfa05)
- Docs: row ARTQUEUE3 (ingest archives what it ingested) (f2953438)
- Docs: decision "Portrait motion per expression and four more faces"; rows PORTRAITFX2-S, -M, -F, -A (7c8c001e)
- Docs: decision "Expression sheets: scale only and aligned cells"; PORTRAITFX1-XA row gains the scale-only reduction and cell alignment (081324f6)
- Docs: decision "ENCOUNTER2-C3 answers" (own tactic for others: auto via ENCOUNTER2-C4, a fallback cell when a locked confirm fails, the schema re-save accepted) (26e73296)
- Decision "Art goes straight to develop from the planner worktree": review boards from the pixelized game-size versions, existing tools only (user 2026-11-01) (1cec434b)
- Decision "Art goes straight to develop from the planner worktree": generation only in the main session; rigging into the game stays an asset task through staging (user 2026-11-01) (3fefa177)
- Decision "Art goes straight to develop from the planner worktree" (user 2026-11-01) (3d2d9851)
- Decision "Expression effects at runtime" (user 2026-11-01); rows PORTRAITFX1-EX, CLN-37A (2645adf5)
- PATCH-NOTES: wrap a path in backticks (markdown lint) (2c6a8582)
- PATCH-NOTES: ROSTER1-L s1-s4, ROSTER1-U, CAMPAIGN1-U s1-s2, -TW, -SIM, -TM, -TM2, STORYUI1-U2, PORTRAITFX1-G, -D, -R, -XT, -A, -X, TERMS1-G, -L, ENCOUNTER2-W, -C1, -C2, INPUT1-M, -MA, -U s1-s3, ARTQUEUE1, CLN-49 (2026-10-31 to 11-01) (e6cdb04b)

## 2026-10-31

In the game:

- **Timeline battle screen, hooks**: ground hazards and arena strike warnings on the board, unit meters as pips or bars, a held-guard edge, HP costs on the strip, push and pull forecasts (arrows, collisions, HOLD), all switchable in Settings > Battle; long hover cards in the intel panel no longer cut off.
- **Hold escapes**: a "Who holds the winch?" pick, the hold tile marked, exits read SHUT or OPEN, and a note on why a withdraw waits; an encounter whose timeout counts as a loss now ends in a loss.
- **Combat log**: phase story lines and scripted kills now show in the combat log.
- **For testers (timeline)**: the AI no longer guards twice without being hit; the test fights were retuned.
- **Campaign worlds**: HUD chips, warnings, a confirm before a step that could end the run, a run-over screen, act-start saves, and story events that play from a queue.

Every change that touched the game:

- ENCOUNTER2-U: the hold-escape screen; phase lines and kills in the combat log (19d4618f)
- RULESET1-A3: tl_ test-fight tuning pass and the kit_culling fix; the final feel run (77343bd2)
- HOOKS1-UI slice 5: the text audit of every hook addition; the intel panel keeps what fits (task done) (f3b79757)
- HOOKS1-UI slice 4: push and pull forecasts; one forced-move step rule for resolution and preview (9316d611)
- INPUT1-S: the one-flow battle input spec (docs/specs/INPUT1.md, INPUT1_TASKS.md), questions Q1-Q9 open (182a18b3)
- CAMPAIGN1-U slice 2: the campaign queue player, the battle bridge and start-AP penalty, world-map trip, camp and deadline texts, purse names in logs, the dialogue rest ask (2d748078)
- ROSTER1-L slice 3: roster rule rows, wounded, depart with keepsakes, the retreat loss, member conditions and party.departed save keys (0ac6eab4)
- CAMPAIGN1-TW: warn on a repeatable trigger with an empty when; the CAMPAIGN1-T review's test gaps (a57c5755)
- PORTRAITFX1-XT: expression sheet tools (generate.py per-block reference and --faces records, sheets.py faces) (cb01e647)
- ROSTER1-L slice 2: the kit fields and settles the party (rpgkit_world::roster), the battle screen calls it (d0f85a36)
- CAMPAIGN1-U slice 1: campaign HUD chips, notices, asks before run-ending steps, the run-over screen, act-start saves (0a98497e)
- PORTRAITFX1-G: the game's portrait binding and the Gallery's Portraits page (7ff3f161)
- ROSTER1-L slice 1: the roster schema, checks and typed readers (no behaviour change) (2fe75314)

Under the hood:

- Decision "ENCOUNTER2-U answers" (user 2026-10-31, all as recommended) (46926df0)
- Decision "RULESET1-A3 loose ends" (user 2026-10-31, as recommended): skirmish caps recorded as known; phase lines in the logs checked in ENCOUNTER2-U; VISCLAR1 takes the intel line step (b6d571d2)
- Decision "Timeline test fights after RULESET1-A3" (user 2026-10-31, as recommended): stop win-rate tuning of the tl_ fights; floors only; reach snap-first; sure-break floor to the skirmish (91a7ea1e)
- Decision "INPUT1 answers, part 3" (user 2026-10-31: melee_diagonal rule option, flank protected spends AP, names and assets layering); TERMS1 row (M2) (955e0d10)
- Decision "INPUT1 answers, part 2" (user 2026-10-31); rows INPUT1-M, INPUT1-PB, INPUT1-AP, ENCOUNTER2-C (M2) (234765f1)
- Decisions "INPUT1 answers, part 1" and "Engine scope: a rule library" (user 2026-10-31) (f0421fa5)
- Decision "Combat input feedback": the target arrow rule (user 2026-10-31: one dashed low-opacity arrow per unit, selected only by default, all on Alt) (2b1485ba)
- Decisions "Combat input feedback" and "ROSTER1-L slice 1 and CAMPAIGN1-U slice 1 answers" (user 2026-10-31, all as recommended); INPUT1-S/-U/-T/-L rows, VISCLAR1 slice A approved, INPUT1 in M1 (3af3bc1b)

## 2026-10-30

In the game:

- **Timeline battle screen**: round intents (Assault, Guard, Support, Maneuver) with keys 1-9, board overlays (target ring, reach band, planned moves, flank arc, facings, stealth veil), enemy intents and impact badges, poise bars and BREAK plates, hit forecasts, a round summary in the log, and a Battle tab in Settings to switch each overlay group.
- **Front and back views**: every humanoid unit now turns to face up and down with its own model (headgear, hair, clothing, weapon, the buckler); clicks on the Stage follow the view; the crawler and beast wind up, stagger and daze.

Every change that touched the game:

- RULESET1-B2: timeline feel rerun against the bands (BALANCE_REPORT) (907a2fce)
- HOOKS1-UI slice 3: unit meters, the held-guard marker and HP costs on the battle screen (7e081dd7)
- HOOKS1-D: the HOOKS1 docs pass and the game-agnostic combat hooks guide (00a471ab)
- RULESET1-AI2: no second low-HP Guard round without a hit (timeline AI) (c7008bcd)
- HOOKS1-UI slice 2: the class path follows the played world; out-of-battle HP under the deck rule (38e57942)
- ENCOUNTER2-D: kit sample encounters kit_winch_escape and kit_culling (695c0599)
- HOOKS1-UI slice 1: ground hazard and arena telegraph marks on the battle board; kit follow-ups (667ddc21)
- DIR4-HB: the Stage hit rect follows a unit's front or back view (788186d2)
- TEXTFIT2: the engine's text audit skips transient float texts (863dd0b9)
- RULESET1-G2: one auto driver for the sim and the battle command; sim feel outcomes and keys; the timed party driver (2a347e2a)
- EFFICIENCY: blank lines around the BUILDPROF1 heading (lost in the staging rebase merge) (314a97a3)
- RULESET1-A2: tl_ test-fight data (Skirmisher Spiker, enemy Guard actives, Crushing Judgment 1.5, tl_flank_elite) (235a5486)
- RULESET1-U slice 6: overlay settings, the [Round] log line, hovered moves above the tokens, the text audit (task done) (6b77c89e)
- DIR4-R3: front and back views for the goblin and named variants, and the buckler (art, kit + game) (6e8e0184)
- BUILDPROF1: smaller dev build caches (line tables only; no debug info for third-party crates) (40e43ea2)
- RULESET1-BX: no pierce bug; a kit test pins pierced and plain hits against two armours (ee5b12a1)
- RULESET1-U slice 5: poise bars, breaks, strip parts, forecasts, the round summary, overlay flags (72795dce)
- RULESET1-U slice 4: round intents in the tells and threat intel, impact badges, full reveal (92bd851b)
- WINDUP1-R3: wind-up, stagger and dazed rows for the crawler and the quadruped (art, kit + game) (b8f48a45)
- RULESET1_BALANCE1 as built (RULESET1-BX: no pierce bug; the 15/20 was a classic probe); RULESET1-G2 starts by rerunning the driver probes under timeline (9254a09d)
- BALANCE_REPORT: RULESET1-B timeline feel report (2026-10-30, 500 fights): Maneuver unreachable by the AI, windup and armour duels unwinnable, standards too weak, sim vs engine driver disagree; baseline numbers (3c1df593)
- Preference "Battle overlays and the clarity pass" (user 2026-10-30): overlays stay for debugging; new row VISCLAR1 (proposed, M1): a visual clarity pass once the fight logic is built (6832aba8)
- ENCOUNTER2-K: phase `kill` effect, phase groups and wave-unit hp_below triggers (kit) (bbd1733f)
- RULESET1-U slice 3: board overlays under the timeline ruleset (476e1c5f)
- ENCOUNTER2-HA: the party AI plays hold escapes; the sim's catalog sweep skips hold fights (kit) (ec1c827b)
- task-landing and known-issues: cleanup.sh removes the task's build cache with its worktree (2026-10-29: stale caches filled drive C:) (65d8864e)

Under the hood:

- Decision "HOOKS1-UI slice 4 answer" (user 2026-10-31, as recommended): push forecasts read the board as it stands (a4c53eaf)
- Decision "RULESET1-B2 tuning and HOOKS1-UI slice 3 answers" (user 2026-10-31, all as recommended); new row RULESET1-A3 (data tuning pass, with the kit_culling fix) (29ece5a9)
- Decision "RULESET1-AI2 answers" (user 2026-10-31, all as recommended): only the low-HP Guard row is skipped after an unhit guard round; an absorbed hit counts as no hit (1abf02a5)
- AGENT_MAP: wind-down 2026-10-30 (lanes routine; M1 53/62 on staging; next tasks) (5766cf20)
- Decision "HOOKS1-UI slice 2 answers" (user 2026-10-30, all as recommended): lenient save validation across worlds; deck-rule numbers on the party screen (127ff6c9)
- Decision "HOOKS1-UI slice 1 answers" (user 2026-10-30, all as recommended): pip cutoff by span; the class path follows the played world (supersedes HOOKS1-C as built); arena marks stay in playback (fec4413f)
- Decision "RULESET1-G2 answers" (user 2026-10-30, all as recommended): feel keeps start jitter; the timed driver stays single-target; RULESET1_BALANCE1 as built for G2 (f1612b19)
- TASKS: one row each for RULESET1-BX and RULESET1-A2 (done), after the staging rebase kept both copies (5420e917)
- Decision "RULESET1-U slice 6 answers" (user 2026-10-30, all as recommended); new row TEXTFIT2 (M1); VISCLAR1 takes the strip icon for short labels (7e5ea080)
- Decision "DIR4-R3 art answers" (user 2026-10-30, all as recommended for now; revisited in combat via DIR4-R1F) (c4bd2024)
- Decision "RULESET1-U slice 5 answers" (user 2026-10-30, all as recommended): poise bar and forecast placement kept; slice 6 logs the round summary and draws hovered revealed moves above the tokens (e7a8ebd7)
- Decisions "RULESET1_BALANCE1 answers" and "Builder efficiency" (user 2026-10-30, all as recommended): balance follow-up rows approved; lane worktrees with warm caches (lane.sh), builders end with the partial gate (pgate.sh); task-landing and EFFICIENCY updated (2f156992)
- Spec RULESET1_BALANCE1 (design-brainstorm, from the RULESET1-B report; proposed, Q1-Q6 open); TASKS rows RULESET1-BX, -G2, -AI2, -A2, -B2 (spec); RULESET1-B done (1ec30429)
- Decision "M1 scope and order" (user 2026-10-30): combat testing first; HOOKS1-A, FX1-A, ENCOUNTER2-W and DIR4-R4 move to M2 (a0096d80)
- Decision "WINDUP1-R3, ENCOUNTER2-K and RULESET1-U slices 3-4 answers" (user 2026-10-30, all as recommended): creature poses kept, scripted kills raise no reactions, overlay and tell choices kept; revealed walks on hover or Alt; intent icons in the next art batch (2c47a828)

## 2026-10-29

In the game:

- **Wind-ups**: units now wind up before multi-block skills (heavy, buff and debuff poses per weapon grip), stagger when interrupted and sway dazed while Staggered; every class has them (warrior, ranger, brawler, priest, rogue, mage and their goblin and named variants). In planning, enemies with visible tells hold the pose of what they intend.
- **Status looks**: statuses show on the battle tokens (stun stars, burn, poison and bleed sparks, rings, tints, stealth fade), appear when the hit lands and cycle when a unit has several; a unit killed by a status tick falls with dust.
- **For testers (timeline ruleset)**: the Dojo plays under the timeline ruleset (toggle, planner view, poise readouts, deck view); the Combat Arena can switch single rule extensions and reveal enemy plans, and shows the ruleset on its card; `sim feel` reports how timeline fights feel. Arena hazards (scheduled strikes with tells) and boss phases that unlock a capstone skill work in the kit; no shipped encounter uses them yet.

Every change that touched the game:

- HOOKS1-U: phase effect `deck` (capstone unlock) and the arena wipe-out skip (kit) (5814b796)
- WINDUP1-R2: wind-up, stagger and dazed rows for the ranger, brawler, priest, rogue and mage (art, kit + game) (c8b94631)
- known-issues: parallel agents overwrote each other's scratchpad output files (name outputs after the task) (89ced7b3)
- CLN-P2: docs audit pass 2, the planner's status and pointer pass (decision "Docs audit pass 2 answers") (2b39c1bb)
- STATUSFX1-L: status looks on battle tokens (00921067)
- RULESET1-G slice 2: Combat Arena rule switches and `sim feel` (078c03d9)
- WINDUP1-R1: rig core and the warrior's wind-ups, stagger and dazed (art, kit + game) (c5aa944f)
- HOOKS1-H: arena hazards and their tells (kit) (46858d32)
- RULESET1-G slice 1: the Dojo under the timeline ruleset (toggle, planner view, poise readouts, the Dojo deck view) (119648dd)
- Skill task-landing (user 2026-10-29): the planner's checklist for landing a reviewed task (docs needed by change type with docs_needed.py, task-row and commit formats, stage/flush/cleanup and docs-conflict rules); planner skill points to it (4b8f6b19)
- WINDUP1-L: wind-ups, intent stance poses, stagger and dazed on the battle board (2aca58e8)
- HOOKS1-M: unit meters (costs, decay, bands, rules, meter skills) and a shared bands helper (057e9c3a)
- RULESET1-DU slice 1: the party screen's deck view under the timeline ruleset (948082f6)
- FX1-L2: the rest of the battle effects (skill, support and item looks, falls, misses, the agent-link fx state) (02189d5f)
- WINDUP1-E: engine clip fields for wind-ups (loop_from, strike) and ClipPlayer::play_from / set_base_from (6f567b9c)
- HOOKS1-X: execute riders (an exploit entry matching on a low HP share) (a1799037)
- RULESET1-X: timeline golden traces and the inert-extensions check (game tests) (d0ccf0a3)
- RULESET1-U slice 2: foresight range measured at scheduling; dialogue stat checks under the deck rule (65984a86)
- DIR4-R2: front and back views for the ranger, brawler, priest, rogue and mage, and the warrior's battle axe and warhammer (art, kit) (1e281a97)
- RULESET1-U slice 1: round controls, board-click target, strip reasons and Rest gain, the Arena ruleset chip (31434d27)
- HOOKS1-AI: AI target preferences, tactics and hazard-aware tiles (kit, rule ai_tactics) (ccd0ecd3)

Under the hood:

- TASKS: drop 19 rows that docs merges revived after CLN-P2 archived them (they are done in TASKS_DONE) (cff05616)
- Decisions "RULESET1-U slice 1 and HOOKS1-E answers" and "WINDUP1-R2 art answers" (user 2026-10-29, all as recommended): round controls in the hero panel, chip hidden in plain classic, unplannable intents disabled in auto; ally_hit pulls after the action; R2 art kept as built (7aebacb7)
- Decision "Front and back views follow the side model" (user 2026-10-29): front and back views keep the side view's headgear, hair, clothing, offhand and weapon; the WINDUP1-R2 review notes go to DIR4-R2 (base classes) and DIR4-R3 (variants) (21465e12)
- Decision "HOOKS1-U unlock on an automatic deck" (user 2026-10-29): an unlock always lands, pushing out the last automatic active; content warning; later option: enemy/NPC class variants with fixed decks (0b830fb2)
- Decision "HOOKS1-H, RULESET1-G and STATUSFX1-L answers" (user 2026-10-29, all as recommended): the fight ends before arena strikes once a side is wiped; Rest dropped on a hero switch; ruleset chip on the Arena battle screen; status looks raised and strengthened (1d486666)
- Decision "WINDUP1-R1 heavy hold, front and back" (user 2026-10-29): the 2H heavy hold rests the weapon over the shoulder in the front and back views; done in WINDUP1-R2 (c00a18d0)
- Docs audit pass 2 (housekeeper) and the user's answers (2026-10-29, all as recommended) (faf33b42)
- Decision "WINDUP1-R1 review answers" (user 2026-10-29): buckler sword raised in front and back views, front debuff blade angled, 2H heavy hold lowered into the frame, greatsword tip over the shoulder from behind (1a3d2021)
- Decision "HOOKS1-M and deck answers" (user 2026-10-29: all as recommended) (ef91d4f9)
- TASKS: FX1-V and FX1-L2 done (their statuses were lost in conflict merges) (5b36e157)
- Decision "ENCOUNTER2-HA and -K answers" (user 2026-10-29, all as recommended): the hold keeper only defends, nearest hero takes over, others leave once the keeper can reach the tile; a phase kill overrides unkillable (ae30889e)
- TASKS: RULESET1-U back to "slice 2 done" and RULESET1-X layer game (a docs merge kept stale text) (a8b1bb9c)
- Decisions "HOOKS1-AI answers" and "DIR4-R2 art answers" (user 2026-10-29, all as recommended): AI tactics kept as built; brawler fists beside the jaw and a wider ranger bow, the rest kept (b173b1b6)
- AGENT_MAP: wind-down 2026-10-29 (release #51 merged; HOOKS1-AI on staging; RULESET1-U slice 1 and DIR4-R2 uncommitted in their worktrees) (501c6e2a)

## 2026-10-28

In the game:

- **Battle effects**: hits land on the attacker's impact frame, with impact looks, sparks, trails, projectiles, camera shakes, falls and misses; story cards can shake.
- **For testers (timeline ruleset)**: enemies plan whole rounds by intent under the timeline ruleset, and the kit gains triggers and reactions, riders, HP-cost skills, executes, unit meters, ground hazards and data zones (no shipped content uses them yet).

Every change that touched the game:

- known-issues: build folders committed by git add -A (2026-10-29) and the guard against it (16bfef6c)
- gitignore: worktree build folders (target-*, engine/target, nested target); decision "HOOKS1-K answers" (user 2026-10-29: both as recommended) (11a0171a)
- FX1-V: vignette card shakes (Card.shake, Player::with_shakes/camera/shake_offset, VignetteScreen::with_shakes; story::vignette_screen passes the game's table) (d83cbf0d)
- HOOKS1-K: riders, HP-cost skills and guard bypass; trigger metrics; a HOOKS1-E crossing fix (4f74ce0e)
- FX1-L1 slice 2: battle effects drawn (looks, trails, projectiles, sparks), battle shakes and the board camera (cafd8dd5)
- STATUSFX1-E: engine looping effects (Orbit, Looped, a closing Ring, Painter::image_alpha) and the Effects page loop (f930e520)
- FX1-L1 slice 1: the battle effects timeline (hits land on the attacker's clip impact frame) (4796fca2)
- RULESET1-AI: the round-intent AI for enemies and party tactics (the pick table, positioning tiers, tactics as intents) (6141accd)
- FX1-S: the shake schema (vignette card shake, encounter phase effect shake) (141904c3)
- EFFICIENCY: task chains get a fresh builder per task; a small same-file next task continues the previous builder (user 2026-10-28) (68fbad43)
- HOOKS1-E: events and triggers, held guard, redirect and intercept (rule triggers) (9eb6d42a)
- HOOKS1-Z: zones as data (encounters and road fights name data zones; board, hold tiles and backdrop) (2a254491)
- HOOKS1-C: class display names and skill paths per world (the world manifest's classes overlay) (aa69a297)
- HOOKS1-GF: ground hazards ignored when the rule is off; trap and guard tests (HOOKS1-G review follow-ups) (80525939)

Under the hood:

- Decision "FX1-L2 answers" (user 2026-10-29): no dodge miss for now; status-tick deaths get the fall dust (STATUSFX1-L) (53bad044)
- AGENT_MAP: wind-down 2026-10-28 (M0 released, M1 progress, tomorrow's 3-lane queue, open questions) (0730aa94)
- Decision "Batched gate": measured times (partial 26-91 s, full 123 s unloaded, 703 s under load for the first flush) (979de9d5)
- Decision "RULESET1-AI answers" refined (user 2026-10-28): positioning tiers (active target, then most hostiles for ranged / close quarters with most hostiles for melee, then own safety or escape at low HP) (4803d034)
- Decision "RULESET1-AI answers" refined (user 2026-10-28): the end-of-approach action is per unit: front line Guards, reckless units have no Guard, supports buff self or an ally (86fda8ae)
- Decision "RULESET1-AI answers" (user 2026-10-28): approach then Guard, Support only when hurt, cover ranked by range (target in range, near hostiles, own safety), tactics skip Manual members; HOOKS1-E answers (1dc19f88)
- Decision "Batched gate" (user 2026-10-28): task landings on a local staging branch under a partial gate, staging pushed to develop in batches under one full gate; CLAUDE.md standing rule and desktop.md Ship updated (4e6a28ff)
- Decision "WINDUP1 and STATUSFX1 spec answers" (user 2026-10-28: all as recommended); WINDUP1-* and STATUSFX1-* approved (48486ddd)
- Specs WINDUP1 and STATUSFX1 (design-brainstorm, from the user's answers 2026-10-28); TASKS rows WINDUP1-E, -L, -R1, -R2, -R3 and STATUSFX1-E, -L (spec; WQ1-WQ5 and SQ1-SQ5 open); ROADMAP rows; MILESTONES M1 (9c305c08)
- Decision "WINDUP1 answers" (user 2026-10-28): wind-up for every multi-block skill (not Defend, not Move), per grip, intent stance poses in planning, stagger for interrupts and the Staggered status, first-pass scope; new row STATUSFX1 (status effects on tokens, M1) (d7ce4802)
- TASKS, ROADMAP: EQ1-EQ9 were answered as recommended (user 2026-10-25; the 'open' note was stale); ENCOUNTER2 L, X, F, R, RA, RU, D2 marked later than the slice (the slice needs T, H, HA, W, U, K, D), so they leave M1 for M2b (3d37c4fa)

## 2026-10-25

In the game:

- **Main menu**: a new sunless main menu: the Shroud over the cathedral and the ward's roofs, with falling ash
  (no more orange sun).
- **Battle stage**: battles are drawn on a tilted stage, with the board, ground, props and a backdrop for the zone
  (Settings: battle view Stage, the default, or Flat).
- **Facing**: units in battle turn up, down, left and right toward their target or step, and on the region map the
  leader turns with every step and people face the party when you talk to them.
- **Region map sprites**: the leader and every person on the region map walk and idle as sprites, with foot shadows
  and names above their heads.
- **For testers**: the Combat Arena has a ruleset choice (World, Classic, Timeline) and the Dojo a Timeline toggle
  (the timeline ruleset has no content yet, so it plays like classic for now). The UI Gallery gains Effects (hit flash,
  camera shake), Timeline and Vignette pages.

Every change that touched the game:

- MENU2: the main menu shows the sunless picture (gen_art menu_sunless writes the game's backgrounds/menu/main_menu.png over the kit's generic path; the sunny menu drawing retires, the menu group keeps the Ashlands title card; main_menu_sunless.png removed); docs (3eea397)
- Planner: milestone releases without asking and the main checkout on main (decisions/workflow.md); known-issues row for tab escapes in Python path strings (5a99850)
- KIT1-DEPS: the game façade crates drop unused dependencies (data: aegis_core, serde_json, indexmap, anyhow; rules: indexmap; battle: emberwake_rules; content: aegis_content, emberwake_battle; world: aegis_save, anyhow; sim: emberwake_data, emberwake_rules, emberwake_battle, indexmap, serde_json); emberwake_sim::parse(d, argv) parses with the game's defaults; STRUCTURE (10ec302)
- Planner: the user approves the sunless main menu (task MENU2) and the world card subtitle (195107e)
- WTCLEAN1: tools/wt_remove.ps1 also deletes the removed worktree's build folder (`<target-dir>-<leaf>`, base from .cargo/config.toml) and gains -Targets [-WhatIf], a sweep of build folders whose worktree is gone (never the base or a live worktree's; stops if git worktree list fails); header updated for C:\dev and `claude/<task>`; desktop.md step 3, known-issues rows, EFFICIENCY and STRUCTURE (d578a23)
- Planner: RULESET1-R, ASSETCHK1, WORLDW1, STORYART1 and CREATURE1 done; RULESET1-S in progress; CAST1-S takes the spit clip mapping (3d48430)
- CREATURE1: code creature rigs (tools/sprites/rig.py: a quadruped and a segmented crawler skeleton with idle, walk, attack, hurt, death; variants.py CREATURES); bodies beast/render_beast and crawler/ash_crawler (with a spit clip) in the game, beast/base and crawler/base in the kit as the fallback; humanoid output unchanged; docs (a4253ef)
- STORYART1: code-drawn placeholder art for the slice in tools/gen_art.py (groups story, world_card, menu_sunless, sunk_backdrops, family_icons; a starless Shroud sky with ash): intro slides 1-4, the Meat Market, the hearth, the sunk_ward world card, a sunless main menu (not switched yet), the seven Sunk Ward battle backdrops, the scourge and sprayer family icons; layout_ok accepts backgrounds/story; docs (557b8b0)
- WORLDW1: the world manifest writer (`aegis content world <id> [json] [full]` and the seed step {op: world}; aegis_content set_world with the shared write-check-rollback, the Hooks world_manifest_text / world_blocking, schema::type_errors; rpgkit_content world_manifest_text, set_world, manifest_blocking; a trimmed write keeps shipped manifests byte-identical, full writes every field; a new world is listed in game.json first); docs (18ed9b0)
- Planner: WTCLEAN1 (worktree removal also deletes its build folder) and the known-issues row for the full disk (d62788b)
- Planner: non-human monsters get code creature rigs (decision, CAST1_ASSETS note, task CREATURE1) (c62f563)
- Planner: the user confirms every planner pick (OPEN_QUESTIONS section 4); story-scene art on hold, code-drawn placeholders instead (STORYUI1_TASKS 1.5, task STORYART1) (2e98702)
- ASSETCHK1: the asset check parses every palette file in every layer, referenced or not (aegis_core::assets::palette_problems with the loader's parse_palette rules; the game test every_shipped_palette_file_parses), so an unreferenced broken palette fails the gate; docs, known-issues row (9bec507)
- RULESET1-R: the ruleset switch (world.world ruleset and plan_reveal_flag; the content.rulesets kind and its checks; rpgkit_rules::ruleset with the rule register, ids and intents; the world lineage rule; EncounterDef.ruleset through the world bridge, host new {ruleset, rules}, aegis battle and sim ruleset=/rules= (a world encounter defaults to its world's), the Arena select and the Dojo toggle; classic stances inert under round_intent); parity strips for the two world fields and the kind (HISTORY row); docs (29d9772)
- Planner: DIR4-M done; WORLDW1 (world manifest writer) before SUNKWARD1; RULESET1-R picks (flag keys, Q1-Q3) in OPEN_QUESTIONS section 4 (21cc0d4)
- DIR4-M: four-way facing on the region map (the leader turns toward each step with Facing::toward, side rows mirrored only facing left, up/down rows drawn when a body has them; people in a conversation face the party four ways; agent link fields facing and view); GAME_BRIEF (f429fe2)
- Planner: DERIVE1-R, DERIVE1-D and KIT1-G done in TASKS (c22df47)
- KIT1-G: the layer check in the gate (tools/fc.sh layers and check_project.ps1 step 3/3: engine and kit crates name no game, read from the folders under games/, and have no path dependency into games/; kit READMEs in the docs lint); docs (809fc5a)
- DERIVE1-R: derived-stat rules (rpgkit_rules::derived: value, breakdown, resist chance and the seeded battle draw, the {resist, potency} validator; passive stat percent on derived ids only); docs (d444fa7)
- DERIVE1-D: the Composure derived-stat row (timeline only; KNOW and CONS based, 0-100, first-guess constants for the balance lane), written with aegis content (seed derive1_d.json) (8354295)
- Planner: PLAYUI1-R done; DIR4-M in progress; DERIVE1-D, DERIVE1-R, KIT1-G in review; RULESET1-P adds gear_mitigation and the Ruleset wrapper (a2d29fc)
- PLAYUI1-R: region map sprites (the leader and every person as body sprites, walk and idle, side facing by the last step, talk facing, per-person idle offsets, foot shadows, names above heads, silhouette behind props, token fallback; agent link state.sprites) (204e1ce)
- Planner: KIT1-SIM, DERIVE1-S, DIR4-G done; DERIVE1-R, KIT1-G, RULESET1-R in progress (91a7aca)
- DIR4-G: battle facing in four directions (up or down when the target or step is more vertical, keeping left/right; tokens and outlines through Art::played_facing with the side-row fallback, so bodies without views draw as FACE1) (6c3f7f6)
- DERIVE1-S: derived-stat rows (content.derived_stats in kits/rpg/schema.json; DerivedStatDef and Catalog.derived_stats; loader, checks with cycle detection and topological load order, writer; derived ids accepted in gear stat bonuses and passive stat effects with optional percent); the Godot schema parity strips the new kind (HISTORY row) (7b10c39)
- KIT1-SIM: the kit sim names no world (SimGame { default_world, story: `Option<StoryRun>` }; run(d, argv, &SimGame), SimArgs::parse_for); the game's emberwake_sim passes the Ashlands story run (game(), run(d, argv); emberwake_world becomes a normal dependency); kit test story_run.rs; the art.rs test literal skarn_ironjaw -> elite_brawler (f889dd9)
- Planner: DIR4-HB (view-aware hit rect, after DIR4-R1) (15025ba)
- Planner: KIT1-DEPS also adds the façade sim parse with the game defaults (f13b06f)
- Planner: STAGE1-L done; DIR4-G and PLAYUI1-R in progress (a425e1e)
- STAGE1-L: the battle stage, option B (stage.rs: 32x24 cells, the terrain stage table, ground under props, lips, blocks, standing props, backdrop and front face, draw passes, ground arrows, feet order and outlines through, unit hit rects; battle.rs draws Stage or Flat by the battle_view setting, default Stage); tuning keys; kit art lookups for stage tiles, props and backdrops (e8d4e96)
- KIT1-P: in-flight specs on the kit paths (16 files), ROSTER1's ENCOUNTER2-W changes applied, the Crucible row enc_evt_06 (C16), HOOKS1 8.4 redirect order (C14), the execute rider HOOKS1 8.6 for G7; CAST1 Throat-Carver and Reflex rows; task rows HOOKS1-X, layers fixed; HQ15-HQ16 planner picks (9d8f334)
- Planner: FX1-E done; STAGE1-L in review (1f6e6d9)
- FX1-E: engine effects kit (aegis_ui::fx: clock with hit-stop, six code-drawn primitives, live list, camera shake with presets, limits and the engine ceiling), Painter::shift, the silhouette hit flash, Settings::reduce_motion; the UI Gallery's Effects page (ticked from gallery.rs) (2b939fc)
- Planner: KIT1-N done; DERIVE1-S and KIT1-SIM in progress (b6f118d)
- KIT1-N: the kit names no game (real rpgkit_* crate names; GAME_ID and find_data_dir in the game façade, the kit's Data::find_data_dir / find_game_data_dir; store hooks RpgContent; the host battle command set moved to rpgkit_host with an emberwake_host façade; neutral comments and test ids; test-only deps to dev-dependencies; kit test data_dir.rs); docs (3e12a9a)
- Planner: KIT1-SIM also renames the skarn_ironjaw test literal; KIT1-DEPS (prune unused façade dependencies) (fa98879)
- Planner: CAST1-R done; CAST1 task dependencies and paths; CAST1-A6b (M2b encounters); KIT1-P also fixes C14, C16 and the G7 execute rider; CQ17-CQ18 planner picks (9945b68)
- CAST1-R: the CAST1 parts use the HOOKS1 hooks, ROSTER1's keepsakes and roster keys, CAMPAIGN1's tags instead of stand-ins; gaps resolved with pointers; CQ1-CQ16 planner picks; new CQ17-CQ18, C14-C16 (2ddce66)
- Planner: STAGE1-T done; FX1-E and STAGE1-L in progress (d595980)
- STAGE1-T: battle stage art by code (tools/gen_art.py stage_tiles, battle_props, battle_backdrops; 64 tiles at 4:3 and 4 wall faces in engine common_stage, kit tree and stone props, 8 zone backdrops; at most 16 colours each); the asset layout test allows backgrounds/battle; READMEs (fe5068e)
- Planner: STAGE1-E done (642ee49)
- STAGE1-E: engine stage pieces (Painter::ellipse_ring, Image::opaque_bounds, Art::body_bounds with a cache) (72e8f6e)
- Planner: STORYUI1-E and KIT1-M done; KIT1-N and CAST1-R in progress (bb80e48)
- KIT1-M: the RPG kit crates by a byte-identical move (kits/rpg/crates/rpgkit_{data,rules,battle,content,world,sim}, 73 files, crate aliases; the emberwake_* crates become façades; schema.json to kits/rpg/); docs for the new layout (CLAUDE.md, README, STRUCTURE, engine README, PORT_NOTES, GAME_BRIEF, kit and asset READMEs, skills) (c1cee15)
- STORYUI1-E: engine vignette player (aegis_ui::vignette: cards in four styles, typewriter at text_speed, fades, skip with ask-first, cues, music wish, code-drawn backdrops, reduce_motion; VignetteScreen; the UI Gallery entry) (7bbbc2e)
- Planner: STAGE1-E and STAGE1-T in progress; FX1 spec being written (64f4419)
- Planner: CAMPAIGN1-E done (d5e76eb)
- CAMPAIGN1-E: keyed saves in aegis_save (write/read/exists/peek by key, the same envelope and safe write as a slot, outside the slot range; for the act-start save); README, PORT_NOTES, authoring note (ec736eb)
- Planner: CAST1-A7 done; ASSETCHK1 (parse every palette file in the asset check) (163014c)
- CAST1-A7: placeholder assets for the Act 1 cast: 22 palette files on existing rig bodies (art-bible hues, computed steps, Cinder Ash skin for the calcified), 59 copied item, status and skill icons; no new art (537b47f)
- Planner: STORYUI1-E, CAMPAIGN1-E, CAST1-A7 in progress; AGENT_MAP (98f83a7)
- Planner: RULESET1-E done, KIT1-M in progress (e0639a8)
- RULESET1-E: timeline widget segment parts (windup, impact, recovery), whole hatching, links and marks; the UI Gallery's Timeline page; plain timelines draw and click as before (61156da)
- DERIVE1-E: engine formula evaluator (aegis_core::formula): parse once with column errors and size limits, + - * / unary minus, min/max/clamp/floor/ceil/round, named inputs, never NaN or infinity; README, STRUCTURE, PORT_NOTES (de770d4)
- Known issues: every trace diverges in a fresh worktree when parity/out was copied from a stale recording (8c9651b)
- Planner: DERIVE1-E and RULESET1-E in progress (487b59d)
- Planner: AGENT_MAP path fix (fabd0f6)
- Planner: dispatch mode standard (user 2026-10-25, building the slice); AGENT_MAP active section (65f5864)
- Planner: the vertical slice approved for building (DERIVE1, RULESET1, ENCOUNTER2 slice tasks); spec rows KIT1, CAMPAIGN1, ROSTER1, STORYUI1 and the SUNKWARD1 data row (4cd0da3)
- Emberwake lore: core story track decided (Q-02, Q-03, Q-06, Q-09, Q-28, Q-35, Q-87, Q-94; story gaps G-08 to G-17, G-26, G-31 to G-33) in CANON 3.3 and 3.6, central_truth and the story guide; RULESET1 TQ22-TQ23 decided (eb8f5c8)
- RULESET1: round stance renamed round intent (RULESET1_INTENT.md; round_intent, intent_bound, intent_buff, RULESET1-IN; the intent tells gain the round intent), intent-bound vs round-bound buffs, carried maneuver moves; TQ22-TQ23 open; ENCOUNTER2 references; task rows CAST1 (approved) and BIGUNIT1 (idea), FX1 adds the camera shake; HISTORY rows (812c94c)
- Emberwake lore: content group 1.7 and the art questions decided (Q-71, Q-74, Q-76, Q-77-Q-82) in CANON and decisions/lore.md (a388a72)
- Emberwake lore: corruption group 1.6 decided (Q-10, Q-62-Q-68, Q-92) in CANON 2.2 and decisions/lore.md; DERIVE1 DQ1-DQ5 decided (caaaed2)
- Emberwake lore: slice group 1.5 decided (Q-86 M2b, Q-88 prototype order and vignettes, Q-89 barks later, Q-90, Q-96 reuse roster and shop) (caea9e1)
- Emberwake lore: hub group 1.4 decided (Q-18, Q-23, Q-42, Q-43, Q-45, Q-46, Q-69) in CANON and decisions/lore.md (f5eaa1f)
- Emberwake lore: round stance renamed to round intent (feeds the intent tells), intent-bound vs round-bound buffs, maneuver skills carry their movement; TQ12-TQ21, Q-37a, Q-40a decided (7390cf9)
- ENCOUNTER2: from Act 2 a downed unit left behind on an escape is captured (Q-32) (5fa87a1)
- Emberwake lore: roster group 1.3 decided (Q-29, Q-32 bleed-out and leaving a unit behind, Q-34 keepsakes, Q-36, Q-37 Martyr deployment counter, Q-40, Q-93 recruits after the Crucible); new Q-37a, Q-40a (fbac138)
- Emberwake lore: cover, stealth and stance persistence; CANON Cut row (facing and stealth return without cones or rolls) (86e45d4)
- RULESET1 revised to the round-stance model (RULESET1_STANCE.md: stances, one target, same-stance follow-ups, auto planner and manual mode, flanks, cover and stealth, Rest, enemy recovery); task rows ST, PL, AI added; ENCOUNTER2 references updated; HISTORY rows for the first draft's chains, base slots, player recovery and Rest AP (5614e23)
- Emberwake lore: round stance details (no recovery for player units, neutral Rest, kiting, Guard cover and stealth, flanking and the Maneuver kinds, Support pouch use, per-stance chain order) in CANON 2.1 and decisions/lore.md (f0737f3)
- Emberwake lore: Q-19a, Q-20a, Q-21a decided (character lean and endings, checkpoint bad ends, dream-echo retries); the round stance (intent) model and the deck without base-move slots in CANON 2.1 (2f97f1c)
- RULESET1 and ENCOUNTER2: the user's answers to TQ1-TQ11 and EQ1-EQ9 (as recommended); RULESET1 4.3 chains marked for revision into the round-stance model (363c58c)
- Emberwake lore: campaign group 1.2 decided (Q-14 Alert floor, Q-17 clock correction and side quests locked at 2.4, Q-19 doctrine by play and story with character alignment, Q-20 checkpoints with day deadlines, Q-21 final-battle retries, Q-22 hub siege as rooms, Q-30 board jobs, forks converge); new Q-19a, Q-20a, Q-21a (19fc838)
- ENCOUNTER2: the room uses the user named (hub siege, floors to climb, waves, rooms to clear) (514fbab)
- Emberwake lore: combat group 1.1 decided (Q-49 test armour 25-50, Q-51, Q-57-Q-61, Q-60 fixed arena plus linked rooms, Q-70 reach 2, Q-72, Q-75 stay-behind objectives, Q-75a the Crucible holds by choice, later losses by fate, Q-91); CANON 2.1 rows, the Cassock as the node 20 reward; ROADMAP rows ENCOUNTER2 and RULESET1 (d76448a)
- tools/lorebook.py: renders a game's lore library (story guide, who's who, registry, canon glossary) as one Worldbook page with a note box per entry (db collection notes) (c1d8411)
- Emberwake lore: the user's answers of 2026-10-25 (Great Tithing B tied to the Clerisy's founding, the Tithing harvest and Lullaby Houses, Nemeth's arc and node 20 fight, Composure as a derived stat, Q-15/38/41/44/50/52-56) in CANON 3.6, decisions/lore.md and every domain doc; questions tagged by focus with systems split into seven groups, Q-92..Q-97; lore modules index LM-01..LM-17; ROADMAP row DERIVE1 (399a74b)
- Emberwake lore: the user's first answers (Q-01 truth A, Q-13, Q-25, Q-31, Q-47, Q-48, Q-49 intent, Q-83..85) in CANON, the question files and decisions/lore.md; STORY_GUIDE.md and who_is_who.md (story, arcs per act, gaps G-01..G-30); tithing_options.md (Q-04 / Q-16 options); HISTORY row for truth B and Composure (153ec78)
- Planner: the Low Sleep decisions (games/emberwake/docs/decisions/lore.md, index row), HISTORY rows for the lore ideas not adopted, LORE1 archived (65c2f3f)
- Engine authoring guide (top-down game design on Aegis, game-agnostic): the layer rule, a game's design library, the consistency pass, naming and asset briefs, the art bible and milestone templates; skill game-design-library; the docs check covers the lore folders and engine/docs subfolders (6a6b160)

Under the hood:

- FX1 spec (engine effects kit and camera shake, impact sync, the game's effect table, story shakes); task rows; FQ1-FQ8 planner picks; HISTORY rows (053d23c)
- HOOKS1 spec (combat hooks for the slice's cast instead of stand-ins); task rows, CAST1 dependencies, CAST1-R; HQ1-HQ14 planner picks; HISTORY and ROADMAP rows (aeb9cbd)
- ROSTER1 spec (unit lifecycle, bunks, recruits behind a flag, keepsakes, temperament; bleed-out and Martyr hooks); task rows; ENCOUNTER2-W depends on ROSTER1-L; RQ1-RQ11 planner picks; HISTORY and ROADMAP rows (43126b6)
- Specs STORYUI1 (vignette player, story sequences, mission board) and CAST1 (the Act 1 combat cast as data, 8 parts); task rows; HOOKS1 spec row (no stand-ins in the slice); SQ and CQ planner picks while the user is away (06d4227)
- KIT1 spec (kit extraction: rpgkit_* crates by a byte-identical move, façades, schema to the kit, layer check); task rows, KIT1-N added to the kit-layer slice tasks' dependencies; KQ1-KQ8 planner picks; DERIVE1-E done; HISTORY and ROADMAP rows (3c4717d)
- CAMPAIGN1 spec (campaign meters, Doom Clock, checkpoints, run over, purse, rests, defeat retreat, triggers); task rows; CMQ1-CMQ9 planner picks while the user is away (OPEN_QUESTIONS section 4) (cce1c7f)
- Engine docs: campaign structure (checkpoints with day deadlines, forks converge, meters across act lines, losses and retries) from the user's 2026-10-25 answers (db2906d)
- Specs RULESET1 (the timeline combat ruleset, M1 feel prototype; renamed from TIMELINE1, which names the done widget task) and ENCOUNTER2 (authored terrain, hold-a-tile escape, impossible fight, boss phase kill/swap/rogue, linked rooms); task rows RULESET1-* and ENCOUNTER2-* (spec, not dispatched) (f0c7676)
- DERIVE1 spec: data-defined derived stats (engine formula evaluator, kit derived-stat rows, resist checks; Composure the first row); task rows DERIVE1-E/S/R/U/D (spec, not dispatched) (f532298)
- Engine docs: authoring rules from the user's 2026-10-25 answers: the slice is data only and starts at the onboarding, options vs extensions (milestones.md); question focus order with systems split per system (consistency_pass.md); lore modules (lore_modules.md) (b9484c3)
- Engine docs: story guide template for a game's design library (engine/docs/authoring/story_guide.md), linked from the README and design_library (8fc2ed9)
- LORE1 (game docs): the lore library moves into games/emberwake/docs/lore/ and becomes consistent: CANON.md, the cast registry, the central truth (proposal), renames, one set of numbers, layer tags, status lines, repo-relative links; the raw export archived; 91 open questions and 29 planner picks in OPEN_QUESTIONS.md; title Emberwake: A Scarlet Lullaby (9452138)
- docs(emberwake): expand campaign architecture and geography for Act 2 and Act 3 (42d9dc2)
- docs(emberwake): flesh out non-hub factional, neutral, and hostile character cast (970f7e7)
- docs(emberwake): add comprehensive items, gear, corruption consumables, monster parts, and named relics specification (75aade3)

## 2026-10-24

Under the hood:

- docs(emberwake): complete Step 5 consistency pass, expand art bible, and author full game systems roadmap (7412844)
- docs(emberwake): add Sunk Ward campaign graph and Phase 1 encounter staging specification (6b4298a)
- docs(emberwake): add safehouse economy, triage systems, and UI architecture specification (f866128)
- docs(emberwake): expand combat and timeline spec with state machine, tie-breakers, and engine mappings (83a1113)
- docs(emberwake): add base/elite enemy bestiary, named bosses, and modular placeholder encounters (b64bad8)
- docs(emberwake): flesh out all base class trees and author founder specializations (DEC-LORE-14) (9dcddf5)
- docs(emberwake): formalize official title as Emberwake: The Low Sleep (DEC-LORE-13) (4b4617c)
- docs(emberwake): organize lore into background, narrative, specs, and systems with AI index (db7d5ee)

## 2026-10-23

In the game:

- **Enemy sprites**: goblins (brawler, berserker, skirmisher, scout, archer, shaman), Grakka Ashmaw, the Horde
  Warchief and the training dummy now animate in battle like the party: idle, walk, attacks, hurt, death and guard.
  Ash goblins and the named goblins (Skarn, Vex, Nok, Bolg, Gruk) wear their own colours. Enemies now face the party.
- **NPC sprites**: Brask Toller, Hest the Quartermaster, Wenn and Orsa Tallow have their own bodies.
- **Save slots**: click anywhere on a slot to save or load. Lock and delete icons sit in the slot's top right; a locked
  save can't be overwritten or deleted (the autosave can't be locked).
- **Map top bar**: the region and world maps show the place name top left and gold, supplies and icon buttons top
  right: Party (P), Journal (J), Stash (I), Help (?) and Menu.
- **Region map**: the map now fills the screen and the camera stays centred on the party (walls continue past the edge).
  A foldable "Here" list floats top right with icons for each marker and what an NPC offers (shop, recruits, quests).
  The latest event shows bottom left; L opens the last 12. Names show when you point at someone (Settings, Gameplay:
  Map names On hover or Always).
- **Icons**: new icons for the save slots, the top bar and the map.
- **For testers**: the F8 note box is taller and edits like a text editor (click to place the caret, select, copy and
  paste, undo, word jumps). In Playtests, click a note to read it whole and copy it.
- **Warrior looks**: a warrior's sprite shows the gear it carries: one-handed with a kite shield or a buckler,
  or a two-handed greatsword, battle axe or warhammer. Hest, Brask and Orsa carry a hatchet and buckler; Grakka a
  two-handed double axe.
- **Shop**: buying and selling is one three-column trade (shop, trade, stash): click to add, +5 / +10 for more,
  see the net gold, then Confirm once.
- **Dialogue**: the talk box sits centred at the bottom with the speaker's portrait, name and faction; the camera
  frames the party and the speaker; the shop opens centred over the dimmed map.

Every change that touched the game:

- Planner: known issue: gate a push on the gate's exit code (its output does not reach the pipeline) (9f01456)
- WAR2H1 (game code): the warrior look follows the equipped gear (Grip, gear_body_chain; a shield of burden 0 or less is a buckler); 7 tests; docs (SPRITE1 status, WAR2H1 row) (837a835)
- WAR2H1 (game art): Hest, Brask and Orsa with a hatchet and buckler; Grakka with a two-handed double-bladed axe; no kite shields (e682d40)
- WAR2H1 (kit art): warrior looks 1h_buckler, 2h_sword, 2h_axe, 2h_hammer with every clip (84820e5)
- WAR2H1 (tools): the rig draws the warrior's looks: weapon drawers (greatsword, battle axe, warhammer, double axe, hatchet), the buckler, the two-hand and buckler grips; variants.py look matrix; export_units bakes looks sharing the base's palette (27a3a33)
- Planner: WAR2H1 row (user 2026-10-23: warrior 2H and 1H + shield looks by equipped gear; Grakka, Hest, Brask, Orsa without the shield) (dacb704)
- PLAYUI1-P (game): the shop is a three-column trade (shop, trade, stash): clicks add 1, +5 / +10 more, a trade line goes back whole, net gold, Confirm settles it all at once through the new all-or-nothing shop::trade; the shop moves to shop.rs (8b25d42)
- Planner: known issue: stale test binaries from a scratch copy in a worktree's gate target folder (187fc12)
- Planner: known issue: patch_notes.py misses commits merged from task branches (f2e05ea)
- PLAYUI1-D (game): the dialogue box: 400 px centred at the bottom, portrait with name and faction under it, the NPC's line above the replies; the camera on the party and the speaker; the shop centred over a dimmed map with the box hidden (c40482b)
- SPRITE2 (game art + data): Grakka, the Horde Warchief, Brask, Hest, Orsa and Wenn bodies; ash and named-goblin palettes; seeds/sprite2.json points the named goblins at their goblin body with a palette and Orsa at her body; the four old named goblin bodies removed; the worldbook regenerated (57c560d)
- SPRITE2 (kit art): six goblin bodies (brawler, berserker, skirmisher, scout, archer, shaman) and the training dummy with every clip (96221e4)
- SPRITE2 (tools): the rig draws goblin variants, the dummy and named bodies (rig scale and girth, goblin and named heads; variants.py; export_units writes kit and game bodies, palettes and review sheets); gen_art writes no bodies (4f5cbd5)
- SPRITE2 (engine + game): foes face the party: Art::played and body_outline take flip (mirrored frames cached apart, body_clip_flipped); the battle draws a foe's token and its outline mirrored (d5a333d)
- PLAYUI1-M (game): the region map full width under the HUD bar, a camera centred on the party with walls drawn past the edge, a floating foldable legend with marker and NPC service icons (explore::npc_services), the event log ticker (L), the map_names setting (On hover / Always); the HUD bar moves to hud.rs (9b16e65)
- PROPS1-A1 (game data): 63 Ashlands props on the four map templates through seeds/props1_ashlands.json; the other templates gain props [] (ecc9d64)
- PROPS1-T (tools): prop sheet prompts (PROPS1_PROMPTS.md: props_kit, props_buildings, props_ashlands; each sheet needs the user's yes) and the `sheets.py props` cut to the layer paths; no API call (b40590d)
- PLAYUI1-H (game): one HUD bar on the region and world maps: the place name top left; gold, supplies and icon buttons Party (P), Journal (J), Stash (I), Help (?) and Menu top right, off during a conversation, shop, turn-in or lore page; the Controls tab lists P, J, I, ? (720a1cb)
- PROPS1-L1 (game): a props list on map templates (stored after paths, default []), checks P1-P12 refuse a bad upsert, prop art folders along the lineage, asset warnings A7-A10 (ee5cb4c)
- PROPS1-E (engine): DepthList (feet-sorted draw list), Image::mirrored and Art::image_mirrored, Art::size_of, Art::varied and Art::pick (tile_in wraps varied) (aab3a09)
- PLAYUI1-S (engine + game): save slots: a row click saves or loads; lock and delete icons top right; engine slot locks in locks.json (write and delete refuse a locked slot, the autosave never locks); a disabled kit icon button swallows its click (6cfeb72)
- PLAYUI1-I (engine + kit art, tools): the 22 UI icons from one image-API sheet (user 2026-10-22: one image), cut by sheets.py ui (5x5, magenta key, 32x32, 16 colours, outlined); toolkit glyphs in engine/assets/ui, HUD, marker and service icons in kits/rpg/assets/ui; the layout test knows the new paths (f83e656)

Under the hood:

- Planner: SPRITE2 done (spec status, STRUCTURE tools row, handover note, task archive) (d74f5a3)
- Planner: PLAYUI1-I in the task archive (238cec5)
- Planner: PLAYUI1-I done; MARKERS1 idea row (two marker icon sets); known issue: a piped lint check hides its failure (f5bcfef)
- Planner: PROPS1 status lint fix (a blank line before Goal) (a47519c)
- Planner: PROPS1 approved (user 2026-10-22: walls only, no prop interaction, three API sheets asked each, the party outline behind roofs); rows PROPS1-E, L1, L2, T, A1, A2 (5b851d7)

## 2026-10-22

In the game:

- **Battle sprites**: the six party classes animate in battle: idle, walk, their attacks (by skill and weapon), hurt,
  death and guard. Each unit stands on a coloured ellipse (blue party, red enemies); the selected hero has a gold
  outline and a hovered unit a faint white one. The animation speed and skip settings apply.
- **Settings**: the music no longer plays garbled while a volume slider is at zero.
- **Art and sound files**: everything moved into three layers: the game's own files, a shared RPG kit (class bodies,
  common portraits, item and status icons, map markers, combat sounds, mood music) and the engine's baseline (the
  common tile set, interface sounds). Nothing should look or sound different; report any missing or changed art.

Every change that touched the game:

- PHASEFOLD-A (game data): the shipped reinforcement waves as arrival phases through seeds/phasefold.json; no wave_* key left in shipped data (691f2ee)
- PROPS1-S (design): props layer spec (proposed; the user answers section 10); HISTORY row for the options not adopted (cbb8231)
- PLAYUI1-T (engine): the kit text editor (aegis_ui::edit: visual-line moves, click caret, selection, word moves, undo, clipboard over arboard with an in-app fallback), a taller F8 note box, the Playtests note view with Copy (cd9d07f)
- PLAYUI1-K (engine): kit icon button (image or drawn glyph, hover card, focus, toggled, disabled) and collapsible section header, on the Kit widgets page (63695f1)
- PHASEFOLD (game tests): the golden of 72 fights and the fold, check, writer, win-rule and morale tests; rewards.rs authors an arrival phase (b2b1806)
- PHASEFOLD (game): the single reinforcement wave becomes an arrival phase (trigger {kind: arrival, round, block}, first in phases, never reserved); legacy wave_* rows fold on load, write and check; battles unchanged (d8352b7)
- SPRITE1-U (game): battle units play their clips; a foot ellipse in the side colour, a gold outline on the selected sprite, a white one on hover (user 2026-10-22) (83545f0)
- SPRITE1-L (engine): clip playback from body sheets (Clip::frame_at, Clips::resolve, ClipPlayer, Art::played), cached silhouette outlines, body checks, Painter::ellipse (e386c1a)
- SPRITE1-A2 (tools): export_units writes every clip by default; READMEs (7682a09)
- SPRITE1-A2 (kit art): the six class bodies carry every clip (48 px frames, 8-11 clips each, per-frame ms, loops); the priest shows the B3 mace (e889140)
- REWARD1-B (balance): rewards report on main 18f3893; seven design issues flagged (23e6575)
- ASSETORG2-T (tools): gen_art, export_units and gen_music write the layered tree (--layer, --root, --overwrite, --compare); the assets READMEs (b41954f)
- Music volume: the settings popup no longer flips the music between the saved and draft levels each frame (engine) (e695e37)
- Music volume: the settings popup no longer flips the music between the saved and draft levels each frame (engine) (4e5c8a8)
- ASSETORG2-K/B (game): the layout and music tests walk the game, kit and engine layers; asset READMEs (2eeaafa)
- ASSETORG2-K (kit): kits/rpg with kit.json; the shared bodies, portraits, icons, markers, chips, combat sounds and mood music move to the kit; game.json names it (a6ae779)
- ASSETORG2-B (engine): engine/assets holds the common tile set and the interface sounds; Audio::offline_layered (9061075)
- ASSETORG2-G (game): Cargo.lock for emberwake_gui's dependency on emberwake_content (e25413e)
- ASSETORG2-D (game data): body keys, portrait tiers and item icons; the Godot asset manifest retired (297e11a)
- ASSETORG2-G (game): every asset on the layered tree; lookup chains, schema and the asset checks (842d786)
- ASSETORG2-G (engine): art by chains only, existence checks cached (17cc42c)
- ASSETORG2-E (engine): asset layers, body sheets and palettes, art and audio over the layers (c92d9e4)

Under the hood:

- Planner: PHASEFOLD-A done (task archive) (4a21258)
- Planner: PLAYUI1-K and -T done (engine README kit, editor and agent link; STRUCTURE edit.rs; PORT_NOTES; task archive) (df3dc3f)
- Planner: PHASEFOLD done (HISTORY row naming the two partial retirements, GAME_BRIEF reinforcements, PORT_NOTES, BOSS1 pointer, STRUCTURE, task archive) (a7249cf)
- Planner: SPRITE1-L and -U done (spec status, roadmap, task archive) (5982e55)
- Planner: SPRITE1-A2 done (task archive) (b728ab3)
- Planner: PLAYUI1 and PHASEFOLD specs approved (user 2026-10-22, all answers); PLAYUI1-K/T/S/H/M/D/P/I/R, PROPS1-S, PHASEFOLD-A rows; HISTORY rows for the replaced UI choices; REWARD1-B report separate (e05cd3b)
- Planner: ASSETORG2-T done (spec 8b, STRUCTURE, handover note, SPRITE1 A2 pointer, task archive) (d92385f)
- Planner: SPRITE1-U highlight (user 2026-10-22: foot ellipse in the side colour, gold outline on the selected sprite; battle first) (8dd7d8c)
- PATCH-NOTES: REWARD1-L, REWARD1-A, REWARD1-S, ASSETORG2-E/G/D/K/B (2026-10-21 and 2026-10-22) (2416656)
- Planner: user decisions 2026-10-22 (ASSETORG2-T acceptance by paths and sizes, PHASEFOLD-S spec first, REWARDXW1 stays an idea); playtest 20261021-193519 to the inbox (5 UI notes) (0353423)
- Planner: SPRITE1-S2 done (task archive) (52f01bb)
- Planner: ASSETORG2-K and -B done (STRUCTURE, README, task archive, agent map); SPRITE1 S2 sheet layout; known issue: staged renames ride along (86e96fe)
- Planner: ASSETORG2-G and -D done; spec 9b (the move as built), HISTORY rows, docs on the layered tree (7da4590)
- Planner: ASSETORG2-E done (README, STRUCTURE, task archive) (a3c47d6)

## 2026-10-21

In the game:

- **Battle rewards**: every enemy you defeat (slain, surrendered or fled) pays its share, whatever the fight's outcome:
  XP by its level and tier, its part of the encounter's gold and a roll on its loot table. Enemies still standing pay
  nothing, and a beaten or fleeing party keeps what it earned. The results screen shows the count ("Foes defeated: 2
  of 3"), the gold and the drops. The Ashlands encounters have loot tables and gold; the Warchief's Axe drops only when
  you kill its bearer and win, and the courtyard objective pays 10 gold once.
- **Skills tab**: the loadout is fixed slots (5 class, 3 extra) beside your skill list. Drag a skill onto a slot, between slots or back to the list (clicks still work), right click a slot to empty it, or Clear all. Each filled slot has < > arrows over the ranks the hero has unlocked; the rank you pick is saved and is the only rank of that skill the hero uses in battle.
- **Text size**: every screen now fits at Small, Normal, Large and Extra large. The battle's top bar sizes its buttons to their text, enemy cards wrap their "Next:" line, the world map's labels and Back button fit, and the Character Stats passives row no longer spills.
- **Portraits and icons**: the first-pass art is in: 36 portraits (the named cast and shared ones for common enemies), 57 skill icons and 16 item icons.
- **For testers (Scene Runner)**: one **Exploration** entry replaces the two debug maps: pick a world and a start (any location, or the world map). In a fight, **Skip** settles it as a win with every enemy's XP, the gold and quest progress, at no cost; Fled, Lost and Fight it are still there. The **UI Gallery** now shows the kit's widgets (try them), every screen on a sample game (Open to try one), and each battle panel on its own.

Every change that touched the game:

- REWARD1-S (game): the sim prints what the defeated enemies pay (rewards=on) (8956341)
- REWARD1-A (game data): loot tables, Ashlands purses, key loot and the courtyard objective reward (6cca4c7)
- REWARD1-L commit 2 (game): rewards per defeated enemy (16da125)
- REWARD1-L commit 1 (game): schema slice for rewards per defeated enemy (6db403e)
- UIGALLERY1 (game): the UI Gallery's kit, screens and battle panels (8248ede)
- UIGALLERY1 (engine): screen and region previews, the kit specimens page (051e5fa)
- RUNNER2 (game): one Exploration scene; a skipped fight settles as a win at no cost (a244790)
- TEXTFIT1 (game): every screen fits at the four text sizes (a873f67)
- TEXTFIT1 (engine): the text_audit agent command, long-word wrap, stroke window buttons (5d2d4c2)
- SKILLUI1-U (game): the Skills tab on fixed slots, drag and drop, rank arrows, right click to take out (3a1ea34)
- SKILLUI1-L (game): the skill rank chosen per member, the only rank a party hero uses in battle (b21a2dc)
- ART2 (game): the baseline art in the game: 36 portraits (18 named, 18 generic shared by mob units), 57 skill icons, 16 item icons (8 new family folders) (86b38cf)

Under the hood:

- Planner: REWARD1-S done; the sim key in WORKFLOW; REWARD1-B notes the sim's expected values (994ee3f)
- Planner: REWARD1-A done; seeds README lists the engine-era seed (d8eb52f)
- Planner: REWARD1-L done; HISTORY rows (win-only rewards, no kill loot, purse loot); REWARDXW1 idea row (fecf455)
- Planner: UIGALLERY1 done (SCREENS, STRUCTURE, README, task archive); known issue: leave() on a Reset (fa131b1)
- Planner: RUNNER2 done (README, SCREENS, HISTORY row, task archive) (2b49d74)
- Planner: TEXTFIT1 done; DRAG1 candidate list; SKILLTEXT2 idea row (a127895)
- Planner: SPRITE2, sprites for the non-party characters (goblins, Grakka, the NPCs, the dummy) (user 2026-10-21) (4602ce0)
- ART2: closed at the baseline; the remaining art waits for alpha feedback (user 2026-10-21) (eea1df6)
- WORLDBUILDING.md and RPG_KIT_CATALOG.md, written from the game data by tools/worldbook.py (review copies for the user) (9a906f2)
- ART2: the 80 icons kept as the first-pass baseline; generated art is an alpha baseline, feedback after the full game runs (user 2026-10-21) (c29a106)

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

- ART2: icon sheets (4 skill sheets, 1 item sheet: 80 icons) and sheets.py (sheet prompts, slicer); generate.py reads the icon prompts (7b04f11)
- Planner: REWARD1 approved (all eight recommendations), build rows approved, PURSE1-A/B superseded; ART2 picks recorded (ec000f5)
- Preferences: one-off images can go the Gemini app route (prompt from the agent, image from the user) (48ef735)
- Preferences: the image API is agreed per run, suggested where it beats code; small assets as sheets; portrait baseline stands (f819e0c)
- ART2: recolor_families (whole shade families, for mob variants: goblin tribes) (19a812a)
- ART2: 12 generic base portraits (six human class defaults, five Ashen Horde goblins, the dummy) and their prompts (97f28a5)
- ART2: the portrait cast of 18 accepted (user 2026-10-20); generic portraits from generated bases plus recolours (3df6a92)
- ART2: portrait sources for 15 more characters (Gemini 3 Pro Image, Mara Voss as the style reference) (0b7a6ba)
- ART2: art source images in Git LFS (art_src/: Mara Voss and her variant sheet, Bryn Hale) (67bd74f)
- ART2: pixelize reduces by the most common colour per cell, so pixel-art sources keep clean clusters (8a308b3)
- ART2: generate.py, portrait source art from the Gemini API into art_src/ (key from GEMINI_API_KEY, never printed) (41451e2)
- Tidy: Python caches out of git (__pycache__ ignored) (59ddcc6)
- ART2: portrait tools (portrait rig, pixelize, recolor), priest mace pick B3, portrait prompts and preferences (f60623c)
- ART2: the art review of 2026-10-20 as a worklist (the user's marks and notes); task row (fd08a4f)
- REWARD1: spec, the user's answers (divisor, weight, key loot, drop rates) and its build rows (75cb1cf)
- Planner: rewards per defeated enemy (user 2026-10-20); REWARD1 spec row (17fb51a)
- Planner: RUNNER2 and UIGALLERY1 answers (user 2026-10-20); LAYOUT1 idea row (5887c54)
- Planner: known issue, Python writes CRLF on Windows (44ecfce)
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
