# REGIONGROUND1: the scene on the ground (tall walls, hierarchy, reading)

Status: spec (2026-11-02), the user's answers from the test scene
(`C:/dev/localgen/review/look1/region_mock/scene_walls_units.png`, `scene_tall_walls.png`). Part of
`games/emberwake/docs/specs/REGIONGROUND1.md` (its rows and order; split for size only).
Depends on: EXPLOREVIEW1-L (done); per row, section 5
Lane: logic (W, S), art (PROPS1-A3), review with the user (REGIONGROUND1-R1)
Touches: section 5. Presentation only: no rule, save, sim, trace or journey changes, and no parity check retires.

## 1. Tall walls

The user: walls about 48 px tall at 720p, as tall as a unit, in place of EXPLOREVIEW1's short face. This is the tilted
view's block. The square view is unchanged (LOOK1 5.3's 8 px face, no lift); question 1 asks whether the answer also
settles EXPLOREVIEW1-R.

### 1.1 Geometry (`mapview.rs`)

| Piece | EXPLOREVIEW1 today | Tall |
| --- | --- | --- |
| `FACE_H`, `BLOCK_LIFT` (logical) | 6, 6 | 24, 24: a unit's height (`SPRITE`) |
| Face, for wall cell (x, y) | cell's bottom 6 px | from `cell.y + 12 - 24` to the cell's bottom edge: 16 x 24 |
| Cap | the cell lifted by 6 | the cell lifted by 24 (16 x 12), its bottom on the face's top |
| Block's reach above its cell | half a row | 2 rows |
| Rows drawn below the area | 1 more | 2 more (the lifted cap reaches 2 rows up) |

`cap_rect`, `block_rect` and `face_rect` keep their formulas in terms of the constants; `face_rect` becomes
`cell.y + cell.h - FACE_H`. A prop box without art is raised by the same `BLOCK_LIFT`.

The map's border on the south row and the east and west columns, and the margin past those edges, stays short (`LOW_FACE`
6, `LOW_LIFT` 6, today's numbers): otherwise the lowest two rows of every map, and its side exits, would sit behind it. The
north row and its margin are tall: they hide nothing.

A wall cell under a standing prop that has art draws no block at all: the art is the height, and a lifted cap would show
behind a roof.

### 1.2 The sorted pass

The sort is unchanged: a block's foot y is its cell's bottom edge, at equal feet block, prop, marker, party (decision
"EXPLOREVIEW1-L click areas and sort"). A taller block does not change who is in front of whom, only how much it covers.

### 1.3 What a wall hides

A wall covers the rows north of its own cell, and only those. A unit is 24 tall, and tilted rows are 12:

| Unit's row, against the wall cell's row r | Covered |
| --- | --- |
| r - 1 (touching, north) | all of it |
| r - 2 | its lower half: the head shows |
| r - 3 and further | nothing |
| South of the wall | nothing: the unit stands in front, its head over the face |

On shipped maps with art on hand today (probe 2026-11-02, under the 1.1 rules):

| Map | Open cells | All hidden | Half hidden | Anchors hidden |
| --- | --- | --- | --- | --- |
| `ashlands_warden_keep` | 1028 | 38 | 38 | none |
| `ashlands_ruined_courtyard` | 1215 | 0 | 0 | none |
| `ashlands_ashmaw_camp` | 1185 | 0 | 0 | none |
| `ashlands_cinder_gate` | 489 | 41 | 38 | `camp_chest`, `ruin_chest` (half) |
| `gorge_ruins` | 152 | 11 | 4 | none |
| `walled_town` | 90 | 12 | 6 | `npc_a`, `npc_b` (half) |

Without the short border the Ashlands exits (`west`, `east`) and several chests would be wholly hidden. The warden keep's
38 and 38 lie north of the buildings that have no art yet: art removes those blocks (1.1), though a tall roof then hides
in the same way.

### 1.4 Outlines

- The party: as today (PROPS1 4.5, `props::party_hidden`), now true on rows r - 1 and r - 2.
- A marker hidden by a block, new: an interactable must never be lost, and moving an anchor would change journeys. A
  person gets `Art::body_outline`, an icon `Image::outline`, in its `marker_style` colour, drawn in the party outline's
  pass. This overrides PROPS1 4.5's "NPCs get no outline", whose reason (A10 keeps anchors clear) does not cover walls.
- Kit check A19 (warning, `REGIONGROUND1_KIT.md` 1.5) lists such anchors, so a new map avoids them.

### 1.5 Clicks and walking: unchanged

`rows` still decide walking and `MapGeom::cell_at` still decides the cell. Consequences, kept on purpose:

- A click on the top half of a face or on a cap lands on a cell north of the wall: if that is open, the party walks
  there and stands behind the wall. A click on the face's bottom half is on the wall's own cell: "You can't get there.".
- A person hidden behind a wall still takes clicks on their own cell, and the legend, edge arrows and pans find them.
- Keys, WALK2 and the talk camera are untouched. All MAPNAV1-U tests run unchanged.

### 1.6 The face's art

| Form | Art px | Logical | Source |
| --- | --- | --- | --- |
| `wall_face`, tall | 128 x 48 | 4 cells wide, 24 tall | a square image's centred band across its whole width (`cut.py --form region_tall_face`) |

Drawn by column: slice `(x mod 4) x 32` of the file, 32 x 48, into the 16 x 24 face, so no per-cell repeat shows. It is
prompted as the old flat front elevation (LOOK1-A; the "Tile" template), wrapping left to right. Fallback when a set has no
tall file: the set's 32 x 12 or 32 x 16 face, or `common_stage`'s, stacked to 24 tall by repeating; never stretched. A
face is a figure, not ground: it is judged by section 2's figure side, and its detail may be stronger. Ashlands needs
one (`rough basalt blocks`, slice A2); LOOK1-A1-A3's faces take the new form. The battle stage's walls are out of scope.

### 1.7 Checks (W; `mapview.rs`, `props.rs`, `region.rs` tests, tilted view)

1. `cap_rect`, `face_rect`, `block_rect` by the 1.1 table; a south, east or west border cell, and a margin cell past
   them, draw short; a north one tall.
2. A unit on r - 1 sorts before the block and is covered; a unit south of it sorts after; a wall cell under a prop with
   art pushes no block, one under a prop without art does.
3. `party_hidden` is true at r - 1 and r - 2, false at r - 3 and south. A marker at r - 1 and r - 2 draws its outline
   (pixel probe in its colour), one at r - 3 does not.
4. Walking: `path_to`, `is_open` and click results equal those of today's constants over a 48 x 30 sweep (rule 1.5).
   A click on the top half of a face walks to the open cell north; on the bottom half logs "You can't get there.".
5. A block whose cell lies 2 rows below the area still draws its cap into it (the rows rule).
6. Square view frames equal today's (hash); journey golden, store resave and battle traces unchanged. These tests take
   the new numbers: EXPLOREVIEW1 check 4 and `mapview.rs`'s `face` assertion, an intended change named in the commit.

### Storeys and how walls sit with the props (user 2026-11-02)

- A wall face is one storey: 24 logical px high (48 px at 720p), its art 128 x 48, repeating sideways and upward, so
  it wraps on all four sides. Each wall or plain structure has a height in storeys (default 1); a taller one stacks the
  same face. Smaller pieces looped several times are not used: a mark in a 12 to 16 px piece repeats that often, the
  same looping the user flagged on the dirt. A later cap band and foot strip may break up a tall wall. Proper buildings
  (the tavern, the keep) stay whole props (PROPS1-A3b), not stacks.
- Walls may stand out, but must not look out of place beside the props. They share the props' light from the upper
  left, dark outline weight, pixel size and palette family, and sit within the props' saturation range; ground stays
  below both. Wall art is prompted with a prop sheet as its reference, and `board.py --ground` shows each wall face
  beside today's kit props and a unit, with a colour and contrast read of the wall against the props. The user judges
  this at R1.
- Region boundary walls (the map's outer edge) are their own art and may take a different style: cliffs, thick
  hedges, ruined ramparts, fog walls. The prop rule above covers structures inside the map, not the boundary. A map's
  boundary art is a set of its own in the tile set, so a location can choose it, and it keeps the short south and side
  borders of this section.

## 2. Hierarchy

The user: "the unit, props and buildings should pop out way more than the terrain, even with layering."

Rule: every ground stays well below every figure (unit, standing prop, building, wall face) in contrast, busyness and
colour. Roads and cobbles are grounds. A road's stones may show, but quietly.

The test scene's cobbled road fails it: it is too strong. Its numbers, with today's figures (64 px form for ground,
opaque pixels of the art for figures; probe 2026-11-02):

| Art | L range (p95 - p5) | Busy (mean step in L) | Colour (mean max - min) |
| --- | --- | --- | --- |
| Target dirt, `dirt_flat` | 2.9 | 0.45 | 10.5 |
| The scene's road (`floor_r2` muted) | 28.8 | 4.39 | 5.5 |
| The same at contrast x .45 | 13.0 | 1.97 | 5.5 |
| Units: median; lowest | 158; 93 | 23.8; 20.1 | 42.3; 24.2 |
| Standing props: the quietest (`boulder`) | 75.8 | 5.93 | 12.3 |

The check, as limits a ground must meet (natural; built), replacing the contrast row of the main spec's section 6:

| Check | Natural | Built | From |
| --- | --- | --- | --- |
| L range | 10 | 14 | at most about a fifth of the quietest standing figure |
| Busy | 1.0 | 2.0 | at most a third of the quietest standing figure, a tenth of a unit |
| Colour | 12 | 12 | at most half of the least coloured unit |

So built grounds take gain .45 (not .75) by default in `flatten.py`, lowered by .05 steps to .25 until they pass, else a
re-roll; the gain goes in the art's record. The built prompt's joints are "only slightly darker", and `paved_road` and
`cobble_road` may not be the brightest thing around a unit.

`groundcheck.py figures GROUND_DIR --art PATHS` is the same limit from the other side, and catches a new figure that is too
quiet: for each ground and each figure, figure range at least 4 x ground range and figure busy at least 3 x ground busy;
a warning naming the pair. Flat props (`cistern`, `cobbles`, `scorch_mark`) are not figures. The board adds one sheet: each
ground under all six class bodies, a crate, a tent and a wall face, at 1x-2x, for the eye.

Layering (the user's "even with layering") follows: wear, strays and patches only darken or repeat a ground's own colours,
so they stay under these limits. A board check measures the composed mix against the same range limit plus 50 %.

## 3. Reading: interactable or not

The user's goal: "to differentiate between interactables and non-interactables when viewing". First step: a soft ground
shadow under units.

### 3.1 What marks each today (MAPREAD1, PLAYUI1-R, PROPS1)

| Thing | Interactable | At a glance | On demand |
| --- | --- | --- | --- |
| Person (`npc`) | yes | a sprite and a hard foot ellipse in the kind's colour (alpha 150); a quest or service badge | name on hover or facing (always, with `map_names`), legend row, edge arrow |
| Icon marker (`exit`, `chest`, `lore`, `encounter`, `roamer`, `point`) | yes | an icon by kind; a spent or locked state | the same |
| The party | no click target but its lead pass | a sprite and foot ellipse; gold outline when hidden | - |
| Standing and ground props, walls, ground | no | nothing separates them from a marker but the art itself | no hover, no name, no legend |

The gap: only a hover or the legend says what can be used. People and props are both figures with art; a crate reads like
a chest; the foot ellipse's hard edge is a small mark, and it is lost on dark ground.

### 3.2 The dark-on-dark check

The rogue is the test: mean L 57.5, its outer pixels (the sprite's ring) mean L 37.5, its dark outline L 17.7. Grounds:
`soot_yard` L about 30, `ash` about 38. So:

- A black shadow does not help a dark unit's body: it is a ground mark at the feet, and at alpha 90 it takes `soot_yard`
  from 30 to about 19, level with the outline, so the feet merge. It must stay small and below the knee.
- The rogue's ring on soot differs by about 7.5 in L, against 40 or more for the priest. A mark on the ground cannot fix the
  upper body; only a rim can.

Measure: `read_contrast`, the mean |L| difference between a sprite's ring pixels and a 2 px band of the frame around them.
It is printed for the six class bodies on each ground, under each option.

### 3.3 Options (a debug select, `region_read`, beside `region_view`; the game forces one until the pick)

| Option | Draws | For | Against |
| --- | --- | --- | --- |
| `shadow` (first step, always on) | a soft neutral shadow under every unit and standing prop with art: (`ground::soft_ellipse`) alpha 90 at the centre, falling to 0 as `(1 - d^2)^2`; radii .45 cell wide, .18 cell tall; replaces the tinted hard ellipse | figures sit on the ground, no clutter | says nothing about interactable; fails the rogue check above the knee |
| `pool` | `shadow`, plus a soft light under each usable marker: kind colour, alpha 46 at the centre, radii .7 and .35 cell, in the pass after ground props; a spent marker has none | reads as "something is here", and icons get it too; quiet | helps only at the feet; adds colour to the ground, which section 2 must still absorb |
| `rim` | `shadow`, plus a thin rim on each usable marker: `Art::body_outline` or `Image::outline` at sprite pixels (1 px at 720p), kind colour at alpha 130; full alpha on hover or facing; a spent marker has none | the one that passes the rogue check over the whole body; props and scenery never have it, so a crate and a chest read apart | more on screen; may read as "selected" |

Recommendation: judge all three on the first real map, with `rim` as the default to beat. `shadow` first, as the user asked,
then `rim`, because a dark-clothed unit on dark ground needs its whole outline to read, and a rim is what a prop can never
have. If `rim` looks too busy, `pool` plus a rim only on hover or facing is the fallback.

The party has `shadow` only, and its gold outline when hidden. Tints stay on the rim or pool, using `marker_style`'s colours,
so kind still reads by colour.

### 3.4 Checks (S; `region.rs` and `ground.rs` tests, both views)

1. `soft_ellipse`: alpha is highest at the centre, 0 at and past the radii, monotonic in between, and the same twice.
2. Each option: a usable marker has its mark, a spent one not, a prop never (pixel probes); hover or facing raises the
   rim's alpha; `region_read` missing or unknown is `shadow`.
3. `read_contrast` of the rogue on `soot_yard` is higher under `rim` than `shadow`, and at least 25 under `rim`. The numbers
   for all bodies and options go on the review board; `shadow` alone is not gated, since it is the first step.
4. No mark changes a hit area, a widget or `text_audit`; walking and click tests run unchanged.

## 4. Prop sizes: order with PROPS1-A3

The scene shows a tent barely taller than a unit and crates nearly as big as one: PROPS1-A3's size classes are needed, and
sizes come from the prompt's class, never from scaling up (decision "Prop sheets by size class"). A review of the first
map at wrong sizes would judge the hierarchy on the wrong picture, so order them:

1. Split PROPS1-A3: A3a, the small-props and standing-props sheets; A3b, the large sheet (tents, buildings). A3a has no reason
   to wait for EXPLOREVIEW1-R, so it runs now, beside REGIONGROUND1-A1 (art lanes, no shared files). A3b keeps its wait,
   because the buildings' footprints follow the view (question 1).
2. REGIONGROUND1-R1 waits for A3a and uses it; tents and buildings there are today's art or boxes, and the board says so.
3. The mark sheets (REGIONGROUND1-A2) follow A3a's prompt form.

Height targets for A3's prompts, in units of a unit's 24 logical px (48 art px), with a unit drawn beside each prop on its
board:

| Class | Props | Height |
| --- | --- | --- |
| Small | crate stack, barrel, fire pit, stones | at most half a unit |
| Standing | signpost, boulder, banner, broken pillar | about 1 to 1.5 units |
| Large | tent at least 2 units; war tent, toll arch, buildings at least 3 | above a unit, and above a wall |

## 5. Rows (status `spec`; added to the main spec's section 8)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| REGIONGROUND1-W | Tall walls in the tilted view (section 1) | game | logic | REGIONGROUND1-L (same file) | `emberwake_gui/src/{mapview.rs, props.rs, region.rs}`, `SCREENS.md` 07, `GAME_BRIEF.md` |
| REGIONGROUND1-S | Soft shadow, `pool`, `rim`, the `region_read` debug select (section 3) | game | logic | REGIONGROUND1-W, REGIONGROUND1-E1 | `emberwake_gui/src/{region.rs, runner.rs, screens.rs, main.rs}` |
| PROPS1-A3a | PROPS1-A3's small and standing sheets, with section 4's targets (edit of the row) | game (art) | art, then asset | PROPS1-A2 | as PROPS1-A3 |
| PROPS1-A3b | PROPS1-A3's large sheet | game (art) | art, then asset | PROPS1-A2, EXPLOREVIEW1-R | as PROPS1-A3 |

Asks of other rows: REGIONGROUND1-T takes section 2's limits, `figures`, the figures sheet, `read_contrast` and
`--form region_tall_face`; REGIONGROUND1-K takes A19 and the new face size; REGIONGROUND1-E1 takes `ground::soft_ellipse`;
REGIONGROUND1-R1 now waits for W, S and PROPS1-A3a, and the review adds: which of `shadow`, `pool`, `rim`, is a wall the
right height, does anything but the units and props catch the eye.

## 6. Open question

1. Does the tall-wall answer also settle EXPLOREVIEW1-R? The scene that set the height is tilted. (a) Yes: tilted ships, the
   square view stays a debug view, and PROPS1-A3b, LOOK1-L2 and LOOK1-A1-A3 are free to go; the planner writes
   EXPLOREVIEW1's rows. (b) No: the user still plays both pairs, and tall walls wait in the tilted view only. Recommend (a)
   if the user confirms, since a tall face has no meaning from straight above; else (b), and nothing here changes.
