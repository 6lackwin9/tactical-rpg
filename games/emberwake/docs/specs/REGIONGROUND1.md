# REGIONGROUND1: region maps with several ground types and natural blends

Status: spec (2026-11-02; direction from the decisions "Region maps mix ground types; layout before tile sets", "Tiles
must connect naturally on all sides", "Ground blends: a generated edge with light wear", "Art scaling by asset type",
"Tile textures must not show a grid when repeated", "Prop sheets by size class; tile textures straight-down")
Depends on: LOOK1-E2, LOOK1-K, LOOK1-T, LOOK1-T2, EXPLOREVIEW1-L, PROPS1-L2 (all built); section 8 per slice. Nothing waits
for EXPLOREVIEW1-R: square and tilted both work.
Lane: logic (E, K, L, T), art (A), asset (D), review with the user (R, planner)
Touches: by slice, section 8. Contracts: `docs/specs/REGIONGROUND1_ENGINE.md` ("EN" below) and
`docs/specs/REGIONGROUND1_KIT.md` ("KT"). Tall walls, hierarchy and reading: `REGIONGROUND1_SCENE.md` ("SC").

History: this reopens a `ground` char layer, which LOOK1 Q9 (b) did not adopt (HISTORY 2026-11-02): Q9's reason was "no
new ground kinds", and the decision "Region maps mix ground types" lifts that for region maps. "Buildings as more tile
kinds" and "painted location backgrounds" (2026-10-22) still hold: buildings stay props, and the picture is composed
from the grid, so a map edit redraws it. Gemini transition sheets (LOOK1 Q8 b) stay rejected: edges are code.
Presentation only: no rule, save, sim, trace or journey reads ground, and no parity check retires.

## Goal

A region map stops being one tile stamped in every cell. It mixes about 8 grounds that follow use: roads where people
walk, a place's own ground around each point of interest, quiet ash or soot everywhere else, as in the user's mocks
(`C:/dev/localgen/review/look1/region_mock/mock_a_outdoor.png`, `mock_b_street.png`). Each ground is one seamless muted
texture of even tone, grounds meet at an irregular generated edge, and the look target is `dirt_flat.png` (right side)
with `dirt_patch_soft.png`. Walking, clicks and everything else stay on cells.

## 1. What this replaces

| Item | Today | After REGIONGROUND1 |
| --- | --- | --- |
| LOOK1 Q9, 5.2 "no new ground kinds" | three kinds | superseded for region maps; battle terrain is unaffected |
| LOOK1 5.2 `floor`, `shallow_water` (region) | a tile per cell | a ground texture on maps with ground; the tiles stay for a map without |
| LOOK1 5.3 region pair `floor` over `shallow_water` | planned | dropped: the generated edge does it. The stage's pairs: question 1 |
| LOOK1-L2 | transitions, wall caps and faces, bound sets | narrowed to wall caps and faces, after EXPLOREVIEW1-R. `tileset` wiring moves to REGIONGROUND1-L; transitions dropped |
| LOOK1-A1, A2, A3 (9 images each) | floor x 5, wall cap, face, water, decals | narrowed to wall cap, wall face, decal sheet (3 each, 9 in all), after EXPLOREVIEW1-R. Floor and water come from section 7 |
| LOOK1-A4, A5, L1 (stage tiles, corner pieces) | built | as built until question 1 is answered |
| LOOK1-R | region set rows | ground rows go with the D slices; its stage and title parts stand |
| Old placeholder ground | `floor`, `shallow_water` in `tilesets/ashlands`, `common` | not read by a map with ground. Dropped by slice C after the last Ashlands map (user: "as we move to fleshing out the old placeholder things can be dropped"); `common` stays as the engine baseline |

## 2. Map data (settled)

A map stores a `ground` grid of chars beside `rows`, with a `ground_legend`, the same form as `zones` and `zone_legend`
(contract KT 1.2). Rejected: named areas and paths that a tool rasterises: a second geometry language in the kit, and two
sources of truth once anyone edits a cell. Rows of chars are exact, checked per cell, diff well and are edited by
`aegis content upsert maps` today.

- `rows` decide walking. Walking, clicks, MAPNAV1-U, WALK2 and journeys never read ground, and an edge's push of at most
  0.3 cell (EN 2.3) never moves a cell.
- A map with `ground: []` draws today's frame exactly; every existing map does until its D slice.
- A map with ground: the composed image replaces `.` and `~` cells. `#` draws as today (square: the wall tile; tilted:
  the block in the sorted pass), and so does the wall margin past the map's edge. The letter under a `#` is what edges
  and patches see under the wall: use the neighbouring ground.
- The picture must not lie: check G7 keeps water grounds on `~` and land grounds off it.
- A ground without art draws its row's flat `colour`, so a layout is played before its tile set (decision "layout before
  tile sets").

## 3. The grounds

`games/emberwake/data/catalog/grounds.json` through `aegis content`. A row's art comes from the location's `tileset`,
then the world lineage, then `common` (`tilesets/<set>/ground/<id>/v0.png`), so `mud` looks different in each world under
one id. `colour` is the mock's swatch. Fields not listed take their defaults (KT 1.1).

| id | name | order | kind | colour | wear | strays | tone | patches | Class, gain (6) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `water` | Water | 5 | water | #0e1018 | 0 | no | .04 | 0 | natural, .55 |
| `ash` | Ash | 10 | land | #2a2622 | .06 | no | .13 | .20 | natural, .55 |
| `soot_yard` | Soot yard | 12 | land | #1e1e23 | .06 | no | .13 | .20 | natural, .55 |
| `scrub` | Scrub | 20 | land | #2b2d22 | .06 | no | .13 | 0 | natural, .55 |
| `moss` | Moss | 22 | land | #20261f | .06 | no | .13 | 0 | natural, .55 |
| `scorched` | Scorched ground | 30 | land | #1c1917 | .06 | no | .13 | 0 | natural, .55 |
| `tallow` | Tallow-stained ground | 32 | land | #2a251e | .06 | no | .13 | 0 | natural, .55 |
| `mud` | Mud | 40 | land | #231d18 | .06 | no | .13 | .15 | natural, .55 |
| `packed_dirt` | Packed dirt | 50 | land | #332e28 | .10 | no | .13 | .20 | natural, .55 |
| `dirt_road` | Dirt road | 60 | land | #39342e | .10 | no | .13 | .15 | natural, .55 |
| `flagstone` | Flagstones | 70 | land | #373636 | .10 | yes | .08 | 0 | built, .45 |
| `cobble_road` | Cobbled road | 78 | land | #2e3137 | .10 | yes | .08 | 0 | built, .45 |
| `paved_road` | Paved road | 80 | land | #3e3f42 | .10 | yes | .08 | 0 | built, .45 |
| `iron_catwalk` | Iron catwalk | 90 | land | #303032 | .06 | no | .08 | 0 | built, .45 |

Marks (slice D2; `marks_per_100` 2): `ash`, `packed_dirt` take `footprints`, `scuff`; `dirt_road` takes `footprints`,
`scuff`, `rut_short`; `soot_yard`, `mud`, `cobble_road` take `footprints`, `drag_mark`. Moss is the Sunk Ward's green:
no ground there is grass.

## 4. Textures

1. One texture per ground, covering 4 x 4 cells (EN 1.1), sampled by art pixel (EN 2.2), so a cell boundary does not
   exist in it. No variants: a variant per cell brings the boundary back.
2. Stored high-res and reduced: `v0.png` is 256 x 256 (4 art px per logical px, twice the cap's limit), reduced in one
   box step to 64 x 64 or 64 x 48 (`Art::image` is a box mean), so the tilted form is one step from the master and never
   a squash of a reduced tile. The cap and why it suits both views: EN 1.1.
3. Fine grain only. Broad light and dark in a texture reads as a loop across the map even when nothing else repeats. So
   the broad tone is flattened out by a tool step (`flatten.py`, section 6), keeping about half the fine detail, and
   broad tone comes from the map-wide noise (about +/-6 %, EN 2.2), which never loops. Prompts still ask for even tone.
4. No visible direction. Angled streaks repeat across the map, so the prompt forbids them and the check fails them.
   Averaging a tile with its rotations cancels the angle but leaves static-like grain (`dirt_no_angle.png`, right): a
   fallback only, `flatten.py --isotropic`, for one texture at a time on the user's say.
5. No shape lined up with the grid, no dark patches of its own, no one-off marks. Footprints and ruts belong to the mark
   layer (EN 3.4) or a PROPS1 `ground` prop along a road.

## 5. Blends, patches, marks and cost

Method: EN 2 and 3, the prototypes' numbers kept (`blend_proto2.py` to `blend_proto5.py`). Order is the table's `order`.
The user's rules on patches, in EN 3: separate from the tile; small and quiet; fainter as they grow; a size cap (260 art
px, about a cell), so large clusters never occur at random; large ones only from a map's `ground_patches`; a patch that
would reach a road or another ground is dropped whole, never clipped. Marks follow the same rules at 60 % alpha.

Connector pieces (the decision keeps them for a pair that wants a hand-made look): opt-in per pair, slice X, not
scheduled; `tilesets/<set>/ground/<upper>/over_<lower>/m<NN>.png` with `aegis_ui::autotile::corner_pieces`.

Cost: compose once, cache per map and scale, blit (EN 4). Art image 1.5 MB; physical copy 5.9 MB at 1280 x 720, 13.3 MB
at 1920 x 1080, 53.1 MB at 4K (48 x 30, square). Changes in a map's state draw on top; ground recomposes only for a new
map, view, size or edit (EN 4.3).

## 6. Tools (`tools/tiles`, slices T and T2)

| Tool | Does |
| --- | --- |
| `cut.py SRC --form ground --out DST` | a box reduce of the whole source to 256 x 256 (square, at most 3 % off, no upscale) |
| `flatten.py SRC --out DST [--gain G] [--isotropic]` | per channel, `mean + (T - low) x G`, `low` a Gaussian blur of the image tiled 3 x 3 (centre crop), sigma 5 at 64 px wide, so 20 at 256. `--isotropic`: `blend_proto6.py`'s average of the 8 rotations and mirrors x 2.1 plus its speckle; never a default |
| `groundcheck.py score PNG --class natural or built` | reduces to 64 x 64, prints the table's scores, exit 1 on a failure |
| `board.py SET_DIR --ground --out DIR` | each ground repeated over 16 x 10 and 32 x 20 cells at 1x-3x with scores and pass or fail; each pair on a straight edge, a T crossing, a 1-cell lane and a lone cell; the mix (T2); `index.html` with `file:///` links |

Per texture: `cut.py --form ground`, `seam.py` (fix, else re-roll), `flatten.py`, `groundcheck.py` (a failure: re-roll
once naming it in the prompt, then show the user), `board.py`. Numpy joins Pillow in the README.

Scores on the 64 x 64 form; L is luminance. Limits are calibrated on today's art (probe 2026-11-02); T may tune one only
by naming the art it must pass and fail, in its row.

| Check | Formula | Natural | Built | Calibration |
| --- | --- | --- | --- | --- |
| Seam (existing) | wrap step mean over inner step mean | 1.5 | 1.5 | `dirt_flat` 1.18 |
| Broad tone | stdev(L of the wrapped sigma 5 blur) / mean(L), % | 1.2 | 3.0 | `dirt_clean` 2.18 (the user saw it loop), `dirt_flat` 0.80 (target), `floor_r2` 5.76 raw, 1.48 flattened |
| Banding | max(stdev of row means, of column means) / stdev(L) | .30 | .30 | `floor_r2` .34 (passed seam, showed bands), `dirt_flat` .15 |
| Direction, peak | gradient energy by angle in 8 bins (wrapped central differences), largest over mean | 1.8 | 1.8 | `mud` 2.65, `dirt_flat` 2.36 fail (the streaks the user spotted); `floor_r2` 1.47, `stone_r2` 1.30 pass; fixtures: diagonal stripes 7.82, square grid 3.47, noise 1.11 |
| Direction, coherence | structure tensor: `sqrt((Jxx - Jyy)^2 + 4 Jxy^2) / (Jxx + Jyy)` | .25 | .25 | `mud` .46, `dirt_flat` .41 fail; `floor_r2` .07 passes; stripes 1.0 |
| Contrast cap (SC 2) | L p95 - p5; busy, mean step in L | 10; 1.0 | 14; 2.0 | `dirt_flat` 2.9; 0.45. The test scene's road 28.8; 4.39 (fails: too strong); at contrast x .45, 13.0; 1.97 |
| Colour cap (SC 2) | mean(max - min of RGB) | 12 | 12 | `dirt_flat` 10.5; `floor_r2` raw 18.2 fails ("too strong in colour") |
| Luminance (existing) | mean(L) at most 25 % | yes | yes | |
| Pair (board, warning) | RGB distance of two grounds' means at least 8, so the edge reads | | | |
| Figures (warning, SC 2) | figure range at least 4 x ground's, busy at least 3 x | | | quietest standing prop 75.8; 5.93 |

The 16-colour check does not apply to a ground (flattening makes many close colours). By these numbers the prototype's
dirt fails on direction, as the user found, so its source (`sunk_ward_stage_mud`) is not reused.

The kit's asset warnings (A16-A18, KT 1.5) cover files: no art along the chain, a wrong size, a missing mark. The look
checks above stay in the tool, since a pick is judged once.

The mix (T2) is drawn by `ground_compose` (EN 6), so the board's picture is the game's. Two fixtures in
`tools/tiles/fixtures/`, 40 x 22 cells after the mocks, the default ground outside the rects (x, then y, inclusive):

- `mix_outdoor.txt` (`ash`): `flagstone` 4-13, 2-8; `paved_road` 17-21, 3-10; `dirt_road` 19-22, 11-21 and 13-18, 13-14;
  `packed_dirt` 24-36, 11-18; `mud` 3-12, 11-18; `water` 5-10, 13-16; `scorched` 27-34, 3-7; `scrub` 1-2, 19-21 and
  37-39, 17-21.
- `mix_street.txt` (`soot_yard`): `flagstone` 14-25, 3-9 and 1-9, 5-9; `cobble_road` 0-39, 10-12; `tallow` 30-38, 4-9;
  `mud` 10-11, 3-9, 28-29, 3-9, 5-6, 13-15 and 32-33, 13-15; `moss` 0-39, 14-15 and 0-39, 20-21; `water` 0-39, 16-19;
  `iron_catwalk` 19-21, 13-21.

As built (the review of slice T, 2026-11-02): natural grounds step their gain down as built grounds do, toward about .17,
because real art cannot meet the natural caps at a fixed .55. `dirt_road` and `packed_dirt` were re-picked to meet the
colour cap of 12. Direction bins are centred on the axes, L and the broad-tone blur stay in floats, so calibration reads
`mud` 2.83, `floor` 2.84, `dirt_flat` 2.60, `floor_r2` 1.77, `stone_r2` 1.65, even noise about 1.1; broad tone `dirt_clean`
1.91, `dirt_flat` .37. The tools stay Pillow only. A tail check catches a repeated dark blotch.

## 7. Prompt template and batches

Gemini route (`tools/portraits/artqueue.py request`, a record beside each image, a `session_ref` chain from the base;
scale only, no pixelize). Slice T adds this to `ART_PROMPT_TEMPLATES.md` as "Ground texture"; "Tile" stays for walls and
the stage. It answers the user's complaint on the last batch: too strong in colour, too busy, dark patches that repeat.

```text
One large seamless ground texture for a dark fantasy tactical RPG, seen from directly overhead: a flat, straight-down orthographic view of the surface, drawn as a square texture of 128 x 128 pixels scaled up (visible square pixels, no anti-aliasing). It sits quietly behind small characters and must never draw the eye. Muted and low contrast: one main colour, {hex}, with at most 3 close shades of it. Even tone across the whole image: no lighter or darker areas, no dark patches, no stains, no blotches, no vignette, no lighting gradient. Fine even grain only, the same in all directions: no direction, no streaks, no diagonal marks, no brush strokes, no lines, no footprints, no tracks, no ruts, nothing that stands out or could be recognised when repeated. It repeats across a large map, so it wraps on all four edges with no seam, and has no grid, rows, tiles, checker or plaid; no shape lines up with the image's edges or with any other shape. No perspective, no horizon, no objects, no border, no frame, no text. Material: {material}.
```

Chain the rest from the base with "In the same pixel size, palette family and flat straight-down view as the first
image." A built ground replaces "Fine even grain only, the same in all directions" with "Small irregular stones of
different sizes, in no rows and no direction, with narrow joints only slightly darker than the stones".

| Batch | Ground (set) | {hex} | {material} |
| --- | --- | --- | --- |
| A1 | `ash` base, n 2 (`ashlands`) | #2a2622 | fine grey ash over dry ground, soft even dust |
| A1 | `scrub` | #2b2d22 | dry ground with sparse tiny flecks of dead grass, nothing taller than a few pixels |
| A1 | `dirt_road` | #39342e | hard trodden brown dirt with tiny pebbles |
| A1 | `packed_dirt` | #332e28 | pressed dry earth, smooth and fine |
| A1 | `paved_road` | #3e3f42 | worn grey paving stones |
| A1 | `flagstone` | #373636 | old grey-brown flagstones, chipped, of many sizes |
| A1 | `mud` | #231d18 | dark wet mud with small even pits |
| A1 | `scorched` | #1c1917 | burnt black earth with fine ash flecks |
| A1 | `water` | #161a1e | still dark cistern water, tiny even ripples, no highlights, no reflections |
| A3 | `soot_yard` base, n 2 (`sunk_streets`) | #1e1e23 | soot-black packed ground with fine grit |
| A3 | `cobble_road` | #2e3137 | damp rounded cobbles |
| A3 | `flagstone` | #38393c | soot-stained flagstones of many sizes |
| A3 | `mud` | #241f1b | black lane mud with small even pits |
| A3 | `moss` | #20261f | low damp moss, close and even, no grass, no blades |
| A3 | `iron_catwalk` | #303032 | riveted dark iron plates of many sizes, in no rows, faint rust |
| A3 | `tallow` | #2a251e | packed ground soaked with pale tallow, fine even mottling |
| A3 | `water` | #0e1018 | still black canal water, tiny even ripples, no highlights, no reflections |
| A2 | 2 mark sheets on magenta, small-props class (PROPS1-A3), straight down | - | Ashlands: `footprints`, `scuff`, `rut_short`; Sunk Ward: `footprints`, `scuff`, `drag_mark` |
| A2 | Ashlands `wall_face`, the "Tile" template's front elevation (SC 1.6) | - | rough basalt blocks |

Images: A1 10, A3 9, A2 3: 22, no batch above the warning line of about 10. The mocks list 8 grounds an area; `water` is
the Ashlands' ninth because `~` cells need a water ground (G7). Named, not scheduled: `sunk_undercroft` and
`sunk_foundry` (LOOK1 5.2), about 6 images each once their maps have layouts.

## 8. Slices (status `spec`; each ends with its gate)

T, E1 and K run in parallel (no shared files), then L, then D1: the first playable result, one real map with its ground
in flat colours. A1 and E2 run beside them. T2, then R1, the slice that needs the user's eyes (one review, patches included).

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| REGIONGROUND1-T | `flatten.py`, `groundcheck.py`, `cut.py --form ground`, repeated and pair boards, prompt template | game (tools) | logic | - | `tools/tiles/{flatten,groundcheck,cut,board,test_tiles}.py`, `README.md`, `fixtures/`; `docs/STRUCTURE.md`; `ART_PROMPT_TEMPLATES.md` |
| REGIONGROUND1-E1 | `aegis_ui::ground`: layers, tone, edges, cache, Gallery view, `ground_compose` | engine | logic | - | EN 6 |
| REGIONGROUND1-K | `grounds` kind, map fields, checks, lookups, plan | kit | logic | - | KT 3 |
| REGIONGROUND1-L | The region map draws a map's ground in both views; the location's `tileset` | game | logic | E1, K | `emberwake_gui/src/{ground.rs (new), region.rs, lib.rs}`, `SCREENS.md` 07, `GAME_BRIEF.md` |
| REGIONGROUND1-D1 | The 14 rows; `ashlands_warden_keep` ground rows (section 9) | game (data) | asset | L | `data/catalog/grounds.json`, `data/maps/maps.json` |
| REGIONGROUND1-A1 | Ashlands textures | game (art) | art | T | `art_src/regionground1/ashlands/`, review folder |
| REGIONGROUND1-T2 | The mix on the board through `ground_compose` | game (tools) | logic | T, E1 | `tools/tiles/{board.py, test_tiles.py, README.md, fixtures/}` |
| REGIONGROUND1-R1 | Place A1's picks; the user plays the map; numbers tuned (section 10) | game | asset, design | D1, A1, A2, T2, E2, W, S, PROPS1-A3a | `assets/tilesets/ashlands/ground/`, review folder |
| REGIONGROUND1-E2 | Random and placed patches, random marks | engine | logic | E1 | EN 6 |
| REGIONGROUND1-A2 | Mark sheets and the Ashlands wall face, cut | game (art) | art | T | `art_src/regionground1/marks/`, `assets/props/{ashlands,sunk_ward}/` |
| REGIONGROUND1-D2 | Mark rows; ground rows for `ashlands_cinder_gate`, `ashlands_ruined_courtyard`, `ashlands_ashmaw_camp` (layouts reviewed as a board first) | game (data) | asset | R1 | `data/catalog/grounds.json`, `data/maps/maps.json` |
| REGIONGROUND1-A3 | Sunk Ward street textures, with R1's tuned prompt | game (art) | art | R1 | `art_src/regionground1/sunk_streets/`, review folder |
| REGIONGROUND1-D3 | Sunk Ward maps' ground rows, A3's picks, `tileset` | game (data) | asset | A3, SUNKWARD1 | `data/maps/maps.json`, `assets/tilesets/sunk_streets/ground/` |
| REGIONGROUND1-C | Drop `floor` and `shallow_water` under `tilesets/ashlands`, and tests naming them | game | logic | D2 | `assets/tilesets/ashlands/`, those tests |
| REGIONGROUND1-X | Connector pieces for a pair, only if the user asks for one (not scheduled) | engine, kit | logic | R1 | later |

Slice L: `ground.rs` is glue: `plan` from the kit, each layer's `ground_dirs` through `Art::pick`, `Art::image` at 64 x 64
or 64 x 48, `draw` over the whole map's rect inside the map area's clip, before ground props. `region.rs`'s ground pass
skips `.` and `~` (and the tilted lips) when the map has a plan. Agent link: `state.ground` is `{"layers": n}`, or null.

Acceptance checks (E: EN 5; K: KT 2).

T (`test_tiles.py`, fixtures only):

1. A wrapped sine tile fails broad tone; after `flatten.py` it passes, its mean within 1 of the source's, its fine detail
   about gain x the source's, its seam score no higher.
2. Diagonal stripes and a one-way streak fail direction, a square grid fails peak, even noise passes; flatten does not
   make a failing direction pass; `--isotropic` lowers the direction scores on a streak.
3. A row-striped fixture passes seam and fails banding; a strong-colour fixture fails the colour cap; a high-contrast
   one the contrast cap; a pair 5 apart warns.

L (`emberwake_gui` tests, under both `mapview::VIEWS`, with a fixture layer):

1. A map with `ground: []` draws today's frame exactly (hash).
2. With ground: walking, `cell_at`, `path_to`, clicks, edge arrows and legend pans equal the same map without it; a click
   on a cell whose pixel shows the neighbour's ground walks to that cell. The MAPNAV1-U and WALK2 tests run unchanged.
3. `ground_composes` stays 1 over 100 frames with steps, a flag change and a marker change; a view flip composes once.
4. A wall and a ground prop draw over ground (pixel probes); the tilted block still sorts with the party.
5. A ground without art draws its `colour` within tone; `text_audit` clean at the four text sizes.
6. Journey golden, store resave and battle traces unchanged; `tools/fc.sh content check` clean.

D1: validation clean, A16 warnings only; the map's `rows` are byte-equal before and after.

## 9. The first map (D1): `ashlands_warden_keep`

Legend: `a` `ash`, `s` `scrub`, `b` `scorched`, `k` `packed_dirt`, `f` `flagstone`, `d` `dirt_road`, `m` `mud`, `p`
`paved_road`, `w` `water`. 48 x 30, every cell `a`, then paint in this order (x, then y, inclusive), later over earlier:

`s` 1-3, 1-2; 1-3, 24-28; 44-46, 24-27; 45-46, 9-12. `b` 1-3, 17-22. `k` 3-44, 9-12. `f` 15-32, 2-8. `d` 23-26, 20-23;
13-22, 20-21; 27-34, 20-21. `m` 19-29, 12-19. `p` 23-26, 24-28; 24, 29. Last, `w` on every `~` cell of `rows` (15 cells; G7
then holds).

That is the muster yard on packed dirt, the court on flagstones, mud around the cistern, a paved road at the gate, dirt
lanes to the forge and the storehouse, and scorched ground by the forge. `rows` are not touched; no `ground_patches`.

## 10. The review (R1)

Prep (planner, sessions muted): A1 on the board with its scores, the user picks the `ash` base and any re-rolls; then
1280 x 720 and 1920 x 1080 shots of the map, square and tilted, in
`C:/dev/localgen/review/look1/regionground1/index.html`, beside `mock_a_outdoor.png` and `dirt_flat.png`. The user plays it
for a few minutes in the Scene Runner. Ask:

- R1. Does ground stay behind the action, with no loop, lattice or direction? Which ground is too strong?
- R2. The edge: its ragged amount (`.62`), worn band and strays; the patches: too much or too little?
- R3. Is the art px right at this density (question 2)? Plus SC 5's asks.

The planner then records the tuned numbers in EN and the decision row; D2 and A3 follow.

## 11. Out of scope

Battle terrain and the stage's art (question 1); animated water; sound by ground; ground affecting movement, encounters
or AI; a map painter in the World Forge; a rasteriser for areas and paths; ground changing in play (EN 4.3 keeps the
door open); a quality setting (decision "Art scaling by asset type"); walls and building art (LOOK1-A1-A3 as narrowed,
PROPS1-A2); Iron Reaches and Tidewrack sets; `walled_town` and `gorge_ruins`, which stay as they are.

## 12. Open questions (recommended first)

1. Battle stage tiles (A4, LOOK1-L1's corner pieces): (a) stay as built for now; their textures go through `flatten.py`
   and `groundcheck.py` once T lands (pieces re-made by the existing `compose.py`, no code), and the user looks again
   after R1, a later row only if the lattice still shows; (b) the composer draws the stage too, now: one system and no
   lattice, but `stage.rs` changes, and terrain rules battle moves, so an edge that wanders 0.3 cell over terrain that
   matters needs its own review; (c) big textures only, keeping the pieces: not recommended, since pieces are composites
   of `v0` that no longer match a sampled texture, so they would be redone anyway. Recommend (a).
2. Ground art px: (a) 16 per cell, the boards' look: 1 art px per logical px, twice a sprite's px and whole at every whole
   scale; (b) 32 per cell, the sprites' own density: finer stones, four times the art memory, uneven pixel widths at 1920
   x 1080 (LOOK1-L1's finding). Both obey "never finer than the units". Recommend (a).
3. First real map: (a) `ashlands_warden_keep`: it exists and has the mock's features; the Sunk Ward set follows when
   SUNKWARD1 makes its maps (a departure from LOOK1 Q3, "the Sunk Ward first", only because no Sunk Ward map exists yet);
   (b) wait for SUNKWARD1. Recommend (a).
