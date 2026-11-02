# tools/tiles (LOOK1-T, REGIONGROUND1-T, -T2, -T3)

Tile and ground texture pipeline. Python 3.12 with Pillow only (decision "Art tools in Python, portable to Rust"); the ground boards also need cargo, for the engine's `ground_compose`. Gemini art is already pixel art, so every step is scale only
(no palette pass unless `--colours` is asked for). Run from this folder.

| Tool | Use |
| --- | --- |
| `cut.py SRC --size WxH --out DST` | `--reduce auto`, the default. Detects the pixel block size (largest b dividing 95 % of the colour runs, RGB step at most 12), samples block centres, trims a uniform border of at most one block; the result must be exactly W x H, else exit 1 (at block 1 px the message names `--reduce box`). |
| `cut.py SRC --size WxH --reduce box --out DST` | A box reduce of the whole image to W x H, no block detection: for images with no clean pixel block (the web route's 1024 px tiles report block 1 px). A source smaller than W x H is refused (no upscale). |
| `cut.py SRC --form F --out DST` | Instead of `--size`; always a box reduce in one step from the source, no upscale. Tiles take a square (at most 3 % off) and squash it when the size is not square: `stage` 64 x 48, `region` 32 x 32, `region_tilted` 32 x 24. Faces crop the centred band of their aspect across the whole width, no squash: `stage_face` 64 x 24, `region_face` 32 x 16, `region_tilted_face` 32 x 12, `region_tall_face` 128 x 48 (REGIONGROUND1_SCENE 1.6). `ground` is a square form of 256 x 256: the master of one ground texture over 4 x 4 cells. |
| `cut.py ... --colours N` | Off by default (the batches accept the board's colours warning). N from 1 to 256: at most N colours (16 for the board's check), each one a colour of the reduced image, no dither, no fixed palette or ramp. |
| `seam.py score PNG` / `seam.py fix PNG [--out DST]` | Wrap step mean over inner step mean (at most 1.5 accepted); `fix` replaces the edge columns and rows by the inner neighbour nearer the tile's mean. |
| `masks.py [N]` | Ragged corner masks 1-14 (bit 1 NW, 2 NE, 4 SW, 8 SE is upper), prints mask N. Library: `mask_grid`. |
| `compose.py --upper U --lower L --size WxH --out DIR` | U and L are PNGs or folders (their `v0.png`); writes opaque `m01.png`-`m14.png` (use `tilesets/<set>/<upper>/over_<lower>`). |
| `board.py SET_DIR --out DIR` | Tile sheets (3 x 3, 1x and 2x) with checks, piece sheets, the sample map square and tilted (1x-3x), `index.html` with `file:///` links. |
| `flatten.py SRC --out DST [--class natural or built] [--gain G] [--colour RRGGBB] [--isotropic] [--record JSON]` | Per channel, `mean + (T - low) x G`, `low` a Gaussian blur of the image tiled 3 x 3 (centre crop), sigma 5 at 64 px wide, 20 at 256. Natural gain from .55 down to .17, built from .45 down to .25, by .05 steps (the last to the floor) until broad tone, range and busy pass, else exit 1 (a re-roll, the last try still written). `--gain` takes no steps. `--colour` sets the mean per channel to the ground's colour in the spec's table, within half a level, by one whole-level shift per channel, so the pattern and the seam stay and the art's own tint no longer decides hue or value; the gain's steps score the coloured image. `--record` merges the class, gain and colour used into the art's record. A source over 4096 px a side is refused. `--isotropic` averages the 8 rotations and mirrors x 2.1 plus a speckle: a fallback on the user's say, never a default. |
| `groundcheck.py score PNG --class natural or built [--ground ID]` | Reduces to 64 x 64, prints the scores against the limits below, exit 1 on a failure. A size that is not a whole multiple of 64 prints a warning: its scores come from a rounded form and can differ. `--ground` applies the ground's named colour cap and the stones warning. |
| `groundcheck.py figures GROUND_DIR --art PATHS` | Warnings: a figure's range under 4 x a ground's, its busy under 3 x. `cistern`, `cobbles`, `scorch_mark` are not figures. |
| `groundcheck.py pair GROUND_DIR` | Every two grounds' separation (below), exit 1 when any two are too alike; the stones warning for a built ground. |
| `groundcheck.py read FRAME GROUND_PNG [--option shadow, pool or rim] [--colour RRGGBB]` | `read_contrast` and the ring step of one figure frame on a ground (below). |
| `board.py SET_DIR --ground --out DIR [--figures ROOT] [--wear X] [--props FILE]` | The set's `ground/<id>/v0.png`: each repeated over 16 x 10 and 32 x 20 cells at 1x-3x, no tone, with scores and pass or fail; each pair on a straight edge, a T crossing, a 1-cell lane and a lone cell (each through `ground_compose`, so no tile tool holds an edge of its own; pairs by the spec's order, lower first); each ground under the six class bodies, a crate, a tent and a wall face at 1x-2x; the figure warnings; `read_contrast` per body, ground and option. Also the mix below and the pairs' separation. `--layout`, `--set-colours`, `--wear` and `--props` as for `--mix`. |
| `board.py SET_DIR --mix --out DIR [--layout FIXTURE] [--set-colours] [--wear X] [--props FILE]` | One picture, `mix.png`, composed by the engine's `ground_compose` (EN 6), so it is the game's picture, random patches included: the set's grounds as a small map (`fixtures/mix_outdoor.txt` by default, `mix_street.txt` for the Sunk Ward; `ID X0-X1 Y0-Y1` rects, later over earlier), at 2x with nothing written over the map, and beside it the grounds, how far each sits from its table colour, the pairs too alike and the plain built grounds. `--set-colours` sets each texture's mean to its row's colour as `flatten.py --colour` does; off, the art shows as it is. A ground without art draws its row's colour. `--wear X` takes the table's wear times X, 0 to 3 (the table's own wear at 1). `--props FILE` draws props and units over the map at their own pixels (32 a cell), by feet, front last, so a ground is judged under what stands on it: `fixtures/mix_outdoor_props.txt` is the outdoor example, `prop FOLDER X Y` or `unit FOLDER X Y` with the foot in cells and FOLDER under `props/` or `units/` of the game's assets, then the kit's. A bad line, missing art, a foot off the map or past 256 lines is refused. A layout that is off its map, past 128 cells, or names a ground outside the table is refused, not cut; so is a texture over 4096 px a side. |

Sizes (kit checks A13-A15): region tiles 32x32 or 32x24, faces 32x16, 32x12 or 128x48, stage tiles 64x48, stage faces 64x24.
Ground masters 256x256.

The ground job (`board.ground_job`): the grounds the map uses, by the table's order then id, as the kit's `plan` lists them; a texture's offset comes from
the place in that list. `board.py` builds `ground_compose` once a run (a linked worktree in its own folder, as `tools/fc.sh`), then runs the binary per picture; a build
needing more than `tools/fc.sh tooltests`' limit fails it, so a fresh worktree runs `tools/fc.sh build` first. Rows live in `board.GROUND_ROWS` until the catalog has them (D1).

Checks on the board: seam at most 1.5, at most 16 colours, mean luminance at most 25 %.

One ground texture: `cut.py --form ground`, `seam.py` (fix, else re-roll), `flatten.py --colour`, `groundcheck.py score --ground`
(a failure: re-roll once naming it in the prompt, then show the user), then for the set `groundcheck.py pair`, `board.py --mix`
and `board.py --ground`.

Ground checks, scored on the 64 x 64 form with L in Rec. 709 on 0-255 (`REGIONGROUND1.md` section 6, `REGIONGROUND1_SCENE.md`
section 2):

| Check | Formula | Natural | Built | What it answers |
| --- | --- | --- | --- | --- |
| Seam | wrap step mean over inner step mean | 1.5 | 1.5 | an edge shows |
| Broad tone | stdev(L of the wrapped sigma 5 blur) / mean(L), % | 1.2 | 3.0 | the looping light and dark |
| Blotch | (mean(L) - lowest L of the wrapped sigma 2 blur) / stdev(L) | 1.4 | 1.4 | dark patches that repeat |
| Banding | max(stdev of row means, of column means) / stdev(L) | .30 | .30 | the plaid: rows and columns of tone |
| Direction, peak | gradient energy by angle in 8 bins over 180 degrees, one centred on each axis; largest over mean | 1.8 | 1.8 | the plaid's axis lines, and angled streaks |
| Direction, coherence | structure tensor, `sqrt((Jxx - Jyy)^2 + 4 Jxy^2) / (Jxx + Jyy)` | .25 | .25 | angled streaks |
| Contrast, range | L p95 - p5 | 10 | 14 | too strong against the figures |
| Contrast, busy | mean step in L | 1.0 | 2.0 | too busy |
| Colour | mean(max - min of RGB); named caps `dirt_road` 24, `packed_dirt` 20, `tallow` 14, each its mock colour's flat swatch (22, 18, 12) plus 2 for the grain's colour noise (real grain adds up to .4; `tallow` on the common 12 could never pass) | 12 | 12 | too strong in colour |
| Luminance | mean(L), % of 255 | 25 | 25 | too bright |

L and the blur stay in floats, unrounded from a master that is a whole multiple of 64: rounded to 8 bits, a quiet grain's
gradients pile up on four angles (even noise then scores 2.4 to 2.9 on peak), and a dark texture's broad tone gains about
.7 % from rounding alone. Direction keeps a bin on each axis: with a bin edge on the axis, the flattened plaid floor scores
about 1.7 and passes; centred, 2.76.

Blotch is a ratio, so no gain cures it. It must fail `dirt_clean` (1.52) and the old `mud` (1.66, 1.57 at gain .17), and pass
`dirt_flat` (1.28), `floor_r2` (.58), `stone_r2` (.55), `rubble_r2` (.52) and even grain (.38 to .51). A tail of the histogram
(p99.5 - p0.5 against p95 - p5) was tried first: three large patches sit inside p5, so it reads them as normal (1.45), and it
scores `dirt_flat` above `dirt_clean` (2.31 against 1.99).

Pair separation (`groundcheck.py pair`, the board, the mix): two grounds of a set clearly differ when either holds:

- Colour: RGB distance of the means at least 5. The closest neighbours in the mocks must pass (`cobble_road` / `iron_catwalk`
  5.5, `ash` / `scrub` 7.1); the alike A1 images (`dirt_road`, `packed_dirt`, `paved_road`, `scrub`) sit at 1.7 at most and must
  fail. This replaces the spec's earlier warning at 8, which the mock's own `ash` / `scrub` failed.
- Pattern: at least .40, the largest of |ln| of the ratio of busy, |ln| of the ratio of the scale of shapes (the lag where the
  wrapped autocorrelation of L falls to one half, so contrast does not move it) and the share of the gradient energy pointing
  another way (half the L1 distance of the direction histograms). Those A1 images score .01 to .06, and .17 at most against
  `flagstone`; the flat `ash` take scores 1.25 and up on busy.

Any two grounds of a set can meet on some map, so all of a set's pairs count. Pictures of one grain pass on colour alone; set
colours from the table and the A1 images all pass that way (0 of 36 too alike, against 28 of 36 as they came), with patterns
still alike at .17 at most. The score says colour or pattern, so the board shows which one is carrying a pair.

Stones (warning, `paved_road`, `flagstone`, `cobble_road`): at least 6 closed stones on the 64 x 64 form. The darkest 30 % of
the fine detail (L less its sigma 4 blur) is taken as joints; a stone is a region of the rest, by sides, wrapped, from 6 art
px to a quarter of the tile. A ratio of the image's own detail, so a flattened quiet road still counts. Real art: A1's
`paved_road` and `flagstone` 0 (plain grain), other A1 grounds 0 to 1, `dirt_flat` 2; `sunk_ward_stage_stone` 13, `_stone_r2` 50,
`_floor_r2` 48, their flattened forms 12, 50, 48.

Art pixels of 8 master px or more fail seam by construction (a score of about 2 at 8 px, 4 at 16): every wrap step is a
block's edge, while most inner steps fall inside a block. The Ground texture prompt's 128 x 128 pixels are 2 master px.

`read_contrast` (`REGIONGROUND1_SCENE.md` 3.2): |mean L of a figure's ring - mean L of the 2 px band of the frame around
it|. The board and `read` print the ring step beside it: the same difference per ring pixel against the band within 2 px of
it, because a dark outline and a bright robe cancel in the mean. What sets the number, for the Rust check to match:

- The frame is padded by `BAND` = 2 px on each side (w + 4 by h + 4), in floats with no rounding, at 1280 x 720's physical
  px: a sprite px is 1 px, a ground art px is 2.
- Solid is alpha at least 128. The ring is the solid pixels with a non-solid one among their 8 neighbours. The band is the
  non-solid pixels within 2 px (a square) of a ring pixel. Off the sprite counts as non-solid.
- The ground has phase 0 at the padded frame's corner: pixel (x, y) takes texture pixel (x // 2 mod 64, y // 2 mod 64).
- The marks' centre is `cx = w / 2 + BAND`, `cy = last solid row - 1`, the row in the padded frame. They are sampled at
  pixel corners: `d^2 = ((x - cx) / rx)^2 + ((y - cy) / ry)^2` with whole x and y, and alpha x (1 - d^2)^2 inside.
- Blending order, each a plain over in floats: `pool` (option `pool` only: colour, alpha 46, radii 22.4 and 11.2), `shadow`
  (every option: black, alpha 90, radii 14.4 and 5.76), `rim` (option `rim` only: colour at alpha 130 on the non-solid pixels
  with a solid one among their 8 neighbours), the sprite. The colour is a person's, (230, 190, 80).
- L is Rec. 709 of the floats.

The radii are on a square cell of 32 px (`CELL`) in both views. A tilted cell is 32 x 24 px, so if the game draws them on its
row height, the tall radii are 4.32 and 8.4: the rogue on flat `#1e1e23` moves from 7.79 to 7.59 under `shadow`, from 3.26 to
3.56 under `pool`, from 33.52 to 33.68 under `rim`.

Golden values (`test_golden_values_for_the_rust_check`): a 48 x 48 frame, solid on x 14-33 and y 8-45, (18, 18, 18) on that
rect's border and (70, 60, 80) inside, on a flat ground of (30, 30, 35); read_contrast, then ring step:

| Option | Flat ground | Checker ground, 20 and 60 by art px, 60 when (x + y) is odd |
| --- | --- | --- |
| `shadow` | 11.476758; 11.443010 | 20.833413; 20.790557 |
| `pool` | 16.708316; 16.432241 | 25.750239; 25.478856 |
| `rim` | 51.270821; 53.059785 | 58.338361; 60.013358 |

Test: `python -m unittest test_tiles` (temp dirs and fixtures only).

The pair check is meant for coloured textures: without `--colour`, flatten's different gains can move `busy` enough to pass a pair on pattern alone.
