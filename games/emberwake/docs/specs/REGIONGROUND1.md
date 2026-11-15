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
| `packed_dirt` | Packed dirt | 50 | land | #352c23 | .10 | no | .13 | .20 | natural, .55 |
| `dirt_road` | Dirt road | 60 | land | #3c3126 | .10 | no | .13 | .15 | natural, .55 |
| `flagstone` | Flagstones | 70 | land | #373636 | .10 | yes | .08 | 0 | built, .45 |
| `cobble_road` | Cobbled road | 78 | land | #2e3137 | .10 | yes | .08 | 0 | built, .45 |
| `paved_road` | Paved road | 80 | land | #3e3f42 | .10 | yes | .08 | 0 | built, .45 |
| `floorboards` | Floorboards (`REVEAL1`, `PR5-54`) | 72 | land | #3a2c22 | .06 | no | .08 | 0 | built, .45 |
| `iron_catwalk` | Iron catwalk | 90 | land | #303032 | .06 | no | .08 | 0 | built, .45 |

Marks (slice D2; `marks_per_100` 2): `ash`, `packed_dirt` take `footprints`, `scuff`; `dirt_road` takes `footprints`,
`scuff`, `rut_short`; `soot_yard`, `mud`, `cobble_road` take `footprints`, `drag_mark`. Moss is the Sunk Ward's green:
no ground there is grass.

**Pipeline update (`2026-11-14`, user ruling `AO-04` / `P-UI-06`)**: All remaining ground, interior-wall (`interior_plaster`, `interior_stone`), and boundary (`drain_brick`, `blind_tenements`, `gorge_rock`) texture generation (`A1..A3`, `floorboards`) moves to **Google Flow** (same workflow as act maps and landmarks) rather than paid Gemini API batches; generic material sets promote to `kits/rpg/assets/tilesets/` under `docs/archive/specs/ASSETORG4.md`. `PR5-54` seeded `floorboards` (`order: 72`, `indoor: true`, `#3a2c22`) with a copied `packed_dirt` placeholder at `kits/rpg/assets/tilesets/common/ground/floorboards/v0.png` (`PR5-56` moved it from the game layer), queued for Flow replacement in `AI-13`; `PR5-59` (`REGIONVIEW2`) applies `AO-09`/`AO-10` visual upgrades (wandering borders, `PATCHY` ash/moss, stone water rim, wall/prop polish, opt-in `dim_ambience`).

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
| A1 | `dirt_road` | #3c3126 | hard trodden brown dirt with tiny pebbles |
| A1 | `packed_dirt` | #352c23 | pressed dry earth, smooth and fine |
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

As built after the first A1 run (2026-11-02): grounds are no longer chained and the long template above gives way to a short
material-first prompt (`queue_rg1_probe.py`'s form), because all grounds came out alike. Colour is set in code from the table
above (slice T3), the dirt grounds keep their mock colours with a named colour cap, and a pair check fails neighbours that do
not clearly differ. A prompt is tested on 2-3 images before a batch (decision "Efficiency pass").

### `REVEAL1` Indoor Rooms, Roofs & Boundaries (`PR5-54` landed `1e08cb16`)

- **Indoor zones (`maps.json` legend / zone)**: zone entries take `indoor` (`bool`) and optional `wall_set` (validated for wall-set paths; indoor rooms use `floorboards` `order: 72`, `#3a2c22`).
- **Roof reveals (`PROPS1` layer + `roof_of`)**: `layer: "roof"` props carry optional `roof_of` naming an indoor zone ID (`R1` unknown/missing zone errors; `R2` requires non-alpha placement coverage); stepping into that zone smoothly fades every matching roof (`180 ms`) and hides covered NPC badges (`REGIONVIEW2`).
- **Boundaries & `REGIONVIEW2` (`PR5-59`)**: perimeter occlusion uses reusable `boundary` prop sets; `REGIONVIEW2` adds EN 2 wandering borders, stone water rims, `PATCHY` ash/moss under-floor overlays, free-standing `sort` `#` floor draw, thin base contact lines (zero cast oval shadows), and opt-in `dim_ambience` + torch pools (`AO-09`/`AO-10`).

## 8. Historical Ashlands Slices & Resolved Q&A

Ashlands pilot slice table (`T`/`E1`/`K`/`L`/`D1`/`R1`), `D1` inspection notes, and resolved `Q1`–`Q4` live verbatim in [REGIONGROUND1_HISTORY.md](../../../../docs/archive/specs/emberwake/REGIONGROUND1_HISTORY.md).
