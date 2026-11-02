# EXPLOREVIEW1: region map view test (square top-down or a tilted Stage-style board)

Status: spec (2026-11-02; the row was approved by the user 2026-10-27 with "needs a spec", decision "Battle and
exploration view" in `games/emberwake/docs/decisions/lore.md`)
Depends on: STAGE1-L (done: the battle stage and its Flat debug view), PROPS1-L2 (done: map props), MAPNAV1-U (on
`staging`: edge arrows, legend pan, click-to-walk refusal, the exploring leader), WALK2 (same `region.rs`; runs first)
Built on by: LOOK1-L2 and LOOK1-A1-A3 (the region tile aspect, LOOK1 Q4), PROPS1-A2 (the building sheets' footprints)
Lane: design (S), logic (L), review with the user (R, planner)
Touches (L): `games/emberwake/crates/emberwake_gui/src/{mapview.rs (new), region.rs, props.rs, runner.rs, screens.rs,
main.rs, lib.rs}`; `games/emberwake/docs/SCREENS.md` row 07, `games/emberwake/docs/GAME_BRIEF.md` (region map lines)

History: the decision reopened "Visual target"'s "the region map stays square top-down"; STAGE1 section 8 recommended
keeping it square and looking again after a review, and this test is that look. STAGE1 option C (isometric diamonds,
not adopted 2026-10-23) holds: the tilt is option B's (shorter rows, no diamonds, no diagonal views). PLAYUI1 2.5 B and
C (not adopted) hold: buildings stay props, no painted map. Presentation only: no rule, save or journey changes.

## Goal

The shipped game has one view style for battle and exploration. Battle ships the tilted Stage (32 x 24 cells) with
Flat as a debug view; the region map is square top-down. This task adds a debug setting that draws the region map as
a tilted board (16 x 12 cells, walls as blocks with faces, everything standing and feet-sorted as in battle), so the
user can play both and pick the one style that ships. The pick fixes the region tile aspect for LOOK1 and the
building footprint depth for PROPS1-A2.

## 1. What the review picks

| Pair | Region map | Battle |
| --- | --- | --- |
| Tilted | tilted (this task) | Stage (shipped) |
| Square | square (today) | Flat (STAGE1-C debug view) |

A split (square map, Stage battle) was STAGE1 8's recommendation; the decision rules it out unless the user reopens
"one view style" in the review (question R1 (c)).

## 2. The setting (schema contract)

| Key | Values | Default | Where |
| --- | --- | --- | --- |
| `region_view` | `square`, `tilted` (labels "Square", "Tilted") | `square` | `screens.rs` `default_settings()` beside `battle_view` |

- Scene Runner, Developer panel: a "Region view" select under "Battle view" (`runner.rs`, at `right.y + 111`), id
  `runner.region_view`, tip "Region view (debug)": "Square: the map seen from above. Tilted: shorter rows with standing
  walls, like the battle stage."; saved to the settings like `battle_view`.
- `main.rs`: outside the Scene Runner the game forces `square` (as STAGE1-C forces `stage`), so the shipped game shows
  one view until the pick.
- Any other or missing value is `square`. The map reads the setting each frame (a switch shows on the next frame).
- `mapview::VIEWS: [(&str, &str); 2]`, `View::of_setting`, like `stage::VIEWS`.
- Agent link: the region map's `state.view` (`square` or `tilted`); every other name stays.

## 3. The tilted view

### 3.1 Geometry (`mapview.rs`, pure functions with tests)

`MapGeom { view, ox, oy }` with cell width `CELL_W` 16 and height 16 (square) or 12 (tilted); `region.rs` and
`props.rs` take every map position from it (map click, markers, the party, talk tiles, names, edge-arrow targets,
`speaker_tile`, the tests' helpers). In the square view every result equals today's formula.

| Piece | Square (today) | Tilted |
| --- | --- | --- |
| Cell (x, y) | (ox + 16x, oy + 16y, 16, 16) | (ox + 16x, oy + 12y, 16, 12) |
| Cell under (mx, my) | (floor((mx - ox) / 16), floor((my - oy) / 16)) | (floor((mx - ox) / 16), floor((my - oy) / 12)) |
| Camera | the focus area's centre on the midpoint of the two focus cells' centres (party; party and speaker; the pan) | the same with 16 x 12 cells |
| Sprite | 24 x 24 (`SPRITE`), bottom-centred on the cell's bottom edge | the same: the head rises 12 px into the row north |
| Foot shadow | ellipse at (cell centre x, cell bottom - 2), radii 0.4 x and 0.14 x the cell's width and height | the same (6.4 x 1.7) |
| Rows drawn | those meeting the map area | one more row below the area (its lifted cap reaches into it) |
| Tilted constants | - | `BLOCK_LIFT` 6, `FACE_H` 6, `LIP_DEPTH` 2 |

### 3.2 Ground, water and walls (STAGE1 3.2 and 3.3 at half size)

1. Ground: every non-wall cell draws its region tile (the location lineage's sets, then `common`, `Art::varied` as
   today) into its 16 x 12 cell: the 32 x 32 art squashed by nearest sampling, a placeholder until LOOK1 makes 4:3
   region art for the winner (question 1).
2. Water (`~`), sunken by `LIP_DEPTH`: when the north cell is not `~`, a band over its top 3 px (the top row
   rgb(118, 116, 62), the rest rgb(66, 54, 30), then one row rgba(60, 40, 26, 110)); when the south cell is not `~`,
   its bottom row rgba(48, 34, 22, 120).
3. Walls (`#`, also the walls drawn past the map's edge) are blocks: the cap is the wall tile drawn at (cell x, cell
   y - 6, 16, 12); when the south cell is not `#`, the face covers the cell's bottom 6 px: `wall_face` from the region
   sets, then `common_stage` (`Art::varied`, 16 x 6), else the cap's top 6 rows at 75 % brightness; a 1 px
   rgb(24, 24, 30) line on the face's top edge and a 2 px rgba(0, 0, 0, 70) shade on the cell south. The cap's top
   edge gets a 1 px rgb(170, 172, 186) highlight when the north cell is not `#`; a side edge a 1 px rgb(30, 30, 38)
   line when that neighbour is not `#`.

### 3.3 Passes, back to front (tilted)

1. Ground tiles and water lips (3.2 1-2).
2. Ground props (PROPS1 4.2).
3. The sorted pass, one `DepthList`: wall blocks whose drawn rect meets the area (foot y the cell's bottom edge, group
   0), standing props (the footprint's bottom edge, group 0), markers and the party (foot y, group 1). `props::sorted`
   gains a block item.
   As built: at equal feet block, prop, marker, party (decision "EXPLOREVIEW1-L click areas and sort").
4. The party's outline (PROPS1 4.5) when a block or standing prop sorted after it meets its sprite rect.
5. Service badges, names, the `party` widget, edge arrows, the legend, the ticker, the talk box, shop, board and lore
   page, the HUD bar: as today.

### 3.4 Props

`props::layout` takes the cell size (w, h) in place of `ts`: a footprint is (ox + 16x, oy + 12y, 16w, 12h); its art
is drawn at the PNG size x 16 / 32 (no squash), bottom-centred on the footprint's bottom edge, so art drawn for square
footprints stands about 4 px deeper per footprint row than the tilted footprint (the review judges it; PROPS1-A2 then
draws for the picked view). A standing prop without art (PROPS1 4.6) is a block: its fill raised 6 px over the
footprint, a 6 px south face in `BOX_FACE`, the 1 px edge. Placements and PROPS1's checks (P8, P9) are unchanged.

### 3.5 Markers, people and the leader

- People and the leader: their sprites as in 3.1; clips, four-way facing, idle offsets and turning to the speaker as
  today.
- Icon markers (chest, exit, lore, fight): the icon 16 x 16 standing on the cell, (cell x, cell bottom - 16, 16, 16);
  without art the drawn stand-in in the same rect.
- Widgets: `marker.<id>` and `party` take the union of the cell rect and the drawn rect (sprite or icon) and are pushed
  in feet order, front-most last (STAGE1 3.8), so a head over the row north is clickable and the front one wins.
  As built (decision "EXPLOREVIEW1-L click areas and sort"): the party and people take their own cell only; icon markers
  keep the union.
- Names and badges over the drawn top, kept inside the map area, as today.

### 3.6 Controls (MAPNAV1-U and WALK2 rules unchanged, the 3.1 geometry)

Click to walk takes the cell from `cell_at` ("You can't get there." unchanged); keys step one row or column (up is
north), E acts on the faced cell, Q passes the lead, a click on the leader passes it; edge arrows point at the tilted
cell centres; a legend row or an arrow pans the camera to the marker's cell, a second row click walks, Esc or a step
recentres; WALK2's hold-to-walk timing is untouched. Steps snap from cell to cell in both views (there is no glide
today; STAGE1 8's "the glide" names none).

### 3.7 The same in both views

Rules, walking, saves (`position`, `explore.leader`), journeys, map templates, PROPS1 placements, the world map, the
battle, the HUD bar, the legend, the ticker, the dialogue box, the shop, and the agent link names (`map`,
`marker.<id>`, `party`, `arrow.<id>`, `state.sprites`, `state.arrows`, `state.pan`, `state.leader`).

## 4. Acceptance checks (`mapview.rs` and `region.rs` tests)

1. Square: `MapGeom` gives today's cell rects, cells under points, origins and sprite rects over a 48 x 30 sweep; every
   existing region test passes unchanged; a frame with `region_view` missing equals one with `square`.
2. Tilted: `cell_at` of each cell's centre and corners-inside returns that cell over a 48 x 30 sweep; a click at a
   floor cell's centre walks there; a click on a wall logs "You can't get there.".
3. Tilted camera: the party's cell centre is the map area's centre; in a talk both speakers' cells lie above the box
   (the PLAYUI1-D test run under both views).
4. Walls: a wall with an open south cell draws its face (pixel probe darker than the cap), one with a wall south does
   not; the leader standing north of a wall is drawn behind its cap and gets its outline.
5. Sprites: the leader's sprite bottom is the cell's bottom edge and `stands_on` maps it back to the party's cell in
   both views; people face the party in a talk.
6. Props: a 10 x 6 footprint at (4, 3) is (ox + 64, oy + 36, 160, 72) tilted; a 64 x 112 PNG draws 32 x 56,
   bottom-centred on it; a prop without art draws the raised box.
7. MAPNAV1-U under both views: an off-screen marker gets an edge arrow; a legend row pans to the marker's cell and a
   second click walks; Q passes the lead; `state.view` names the view.
8. `text_audit` clean at the four text sizes in both views (the region readability test run under both).
9. `main.rs` forces `square` outside the Scene Runner; the runner's select saves `region_view`.
10. Journey golden, store resave and battle traces unchanged (presentation only; no parity check retires).

## 5. Slices and rows (status `spec`; the planner writes them)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| EXPLOREVIEW1-L | The `region_view` debug setting and the tilted region map (sections 2-4) | game | logic | MAPNAV1-U, WALK2 | `emberwake_gui/src/{mapview (new),region,props,runner,screens,main,lib}.rs`, `SCREENS.md` 07, `GAME_BRIEF.md` |
| EXPLOREVIEW1-R | Review session with the user (section 6): the pick, and the follow-up rows | game | design (planner) | EXPLOREVIEW1-L | `C:/dev/localgen/review/exploreview1/`, decisions, HISTORY |

## 6. The review session (EXPLOREVIEW1-R)

Prep (the planner over the agent link, sessions muted): the maps with props, `ashlands_cinder_gate` (22 props),
`ashlands_ruined_courtyard` (15), `ashlands_warden_keep` (10), and `walled_town` (no props); for each, 1280 x 720
shots in both views: the leader on open ground, beside a wall to its north and to its south, behind a standing prop,
in a talk, with the legend open and an edge arrow, at Normal and Extra large text; one Ashlands battle with walls and
water in Stage and in Flat; a building still (question 3). The board
`C:/dev/localgen/review/exploreview1/index.html` shows the two pairs of section 1 side by side with `file:///` links.

Session: the user plays both pairs in the Scene Runner (Exploration entry, the two Developer selects) for about ten
minutes each: walk with keys and clicks, pass the lead, pan by the legend, talk, start a fight. Then the questions:

- R1. Which pair ships: (a) Tilted, (b) Square, (c) reopen the decision for a split (square map, Stage battle).
- R2. If Tilted: the building art (PROPS1-A2): (a) bases drawn for 4:3 footprints; (b) square-drawn art as today.
- R3. The other view: (a) stays a debug setting (the decision's rule); (b) a cleanup row removes it.

The planner then writes: the decision row; a HISTORY row for the view not picked; LOOK1 5.1's region sizes (32 x 32
or 32 x 24 art) and the unblocking of LOOK1-A1-A3, LOOK1-L2 and PROPS1-A2; a follow-up row that makes the pick the
shipped view (`main.rs` forces it; for (b) the battle ships Flat, which retires STAGE1's Stage as the default and its
see-through rules, named with the HISTORY row).

## 7. Out of scope

Isometric diamonds and diagonal views; a camera glide, smooth scrolling or a clamp; new tile or building art (LOOK1,
PROPS1-A2); the world map; battle changes; a grid or hover plate on the region map; map rotation; the art pipeline.

## 8. Open questions (recommended first)

1. Ground art in the tilted test: (a) each location's square tiles squashed into 16 x 12 (keeps every place's
   colours; LOOK1 makes 4:3 art for the winner); (b) the battle's `common_stage` 4:3 tiles for the three kinds (true
   4:3 art, but generic and 8 art px per logical px, finer than the sprites). Recommend (a).
2. Switching: (a) the Developer panel select only, like STAGE1-C; (b) also a debug-build key (F7, unused today) that
   flips the region view live. Recommend (a): the board gives the side-by-side, and the shipped keys stay as they are.
3. The building still for R2: (a) two local images (FLUX.2 Klein, no approval: one building with its base drawn for a
   square footprint and one for 4:3) composed onto both views' shots on the board; (b) the PROPS1 4.6 boxes only.
   Recommend (a): STAGE1 8 named the building art as the main cost of a tilt.
