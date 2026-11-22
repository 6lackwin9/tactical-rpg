# A game's battle stage art

Status: canon (ENGINE-GUIDE2, 2026-11-22). Game-agnostic: this page names no game. It tells a new game on the RPG kit
how to dress its tilted battle board: a composed ground, stage sets and their `stage.fx.json`, decor, and painted unit
sheets. Everything here is asset files and data through `aegis content`; no rule, save, path or fight reads any of it,
so art can change at any time without moving a golden.

Where each part lives:

| Part | Layer | Where |
| --- | --- | --- |
| The ground composer (one picture from a cell grid) | engine | `aegis_ui::ground` (also the region maps' ground, [region_grounds.md](region_grounds.md)) |
| Painted sheets (`clips.json` `foot`, `painted`) | engine | `aegis_core::assets::Clips`, `aegis_ui::art::load_body` |
| File sizes and the decor checks | kit | `rpgkit_content` (`check/assets.rs`, `check/encounter.rs`) |
| Decor placement on a generated board | kit | `rpgkit_battle::encounter::generated_decor` |
| The stage drawing: geometry, the `stage.fx.json` reader, the composed board, decor, wall-run caps, the exit path and gate, the draw passes | kit | `rpgkit_ui::stage` (ENGINE-41); a game gives its sizes through `StageSizes` and runs the passes from its battle screen |

Rule one: **the board's `rows` and terrain alone decide the fight.** A sheet, a tint, a decal, a light or a decor
piece is a picture.

## The order of work

1. Play the board in the plain stage tiles (`<terrain>/v*`, 64 x 48 a cell) first.
2. Add a `sheet.png` for each ground terrain the board shows, so the floor is one picture with no cell edges.
3. Give zones their own look in `stage.fx.json` (a set, a tint, terrain overrides) only where a place needs one.
4. Add lights, decals and decor last.

## Stage set files

A stage set is a folder `tilesets/<set>/` over the asset layers, named by the world manifest's `stage_set` (the nearest
one along the world's lineage); a file not in it is read from `common_stage`. Sizes are art pixels; a stage cell is 64 x 48 (two art pixels a
logical pixel). The kit's content check refuses a file of the wrong size.

| File | Size | Notes |
| --- | --- | --- |
| `<terrain>/v<n>.png` | 64 x 48 | one tile of a terrain; picked per cell by a hash |
| `<terrain>/sheet.png` | whole 64 x 48 cells | the terrain's ground texture for the composed board (below) |
| `<terrain>/decal/v<n>.png` | 64 x 48 | an accent laid over the composed ground on some cells |
| `<terrain>/front_face/v<n>.png` | 64 x 24 | the face under a raised terrain's front edge (an override folder may hold its own) |
| `lip/<terrain>/v<n>.png` | 64 wide, 1 to 48 high | the art lip on a wall cap over that terrain, drawn before the flat lip colour |
| `cover/v<n>.png` | 64 x 72 | the prop an inner wall block (one not joined to the board's edge) draws as cover |
| `<upper>/over_<lower>/m01..m14` | 64 x 48 | transition pieces (stage sets only; region sets refuse `over_*`) |
| `stage.fx.json` | - | the set's look keys (below); optional |

## The composed ground

When any ground terrain on the board has a `sheet.png` (the zone's override folder first, then the zone's set, the world
set and `common_stage`), the whole floor composes through the engine's ground composer: each terrain is one layer whose
texture is its sheet, sampled by art pixel so no cell edge shows; layers meet at the composer's worn edges, under a
board-wide tone and two or three quiet blobs. A terrain with no sheet gives its first tile as its texture. With no sheet
at all the board draws its tiles as before.

Over the composite, decals (`<terrain>/decal/v*`) fall on about 10 percent of the cells (`decal_density`), and about 35
percent beside a block, a cover prop or a ground prop. The picture is built when the board, the sets, the zone look or
the art change, never per frame.

A sheet follows the region ground rule: fine grain only, no broad light and dark, no direction, nothing lined up with
the grid, and no one-off marks; the composer's tone and the decals carry those.

## `stage.fx.json`

A JSON object in `tilesets/<world set>/stage.fx.json`. Every key is optional; a bad value is left out, never an error at
run time. With no file the stage draws its default look.

| Key | Where | Holds |
| --- | --- | --- |
| `lip` | top level | `"#rrggbb"`: the flat lip line on wall caps (the file present but no key: a bronze default) |
| `spots` | top level, or `zones.<zone>.spots` (the zone's list wins) | `[{"at": [u, v]}]`, u and v 0 to 1 across the backdrop: a warm pool of light on the back rows under each |
| `lights` | top level, or `zones.<zone>.lights` (the zone's entry wins per key) | `{"<art key>": {"color": "#rrggbb", "radius": r}}`: the art (`cover/v3`, `<prop>/v1`, or a bare `<prop>` for every variant) casts a pool of `radius` cells, 0 to 4 |
| `decal_density` | top level, or per zone (the zone's wins) | 0 to 1: the share of ground cells with a decal (default 0.10) |
| `zones.<zone>.set` | per zone | a stage set id read before the world set for the floor and every terrain tile, the wall face and foot, cover props, front faces and transition pieces; a set holding only some terrains keeps the world set's for the rest |
| `zones.<zone>.tint` | per zone | `"#rrggbb"` multiplied over the zone's tiles, walls, cover, pieces and faces, except terrains with an override |
| `zones.<zone>.terrain.<terrain>` | per zone | a folder `<set>/<name>` under `tilesets/` (a bare `<name>`: the zone's set, else the world's), or `{"folder": ..., "tint": "#rrggbb"}`; its `v*` replace the terrain's tiles in the zone and its `front_face/v*` the face under it; its tint (none: untinted) replaces the zone's |

```json
{
  "lip": "#70583a",
  "decal_density": 0.08,
  "lights": {"lantern_pole": {"color": "#ffb060", "radius": 2.5}},
  "zones": {
    "vats": {
      "set": "harbour_vats_stage",
      "tint": "#e8d8c0",
      "terrain": {"lava": {"folder": "harbour_vats_stage/lava", "tint": "#ffd0a0"}},
      "spots": [{"at": [0.3, 0.4]}]
    }
  }
}
```

A folder with `..` or an empty part is ignored. The keys are read once per set and zone.

## Decor

Decor is display-only art standing on board cells. A decor art key is `cover` (any cover variant of the stage set),
`cover/v<n>`, `<prop>` (any variant of a prop) or `<prop>/v<n>`; props come from the battle prop sets.

| Field | On | Holds |
| --- | --- | --- |
| `decor` | an encounter | `[{"art": "<key>", "x": c, "y": r}]`, at most one piece a cell; stored as no key when empty |
| `decor_pool` | a world manifest | decor art keys a generated board draws from; a world with none uses its nearest parent's |

A generated board (an encounter with no `rows` and no `decor`) draws 2 to 6 pieces from its world's pool at session
start, on its own seeded stream, so no other draw moves and a replay draws the same: cover keys on free blocked inner
cells, else an edge cell; prop keys only on back cells (row 0, or the side columns in rows 0 to 2), so nothing tall
stands in the fight.

The content check refuses a piece that is not `{art, x, y}`, names art no layer holds, is off the grid, or stands on an
exit, a deploy cell, an objective tile or another piece; and a pool that is not a list of known keys, or names a key
twice.

## Painted sheets

A unit body sheet's `clips.json` may carry two engine keys:

| Key | Default | Does |
| --- | --- | --- |
| `foot` | none | the frame row the feet stand on: a stage that reads it scales the body so frame top to foot row is a set height per footprint cell (the kit stage: `PAINTED_CELLS`, 1.15 cells), centred on the foot point |
| `painted` | false | a painted sheet takes no palette swap (`aegis_ui::art::load_body`); the kit's sprite check A4 skips it |

A sheet without `foot` draws in the token square as before. A bad value is a listed problem, not a crash.
