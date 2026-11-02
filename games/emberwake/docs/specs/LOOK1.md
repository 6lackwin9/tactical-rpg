# LOOK1: visual pass (a game theme, menu screens and Sunk Ward tile sets)

Status: approved (user 2026-11-02, decision "LOOK1 answers" in `docs/decisions/workflow.md`, all as recommended);
revised to the answers by LOOK1-S. Was: proposed (options spec 2026-11-02, the user's "lets start adding some better
tile sets, menu screens, layout and styling for the game assets etc")
Depends on: UIKIT1, UIKIT2, STAGE1, PROPS1-L2, PLAYUI1, EXPLOREVIEW1 (before the region art), the art route decisions
in `docs/decisions/workflow.md` (Gemini web app route, scale-only sheets, art from the planner worktree)
Lane: design, then logic (E1, G1, G2, ME, M1, M2, E2, K, L1, L2, T), art (F, A1-A7), asset (R)
Touches: by slice, `games/emberwake/docs/specs/LOOK1_TASKS.md` (briefs: touches, steps, tests, docs delta). Crates:
`aegis_*` in `engine/crates/`, `rpgkit_*` in `kits/rpg/crates/`, `emberwake_*` in `games/emberwake/crates/`.

History: "Buildings as more tile kinds" and "painted location backgrounds as the region map image" (2026-10-22, not
adopted) hold (buildings stay PROPS1 props; location pictures only as save-slot thumbnails); STAGE1 options A and C
hold; the Godot HUD navy palette is replaced by this theme (HISTORY 2026-11-02).

## 1. Goal

One look that matches the Gemini pixel-art portraits and props: an engine runtime theme (colour roles, three font
faces, nine-slice frames, layout values) that the game sets in `ui/theme.json`; a title screen and restyled menus
built from reusable engine panes; tile sets for the Sunk Ward (region maps and the battle board) with wall faces and
code-composed edge transitions. Presentation only: no rule, save, sim, trace or journey reads any of it, and no Godot
parity check retires.

## 2. Style: Tallow and Iron (Q1)

The art bible's section 6 taken literally: iron-edged pixel panels holding warm lantern light. Square corners
(radius 0), Void Abyss fills at 94 %, a 1 px Scavenged Iron edge with a darker inner bevel and riveted corners;
parchment insets (Bleached Linen on Rotting Oak) for detail panes; serif display headers in Candle Gold; body text in
Bleached Linen with OpenSans; Tallow Amber marks focus and the one primary action per screen (gold stays on headers).
B's ornament (gilt basalt corners, the crest) appears only on the title screen and the one-time cards (story
vignettes, the run-over screen) through the `ornate` frame. Tiles are dark and low-contrast (environment at most 25 %
luminance, the bible's readability rule) so units and the amber UI pop; edges and wall faces carry the shapes.
Telemetry keeps today's foe red and good green (Q10).

## 3. Theme contract (`ui/theme.json`, LOOK1-E1; schema-first: yes)

### 3.1 File and merge

`ui/theme.json` may sit in any asset layer (engine `engine/assets/`, kit `kits/rpg/assets/`, game
`games/emberwake/assets/`). The loader reads them bottom-up (engine, kit, game); each file sets only the keys it
names; a key no file sets keeps the code default `Theme::default()`, which is today's look (every constant of
`aegis_ui::ui` as of develop 11c29695). The engine ships no theme file; Emberwake ships the game theme (LOOK1-G1).
Loaded at start and again on the Scene Runner's Reload data; any error keeps the previous theme (at start the default)
and is shown through `show_errors` like a data load error. It is an asset file, not content: no `aegis content` row.

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `version` | int | required | 1; anything else: `ui/theme.json (<layer>): version must be 1` |
| `colors` | object, role to colour | `{}` | keys from 3.2 only; else `unknown colour role '<k>' (allowed: ...)` |
| `game_colors` | object, name to colour | `{}` | names `[a-z][a-z0-9_]{1,31}`; read by game code only (`Theme::game(name, fallback)`) |
| `fonts` | object, face to font | `{}` | faces `body`, `display`, `mono` (3.3) |
| `layout` | object | `{}` | keys from 3.4 with their ranges |
| `frames` | bool | true | false ignores every frame file (vector panels in the colours; a Gallery comparison switch) |

A colour is `#rrggbb` or `#rrggbbaa` (hex, either case); anything else is an error naming the file and key.

### 3.2 Colour roles

E1 maps every colour constant and literal the engine widgets draw with to a role; a value the table lacks gets a new
role with that value as its default, named in E1's report (the planner adds it here).

| Role | Drawn for | Default (today) | Tallow and Iron (G1) |
| --- | --- | --- | --- |
| `bg` | screen ground | `BG` #0a0e18 | Void Abyss #0a0b10 |
| `panel` | `Ui::frame` and popup fill | `MODAL` #121a2cff | #0a0b10f0 (94 %) |
| `panel_edge` | frame edge | `FRAME` #8cc4ec | Scavenged Iron #4b5563 |
| `card` | button, card and row fill | `CARD` #10182afa | Damp Basalt #14171ffa |
| `card_hot` | hovered fill | `PANEL_HI` #223252fa | Wet Slate #1e232dfa |
| `selected` | active or selected fill | `SELECTED` #564416fa | Rotting Oak #2c221efa |
| `edge` | idle and disabled border | `BORDER` #3a4e74 | #4b5563 |
| `edge_live` | enabled button border | #6482ba | Cinder Ash #6b7280 |
| `edge_hot` | hovered card border | #6e8cc4 | Tallow Amber #d97706 |
| `focus` | hover and active button border, focus ring, primary action | `GOLD` #f4c654 | Tallow Amber #d97706 |
| `text` | body text | `TEXT` #dee4f0 | Bleached Linen #d9cbb4 |
| `dim` | secondary and disabled text | `DIM` #8492ac | Cinder Ash #6b7280 |
| `gold` | headers, titles, tip titles | `GOLD` #f4c654 | Candle Gold #fbbf24 |
| `info` | neutral highlight | `INFO` #8cc4ec | Salt Cyan #06b6d4 |
| `foe` | foes, damage | `FOE` #f0645c | not set (today's) |
| `good` | good changes, HP | `GOOD` #60d26e | not set (Q10) |
| `ink` | text on light panes (parchment) | #101014 | Rotting Oak #2c221e |

`frame_line` (default #92400e; Tallow and Iron #991b1b): the thin line of a frame that has one (LOOK1-R1).

LOOK1-E1B adds 14 roles, each defaulting to the value drawn before; their Tallow and Iron values come with LOOK1-G2:
`title_idle` #1a243a, `ruler` #101624, `playing` #346096, `playing_edge` #80dcff, `lane` #181e2c, `lane_stripe`
#3c42548c, `lane_hatch` #7880965a, `impact_edge` #f0f4ffdc, `socket` #060910, `socket_edge` #c8d2e4dc, `tool_bg`
#0c101a, `tool_pane` #121826, `stage` #0a0b10, `mote` #ceccc8.

### 3.3 Fonts

`fonts.<face>` is `{"file": "fonts/<name>.ttf", "scale": 1.0}`: `file` layer-relative, found through the layers (game
first), a missing or unreadable file is an error; `scale` 0.5-2.0 multiplies every size drawn in that face (evens out
x-heights; the text size setting still applies on top). Defaults: `body` = `fonts/OpenSans-SemiBold.ttf` at 1.0;
`display` and `mono` = the body face, so the default theme draws one face as today.

| Face | Drawn for |
| --- | --- |
| `body` | every text as today |
| `display` | `Ui::frame` titles, the new `Ui::header`, the title screen's logo and subtitle, `SlotCard` and `MenuColumn` titles |
| `mono` | the new `Ui::badge` (numbers on badges); default the body face (question 11.1) |

### 3.4 Layout values (logical px; used by the ME panes and restyled screens, old layouts keep their numbers)

| Key | Type | Default | Range | Tallow and Iron |
| --- | --- | --- | --- | --- |
| `radius` | float | 2.5 | 0-8 | 0 |
| `frame_radius` | float | 4.0 | 0-8 | 0 |
| `frame_title_h` | int | 26 | 12-48 | 26 |
| `grid` | int | 4 | 1-8 | 4 |
| `margin` | int | 8 | 0-32 | 8 |
| `gutter` | int | 6 | 0-24 | 6 |
| `padding` | int | 6 | 0-24 | 6 |
| `focus_glow` | int | 0 | 0-3 | 1 (a 1 px `focus` ring outside the hovered or primary button) |

Type sizes stay code constants (`BODY` 7, `SMALL` 6, `TITLE` 7.6, `HEAD` 12, `LINE` 9, times the text size).

### 3.5 Frames and their sidecars

A frame is `ui/frames/<role>.png` with `ui/frames/<role>.json` beside it, both from one layer (game first). Roles and
fallbacks (a role with no file uses the next; none: today's vector panel in the role's colours and radius):

| Role | Fallback | Drawn by |
| --- | --- | --- |
| `panel` | vector | `Ui::frame`, popups, the ask dialog |
| `inset` | vector | `Ui::inset` (new), sections inside a panel |
| `parchment` | `inset` | `Ui::inset` with parchment on, detail panes; text in its `ink` |
| `button`, `button_hot`, `button_down`, `button_off` | `button_hot` and `button_off` to `button`, `button_down` to `button_hot` | `Ui::button_full` idle, hover, active, disabled |
| `tab`, `tab_on` | `tab` to `button`, `tab_on` to `button_down` | `Ui::tab`, `TabRail` |
| `card`, `card_hot`, `card_on` | `card_hot` to `card`, `card_on` to `card_hot` | `Ui::card`, `SlotCard` |
| `tip` | `panel` | tooltips (`draw_tip`) |
| `ornate` | `panel` | the title screen panels and the one-time cards |

| Sidecar key | Type | Default | Validation |
| --- | --- | --- | --- |
| `slice` | [left, top, right, bottom] art px | required | each at least 1; left + right below the PNG width, top + bottom below its height |
| `scale` | int, art px per logical px | 2 | 1-4 |
| `edges` | `stretch` or `tile` | `stretch` | |
| `center` | `stretch`, `tile` or `none` | `stretch` | `none` draws the role's fill colour instead |
| `pad` | [l, t, r, b] logical px | the slices divided by `scale` | 0-32 each; the content inset |
| `ink` | colour role | `text` | a 3.2 role |

A PNG without its sidecar, a sidecar without its PNG, an unknown role file in `ui/frames/` or a bad key is an error.
`Painter::nine` draws the corners at `slice / scale` logical px, nearest sampling; a rect smaller than two corners
shrinks the corners to half the rect each way.

### 3.6 Accessors

`aegis_ui::theme`: `Theme` (the roles as fields, `game_colors`, faces, layout, frames), `th()` (the current theme,
cheap enough per draw call), `set_theme`, `load(&Layers) -> Result<Theme, Vec<String>>`, `contrast(fg, bg) -> f32`
(WCAG 2 ratio, a translucent fg or bg composited over `bg` first). Engine widgets draw only through `th()`. The `ui`
colour constants stay as the default values until LOOK1-G2 has moved the game's uses, then a cleanup row deletes them.

## 4. Fonts: the pick board (LOOK1-F, Q6)

Candidates, all SIL OFL 1.1 from `github.com/google/fonts` (`ofl/alegreyasc`, `ofl/cinzel`, `ofl/imfellenglishsc`):
Alegreya SC, Cinzel, IM Fell English SC; OpenSans SemiBold stays the body face. Board, licence record and the pick
(one face and its `scale`, recorded by the planner as a decision row): LOOK1-F in `LOOK1_TASKS.md`.

## 5. Tile sets

### 5.1 Sizes (2 art px per logical px everywhere, Q7)

| Surface | Logical px | Art px |
| --- | --- | --- |
| Region tile | 16 x 16 square or 16 x 12 tilted (EXPLOREVIEW1 picks, Q4) | 32 x 32 or 32 x 24 |
| Region wall face | 16 x 8 square or 16 x 6 tilted | 32 x 16 or 32 x 12 |
| Stage cell | 32 x 24 (STAGE1 3) | 64 x 48 |
| Stage wall face | 32 x 12 | 64 x 24 |
| Stage backdrop | 384 x 52 | 384 x 52 (code placeholders until a later batch) |

### 5.2 Sets (the Sunk Ward first, Q3; one battle stage set per world, Q11; no new ground kinds, Q9)

| Set | Places | `floor` | `wall` (cap) and `wall_face` | `shallow_water` (`~`, art only) | Decals (PROPS1 ground props) |
| --- | --- | --- | --- | --- | --- |
| `sunk_undercroft` | the Hearth hub, Sluice Undercroft | worn basalt flagstones | basalt vault blocks, bone inlay | black sluice water, violet sheen | drain grate, cracked flag, bone shards, candle stubs, chains |
| `sunk_streets` | Cinder Alley, Meat Market, Canal Bridge, rooftops | wet cobbles | soot-black brick, timber beams | gutter and canal water | puddles, sewer grate, linen scrap, litter, lamp-light pool |
| `sunk_foundry` | The Fourth Kiln, Tallow Vats, Crucible Watchpost | iron plates, slag crust | kiln brick, iron bands | slag pool (glowing amber) | slag drips, coal heap, vent grate, grease runnel, chain coil |

Stage set `sunk_ward_stage`: `floor`, `stone` (rooftop slate), `rubble`, `bridge` (iron catwalk), `deep_water`,
`shallow_water`, `lava` (slag, vats), `toxic_bog` (grease, violet mist), `burning_ground` (brazier coals), `mud`,
`wall`, `wall_face`. A terrain without art falls back to `common_stage`, as today.

### 5.3 Walls and transitions (Q8: composed in code)

- Walls: STAGE1 3.3's block rule. Battle as built; the region map's walls follow the EXPLOREVIEW1 pick (square: the
  cap is the tile and `wall_face` covers the bottom 8 px when the south neighbour is not a wall; tilted: EXPLOREVIEW1
  section 3's lifted blocks). 1 px code lines on the cap's open north and side edges.
- Transitions, a dual-grid corner scheme: for an ordered pair (upper over lower) the 4 cells around a tile corner give
  a mask (bit 1 north-west, 2 north-east, 4 south-west, 8 south-east is upper); masks 1-14 draw piece `m<NN>`
  centred on that corner, after the ground pass; 0 and 15 draw nothing extra. Pieces are opaque cell-sized composites
  made offline by LOOK1-T from the two `v0` textures (tiled half a cell offset, so their pixels match the tiles under
  them) and a deterministic ragged mask with a 1 px dark rim on the lower side, light from the upper left. No image
  model draws a transition.
- Pairs: region sets `floor` over `shallow_water`; `sunk_ward_stage` `floor` over `deep_water`, `shallow_water`,
  `lava`, `toxic_bog` and `mud` (5 pairs, 70 pieces).

### 5.4 Binding (LOOK1-K; schema-first: yes)

| Piece | Layer | Contract |
| --- | --- | --- |
| Location field `tileset` | kit schema, world data | string, default "" (the world lineage's sets, then `common`, as today); `[a-z0-9_]{2,32}`; warning when no layer holds `tilesets/<set>/floor/v0.png` |
| World manifest field `stage_set` | kit schema, world data | string, default "" (`common_stage`); same id rule and warning; one per world (Q11); a zone field is out of scope |
| Lookups | kit `rpgkit_content::art` | region `tilesets/<location tileset>/<tile>`, then today's chain; stage `tilesets/<world stage_set>/<terrain>`, then `BATTLE_STAGE_SETS` |
| Files | asset layers | `tilesets/<set>/<tile>/v<n>.png` (as today), `tilesets/<set>/wall_face/v<n>.png`, `tilesets/<set>/<upper>/over_<lower>/m<NN>.png` (NN 01-14) |
| Masks | engine `aegis_ui::autotile` (LOOK1-E2, names no terrain) | `corner_mask`, `block_edges` (`LOOK1_TASKS.md` E2) |
| Asset checks | kit `check/assets.rs`, the next free A codes | a bound set lacks a kind its map uses (warning); a pair with some of its 14 pieces missing (warning); a tile, face or piece of the wrong size (error) |

### 5.5 Generation

Gemini web queue, one in-session chain per set, scale-only reduction and the tile checks before a review board:
`LOOK1_TASKS.md` A1-A7 (the "Tile" prompt template, the chain order, the check numbers).

## 6. Menu screens and layout

Layout rules: every rect edge on the 4 px grid; margin 8, gutter 6, padding 6; hierarchy L0 backdrop (a picture, or
the dimmed map: 50 % veil for menus, 35 % for battle talk as today), L1 one `panel` (or `ornate`) frame, L2 `inset` or
`parchment` sections, L3 controls; one primary action per screen in `focus`; popups centred, 600 x 300 at Normal and
growing with the text; every restyled screen passes `text_audit` at the four text sizes.

Engine panes (LOOK1-ME, each on the UI Gallery's Kit widgets page, naming no game): `Backdrop`, `MenuColumn`,
`TabRail`, `SlotCard`, the themed ask dialog.

| Screen | Layout (functions stay as `SCREENS.md` lists them) |
| --- | --- |
| Title / main menu (00, M1) | `Backdrop` of `backgrounds/menu/main_menu.png` (the A6 title backdrop once picked); the logo top right (the title in the display face over the A6 crest, the subtitle under it); `MenuColumn` at x 420 from y 150, 160 x 18, step 22; Continue's peek (location, day, 20 px party portraits) in an `ornate` panel left of the column; the version bottom right |
| New game (01, M2) | world cards with `backgrounds/title/<world>.png` |
| Load / save (02, 17, M2) | six `SlotCard`s in two columns (290 x 84 at Normal): the location picture as a 112 x 63 thumbnail, name, place, day and time, the party's 20 px portraits, lock and delete top right |
| Settings (03-06, M2) | `TabRail` left (120 wide), the form right |
| Game menu (11, M2) | over the dimmed map: a left rail (Resume, Party, Journal, Save, Load, Settings, Quit to menu) and the party box with portraits; refusal reasons kept |
| Story vignette cards, run over (M2) | the `ornate` frame |
| Everything else, the battle HUD | the theme and frames only (G1, G2); no layout change; battle overlays stay VISCLAR1's |

## 7. Art (Gemini web queue; no approval, a warning above about 10 images)

| Batch | Content | Images |
| --- | --- | --- |
| A1 | `sunk_undercroft`: floor base (n 2), floor v1-v3, wall cap, wall face, sluice water, decal sheet (3 x 3 on magenta) | 9 |
| A2 | `sunk_streets`: as A1 | 9 |
| A3 | `sunk_foundry`: as A1 (slag for the liquid) | 9 |
| A4 | `sunk_ward_stage` part 1: floor, stone, rubble, bridge, deep_water, shallow_water, wall_face | 7 |
| A5 | `sunk_ward_stage` part 2: lava (slag), toxic_bog (grease), burning_ground, mud, wall cap | 5 |
| A6 | title backdrop 16:9 (n 3, the MENU2 composition), logo crest 1:1 without text (n 2), `ornate` corner sheet (4 gilt basalt corners and a divider, magenta key; n 2) | 7 |
| A7 (later) | Sunk Ward district map backdrop 16:9 (n 2), location pictures for the Hearth and the 4 slice nodes (16:9) | 7 |

A1-A3 wait for EXPLOREVIEW1's pick (Q4: their aspect follows it). Frames other than `ornate` and the theme swatches
are code (`tools/gen_art.py` group `ui_frames`, exact pixels), never the image model.

## 8. Checks (rule-derived; each slice's tests in `LOOK1_TASKS.md`)

1. Default theme: every Kit widgets page and screen shot is pixel-identical to develop before E1 (frame hashes
   recorded before E1's first edit); a theme file changes only the roles it sets.
2. Contrast on `panel`, `card` and `selected` over `bg`: `text`, `gold`, `info` and `ink` (on parchment) at least
   4.5:1; `dim` at least 3:1. Tallow and Iron: `text` on `panel` 12.3:1, `dim` on `card` 3.7:1, `gold` on `selected`
   9.3:1, `ink` on parchment 9.7:1.
3. `corner_mask` gives each of the 16 masks on a fixture grid; an isolated water cell draws 4 pieces; the result
   depends only on the grid.
4. Before and after shots of every player screen over the agent link (`shot`, 1280 x 720) with `text_audit` clean at
   the four sizes.
5. Journey golden, store resave and battle traces unchanged; the tile checks (A briefs) and 5.4's asset warnings.

## 9. Slices (briefs in `LOOK1_TASKS.md`)

| Id | Title | Layer | Lane | Depends on |
| --- | --- | --- | --- | --- |
| E1 | `aegis_ui::theme`: roles, layered `ui/theme.json`, faces, nine-slice frames, layout values; Theme page | engine | logic | S |
| F | Display font pick board and licence record | game (art) | art | - |
| G1 | The Tallow and Iron theme file, `ui_frames` art, the picked font | game | logic | E1, F |
| G2 | `emberwake_gui` reads the theme (constants and chrome literals) | game | logic | G1; a flush point after CLN-52 |
| ME | Engine panes `Backdrop`, `MenuColumn`, `TabRail`, `SlotCard`, themed ask | engine | logic | E1 |
| M1 | Title / main menu | game | logic | ME, G1 |
| M2 | New game, load / save, settings rail, game menu, `ornate` one-time cards | game | logic | M1, G2 |
| E2 | `aegis_ui::autotile` and the Gallery Tiles pane | engine | logic | S |
| K | `tileset`, `stage_set`, lookups, asset checks | kit | logic | S |
| T | `tools/tiles/`: cut, seam check and fix, transition composer, boards | game (tools) | logic | S |
| L1 | Battle stage: the world's stage set, transitions, wall faces | game | logic | E2, K |
| L2 | Region map: the bound sets, transitions, wall caps and faces in the picked view | game | logic | L1, EXPLOREVIEW1-R |
| A1-A7 | Art batches of section 7 | game (art) | art | T; A1-A3 after EXPLOREVIEW1-R; A6 after F |
| R | Place the picks; `tileset` and `stage_set` rows through `aegis content` | game (data) | asset | K, T, the picks; SUNKWARD1 for location rows |

First slice (Q2): E1, F, G1, ME, M1, E2, K, T, L1, A4-A6, R (stage set and title only). Then G2 at its flush point,
M2, EXPLOREVIEW1's review, A1-A3, L2, R (region sets), A7. E1, E2, K and T share no files and run in parallel.

## 10. Decided and rejected

Decided: decision "LOOK1 answers" (user 2026-11-02), Q1-Q11 all as recommended; sections 2-9 are written to it.
Rejected (not adopted; the planner adds the HISTORY rows): Cathedral Reliquary everywhere (Q1 b: clutter on the
640 x 360 battle HUD and small popups at Extra large text); retint only (Q1 c: smooth vector panels around hard pixel
art); menus only or tiles only first (Q2 b, c); the Ashlands first (Q3 b); region art before the view test (Q4 b);
retuning the engine constants (Q5 b: every game would inherit Emberwake's colours); a pixel font everywhere (Q6 b:
only whole multiples look right, breaking the four text sizes) or OpenSans alone (Q6 c); 128 x 96 stage tiles (Q7 b:
finer than the units on them); Gemini transition sheets (Q8 b: seams and drift) or none (Q8 c); a `ground` char layer
for more region kinds (Q9 b); Salt Cyan for good (Q10 b); a stage set per zone (Q11 b, about 35 more images).

## 11. Open questions (recommended first)

1. Badge numbers (the bible's "bold monospace"): (a) add two OFL mono candidates (JetBrains Mono Bold, IBM Plex Mono
   Bold) to the font board, shipped only if picked; (b) badges stay in the body face. Recommend (a).
2. The title screen subtitle (`game.json` `subtitle`, today "A small campaign of tactical battles."): (a) "A Scarlet
   Lullaby" (decision "Title"); (b) keep it. Recommend (a).

## 12. Out of scope

Battle overlay choices (VISCLAR1); layout mockups and resizable layouts (LAYOUT1); controller input (UIR-7); new
screens or functions; animated tiles; a press-any-key splash; a backdrop drift (no reduce-motion setting exists);
Ashlands, Iron Reaches and Tidewrack sets; zone stage sets; large-unit art; music and sound.
