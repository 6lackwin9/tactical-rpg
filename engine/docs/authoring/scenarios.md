# Scenarios: walking a game's real screens

`aegis scenario` plays a game with no window, at a chosen size: JSON files click, wait on state and assert. It is an engine
tool, so any game can use it. The spec is `docs/specs/AGENTROLES1.md` section 4; the mode with no window is HEADLESS1.

A scenario checks that the built game still works end to end. A rule or a screen's own behaviour belongs in a test.

## Commands

| Command | Does |
| --- | --- |
| `aegis scenario check [<file> or all]` | Parses the files. No game starts. Exit 1 on a problem, each naming the file and step |
| `aegis scenario run <id, all or a .json path> [size=WxH] [out=<dir>] [slow=N] [exe=<program>] [data=<dir>] [window=1] [fps=N] [dpr=X]` | Runs them |
| `aegis scenario shots <name> [screen=] [scene=] [ruleset=] [seed=] [do=<a>,<b>,...]`, plus the keys of `run` (`encounter=` accepted as alias; `rules=` refused on CLI) | Proof shots of a fight or screen, with no file to write |
| `tools/fc.sh scenario check [<file> or all]` | The same check |
| `tools/fc.sh scenario <id, all or a .json path> [profile=dev or release]`, plus the keys of `run` | Builds the game and the CLI in one profile, then runs |
| `tools/fc.sh scenario shots <name>`, plus the keys of `shots` | The same for proof shots |

Exit codes: 0 pass, 1 a step failed, 2 could not start.

The files lie in `games/<id>/scenarios/<id>.json`; the folder names the game, the file names nothing else. The game's
program is the file named as the manifest's `id`, beside the CLI (`exe=` says otherwise). `size=` replaces the file's sizes.
`slow=N` multiplies every timeout, 1 to 20, for a machine under load. `data=` opens a scratch copy of the data folder.
`window=1` runs every scenario in a real window. `fps=` (12 to 240, default 15) and `dpr=` (0.5 to 4, default 1) are for a run
with no window. `tools/fc.sh gate` runs every scenario of the registered game (the one the CLI is built for) with no window (PR7-4), besides a game's own test that parses its
files with `check_files` and the driver's tests on a fake link; `aegis scenario check` is for a file being written.
`record=1` writes the baselines of the golden shots it passes (see "Golden shots").

## The file

| Key | Type | Default | Rule |
| --- | --- | --- | --- |
| `id` | text | required | Equals the file's name; letters, digits, `_`, `-` |
| `title` | text | "" | - |
| `status` | `active`, `waiting` | `active` | `waiting` needs `why`; it is parsed and never run |
| `why` | text | - | Why it waits |
| `launch` | object | `{"screen": "menu"}` | `screen` is any screen id the game opens on (its `Game::launch_screens`, checked by `check`, `run` and `shots`; Emberwake: `menu`, `arena`, `runner`); optional `scene` (alias `encounter`), `seed`, `ruleset` (alias `rules`; the game says which screens take it, Emberwake: `arena`), and `save`: a folder name under `scenarios/saves/`, copied into the scratch saves before the start (PR7-4) |
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
| `key` | `key`, `times` = 1 | Key presses, the frames waited after each; through the player's control map (`controls` in the settings) as a real key is |
| `type` | `text` | Into the field being edited |
| `drag` | `id`, `to` | Press, move, release; `to` is a widget id or `[x, y]` |
| `back` | - | Back |
| `link` | `body` | Any other request, as `{"cmd": ...}`: the game's own commands, screen commands. Not `quit` |
| `wait` | `until`, `timeout_ms` = 10000 | Polls until true; a timeout fails |
| `assert` | `until` | True now or fail |
| `shot` | `name`, `max_w` = 1920, `golden`, `t` = 0 | Screenshot into the bundle, once the frames after the last change are drawn. A blank frame fails the step. `golden`: compared with its baseline (below). `t`: no window only, seconds (0 to 60) the top screen has been open first, on the shot clock (below): a screen that fades in takes the time of its fade |
| `clip` | `name`, `frames` = 30, `every_ms` = 66, `max_w` = 1280 | No window only. A numbered sequence, `<name>/frame_0001.png` on: at most 120 frames, 10 to 2000 ms apart, 15 s in all |
| `perf` | `name`, `frames` = 300, `warm` = 60, `budget` | No window only. Frame times (ENGINE-2): see "Frame times" |
| `pause` | `ms` | Time passes, for a picture at a moment of an animation, with a look at the game each second; with no window, that much game time on the shot clock, drawn at once. To wait for a result, use `wait` |
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

## Golden shots

A `shot` with `"golden": true` (exact) or `"golden": {"tolerance": T, "pixels": N}` compares the frame with the committed
baseline `<game folder>/scenarios/baselines/<scenario id>/<WxH>/<name>.png`. A pixel differs when one RGBA channel is more
than `T` (0 to 255, default 0) apart; the step fails when more than `N` pixels (default 0) differ, naming the count, the
largest difference and their box, and keeps `<name>.diff.png`. A missing baseline fails too.

`aegis scenario run <id> record=1` (or `tools/fc.sh scenario <id> record=1`) writes the baselines instead, once every step
has passed, and lists them in `result.json` under `recorded`. A new or changed baseline needs the user's approval: it lands as a named re-record in the
commit and a `docs/HISTORY.md` row, like an engine golden.

## The shot clock

A run with no window is on the shot clock (ENGINE-17, `aegis_core::clock`): the game's time moves only by the frames the
driver asks for, one frame step (1 / fps) each, and the date starts at 2026-01-01 (`clock::system_now`, which a new
game's clock seed reads). The start, each action, each `wait` look (one frame), a `pause` and a shot's `t` ask for set
frames, so two runs of one build draw the same frames and their shots are byte for byte the same; the gate's
`shotsexact` step runs every scenario twice and checks that. An animation, a seed or a timer that reads the real clock
(`std::time` outside `aegis_core::clock`, a thread's own timing) breaks it, and that step names the shot. Golden shots
therefore need no `pixels` allowance for idle sprites; a window run is on real time and never frame-exact.

## Frame times

A `perf` step (no window only) draws `warm` frames untimed, then times `frames` whole frames back to back (1 to 2000; `warm`
0 to 600), with a fixed CPU workload (the calibration) timed before and after. Its numbers go to `perf_<name>.json` in the
bundle and under `perf` in `result.json`: `stats` (`p10`, `p50`, `p95`, `max`, `mean` in ms, `count`), `calib_before`,
`calib_after`, `steady`, `canvas`, `build` and `verdict`.

| `verdict` | When |
| --- | --- |
| `recorded` | No `budget` |
| `within` | Every number named in `budget` (`p50_ms`, `p95_ms`, `max_ms`) is at or under it |
| `over` | One is over it on a steady machine: the step fails, naming it |
| `no verdict` | The two calibrations differ by more than 20 %: the machine's load changed, so the numbers judge nothing and the step passes |

A frame with no window is the whole frame's cost (nothing is presented), not the frame rate. A dev build is several times
slower than a release one, so a budget in a scenario the gate runs is a loose ceiling that catches a gross regression; a tight
one belongs in a release run (`tools/fc.sh scenario <id> profile=release`). The painter's own costs (fill, panels, text) are
`tools/fc.sh bench`.

## The bundle

`<out>/<id>/<WxH>/`, default `out` is `.agent/scenario`. A re-run replaces the folder if it holds a `result.json`.

| File | When | Holds |
| --- | --- | --- |
| `result.json` | Always | `result`, `exit`, `step`, `message`, `window`, `px`, `canvas`, `capped`, `game_exit`, `steps`, `frames`, `seconds`, `trail` (flag counts) |
| `stderr.txt` | Once started | The game's output |
| `recording.json`, `trail.json` | When it recorded | The playtest recording and its trail report |
| `<name>.png` | `shot` | The canvas |
| `<name>.diff.png` | A golden shot that failed | The baseline darkened, the differing pixels red |
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

## Settings, audits and saves for a whole run

Three keys change a run without touching its files (ENGINE-33, ENGINE-36). `set=` and `audit=` go on `run`,
`tools/fc.sh scenario` and `shots-all`; `save=` goes on `shots`:

| Key | Does |
| --- | --- |
| `set=<key>:<value>,...` | Settings laid over each scenario's own `settings`, for example `set=text_scale:1.3` takes every screen at a larger text size. The keys a file may not set are refused here too |
| `audit=1` | After every shot, the text audit of that frame (cut, edge, overlap, spill) is kept beside it as `<shot>.audit.json`; `shots-all` collects them into `audit.txt`, the shots with a problem first |
| `save=<name or absolute folder>` | On `shots`: launch from a save. A name is a folder under `scenarios/saves/`; an absolute path is any saves folder, such as one an agent playtest kept (`keep_saves=`, [agent_playtest.md](agent_playtest.md)). A file's `launch.save` takes the same two forms |

A sweep at two text sizes, compared shot by shot:

```sh
tools/fc.sh scenario shots-all out=target/xl_normal audit=1
tools/fc.sh scenario shots-all out=target/xl_large set=text_scale:1.3 audit=1 compare=target/xl_normal
```

## The shots a change moves

`tools/fc.sh shots diff [base=<sha>] [scenarios=a,b or all] [out=<dir>] [tolerance=0] [pixels=0] [limit=12]` shows what a
change does to the screens before it lands (SHOT-DIFF1):

- With no `scenarios=`, it picks the scenarios the change can show from the paths changed since the base (committed or
  not, new files too): only the changed scenario files when nothing else changed, else all; a docs-only change runs
  nothing.
- It runs them on the working tree and on the base commit (default `origin/develop`). The base's shots are cached in
  `target/shots_cache/<sha>/`, so the next diff against that commit runs only the working tree.
- Shots run on the shot clock, so a comparison is exact by default; `tolerance=` and `pixels=` loosen it.
- The out folder (default `target/shots_diff/`) holds `index.md` (every changed, new and gone shot, most changed first),
  `sheet.png` (before, after and diff of the most changed shots at half size), `compare/` (each at full size) and
  `head/`.

Put the sheet beside a change's report; a reviewer sees the screens without running anything.

## Not here yet

Timing (`perf`) comes with INTEG1-E2. Region map walking (`walk_to`) comes with INTEG1-G. There are no save fixtures or pixel
asserts.
