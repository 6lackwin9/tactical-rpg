# The Aegis engine API: a guide for a new game

Status: guide (ENGINE-6, NEW-GAME1). Game-agnostic. Start a game with `aegis new game <id>` (any game's command line
has it), read what it wrote, then this page. The sample game `games/hello/` is the next step up: a field, a sprite, a
save. Nothing here is a stable, versioned API yet; the engine changes with its games, and the gate tells you when.

## The crates

| Crate | A game uses it for |
| --- | --- |
| `aegis_core` | the asset layers (`assets::Layers`), the clock (`clock`, never `std::time`), files (`vfs`, never `std::fs` in runtime code), the deterministic RNG, JSON helpers (`json::Dict`), temp folders in tests (`scratch::Scratch`) |
| `aegis_layout` | rects on whole grid units: `Rect::place(Anchor, w, h)`, `cut_top` / `cut_left` / ..., `pad(Insets)`, `split_x`, `Part` (fractions, ratios, fills) |
| `aegis_ui` | the window shell (`shell::run`), screens (`screen`), widgets (`ui::Ui`), the painter, the theme, art and audio, panes, screen layouts (`screen_layout`) |
| `aegis_host` | the command API: one JSON request in, one reply out, every change recorded for replays (`Host`, `Commands`) |
| `aegis_save` | versioned save slots with migrations, the per-game user folder (`user_dir(id)`) |
| `aegis_content` | the manifest (`manifest::Manifest`, `game.json`), the data store and its writer (`store::Hooks`) |
| `aegis_cli` | the command line for agents and humans: `scenario`, `playtest`, `replay`, `link`, `content`, `new game` (`Game`) |

A kit (`kits/<genre>/`, e.g. the RPG kit) sits between the engine and a game; a game on the engine alone names no kit
(`"kit": ""` in `game.json`). The engine never names a game (`tools/fc.sh layers`).

## The parts a game writes

### The context

`Cx<G>` (`aegis_ui::screen`) is what every screen gets: `settings`, `audio`, `art`, `nav` (push `Nav::Push`, `Pop`,
`Reset`), `notice` (lines the shell shows), `quit`, and `app: G`, the game's own state. `G` implements
`screen::Context`: `MORE_COMMANDS` (a line for the agent link's help), `note` (event lines for screenshots),
`agent_summary` and `agent_game` (what the link's `summary` and `game` requests answer).

### Screens

A screen implements `screen::Screen<G>`. Required: `id` (the name agents and scenarios wait for), `draw` (paint and
collect widgets, every frame), and the widget plumbing, which the macro `widget_plumbing!(Cx<G>)` writes when the
screen keeps `w: Vec<Widget<A>>` and has `fn act(&mut self, cx, a: A)`. Optional: `grid` (`Grid::Native` is 1280 x 720
units; the default `Grid::Old` is 640 x 360), `key`, `back` (Esc and right click), `tick`, `wheel`, `frame_need`
(`Idle` for a still screen), `auto_focus`, `state` (what `ui_tree` and scenarios read as `state.*`) and `command` (the
screen's own link requests).

In `draw`: `let mut ui: Ui<A> = Ui::new(g, mouse, live);` then `ui.m()` (sizes in the screen's units: `body`, `title`,
`row_h`, `margin`, `u(n)`), `ui.screen()` (the whole area), `ui.t` (the theme's colours), `ui.p` (the painter: `fill`,
`text`, `text_w`, `image`, `panel`), and widgets: `button`, `row`, `tab`, `checkbox`, `slider`, `para`, `image_in`,
`frame_body(role, rect)` (a themed frame, returning its inner rect). End with `self.w = ui.widgets;`. Widget ids are how
agents click (`menu.start`); keep them stable.

A screen can keep its layout out of code: build a `screen_layout::View` and let a layout file place it
(`engine/docs/authoring/screen_layouts.md`).

### Launch

`launch` builds the `Cx`, the first screen and `shell::Options` through the engine's game start
(`aegis_ui::start::Start::new(id, game folder, engine folder, launch keys, headless)`): it reads `game.json`, stacks the
asset layers (`Layers::for_game(dir)`: the game's `assets/`, its kit's and the engine's), picks the saves folder
(`saves=`), the link port (`port=`) and the options (title, font, icon, canvas, window sizes); `settings`, `audio` and
`cx` finish the context around the game's own state. `run_native` reads launch keys (`headless=WxH` opens no window)
and calls `shell::run`. The game's command line program is named in `game.json` (`cli`; `<id>_cli` when missing), and
`tools/fc.sh` runs scenarios and playtests through it.

### The command host

`aegis_host::Commands`: `handle(cmd, req) -> reply`, `mutating` (recorded), `restarts` (starts a new history) and
`checkpoint` (the state after each change, compared by replays). `Host::new(commands)` wraps it; `aegis_cli`'s `serve`
and `replay` drive it. Keep rules here, not in screens, so they test and replay without a window.

### The command line

A second program (`<id>_cli`) implements `aegis_cli::Game` (`USAGE`, `run` for the game's own commands, `data_dir`,
`manifest_path`, `content` (the `store::Hooks` of its data kinds), `host`, `driver`) and calls `aegis_cli::run`. Keep it
apart from the game program: the file and clock lints exempt only the CLI crate.

### Controls

Screens read `Key`s. Every key, and every gamepad button, passes through the control map first (`aegis_ui::controls`,
settings key `controls`: `{"keys": {"i": "up"}, "pad": {"south": "enter"}}`), so a player can rebind keys and a pad
drives the same screens; bad entries are shown and keep their defaults. Pad defaults: south Enter, east Esc (Back), west
Space, north Tab, the bumpers Page Up / Down, Select Shift+Tab, Start Esc, the d-pad and left stick the arrows (held, they
repeat after 350 ms every 110 ms and set `Held::dir`). The pad is read on Windows; anywhere, and with no window, the link
requests `{"cmd": "pad", "button": "south"}` and `{"cmd": "controls"}` press a button and show the map.

### Words on screen

Put a screen's words in `assets/strings/<language>.json` (a flat object, `"menu.start": "Start"`, `{name}` filled by
`tr_with`) and draw `aegis_ui::strings::tr("menu.start")`. Tables layer like every asset (engine, kit, game; each sets
only the keys it names). The language is the settings key `language` (default `en`); a key missing in it falls back to
`en`, then shows as the key. The gate (`tools/fc.sh layouts`) wants every language to hold the keys of `en`, with the
same `{name}`s, and no more. The scaffold's menu already reads its words this way.

### Log and crash report

A windowed native run writes `game.log` (the run before kept as `game.1.log`) in `logs/` beside the player's settings,
with every screen change and notice; `aegis_ui::crashlog::line` adds to it. A panic writes `crash-<unix seconds>.txt`
there (message, place, backtrace, the last 200 log lines; the newest 5 are kept) and then goes on as before. The web and
a run with no window write none.

## Checks and tools

- `tools/fc.sh gate`: builds every crate (a warning fails), runs every test, every game's scenarios with no window,
  `tools/fc.sh newgame` (these templates still make a working game), the layer, layout and lint checks.
- Scenarios (`games/<id>/scenarios/*.json`, `engine/docs/authoring/scenarios.md`): wait, click, key, assert, shot,
  perf, golden shots. `<id>_cli scenario run all`.
- Agent play: `<id>_cli playtest start`, `do click menu.start`, `stop` (`engine/docs/authoring/agent_playtest.md`).
- Theme and frames: `ui/theme.json` in the game's assets (`engine/docs/authoring/theme_and_tiles.md`).

## Rules the engine holds every game to

Time through `aegis_core::clock`, files through `aegis_core::vfs` (lints in `aegis_core` tests); no compiler warning;
no `unsafe` (the workspace deny set); data edited through the engine's writer, never by hand; goldens re-recorded only
for an intended change.
