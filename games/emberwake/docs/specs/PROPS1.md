# PROPS1: props and buildings on region maps

Status: proposed (spec 2026-10-22; the user answers section 10 before any build row starts)
Depends on: PLAYUI1 (approved; this is its section 2.5 option A, user answer 9b.6); per row in section 11
Lane: logic (engine draw list, game schema, checks and drawing, the sheet tool), asset (placements, art)
Touches: `engine/crates/aegis_ui/src/{depth,art,canvas,lib}.rs`;
`games/emberwake/crates/emberwake_content/src/{source,store,art}.rs`, `.../check/{map,assets,mod}.rs`;
`games/emberwake/crates/emberwake_world/tests/{store,data_source}.rs`;
`games/emberwake/crates/emberwake_gui/src/{region,props}.rs`; `tools/portraits/sheets.py`;
`games/emberwake/data/maps/maps.json` (through `aegis content`); `kits/rpg/assets/props/`,
`games/emberwake/assets/props/`

Sources: PLAYUI1 2.5 and 9b; preferences `assets-world.md` (Style A, 16-colour palette, layers, image API rule) and
`ui.md` ("Maps"); ASSETORG2 sections 2-4 (layers, tree, tile lookup 4.5); the map templates in
`games/emberwake/data/maps/maps.json` and their checks (`emberwake_content::check::map`); `region.rs` as it is on
2026-10-22 (16 logical px tiles, tiles then markers then the party, no sort).

Status update (2026-10-22, user): approved, section 10 answered: Q1 A (standing props only on existing walls and the map border; flat decals elsewhere), Q2 A (props have no interaction; a sign's text comes from a marker beside it), Q3 A (three image-API sheets, each asked first), Q4 the party's outline shows behind a roof. Q5 (building roles) not asked: the spec's names stand until the user renames them.

## Goal

Region maps read as places: the Warden Keep yard has its keep, barracks, forge and storehouse, the Ashmaw Camp its
ring of tents, the courtyard its broken pillars, the gorge its cliffs and the toll arch. Props are art drawn over the
tiles, one or more tiles big, sorted with the party and the NPCs by their feet so a character can walk behind a roof.
Walls and walking stay exactly as the template `rows` say: a prop never changes where anyone can step, so saves,
anchors, roamer paths and the journey parity are untouched.

## 0. Layers

| Piece | Layer | Row |
| --- | --- | --- |
| Depth-sorted draw list; mirrored images; native image size; variant pick over folders | engine (`aegis_ui`) | PROPS1-E |
| `props` field on map templates: normalize, storage, checks P1-P12 | game (`emberwake_content`) | PROPS1-L1 |
| Prop art chain `props/<set>/<art>/`, asset checks A7-A10 | game (`emberwake_content`) | PROPS1-L1 |
| Drawing on the region map: passes, sort, fallback, party outline; layout test pattern | game (`emberwake_gui`) | PROPS1-L2 |
| Prop sheet prompts and slicer | tools (`tools/portraits/sheets.py`) | PROPS1-T |
| Generic prop art (`kits/rpg/assets/props/common/`) | kit | PROPS1-A2 |
| Ashlands prop art (`games/emberwake/assets/props/ashlands/`) | game | PROPS1-A2 |
| Ashlands placements (`props` on the four Ashlands templates) | game data | PROPS1-A1 |

Changed from PLAYUI1 2.5: the prop field `solid` folds into `layer` (rule P9: a standing prop is solid because it
stands on wall tiles; a ground prop is walkable where the tiles are). Art paths gain a set level, as tile sets have:
`props/<set>/<art>/v<n>.png`, so a template shared by two worlds draws each world's version of the same prop.

## 1. Schema contract (PROPS1-L1, schema-first: yes)

New field on map templates (`maps` kind, `data/maps/maps.json`, still version 1): `props`, a list, default `[]`.
`source::map_template` adds it after `paths` (before `tags`); `store::GAME_KINDS` stores it in that place. Each entry
is normalized to the full shape below and stored with every key in this order.

| Key | Type | Default | Allowed | Meaning |
| --- | --- | --- | --- | --- |
| `id` | string | required | `[a-z0-9_]`, 2-24 chars, unique in the template | name for edits, tests and messages |
| `art` | string | required | `[a-z0-9_]`, 2-32 chars | the art folder name (section 3) |
| `x`, `y` | int | required | footprint inside the map | top-left tile of the footprint (column, row) |
| `w`, `h` | int | 1 | 1-16 | footprint size in tiles |
| `layer` | string | `sort` | `sort`, `ground` | `sort`: stands up, sorted with characters; `ground`: flat, under every character |
| `variant` | int | -1 | -1 to 15 | -1: picked by position (3.2); 0-15: that file |
| `flip` | bool | false | true, false | mirrored left to right |

Constants (`emberwake_content::check::map`): `PROPS_MAX` 128, `PROP_SIDE_MAX` 16, `PROP_VARIANT_MAX` 15.
Nothing else changes: no save field, no `GameState` key, no world-layer field, no `marker_view` change.

Parity retired (named in L1's commit and a HISTORY row): the writer parity (`tests/store.rs` `AFTER_GODOT`) drops the
line `"props": []`; the loader parity (`tests/data_source.rs`) takes the key `props` off the engine's map templates
before comparing (Godot's MapCatalog had no such field). The re-save of the live data stays byte for byte.

## 2. Validation (PROPS1-L1)

Errors, run by `check::validate_all` under the key `maps:<id>`, so `aegis content` refuses an upsert that breaks them
and `tools/fc.sh content check` lists them. The engine has no port of Godot's `template_errors`; only the prop checks
are added (porting the rest is out of scope). Messages, in the order they run:

| Check | Fails when | Message |
| --- | --- | --- |
| P1 | `props` is not a list of objects | `Props must be a list of objects.` |
| P2 | more than `PROPS_MAX` entries | `At most 128 props per map (got N).` |
| P3 | unknown key in a prop | `Prop '<id>': unknown field '<key>'.` |
| P4 | bad or missing id | `Prop N: id '<id>' must be lowercase [a-z0-9_], 2-24 characters.` |
| P5 | id used twice | `Prop id '<id>' is used twice.` |
| P6 | bad or missing art | `Prop '<id>': art '<art>' must be lowercase [a-z0-9_], 2-32 characters.` |
| P7 | x, y not whole numbers, or w, h not 1-16 | `Prop '<id>': x and y are whole numbers; w and h are 1-16.` |
| P8 | footprint leaves the map | `Prop '<id>': its footprint (x..x2, y..y2) leaves the map (W x H).` |
| P9 | a `sort` prop has a footprint tile that is not `#` | `Prop '<id>': a standing prop needs wall tiles under its whole footprint; (x, y) is '<c>'.` |
| P10 | two `sort` props share a tile | `Props '<a>' and '<b>' stand on the same tile (x, y).` |
| P11 | layer not `sort` or `ground` | `Prop '<id>': layer must be sort or ground.` |
| P12 | variant not -1..15, or flip not a bool | `Prop '<id>': variant is -1 (by position) or 0-15; flip is true or false.` |

P9 names the first offending tile (row by row); P8, P9 and P10 run only when the rows have a usable grid and P7
passed, and P9 and P10 only for props that pass P8. Ground props may lie on any tiles (floor, wall, water) and may
overlap each other and sort props. `(x, y)` in messages are the tile's column and row.

## 3. Art lookup and asset checks (PROPS1-L1)

### 3.1 Paths and chain

- File: `props/<set>/<art>/v<n>.png` in any layer root (game, kit, engine; ASSETORG2 2). Generic props live in the
  kit's `props/common/`, world props in the game's `props/<world>/`. A game file at the same path overrides the kit.
- Chain, as tiles (ASSETORG2 4.5): the world lineage child first, then `common`. For each set, the first layer holding
  `props/<set>/<art>/v0.png` gives the prop; its variant count n is the run of `v0..v(n-1)` in that layer.
  `emberwake_content::art::prop_dirs(lineage_child_first, art) -> Vec<String>` returns the folders in order.
- Art scale: 32 px of art per tile (the tile sets' scale). A standing prop's PNG is 32 x w to 32 x w + 32 px wide
  and 32 x h to 32 x h + 128 px tall (up to 4 tiles of roof, chimney or canopy above the footprint); a ground prop's
  PNG is exactly 32 x w by 32 x h.

### 3.2 Variant

-1: (x x 7 + y x 13) mod n (Euclidean, x and y the footprint's top-left), the tile formula. 0-15: that file; a value
at or above n shows `v0` (and warns, A9).

### 3.3 Asset checks (warnings, never blocking; `check::assets`, beside A1-A6)

Run for every world location with a map, over its template's props, under that world's chain; one line per
(map, prop art, world).

| Check | When | Message |
| --- | --- | --- |
| A7 | no `v0.png` along the chain | `Map '<map>' prop art '<art>': no file props/<set>/<art>/v0.png for world '<w>'; it draws its shape.` |
| A8 | PNG size outside 3.1 | `Prop art '<rel>' is WxH px; a w x h <layer> prop wants <range>.` |
| A9 | explicit variant at or above n | `Map '<map>' prop '<id>': variant V, but '<art>' has n files; it shows v0.` |
| A10 | a standing prop's drawn rect (4.3) covers the centre of an anchor or path tile | `Map '<map>' prop '<id>' hides '<anchor or path>' at (x, y) behind its art.` |

A10 uses the art's PNG size when the file exists, else the footprint (which holds no anchor, P9).

## 4. Drawing (PROPS1-E engine pieces, PROPS1-L2 game)

### 4.1 Engine pieces (PROPS1-E, `aegis_ui`, game-agnostic)

- `depth.rs`: `DepthList<T>`: `push(foot_y: f32, group: u8, x: f32, item: T)`; `into_sorted() -> Vec<T>` ordered by
  foot_y, then group, then x, then push order (stable). No drawing of its own; any screen (region map, later the
  battle grid) sorts what it draws with it.
- `Image::mirrored(&self) -> Image` (left to right) and `Art::image_mirrored(rel, w, h)` cached like `Art::image`.
  If PLAYUI1-R lands first with the same helper, E reuses it.
- `Art::size_of(rel) -> Option<(i32, i32)>`: the PNG's own size, cached.
- `Art::varied(dirs: &[&str], variant: Option<u32>, x: i32, y: i32, w: i32, h: i32) -> Option<&Image>`: the first
  folder holding `v0.png` in any layer, then the explicit variant (out of range: `v0`) or the position pick of 3.2.
  `tile_in` calls it (same results as today; its tests stay).

### 4.2 Passes on the region map (PROPS1-L2)

1. Tiles (today), then the wall margin past the map edge (PLAYUI1-M).
2. Ground props, in list order.
3. The sorted pass: standing props, NPC and other markers, the party (PLAYUI1-R's sprites when they land), through one
   `DepthList`.
4. The party outline when hidden (4.5).
5. Overlays, never covered by a prop: names and hover labels (`map_names`), quest and service badges, the hover and
   facing line, the legend, the log ticker, the HUD, the dialogue box, popups.

### 4.3 Placement

- Drawn size: the PNG size x ts / 32 (ts = 16 logical px today; the same scale as tiles).
- Bottom edge on the footprint's bottom edge ((y + h) x ts from the map origin); centred horizontally on the
  footprint. `flip` mirrors the image around that centre.
- A prop draws when its drawn rect (not its footprint) meets the map area, so a roof whose footprint is off screen
  still shows, and an overhang above row 0 draws over the wall margin.
- The pure placement lives in a new `emberwake_gui/src/props.rs`: `layout(t, origin, ts, size_of) -> Vec<PropDraw>`
  (rect, art folders, variant pick, flip, layer, foot_y), so region.rs only calls it and draws.

### 4.4 Sort keys (pass 3)

| Item | foot_y (logical px) | group |
| --- | --- | --- |
| Standing prop | the footprint's bottom edge | 0 |
| Marker (NPC, chest, lore, exit, encounter, roamer, point) | its tile's bottom edge | 1 |
| The party | its drawn tile's bottom edge, the glide's interpolated position included | 2 |

x is the item's left edge. Effect: a character south of a building draws over it; a character north of it, on the
open row under its roof, is covered by the overhang; at equal feet the character is in front. Markers keep their tile
hit rects: a marker behind a roof is still clickable at its tile and listed in the legend.

### 4.5 Party behind a prop

When the party's drawn rect (its tile, or PLAYUI1-R's 16 x 24 sprite rect) meets the drawn rect of a standing prop
sorted after it, pass 4 draws the party's outline over the props: before PLAYUI1-R a 1 px ring of the party colour
(rgb 250, 214, 60) at radius ts x 0.3; after it, `Art::body_outline` of the leader's current frame in that colour.
NPCs get no outline (A10 keeps their anchors clear).

### 4.6 Missing art

A standing prop with no file draws its footprint filled rgb(52, 46, 42) with a 1 px rgb(96, 84, 72) border; a ground
prop with no file draws nothing. `Art::image` already reports the missing file to the playtest asset log once.

## 5. Interaction

None in PROPS1. Props have no widget, hover, tooltip or click: a click on a prop's art is a click on the tile under
the cursor (click to walk, or a marker's tile), as today. A sign's words come from the world layer: a `point` marker
(its `note` shows on stepping there) or a `lore` marker on the open tile beside the sign prop (question 2).

## 6. Assets (PROPS1-A2 art, PROPS1-A1 placements)

### 6.1 Art rules

- Style A: 16-bit pixel art, a 16-colour palette per image, dark outlines, light from the upper left; the 3/4
  top-down view of the region tiles (top and south face visible); transparent background; no text or letters.
- Standing props: the base fills the footprint's width at the bottom of the image; anything above is roof, chimney,
  banner or canopy. Ground props: flat, seen from above, ragged edges fading to transparent.
- `cliff_face` variants tile left to right with each other.

### 6.2 Art list (Ashlands first: 21 arts, 28 files)

Kit, `kits/rpg/assets/props/common/<art>/v<n>.png`:

| Art | Files | PNG px | Footprints used | Layer | What |
| --- | --- | --- | --- | --- | --- |
| `tent` | v0, v1 | 96 x 144 | 3 x 3 | sort | hide tent on poles, rope pegs; v0 tan, v1 grey hide |
| `broken_pillar` | v0-v2 | 64 x 112 | 2 x 2 | sort | broken stone pillar on a square base, rubble at its foot; heights differ |
| `fallen_column` | v0 | 96 x 48 | 3 x 1 | sort | a toppled column lying east to west |
| `rubble_wall` | v0 | 128 x 64 | 4 x 1 | sort | low broken wall of fitted stones |
| `boulder` | v0, v1 | 64 x 80 | 2 x 2, 2 x 1 | sort | dark rock boulder |
| `rock_spire` | v0 | 64 x 144 | 2 x 3, 2 x 4 | sort | tall jagged rock spire |
| `cliff_face` | v0, v1 | 128 x 64 | 4 x 1 | sort | south-facing cliff face, horizontal strata, tiles left to right |
| `signpost` | v0 | 32 x 64 | 1 x 1 | sort | wooden post with two blank arrow boards |
| `banner_pole` | v0 | 32 x 96 | 1 x 1 | sort | tall pole with a plain red cloth banner |
| `gatepost` | v0 | 32 x 80 | 1 x 1 | sort | squat stone gatepost with a cap and a lit torch bracket |
| `crate_stack` | v0 | 64 x 64 | 2 x 1 | sort | two crates and a barrel |
| `fire_pit` | v0 | 32 x 32 | 1 x 1 | ground | ring of stones around embers |
| `cobbles` | v0, v1 | 128 x 96 | 4 x 3 | ground | sooty cobblestone patch, ragged edges |
| `scorch_mark` | v0, v1 | 64 x 64 | 2 x 2 | ground | burnt ground and ash |

Game, `games/emberwake/assets/props/ashlands/<art>/v0.png`:

| Art | PNG px | Footprint | Layer | What |
| --- | --- | --- | --- | --- |
| `keep_hall` | 320 x 256 | 10 x 6 | sort | squat basalt keep hall, slate roof, two short corner towers, red Warden banners with the ember sun, a glowing hearth door |
| `barracks` | 320 x 224 | 10 x 6 | sort | long basalt barracks, timber and slate roof, shuttered windows, weapon racks in front |
| `forge` | 288 x 272 | 9 x 7 | sort | basalt smithy, tall chimney, open front with a glowing forge and anvil |
| `storehouse` | 288 x 256 | 9 x 7 | sort | basalt storehouse, wide double doors, crates and sacks in front |
| `cistern` | 224 x 160 | 7 x 5 | ground | stone rim around a dark pool, the water filling the middle 5 x 3 |
| `war_tent` | 96 x 160 | 3 x 3 | sort | Grakka's large hide tent: bone poles, red goblin pennants, skulls |
| `toll_arch` | 64 x 192 | 2 x 5 | sort | the surviving half of a collapsed basalt toll arch, a broken stub leaning west |

### 6.3 Placements (PROPS1-A1)

Every footprint below was checked against the shipped rows: standing props lie on `#` tiles only, no two share a
tile, and no anchor or path tile lies under an overhang (A10 clean with the 6.2 sizes). `variant` is -1 unless named;
`flip` is false unless named. Entries go into each template's `props` in the order listed.

Warden Keep (`ashlands_warden_keep`, 10 props):

| id | art | x, y | w x h | layer | note |
| --- | --- | --- | --- | --- | --- |
| `keep_hall` | `keep_hall` | 4, 3 | 10 x 6 | sort | the north-west block |
| `barracks` | `barracks` | 34, 3 | 10 x 6 | sort | the north-east block |
| `forge` | `forge` | 4, 17 | 9 x 7 | sort | the south-west block |
| `storehouse` | `storehouse` | 35, 17 | 9 x 7 | sort | the south-east block |
| `cistern` | `cistern` | 21, 13 | 7 x 5 | ground | around the 5 x 3 pool |
| `gatepost_w` | `gatepost` | 23, 29 | 1 x 1 | sort | west of the gate |
| `gatepost_e` | `gatepost` | 25, 29 | 1 x 1 | sort | east of the gate, flip true |
| `road_sign` | `signpost` | 0, 27 | 1 x 1 | sort | on the west wall by the road exit |
| `stores_crates` | `crate_stack` | 1, 0 | 2 x 1 | sort | on the north wall above the stores chest |
| `muster_banner` | `banner_pole` | 47, 11 | 1 x 1 | sort | on the east wall by the muster NPC |

Ruined Courtyard (`ashlands_ruined_courtyard`, 15 props):

| id | art | x, y | w x h | layer | note |
| --- | --- | --- | --- | --- | --- |
| `pillar_1` to `pillar_7` | `broken_pillar` | 14, 6; 22, 6; 30, 6; 14, 21; 22, 21; 30, 21; 38, 21 | 2 x 2 | sort | one per 2 x 2 block |
| `fallen_column` | `fallen_column` | 18, 10 | 3 x 1 | sort | |
| `rubble_a` | `rubble_wall` | 25, 10 | 4 x 1 | sort | |
| `rubble_b` | `rubble_wall` | 18, 17 | 4 x 1 | sort | flip true |
| `cobbles_a`, `cobbles_b`, `cobbles_c` | `cobbles` | 16, 12; 24, 13; 31, 16 | 4 x 3 | ground | the sooty square |
| `scorch_a`, `scorch_b` | `scorch_mark` | 9, 17; 40, 9 | 2 x 2 | ground | |

Cinder Gate (`ashlands_cinder_gate`, 22 props):

| id | art | x, y | w x h | layer | note |
| --- | --- | --- | --- | --- | --- |
| `cliff_1` to `cliff_12` | `cliff_face` | x = 0, 4, 8 ... 44; y = 8 | 4 x 1 | sort | the north cliff's face over the pass |
| `boulder_1` to `boulder_4` | `boulder` | 9, 9; 15, 9; 33, 9; 39, 9 | 2 x 2 | sort | the rock teeth under the cliff |
| `spire_n` | `rock_spire` | 23, 9 | 2 x 4 | sort | |
| `spire_s1`, `spire_s2`, `spire_s3` | `rock_spire` | 25, 18; 35, 18; 41, 18 | 2 x 3 | sort | `spire_s2` flip true |
| `boulder_5` | `boulder` | 17, 18 | 2 x 1 | sort | |
| `toll_arch` | `toll_arch` | 43, 12 | 2 x 5 | sort | the column by the east exit |

The south rock teeth at (11, 18) (1 x 3, irregular) and (17, 20) (2 x 1, its overhang would hide the patrol path)
keep the plain wall tile.

Ashmaw Camp (`ashlands_ashmaw_camp`, 16 props):

| id | art | x, y | w x h | layer | note |
| --- | --- | --- | --- | --- | --- |
| `war_tent` | `war_tent` | 26, 4 | 3 x 3 | sort | north of Grakka (anchor `site_b` 24, 9) |
| `tent_1` to `tent_8` | `tent` | 8, 8; 12, 11; 34, 11; 23, 13; 18, 18; 30, 18; 12, 22; 38, 22 | 3 x 3 | sort | one per 3 x 3 block |
| `fire_1` to `fire_4` | `fire_pit` | 16, 15; 28, 16; 22, 10; 36, 20 | 1 x 1 | ground | walkable embers |
| `banner_n` | `banner_pole` | 0, 12 | 1 x 1 | sort | on the west wall north of the exit |
| `banner_s` | `banner_pole` | 0, 17 | 1 x 1 | sort | south of the exit, flip true |
| `scorch` | `scorch_mark` | 20, 6 | 2 x 2 | ground | |

Not placed: `walled_town` and `gorge_ruins` (Iron Reaches and older test maps; one world first).

Authoring: one seed `games/emberwake/data/seeds/props1_ashlands.json` applied with `aegis content apply`, one
`upsert` of kind `maps` per template with the full entry: `rows`, `zones`, `zone_legend`, `anchors`, `paths`, `tags`
copied unchanged, `props` added.

## 7. Art plan (PROPS1-T tool, PROPS1-A2 art; every image run asked first)

Three image API runs, each a sheet of square cells on a flat dark background with black gutters (the ART2 icon sheet
form), proposed to the user one at a time with the prompt, the model and the image count (one image each):

| Run | Grid | Cells, row by row |
| --- | --- | --- |
| S1 kit standing props | 4 x 4 | `tent` v0, v1; `broken_pillar` v0, v1, v2; `fallen_column`; `rubble_wall`; `boulder` v0, v1; `rock_spire`; `cliff_face` v0, v1; `signpost`; `banner_pole`; `gatepost`; `crate_stack` |
| S2 Ashlands buildings | 2 x 2 | `keep_hall`, `barracks`, `forge`, `storehouse` |
| S3 Ashlands and ground | 3 x 3 | `war_tent`, `toll_arch`, `cistern`, `fire_pit`, `cobbles` v0, v1, `scorch_mark` v0, v1, a second `war_tent` take (the user picks) |

PROPS1-T extends `tools/portraits/sheets.py` (Python and Pillow, portable to Rust): `props-prompts <out.md>` writes
the three prompts to `games/emberwake/docs/specs/PROPS1_PROMPTS.md` from a cell table in the script (cell, art, layer
root, set, variant, PNG size, description of 6.2); `slice-props <sheet.png> <sheet_id> <repo root>` cuts each cell,
removes the background (pixelize's edge flood fill), trims to the content, scales it with nearest neighbour to fit the
PNG size of 6.2 keeping its aspect, places it bottom-centred on a transparent canvas of exactly that size, snaps it to
at most 16 colours with a dark 1 px outline (pixelize's rules) and writes it to its layer path. Until a file lands the
map draws 4.6's shape.

## 8. Acceptance and tests

PROPS1-E (unit tests in `aegis_ui`):

- `DepthList` orders by foot_y, group, x and push order; equal keys keep push order.
- `Image::mirrored` swaps the pixel columns of a 3 x 2 test image; mirroring twice gives the original.
- `Art::varied` on a temp tree: explicit variant, out-of-range variant gives `v0`, position pick equals the
  `tile_in` result for the same folders; `Art::size_of` reads a PNG's size.

PROPS1-L1 (`emberwake_content` tests, `emberwake_world/tests/store.rs`):

- A template without `props` loads with `props: []`; a prop with only `id`, `art`, `x`, `y` is stored with
  `w` 1, `h` 1, `layer` "sort", `variant` -1, `flip` false, keys in section 1's order.
- One failing case per P1-P12 gives exactly its message; the shipped data stays clean; `aegis content upsert` of a
  map with a P9 prop is refused and leaves `maps.json` unchanged.
- `art::prop_dirs(["ashlands"], "tent")` is `["props/ashlands/tent", "props/common/tent"]`.
- A7-A10 on a temp asset tree each give their line once; A10 fires for a 1 x 1 standing prop with 32 x 80 art right
  south of an anchor and not for 32 x 32 art.
- The parity suites pass with the two retired keys (section 1); the re-save of the live data is byte for byte.

PROPS1-L2 (`emberwake_gui` tests, agent link):

- `props::layout`: `keep_hall` (10 x 6 at 4, 3, art 320 x 256) at origin (ox, oy), ts 16 gives the rect
  (ox + 64, oy + 16, 160, 128); `flip` keeps the rect; culling keeps a prop whose footprint is outside the map area but
  whose overhang meets it.
- Sort: the party at (8, 2) (north of `keep_hall`, foot 48) draws before the hall (foot 144); at (8, 9) after it.
- The party at (8, 2) is hidden by the hall, so the outline is drawn; at (24, 2) it is not.
- The layout test (`region.rs` tests) accepts `props/<set>/<art>/v<n>.png` and still fails a stray file.
- Over the agent link, with A1's data: a screenshot of each Ashlands map at its spawn for the user's spot check;
  `text_audit` stays clean at the four text sizes; walking, markers, exits and the journeys behave as before (the
  journey and save suites pass unchanged).

PROPS1-A1: `tools/fc.sh content check` clean; warnings A9 and A10 none, A7 only for art not landed yet; the diff of
`maps.json` adds `props` arrays and changes nothing else.

PROPS1-A2: the 28 files at the 6.2 paths with the 6.2 pixel sizes, transparent backgrounds, at most 16 colours each;
the layout test passes; A7 and A8 none for the Ashlands; a visual spot check by the user (the ASSETORG2-T rule:
paths and sizes plus a look, no byte-for-byte check).

## 9. Out of scope

Blocking from props (walls stay in `rows`); new wall tiles in the templates (question 1); prop interaction, prop
hover names and markers drawn as props (question 2); an `over` layer drawn above every character (arches, canopies
the party walks under; no Ashlands placement needs it); animated props (fire, banners); props on battle maps and the
world map; props for `walled_town`, `gorge_ruins` and the other worlds; a props view in the Content or World Forge;
porting Godot's full `template_errors`; light and shadow effects.

## 10. Open questions (the user decides; recommendation first)

1. Clutter on open ground (crates, barrels, standing campfires): A, first pass puts standing props only on existing
   walls and the map border, flat decals elsewhere (recommended; nothing in the maps moves); B, a later map-edit task
   adds `#` tiles for clutter spots in the Ashlands templates (the walking changes: the journey parity for those maps
   and saves standing on a new wall need a rule).
2. Signs: A, props have no interaction and a sign's text is a `point` or `lore` marker beside it (recommended; no
   code); B, a later PROPS2 lets a location marker be drawn as a prop (its square hidden, the prop the thing you
   face).
3. Art source: A, the three image API runs of section 7, each asked before generating (recommended); B, drawn
   placeholders from `tools/gen_art.py` now and the API later; C, the drawn footprint boxes (4.6) until the full game
   runs (preference "asset feedback after the full game runs").
4. Party behind a roof: draw its outline over the prop (recommended), or let it vanish behind the art?
5. The four Warden Keep blocks as keep hall, barracks, forge and storehouse, and the Ashmaw block north of Grakka as
   his war tent: keep these roles or rename them?

## 11. Rows wanted (status `spec`; the planner writes them)

| Id | Title | Layer | Lane | Depends on |
| --- | --- | --- | --- | --- |
| PROPS1-E | Engine draw pieces: `DepthList` (sort by feet), mirrored images, native image size, `Art::varied` (4.1) | engine | logic | - |
| PROPS1-L1 | Props on map templates: `props` field, storage, checks P1-P12, art chain `props/<set>/<art>/`, asset checks A7-A10, parity retirements (1-3) | game | logic | - |
| PROPS1-L2 | Region map draws props: ground pass, feet-sorted pass with markers and the party, placement, fallback, party outline, layout pattern (4.2-4.6) | game | logic | PROPS1-E, PROPS1-L1, PLAYUI1-M |
| PROPS1-T | Prop sheet prompts and slicer in `tools/portraits/sheets.py` (7) | game | logic | - |
| PROPS1-A1 | Ashlands placements: `props` on the four Ashlands templates through a seed (6.3) | game | asset | PROPS1-L1 |
| PROPS1-A2 | Prop art: 3 image API sheets (asked one at a time), sliced into 21 kit and 7 game files (6.2, 7) | kit + game | asset | PROPS1-T, PROPS1-L2, question 3 |

Touches: E `engine/crates/aegis_ui/src/{depth,art,canvas,lib}.rs`; L1
`games/emberwake/crates/emberwake_content/src/{source,store,art}.rs`, `.../check/{map,assets,mod}.rs`,
`games/emberwake/crates/emberwake_world/tests/{store,data_source}.rs`, `games/emberwake/data/maps/README.md`; L2
`games/emberwake/crates/emberwake_gui/src/{region,props}.rs`; T `tools/portraits/sheets.py`,
`games/emberwake/docs/specs/PROPS1_PROMPTS.md`; A1 `games/emberwake/data/maps/maps.json`,
`games/emberwake/data/seeds/props1_ashlands.json`; A2 `kits/rpg/assets/props/common/`,
`games/emberwake/assets/props/ashlands/`. L2 joins the `region.rs` sequence (PLAYUI1-H, M, D, P, R) after PLAYUI1-M;
if it runs before PLAYUI1-R, R pushes its sprites into the same `DepthList` with the 4.4 keys. E, L1 and T run beside
anything; A2's files need L2's layout pattern to pass the layout test.

Docs delta for the builders: `docs/STRUCTURE.md` (`aegis_ui/src/depth.rs`, `emberwake_gui/src/props.rs`, the `props/`
folders in the kit and game asset rows), `kits/rpg/README.md` and `games/emberwake/assets/README.md` (the `props/`
tree), `games/emberwake/data/maps/README.md` (the `props` field; edits through `aegis content`),
`games/emberwake/docs/GAME_BRIEF.md` "Region maps" (props and the draw order).
