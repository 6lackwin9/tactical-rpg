# STAGE1: battle stage look (tiles, props and units in one view)

Status: approved (user 2026-10-23): option B in full, the oblique board with 32 x 24 cells and the zone backdrops in
the same task; units 40 px; trees, stones, the wall face and the backdrops drawn by code now (an image-API sheet only
if asked later); a Gameplay setting "Battle view: Stage / Flat" until the user's review (default Stage; Flat removed
after the review); intent arrows on the ground under the units; the faint grid always on; a unit behind a tree canopy
gets its outline drawn through the canopy (no fade). Why B: the tilted ground matches the angle of the side-view
sprites (the playtest complaint). A and C are not adopted (section 2). Was: proposed (spec STAGE1-S, 2026-10-23)
Depends on: PROPS1-E (done: `DepthList`, `Art::varied`, `Art::size_of`, mirrored images); FACE1 (same file
`battle.rs`). Built on by: DIR4-G (facing in the stage), FX1 (effects start and land at the stage's chest points);
DIR4-R2 and R3 may roll out (B keeps DIR4's front and back views, DIR4 section 9).
Lane: logic (engine STAGE1-E, game STAGE1-L), asset (STAGE1-T art by code)
Touches: `engine/crates/aegis_ui/src/{painter,canvas,art}.rs`; `games/emberwake/crates/emberwake_gui/src/{stage,battle,
lib,screens,menus}.rs` (`stage.rs` new); `games/emberwake/crates/emberwake_content/src/art.rs`;
`games/emberwake/tuning.json`; `tools/gen_art.py`; `engine/assets/tilesets/common_stage/`;
`kits/rpg/assets/props/common/{tree,stones}/`; `games/emberwake/assets/backgrounds/battle/`

Sources: playtest note 20261022-204440 (note 1 and `note_1.jpg`); `battle.rs` as on develop fee46c3 (TILE 32, grid
12 x 9 at (128, 30), `token_px` 28, filled side-coloured foot ellipse, arrows centre to centre over the units); the
zone rows in `emberwake_battle::grid::zone_rows`; PROPS1 sections 3, 4 and 6.1; DIR4 sections 1, 8 and 9; preferences
`ui.md` (COMBAT-UI3 layout A) and `assets-world.md` (Style A, 16-colour palette, layers, image API rule).

## Goal

The user's note: "The token looks weird now since the sprites are standing while the ground tiles are flat making it
feel disconnected from the background, I guess the ground/tiles needs to feel isometric, open to suggestions." The
units are upright side-view figures since SPRITE1-U, but the board is a map seen from straight above: trees are map
symbols in the tile, units float on a coloured disc, nothing stands or casts a shadow. STAGE1 tilts the board: rows
are drawn 24 px deep under 32 px wide cells, so the ground recedes at about the angle the sprites are drawn from;
trees and walls stand up and are drawn in order with the units by their feet; units cast contact shadows; a zone
backdrop closes the far edge and a front face the near edge. The rules, the grid, the plan and every battle number stay
as they are: this is a view change.

## 1. The mockups

Drawn from the real assets by `stage1_render.py` (Python + Pillow, kept beside the images) in the game's logical
pixels (640 x 360) scaled 3x by nearest neighbour, inside the HUD as built (COMBAT-UI3 layout A). The scene is
`goblin_ambush` on `forest_clearing`, round 2 planning after the B4 ambush: Warrior (selected, a planned move to
(6, 4)) and Ranger on the road; Enemy Brawler, Fleeing Scout, Goblin Berserker (a sweep on (6, 5), (6, 4), (7, 4)),
Skirmisher, Archer and Shaman; the hover cell (4, 2); the exit (11, 4). Trees and the treeline are code-drawn
stand-ins; B's ground is today's square tiles squashed (STAGE1-T redraws them, 7.1).

| File | Shows |
| --- | --- |
| `stage1_B_frame.png`, `stage1_B_zoom.png` | B as approved: 32 x 24 cells, the forest backdrop, the front face, ground arrows, 40 px units |
| `stage1_overview.png` | today, A, B and C side by side (B with the approved ground arrows) |
| `stage1_0_today_frame.png`, `stage1_0_today_zoom.png` | today re-rendered: the Flat view of the setting (section 4.1) |
| `stage1_scale.png` | token sizes 28, 36, 40 and 48 (on square ground) |
| `stage1_A_frame.png`, `stage1_A_zoom.png`, `stage1_A_behind.png`, `stage1_A_walls.png` | A (not adopted); `A_behind` and `A_walls` also show the outline through a canopy and the wall blocks that B keeps |
| `stage1_C_frame.png`, `stage1_C_zoom.png` | C (not adopted): isometric 2:1 diamonds with today's side sprites |

## 2. Options

Question: how should the battle board be seen so upright sprites belong on it? The user picked B (2026-10-23).

| Option | Look | Status and reason (for HISTORY) |
| --- | --- | --- |
| A. 3/4 view on the same square grid | Square 32 px cells seen from above, upright props and units feet-sorted, contact shadows | Not adopted (user 2026-10-23): the ground stays seen from straight above, so it does not match the angle the sprites are drawn from, which was the playtest complaint; B keeps all of A's pieces (props, feet sort, shadows, blocks) and tilts the ground |
| B. Oblique board, cells 32 x 24 | Rows 24 px deep, a zone backdrop above the far edge, a front face under the near edge; A's upright props and feet sort | Adopted (user 2026-10-23, in full: backdrops in the same task) |
| C. True isometric, 2:1 diamonds | Diamond cells 36 x 18, the board 378 x 189 px, grid axes diagonal on screen | Not adopted (user 2026-10-23): it needs every body redrawn in two 3/4 diagonal views (DIR4 option D, already not adopted on 2026-10-23 because no isometric camera was picked; that reason stands), diamond tiles for every terrain, diamond hit tests and every overlay redone; the board shrinks; party left vs foes right (GRID12) turns into a diagonal |

## 3. Stage geometry (STAGE1-L, `stage.rs`, pure functions with tests)

All in logical px. The grid panel stays where it is: (126, 28, 388, 292), interior (128, 30, 384, 288); every
pass below is clipped to the interior (`Painter::clip`).

| Piece | Rect or point | Notes |
| --- | --- | --- |
| Backdrop | (128, 30, 384, 52) | the zone image (7.3), or the fallback (3.4) |
| Board | origin (128, 82), 12 x 9 cells, 384 x 216 | `CELL_W` 32, `CELL_H` 24, `BOARD_Y` 82 (code constants) |
| Cell (x, y) | (128 + 32x, 82 + 24y, 32, 24) | `stage::cell_rect` |
| Front face | (128, 298, 384, 12) | `FACE_H` 12 (3.4) |
| Drop shadow | (128, 310, 384, 8) | rows of rgba(0, 0, 0, a), a = 120 - 15k for row k |
| Foot point of (x, y) | (128 + 32x + 16, 82 + 24y + 24 - `foot_inset_px`) | 5 px above the cell's bottom edge by default |
| Chest point | foot point raised `token_px` / 3 | floats start 14 px above it; FX1's anchor |
| Token rect | `token_px` square, bottom edge at foot y + 1, centred on foot x | the frame from `token_frame` (FACE1, DIR4-G) |

Checks on the numbers: a row-0 unit at 40 px has its foot at y 101 and its top at y 62, inside the backdrop band
(no clipping, nothing over the planning strip, which ends at y 26); its chip at y 61; a row-8 unit's foot is at y 293
and its HP bar ends at y 298, on the board. While walking, the slide offset is (dx x 32, dy x 24) x the remaining
fraction (today's rule with the row height).

### 3.1 Terrain stage table (code, over `emberwake_battle::grid::TERRAIN_IDS`; no data change)

| Terrain | Stage kind | Depth (px down) | Prop |
| --- | --- | --- | --- |
| floor, grass, stone, sand, snow, bridge, burning_ground, ice | ground | 0 | none |
| dirt, toxic_bog | sunken | 2 | none |
| mud | sunken | 1 | none |
| shallow_water | sunken | 3 | none |
| deep_water, lava | sunken | 5 | none |
| rubble | ground | 0 | ground prop `stones` |
| tree | standing | 0 | standing prop `tree` |
| wall | block | lift 12 | none (cap and face, 3.3) |

An unknown terrain id is ground, depth 0. Walking, cover and line of sight stay with the terrain catalog and `grid.rs`.

### 3.2 Ground

1. Every cell draws its stage tile: `Art::varied(battle_stage_dirs(id), None, x, y, 32, 24)` over
   `emberwake_content::art::BATTLE_STAGE_SETS` = `["common_stage"]`; missing, the square `common` tile drawn into the
   32 x 24 cell (squashed, as in the mockup). A standing cell draws the ground under its prop: the terrain most of its
   four neighbours have among ground and sunken cells (ties: first in the order north, east, south, west); none,
   `grass`.
2. Lips: a sunken cell whose north neighbour exists and is shallower (block and standing count as depth 0) draws a
   band over its top `d - d_north + 1` px: the top row rgb(118, 116, 62), the rest rgb(66, 54, 30), then one row
   rgba(60, 40, 26, 110). A sunken cell whose south neighbour is shallower draws its bottom row rgba(48, 34, 22, 120).
3. Ground props (`stones` on rubble), placed and varied as in 3.5.
4. The grid, always on: 1 px lines on every cell edge, black at `grid_alpha`.

### 3.3 Blocks (walls)

A block cell's cap is its stage tile drawn at (cell x, cell y - 12), 32 x 24; when its south neighbour is not a block
(or it is row 8) its face covers its own bottom 12 px: `common_stage/wall_face/v<n>.png` (7.1) varied by position,
else the cap's top 12 rows at 75 % brightness; a 1 px rgb(24, 24, 30) line on the face's top edge and a 3 px
rgba(0, 0, 0, 70) shade on the cell south. The cap's top edge gets a 1 px rgb(170, 172, 186) highlight when the cell
north is not a block, a side edge a 1 px rgb(30, 30, 38) line when that neighbour is not a block. A row-0 block's cap
reaches 12 px into the backdrop band. Sort: foot y = the cell's bottom edge, group 0.

### 3.4 Backdrop and front face

- Backdrop: `backgrounds/battle/<zone>.png` (384 x 52, drawn 1:1 at (128, 30); the zone is `EncounterDef.zone`)
  through the art layers (game, kit, engine). Missing or no zone: a vertical gradient from rgb(10, 14, 24) at the top
  to the zone's most common row-0 ground terrain colour (`terrain_color`) at 40 % at the bottom.
- Front face, per column (32 x 12 under row 8): its top row is the row-8 ground terrain's colour lightened 25 %; the
  body is the `wall_face` art when row 8 is a block, rgb(30, 54, 110) x 0.7 for water, rgb(150, 50, 20) with
  rgb(255, 160, 60) flecks for lava, else earth rgb(70, 50, 32) with 2 x 1 rgb(52, 36, 24) flecks every 7 px
  (y offset (x x 7) mod 6). Then the drop shadow (section 3 table).

### 3.5 Standing props

`props/<set>/<art>/v<n>.png` over `BATTLE_SETS` (`["common"]`; new `emberwake_content::art::battle_prop_dirs(art)`),
drawn at the PNG's size with its bottom edge on the cell's bottom edge, centred (PROPS1 4.3 with the battle's 32 px
cell width as the art scale), variant by the PROPS1 3.2 position pick. Shadow: an ellipse at (cell centre x + 2,
cell bottom - 5), radii 10 x 2.7, rgba(10, 16, 6, 95). Sort: foot y = the cell's bottom edge, group 0, x = cell x.
Missing art: the cell draws its `common` tile (today's flat tree) squashed into the cell, nothing stands.

### 3.6 Units

- Shadow pass: an ellipse at (foot x + 1, foot y), radii 0.2 x `token_px` by 2.4, rgba(8, 10, 6, `shadow_alpha`);
  then a 1 px ring (`Painter::ellipse_ring`) at the foot point, radii 0.25 x `token_px` by 3.2, in the side colour
  (ally rgb(110, 170, 255), foe rgb(240, 100, 92)) at `foot_ring_alpha`.
- The sprite in the feet pass (foot y = the foot point's y with the slide, group 1, x = foot x), then its selection
  outline (gold) or hover outline (white at `hover_outline_alpha`) as today.
- Top pass: the HP bar (foot x - 12, foot y + 3, 24, 2); a fallen unit's "x" at the foot point.

### 3.7 Passes, back to front

1. Backdrop (3.4).
2. Ground tiles, lips, ground props, the grid (3.2).
3. Ground markers, all cell plates in the 32 x 24 cell: the hover cell (rgba(255, 255, 255, 30) fill,
   rgba(255, 255, 255, 160) 1 px edge); area templates (rgba(240, 100, 92, 70) fill, rgba(240, 100, 92, 230) edge,
   inset 1 px); the planned path (a 2 px gold line between foot points raised 3 px) and the gold ghost cell (inset
   3 px across, 2 px down); the exit plate (rgba(70, 200, 90, 90) fill, rgba(200, 255, 210, 200) edge, inset 3 x 2);
   any later move-range highlight uses the same plate (inset 1 px).
4. Ground arrows: the intent tells and the planning attack arrows (`intent_arrows`), foot point to foot point (a cell
   target: that cell's foot point), shortened in flat space (y / 0.75) by 10 px at the start and 12 px at the end so
   the head stops at the target's foot ring, 1.5 px, rgba(240, 100, 92, 210), dashed 3 px for move and flee, head 4.
   The units drawn later cover parts of the shafts; the heads stay clear.
5. Shadows of props and units, the foot rings.
6. The front face and its drop shadow.
7. The feet pass: one `DepthList` of blocks, standing props and units.
8. Outlines through: (a) a unit whose body rect (3.8) meets the drawn rect of a block or standing prop sorted after
   it gets `Art::body_outline` of its current frame drawn again, in its side colour at alpha 230 (user: through the
   canopy, no fade); (b) the selected unit and the hovered unit (hovered on the board or through its party or hostile
   card) get theirs drawn again when anything sorted after them meets their body rect, gold or white.
9. Top: HP bars, intent chips (10 x 10 at (foot x - 15, foot y - 0.95 x `token_px` - 2)), the EXIT sign (a 22 x 9
   green chip with "EXIT" centred 2 px above the exit cell's centre), floating numbers; the outcome banner and the
   log panel as today.

### 3.8 Hit testing

- Cell widgets stay `tile.<x>.<y>` with the actions of today; rect = `cell_rect` (32 x 24), pushed first. A point
  maps to (floor((mx - 128) / 32), floor((my - 82) / 24)) inside the board; the backdrop and the front face hold no
  cell widget.
- Unit widgets stay `unit.<i>`, pushed in feet-pass order (front-most last; `aegis_ui::hit` takes the topmost); rect =
  the union of the cell rect and the body rect: `Art::body_bounds` of the body's `idle` frame 0 at `token_px`, placed
  in the token rect (cached per body, palette, size and mirror; animation frames do not move it). A row-0 unit's head
  in the backdrop band is clickable through it.
- Props and blocks have no widget (PROPS1 5): a click on their art is the click on the cell or unit under it.
- The agent link, the playtest trail (`ctl.path`) and the UI Gallery's `grid` panel keep their names; cell rects are
  32 x 24 in Stage.

### 3.9 Crowding: 40 px units on 24 px rows

The figure stands about 36 px tall, so a unit's head rises about 17 px into the cell north of it and covers the lower
12 px of a unit standing there (its legs). Standard deployment fills the front columns from row 4 outward, so stacked
neighbours are common. Bodies are about 17 px wide, so only units in the same column overlap. What keeps it readable:

1. Feet sort: the unit in front is always whole; the one behind loses only its legs.
2. HP bars, foot rings and chips never hide: rings and shadows are on the ground, bars and chips in the top pass.
3. Pass 8b: the selected and the hovered unit show their full outline through whatever covers them; hovering a
   party or hostile card hovers the unit.
4. Clicks: the front unit wins where rects overlap; the unit behind is clickable on its head and upper body (above
   the front unit's head), on the free parts of its cell, and through its card.
5. Tuning: `token_px` 36 lowers the overlap by 4 px if the review asks for it; no other change is needed.

## 4. Schema contract

Schema-first: no content schema change (the stage table is code, 3.1). New file paths: section 7.

### 4.1 Setting (Gameplay tab, `menus.rs`; default in `screens.rs` beside `map_names`)

| Key | Values | Default | Label and tip |
| --- | --- | --- | --- |
| `battle_view` | `stage`, `flat` | `stage` | "Battle view", options "Stage" and "Flat"; tip: "Stage: a tilted board with standing trees and walls. Flat: the board seen from above, as before." |

An unknown value reads as `stage`. The setting applies at the next frame (no restart). Flat is today's draw exactly:
32 x 32 cells at (128, 30), the `common` tiles, tree tiles flat, tokens `flat_token_px`, the filled foot ellipse at
`foot_alpha`, arrows centre to centre over the units, today's cell widgets. Both views use the same `Tokens`, actions,
widget names and HUD. After the user's review a cleanup row (STAGE1-C) removes Flat, the key and `flat_token_px`.

### 4.2 Tuning (`games/emberwake/tuning.json`, live reload)

| Key | Type | Default | Allowed | Meaning |
| --- | --- | --- | --- | --- |
| `token_px` | int | 40 (was 28) | 24-64 | Stage unit sprite square |
| `flat_token_px` | int | 28 | 24-64 | Flat unit sprite square (today's value) |
| `foot_inset_px` | int | 5 | 2-10 | Stage foot point above the cell's bottom edge |
| `shadow_alpha` | int | 120 | 0-255 | Stage unit contact shadow |
| `foot_ring_alpha` | int | 200 | 0-255 | Stage side-colour foot ring |
| `foot_alpha` | int | 110 (unchanged) | 0-255 | Flat filled foot ellipse |
| `grid_alpha` | int | 34 | 1-255 | Stage grid lines (always on) |

Values outside a range are clamped where they are read.

### 4.3 Engine pieces (STAGE1-E, `aegis_ui`, game-agnostic)

- `Painter::ellipse_ring(cx, cy, rx, ry, col)`: a 1 px ellipse outline, no fill.
- `Image::opaque_bounds(min_alpha) -> Option<Rect>`: the bounding box of pixels at or above `min_alpha`.
- `Art::body_bounds(key, palette, size, flip) -> Option<Rect>`: `opaque_bounds(OUTLINE_ALPHA)` of the `idle` frame 0
  fitted as `body_clip`, mirrored with `flip`, cached.

### 4.4 Art lookups (STAGE1-L, `emberwake_content::art`)

`BATTLE_STAGE_SETS` = `["common_stage"]`; `battle_stage_dirs(terrain) -> Vec<String>` (`tilesets/common_stage/<id>`);
`battle_prop_dirs(art) -> Vec<String>` (`props/common/<art>`); `battle_backdrop(zone) -> String`
(`backgrounds/battle/<zone>.png`). Unit tests as the other chains.

## 5. Acceptance checks (STAGE1-E engine tests, STAGE1-L `stage.rs` and `battle.rs` tests)

1. Geometry: `cell_rect(0, 0)` = (128, 82, 32, 24), `cell_rect(11, 8)` = (480, 274, 32, 24); foot of (5, 4) =
   (304, 197) at `foot_inset_px` 5; the token rect of a row-0 unit at 40 px starts at y 62 (inside (128, 30, 384,
   288)); `cell_at` maps every cell's centre back to the cell and (300, 50) and (300, 305) to none.
2. Ground under a prop: `forest_clearing` (0, 0) and (10, 2) give `grass`; `frozen_pass` (5, 0) gives `snow`; a tree
   with only block and standing neighbours gives `grass`.
3. Lips: `forest_clearing` (0, 4) (dirt, north grass) a 3 px band; (4, 3) (dirt, north dirt) none; (4, 2) (dirt,
   north grass) a band; `lava_cavern` (1, 1) (lava, north a wall) a 6 px band.
4. Order: units at (10, 1) and (10, 3) and the tree at (10, 2) sort unit (10, 1), tree, unit (10, 3); a block and a
   unit with equal foot y: the block first; two units in one column: the southern one last.
5. Outlines through: in that scene the unit at (10, 1) is outlined (side colour) and the unit at (10, 3) is not; a
   selected hero north of another unit is outlined gold; an unselected, unhovered unit behind a unit is not.
6. Hit: the Warrior's helmet pixel (above his cell) hits `unit.0`; the top-left 2 x 2 px of a free cell north of a
   unit hits that cell; where two unit rects overlap the southern unit wins; a point in the backdrop or the face hits
   no cell.
7. Ground arrows: an arrow to a unit ends 12 flat px short of its foot point; a move or flee tell is dashed; arrows
   draw before the feet pass (a draw-order list from `stage::passes` in the test).
8. Blocks: a wall with a wall south draws no face; a row-8 wall draws one; a missing `wall_face` uses the darkened
   fallback; a missing stage tile uses the squashed `common` tile; a missing tree prop draws the flat tree tile.
9. Backdrop: `forest_clearing` loads `backgrounds/battle/forest_clearing.png`; an unknown zone draws the gradient.
10. Setting: `battle_view` `flat` gives today's cell rects (32 x 32 at (128, 30)) and token rects at `flat_token_px`;
    `stage` the rects of check 1; an unknown value acts as `stage`; the default is `stage`.
11. View only: every battle trace, sim, journey, save and checks parity run unchanged (the gate); the existing
    `battle.rs` tests (`tokens_follow_the_battle_step_by_step`, the facing tests) pass unchanged.
12. Engine: `ellipse_ring` sets only ring pixels; `opaque_bounds` of an empty image is None and of a one-pixel image
    that pixel; `body_bounds` with `flip` is the mirror of the unflipped bounds.

## 6. Parity impact

None expected: rules, plan, AI, session, saves and content are untouched and no parity suite draws the GUI, so no
recorded trace or expectation changes and no parity check retires. The playtest trail's pixel `pos` of older
recordings points at the flat picture; it is a note, replay runs on requests.

## 7. Assets (STAGE1-T, by code in `tools/gen_art.py`; no image API)

Style A (preferences `assets-world.md`): 16 colours per image, dark outlines on props, light from the upper left;
colour and pixel fixes stay code. The flat `common` tiles are not touched (Flat draws them until it is removed).

### 7.1 Stage tiles (a redraw at 4:3, not a squash)

| Path | Files | PNG px | Content |
| --- | --- | --- | --- |
| `engine/assets/tilesets/common_stage/<terrain>/v0-v3.png` for the 16 terrains other than `tree` | 64 | 128 x 96 (drawn at 32 x 24) | each terrain's texture redrawn for the tilted ground: round features (stones, dirt specks, bog bubbles, lava cracks) as 4:3 ellipses, strata and ripples horizontal, upright features (grass blades, reeds, ice glints) at full height; no frame, no vignette; edges that run on into the neighbour; the `common` palette per terrain |
| `engine/assets/tilesets/common_stage/wall_face/v0-v3.png` | 4 | 128 x 48 (drawn at 32 x 12) | brick courses of the wall texture at 75 % brightness, mortar every 6 drawn px, joints offset per variant |

### 7.2 Props

| Path | Files | PNG px | Content |
| --- | --- | --- | --- |
| `kits/rpg/assets/props/common/tree/v0-v3.png` | 4 | 40 x 52 | v0-v2 leafy trees (canopy 28-34 px wide, top 0-4 px from the PNG top, the olive greens of `grass`), v3 the bare tree of today's tile; trunk 4 px wide centred at x 20, its base 6 px above the PNG bottom; outline rgb(22, 26, 16) |
| `kits/rpg/assets/props/common/stones/v0-v1.png` | 2 | 32 x 24 | 2-3 small grey stones in the lower half (largest 16 x 9), outline rgb(24, 24, 30), transparent elsewhere |

### 7.3 Zone backdrops (`games/emberwake/assets/backgrounds/battle/<zone>.png`, 384 x 52, opaque)

Every backdrop: its bottom 4 px are the far ground, the zone's main ground colour darkened 40 %; above it the scenery
of the table, darkest at the top (rgb(10, 14, 24) top row) so it meets the panel; the 12 columns of 32 px line up
with the board, so an opening in row 0 (a gap in a wall, a path) continues into the backdrop.

| Zone | Scenery |
| --- | --- |
| `stone_arena` | a stone arena wall: two courses of grey blocks in the lower 26 px, a darker parapet line, a dim torch every 96 px |
| `forest_clearing` | two treeline silhouettes from the tree shapes (far rgb(16, 28, 16), near rgb(24, 40, 22)), a gap over columns 3-4 and 7-8 where row 0 is open |
| `river_crossing` | the forest treeline, broken over columns 8-10 where the river runs on into the distance, narrowing upward between dark banks |
| `ruined_courtyard` | a broken courtyard wall (block caps and faces as 3.3) with gaps over columns 3 and 7, rubble at its foot |
| `desert_outpost` | dunes in two sand tones, a mesa silhouette, a dusk gradient |
| `frozen_pass` | snowy ridges and pine silhouettes in cold blues, the pass opening over columns 3-8 |
| `lava_cavern` | the cave wall in dark basalt with glowing orange cracks, stalactite tips along the top |
| `swamp_bog` | dead trees and reeds in silhouette, a pale mist band over the bottom 10 px |

## 8. The region map (unchanged)

The region map stays as PROPS1 and PLAYUI1-R build it: square 16 px tiles seen from above, upright props and sprites
feet-sorted on them, the party outline behind a prop. The battle and the region map then share the sprites, the feet
sort and the outline-through rule, and differ in the ground: square and top-down on the map, tilted 3:4 with a
backdrop and a front face in battle. Tilting the region map later (16 x 12 cells) would touch `region.rs` (cell
mapping, click to walk, the glide, `camera_offset`, labels), PROPS1's placements and, above all, the building art:
PROPS1-A2's sheets draw building bases for square footprints, which would look too deep on 16 x 12 cells. So if the
region map is ever to tilt, decide it before PROPS1-A2 is generated (question 1); the recommendation is to keep it
square (a map read from above, a battle seen at an angle is a common split) and look again after the review.

## 9. Costs and risks

| Part | Size | Builder sessions |
| --- | --- | --- |
| STAGE1-E (ring, bounds) | S | about 0.5 |
| STAGE1-L (`stage.rs`, passes, hit rects, setting, both views, tuning, art lookups, tests) | M+ | about 3 |
| STAGE1-T (64 stage tiles, 4 wall faces, 6 props, 8 backdrops, by code) | M | about 2 |

Risks: crowding on 24 px rows (3.9; the review judges it, `token_px` 36 is the lever); the squashed fallback tiles
while STAGE1-T is out (it lands before the review); `battle.rs` is shared with FACE1 and DIR4-G (order the rows,
section 12); two views to keep working until STAGE1-C (both are tested, check 10).

## 10. Out of scope

- Any rule: height, line of sight, cover by props; the grid size; deployment.
- Camera zoom, pan or rotation; the HUD panels and their sizes.
- The region map (section 8); DIR4 views, FACE1 and DIR4-G facing; FX1 effects (they start at the chest point).
- Ashlands battle tile sets (battles use `common` and `common_stage`); image-API art (STAGE1-A2, asked first).

## 11. Open questions (recommended answer first)

1. Region map: stay square (recommended; PROPS1-A2 draws building bases for square footprints), or plan a tilt and
   hold PROPS1-A2's building sheets until it is specified?
2. After the review, does Flat go entirely (recommended, as decided), or stay as an accessibility option?

## 12. Rows wanted (status `spec`; the planner writes them)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| STAGE1-E | Stage pieces: `Painter::ellipse_ring`, `Image::opaque_bounds`, `Art::body_bounds`, tests (4.3, check 12) | engine | logic | none | `engine/crates/aegis_ui/src/{painter,canvas,art}.rs` |
| STAGE1-L | Battle stage B: `stage.rs` (geometry, stage table, ground under props, lips, blocks, props, backdrop and face, passes, ground arrows, outlines through, hit rects), `battle.rs` draws Stage or Flat by the `battle_view` setting, tuning keys, art lookups, tests (sections 3-5) | game | logic | STAGE1-E, FACE1 (same file) | `games/emberwake/crates/emberwake_gui/src/{stage,battle,lib,screens,menus}.rs`, `games/emberwake/crates/emberwake_content/src/art.rs`, `games/emberwake/tuning.json` |
| STAGE1-T | Stage art by code: 64 stage tiles at 4:3, wall faces, tree and stones props, 8 zone backdrops (section 7) | engine + kit + game | asset | none (new folders; drawn once STAGE1-L reads them; before the review) | `tools/gen_art.py`, `engine/assets/tilesets/common_stage/`, `kits/rpg/assets/props/common/{tree,stones}/`, `games/emberwake/assets/backgrounds/battle/` |
| STAGE1-C | Remove the Flat view after the user's review: the `battle_view` setting, `flat_token_px`, the flat draw path and its tests | game | logic | STAGE1-L, STAGE1-T, user review | `games/emberwake/crates/emberwake_gui/src/{stage,battle,screens,menus}.rs`, `games/emberwake/tuning.json` (status: waits for the review) |
| STAGE1-A2 | Image-API sheet for battle trees and rocks (asked first) | kit | asset | STAGE1-T, user ask | `kits/rpg/assets/props/common/` (status `idea`) |

Rows that build on it: DIR4-G after STAGE1-L (or rebased on it; both edit `battle.rs`); FX1's spec takes the chest
and foot points of section 3 and draws in pass 9 (effects) or pass 4 (ground marks).
