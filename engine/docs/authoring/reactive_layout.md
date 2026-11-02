# Reactive layout: writing a screen that is calculated

Status: canon (decision "Engine layout is relative", user 2026-11-02). Game-agnostic: this page names no game.

Once a layout is selected, the position and size of each element are calculated from it. A moved screen has no fixed
positions, only relative positions and sizes; the actual size at render time comes from the layout. Changing a layout's
dimensions needs few or no screen edits.

Built so far: the crate `aegis_layout` (`Rect`, `Insets`, `Anchor`, `Part`, `frac`, `fill`, `resolve`, `Stack`, `pick`)
and `aegis_ui::shot_diff`. The later slices of the engine's layout work (`docs/specs/LAYOUT720.md`) add the rest: E1
`Grid` and `Screen::grid`; E3 `Metrics` with `hair`, `Env`, `Ui::m`, `Ui::env`, `Ui::screen`, `Ui::image_in`,
`Ui::frame_body` and `Ui::corner_size`; E5 `CanvasPolicy` and `layout_audit`. A name on this page that the code does not
have yet is one of those.

## Window, policy, canvas and unit

A game relates its canvas to the window through a policy. The canvas is measured in whole grid units, and `s` is how many
physical pixels one unit is.

| Policy | Pixels per unit | Canvas in units | Use |
| --- | --- | --- | --- |
| `Fixed { w, h }` | `min(pw / w, ph / h)` | always `w` x `h`, with bars | A game with one canvas; only `s` changes |
| `Fit { w, h }` | `min(pw / w, ph / h) * ui_scale` | `pw / s` x `ph / s` | Any aspect, at least `w` x `h` |
| `Dp` | `dpr * ui_scale` | `pw / s` x `ph / s` | A browser or phone: 1 unit = 1 CSS px |

The engine supports any canvas; a game decides what it offers. The grid's size is set in one place and is never a number
in a screen: every screen receives its canvas as a rect.

## Writing a screen

1. Cut from `ui.screen()`, the canvas as a rect. Read it each frame and assume no aspect.
2. Take tokens from `ui.m()`.
3. Draw frames through `ui.frame_body(role, r)`, which returns the content rect, so content never reads a slice number.
4. Cut every pane first, then choose one corner size for the smallest (`ui.corner_size`), then draw.
5. Keep no `Rect` across frames. A resize replaces the canvas and the view.

```rust
let (m, area) = (ui.m(), ui.screen());
let r = area.place(Anchor::Center, frac(703), frac(861));
let mut body = ui.frame_body("panel", r);
let tabs_row = body.cut_top(m.tab_h);
let page = body.pad(Insets::all(m.padding));
```

## What is calculated, and what the layout supplies

Every position and every panel, pane or element size is calculated. Text, spacing and thin lines are not stretched by a
fraction: the layout supplies them as tokens. They stay fixed from the layout, so they are fine on larger layouts, more
pronounced on smaller ones and always legible, and at one layout they keep the same share of the window at any resolution.

| Kind of value | How it is written | Examples |
| --- | --- | --- |
| Position | Anchor, cut, split, stack; `at` for a free point. Never an absolute canvas number | A dialog, a logo, a panel in its screen; a close button in its frame |
| Panel, pane and element size | `frac` or `fill`, or a token for a control's height | Dialog width, side rail, columns, list height |
| Inner layout of a widget | Relative to its own rect | Icon centred in a button, slider knob by value |
| Spacing | Tokens from `Metrics`, not fractions | Margin, gutter, padding; bands of a frame |
| Thin things | Tokens; `Metrics.hair` at small scales | Lines, borders, focus ring, scroll bar, grips |
| Text | Token times the Text size setting; boxes grow from measured text | Body, head, line step |
| Small icons and controls | Tokens | Glyph icons, checkbox, stepper, row height |
| Frame parts | Sidecar corner and band sizes, not fractions; stretch or tile only along the edge | Corners, banners |
| Images | Drawn in the rect the layout gives, through `Ui::image_in`; the per-image rule is later | Portraits, thumbnails, title art |
| Board and map | Cell math, their own world scale, not layout | Cells, sprites on cells, effects |

**The rule that any engine layout code touched moves to relative.** Any engine layout code a task touches moves to relative
in that task, at function level: a change that touches no layout moves nothing. New engine code is relative from its first
line. The existing picture of a screen still drawn in old units must stay bit-identical; where a share cannot reproduce
today's value exactly on every grid, the piece keeps a unit-scaled number and a share arrives with its first native caller.

**No fixed positions on a moved screen.** Review checks moved code for `Rect::new` with numbers. Exact alignment to art uses
an anchor plus a token.

## From fractions to grid to pixels

| Step | Input | Output | Rule |
| --- | --- | --- | --- |
| 1. Viewport | Window px, policy | Canvas in whole grid units, `s` | The table above |
| 2. Resolve | Anchors, shares, tokens, the canvas | Rect edges in whole grid units | Round down, edges not sizes |
| 3. Scale | Grid edges, `s` | Physical px | Each edge rounds on its own, so one grid edge is one pixel edge |

A changed viewport re-runs all three with no screen edit.

**Resolve shared edges, not sizes.** A split works out each boundary from its running total, rounded down once, and each
cell's width is the difference between its two boundaries. For fills over `room`, boundary `i` is
`room * sum(weights up to i) / sum(all weights)`, rounded down; the last is exactly `room`. Fixed gaps are added after.

- Two neighbours read one number for their shared edge, so there is no gap or overlap in units, nor in pixels at any scale.
- The total always equals the room, so nothing is lost at the far end.
- Cells can differ by 1 unit. That is the whole cost, and it never accumulates.

`resolve(parts, room)` works out sizes in one place: tokens first, then shares of what is left, then fills by weight. The
one exception is `Stack::next`, which reads a token and an over-budget share itself so `overflow` can count what did not
fit. A size that cannot fit becomes 0; nothing panics and nothing goes negative. `cut_*` share their edge by construction,
and `Stack` and `grid` use running totals. Parts with no fill whose shares total 1,000 end exactly on the area's edge. A
`Stack` share is of the whole area, unlike `split_y`, whose shares are of what the tokens leave: at height 1000, a token 40
then `frac(500)` twice gives 40, 500 and a 0-high rect with overflow 40 in a `Stack`, and 40, 480, 480 in `split_y`. `place` and `at` clamp to the parent, round the start down once and add the size, so a lone element has one
rounding only, and a centred odd remainder rounds toward the top left. Never add a rounded size to a rounded position across
neighbours: a screen that needs a shared edge cuts or splits.

**Worked example.** A screen: margin 16, a rail of `frac(186)`, gap 12, then a list and a detail pane split `fill(2)`,
`fill(1)` with gap 12. These numbers are pinned by a test in `aegis_layout/tests/layout.rs`.

| | Canvas 1280 x 720 | Canvas 1720 x 720 |
| --- | --- | --- |
| Area after margin | x 16 to 1264, 1248 wide | x 16 to 1704, 1688 wide |
| Rail | 186 per mille of 1248 is 232.1, so 232; x 16 to 248 | 313.97, so 313; x 16 to 329 |
| Room for the panes | 260 to 1264, less the gap: 992 | 341 to 1704, less the gap: 1351 |
| Boundary at 2 of 3 | 661.3, so 661 | 900.7, so 900 |
| List | x 260 to 921 (661 wide) | x 341 to 1241 (900 wide) |
| Detail | x 933 to 1264 (331 wide) | x 1253 to 1704 (451 wide) |
| Ends on the area's edge | Yes | Yes |

Rounding sizes instead would give 661 + 330 and 900 + 450, ending at 1263 and 1703: the off-by-one gap this rule prevents.

```rust
let mut body = canvas.pad(Insets::all(16));
let rail = body.cut_left(frac(186));
body.cut_left(12);
let panes = body.split_x(&[fill(2), fill(1)], 12);
```

## Size classes and `pick`

Selecting a layout means `pick`, a size class, or the only layout a screen has. The engine never rearranges by itself: the
screen writes each arrangement. `pick(area, choices, last)` returns the first choice whose `Need` fits `area`, else `last`.
It reads any rect, so a pane can rearrange inside itself. Size classes (`Compact`, `Medium`, `Expanded`, per axis) are
decided on the canvas in units, so a larger UI scale moves a screen to a smaller class by itself. A narrow tall canvas is
width Compact and its `pick` chooses a stack, so portrait needs no special case.

## Overflow per container

Every stack or list declares what it does when it does not fit; the default is Report.

| Policy | Mechanism | Use for |
| --- | --- | --- |
| Scroll | The widget kit's scroll area, content height from `Stack` | Lists, long text |
| Collapse | The kit's section, or tabs through `pick` | Secondary panels |
| Truncate | Fitted text with the full text on a tip and on focus | Names, one-line labels |
| Report | `Stack::overflow()` goes into the audit; drawing is clipped to the area | Anything not designed to overflow |

## `dpr`, Text size and UI scale

`dpr` is window px per CSS px, from the window system or the browser. It is not a true measurement: a handheld can report 1,
so a game targeting one also sets a default UI scale. Text size scales text tokens only; boxes grow from measured text, so
every moved screen has to hold at each text size. UI scale changes `s` under `Fit` and `Dp`, so the canvas in units shrinks
and the size class may change.

## Art and units

- Art lines up with layout only at 1 unit = 1 art px at the canvas's base window.
- Even unit values land exactly at scale 1.5. Odd values are allowed, but they fall on a half pixel and round up, so a
  3-unit bar is 4 or 5 px. Repeated thin elements (pips, rulers) use even steps, or uneven gaps show.
- Below scale 1 a 1-unit fill can round to 0 px: use `Metrics.hair`.
- Frame sidecars hold art pixels and a scale; they do not change when a screen moves. Corners keep one size on any panel,
  and edges stretch or tile.
- Art on a moved screen is authored in a rect of its own proportions until the image rule below exists.

## What is later, and where it attaches

Each is its own later task, and no caller changes when it lands.

| Later work | Where it attaches | Until then |
| --- | --- | --- |
| Images: a scale or fill rule per image | `Ui::image_in(img, r)`: a rule argument with a default, aspect-kept drawing, a `stretched_art` audit kind | Drawn in the given rect |
| Minimum size caps, so some elements stay legible | `Part` (private fields, so a `.min(n)` adds without breaking callers); `Ui::m()`; `Env.dpr` | Parts have no caps; text keeps its pixel floor |
| Space priority: what gives way when the minimums do not fit | `Part` (a `.give_way(rank)`), applied inside `resolve` before boundaries are worked out | Too little room gives 0 size, and the audit reports it |

## No hover-only information

Nothing needed to play lives only in a hover card. A tip is also reachable by focus or press. Dragging keeps a non-drag
route, and a right click keeps a visible button. Touch input is later, and this rule keeps it possible.

## What layout code may not use

No file, thread, socket or clock: the canvas comes in as a value. `aegis_layout` is `no_std` with no dependencies, so the
compiler proves this for the crate. `Rect` stays whole units and shares stay per mille, so layout is deterministic and
portable. No `Native` screen assumes a wide canvas; one with no compact choice reports overflow.

## Testing

- **Relations, not numbers.** "Yes is left of No, inside the frame, one gutter apart", over several canvases. Derive every
  expected value from the rules.
- **Profiles.** A moved screen must pass the `desktop` profile at each text size. Other profiles (wide, ultrawide, handheld,
  browser, phone) report only until the minimums exist, and must resolve without a panic.
- **`shot_diff`.** `aegis_ui::shot_diff::shot_diff(a, b, s)` compares two shots at `s` physical px per grid unit. A pixel
  matches if some pixel of the other image within `ceil(s)`, in a square window, is within 3 per channel. Both images are
  read against each other, and any unmatched pixel fails with a bounding box. It refuses shots of different sizes. It knows
  nothing of animation: freeze the clock, or paint an animated area the same colour in both shots first.

## Sorting existing screens

A screen picks with `fn grid(&self) -> Grid`; a preview passes the shown screen's `grid()`.

Sort each existing screen before touching it. Move a screen only inside a task that changes it: there is no sweep and no end
date for the old mapping.

| Class | Meaning | Now | Later |
| --- | --- | --- | --- |
| Finalized | Real screens the user has signed off | Exactly today's picture | Moves, with a before and after check, when a task changes it |
| Placeholder | Real screens likely to be redone | Quadrant or wrapper. It only has to draw and click: no shots, no parity, no fixes | Converted by the task that rebuilds it |
| Debug or tools | Screens the player never sees | Quadrant or wrapper, same bar | Never, unless a task wants it |

Two ways keep an unmoved screen running, both one field in the view and neither editing the screen's numbers:

| | Wrapper | Quadrant |
| --- | --- | --- |
| Picture | Exactly today, filling the window | Everything at half size, top left; the rest stays the background |
| Screen cost | None; it is the default | One line, reversible by deleting it |
| Choose for | What a user plays or judges by eye | Other placeholder and tool screens |
