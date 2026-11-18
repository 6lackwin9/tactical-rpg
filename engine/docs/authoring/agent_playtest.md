# Agent playtest: playing the real game one command at a time

`aegis playtest` (PR7-22, AGENT-PLAYTEST1) lets an agent session play a game's real program the way a player would:
look at the screen, click or press, wait for the result, take a picture, and judge. It is an engine tool, so any game
can use it. It sends the same agent link requests as a [scenario](scenarios.md) and adds no game logic; where a scenario
is a fixed test, a playtest is driven step by step and keeps a log of what happened.

## Commands

| Command | Does |
| --- | --- |
| `tools/fc.sh playtest [start] [save=<dir or file>] [out=<dir>] [size=WxH] [seed=N] [shots=each or off] [window=1] [idle_exit=S] [profile=dev or release]` | Builds the game and the CLI, starts the game with no window, waits for its first frames, prints the run folder |
| `tools/fc.sh playtest do <command ...>` | One command on the running game: a log line, a shot after it, and what it found printed |
| `tools/fc.sh playtest run <script or -> [the keys of start]` | Starts, runs every line of the file (`-`: stdin), stops; the first failed line ends it |
| `tools/fc.sh playtest stop` and `status` | Ends the game, or says whether it runs |

The same verbs work on the CLI as `aegis playtest ...` (for example `target/debug/aegis playtest do screen`), which skips
the build. Exit codes: 0 ok, 1 the command failed (a refused click, a wait that timed out, a failed assert), 2 could not
start, a bad line, or no game running.

`start` keys: `save=` copies a saves folder (or one save file) into the run's profile before the game starts, so a
mid-campaign save can be played from. `shots=each` (default) takes a picture after every command that may change the
screen; `off` takes one only on `shot` and on a failure. `idle_exit=` is how long the game waits for a command before it
leaves (default 3600 s for `start`, 300 s for `run`), so a session left alone does not leave a game behind. `window=1` runs a
real muted window, which has no idle exit (close it with `stop`) (it needs a display; on a machine without one, start `Xvfb` and set `DISPLAY` first). `screen=`, `seed=`,
`scene=`, `data=`, `exe=` and `fps=` reach the game as in a scenario.

## The run folder

`<out>/<YYYY-MM-DD>/` (`-2`, `-3`... when taken), default `out` `playtests/agent` (git-ignored); `<out>/current` names the
newest, which `do`, `stop` and `status` use (`run=<dir>` names another).

| File | Holds |
| --- | --- |
| `log.md` | One table row per command: number, time (UTC), the command, ok or FAIL, what it found, the screen after, the shot |
| `log.jsonl` | The same rows as JSON lines |
| `NNN_<command>.png` | The shot after command `NNN`; `NNN_<name>.png` for `shot <name>`; `NNN_fail.png` and `NNN_fail_tree.json` on a failure |
| `NNN_tree.json` | The whole `ui_tree` reply, from `tree` |
| `saves/<name>/` | The saves folder kept by `save <name>` |
| `profile/` | The game's scratch user folder (settings muted, saves, the playtest recording) |
| `game.log`, `session.json` | The game's output; the port, process and step count `do` reads |

The run folder is the hand-off: a report written beside `log.md` links the shots by their file names. To pass it to
another session, copy the folder somewhere shared (in the cloud, the project files folder).

## The language

One command per line (`do` takes one; `run` takes a file of them). Blank lines and `#` lines are skipped.

| Command | Does |
| --- | --- |
| `screen` | The top screen, the stack, its state in short, the clickable widgets (id and text) and the labelled ones. A family of more than 12 clickable widgets with one id prefix (a board's `tile.*`) prints as one line |
| `tree` | The whole `ui_tree` reply into `NNN_tree.json` |
| `click <id>`, `hover <id>` | The widget by id |
| `key <key> [n]` | A key, n times: `up`, `down`, `left`, `right`, `enter`, `space`, `esc`, `tab`, `backtab`, `pageup`, `pagedown` or a character; editing keys while a field edits |
| `type <text>` | Into the field being edited (click it first) |
| `back` | Back, as Esc |
| `drag <id> <to id or x,y>` | Press, move, release |
| `link <json>` | Any other link request, `{"cmd": ...}`: a screen's own commands. Not `quit` |
| `wait <condition> [ms=N]` | Until the condition holds (default 10000 ms, at most 600000) |
| `assert <condition>` | Holds now, or the command fails |
| `get <path>` | A value of the `ui_tree` reply; `get game:<path>` of the `game` reply |
| `shot [name]` | A picture now |
| `pause <ms>`, `frames <n>` | Time passes; n frames drawn |
| `save <name>`, `load <name>` | Keep the saves folder as `saves/<name>`; put it back and start the game again on it (at the main menu) |
| `note <text>` | A line of your own in the log: a judgement or a finding, nothing sent |
| `repeat <n> until <condition> : <command>` | The command until the condition holds, at most n times (the colon between spaces) |
| `quit` | The game leaves |

After every action the driver waits for two more frames, so the shot and the next look show what the action changed;
nothing needs a sleep.

Conditions are the scenario conditions ([scenarios.md](scenarios.md) "Conditions") in a short form, or the JSON form
itself:

| Short form | Holds when |
| --- | --- |
| `screen=<id>` | The top screen is `<id>` (any path works the same: `state.outcome=won`) |
| `widget=<id>` | The widget is on screen |
| `<path>!=<v>` | The value is not `<v>` (a missing path fails) |
| `<path>><n>`, `<path><<n>` | A number above or below n |
| `<path>~<v>` | A list holds `<v>`, an object has the key, a text contains it |
| `<path>?` | The path is missing or null |
| `game:<path>...` | Any of the above on the `game` reply |
| `{"all": [...]}` | The JSON form, for several at once |

A value reads as JSON when it can (`3`, `true`, `null`, `""`), else as text.

## Example

```text
$ tools/fc.sh playtest
§ started .../emberwake (pid 4242, port 38679); run folder playtests/agent/2026-11-18
$ target/debug/aegis playtest do screen
§ 001 ok screen [screen main_menu]
screen: main_menu
clickable (6):
  menu.continue  "Continue"
  menu.new_game  "New game"
  ...
$ target/debug/aegis playtest do click menu.new_game
§ 002 ok click menu.new_game: clicked="menu.new_game" [screen new_game]
§ shot playtests/agent/2026-11-18/002_click_menu_new_game.png
$ target/debug/aegis playtest do note the new game screen shows no world description
$ target/debug/aegis playtest stop
```

A game's smoke script lies in its folder (`games/<id>/playtests/agent/smoke.play`); `tools/fc.sh gate` runs it, so the
harness and the path it walks stay working.
