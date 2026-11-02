# A game's theme and tile sets

Status: canon (LOOK1-E1, LOOK1-E2). Game-agnostic. Everything here is asset files over the layers (engine, kit, game,
the game's file winning); no engine code changes. Check both in the UI Gallery's Kit widgets entry: pages Theme and
Tiles.

## Runtime theme

With no theme file the engine draws its default look. To add one, put `ui/theme.json` in a kit's or game's assets:

```json
{
  "version": 1,
  "colors": { "text": "#d9cbb4" },
  "game_colors": { "my_accent": "#fbbf24" },
  "fonts": { "display": { "file": "fonts/MyFace.ttf", "scale": 1.0 } },
  "layout": { "radius": 0 },
  "frames": true
}
```

- Each file sets only the keys it names; the rest comes from the layer below. The role list is `theme::ROLES`; game
  code reads `game_colors` through `Theme::game`.
- The faces are `body`, `display` and `mono`. A font file is found through the layers. Keep a font's licence beside it.
- The shell loads the theme at start. A game calls `theme::reload` on its Reload data. Problems show on screen and the
  theme already in force stays.
- Engine widgets draw through `th()`; a game's screens should too, so the theme reaches them.

## Frames

A frame is `ui/frames/<role>.png` with a `ui/frames/<role>.json` sidecar beside it (roles: `theme::FRAME_ROLES`; a
missing role falls back to another). `"frames": false` in the theme draws vector panels instead.

| Sidecar key | Meaning | Default |
| --- | --- | --- |
| `slice` | [left, top, right, bottom] in art px | required |
| `scale` | art px per logical px, 1-32 | 2 |
| `edges` | `stretch` or `tile` | `stretch` |
| `center` | `stretch`, `tile` or `none` (the role's fill colour) | `stretch` |
| `pad` | content inset, [l, t, r, b] in logical px | the slices divided by `scale` |
| `ink` | the colour role of text on the frame | `text` |
| `corner_arm` | with `straddle`: each arm of a corner reaches this share (above 0, at most 0.5) of its own edge, at one scale in both axes, cut or rebuilt, never squashed; two arms on an edge meet at most | whole corners |
| `corner_arm_min`, `corner_arm_max` | logical px limits on an arm's reach | none |
| `corner_arm_full` | `true`: the arms run to the middle of each edge (a portrait frame may want this) | off |
| `corner_elbow` | art px from the anchor that a shorter arm must still show: it limits how large a very wide panel draws the art, whose longer arm then stops at the art's end | 0 |
| `corner_art` | art px per logical px of the corners, whatever the panel, smaller only when a panel cannot show the elbow and the finish | the natural arm fills the longer reach |
| `corner_splice` | `{"x": [c, d], "y": [c, d]}` art px: an arm's own finish, kept when the arm is cut shorter than the art; an arm is never longer than the art | arms cut at their end |
| `corner_inner` | [x, y] art px from the anchor that the art reaches into the panel: `painter::nine_content_rect` keeps content past it at the size the arms resolve to | none (`pad` alone) |
| `straddle` | `true`: half in, half out of the panel: edge bands and corners centred on its boundary and corner points, the centre filling exactly the panel; the panel's rect, layout, widget and hit area stay, the overhang only draws, under the canvas's clip; always averaged when reduced, whatever `reduce` says | off (inside the rect) |
| `corner_anchor` | [x, y] art px of the top-left corner placed on the panel's corner point, mirrored for the others, so the boundary runs along the art's arms | the corner's middle |
| `sharpen` | 0-2: an unsharp mask on the corners once reduced to the window | 0 |
| `reduce` | `average`: a high-resolution frame, finer than the window, is averaged down to the window's physical size (cached per frame and size) instead of sampled; only when `scale` exceeds the view's scale (`straddle` always averages) | off (nearest, exact pixels) |

A PNG without its sidecar, a sidecar without its PNG, or an unknown role file is an error.

## A set's files

A set is a folder under `tilesets/`. Kind names are the game's or kit's own; the engine's rules know none of them.

| File | What it is |
| --- | --- |
| `tilesets/<set>/<kind>/v<n>.png` | A ground tile and its variants, from `v0` in an unbroken run; seamless, opaque, one cell |
| `tilesets/<set>/wall_face/v<n>.png` | The face of a block, one cell wide, drawn over the cell's lower part when its south is open |
| `tilesets/<set>/<upper>/over_<lower>/m<NN>.png` | A transition piece, `NN` 01-14, opaque, one cell |

Keep art px a whole multiple of the logical cell, so that nothing resamples unevenly. A kit binds sets to places and
checks them; see its own docs.

## The corner mask

`aegis_ui::autotile` holds the masks. The caller's closures say what a cell is, off-grid cells included (taking the
nearest cell's kind draws no edge at a map's rim).

The four cells around a corner give its mask (a dual grid). For the corner at the top-left of cell (cx, cy):

| Bit | Cell | Position |
| --- | --- | --- |
| 1 | (cx - 1, cy - 1) | north-west |
| 2 | (cx, cy - 1) | north-east |
| 4 | (cx - 1, cy) | south-west |
| 8 | (cx, cy) | south-east |

A set bit means that cell is the upper kind. `corner_pieces(w, h, is_upper)` lists the corners with masks 1-14, each of
which takes piece `m<NN>` centred on that corner; 0 and 15 take none. Masks 6 and 9 are the two diagonal touches.

Draw in this order:

1. The ground pass, one tile per cell.
2. The pieces, clipped to the map.
3. The blocks: the cap, then the face where `block_edges(is_block, x, y).south` is open, then 1 px lines on the cap's
   open north and sides.

A piece must match the tiles under it along its border, so compose pieces from the two `v0` tiles rather than draw them
by hand or with an image model.
