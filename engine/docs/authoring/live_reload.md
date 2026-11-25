# Live reload: edit art, layouts, theme, sound and data while the game runs

Status: canon (ENGINE-64, LIVE-RELOAD1). Game-agnostic. A dev run only: never in a player's build, never on the web.

## Turning it on

Launch with `dev=1`. `aegis_ui::start::Start` turns `Options::live` on for it, so every game that starts through
`Start` (the hello and kit sample games too) reloads its art with no code of its own. A game with dev tools of its
own keeps it to the builds that have them (Emberwake: `options.live &= DEV_TOOLS`), so a player's build never watches.
Scenario runs pass `dev=1` with `live=0`, so they never watch and their shots never move.

## What reloads

A thread looks at every file under the asset layers and the game's data folders every 500 ms (`live::POLL`), by change
time and length. Before the next frame the shell reloads what changed and keeps the game state:

| Changed | What happens |
| --- | --- |
| A picture or a sidecar beside it (`*.png`, sheets, `stage.fx.json`) | `Art::forget` drops what was made from it, and every body's fitted frames; the art generation moves on, so a composed ground is made again |
| `ui/theme.json` | `theme::reload` (a problem keeps the theme in force and shows a notice) |
| `ui/layouts/*.json` | The screen's `Layouts` reads the file again on its next draw |
| A sound (`.ogg`, `.wav`, `.mp3`, `.flac`) | `Audio::forget`: the next play reads it again; one playing now plays on |
| A file in a data folder | `Context::reload_data`, then `Screen::data_reloaded` on every screen, top first |

Each reload shows one line at the bottom left for 3 seconds ("Reloaded 3 art files", "Data reloaded"); the agent
link's `ui_tree` gives it as `live` while it shows. Nothing reloads while a save is being written
(`aegis_save::writing`): the finds wait for the next frame.

## The game's data hook

- `Context::data_dirs`: the folders to watch (none by default).
- `Context::reload_data(cx, changed)`: load the data as a start does. Swap it in and return `Ok` (extra problems for
  the notice), or return `Err` naming the file and swap nothing. The shell shows "Data not reloaded: ERROR" as a notice.
- `Screen::data_reloaded(cx, top)`: a screen built from the old data rebuilds itself (Emberwake's region map, when on
  top with no conversation open, pushes a new map built from the game state), or returns a line saying what it keeps
  (a battle keeps its units until it ends).

Emberwake reads `data/` again with `Data::open`. Any load problem keeps the old data and names the first problem.
