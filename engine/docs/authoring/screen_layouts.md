# Screen layouts: behaviour apart from layout and look

Status: engine framework of UISPLIT1 (`docs/specs/UISPLIT1.md`, row PR7-38). Game-agnostic: this page names no game.

A split screen has three parts. Its **view** (screen code) lists what it shows and the actions it offers. Its **layout
file** (Art/UI) places regions on the screen and binds each to one component. Its **look** stays in `ui/theme.json`.
Screen code never places a rect for a split screen; a layout never names a colour.

## Files

Both live in the asset layers (engine, kit, game, bottom-up; the first layer holding a file wins, as for any asset):

| File | Holds | Owner |
| --- | --- | --- |
| `ui/views/<screen>.view.json` | the view fixture: sample values for every field and action, for preview and the check | screen code |
| `ui/layouts/<screen>.json` | the layout: regions, components, bindings | Art/UI |

A screen with no layout file draws the **default**: a main panel shell titled by the text field `title`, every other
field top to bottom under its name, and every action in a footer button bar. A list, tab, slot or icon field `<name>`
is wired to the action of the same id. Screen code ships on the default; a layout replaces it with no code change.

## The view

`aegis_ui::screen_layout::View`: a screen id, named fields in order, and actions (`id`, `label`, `enabled`). In JSON
each field is an object with one key, its kind:

| Kind | JSON | Drawn by |
| --- | --- | --- |
| text, number, flag | `{"text": "..."}`, `{"number": 3}`, `{"flag": true}` | `text_block`, `tooltip` |
| picture | `{"picture": "ui/help.png"}` (an asset path) | `portrait` |
| meter | `{"meter": [7, 20]}` (value, max) | `bar_meter` |
| rows | `{"rows": [{"id", "label", "detail", "detail2", "sub", "picture", "enabled"}]}` | `list_row` |
| stats | `{"stats": [["Weight", "3"]]}` | `stat_block` |
| tabs | `{"tabs": {"labels": [...], "open": 0}}` | `tab_rail` |
| slots | `{"slots": [{"id", "title", "lines", "stamp", "picture"}]}` | `slot_card` |
| icons | `{"icons": ["path.png"]}` | `icon_grid` |

A click on a drawn widget hands the screen an `Act { action, arg }`: a button's action id, or a list's or slot's
`select` action with the item id as `arg`, a tab's or icon's with its index. Widget ids are `<screen>.<action>` for
buttons and `<screen>.<region>.<i>` for the items of a region, so agents and scenarios find them by name.

## The layout

```json
{"screen": "shop", "regions": [
  {"id": "shell", "component": "panel_shell", "bind": {"title": "title"}, "anchor": "center", "w": "94%", "h": "92%"},
  {"id": "goods", "in": "shell", "component": "list_row", "bind": {"rows": "goods", "select": "pick"},
    "anchor": "top_left", "w": "55%", "h": "fill", "y": 5, "when": [{"text_scale_min": 1.25, "w": "65%"}]},
  {"id": "foot", "in": "shell", "component": "button_bar", "bind": {"actions": ["buy", "leave"]}, "anchor": "bottom_right", "w": 60, "h": 7}
]}
```

- `anchor`: `top_left`, `top`, `top_right`, `left`, `center`, `right`, `bottom_left`, `bottom`, `bottom_right`.
- `w`, `h`: a whole number of grid tokens (the theme's `grid`, 1 to 1000), a share `"55%"`, or `"fill"`.
- `x`, `y`: an offset in grid tokens after anchoring, on the screen's axes (`y: -6` moves a `bottom` region up).
- `in`: an earlier region; the region is placed inside its content (a panel shell's body, a tab rail's page).
- `when`: overrides of the place keys from a text scale up (`text_scale_min`), the last matching one wins.
- `style`: `main` (the `ornate` frame) or `popup` (the `panel` frame) for a `panel_shell`; `muted` (both columns
  dim) for a `stat_block`. `text`: a `heading`'s own words (a heading only). `note`: free text.

| Component | Binds (required, then optional) |
| --- | --- |
| `panel_shell` | (`title`) |
| `text_block`, `tooltip` | `text` |
| `stat_block` | `stats` |
| `list_row` | `rows` (`select`) |
| `heading` | none: its `text` key is the heading (gold, display face, one line) |
| `tab_rail` | `tabs`, `select` |
| `slot_card` | `slots` (`select`) |
| `menu_column`, `button_bar` | `actions`: a list of action ids, or `"all"` |
| `portrait` | `picture` |
| `icon_grid` | `icons` (`select`) |
| `bar_meter` | `meter` (`label`) |
| `custom` | `slot`: a name; the region's rect goes back to screen code (`Drawn::custom`), for a board or a map |

A `list_row` region may be shorter than its list (ENGINE-18): the rows scroll inside it (the wheel; the keys and a pad
keep the focused row and one row each side in view), with a scroll bar, and every row stays a widget whether it shows
or not, so `<screen>.<region>.<n>` resolves for any row. A list whose rows have no picture draws no icon tiles. Rows
with a `detail2` show two numbers columns (`detail`, then `detail2` at the right end), each as wide as its widest
entry in the list, so they line up. The default layout gives a list its whole height, so it never scrolls.

A tab rail lower than 24 units draws as a horizontal strip, and a region `in` it goes below the strip. A region id
may not also be an action id (their widget ids would clash). A list, tab, slot or icon whose `select` action is missing
or disabled draws but does nothing.

## Check, fallback and preview

- `tools/fc.sh layouts` (in the gate) runs `ui_layout check` on every game folder: each layout parses, names known
  components and keys, binds what its component needs, and fits its fixture (fields exist with the right kind, actions
  exist). A layout with no fixture fails.
- At run time `screen_layout::Layouts` loads and caches layouts; one that fails the check against the live view draws
  the default and reports why once per change (`take_problems`). In a native debug build a saved file reloads.
- The UI Gallery's Screen layouts entry draws every fixture through its layout or the default, with the problems and
  the action a click would run.
- `ui_layout shot <game dir> <screen> [size=1280x720] [out=shot.png] [default=1]` writes a PNG for review beside the
  HTML mock (`cargo run -p aegis_ui --bin ui_layout -- ...`).

The engine ships one sample, `engine/assets/ui/layouts/layout_sample.json` with its fixture.
