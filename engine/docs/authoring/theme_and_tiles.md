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

### Tokens on the layout grid

`layout` holds values in old units, for screens still on the old numbers. A screen on the layout grid reads its tokens
through `Ui::m`, and the theme's optional `metrics` block sets them, in grid units (`reactive_layout.md`):

```json
{ "version": 1, "metrics": { "body": 15, "padding": 14, "breakpoints": { "width": [800, 1200] } } }
```

| Key | Type and limits | With no key |
| --- | --- | --- |
| `body`, `small`, `title`, `head` | number, 1-64 | the old constant doubled: 14, 12, 15.2, 24 |
| `line` | number, 1-96 | 18 |
| `radius` | number, 0-16 | `layout.radius` doubled |
| `margin` | whole, 1-64 | `layout.margin` doubled |
| `gutter`, `padding` | whole, 1-48 | `layout.gutter`, `layout.padding` doubled |
| `grid` | whole, 1-16 | `layout.grid` doubled |
| `frame_title_h` | whole, 24-96 | `layout.frame_title_h` doubled |
| `tab_h`, `row_h` | whole, 1-96 | 36, 28 |
| `focus_glow` | whole, 0-6 | `layout.focus_glow` doubled |
| `breakpoints` | `width` and `height`, each `[compact, medium]`, whole, ascending, 1-100000 | `[800, 1200]`, `[480, 700]` |

An unknown key is an error. With no key a token is today's value doubled, so a screen moved to the grid draws its old
picture under any theme. On a screen still on the old numbers, `metrics` changes nothing.

## Frames

A frame is `ui/frames/<role>.png` with a `ui/frames/<role>.json` sidecar beside it (roles: `theme::FRAME_ROLES`; a
missing role falls back to another). `"frames": false` in the theme draws vector panels instead.

| Sidecar key | Meaning | Default |
| --- | --- | --- |
| `slice` | [left, top, right, bottom] in art px | required |
| `scale` | art px per logical px, 1-32 | 2 |
| `edges` | `stretch` or `tile` (with `straddle`, a tiled edge repeats at its art's size, under the frame's `sharpen`) | `stretch` |
| `center` | `stretch`, `tile` or `none` (the role's fill colour) | `stretch` |
| `pad` | content inset, [l, t, r, b] in logical px | the slices divided by `scale` |
| `ink` | the colour role of text on the frame | `text` |
| `corner_inner` | [x, y] art px from the anchor that the art reaches into the panel: `painter::nine_content_rect` keeps content past it at the size the corners draw | none (`pad` alone) |
| `straddle` | `true`: half in, half out of the panel: edge bands and corners centred on its boundary and corner points, the centre filling exactly the panel; the panel's rect, layout, widget and hit area stay, the overhang only draws, under the canvas's clip; always averaged when reduced, whatever `reduce` says; the corners follow the window at `scale` and keep one size on any panel, as large as fits on a panel too small for two, mirrored art landing as mirrored pixels | off (inside the rect) |
| `corner_anchor` | [x, y] art px of the top-left corner placed on the panel's corner point, mirrored for the others, so the boundary runs along the art's arms | the corner's middle |
| `sharpen` | 0-2: an unsharp mask on the corners once reduced to the window | 0 |
| `reduce` | `average`: a high-resolution frame, finer than the window, is averaged down to the window's physical size (cached per frame and size) instead of sampled; only when `scale` exceeds the view's scale (`straddle` always averages) | off (nearest, exact pixels) |
| `corner_size` | 0.25-3: the corners' drawn size against `scale`, from the panel's corner point; inside the rect the edge bands go with it; `corner_inner` follows the size drawn | 1 |
| `tip` | [from, half] art px, with `straddle`: the head at each arm's end of the top-left corner, from `from` to the slice's end and `half` px either side of the anchor, mirrored for the others | none |
| `tip_size` | 0.25-3, with `tip`: the head's size against the art, from its base; the arm stays | 1 |
| `line_width` | 0.5-8 logical px, with `straddle` and stretched edges: the edge drawn as a solid line of this width, centred on the boundary; the rest of the band goes | the art's width |
| `line_color` | a colour role or `#rrggbb`, as `line_width` | the art's colour |

A PNG without its sidecar, a sidecar without its PNG, or an unknown role file is an error, and so is a key outside its limits or on a frame that cannot draw it.

### Tuning a frame

The UI Gallery's Frame tuner entry (the same pane as the Kit widgets page Frames, `aegis_ui::frame_tuner`) shows a panel with a
portrait box and sample text, at the window's real pixels, on sliders you judge by eye:

- Pick the frame role (PageUp and PageDown go through them; a role with no file tunes the file that draws it).
- Width and Height are the panel's, read out and not saved. Corner, Tip, Line, the colour (a role or the hex field) and Padding
  are sidecar keys, and Straddle turns `straddle` on and off. A slider the frame cannot use is off: Tip needs a head on the art
  (`frame_tuner::find_tip`, or a `tip` key), Line and the colour a plain line on a straddling frame's edge. Padding sets all four
  sides. Big preview gives the stage the whole page; Esc leaves it.
- The keyboard walks the controls as anywhere (Tab, the arrows, Enter; Left and Right move a focused slider).
- The sliders change this pane's own copy, never the theme in force or a file, until Save. The read-out under the stage gives the
  sidecar as it stands, the panel's size, its content rect and its size in the window.
- Save, in a debug build only, merges the keys that changed into the sidecar of the file that draws the role, keeping its other
  keys and their order, and loads the theme again; a file the theme refuses goes back as it was. With nothing changed nothing is
  written. Revert drops what is not saved, and then puts the file back as it was before the session's first save. A release build
  shows the read-out only.
- Saving writes the source file the game runs from, so it shows in `git status`. A game whose art tool writes sidecars should
  keep these keys on a regenerate (`tools/gen_art.py ui_frames` does, and names them, unless the slices changed).

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
