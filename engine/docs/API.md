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

`launch` builds the `Cx`, the first screen and `shell::Options` (title, font, link port, icon, canvas, window sizes,
`headless`); `run_native` reads launch keys (`headless=WxH` opens no window) and calls `shell::run`. The asset layers
come from the game folder: `Layers::for_game(dir)` stacks the game's `assets/`, its kit's and the engine's.

### The command host

`aegis_host::Commands`: `handle(cmd, req) -> reply`, `mutating` (recorded), `restarts` (starts a new history) and
`checkpoint` (the state after each change, compared by replays). `Host::new(commands)` wraps it; `aegis_cli`'s `serve`
and `replay` drive it. Keep rules here, not in screens, so they test and replay without a window.

### The command line

A second program (`<id>_cli`) implements `aegis_cli::Game` (`USAGE`, `run` for the game's own commands, `data_dir`,
`manifest_path`, `content` (the `store::Hooks` of its data kinds), `host`, `driver`) and calls `aegis_cli::run`. Keep it
apart from the game program: the file and clock lints exempt only the CLI crate.

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
