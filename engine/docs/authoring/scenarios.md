# Scenarios: walking a game's real screens

`aegis scenario` plays a game with no window, at a chosen size: JSON files click, wait on state and assert. It is an engine
tool, so any game can use it. The spec is `docs/specs/AGENTROLES1.md` section 4; the mode with no window is HEADLESS1.

A scenario checks that the built game still works end to end. A rule or a screen's own behaviour belongs in a test.

## Commands

| Command | Does |
| --- | --- |
| `aegis scenario check [<file> or all]` | Parses the files. No game starts. Exit 1 on a problem, each naming the file and step |
| `aegis scenario run <id, all or a .json path> [size=WxH] [out=<dir>] [slow=N] [exe=<program>] [data=<dir>] [window=1] [fps=N] [dpr=X]` | Runs them |
| `aegis scenario shots <name> [screen=] [scene=] [ruleset=] [seed=] [do=<a>,<b>,...]`, plus the keys of `run` (`encounter=` and `rules=` accepted as aliases) | Proof shots of a fight or screen, with no file to write |
| `tools/fc.sh scenario check [<file> or all]` | The same check |
| `tools/fc.sh scenario <id, all or a .json path> [profile=dev or release]`, plus the keys of `run` | Builds the game and the CLI in one profile, then runs |
| `tools/fc.sh scenario shots <name>`, plus the keys of `shots` | The same for proof shots |

Exit codes: 0 pass, 1 a step failed, 2 could not start.

The files lie in `games/<id>/scenarios/<id>.json`; the folder names the game, the file names nothing else. The game's
program is the file named as the manifest's `id`, beside the CLI (`exe=` says otherwise). `size=` replaces the file's sizes.
`slow=N` multiplies every timeout, 1 to 20, for a machine under load. `data=` opens a scratch copy of the data folder.
`window=1` runs every scenario in a real window. `fps=` (12 to 240, default 15) and `dpr=` (0.5 to 4, default 1) are for a run
with no window. No scenario runs in the gate. The gate runs a game's own test that parses its files with `check_files`, and the
driver's tests on a fake link; `aegis scenario check` is for a file being written.

## The file

| Key | Type | Default | Rule |
| --- | --- | --- | --- |
| `id` | text | required | Equals the file's name; letters, digits, `_`, `-` |
| `title` | text | "" | - |
| `status` | `active`, `waiting` | `active` | `waiting` needs `why`; it is parsed and never run |
| `why` | text | - | Why it waits |
| `launch` | object | `{"screen": "menu"}` | `screen` is `menu`, `arena` or `runner`; optional `scene` (alias `encounter`), `seed`, and `ruleset` (alias `rules`) with `arena` |
| `sizes` | list of `WxH` | `["1280x720"]` | Each from 1280x720 to 7680x4320 |
| `settings` | object | `{}` | Merged over the scratch profile. Refused: `mute` false, `start_inactive`, `window_size`, `fullscreen`, `borderless`, `window_mode` |
| `timeout_s` | whole number | 300 | 1 to 3600, for the whole run at one size |
| `window` | true or false | false | True: this one needs a real window (see "When a real window is needed"); it then has no `clip` |
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
| `shot` | `name`, `max_w` = 1920 | Screenshot into the bundle, once the frames after the last change are drawn. A blank frame fails the step |
| `clip` | `name`, `frames` = 30, `every_ms` = 66, `max_w` = 1280 | No window only. A numbered sequence, `<name>/frame_0001.png` on: at most 120 frames, 10 to 2000 ms apart, 15 s in all |
| `pause` | `ms` | Time passes, for a picture at a moment of an animation, with a look at the game each second. To wait for a result, use `wait` |
| `audit` | `scales`, `fail` = all of `cut`, `edge`, `overlap`, `spill` | The text audit per text size; a kind in `fail` found fails the step |
| `repeat` | `times`, `steps`, `until` | A bounded loop. `until` is read before each turn and ends it; turns running out with `until` still false fails |

After each action the driver gets two more frames: with no window it asks for them and they are drawn before the reply, and
in a window it polls until they have come. Nothing needs a sleep; to wait for a result, use `wait`.

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
| Port | From the OS per start. If it answers for another process, that process is left alone and the next port is tried, 3 at most |
| Profile | A scratch folder as the game's `APPDATA` and `saves=`, removed after; the real profile is never read or written |
| Settings | `mute` true, windowed at the size, `vsync` false, `record_playtests` true, `start_inactive` true |
| Sound | No window: no device is opened or needed. A real window: muted |
| Process | Only the one it started: asked to leave over the link, killed after 5 s times `slow`. If the driver itself is killed, a game with no window leaves on its own (see below); a real window stays open until it is closed |

## The bundle

`<out>/<id>/<WxH>/`, default `out` is `.agent/scenario`. A re-run replaces the folder if it holds a `result.json`.

| File | When | Holds |
| --- | --- | --- |
| `result.json` | Always | `result`, `exit`, `step`, `message`, `window`, `px`, `canvas`, `capped`, `game_exit`, `steps`, `frames`, `seconds`, `trail` (flag counts) |
| `stderr.txt` | Once started | The game's output |
| `recording.json`, `trail.json` | When it recorded | The playtest recording and its trail report |
| `<name>.png` | `shot` | The canvas |
| `<name>/frame_NNNN.png` | `clip` | The canvas over time |
| `audit_*.json` | `audit` | One reply per text size |
| `fail.png`, `fail_tree.json` | A failed step, the game still alive | The canvas and the `ui_tree` reply |

`capped` means a real window was not at the asked size when the steps began, so `px` is the truth. There are three causes, which it does not tell
apart: the size does not fit the usable area, the game moved it to the nearest size it offers, or the window took longer than 1.5 s times `slow`.
A run with no window is at the asked size from its first frame.

`game_exit` holds `code`, `killed` and `panicked`. A run whose steps all passed still fails, at step `stop`, if the game then exits with a
code other than 0, has to be killed, or wrote a panic to stderr at any point in the run.

## With no window

The same game loop, screens and agent link at the chosen size, the frames left in the shell's software canvas. No `winit` window,
GPU surface or sound device exists, so it needs no display. A game selects it with the launch arguments `headless=WxH`, `fps=N`, `dpr=X`
and `idle_exit=S` (`aegis_ui::shell::headless::Headless::from_args`, handed to the shell as `Options::headless`). `ui_tree` answers as in a window.

| Thing | Rule |
| --- | --- |
| Frames | At most `fps`, 15 by default. A screen at rest draws at its `Screen::frame_need` (`aegis_ui::pacing`), so a resting menu draws about one a second |
| The link | Answered at once, and it never forces a frame as it does in a window. A request keeps frames coming for 250 ms |
| `{"cmd": "frames", "frames": n}` | Draws 1 to 240 frames before the reply. Any other name a screen answers (a battle's `step`, which plays a turn) still reaches it |
| `{"cmd": "clip", "dir": ..., "frames": n, "every_ms": m, "max_w": w}` | The numbered sequence; it replies when done, with `saved`, `blank` and `ms`. It refuses a folder that holds frames, and stops and fails past 30 s or 256 MB. The screen keeps running; other requests wait |
| Clocks | Wall time, as in a window, so `frames` does not move time on. Under 12 fps is refused: a clock that limits one frame's step would fall behind |
| No request for a while | The screens leave and the game exits with an error on stderr, after 120 s by default. `idle_exit=S` (1 to 86400) sets it for a game driven by hand. Drawing does not count, a long request does |
| A port already taken | The game exits with an error |

A timer that runs out between two frames fires at the next frame, so timed playback runs a little long at a low rate. Measured at 1280x720 in a dev build
with nothing asking, 2026-11-05: a resting menu 0.3 % of a core at 1.2 fps, a battle waiting for input 13.6 % at 14.4 fps.

### Blank frames

A shot of a canvas under 64 px on a side, or of one colour all over (within 3 on each channel), is refused and writes nothing: the
`screenshot` reply has `ok` false and the step fails. This holds in a window too, so a minimized window's 1 x 1 canvas never makes a blank PNG. A `clip`
skips a blank frame, leaving a gap in the numbers and its number under `blank`; a clip of nothing but blank frames fails. To shoot past a fade, `wait`
for the state after it.

### Proof shots

`aegis scenario shots <name>` is a scenario with no file, written to `.agent/shots/<name>/<WxH>/` by default. It prints each picture.

```bash
tools/fc.sh scenario shots menu
tools/fc.sh scenario shots fight screen=arena scene=tl_flank ruleset=timeline seed=7 do=shot:start,key:a,clip:play:12:250,pause:500,shot:mid
```

| In `do=` | Step |
| --- | --- |
| `key:<k>`, `click:<id>`, `hover:<id>` | The same step |
| `screen:<id>`, `widget:<id>` | `wait` until that screen is on top, or that widget exists |
| `pause:<ms>` | `pause` |
| `shot:<name>` | `shot` |
| `clip:<name>[:<frames>[:<every_ms>]]` | `clip` |

With no shot or clip in the list, one shot named `shot` ends it. To compare two shots, run the bin `shot_compare <a.png> <b.png> [s=1]` (`aegis_ui::shot_diff`; exit 0
match, 1 differ, 2 could not compare). A performance probe is INTEG1's.

## When a real window is needed

Set `"window": true` in the file, or pass `window=1`, for what only a window has:

| Need | Why |
| --- | --- |
| Window behaviour | Sizes capped to the screen, fullscreen, borderless, focus, restoring from minimized: a run with no window has none of them |
| Real input | The OS's mouse and keys, the pointer moved onto a widget, held walk keys: the link does not pass through them |
| Final performance numbers | Frames at the real rate, with presenting: 15 fps in memory says nothing about what a player gets |

None of the scenarios in `games/emberwake/scenarios/` needs one.

## The window during a run

Measured on Windows 11 with a dev build on a loaded machine, 2026-11-04 (INTEG1-E1). Other platforms are not checked. This is a real window's behaviour only.

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

Timing (`perf`) comes with INTEG1-E2. Region map walking (`walk_to`) comes with INTEG1-G. There are no save fixtures or pixel
asserts.
