# LOOK1: visual pass (tile sets, menu screens, layout and a game theme)

Status: proposed (options spec 2026-11-02, from the user's request of 2026-11-02: "lets start adding some better tile
sets, menu screens, layout and styling for the game assets etc"); nothing here is decided until the user answers
section 9
Depends on: UIKIT1, UIKIT2 (the widget kit), STAGE1 (the tilted battle board), PROPS1-L2 (map props), PLAYUI1 (play
screens), the art routes (decisions "Gemini web app as the primary image route", "Expression sheets: scale only and
aligned cells", "Art goes straight to develop from the planner worktree" in `docs/decisions/workflow.md`); the region
tile art also waits on EXPLOREVIEW1 (Q4)
Lane: design (this spec), then logic (E1, G1, ME, M, E2, K, L, T), art (A1-A7, planner worktree), asset (R)
Touches: by slice, section 8
Paths: `aegis_*` crates are under `engine/crates/`, `rpgkit_*` under `kits/rpg/crates/`, `emberwake_*` under
`games/emberwake/crates/`.

History: "Buildings as more tile kinds" and "painted location backgrounds as the region map image" (2026-10-22, not
adopted) hold: buildings stay PROPS1 props and no painting replaces the tile map; location pictures appear here only
as save-slot thumbnails. STAGE1 options A and C (not adopted) hold: the battle board stays option B. Q9 (b) touches
the "more tile kinds" row; see there.

## 1. Goal

Give the game one look that matches its Gemini pixel-art portraits and props: a theme the game sets on an engine
theme (palette, panel frames, fonts, spacing), a title screen and restyled menus built from reusable engine panes, and
tile sets for the slice world (region maps and the battle board) with wall faces and edge transitions. Presentation
only: no rule, save, sim, trace or journey reads any of it.

## 2. Survey: where the game looks weakest (develop at 6d4eb9c5)

Screens judged from `games/emberwake/docs/SCREENS.md`, the engine shots in `games/emberwake/docs/screens/engine/`
(taken 2026-10-20, before PLAYUI1: new shots are LOOK1's first check, 7.4) and the asset folders.

| Area | Today | Why it reads weak |
| --- | --- | --- |
| Region tiles | `tilesets/ashlands/{floor,wall,shallow_water,...}` 32 x 32 code-drawn noise (`gen_art.py`), 1-3 variants; map rows know 3 kinds (`.`, `#`, `~`) | Flat speckle; a wall is a square brick tile with no top or face; hard square edges between floor and water; the Sunk Ward has no set (CAST1_ASSETS: "Ashlands tiles in the Sunk Ward") |
| Battle board | `common_stage/<terrain>` 128 x 96 code-drawn, one set for every zone (`rpgkit_content::art::BATTLE_STAGE_SETS = ["common_stage"]`) | Slag and vats read as lava, grease as green bog, rooftops as flat stone (CAST1_ASSETS 150-162); no edges between terrains |
| Pixel density | region tile, prop and map sprite: 2 art px per logical px; stage tile 4; battle token 1.2 (48 px frame at 40); backdrop 1 | The board's texture is finer than the units on it; Gemini props (about 2) sit on 4 |
| Title and main menu | A centred column of navy buttons on a flat background, the title in OpenSans, subtitle "A small campaign of tactical battles." (`game.json`) | `backgrounds/menu/main_menu.png` (MENU2), `backgrounds/title/<world>.png` and `backgrounds/location/<id>.png` exist but no screen draws them (only the world map backdrop is drawn); no logo; the lore title is "Emberwake: A Scarlet Lullaby" (decision "Title") |
| Menus and popups | Centred navy frames (`ui::frame`: `MODAL` fill, `FRAME` sky-blue edge, radius 4), buttons in a grid (game menu, screen 11) | Generic tool look; one depth level everywhere; no portraits or pictures on slots or the game menu |
| Panels, colours | `aegis_ui::ui` constants: `BG` rgb(10, 14, 24), `BORDER` rgb(58, 78, 116), `GOLD` rgb(244, 198, 84), `RADIUS` 2.5 | The Godot HUD navy the art bible retired (section 3: "the UI panel rgba(14, 20, 34, 0.94) and border #334155"); smooth rounded vector panels around hard pixel art |
| Fonts | One face, OpenSans SemiBold, for titles, body and numbers | The bible asks serif headers in Candle Gold and bold monospace badges (section 6) |
| Icons | PLAYUI1-I API sheet (HUD, pins, badges), ART2 skill and item icons, code-drawn status and placeholder glyphs | Three styles side by side; HUD icons fine, status glyphs the weakest |
| Hard-wired colours | About 1,000 uses of the constants in `emberwake_gui` (TEXT 233, GOLD 208, DIM 185, FOE 89, BORDER 59) and 237 raw `rgb(` / `rgba(` literals | Restyling means touching these; a theme needs a mechanism first (Q5) |

Strong today, keep: the Gemini portraits (faction backdrops, 120 px grid), the PROPS1 web props, the sprite rig, the
sunless menu composition (cathedral left, open sky right), the STAGE1 board geometry.

## 3. Style directions (pick one, Q1)

All three use the art bible's 16 hues (section 3) and its readability rule (environment at most 25 % luminance,
characters 30-65 %, telemetry at least 70 %).

| | A. Tallow and Iron (recommended) | B. Cathedral Reliquary | C. Retint only |
| --- | --- | --- | --- |
| Idea | The bible's section 6 taken literally: iron-edged pixel panels holding warm lantern light | Gothic ornament: carved basalt frames, gilt filigree, the weeping cathedral eye, per-faction trims | Today's vector widgets in the bible's colours |
| Panels | Void Abyss `#0a0b10` at 94 %, 1 px Scavenged Iron `#4b5563` edge, a 1 px darker inner bevel, 2 x 2 rivets at the corners, square corners (radius 0); parchment inset (Bleached Linen on Rotting Oak) for detail panes | Basalt frames with gilt corner pieces and dividers; Clerisy cards white and gold, Inoculated cards violet bone (section 6 deck cards) | Same fills and edges as A, rounded, no frame art |
| Type | Serif display face in Candle Gold `#fbbf24` for titles and headers; body Bleached Linen `#d9cbb4`; dim Cinder Ash `#6b7280`; badges in a monospace face | As A plus drop caps and an illuminated logo | Serif headers, the rest as today |
| Accent | Tallow Amber `#d97706` for focus and the primary action (a warm edge glow); gold only on headers | As A with gilt | As A |
| Tiles | Dark, low-contrast grounds so units and the UI's amber pop; edges and wall faces carry the shape | As A, plus ornamental floor inlays in the hub | Tiles still improved by section 4; the UI does not match them |
| Engine work | Theme + frame drawing (E1), menu panes (ME) | E1, ME, plus frames with fixed corner art at several sizes | Theme only (E1 without frames) |
| Art | Frames code-drawn (`gen_art.py`, exact pixels, palette hues); ornaments only on the title (A6) | Frames and ornaments from Gemini sheets cut to 9-slices: about 3 more batches (about 25 images) | None for the UI |
| Risk | Dark on dark: needs the 7.3 contrast check; gold overuse | Clutter on the dense battle HUD at 640 x 360; ornaments crowd small popups at Extra large text; against the preference "hide engine detail, cut information bloat" | Keeps the smooth-UI-over-pixel-art mismatch |

Recommendation: A, with B's ornament limited to the title screen and the one-time cards (story vignettes, run over),
where space allows it.

## 4. Tile-set plan

### 4.1 Sizes and the density rule

| Surface | Logical px (640 x 360 canvas) | Art px today | Rule proposed |
| --- | --- | --- | --- |
| Region tile | 16 x 16 (`region.rs` `TILE`) | 32 x 32 | keep: 2 art px per logical px |
| Region prop, map sprite | 32 px per tile; 48 px frame at 1.5 tiles | as today | keep |
| Stage cell | 32 x 24 (STAGE1 3) | 128 x 96 | new sets 64 x 48 (Q7), so the board matches the props and the map |
| Stage wall face | 32 x 12 | 128 x 48 | 64 x 24 (Q7) |
| Stage backdrop | 384 x 52 | 384 x 52 | keep (code placeholders until a later batch) |

### 4.2 Sets for the slice world (the Sunk Ward, Q3)

Region sets are bound per location (4.4); every set has the three map kinds plus a wall face and decals.

| Set | Places (CAST1 N10 briefs) | `floor` | `wall` (top) and `wall_face` | `shallow_water` (the `~` tile, art only) | Decals (PROPS1 ground props) |
| --- | --- | --- | --- | --- | --- |
| `sunk_undercroft` | the Hearth hub, Sluice Undercroft | worn basalt flagstones | basalt vault blocks, bone inlay | black sluice water, violet sheen | drain grate, cracked flag, bone shards, candle stubs, chains |
| `sunk_streets` | Cinder Alley, Meat Market, Canal Bridge, rooftops | wet cobbles (Wet Slate) | soot-black brick, timber beams | gutter and canal water | puddles, sewer grate, linen scrap, litter, lamp-light pool |
| `sunk_foundry` | The Fourth Kiln, Tallow Vats, Crucible Watchpost | iron floor plates, slag crust | kiln brick, iron bands | slag pool (glowing, Tallow Amber) | slag drips, coal heap, vent grate, grease runnel, chain coil |

Battle stage set `sunk_ward_stage` (one per world, Q11): the terrains the slice's fights use: `floor`, `stone`
(rooftop slate), `rubble`, `bridge` (iron catwalk), `deep_water`, `shallow_water`, `lava` (slag, vats), `toxic_bog`
(grease, violet mist), `burning_ground` (brazier coals), `mud`, `wall`, `wall_face`. A terrain without art falls back
to `common_stage`, as today.

### 4.3 Walls and transitions

- Region walls take STAGE1 3.3's block rule at 16 px: the `wall` art is the cap; a wall cell whose south neighbour is
  not a wall draws `wall_face` over its bottom 8 px; 1 px code lines on the cap's north and side edges where the
  neighbour is not a wall. Battle and map walls then read alike.
- Transitions (Q8 (a)): a dual-grid corner scheme. For an ordered pair (upper over lower, e.g. `floor` over
  `shallow_water`), the 4 cells around each tile corner give a 4-bit mask; masks 1-14 draw a corner piece centred on
  that corner (0 and 15 are the plain tiles). The 14 pieces per pair are composed offline by a tool (LOOK1-T) from the
  two base textures and one hand-made mask set (a ragged pixel edge with a 1 px dark rim on the lower side, light from
  the upper left), so no image model draws transitions. The battle board uses the same pieces at 64 x 48 (Q7).
- Pairs per region set: `floor` over `shallow_water`; per stage set: `floor` over each of `deep_water`,
  `shallow_water`, `lava`, `toxic_bog`, `mud` (5 pairs, 70 pieces, all tool output).

### 4.4 How the art reaches the game (schema sketch; fixed after the answers)

| Piece | Layer | Contract |
| --- | --- | --- |
| Location field `tileset` | kit schema, world data | string, default "" (the world lineage's sets, then `common`, as today); `[a-z0-9_]{2,32}`; warns when no layer holds `tilesets/<set>/floor/v0.png` |
| Zone field `stage_set` (`rpgkit_data::ZoneDef`) | kit schema, catalog data | string, default "" (`common_stage`); same id rule and warning on `tilesets/<set>/floor/v0.png` |
| Lookup | kit `rpgkit_content::art` | region: `tilesets/<location set>/<tile>`, then today's chain; stage: `tilesets/<stage_set>/<terrain>`, then `BATTLE_STAGE_SETS` |
| Files | asset layers | `tilesets/<set>/<tile>/v<n>.png` (as today), `tilesets/<set>/wall_face/v<n>.png`, `tilesets/<set>/<upper>/over_<lower>/m<NN>.png` (NN 01-14) |
| Masks and draw order | engine `aegis_ui::autotile` (new, names no terrain) | `corner_mask(grid, x, y, upper, lower) -> u8`, `block_edges(grid, x, y)`; the caller passes kinds and the pair list |
| Drawing | game `emberwake_gui` `region.rs`, `stage.rs` | ground, transition pieces, wall caps and faces, then PROPS1's passes unchanged; clicks, walking and `camera_offset` unchanged |
| Checks | kit `check/assets.rs` (next free A codes) | a bound set lacks a kind its map uses (warn); a pair with some of its 14 pieces missing (warn); a piece of the wrong size (error) |

### 4.5 Generation (Gemini web queue, scale only)

- One in-session chain per set (`metadata.record.session_ref`, no attachments): first the base `floor` (n 2, the user
  picks), then each other tile "in the same style and palette as the first image": floor variants, wall cap, wall
  face, liquid, the decal sheet (3 x 3 on a magenta key). Stage tiles chain from the region base of the same terrain,
  "the same texture seen at a low angle, 4:3".
- Prompt: a new "Tile" template in `ART_PROMPT_TEMPLATES.md` (LOOK1-A1 adds it): "A single seamless square ground
  texture tile for a dark fantasy tactical RPG, top-down, drawn as a 32 x 32 pixel tile scaled up (visible large
  pixels, no anti-aliasing), edges that wrap without a seam, no border, no frame, no vignette, no lighting gradient,
  limited palette of 4-6 colours from {hex list of the set}, light from the upper left", plus the set's material line.
- Reduction: scale only (decision "Expression sheets: scale only and aligned cells": Gemini art is already pixel art):
  detect the pixel grid, then a nearest-neighbour reduction to 32 x 32 (64 x 48 stage); no `pixelize` palette pass.
- Checks before the review board (`C:/dev/localgen/review/look1/`): size exact; seam score (mean absolute RGB step
  across the wrapped edges at most 1.5 x the mean step inside the tile; else a code wrap-fix, else re-roll); at most
  16 colours (warn); mean luminance at most 25 % (art bible readability rule); the board shows each tile 3 x 3 tiled
  and a sample map at 1x, 2x and 3x.

## 5. Menu screens and layout

### 5.1 Layout rules (engine theme values, logical px)

| Rule | Value |
| --- | --- |
| Grid | 4 px; every rect edge on it |
| Outer margin, gutter, panel padding | 8, 6, 6 |
| Type sizes | as today (`BODY` 7, `SMALL` 6, `TITLE` 7.6, `HEAD` 12, `LINE` 9) times the text size; headers in the display face |
| Hierarchy | L0 backdrop (picture or the dimmed map: 50 % veil for menus, 35 % for battle talk as today); L1 one screen or popup frame (`panel`); L2 sections (`inset`, or `parchment` for detail panes); L3 controls (`button`, `button_hot`, `button_down`, `button_off`, `tab`, `tab_on`, rows) |
| Accent | One primary action per screen in the accent role; gold for headers only |
| Popups | Centred, 600 x 300 at Normal growing with the text (as the shop and board); right click and Esc close where the UI preference says |
| Text | Every restyled screen passes `text_audit` at the four text sizes (preference "Text size") |

### 5.2 Engine pieces (standalone, each on the UI Gallery's Kit widgets page)

`Theme` and frames (E1); `Backdrop` (cover-fit picture, vignette, an optional slow drift that reduce motion stops);
`MenuColumn` (framed buttons, hotkeys, focus ring, an optional hover peek panel); `TabRail` (vertical tabs);
`SlotCard` (thumbnail, title, lines, a portrait row, corner icons); a themed `ask` dialog. They name no game.

### 5.3 Game screens (what changes; functions stay as SCREENS.md lists them)

| Screen | Layout |
| --- | --- |
| Title / main menu (00) | `Backdrop` of `backgrounds/menu/main_menu.png` (MENU2 today, A6 later); the logo top right (the title in the display face over the A6 crest, the subtitle line under it); `MenuColumn` right of centre over the open sky (x 420, from y 150, 160 x 18, step 22); Continue's peek: location, day, party portraits (20 px); version bottom right |
| New game (01) | World cards with `backgrounds/title/<world>.png` (already made) |
| Load / save (02, 17) | Six `SlotCard`s in two columns (290 x 84 at Normal): the location picture as a 112 x 63 thumbnail, name, place, day and time, the party's 20 px portraits, lock and delete top right |
| Settings (03-06) | `TabRail` left (120 wide), the form right |
| Game menu (11, Esc) | Over the dimmed map: a left rail (Resume, Party, Journal, Save, Load, Settings, Quit to menu) and the party box with portraits; the refusal reasons kept |
| Every other screen and the battle HUD | Theme and frames only (G1); no layout change. The battle overlays stay VISCLAR1's |

## 6. Art asks (Gemini web queue; no approval, a warning above about 10 images)

| Batch | Content | Images |
| --- | --- | --- |
| LOOK1-A1 | `sunk_undercroft`: floor base (n 2), floor v1-v3, wall cap, wall face, sluice water, decal sheet | 9 |
| LOOK1-A2 | `sunk_streets`: as A1 | 9 |
| LOOK1-A3 | `sunk_foundry`: as A1 (slag for the liquid) | 9 |
| LOOK1-A4 | `sunk_ward_stage` part 1: floor, stone, rubble, bridge, deep_water, shallow_water, wall_face | 7 |
| LOOK1-A5 | `sunk_ward_stage` part 2: lava (slag), toxic_bog (grease), burning_ground, mud, wall cap | 5 |
| LOOK1-A6 | Title backdrop 16:9 (n 3, the MENU2 composition), logo crest 1:1 without text (n 2) | 5 |
| LOOK1-A7 (later) | Sunk Ward district map backdrop 16:9 (n 2), location pictures for the Hearth and the 4 slice nodes (thumbnails, 16:9) | 7 |

Frames and the theme's swatches are code (`gen_art.py` group `ui_frames`), never the image model (exact pixels).

## 7. Checks (rule-derived)

1. E1: with the engine default theme every Kit widgets page and screen shot is pixel-identical to today's (the
   default reproduces the constants); a game theme changes only the roles it sets.
2. E2: `corner_mask` gives each of the 16 masks on a fixture grid; an isolated water cell draws 4 corner pieces; the
   result depends only on the grid.
3. G1: text roles against the panel fill at least 4.5:1 contrast (dim at least 3:1); Bleached Linen on Void Abyss
   passes.
4. Before and after shots of every player screen over the agent link (`shot`) at 1280 x 720, with `text_audit` clean
   at the four sizes.
5. L: a wall cell with an open south neighbour draws its face; the journey golden, store resave and battle traces are
   unchanged (presentation only; no Godot parity check retires).
6. T and K: section 4.5's tile checks; 4.4's asset warnings on a fixture world.

## 8. Slices and task rows (the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| LOOK1-S | The answers to section 9 and the spec revised to them (theme schema fixed) | game + engine | design | - | `games/emberwake/docs/specs/LOOK1.md` |
| LOOK1-E1 | `aegis_ui::theme`: roles, `ui/theme.json` over the asset layers (engine default = today), body / display / mono fonts, `Painter::nine` frames with sidecars, layout values; Kit widgets Theme page | engine | logic | LOOK1-S | `aegis_ui/src/{theme (new),ui,painter,shell,specimens}.rs`, `engine/assets/ui/`, tests |
| LOOK1-G1 | The game theme (direction of Q1) and the `ui_frames` art; `emberwake_gui` reads the theme (Q5 migration of the constants and chrome literals) | game | logic | LOOK1-E1; a flush point (after CLN-52) | `emberwake_gui/src/**`, `games/emberwake/assets/ui/`, `tools/gen_art.py` |
| LOOK1-ME | Engine panes: `Backdrop`, `MenuColumn`, `TabRail`, `SlotCard`, themed ask; Gallery entries | engine | logic | LOOK1-E1 | `aegis_ui/src/{ui,specimens}.rs` or new modules, tests |
| LOOK1-M | Title / main menu, new game cards, save / load cards, settings rail, game menu (5.3) | game | logic | LOOK1-ME, LOOK1-G1 | `emberwake_gui/src/{menus,screens,hub,gallery}.rs` |
| LOOK1-E2 | `aegis_ui::autotile` (corner masks, block edges) and the Gallery Tiles pane (a set on a sample grid, square and tilted, 1x-3x) | engine | logic | LOOK1-S | `aegis_ui/src/autotile.rs` (new), `specimens.rs`, tests |
| LOOK1-K | `tileset` on locations, `stage_set` on zones, the lookups and A-code checks | kit | logic | LOOK1-S | `kits/rpg/schema.json`, `rpgkit_data/src/lib.rs`, `rpgkit_content/src/{art.rs,check/assets.rs}`, tests |
| LOOK1-L | Region and stage drawing: transitions, wall caps and faces, the bound sets | game | logic | LOOK1-E2, LOOK1-K; EXPLOREVIEW1 for the region part (Q4) | `emberwake_gui/src/{region,stage}.rs` |
| LOOK1-T | `tools/tiles/`: grid detect and scale-only cut, seam check and wrap fix, the transition composer, review boards; tests | game (tools) | logic | LOOK1-S | `tools/tiles/` (new) |
| LOOK1-A1 to -A7 | The art batches of section 6 (queue, ingest, boards; planner worktree) | game (art) | art | LOOK1-T; A1-A3 after Q4 | `art_src/`, `C:/dev/localgen/review/look1/`, `ART_PROMPT_TEMPLATES.md` |
| LOOK1-R | Place the picked art: tiles, pieces, decal props, backdrops; data rows (`tileset`, `stage_set`) through `aegis content` | game (data) | asset | LOOK1-K, LOOK1-T, the picks; SUNKWARD1 for the location rows | `games/emberwake/assets/{tilesets,props,backgrounds,ui}/`, world and catalog rows |

Order: S, then E1 and E2 and T and K in parallel (no shared files), then G1 at a flush point, ME, M, L; the art
batches as soon as T lands (A4-A6 do not wait on EXPLOREVIEW1).

## 9. Open questions (recommended option first)

- Q1. Style direction: (a) A Tallow and Iron, with B's ornament on the title and one-time cards; (b) B Reliquary;
  (c) C Retint only. Recommend (a).
- Q2. First slice: (a) theme, frames, fonts, the title screen and the battle stage set (E1, G1, ME, M for the title
  only, E2, K, T, L stage part, A4-A6); (b) menus only; (c) tiles only. Recommend (a): the most visible change, and
  none of it waits on the region view.
- Q3. World first: (a) the Sunk Ward (the M2 slice); the preference "One world first: per-world art waits until the
  Ashlands are complete" (2026-10-18) predates the slice and STORYART1 already drew Sunk Ward art, so confirm it is
  lifted for tiles; (b) the Ashlands (today's start world). Recommend (a); the sets are reviewed in the Gallery Tiles
  pane before SUNKWARD1 binds them.
- Q4. Region view: (a) EXPLOREVIEW1's test first (decision "Battle and exploration view": the style is picked after
  it), then A1-A3; (b) keep square now (STAGE1 section 8's recommendation) and generate. Recommend (a).
- Q5. Theme mechanism: (a) a runtime theme from `ui/theme.json` over the asset layers, the constants becoming accessors
  (a mechanical edit of about 1,000 uses, at a flush point); (b) retune the engine constants (every game inherits
  Emberwake's colours). Recommend (a).
- Q6. Fonts: (a) an OFL serif display face for titles and headers (a pick board: Alegreya SC, Cinzel, IM Fell English
  SC) and OpenSans kept for body text; (b) a pixel font everywhere (looks right only at whole multiples, so it breaks
  the four text sizes); (c) keep OpenSans. Recommend (a).
- Q7. Stage tile density: (a) new sets at 64 x 48 (2 art px per logical px, as the map, props and sprites); (b) 128 x
  96 like `common_stage`. Recommend (a).
- Q8. Transitions: (a) code-composed corner pieces from the Gemini bases; (b) Gemini transition sheets per pair (seams
  and drift between pieces likely); (c) none, wall faces and decals only. Recommend (a).
- Q9. Ground kinds on region maps: (a) not now: three kinds per set plus decals; (b) a `ground` char layer in map
  templates for more kinds with transitions (HISTORY's "Buildings as more tile kinds" was about buildings looking
  blocky; transitions remove the blockiness, but it is a kit schema change). Recommend (a).
- Q10. Green: the bible has no green hue; (a) keep today's green for HP bars and good changes as a telemetry
  exception; (b) Salt Cyan for good. Recommend (a).
- Q11. Battle stage sets: (a) one per world (`sunk_ward_stage`), a zone may name its own later; (b) one per zone
  (7 sets, about 35 more images). Recommend (a).

## 10. Docs delta (when approved)

- Art bible section 6: the theme roles and frames; section 7.3's "Tiles, props, backdrops: code first, image-API
  sheets only when asked" now follows the Gemini queue decisions; section 2's region map row gets the set binding.
- `assets/README.md`: `wall_face`, `over_<lower>/m<NN>.png`, `ui/frames/`, `ui/theme.json`. `engine/README.md`: theme,
  autotile, the new panes. `engine/docs/authoring/`: a game-agnostic tiles note ("promote generic methods").
- ROADMAP: a LOOK1 row, `proposed`. HISTORY (if Q1 (a) or (c)): "The Godot HUD navy palette as the UI look" replaced
  by the game theme.

## 11. Out of scope

Battle overlay choices (VISCLAR1); layout mockups and resizable layouts (LAYOUT1); controller input (UIR-7); new
screens or functions; animated tiles; a press-any-key splash; Ashlands, Iron Reaches and Tidewrack sets; large-unit
art; music and sound.
