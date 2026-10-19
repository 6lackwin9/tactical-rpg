# Project Aegis Engine

This directory contains the source code for the **Aegis Engine**, an agent-native game engine written in Rust that runs in parallel with the Godot build and aims for feature parity with it.

**Current direction:** [docs/ENGINE_SPEC.md](docs/ENGINE_SPEC.md) (draft, 2026-10-18): headless deterministic sim, one command API shared by GUI and CLI, feel as data, parity with Godot (exact for rules, statistical for battles). All engine work stays under `engine/`; `.gdignore` keeps Godot from scanning it.

* **Original proposal (Markdown-as-language, superseded by the spec):** [ENGINE_PROPOSAL.md](ENGINE_PROPOSAL.md)
* **Active Branch:** `engine`
* **Toolchain:** Rust `stable-x86_64-pc-windows-gnullvm`, pinned in `rust-toolchain.toml`. The MinGW on PATH is llvm-mingw, and the plain `gnu` toolchain fails to link against it (`unable to find library -lgcc_eh`).
* **Build output:** `.cargo/config.toml` puts it in `C:/Users/tusha/AppData/Local/Temp/aegis-target`, outside OneDrive (the folder is several GB). Binaries: `.../aegis-target/release/aegis.exe`, `aegis-gui.exe`.
* **Game data:** the project's `data/` folder (catalog, worlds, maps, quests, dialogue services, XP curve), read by the
  engine's own loader (`aegis_world::source`) with the field tables in `engine/data/schema.json`; no Godot export is
  needed to play. `data=<dir>` points the GUI or CLI at another data folder, or at a Godot export folder.
* **Saves and settings:** `%APPDATA%/Aegis/saves` and `%APPDATA%/Aegis/settings.json` (outside the repo; `saves=<dir>` points the GUI elsewhere). The files use Godot's save format: Godot saves load in the engine unchanged.

## Workspace

| Crate | Role |
| :--- | :--- |
| `crates/aegis_data/` | Typed content catalog (classes, units, skills, items, passives, encounters, terrain) |
| `crates/aegis_rules/` | Pure rules: stats, gear, unit building, damage, statuses, Godot RNG, sort and `hash()` |
| `crates/aegis_battle/` | Battle sim: grid, paths, commands, resolver, planner, AI, session, balance-sim fight |
| `crates/aegis_world/` | The data loader (`source`: reads `data/` like Godot's catalogs) and the game loop around battles: game state, new game, saves (Godot format, every migration), exploration, quests, dialogue, shops, travel, rest, battle bridge |
| `crates/aegis_host/` | The command API (JSON request in, JSON reply out): state, skills, plan, preset, confirm, step, resolve, advance, autoplay, history |
| `crates/aegis_cli/` | `aegis` command line: `sim duel`, `battle <encounter>`, `serve` (the command API over stdin / stdout), `link` (the GUI's agent link) |
| `crates/aegis_gui/` | `aegis-gui`: every game screen (menus, region map, dialogue, shop, world map, game menu, character, journal, results, battle) and the dev tools (Scene Runner, Content Forge, World Forge, Gear Dojo); agent link on port 47800 |
| `crates/aegis_app/` | Old egui demo (superseded by `aegis_gui`; to retire) |
| `crates/aegis_core/` | Old shared types (used by `aegis_app` only) |

## Parity with Godot

The Godot game is the reference: the frozen copy in `legacy/` (main at 7fa10da, see `legacy/LEGACY.md`; branch
`legacy` to resume Godot work). `parity/` holds the tools that run it and check the engine against it (new files only;
no Godot file is edited):

```bash
engine/parity/export.sh                      # catalog, worlds, game data, schema, unit snapshots, damage cases, RNG, save fixtures (~5 s)
engine/parity/trace.sh mode=duel level=5    # record Godot sim fights step by step (duel | group | teams | encounters)
engine/parity/trace.sh mode=duel level=5 pouch=con_heal_potion,con_fire_bomb   # the same with battle items (party AI uses them)
engine/parity/trace.sh mode=world preset=withdraw   # every world encounter as WorldEncounter plays it (objectives, phases, morale)
engine/parity/journey.sh seeds=1,2,3,4 steps=700   # record seeded random game-loop journeys (~4 min)
cargo test -p aegis_rules -p aegis_battle -p aegis_host -p aegis_world
    # exact parity: the data loader, units, damage, RNG, battle, item and world battle traces, journeys (result, state and screen
    # views per action), saves
cargo run --release -p aegis_cli -- sim duel 1 20   # the balance-sim duel table
```

Status and the file-by-file map: [docs/PORT_NOTES.md](docs/PORT_NOTES.md).

## Driving a battle from the CLI (agents)

`aegis serve` reads one JSON request per line and answers one JSON line each (full list in
`crates/aegis_host/src/lib.rs`):

```bash
printf '%s\n' '{"cmd":"new","encounter":"goblin_skirmish"}' \
  '{"cmd":"plan","unit":"Warrior","steps":[{"mode":"engage","skill":"strike","target":"Goblin Brawler"}]}' \
  '{"cmd":"preset","unit":"Ranger","preset":"focus_weakest"}' '{"cmd":"resolve"}' | aegis serve
```

Every mutating request is recorded; `{"cmd":"history"}` returns them, and feeding them to a fresh `serve` replays the
battle exactly.

## The game (aegis-gui)

```bash
cargo run --release -p aegis_gui                       # the main menu; screen=runner (dev tools) or screen=arena
cargo run --release -p aegis_gui -- screen=arena encounter=goblin_ambush seed=3
```

The main menu leads to New game (world cards), Continue / Load, Settings and Tools (the Scene Runner: Combat Arena on
any catalog or world encounter, Content Forge, World Forge, Gear & Skill Dojo, debug region and world maps with
Won / Fled / Lost buttons). In play: arrows / W A S D walk, E interacts with the marker you face, a click walks there
(or uses a marker), Esc opens the game menu (party, journal, save, load, settings, quit). Talking opens the dialogue
dock (1-9 choose); shops, turn-ins and lore open as popups; a world-map exit opens the world map (click a node, Travel,
Make camp, Enter). Fights open the battle screen with the saved party; Continue settles the outcome and shows the
results. The screen list and what is left: [docs/SCREENS.md](docs/SCREENS.md).

In battle: hover anything (units, cards, plan blocks, menu options) for a detail card. Click a hero to select it; click
a foe, an ally, the hero itself or an empty tile for its options (each shows AP and the expected damage or heal); right
click closes a menu or undoes the last step. Strip: the Party and Hero dropdowns draft a tactic (with its
description), Auto plays the battle out, Confirm resolves the round. The log (Expand) wraps and scrolls with the wheel
or PageUp / PageDown. Keys: Enter confirm, Space step / next round, A autoplay, Z undo, L log, R restart (arena),
F12 screenshot.

Feel values (timings, floating numbers, hit flash, colours) live in `engine/data/tuning.json` and reload while the game
runs. The GUI serves an agent link on `127.0.0.1:47800`: `ui_tree` (the screen stack, the top screen's state and every
widget with its id, text, rect, state and hover card), `click {id}`, `hover {id}`, `key {key}` (up / down / left /
right / enter / space / esc / a letter), `back`, `screenshot {path, max_w}`, `game` (the full game state), and the
battle command API while a battle is open. An agent sees and does exactly what the player sees and does:

```bash
aegis link '{"cmd":"ui_tree"}' '{"cmd":"click","id":"menu.new_game"}' '{"cmd":"click","id":"new_game.start"}' '{"cmd":"key","key":"right"}'
```
