# A game's region grounds

Status: canon (REGIONGROUND1-K, 2026-11-02). Game-agnostic: this page names no game. It tells a new game on the RPG kit
how to give a walkable map several ground types, its walls a height and its outer edge its own art. Everything here is
data through `aegis content` and asset files over the layers; no code changes. The engine's composer turns the result
into one picture, and the game's screen draws it.

Rule one: **`rows` alone decide walking.** Ground is a picture. No rule, save, click, path or fight reads it, so a layout
can change at any time without moving a journey or a trace.

## The order of work

1. Write the rows of `catalog/grounds.json`, one per ground the game wants, with a flat `colour` and no art.
2. Paint one map's `ground`, then play it in flat colours. Layout comes before tile sets.
3. Make the textures, one per ground and tile set, and place them. Art replaces the flat colour with no data change.
4. Add marks and placed patches last, and only where a map needs them.

## The rows

A row of the kind `grounds`, written with `aegis content upsert grounds '<json>'`:

| Key | Default | Allowed |
| --- | --- | --- |
| `id` | required | 2-24 characters of a-z, 0-9 and `_`, a letter first (the writer refuses others); unique |
| `name` | required | at most 32 characters |
| `order` | required | 0-999, unique: low paints first, and a higher ground's edge lies over a lower one |
| `kind` | `land` | `land`, or `water` (sunken, with a rim, and only on `~` cells) |
| `colour` | required | `#rrggbb`: what the ground draws with no art |
| `wear` | .10 | 0-.3: darkening of the worn band beside its edge |
| `strays` | false | bits of this ground scattered past its edge: stones off a road, never moss |
| `tone` | .13 | 0-.2: amplitude of the map-wide broad tone |
| `patches` | 0 | 0-.3: darkening of a random patch's core; 0 for none |
| `marks` | `[]` | at most 4 prop art ids |
| `marks_per_100` | 0 | 0-10 tries per 100 cells |
| `origin` | `system` | `system` (left out when stored) or `user`; set by the Forge, never typed |

Put natural ground low and built ground high, so a road's edge lies over the dirt beside it. An unknown key is an error.

## A map's ground

Four keys on a map template, beside `rows` and in the form of `zones`, written with `aegis content upsert maps`. Each is
stored even when empty. A map with `ground: []` draws as it did before grounds existed.

| Key | Holds |
| --- | --- |
| `ground` | one string per row, one character per cell |
| `ground_legend` | one lowercase letter or digit to a ground id, at most 12 |
| `ground_patches` | at most 16 large patches on purpose: `{id, x, y, w, h, strength}`, strength .02-.25, default .12 |
| `wall_storeys` | rects of tall wall: `{x, y, w, h, storeys}`, 2-4 storeys; a wall not in a rect is 1 |

Paint this way:

- Start with the quiet base ground everywhere, then roads and each place's own ground, then water last.
- A `water` ground goes on `~` cells and a land ground on `.` cells, so the picture never lies about walking. A `#` cell
  takes any: use the neighbouring ground, because edges and patches see it under the wall.
- A placed patch needs its rect and a ring of one cell to be one land ground with no wall. Make random patches and marks
  do the small work; a placed patch is for a large stain where a place needs one.
- Wall storeys sit on `#` cells only, never on the south row or the side columns: those stay short so nothing hides
  behind them. Proper buildings stay whole props, not stacks.

## The art

Files follow the lookup chain: the location's `tileset`, then the world lineage, then `common`.

| File | Size | Notes |
| --- | --- | --- |
| `tilesets/<set>/ground/<ground>/v0.png` | 256 x 256 | one seamless texture; only `v0` is read, because a variant per cell brings the grid back |
| `props/<set>/<mark>/v<n>.png` | at most 64 x 64 | transparent, straight down |
| `tilesets/<set>/wall_face/v<n>.png` | 128 x 48 | one storey, 4 cells wide, wrapping on all four sides |
| `tilesets/<set>/boundary/<name>/{wall,wall_face}/v<n>.png` | as `wall` and `wall_face` | the map's outer walls |

A texture carries fine grain only: no broad light and dark, no direction, no shape lined up with the grid, and no one-off
marks. Those come from the composer's tone and the mark layer, which never loop.

## Walls by kind

- Prop walls extend a prop or make an indoor space. They belong to their building's set and match its props.
- Boundary walls frame the map and may have their own form: cliffs, hedges, ramparts, fog. A location chooses them with
  its `boundary` field; empty means the set's own walls. The south and side borders stay short whatever the art.
- Each world has one `material_family` in its manifest: `{"palette": ["#rrggbb"], "materials": ["..."]}`, a short palette
  and material list both kinds of wall are prompted from. Art tools read it; no rule or screen does.

A doorway needs no field today: leave a `.` cell in the wall's `rows` and lay its threshold as a `ground` prop.

## The checks

`aegis content check` runs them all; an error refuses the write that adds it.

| Check | Level | Fails when |
| --- | --- | --- |
| G1-G6 | error | legend or patches with no rows; a wrong size; a bad legend key; a cell not in the legend; a legend id with no row; more than 12 |
| G7 | error | water on `.`, or land on `~` |
| G8 | error | a placed patch with bad keys, off the map, a wrong strength, or a mix under it or its ring |
| G9 | error | a storey rect off the map, off the walls, on a short border, overlapping, or a wrong height |
| G10 | error | a material family of the wrong shape |
| A16 | warning | a ground a map uses has no art along the chain; it draws its colour |
| A17 | error | a ground file not 256 x 256 |
| A18 | warning | a mark with no file, or larger than 64 x 64 |
| A19 | warning | a marker's anchor on a row a wall covers (2 per storey), not counting short borders and walls under a standing prop with art; move the anchor, or accept the outline it draws |
| A20 | warning | a location's `boundary` with no art along its chain |
| Legend | warning | a legend entry no cell uses |

A look check (tone, direction, contrast, colour) stays in the art tools, since a pick is judged once.
