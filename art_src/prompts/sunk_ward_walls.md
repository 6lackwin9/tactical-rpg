# Sunk Ward walls, boundaries and floor textures (prompt file)

Art/UI batch 2026-11-15, from the asset sweep ([ASSET_PLAN.md](ASSET_PLAN.md)). `REVEAL1` (`docs/specs/REVEAL1.md`,
`PR5-54`) adds a location `boundary` art set, an indoor zone `wall_set` and the `floorboards` ground; `PR5-55` imports
the nine Sunk Ward maps on placeholders. This file holds the prompts for the art those placeholders wait for.

**Held**: the Sunk Ward ground batch waits for the `REGIONGROUND1` R1 tuning (Design, `AO-04` (1)), and every run
needs the user's go-ahead on the model and the count (`P-UI-06`). Local takes first (ComfyUI, `C:\dev\localgen`).
Reworded 2026-11-15 after the first run: `gorge_rock` top and both `interior_plaster` tiles came back near-white ("sooty" and "ash" were read as pale), so they now say dark and low-contrast outright.
Prompts follow REGIONGROUND1's short material-first form: one material, even detail, no light direction, no objects;
the colour is set in code from the ground's catalog colour. Sizes and the wall-face layout are REGIONGROUND1_SCENE's;
cut with `tools/tiles/cut.py`.

## 1. Floor textures (grounds, 256 x 256, seamless, straight down)

The nine `sunk_streets` grounds are in `REGIONGROUND1.md` section 7 (A3): `soot_yard` (2 takes), `cobble_road`,
`flagstone`, `mud`, `moss`, `iron_catwalk`, `tallow`, `water`. One new ground:

`floorboards` (`#3a2c22`), `games/emberwake/assets/tilesets/sunk_streets/ground/floorboards/v0.png`:

```text
Seamless top-down pixel-art floor texture, 16-bit RPG tile style: old dark wooden floorboards of uneven widths, worn smooth, soot in the seams, a few nail heads, no shine. Even detail edge to edge, no objects, no shadows, no light direction, flat and matte, tiles seamlessly on all four sides.
```

## 2. Boundary sets (the border round and past a map)

`games/emberwake/assets/tilesets/sunk_streets/boundary/<set>/{wall,wall_face}/v<n>.png`: `wall` is the flat top seen
from above, `wall_face` the front elevation, each in two or three takes so a long border does not repeat.

`drain_brick` (the hub, the Grate):

```text
wall: Seamless top-down pixel-art texture, 16-bit RPG tile style: the top of a thick storm-drain wall of soot-black brick, damp, a little dark moss in the joints. Even detail edge to edge, no objects, no shadows, flat and matte, tiles seamlessly on all four sides.
wall_face: Pixel-art wall face seen straight on, 16-bit RPG tile style: courses of soot-black storm-drain brick, damp streaks, one iron rung or pipe stub, a darker course at the foot. Tiles seamlessly left to right. Flat and matte, no light direction.
```

`blind_tenements` (Junction 7, the Crossroads, the Kiln, the Vats, Corvo's Gate):

```text
wall: Seamless top-down pixel-art texture, 16-bit RPG tile style: dark slate roof tiles under soot, a few missing, seen from straight above. Even detail edge to edge, no chimneys, no objects, no shadows, flat and matte, tiles seamlessly on all four sides.
wall_face: Pixel-art wall face seen straight on, 16-bit RPG tile style: the blind back wall of a tenement in soot-black brick, one bricked-up window, a rusted drainpipe, a darker course at the foot. Tiles seamlessly left to right. Flat and matte, no light direction, no lit windows.
```

`gorge_rock` (the Crucible, the Slag Gate):

```text
wall: Seamless top-down pixel-art texture, 16-bit RPG tile style: the top of very dark gorge rock, broken into angular slabs, the cracks between them filled with dark charcoal-grey ash only a little lighter than the rock. Dark overall and low contrast, no white, no pale lines. Even detail edge to edge, no objects, no shadows, flat and matte, tiles seamlessly on all four sides.
wall_face: Pixel-art wall face seen straight on, 16-bit RPG tile style: a tall face of dark gorge rock in broken vertical slabs, a thin slag-orange glint deep in one crack, darker at the foot. Tiles seamlessly left to right. Flat and matte, no light direction.
```

## 3. Interior wall sets (an indoor zone's `wall_set`)

`games/emberwake/assets/tilesets/sunk_streets/walls/<set>/{wall,wall_face}/v<n>.png`. Seen once a building's roof has
faded, so they sit under the furniture in contrast (the hierarchy rule).

`interior_plaster` (homes, shops, the doss house, the hovel, the workshop):

```text
wall: Seamless top-down pixel-art texture, 16-bit RPG tile style: the top of an interior wall, plain dark soot-grey plaster, smooth, with faint hairline cracks. Dark overall and low contrast, no white, no bare brick, no stains. Even detail, no objects, no shadows, flat and matte, tiles seamlessly on all four sides.
wall_face: Pixel-art interior wall face seen straight on, 16-bit RPG tile style: dark soot-grey plaster, grimy, a few hairline cracks, one small patch of dark brick showing through, a dark plank skirting at the foot. Dark overall and low contrast, no white, no bright patches. Tiles seamlessly left to right. Flat and matte, no light direction, no furniture, no pictures.
```

`interior_stone` (shrines, the armory, the guardroom, the sentry hut, the still-room, Orlov's store):

```text
wall: Seamless top-down pixel-art texture, 16-bit RPG tile style: the top of an interior wall of dressed dark stone blocks, tight joints. Even detail, no objects, no shadows, flat and matte, tiles seamlessly on all four sides.
wall_face: Pixel-art interior wall face seen straight on, 16-bit RPG tile style: dressed dark stone blocks in even courses, an empty iron torch ring, a darker course at the foot. Tiles seamlessly left to right. Flat and matte, no light direction.
```
