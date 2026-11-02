# REGIONGROUND1: engine part (the ground composer)

Status: spec (2026-11-02), with the kit part `docs/specs/REGIONGROUND1_KIT.md` and the game part
`games/emberwake/docs/specs/REGIONGROUND1.md` (goal, grounds, art, slices, questions). Rows: REGIONGROUND1-E1, -E2.
Depends on: LOOK1-ME (`panes` compose and blit, done), PROPS1-E (`Art`, done). Nothing here waits for EXPLOREVIEW1-R:
both views are built.
Lane: logic
Touches: section 6.

The engine names no game and no ground: a layer is a texture and numbers. The kit holds what a ground kind is, the map
fields and their checks. The game holds its rows and art.

## 1. Engine: `aegis_ui::ground` (new)

A pure function from a cell grid to one opaque image, then a cache and a blit. No rule, save, walk or click reads it.

```rust
pub struct GroundMark<'a> { pub img: &'a Image }      // art at the job's density; see 3.4
pub struct GroundLayer<'a> {
    pub order: i32,                 // paint order, low first; unique in a job
    pub tex: Option<&'a Image>,     // texture at TEX_CELLS x cell px; None: flat `colour`
    pub colour: Rgba,
    pub wear: f32,                  // 0.0-0.3: darkening of the worn band on the ground below
    pub strays: bool,               // bits of this ground scattered just past its edge
    pub tone: f32,                  // 0.0-0.2: amplitude of the map-wide tone (2.2)
    pub patches: f32,               // 0.0-0.3: darkening of a random patch's core; 0 none
    pub sunken: bool,               // a rim on its edge (2.6)
    pub marks: Vec<GroundMark<'a>>, // random one-off marks (3.4)
    pub marks_per_100: i32,         // tries per 100 cells of this ground, 0-10
}
pub struct PlacedPatch { pub x: i32, pub y: i32, pub w: i32, pub h: i32, pub strength: f32 }
pub struct GroundJob<'a> {
    pub seed: u64,
    pub cols: i32,
    pub rows: i32,
    pub cell: (i32, i32),           // art px per cell: CELL_PX, or (16, 12) tilted
    pub cells: &'a [u8],            // layer index per cell, row by row
    pub blocked: &'a [bool],        // cells a patch or mark may not reach (walls)
    pub layers: &'a [GroundLayer<'a>],
    pub placed: &'a [PlacedPatch],
}
pub fn compose(job: &GroundJob) -> Image;             // cols x cell.0 by rows x cell.1 art px, opaque
pub fn draw(p: &mut Painter, key: GroundKey, r: Rect, job: impl FnOnce() -> Image); // cache, section 4
pub fn soft_ellipse(rx: i32, ry: i32, col: Rgba, alpha: u8) -> Image; // alpha x (1 - d^2)^2, d the distance over the radii; 0 at and past them
pub fn ground_composes() -> u64;                      // cache misses that composed; tests and timings read it
pub fn ground_expands() -> u64;                       // physical copies made
```

Off-map cells clamp to the nearest edge cell, so an edge never curves at the map's border. A job with empty `cells`
composes nothing.

### 1.1 The pixel-size cap

`CELL_PX` is 16 art px per cell (`TEX_CELLS` 4, so a texture is 64 x 64 square, 64 x 48 tilted), the density of the
reviewed boards. A ground art px is then 1 logical px: 2 physical px at 1280 x 720, 3 at 1920 x 1080, 6 at 3840 x 2160,
always a whole number at a whole scale, and twice a unit sprite's art px (48 px over 24 logical), so ground is never
finer than the units on it (decision "Art scaling by asset type"). The cap's limit is 32 per cell (the sprites' own
density). All section 2 numbers scale with `cell.0 / 16` if the constant moves (game part, question 2).

## 2. The method (from `blend_proto2.py` to `blend_proto5.py`; "art px" at 16 per cell)

### 2.1 Noise

`N(step, ch)`: value noise on a lattice of `step` art px (`step_y` in the tilted form); each lattice point's value is
`splitmix64(seed ^ ch ^ lx << 20 ^ ly << 40)` mapped to 0..1, a pure function of those inputs with no RNG state; the
value at a pixel is Catmull-Rom bicubic over the 4 x 4 lattice points (the prototype's `BICUBIC` resize), clamped to
0..1. `seed` is the job's. No game RNG stream is touched, so traces, saves and journeys cannot move.

| Field | Formula | Used by |
| --- | --- | --- |
| Edge noise `n` | `.6 N(10, 1) + .4 N(4, 2)`, one per job, shared by all layers | 2.3 |
| Stray noise `s` | `N(3, 3)` | 2.4 |
| Bite noise `b` | `N(3, 4)` | 2.4 |
| Patch noise `q` | `.7 N(14, 5) + .3 N(5, 6)` | 3.1 |
| Tone `t` | `.7 N(48, 7) + .3 N(20, 8) - .5` | 2.2 |

Tilted steps (x, then y as round(step x .75)): 10/8, 4/3, 3/2, 14/11, 5/4, 48/36, 20/15. The edge noise is shared so
that two layers that end on the same cell boundary end on the same contour, with no sliver of a third ground between.

### 2.2 Sampling and tone

A layer's colour at art pixel (ax, ay) is its texture at `((ax + ox) mod tw, (ay + oy) mod th)`, the offset from
`splitmix64(seed ^ layer index)`, times `1 + tone x t(ax, ay)`. It is sampled by art pixel, never by cell, so no cell
edge exists in it; `tex` None gives `colour` with the same tone. The tone is map-wide and has no period: it is what
carries the broad light and dark that the textures must not (game part, section 4). `tone` .13 is the prototype's
(about +/-6 %).

### 2.3 Edges and order

Paint low `order` first. For each layer k, in order:

- `U_k`: the cells whose layer has order at least k's (a lower ground runs under the upper ones, so the base never
  peeks between two upper grounds).
- `bl_k`: `U_k` as 0..1 per art pixel, blurred by three box passes of width 11 x, 11 y (tilted: 11 x, 9 y), standard
  deviation about 5.5 art px, as PIL's `GaussianBlur(5.5)` does. Running sums in integers, limited to the layer's
  bounding box plus 18 art px. The lowest layer has no field: it fills the image.
- `v_k = bl_k + (n - .5) x .62`, and `m_k = v_k > .5`.

The top ground of a pixel is the highest k with `m_k`. The largest push past a cell edge follows from `bl`: the noise
term is at most .31, so an edge pixel has `bl` in (.19, .81): at most 5 art px (0.3 cell) from the cell boundary, 6 in
checks. A 1-cell lane keeps going: at its centre `bl` is about .85, so `v` is at least .54. A lone cell may read small
(`bl` about .73 at its centre) and that is accepted.

### 2.4 Worn band, strays and bites

For a pixel with top ground g:

| Piece | Where | Colour |
| --- | --- | --- |
| Worn band | for layers k above g, the largest `wear_k` with `v_k > .34` | g's colour x (1 - wear): taken once, never stacked |
| Stray | a layer k above g with `strays`, `bl_k` in (.16, .42) and `s > .82` | k's colour x .92; replaces g's, no wear |
| Bite | `v_g` in (.5, .56), `b > .7`, and g is not the lowest layer | the colour of the ground under g at this pixel x .95 |

Order: base colour, worn band, stray, bite. A stray only comes from a layer whose row says so (stones off a road, none
off moss).

As built (REGIONGROUND1-E1): `U_k` holds every higher rank, so where two grounds meet a middle one's `bl` reads the same
with none of its cells near, and the table read literally shows a third ground between them. So these three read a layer's
own cells' share, `own_k = bl_k - bl_(k+1)` (exactly 0 with none of its cells in reach; equal to `bl_k` on the top rank
and on two layers, whose output is unchanged). Worn band: the largest `wear_k` with `own_k + (n - .5) x .62 > .34`, so a
ground's wear fades in with its share. Stray: `own_k` in (.16, .42). Bite: the ground under g with the largest share
(`1 - bl_1` for the lowest).

### 2.5 Crispness

Every test is a threshold on art pixels and colours are never blended between grounds, so an edge is hard at art
pixels (decision "Ground blends"). There is no anti-aliasing and no alpha in the output.

### 2.6 Sunken layers

A `sunken` layer gets a dark rim on the side of its neighbour: an art pixel with top ground g sunken whose north pixel
has another top ground takes g's colour x .55 on the first art row, and in the tilted form also x .7 on the next two;
each other side takes .7 on one art px. It replaces EXPLOREVIEW1 3.2's cell-aligned lips on a map with ground; those
lips stay for a map without.

### 2.7 Determinism

`compose` is a pure function of the job. Float steps are add and multiply only, no libm; the box passes are integers.
As built (REGIONGROUND1-E1): add, multiply and IEEE division (integer sums over a constant, lattice fractions), all
correctly rounded; no libm.
The output does not depend on the view's scale, the camera, the frame or the party. Nothing it makes reaches a save.

## 3. Patches and marks (slice E2; E1 takes the fields and ignores them)

Dark patches and one-off marks are a layer of their own, never in the texture, so they never repeat with it.

### 3.1 Random patches

For a ground g with `patches` above 0:

1. Candidates: the 4-connected groups of art pixels with `q > .70` and top ground g.
2. Other-ground field `o_g`: the box-blurred (as 2.3) mask of cells that are not g or are `blocked`.
3. A group is dropped whole when it holds any pixel with `o_g > .05` (it would reach a road or another ground: about 9
   art px of clearance), or when its size exceeds `PATCH_CAP` 260 art px (about one cell; 195 in the tilted form). It
   is never clipped.
4. A kept group of size a has `f = 1 - .45 a / PATCH_CAP`: the larger, the fainter.
5. Its pixels with `q > .74` are the core, x (1 - patches x f); the rest the rim, x (1 - .45 patches x f). (.20 and .09
   are the prototype's: `patches` .20 gives them.)

Size is capped, so a large cluster cannot occur at random whatever the noise.

As built (REGIONGROUND1-E2): the cap scales with the cell's area (`patch_cap`: 260 square, 195 tilted, 1040 at 32 x 32). A
`patches` past `PATCHES_MAX` .3, below 0 or NaN counts as 0. `patch_groups(job)` returns every candidate whole with its
`kept` flag, the one code path `compose` draws from, so a test or tool reads what was dropped.

### 3.2 Placed patches

Large ones exist only where a map asks (`ground_patches`, kit part 1.2). In the rect's art px, with centre (cx, cy) and
half sizes (rx, ry), `e = 1 - ((ax - cx) / rx)^2 - ((ay - cy) / ry)^2`; the shape is `e + (q - .5) x .6 > 0`, the core
`e + (q - .5) x .6 > .18`, darkened x (1 - strength) and the rim x (1 - .45 strength). The shape stays in its rect. The
kit's check G8 keeps the rect and a ring of one cell on one ground, so it needs no drop rule; `compose` ignores one that
breaks it (the check refuses it first).

As built (REGIONGROUND1-E2): `ax`, `ay` are pixel centres (+.5), so a shape is symmetric in its rect. The outward push is
`min((q - .5) x .6, 2 x inside)`, `inside = 1 - max(|ax - cx| / rx, |ay - cy| / ry)`: the formula read literally is stopped by
the rect's side wherever `q` is above .5, a straight cut up to a third of a side, which is the clipped look the rule forbids;
now the shape closes before the side. Only the outer tenth of a half size differs. The ring is cut off at the map's edge, as
G8's is. `compose` takes a strength of 0-.3 (G8's .02-.25 is narrower).

### 3.3 Paint order

After section 2: random patches, placed patches, marks. Each is a multiply on the composite, or the mark's blend.

### 3.4 Random marks

One-off marks (footprints, ruts, scuffs) are small transparent images, placed under the patch rules:

- Tries for ground g: `marks_per_100 x (its cells) / 100`; try i takes its position, its mark and a left-right flip
  from `splitmix64(seed ^ g ^ i)`. There is no rotation (rotated pixel art breaks up).
- A try is dropped whole when any opaque pixel falls on another top ground, has `o_g > .05`, or overlaps a kept patch
  or an earlier mark.
- A kept mark blends at alpha 60 %, so it stays quiet.
- A mark's image is the art at the job's density: the caller reduces a 32-per-cell file by half (box, alpha thresholded
  at 50 %). At most 32 x 32 art px after reduction (2 x 2 cells): larger is refused by A18.

A mark placed on purpose (ruts along a road) is a PROPS1 `ground` prop in the map's `props`: no new field.

As built (REGIONGROUND1-E2): g in the hash is the layer's index in the job's list, shifted (`seed ^ g << 32 ^ i`), so one
ground's try i is not another's try i + 1. A pixel with any alpha holds its place and blends at alpha x 60 %. A placed patch
counts as a kept patch. A mark always fits the map (the position is drawn inside it), so a map's border never clips one. Art
that is larger than 32 a side, torn or fully transparent, or a `marks_per_100` outside 0-10, draws nothing.

Limits (REGIONGROUND1-E2): `PLACED_MAX` and `MARKS_MAX` (a layer's list) are 64 each, the kit's 16 and 4 being narrower; past
either, or with `blocked` shorter than the grid, a job composes nothing, like the size limits. `ground_compose`'s job file
refuses all of these with a message rather than ignoring them, a placed patch that breaks the ring rule included.

## 4. Cache, memory and states

### 4.1 Two levels, like `panes`' compose-and-blit

| Level | Key | Holds | Kept |
| --- | --- | --- | --- |
| Art image | `GroundKey`: map id, content hash, cell size | the `compose` output (scale free) | 4, oldest dropped |
| Physical copy | art key plus physical size | the art image expanded by whole-pixel nearest to physical px (`v.rect(r)`), opaque | 1 |

The content hash covers the job's cells, legend rows and each texture's identity (`panes::picture_id`, made
`pub(crate)`). `draw` copies the physical copy's rows inside the clip, as `panes::blit` does. No per-pixel blend, so a
The content hash covers the job's cells, legend rows and each texture's identity. As built (REGIONGROUND1-E1): every
pixel of each texture, not `panes::picture_id` (which stays private): real content, so no address, about 16 KB a texture,
hashed by `GroundKey::of` when the job changes, never each frame. `draw` copies the physical copy's rows inside the clip, as `panes::blit` does. No per-pixel blend, so a
hit costs less than today's per-cell tile loop. A map whose physical copy would pass `PHYSICAL_MAX` 64 MB draws the art
image through `Painter::image` instead (nearest, per frame; slower, never wrong). No shipped map reaches it. As built
(REGIONVIEW1): the map zoom reaches it (Warden Keep, square view, from 2.25x at 1080p and from 1.25x at 4K), so that path now
copies only what shows (`blit_scaled`, the same pixels as `Painter::image`).

### 4.2 Memory (48 x 30 cells, the largest shipped map)

| Window | Scale | Physical copy, square | Tilted (x 12 rows) | With 4 art images (1.5 MB each) |
| --- | --- | --- | --- | --- |
| 1280 x 720 | 2 | 1536 x 960: 5.9 MB | 1536 x 720: 4.4 MB | at most 11.9 MB |
| 1920 x 1080 | 3 | 2304 x 1440: 13.3 MB | 2304 x 1080: 10.0 MB | at most 19.3 MB |
| 3840 x 2160 | 6 | 4608 x 2880: 53.1 MB | 4608 x 2160: 39.8 MB | at most 59.1 MB |

Reduced textures cost 16 KB each; the 256 px files stay in `Art`'s cache at 256 KB each (about 2 MB a map).

### 4.3 What recomposes

| Event | Art image | Physical copy |
| --- | --- | --- |
| A frame, a step, the camera moving, a pan | hit | hit |
| A marker or prop changing with a flag, a fight, a talk, a save, a load | hit (they draw on top each frame) | hit |
| Window resize | hit | one new copy, the old one freed at once |
| View flip (the Developer select, F7) | hit if composed before, else one compose | one new copy |
| Entering another map | hit if among the last 4, else one compose | one new copy |
| A data or art edit in a running build (a Forge save, `aegis content`) | new hash: one compose | one new copy |

Maps do not change ground in play today: `rows` and templates are static. A future map state that changes ground is a
new grid and so a new content hash: one whole compose, no partial redraw, and the old image stays among the 4, so
flipping back costs only the copy. Its data is out of scope.

Budget in a release build for a 48 x 30, 8-layer job: compose at most 100 ms, a copy at most 20 ms at 4K. E1 records both
in `docs/EFFICIENCY.md`. It runs once on the map's first frame, so it never touches walking.

## 5. Acceptance checks (rules, not pictures)

E1 (`engine/crates/aegis_ui/tests/ground.rs`, fixtures of flat-colour and two-tone layers):

1. Same job twice: identical bytes. Another seed: different. A 4:3 cell gives a 4:3 image with the same rules.
2. Edge bound: on a straight boundary over 30 cells, no upper pixel is more than 6 art px past the cell edge and no
   lower pixel more than 6 inside it; the boundary is not straight (at least half its columns are off the cell edge).
3. A 1-cell lane over 30 cells has upper pixels in every column along its centre.
4. No cell lattice: with one layer and one texture of two tones, no output row or column equals the row or column one
   texture period away once tone is above 0; with tone 0, the output is the texture by art pixel with no break at cell
   edges (sampling by pixel).
5. Tone has no period: `t` differs from `t` shifted by 64 art px somewhere on every 32-cell row; its range is within
   +/-(tone / 2).
6. Order: a lower ground under two meeting upper grounds shows no pixel along their shared boundary's centre line.
7. Worn band never stacks: two layers ending on one boundary darken a lower pixel at most once. `wear` 0 and
   `strays` false give no pixel outside the two textures' colours x tone. A stray is at most 6 + 6 art px past an edge.
8. Sunken rim: a pixel with a sunken top ground under a non-sunken one is darker than its plain colour.
9. Cache: 100 `draw` calls with one key compose once and expand once; a resize expands again with no compose; two
   views compose twice; a blit equals the same image drawn through `Painter::image` at a whole scale.
10. `specimens`: the Tiles page's new Ground view draws at 1x-3x, square and tilted, with no text cut (`text_audit`).

E2:

1. Every kept random patch's pixels are all of one ground, at most `PATCH_CAP` in size and with `o_g` at most .05; every
   dropped candidate has no darkened pixel at all (dropped whole). Fuzz 20 seeds on a road-and-yard fixture.
2. Fainter as it grows: among kept patches, darkening never rises with size.
3. `patches` 0 leaves every pixel of the ground unchanged. A placed patch exceeds `PATCH_CAP` and draws inside its rect.
4. A mark over another ground or a kept patch is dropped whole; a kept mark's opaque pixels are all on its own ground.

## 6. Touches

| Row | Files |
| --- | --- |
| E1 | `engine/crates/aegis_ui/src/{ground.rs (new), panes.rs (`picture_id` visibility), specimens.rs, lib.rs}`, `src/bin/ground_compose.rs` (new), `tests/ground.rs` (new), `engine/README.md`, `docs/EFFICIENCY.md` |; `engine/crates/aegis_ui/src/{ground.rs (new), specimens.rs, lib.rs}`, `src/bin/ground_compose.rs` (new), `tests/ground.rs` (new), `engine/README.md`, `docs/EFFICIENCY.md` |
| E2 | `engine/crates/aegis_ui/src/{ground.rs, bin/ground_compose.rs}`, `tests/ground.rs`, `tests/fixtures/ground/` (two new pins), `docs/EFFICIENCY.md` |

`ground_compose`: `cargo run -q -p aegis_ui --bin ground_compose -- JOB.json OUT.png`. The job names cells, a legend of
layer fields with texture paths, cell size and seed. It is the one implementation, so the review board's picture is the
game's picture (`aegis_cli` has no `aegis_ui` dependency, so it stays a bin of `aegis_ui`).

`aegis_ui::autotile` is not used by the generated edge: its `corner_pieces` serves opt-in connector pieces (game part,
slice X) and its `block_edges` the walls, as today.
