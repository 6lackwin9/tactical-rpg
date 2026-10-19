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
* **Saves and settings:** per game under its id, `%APPDATA%/Aegis/emberwake/saves` and `%APPDATA%/Aegis/emberwake/settings.json` (outside the repo; the first start after SPLIT1-8 copies the old shared `%APPDATA%/Aegis/saves` and `settings.json` there and leaves them; `saves=<dir>` points the GUI elsewhere). The files use Godot's save format: Godot saves load in the engine unchanged.

## Workspace

| Crate | Role |
| :--- | :--- |
| `crates/aegis_core/` | Engine core, game-agnostic (SPLIT1-2): Godot RNG, sort and `hash()`, Variant helpers (`json`), lenient field readers (`value`) |
| `crates/aegis_content/` | Engine content layer, game-agnostic (SPLIT1-3): data file format, schema normalize and storage, content store and world graph, upsert / remove over the game's hooks |
| `crates/aegis_save/` | Engine save framework, game-agnostic (SPLIT1-4): slot files, envelope, atomic write, migration runner, slot headers |
| `../games/emberwake/crates/emberwake_data/` | Typed content catalog (classes, units, skills, items, passives, encounters, terrain) |
| `../games/emberwake/crates/emberwake_rules/` | Pure rules: stats, gear, unit building, damage, statuses |
| `../games/emberwake/crates/emberwake_battle/` | Battle sim: grid, paths, commands, resolver, planner, AI, session, balance-sim fight |
| `../games/emberwake/crates/emberwake_content/` | The game's data loader and writer on `aegis_content` (`source` reads `games/emberwake/data/` like Godot's catalogs, `store` writes it like Godot's saves) |
| `../games/emberwake/crates/emberwake_world/` | The game loop around battles: game state, new game, saves (Godot format, every migration step, on `aegis_save`), exploration, quests, dialogue, shops, travel, rest, battle bridge |
| `crates/aegis_host/` | Engine command API protocol, game-agnostic (SPLIT1-6): JSON request in, JSON reply out, history and step counter over the game's command set |
| `../games/emberwake/crates/emberwake_host/` | The game's command set (state, skills, plan, preset, confirm, step, resolve, advance, autoplay) and CLI commands; holds the `aegis` binary (SPLIT1-6) |
| `crates/aegis_cli/` | Engine command line shell (SPLIT1-6; the binary is built from `emberwake_host`): the game's `sim duel`, `battle <encounter>`, and `serve` (the command API over stdin / stdout), `link` (the GUI's agent link), `content` (edit `games/emberwake/data/`: check, upsert, remove, apply) |
| `crates/aegis_ui/` | Engine UI, game-agnostic (SPLIT1-5): canvas and painter, widget kit, art loader, live tuning files, agent link, screen stack with the game's context, window shell and event loop |
| `../games/emberwake/crates/emberwake_gui/` | `emberwake`: every game screen (menus, region map, dialogue, shop, world map, game menu, character, journal, results, battle) and the dev tools (Scene Runner, Content Forge, World Forge, Gear Dojo) on `aegis_ui`; agent link on port 47800 |
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
cargo test -p emberwake_rules -p emberwake_battle -p emberwake_host -p emberwake_world
    # exact parity: the data loader, units, damage, RNG, battle, item and world battle traces, journeys (result, state and screen
    # views per action), saves
cargo run --release -p emberwake_host -- sim duel 1 20   # the balance-sim duel table
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

Every mutating request is recorded; `{"cmd":"history"}` returns them, and feeding them to a fresh `serve` replays the
battle exactly.

## The game (emberwake)

```bash
cargo run --release -p emberwake_gui                       # the main menu; screen=runner (dev tools) or screen=arena
cargo run --release -p emberwake_gui -- screen=arena encounter=goblin_ambush seed=3
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

Feel values (timings, floating numbers, hit flash, colours) live in `games/emberwake/tuning.json` and reload while the game
runs. The GUI serves an agent link on `127.0.0.1:47800`: `ui_tree` (the screen stack, the top screen's state and every
widget with its id, text, rect, state and hover card), `click {id}`, `hover {id}`, `key {key}` (up / down / left /
right / enter / space / esc / a letter), `back`, `screenshot {path, max_w}`, `game` (the full game state), and the
battle command API while a battle is open. An agent sees and does exactly what the player sees and does:

```bash
aegis link '{"cmd":"ui_tree"}' '{"cmd":"click","id":"menu.new_game"}' '{"cmd":"click","id":"new_game.start"}' '{"cmd":"key","key":"right"}'
```
