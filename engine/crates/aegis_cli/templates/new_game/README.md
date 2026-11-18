# __TITLE__

A game on the Aegis engine, made by `aegis new game __ID__`. It runs on the engine alone (no kit): a main menu, a play
screen that says where the game starts, the engine's interface sounds and a scenario the gate runs.

| Path | Holds |
| --- | --- |
| `game.json` | the manifest: id, title, kit (none), worlds |
| `crates/__ID___game/` | the game: `src/lib.rs` (context and launch), `src/screens.rs` (the screens), `src/main.rs` (the program `__ID__`), `src/bin/__ID___cli.rs` (the engine's command line for it, `__ID___cli`) |
| `data/` | the data folder the engine's command line edits |
| `assets/` | the game's asset layer, over the engine's (`ui/theme.json`, `sprites/`, `audio/`, ...) |
| `scenarios/` | scenario files the gate runs with no window (`__ID___cli scenario run all`) |

Run it: `cargo run -p __ID___game --bin __ID__`. Walk its screens with no window: `cargo build -p __ID___game` then
`target/debug/__ID___cli scenario run all`. The engine's API guide is `engine/docs/API.md`; the engine's sample
game, beside this one in `games/`, shows a field, a sprite and a save on top of this.
