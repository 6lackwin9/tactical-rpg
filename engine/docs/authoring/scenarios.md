# Scenarios: walking a game's real screens

`aegis scenario` plays a game in a real muted window that does not take the focus: JSON files click, wait on state and
assert. It is an engine tool, so any game can use it. The spec is `docs/specs/AGENTROLES1.md` section 4.

A scenario checks that the built game still works end to end. A rule or a screen's own behaviour belongs in a test.

## Commands

| Command | Does |
| --- | --- |
| `aegis scenario check [<file> or all]` | Parses the files. No window. Exit 1 on a problem, each naming the file and step |
| `aegis scenario run <id, all or a .json path> [size=WxH] [out=<dir>] [slow=N] [exe=<program>] [data=<dir>]` | Runs them |
| `tools/fc.sh scenario check [<file> or all]` | The same check |
| `tools/fc.sh scenario <id, all or a .json path> [size=] [out=] [profile=dev or release] [slow=] [data=]` | Builds the game and the CLI in one profile, then runs |

Exit codes: 0 pass, 1 a step failed, 2 could not start.

The files lie in `games/<id>/scenarios/<id>.json`; the folder names the game, the file names nothing else. The window's
program is the file named as the manifest's `id`, beside the CLI (`exe=` says otherwise). `size=` replaces the file's sizes.
`slow=N` multiplies every timeout, 1 to 20, for a machine under load. `data=` opens a scratch copy of the data folder.
A real window never runs in the gate. The gate parses the files and runs the driver's tests on a fake link.

## The file

| Key | Type | Default | Rule |
| --- | --- | --- | --- |
| `id` | text | required | Equals the file's name; letters, digits, `_`, `-` |
| `title` | text | "" | - |
| `status` | `active`, `waiting` | `active` | `waiting` needs `why`; it is parsed and never run |
| `why` | text | - | Why it waits |
| `launch` | object | `{"screen": "menu"}` | `screen` is `menu`, `arena` or `runner`; optional `encounter`, `seed` |
| `sizes` | list of `WxH` | `["1280x720"]` | Each from 1280x720 to 7680x4320 |
| `settings` | object | `{}` | Merged over the scratch profile. Refused: `mute` false, `start_inactive`, `window_size`, `fullscreen`, `borderless`, `window_mode` |
| `timeout_s` | whole number | 300 | 1 to 3600, for the whole run at one size |
| `steps` | list | required | Not empty; at most 500 with every `repeat` and `times` counted out |

An unknown key or step is a problem, not a thing ignored.

```json
{
  "id": "menu_opens",
  "title": "The main menu opens the new game screen",
  "steps": [
    {"step": "wait", "until": {"path": "screen", "eq": "menu"}},
    {"step": "shot", "name": "menu"},
    {"step": "click", "id": "menu.new_game"},
    {"step": "wait", "until": {"widget": "new_game.start"}}
  ]
}
```

## Steps

| `step` | Keys | Does |
| --- | --- | --- |
| `click`, `hover` | `id` | The link request; a reply with `ok` false fails the step |
| `key` | `key`, `times` = 1 | Key presses, the frames waited after each |
| `type` | `text` | Into the field being edited |
| `drag` | `id`, `to` | Press, move, release; `to` is a widget id or `[x, y]` |
| `back` | - | Back |
| `link` | `body` | Any other request, as `{"cmd": ...}`: battle commands, screen commands. Not `quit` |
| `wait` | `until`, `timeout_ms` = 10000 | Polls until true; a timeout fails |
| `assert` | `until` | True now or fail |
| `shot` | `name`, `max_w` = 1920 | Screenshot into the bundle, once the frames after the last change are drawn |
| `audit` | `scales`, `fail` = all of `cut`, `edge`, `overlap`, `spill` | The text audit per text size; a kind in `fail` found fails the step |
| `repeat` | `times`, `steps`, `until` | A bounded loop. `until` is read before each turn and ends it; turns running out with `until` still false fails |

After each action the driver waits for two more frames. The link replies before the frame is drawn, so nothing needs a
sleep; to wait for a result, use `wait`.

## Conditions

```json
{"path": "state.location", "eq": "harbour"}
{"path": "gold", "source": "game", "gt": 0}
{"widget": "results.continue"}
{"all": [{"path": "screen", "eq": "shop"}, {"path": "stack", "has": "town"}]}
```

| Op | Holds when |
| --- | --- |
| `eq`, `ne` | The value equals, or does not; a missing path fails both |
| `gt`, `lt` | The value is a number above or below |
| `has` | A list holds the value, an object has the key, a text contains it |
| `absent` | Written `"absent": true`; the path is missing or null |

Paths are dotted, with numbers for list places (`state.units.0.hp`), and read the `ui_tree` reply. `"source": "game"` reads
the `game` reply. Only `absent` passes on a missing path, so a mistyped path never reads as true.

## What a run owns

| Thing | Rule |
| --- | --- |
| Port | From the OS per window. If it answers for another process, that process is left alone and the next port is tried, 3 at most |
| Profile | A scratch folder as the game's `APPDATA` and `saves=`, removed after; the real profile is never read or written |
| Settings | `mute` true, windowed at the size, `vsync` false, `record_playtests` true, `start_inactive` true |
| Process | Only the one it started: asked to leave over the link, killed after 5 s times `slow`. A run that is itself killed leaves its window open |

## The bundle

`<out>/<id>/<WxH>/`, default `out` is `.agent/scenario`. A re-run replaces the folder if it holds a `result.json`.

| File | When | Holds |
| --- | --- | --- |
| `result.json` | Always | `result`, `exit`, `step`, `message`, `px`, `canvas`, `capped`, `game_exit`, `steps`, `frames`, `seconds`, `trail` (flag counts) |
| `stderr.txt` | Once started | The game's output |
| `recording.json`, `trail.json` | When it recorded | The playtest recording and its trail report |
| `<name>.png` | `shot` | The canvas |
| `audit_*.json` | `audit` | One reply per text size |
| `fail.png`, `fail_tree.json` | A failed step, the game still alive | The canvas and the `ui_tree` reply |

`capped` means the window was not at the asked size when the steps began, so `px` is the truth. There are three causes, which it does not tell
apart: the size does not fit the usable area, the game moved it to the nearest size it offers, or the window took longer than 1.5 s times `slow`.

`game_exit` holds `code`, `killed` and `panicked`. A run whose steps all passed still fails, at step `stop`, if the game then exits with a
code other than 0, has to be killed, or wrote a panic to stderr at any point in the run.

## The window during a run

Measured on Windows 11 with a dev build on a loaded machine, 2026-11-04 (INTEG1-E1). Other platforms are not checked.

| Situation | What happens |
| --- | --- |
| Opening | The foreground window never changed. The window opens directly under the foreground window, so it does not cover what the user is working in |
| Fully behind another window | It keeps drawing at the same rate, about 250 to 300 frames a second, and the link answers |
| Minimized, by hand or by Show desktop | The link still answers but the canvas is 1 x 1, so the run fails and says so. A restored window draws normally |

A run takes most of one CPU core, covered or not: 89 to 104 % with `vsync` off and about 80 to 90 % with it on. The setting `fps_cap` does not slow it.

## Proving what the player sees

A link request can reach a result without the screen ever showing it: a game's command API may resolve work in one request. To prove a
screen, drive what the player drives (`click`, `key`) and `wait` on the state. A game's own notes on this lie beside its scenarios, in
`games/<id>/scenarios/README.md`.

## Not here yet

Timing (`perf`) comes with INTEG1-E2. Region map walking (`walk_to`) comes with INTEG1-G. There are no save fixtures, headless
runs or pixel asserts.
