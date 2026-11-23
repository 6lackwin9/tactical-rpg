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
| `feel` | `name`, `script`, `gap` = 2, `budget`, `step_ms`, `motion` = false, `switch` = false | No window only. Input times (ENGINE-45), movement (ENGINE-49) and screen switches (ENGINE-51): see "Input times" |
| `reach` | `name`, `back` = false, `fail` = false, `allow` = [] | No window only. What keys and a pad reach (ENGINE-48): see "Input reach" |
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
| Sound | No window: a virtual output device on its own thread (ENGINE-55), so `feel` hears sounds while muted. A real window: muted |
| Developer surfaces | `dev=1` on every launch (ENGINE-57): a game with dev tools in the build shows them in a release run too, as the gate's dev build does. One script then takes the same path in either profile. A build without the dev tools ignores the key |
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

## Input times

A `feel` step (no window only, ENGINE-45 FEEL-PROBE1) replays `script` one input at a time and times each with the shell's
feel probe (`aegis_ui::feel`). An input is `{"key": name}`, `{"click": widget id}` or `{"pad": button}` (a name the `pad`
link request takes: `south`, `east`, `dpad_up`...), or `{"frames": n}` (1 to 240 frames drawn, no input), each with an
optional `times` (1 to 200). After each input the next frame is drawn, then `gap` more (0 to 240) before the next input.
Each input gets: `to_state_ms` (arrival to its handler done), `to_frame_ms` (to the start of the frame that shows it),
`to_present_ms` (to that frame done), `frame_ms`, the `tick_ms` and `draw_ms` the screens took in it, and `spent_in` (the
largest of `handler`, `wait` and `frame`). The step writes `feel_<name>.json` to the bundle and adds it under `perf` in
`result.json` (`kind` `feel`): `to_state`, `to_present` and `frame_time` each as `count`, `p50`, `p95` and `worst`, every
input, `flagged_inputs` (input-to-present over 100 ms) and `frames_over` (frames over 33 ms). `budget` takes `input_ms`
(the worst input-to-present), `frame_ms` (the worst frame) and `frame_p50_ms` (the median frame, ENGINE-50); one over it
fails the step. In a dev build (the gate's) each
budget counts 2.5 times (`FEEL_DEV_ALLOWANCE`, ENGINE-46): a dev build draws about twice as slow, so the gate catches a gross
slip while the release run holds the real bar. A refused input stops the step with its place.

Movement (ENGINE-49 WALK-FEEL1): the script also takes `{"press": key}` and `{"release": key}` (a movement key held down
and let go, as a window's key down and up: `Held::dir` follows the latest held), `{"pad_press": button}` and
`{"pad_release": button}` (a d-pad direction the same way), `{"wait_ms": n}` (frames drawn for n ms of the run's clock,
1 to 60000), `{"spike_ms": n}` (one frame that takes n ms, 1 to 2000) and `{"stall_ms": n}` (ENGINE-50 MAP-FRAME1: the
clock moves n ms with no frame drawn, 1 to 2000, so the next input arrives during a hitch and its frame comes after it). `step_ms` (1 to 200) sets the shot clock's frame
step for the replay (16.667 for 60 frames a second). With `motion` true the top screen's `Screen::motion` is read after
every frame and the reply's `motion` (`aegis_ui::walkfeel::report`) gives, per input, `to_move_ms` (to the first frame the
mover is drawn elsewhere) and `to_step_ms` (to its first new cell), a turn's `turn_ms` and `turn_gap_ms` (a press while
another direction is held), `after_release_steps`, a long frame's `next_steps_ms`; and over the run `first_move_ms` (the
worst first move; a press that moves nothing, into a wall, counts under `still`), `cadence_ms` and `cadence_spread_ms`
(the gaps between steps with no input between them; the stretch after a long frame is read apart), `overshoot_steps`,
`stutters` (a turn later than a usual gap allows while walking), `camera_lag_tiles` (the camera's distance from where it
wants to be) and `camera_jump_tiles` (its largest move in a frame, per frame of `step_ms`), and `path` (each step's time
and cell). `budget` takes each of `first_move_ms`, `overshoot_steps`, `cadence_spread_ms`, `stutters`,
`camera_lag_tiles` and `camera_jump_tiles`; one over fails the step, as does a screen that reports no motion.

Screen switches (ENGINE-51 SWITCH-FEEL1): with `switch` true each key, click or pad input is followed, frame by frame,
until its new screen settles (`aegis_ui::switchfeel`), and the reply's `switch` gives per input `from` and `to` (top
screen and stack depth), `handler_ms`, `first_ms` (to the first frame of another screen, or the same at another depth),
`settled_ms` (to the first frame of the last screen it came to drawn in 33 ms, 82.5 in a dev build: its loads are done), `own_ms` (from the old
screen's last frame to settled: a walk to an exit or the old screen's fade out left out), `chain` (a story card that
hands over to a stage, the stage to a fight), `worst_frame_ms`, `long_frames` (over 100 ms, the handler counted with the
first; each with `covered_by`, the transition shown before it, `Screen::transition`, or null) and `transitions`; over the
run `worst_settled_ms`, `uncovered` and `misses`. A script item takes `switch_ms` (the input's `own_ms` past it fails
the step, as does no other screen coming up) and `to` (the screen it is meant to reach: it settles on that screen's first
quick frame and is followed no further, so a fight that opens on its own after a results screen is not part of it; an
input with a `to` also waits for a walk to the exit, up to 600 frames, where one without stops after 180 frames with no
other screen). `budget` takes `uncovered_frames` (long frames no transition covered). The dev build's allowance stretches
`switch_ms` and the 100 ms alike.

Sound (ENGINE-55 AUDIO-FEEL1): every `feel` reply also has `audio` (`aegis_ui::audiofeel`). With no window the sound
plays into a virtual output (`aegis_ui::audio_out`, a 48 kHz stereo device on its own thread asking for 512-frame
buffers on time, so a sound is heard when its buffer plays, even in a muted run). Each sound and track started in the
replay gets `rel`, `bus`, `asked_ms`, `heard_ms` (its first sample over 1/1024), `after_ask_ms`, `after_input_ms` (from
the last input within 250 ms before the ask), `beat_ms` (an effect asked in a frame, heard minus that frame's start plus
one period: when the frame is shown) and `decode_ms` (reading or decoding on the ask); over the run `ui_sound_ms` (the
worst interface sound after its input), `hit_beat_ms` (the worst effect off its frame, either way), `music_ms` (the worst
track after its ask), `decode_ms`, `unheard`, `underruns` (a buffer mixed after it should have played) with
`underrun_ms`, `buffer_frames`, `buffer_ms`, `buffers` and `worst_fill_ms`. `budget` takes each of `ui_sound_ms`,
`hit_beat_ms`, `music_ms`, `underruns` and `decode_ms`; one over, or one nothing measured, fails the step. In a dev build
`decode_ms` counts 2.5 times and `underruns` allows one more per 2 s of replay or part of it (`DEV_UNDERRUN_EVERY_MS`: a starved
debug thread). A window's feel overlay adds the real device's buffer and underruns.

With no window nothing is presented, so a frame is done once drawn; a window adds the copy to the GPU and the present (and
vsync's wait). The same numbers come from a real window with `--feel` (or `feel=1`) on the game's command line, or the
setting `"feel": true`: an overlay at the top left (the last frame's ms, the worst frame of the last second, the last
input's input-to-present ms) and a line per input in `feel.log` beside the run's `game.log`. Use a release build for numbers
a player would see (`tools/fc.sh scenario <id> profile=release`).

## Input reach

A `reach` step (no window only, ENGINE-48 INPUT-REACH1) walks the top screen as a keyboard and as a pad would (the link
request `reach`, `aegis_ui::reach`). A mouse reaches every enabled widget it can click, pick, drag, drop on or type into,
less what a popup's backdrop covers. Keys reach the focus ring (Tab walks all of it). A pad reaches what its buttons map to
in the player's `controls`: the ring when a button gives Tab, else what its d-pad walks to from the opening focus, and only
on a screen whose arrows move the focus (`Screen::arrow_focus`); a focus group (a board, PR7-92) is one Tab stop whose
members the arrows walk. The step writes `reach_<name>.json` to the bundle and adds
it under `perf` in `result.json` (`kind` `reach`): `mouse` (a count), `ring` (a focus group, `Screen::focus_group`, once; its prefixes in `group`), `keys_miss`, `pad_miss` and `dpad_miss` (each
`{id, kind, use}`), `invisible` (ring widgets whose focus draws nothing within `REACH_MARGIN_PX` of their box), `unsure`
(the box changes from frame to frame, so it cannot tell) and, with `back` true, what one Esc did: `quits`, `closed`,
`opened`, `swapped`, `changed` (the same screen drew differently) or `nothing`, and the top screen after it. `dead_ends`
lists `keys <id>`, `pad <id>`, `invisible <id>`, `back nothing` and `back quits`. With `fail` true a dead end fails the step
unless `allow` names its widget id, or `back` for a back press (a root screen where Esc rightly does nothing).

## Reading the feel table

`tools/fc.sh feel [out=<dir>]` (ENGINE-57 FEEL-RELEASE1) runs every scenario that has a `feel` or `reach` step, in each
game, with no window and in a release build. Release budgets apply: none of the dev build's allowances. It then prints one
table (`tools/planner/feel_table.py`) and exits 1 when a scenario failed. It takes a few minutes and is never in the gate.
Run it on a quiet machine, and run a scenario again alone before trusting a single slow row.

Each row is one step, named `<scenario>/<step>`. The columns:

- `result`: `ok`, `FAIL: <why>` on the step that failed, or `not run` for the steps after a failure (also when the
  scenario did not start).
- Times are in ms, and a value over that step's own budget ends in `*`.
  - `input`: the worst input to present.
  - `frame p50` and `frame max`: the median and the worst frame.
  - `switch`: the worst switch's own part.
  - `move`: the worst first move.
  - `ui snd`: the worst interface sound after its input.
  - `hit`: the worst effect off its frame, either way.
  - `music`: the worst track after its ask.
- `underruns` is a count.
- `stalls` is a count (ENGINE-60): the step's stalled frames plus the underruns heard during one. A stalled frame took
  over 33 ms but ran on the CPU for under half of it, so the machine stopped the game rather than the game working that
  long (Linux reads the CPU time; elsewhere nothing is marked). A stall is in no other column and fails nothing.
- `dead ends` is the count on a `reach` row; its other columns are `-`.
- `-` means nothing measured it.

Under the table you get a count of the steps, then one line for each failed scenario with its step number and message,
then one `stall` line for each step with stalls (each stalled frame's wall and CPU ms, and its stalled underruns).
The bundles and each scenario's log stay under `out`. Open `feel_<step>.json` to see which input or frame made a number.
A row that fails here but passes in the gate is over its release budget: the dev build's 2.5 times hides it.

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
