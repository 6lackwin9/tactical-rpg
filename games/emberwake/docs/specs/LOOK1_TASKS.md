# LOOK1 slice briefs (touches, steps, tests)

Companion to `games/emberwake/docs/specs/LOOK1.md` (the contract: theme 3, fonts 4, tiles 5, screens 6, art 7, checks
8, order 9). Status: approved with LOOK1 (decision "LOOK1 answers", user 2026-11-02). Every logic slice ends with the
gate, `tools/fc.sh docs` and the docs delta of its section; every screen slice adds before and after shots over the
agent link (`shot`, 1280 x 720) and `text_audit` clean at the four text sizes.

## E1: `aegis_ui::theme` (engine, logic; depends on LOOK1-S)

Touches: `engine/crates/aegis_ui/src/{theme.rs (new), ui.rs, painter.rs, shell.rs, screen.rs, specimens.rs, lib.rs}`,
`engine/crates/aegis_ui/tests/theme.rs` (new) with `tests/fixtures/theme/`, `engine/README.md`,
`engine/assets/README.md`.

1. Before the first edit: record the frame hash of every Kit widgets page at 1280 x 720 at the four text sizes on
   develop into `tests/fixtures/theme/default_hashes.json`.
2. `theme.rs` per LOOK1 3 (merge, validation messages, roles, layout, `game_colors`, `th`, `set_theme`, `load`,
   `contrast`).
3. `Fonts` holds three faces with a glyph cache each (`Fonts::new(body)` keeps working: display and mono = body);
   `Painter::text_face`, `text_w_face`, `line_h_face`; a face's `scale` multiplies its sizes.
4. `Painter::nine(img, &Nine, r)` (LOOK1 3.5).
5. Engine widgets through `th()` and the frame roles: `button_full`, `tab`, `card`, `row`, `select`, fields, scroll
   bars, `frame` (title in the display face), `draw_tip`, the ask dialog; new `Ui::header`, `Ui::badge(r, text,
   role)`, `Ui::inset(r, parchment) -> Rect` (the content rect).
6. The shell loads the theme from the game's `Layers` at start and on Reload data; errors through `show_errors`.
7. Kit widgets page "Theme": every role's swatch, name and value; each frame role at 40 x 16, 120 x 40, 300 x 120;
   the three faces at the four text sizes; the LOOK1 8.2 contrast pairs with pass or fail; a switch engine default /
   the layers' theme / frames off.

Tests: (1) the default hashes match; (2) merge: a fixture kit layer sets `text`, a game layer sets `gold` and
`layout.radius`: both apply, every other role is the default; (3) one test per error: version 2, unknown role, colour
`#12345`, game colour name `Bad`, missing font file, unknown face, `layout.grid` 9, PNG without sidecar, sidecar
without PNG, slice wider than the PNG, unknown frame role file, unknown `ink`; (4) nine: a 16 x 16 fixture (slice 4
each, scale 2) into a 40 x 20 logical rect at view scale 2: the four 8 x 8 corner blocks equal the source corners
pixel for pixel, `center: none` shows the fill, a 6 x 6 rect shrinks the corners; (5) faces: the fixture display face
(OpenSans at `scale` 1.5) measures 1.5 x the body width within 1 px; with no fonts block both measure alike; (6)
`contrast(#ffffff, #000000)` is 21.0 and `contrast(#777777, #ffffff)` 4.48 within 0.01; (7) `frames: false` draws the
vector panel; (8) `text_audit` clean on the Theme page at the four sizes; (9) with a fixture theme the hovered
button's border pixel is the `focus` colour.

## F: display font pick board (game art; planner worktree; no dependency)

1. Download `ofl/alegreyasc`, `ofl/cinzel`, `ofl/imfellenglishsc` (and, with LOOK1 question 11.1 (a),
   `ofl/jetbrainsmono`, `ofl/ibmplexmono`) from `github.com/google/fonts`: the TTFs, `OFL.txt` and `METADATA.pb` of
   each into its own new folder `C:/dev/localgen/fonts/<family>/` (untrusted downloads: scripts elsewhere, read only
   as data). Prefer a static TTF; a variable file (for example `Cinzel[wght].ttf`) is shown at its default instance,
   the only one fontdue draws.
2. The board script lives in the planner's scratchpad (not shipped): Python 3.12 with Pillow (full path),
   `ImageFont.truetype` at the physical px the engine draws at 1280 x 720 (logical size x 2 x the text size). Per
   candidate a 1280 x 300 strip on Void Abyss with a G1-style panel: "Emberwake" at 56 px and "A Scarlet Lullaby" at
   20 px in Candle Gold; "The Hearth", "Sluice Undercroft", "Party", "Victory", "Round 3 of 12" at 20, 24, 28 and
   31 px (HEAD at Small to Extra large); under each a body line in OpenSans SemiBold 14 px, Bleached Linen; the digits
   and `.,:;!?'-()`; the `scale` that gives OpenSans's cap height at 24 px, printed. Mono candidates: "12", "x3",
   "-45", "100%" at 12 and 14 px in Arterial Red and Salt Cyan on badge chips.
3. Output `C:/dev/localgen/review/look1/fonts/board.png` and `index.html` with `file:///` links; the user picks a face
   and its `scale`; the planner writes the decision row.
4. Licence record for the pick: the TTF unmodified as `games/emberwake/assets/fonts/<File>.ttf`, its `OFL.txt` beside
   it as `<Family>-OFL.txt`, and a line in `games/emberwake/assets/README.md`: family, version (name table), "SIL
   Open Font License 1.1", the copyright line, the Reserved Font Name if `OFL.txt` lists one, the source URL at its
   commit. Never subset, rename or modify the file (an RFN forbids shipping a modified font under that name).

## G1: the Tallow and Iron theme (game, logic; depends on E1, F)

Touches: `games/emberwake/assets/ui/{theme.json, frames/}` (new), `games/emberwake/assets/fonts/` (F's files),
`tools/gen_art.py` (group `ui_frames`), `games/emberwake/assets/README.md`,
`games/emberwake/crates/emberwake_gui/tests/theme.rs` (new), the art bible section 6. No `emberwake_gui/src` edit.

`theme.json`: `version` 1, the 3.2 Tallow and Iron column, the 3.4 layout column, `fonts.display` = the pick with its
`scale`. Frames, code-drawn at scale 2 with only art bible hues (`gen_art.py ui_frames`, byte-identical under
`--compare`); rings are counted from the outside in art px:

| Role | PNG, slice | Pixels |
| --- | --- | --- |
| `panel` | 24 x 24, 8 | ring 0-1 #4b5563, ring 2-3 #14171f, centre #0a0b10f0; rivets 4 x 4 #6b7280 with a 2 x 2 #9ca3af top-left, 2 px in from each corner |
| `tip` | 16 x 16, 4 | `panel` without rivets |
| `inset` | 16 x 16, 4 | ring 0-1: top and left #050608, bottom and right #1e232d; centre #0e1016e0 |
| `parchment` | 16 x 16, 4 | ring 0-1 #2c221e, centre #d9cbb4; `ink` = `ink` |
| `button`, `button_hot`, `button_down`, `button_off` | 16 x 16, 4 | ring 0-1 #6b7280, #d97706, #d97706, #4b5563; centre #14171f, #1e232d, #2c221e, #0e1016; ring 2 top row #1e232d (a lit edge) on the first three |
| `tab`, `tab_on` | 16 x 16, 4 | as `button`, `button_down` |
| `card`, `card_hot`, `card_on` | 16 x 16, 4 | ring 0-1 #4b5563, #d97706, #d97706; centre #14171f, #1e232d, #2c221e |
| `ornate` | 48 x 48, 16 | the A6 corner sheet's corners, edges ring 0-1 #4b5563 with a #fbbf24 line at ring 3; `gen_art.py ui_frames --ornate SHEET` composes it; without the sheet no file (falls back to `panel`) |

Until G2 the screens' own fills keep today's navy beside the restyled engine widgets (accepted between slices; the
user reviews at the end). Tests: the game theme loads clean from the game layers; every LOOK1 8.2 pair passes
(`theme::contrast`); every frame's size and sidecar are valid and its colours come from the palette list; shots of the
Kit widgets page and the main menu.

## G2: `emberwake_gui` reads the theme (game, logic; G1; a flush point after CLN-52)

Touches: `games/emberwake/crates/emberwake_gui/src/**`, `games/emberwake/assets/ui/theme.json` (`game_colors`).

1. Each use of `TEXT`, `DIM`, `GOLD`, `INFO`, `FOE`, `GOOD`, `BORDER`, `CARD`, `PANEL_HI`, `MODAL`, `FRAME`, `BG`,
   `SELECTED` becomes `th().<role>` (a mechanical edit).
2. Chrome literals (pane fills, veils, borders, for example `region.rs` `PANE`, `SHOP_DIM`) become the matching role,
   else a `game_colors` entry named by use (`map_pane`, `shop_veil`, ...) read with today's literal as the fallback.
3. Not migrated (board and art colours, code by design): terrain fallback fills, side colours, HP bars, stage lips and
   faces, FX, portrait rims, timeline block colours.
4. Then `theme.json` gets the Tallow and Iron value of each new `game_colors` entry (listed with shots in the report).

Tests: every existing `emberwake_gui` test passes unchanged under the engine default theme (no expectation edited); a
source scan test finds no `ui` colour constant in `emberwake_gui/src`; the contrast check over each `game_colors`
fill drawn behind text; shots of every player screen. Afterwards a cleanup row (engine) deletes the `ui` constants.

## ME: engine panes (engine, logic; depends on E1)

Touches: `engine/crates/aegis_ui/src/{panes.rs (new), shell.rs, specimens.rs, lib.rs}`,
`engine/crates/aegis_ui/tests/panes.rs` (new).

- `backdrop(p, img, r, veil: u8, vignette: f32)`: cover fit (scale max(r.w / w, r.h / h), centred crop, nearest);
  vignette 0-1 darkens toward the corners with `bg` up to vignette x 255 alpha; then `bg` at `veil` alpha over all.
- `menu_column(ui, id, x, y, w, h, step, items) -> Option<usize>` (the hovered item): `MenuItem { key, label, hotkey,
  action, enabled, tip }`; `button` frames, the hotkey right-aligned in `dim`, the first enabled item the primary
  (`focus_glow`); ids `<id>.<key>`; keyboard focus top-down (PORT-FOCUS).
- `tab_rail(ui, id, r, tabs, open, act)`: rows 18 high, step 20, `tab` and `tab_on`; ids `<id>.<i>`.
- `slot_card(ui, id, r, view, action, selected, corner)`: `SlotView { thumb, title, lines, portraits }`; the thumbnail
  112 x 63 at (x + 6, centred), text from x + 124, the 20 px portrait row along the bottom, corner icons 12 px top
  right; no thumbnail: an `inset` with the title's initial.
- The ask dialog: `panel` frame, the title in the display face, Yes as the primary.

Tests: cover fit crops a 16:9 image into a 4:3 rect by the right source offset; `menu_column` ids and rects at Normal
and Extra large, the hover index, a disabled item not clickable and showing its reason; `slot_card` with long names
clean in `text_audit` at the four sizes (cut with the full text on hover); `tab_rail` ids; the ask dialog's keys
(Enter yes, Esc no) unchanged. Gallery entries for each pane.

## M1: title / main menu (game, logic; ME, G1)

Touches: `games/emberwake/crates/emberwake_gui/src/menus.rs`, `games/emberwake/game.json` (`subtitle`, LOOK1 question
11.2), `games/emberwake/docs/SCREENS.md` row 00.

`backdrop` of `backgrounds/menu/title.png` (A6) when present, else `backgrounds/menu/main_menu.png`, veil 0,
vignette 0.4. Logo: the title in the display face at 28 (times the text size) in `gold` with a 1 px `bg` shadow,
right-aligned at x 620, y 40; the subtitle under it in the display face at 10 in `text`; `ui/logo_crest.png` (A6)
behind the title when present. `menu_column` at x 420, y 150, 160 x 18, step 22 with today's items, ids and hotkeys.
Continue's peek: while Continue is hovered or focused an `ornate` panel at (250, 150, 160, 64): the latest save's
location, "Day N", up to 4 party portraits at 20 px. The version bottom right in `dim` at `SMALL`. Tests: every main
menu widget id and action unchanged; the peek reads a fixture save; without the A6 files the MENU2 picture and no
crest; `text_audit` at the four sizes.

## M2: the other menus (game, logic; M1, G2)

Touches: `emberwake_gui/src/{menus,screens,hub,story,runover,gallery}.rs`, `SCREENS.md` rows 01-06, 11, 17. Layouts
of LOOK1 6 (new game cards, six `slot_card`s, the settings `tab_rail`, the game menu rail and party box, the `ornate`
frame on story vignette cards and the run-over screen). Tests: ids and actions unchanged, slot actions (load, save,
lock, delete) unchanged, `text_audit` at the four sizes, shots.

## E2: `aegis_ui::autotile` and the Tiles pane (engine, logic; depends on LOOK1-S)

Touches: `engine/crates/aegis_ui/src/{autotile.rs (new), specimens.rs, lib.rs}`, `tests/autotile.rs` (new).

- `corner_mask(is_upper, cx, cy) -> u8`: the corner at the top-left of cell (cx, cy); bit 1 cell (cx - 1, cy - 1), 2
  (cx, cy - 1), 4 (cx - 1, cy), 8 (cx, cy); the caller's closure answers off-grid cells.
- `corner_pieces(w, h, is_upper) -> Vec<(i32, i32, u8)>`: corners 0..=w by 0..=h with masks 1-14.
- `block_edges(is_block, x, y) -> BlockEdges { north, east, south, west }`: true where that neighbour is not a block.
- Gallery Tiles pane: a set picker over the layers' `tilesets/`; a 12 x 9 fixture (a pond, a wall run, a lone water
  cell, a 1-wide channel) drawn square (16 px) or tilted (32 x 24) at 1x-3x, with pieces when the set has them, else
  the mask numbers.

Tests: a fixture holding all 16 masks finds each; a lone upper cell gives masks 8, 4, 2, 1 at its corners; a 3 x 3
upper block has 15 inside (no piece); the same grid gives the same list twice; `block_edges` on a T shape.

## K: set binding (kit, logic; depends on LOOK1-S)

Touches: `kits/rpg/schema.json` (`world.locations` `tileset`, `world.world` `stage_set`), `rpgkit_data/src/lib.rs`,
`rpgkit_content/src/{art.rs, check/assets.rs}`, `rpgkit_content/tests/look1_sets.rs` (new), the `emberwake_content`
façade if it re-exports the lookups.

Fields, lookups and checks per LOOK1 5.4: `region_sets(lineage, tileset)` puts a non-empty `tileset` first;
`battle_stage_dirs(terrain, stage_set)` puts a non-empty `stage_set` first; `transition_dirs(set, upper, lower)`.
Both fields are engine-only: follow the convention of ENCOUNTER2-T's `rows` (the schema parity strips them;
`engine/docs/PORT_NOTES.md`), and shipped manifests still re-save byte for byte. Tests: the id rule; each warning and
the size error on a fixture world and layer; the lookups' order; empty fields give today's chains.

## T: `tools/tiles/` (game tools, logic; depends on LOOK1-S)

Touches: `tools/tiles/{cut.py, seam.py, masks.py, compose.py, board.py, test_tiles.py, README.md}` and
`tools/tiles/fixtures/` (new), `docs/STRUCTURE.md` row. Python 3.12 with Pillow (full path).

- `cut.py SRC --size WxH --out DST`: the pixel block size from the run lengths of equal colours (RGB step at most 12)
  along rows and columns; block centres sampled; trims a uniform border of at most one block; the result must be
  exactly W x H, else an error.
- `seam.py score PNG` (wrap step mean over inner step mean) and `seam.py fix`: columns 0 and w - 1 take, per pixel, the
  colour of column 1 or w - 2 that is nearer in RGB to the tile's mean; rows alike; then re-scored.
- `masks.py`: deterministic ragged masks for 32 x 32, 32 x 24 and 64 x 48, masks 1-14: quarters by bit, the boundary
  moved by 1D value noise (seed the mask number, amplitude 2 art px, period 6 art px) that fades to 0 within 3 art px
  of the piece border, so neighbouring pieces meet.
- `compose.py --upper U --lower L --size WxH --out DIR`: lower and upper `v0` tiled at an offset of half a cell, upper
  where the mask is set, a 1 px rim on the lower side at 60 % brightness; writes opaque `m01.png`-`m14.png`.
- `board.py SET_DIR --out C:/dev/localgen/review/look1/<set>/`: each tile 3 x 3 at 1x and 2x, the 14 pieces, the
  fixture map `fixtures/sample_map.txt` (24 x 16 of `.`, `#`, `~`) square and, for 4:3 tiles, tilted at 1x-3x with
  walls, faces and pieces; under each tile its checks (size, seam ratio, colours, mean luminance); `index.html` with
  `file:///` links.

Tests (`test_tiles.py`, unittest): cut recovers a 32 x 32 fixture scaled 7 x with a one-block border; seam score at
most 1.0 on a seamless fixture, above 1.5 on a striped one, lower after `fix`; masks 1-14 distinct, quarters follow
the bits, zero displacement at piece borders; compose output size, full opacity, a corner pixel equal to the `v0`
pixel under it.

## L1: battle stage sets (game, logic; E2, K)

Touches: `games/emberwake/crates/emberwake_gui/src/{stage.rs, battle.rs}`.

Tiles and the wall face through `battle_stage_dirs(terrain, world stage_set)` (64 x 48 into 32 x 24); after the
ground pass the pieces of each pair the set has, 32 x 24, centred on the corner points (128 + 32 cx, 82 + 24 cy),
clipped to the board; then lips (STAGE1 3.2), ground props and the grid as today. Tests: a fixture layer with a
two-colour set puts pieces on a lone water cell's 4 corners (pixel probes); a terrain missing in the set falls back to
`common_stage`; an empty `stage_set` draws today's frame (hash); battle traces unchanged.

## L2: region sets (game, logic; L1, EXPLOREVIEW1-R)

Touches: `emberwake_gui/src/{region,mapview,props}.rs` (`mapview.rs` from EXPLOREVIEW1-L). The location's
`tileset`; square view: `wall_face` over a wall's bottom 8 px when the south tile is not `#`, cap edge lines (E2
`block_edges`); tilted view: EXPLOREVIEW1 3.2's blocks with the set's `wall_face`; `floor` over `shallow_water` pieces
centred on the corners at the view's cell size. Tests: face present or absent by the south neighbour; pieces at a
pond's corners; a location without `tileset` draws today's frame (hash); journey golden unchanged.

## A1-A7: art batches (game art, planner worktree; T; A1-A3 after EXPLOREVIEW1-R; A6 after F)

Chain per set (`metadata.record.session_ref`, no attachments): the base `floor` first (n 2, the user picks), then
each other tile "in the same style and palette as the first image"; stage tiles chain from the region base of the
same terrain ("the same texture seen at a low angle, 4:3"). A1 adds the "Tile" template to `ART_PROMPT_TEMPLATES.md`:
"A single seamless ground texture tile for a dark fantasy tactical RPG, top-down, drawn as a 32 x 32 pixel tile
scaled up (visible large pixels, no anti-aliasing), edges that wrap without a seam, no border, no frame, no vignette,
no lighting gradient, limited palette of 4-6 colours from {the set's hex list}, light from the upper left", plus the
material line (LOOK1 5.2) and the size: "32 x 32", "4:3, 32 x 24" (tilted region) or "4:3, 64 x 48" (stage).

Per batch: `tools/portraits/artqueue.py request` per image with its record; the user runs the Gemini web app;
`artqueue.py ingest` into `art_src/look1/<set>/`; `tools/tiles/cut.py` (scale only), `seam.py` (score at most 1.5,
else `fix`, else a re-roll), at most 16 colours (warning) and mean luminance at most 25 %; `board.py`; the user picks
on the board. A6: the title backdrop (16:9, the MENU2 composition: cathedral left, open sky right; n 3), the crest
(the weeping cathedral eye in gilt and basalt, no text, centred on a flat dark ground; n 2), the `ornate` corner sheet
(4 gilt basalt corners and a divider on magenta, cut with `tools/sprites/props_cut.py`; n 2).

## R: place the picks (game data, asset; K, T, the picks; SUNKWARD1 for location rows)

Files: region sets `games/emberwake/assets/tilesets/<set>/{floor,wall,wall_face,shallow_water}/v<n>.png` and
`floor/over_shallow_water/m<NN>.png`; `tilesets/sunk_ward_stage/<terrain>/v<n>.png` with the 5 pairs' pieces; decals
as PROPS1 ground props `props/sunk_ward/<decal>/v0.png`; `backgrounds/menu/title.png`, `ui/logo_crest.png`, the
`ornate` frame through `gen_art.py ui_frames --ornate`. Rows through `aegis content`: world `sunk_ward` `stage_set`
`sunk_ward_stage`; the locations' `tileset` per LOOK1 5.2 once SUNKWARD1 has made them. Checks: K's asset warnings
clean for the bound sets; `tools/fc.sh content` validation clean.

## Docs delta (by slice)

- Art bible section 6: the theme roles and frames (G1); section 7.3 follows the Gemini queue decisions; section 2's
  region map row gets the set binding (K).
- `assets/README.md` (engine, E1; game, G1 and R): `ui/theme.json`, `ui/frames/`, `fonts/` with the licence lines,
  `wall_face`, `over_<lower>/m<NN>.png`. `engine/README.md`: theme (E1), panes (ME), autotile (E2).
- `engine/docs/authoring/`: a game-agnostic theme and tiles note (E1, E2; "promote generic methods").
- `docs/STRUCTURE.md`: `tools/tiles/` (T), `theme.rs`, `panes.rs`, `autotile.rs` (E1, ME, E2).
