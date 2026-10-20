# Project Aegis Engine

This directory contains the source code for the **Aegis Engine**, an agent-native game engine written in Rust that runs in parallel with the Godot build and aims for feature parity with it.

**Current direction:** [docs/ENGINE_SPEC.md](docs/ENGINE_SPEC.md) (draft, 2026-10-18): headless deterministic sim, one command API shared by GUI and CLI, feel as data, parity with Godot (exact for rules, statistical for battles). All engine work stays under `engine/`; `.gdignore` keeps Godot from scanning it.

* **Original proposal (Markdown-as-language, superseded by the spec):** [ENGINE_PROPOSAL.md](ENGINE_PROPOSAL.md)
* **Active Branch:** `engine`
* **Toolchain:** Rust `stable-x86_64-pc-windows-gnullvm`, pinned in `rust-toolchain.toml`. The MinGW on PATH is llvm-mingw, and the plain `gnu` toolchain fails to link against it (`unable to find library -lgcc_eh`).
* **Build output:** `.cargo/config.toml` puts it in `C:/Users/tusha/AppData/Local/Temp/aegis-target`, outside OneDrive (the folder is several GB). Binaries: `.../aegis-target/release/aegis.exe`, `emberwake.exe`.
* **Game data:** the project's `games/emberwake/data/` folder (catalog, worlds, maps, quests, dialogue services, XP curve), read by the
  engine's own loader (`emberwake_content::source`) with the field tables in `games/emberwake/schema.json`; no Godot export is
  needed to play. `data=<dir>` points the GUI or CLI at another data folder, or at a Godot export folder.
* **Saves and settings:** per game under its id, `%APPDATA%/Aegis/emberwake/saves` and `%APPDATA%/Aegis/emberwake/settings.json` (outside the repo; the first start after SPLIT1-8 copies the old shared `%APPDATA%/Aegis/saves` and `settings.json` there and leaves them; `saves=<dir>` points the GUI elsewhere). The files use Godot's save format: Godot saves load in the engine unchanged. The engine owns the sound, display and text-size settings (`aegis_ui::screen::Settings`) and applies them when they change (PORT-SETTINGS): window mode (windowed at the chosen size, borderless or exclusive fullscreen), vsync, frame-rate cap and text size (Small 0.85 to Extra large 1.3) and the four volumes and mute (PORT-AUDIO); the game adds its own keys.
* **Drag and drop (UIKIT2):** a screen marks a widget draggable with a payload key (`Ui::draggable("stash.3")`) and widgets as drop targets (`Ui::drop_target("slot.weapon")`); `Screen::can_drop` says which targets take a payload (outlined while dragging, gold under the cursor) and `Screen::accept_drop` acts on a release over one. A press on a draggable widget clicks on release when the mouse did not move; a move of more than 3 canvas pixels picks it up (a ghost card follows the cursor, `Ui::dragging` lets the screen preview); Esc, right click or a release elsewhere cancels.
* **Windows (UIKIT3):** a screen can keep a `window::Desk` of windows inside an area (returned from `Screen::desk`): each window has a title bar to drag it, a corner grip to resize it, collapse and close buttons, and a body the screen fills in `Desk::draw` (drawing and widgets clipped to it). A press on a window raises it, and a window covers what lies under it (clicks, hover, drops). `Desk::toolbar` draws a toggle per window and a button per layout preset; the shell handles all of these itself (widget ids `win.<id>.<part>`, `desk.toggle.<id>`, `desk.layout.<name>`) and saves the desk in the settings under `desks.<screen id>` after each change (`Desk::restore` reads it back). Geometry is stored as fractions of the area. Agent link: `windows` lists them, `window {id, op: open | close | raise | collapse | move | size, by: [dx, dy]}` or `window {layout}` drives them.
* **Keyboard focus (PORT-FOCUS):** every screen's clickable widgets, choice fields and sliders take the focus with no
  code of their own (`aegis_ui::focus`, kept in the kit by the shell): Tab / Shift+Tab walk them in drawing order, the
  arrows move to the nearest one that way (a screen that walks or steers with the arrows says `Screen::arrow_focus`
  false: the region map, the journal), Enter or Space presses the focused one (a choice field opens its list and the
  arrows pick in it; Left / Right step a slider). A white ring shows the focus after a key moved it; a click hides it.
  `Screen::auto_focus` starts a keyboard menu focused (the main menu). The agent link's `ui_tree` reports `focus` and
  `focus_visible`; `key` takes `backtab`; the link's `click` hides the ring like a mouse click. The focus starts over
  hidden on every new screen (a screen's own Enter / Space keep working); widgets under a popup or window take none;
  window title bars and toolbars take none yet, and a focused row is not scrolled into view.
* **Scroll areas and rich text (PORT-DOJO):** `Ui::scroll_begin(id, view, content_h)` / `scroll_end` clip content to a view and return the offset to draw it at; the kit keeps each area's offset (an overlay or a return keeps it; a new stack clears them all), the wheel closes an open choice list or scrolls the topmost area under the mouse (a window over it stops it; otherwise the screen's `wheel` gets the lines) and the agent link has `scroll {id, by | to}`. Clips nest (`push_clip` / `pop_clip`). `Painter::rich` draws markup (`[color=#rrggbb]..[/color]`, `[b]..[/b]`) wrapped to a width, `rich_height` measures it, `strip_markup` drops the tags. `Painter::arrow` draws a line (solid or dashed) with an arrowhead.
* **Timeline (TIMELINE1):** `aegis_ui::timeline::Timeline` draws a block timeline: a ruler of numbered blocks with a
  playhead, sections of labelled tracks (one row each, scrolling when they overflow), segments spanning blocks (filled
  or dashed, with a hover tip) and masked tracks (one hatched bar with a line of text). Blocks, track labels and
  segments are clickable (`<id>.block.<b>`, `<id>.track.<key>`, `<id>.seg.<track>.<i>` on the agent link).
* **Sound and music (PORT-AUDIO):** `aegis_ui::audio` plays the game's `assets/audio` files (WAV and Ogg Vorbis, through rodio) on four levels (master times music, effects or interface), 8 effect voices and 2 interface voices (the oldest restarts when all are busy), and one looping music track that crossfades (1 s; 2 s from silence; 6 dB under the effects). Each screen names its track (`Screen::music`); the shell plays the topmost screen's choice, so overlays keep the music below. Emberwake picks tracks with the MUSIC1 chain (`emberwake_gui/src/music.rs`: menu, region by location kind or shop, travel, combat or boss, each tried per region, world and lineage before the plain context) and battle sounds from the resolver log; the settings popup plays the draft levels and a sample per volume slider. Without a sound device everything stays silent.

## Workspace

| Crate | Role |
| :--- | :--- |
| `crates/aegis_core/` | Engine core, game-agnostic (SPLIT1-2): Godot RNG, sort and `hash()`, Variant helpers (`json`), lenient field readers (`value`), Godot number formatting (`fmt::fixed`: `%.Nf` with ties away from zero) |
| `crates/aegis_content/` | Engine content layer, game-agnostic (SPLIT1-3): data file format, schema normalize and storage, content store and world graph, upsert / remove over the game's hooks |
| `crates/aegis_save/` | Engine save framework, game-agnostic (SPLIT1-4): slot files, envelope, atomic write, migration runner, slot headers |
| `../games/emberwake/crates/emberwake_data/` | Typed content catalog (classes, units, skills, items, passives, encounters, terrain) |
| `../games/emberwake/crates/emberwake_rules/` | Pure rules: stats, gear, unit building, damage, statuses |
| `../games/emberwake/crates/emberwake_battle/` | Battle sim: grid, paths, commands, resolver, planner, AI, session, action log, battle metrics and report, balance-sim fight |
| `../games/emberwake/crates/emberwake_content/` | The game's data loader and writer on `aegis_content` (`source` reads `games/emberwake/data/` like Godot's catalogs, `store` writes it like Godot's saves) |
| `../games/emberwake/crates/emberwake_world/` | The game loop around battles: game state, new game, saves (Godot format, every migration step, on `aegis_save`), exploration, quests, dialogue, shops, travel, rest, battle bridge |
| `crates/aegis_host/` | Engine command API protocol, game-agnostic (SPLIT1-6): JSON request in, JSON reply out, history and step counter over the game's command set |
| `../games/emberwake/crates/emberwake_sim/` | The balance simulator (PORT-SIM): sandbox units, the nine modes (duel, tier, levels, group, stats, party, role, sweep, campaign), scenarios, roles, targets and KPI tables; `aegis sim <mode>` prints its lines |
| `../games/emberwake/crates/emberwake_host/` | The game's command set (state, skills, plan, preset, confirm, step, resolve, advance, autoplay) and CLI commands; holds the `aegis` binary (SPLIT1-6) |
| `crates/aegis_cli/` | Engine command line shell (SPLIT1-6; the binary is built from `emberwake_host`): the game's `sim <mode>`, `battle <encounter>`, and `serve` (the command API over stdin / stdout), `link` (the GUI's agent link), `content` (edit `games/emberwake/data/`: check, upsert, remove, apply) |
| `crates/aegis_ui/` | Engine UI, game-agnostic (SPLIT1-5): canvas and painter, widget kit, art loader (`assets/art/<category>/<id>/<variant>.png`, one folder per thing), live tuning files, agent link, screen stack with the game's context, window shell and event loop |
| `../games/emberwake/crates/emberwake_gui/` | `emberwake`: every game screen (menus, region map, dialogue, shop, world map, game menu, character, journal, results, battle) and the dev tools (Scene Runner, Content Forge, World Forge, Gear Dojo, UI Gallery) on `aegis_ui`; agent link on port 47800 |
| `crates/aegis_app/` | Retired 2026-10-19 (transition step 2): the old egui demo, superseded by `emberwake_gui`; in git history |
| `crates/aegis_core/` (old) | Retired 2026-10-19 with `aegis_app` (its only user); in git history. The name is reused by the engine core above (SPLIT1-2) |

## Parity with Godot

The Godot game is the reference: the frozen copy in `legacy/` (main at 7fa10da, see `legacy/LEGACY.md`; branch
`legacy` to resume Godot work). `parity/` holds the tools that run it and check the engine against it (new files only;
no Godot file is edited):

```bash
games/emberwake/parity/export.sh                      # catalog, worlds, game data, schema, unit snapshots, damage cases, RNG, save fixtures (~5 s)
games/emberwake/parity/trace.sh mode=duel level=5    # record Godot sim fights step by step (duel | group | teams | encounters)
games/emberwake/parity/trace.sh mode=duel level=5 pouch=con_heal_potion,con_fire_bomb   # the same with battle items (party AI uses them)
games/emberwake/parity/trace.sh mode=world preset=withdraw   # every world encounter as WorldEncounter plays it (objectives, phases, morale)
games/emberwake/parity/journey.sh seeds=1,2,3,4 steps=700   # record seeded random game-loop journeys (~4 min)
games/emberwake/parity/sim.sh                         # record the Godot balance sim's tables for 20 command lines (~25 min)
games/emberwake/parity/checks.sh                      # record the Godot content checks: broken entries, world lints, reports (~10 s)
cargo test -p emberwake_rules -p emberwake_battle -p emberwake_host -p emberwake_world -p emberwake_sim -p emberwake_content
    # exact parity: the data loader, units, damage, RNG, battle, item and world battle traces (with their action logs,
    # metrics and reports), journeys (result, state and screen views per action), saves, the sim's printed tables,
    # the content checks (errors and warnings, message for message)
cargo run --release -p emberwake_host -- sim duel 1 20   # the balance-sim duel table (sim help: every mode and key)
```

Status and the file-by-file map: [docs/PORT_NOTES.md](docs/PORT_NOTES.md).

## Driving a battle from the CLI (agents)

`aegis serve` reads one JSON request per line and answers one JSON line each (full list in
`../games/emberwake/crates/emberwake_host/src/lib.rs`):

```bash
printf '%s\n' '{"cmd":"new","encounter":"goblin_skirmish"}' \
  '{"cmd":"plan","unit":"Warrior","steps":[{"mode":"engage","skill":"strike","target":"Goblin Brawler"}]}' \
  '{"cmd":"preset","unit":"Ranger","preset":"focus_weakest"}' '{"cmd":"resolve"}' | aegis serve
```

Every mutating request is recorded; `{"cmd":"history"}` returns them (and the checkpoints: the round outcome after
every concluded round), and feeding them to a fresh `serve` replays the battle exactly.

## Playtest recordings and replay (PORT-PLAYTEST)

While recording is on (the setting `record_playtests`, on in debug builds), the GUI keeps one recording per run in
`%APPDATA%/Aegis/<game>/playtests/<id>.json`: every arena battle's command-host session (its requests from `new`
and the checkpoints), saved when the battle ends, restarts or the screen closes, with the content fingerprint of the
data folder (`aegis_host::playtest`, the hub `aegis_ui::playtest`). Party battles of the world map start outside
`new`, so their replay waits for PLAYTEST2.

```bash
aegis replay <recording.json>                       # every battle: each checkpoint match / diverge (path, values) / unverified
aegis replay <file> battle=0 until=12 json          # stop after 12 requests (what a take-over replays), JSON report
aegis replay <file> variant=patch.json write=out/   # {"at", "remove", "requests", "then": "stop"|"recorded"}: DIFF lines, saved run
aegis replay <file> seeds=1..50 fallback='{"cmd":"autoplay","preset":"balanced"}'   # results under other seeds
```

Exit 0 when everything matches (changed data files only warn: "content stale"), 1 on a divergence or a refused
request, 2 when the recording does not load. `tools/fc.sh replay ...` is the same.

## The game (emberwake)

```bash
cargo run --release -p emberwake_gui                       # the main menu; screen=runner (dev tools) or screen=arena
cargo run --release -p emberwake_gui -- screen=arena encounter=goblin_ambush seed=3
```

The main menu leads to New game (world cards), Continue / Load, Settings and Tools (the Scene Runner: Combat Arena on
any catalog or world encounter, Content Forge, World Forge, Gear & Skill Dojo, debug region and world maps with
Won / Fled / Lost buttons, and the UI Gallery of standalone panes). In play: arrows / W A S D walk, E interacts with the marker you face, a click walks there
(or uses a marker), Esc opens the game menu (party, journal, save, load, settings, quit). Talking opens the dialogue
dock (1-9 choose); shops, turn-ins and lore open as popups; a world-map exit opens the world map (click a node, Travel,
Make camp, Enter). Fights open the battle screen with the saved party; Continue settles the outcome and shows the
results. The screen list and what is left: [games/emberwake/docs/SCREENS.md](../games/emberwake/docs/SCREENS.md).

In battle: hover anything (units, cards, plan blocks, menu options) for a detail card. Click a hero to select it; click
a foe, an ally, the hero itself or an empty tile for its options (each shows AP and the expected damage or heal); right
click closes a menu or undoes the last step. Strip: the Party and Hero dropdowns draft a tactic (with its
description), Auto plays the battle out, Confirm resolves the round. The log (Expand) wraps and scrolls with the wheel
or PageUp / PageDown. Keys (while no control shows the focus ring): Enter confirm, Space step / next round, A autoplay, Z undo, L log, R restart (arena),
I enemy tells, F12 screenshot.

Feel values (timings, floating numbers, hit flash, colours) live in `games/emberwake/tuning.json` and reload while the game
runs. The GUI serves an agent link on `127.0.0.1:47800`: `ui_tree` (the screen stack, the music wanted, the top screen's state and every
widget with its id, text, rect, state, hover card and drag or drop key), `click {id}` (a select opens its list and replies with the option ids, `<id>.opt.<n>`), `drag {id, to}` (press, move, release to `[x, y]` or onto a widget id: slider drags and drag and drop), `hover {id}`, `key {key}` (up / down / left /
right / enter / space / esc / a letter), `back`, `screenshot {path, max_w}`, `game` (the full game state), and the
battle command API while a battle is open. An agent sees and does exactly what the player sees and does:

```bash
aegis link '{"cmd":"ui_tree"}' '{"cmd":"click","id":"menu.new_game"}' '{"cmd":"click","id":"new_game.start"}' '{"cmd":"key","key":"right"}'
```
