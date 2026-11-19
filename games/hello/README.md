# Hello: the engine's sample game

ENGINE-1 (`HELLO-GAME1`). The smallest whole game on Aegis, built on the engine alone with no kit: it shows that a game
needs nothing of the RPG kit, and it is the place to look first when starting a new game.

| Part | Where | What it shows |
| --- | --- | --- |
| Rules | `crates/hello_game/src/walk.rs` | A walker on a walled field; also the command set (`new`, `move`, `state`) the engine's host answers with |
| Save | `crates/hello_game/src/save.rs` | One slot through `aegis_save` (the versioned envelope) |
| Screens | `crates/hello_game/src/screens.rs` | A main menu and the field, with the engine's widgets and default theme |
| Start | `crates/hello_game/src/lib.rs` | The manifest, the asset layers (the game's `assets/`, then the engine's), the context and the shell's options |
| Command line | `crates/hello_game/src/bin/hello_cli.rs` | `aegis_cli::Game` with no content kinds |
| Sprite | `assets/sprites/walker.png` | 16 x 16, drawn by `tools/walker_png.py` |
| Sounds | the engine's `assets/audio/sfx/ui_*.wav` | Found through the asset layers; the game ships none |
| Scenario | `scenarios/hello_walk.json` | Menu, a new walk, steps and bumps, a save, back to the menu, continue, and frame times with a loose budget; in the gate |

## Running it

Two programs, as Emberwake has: `hello` is the game, `hello_cli` the engine's command line for it (a scenario starts the
`hello` beside it). The game links no command line code.

| Command | Does |
| --- | --- |
| `cargo run -p hello_game` | The game in a window |
| `target/debug/hello headless=1280x720` | The game with no window, driven over the agent link (port 47800) |
| `target/debug/hello_cli scenario run all` | Its scenarios (the gate runs them, `tools/fc.sh gate`, step `scenarios_all`) |
| `target/debug/hello_cli playtest start` | The agent playtest harness on this game (`engine/docs/authoring/agent_playtest.md`) |
| `target/debug/hello_cli serve` | The walk's command host on stdin and stdout |

Keys on the field: the arrows or W A S D step, Esc goes back to the menu.
