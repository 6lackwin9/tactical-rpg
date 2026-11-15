# REGIONGROUND1 Historical Slices, Review & Open Questions

Archived 2026-11-15 (CTX-CLEANUP1-DESIGN), Ashlands pilot slice log (`D1`/`R1`) and resolved Q&A moved from [REGIONGROUND1.md](../../../../games/emberwake/docs/specs/REGIONGROUND1.md).

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

As built (REGIONGROUND1-L, 2026-11-02): the module is registered in `main.rs` (`emberwake_gui` is a bin, with no
`lib.rs`). `MapGround` compares the map, the view, the sets, the data and the art each frame and makes the job and key only
when one changed; for the art it reads `Art::generation` (new in `aegis_ui::art`), and the Scene Runner's Reload data now
opens a new `Art`, since `Art` never re-reads a file and an art edit could not otherwise reach a running build. A layer's
`marks` stay empty until the mark rows (E2, D2) need them. A row whose `colour` is not `#rrggbb`, or a map past the
composer's `ART_PX_MAX`, draws as if it had no ground. Check L1's hash was a one-off (every shipped map, both views, art on
and off, identical before and after); the kept test probes every tile's colour on a fixture map instead, so another lane's
HUD change cannot fail it.

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
