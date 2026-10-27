# Kit layout

Status: canon (user 2026-10-25, KIT1). Game-agnostic: this page names no game. A kit is the middle layer between the
engine and a game: the genre's rules and hooks ([layer_rule.md](layer_rule.md), `[kit hook]`), story-free and filled by
each game's data.

## Folders

| Path | Holds |
| --- | --- |
| `kits/<kit>/kit.json` | The kit's manifest: `id`, `title`, `version` and the engine version it needs |
| `kits/<kit>/crates/<kit>kit_*` | The genre rule crates (for example rules, battle, world, content, data, host, sim); the workspace members include `kits/*/crates/*` |
| `kits/<kit>/schema.json` | The field tables of the kinds the kit defines; every game's data is checked against them |
| `kits/<kit>/assets/` | Shared genre assets (base class bodies, generic UI pieces); a game's assets layer over them |
| `kits/<kit>/README.md` | The crates, the schema, the base classes as the kit's contract, and the kit's sample world |

## Rules

- A kit crate depends on engine crates, never on a game crate, and names no game: no game id, title, content id or data
  path. The engine depends on no kit. The gate's layer check (`tools/fc.sh layers`) enforces both.
- A game's crates are façades over the kit crates: they re-export them and add only the game's own code (story
  mechanics no kit hook can express, with a one-line reason) and the game's tests.
- Hooks carry genre names (`meter`, `facility`, `stage track`); story names arrive as data.
- Kit tests use neutral fixture ids. A sample world exercises the hooks on the kit's classic options; when that world
  replays a recorded reference, those suites guard every hook added later.

## Extracting a kit from a game

Do it in small steps, each landing on a green gate:

1. Move byte for byte: the genre modules move into kit crates unchanged, and the game crates keep their names as
   façades that re-export them, so no caller changes. The gate and every replay suite stay green.
2. Move the schema's field tables to the kit's `schema.json`.
3. De-name: real kit crate names, the game id and the data-folder lookup moved to the game, game hooks behind a kit
   trait, neutral fixture ids in kit tests.
4. Take the sample world out of the kit's own tools and sims (the game passes it in).
5. Add the layer check to the gate and the kit README to the docs lint.
6. Prune the façade crates' unused dependencies.
