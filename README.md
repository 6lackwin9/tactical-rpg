# Tactical RPG (Aegis engine)

A turn-planned tactical RPG: parties plan every unit's round on a shared block timeline, then the round resolves block by
block (stances, reach, cover, statuses, riposte, morale, boss phases) on a 12x9 grid, inside a world of regions, quests,
dialogue, shops and travel. Since 2026-10-19 the game runs on **Aegis**, its own Rust engine (`engine/`). The Godot 4
build it replaced is frozen in `legacy/` and stays the reference the engine is checked against.

## Layout

| Folder | What it holds |
| :--- | :--- |
| `engine/` | The Aegis engine: Rust workspace (rules, battle, world, data loader and writer, command API, CLI, GUI), parity tools, engine docs. Start at [engine/README.md](engine/README.md). |
| `games/emberwake/data/` | The game content: catalog (classes, units, skills, items, item families, passives, encounters, terrain), worlds, region maps, quests, dialogue services, XP curve, asset manifest. Edited only through `aegis content` (below). |
| `games/emberwake/assets/` | The game's asset layer: bodies, portraits, icons, tiles, backgrounds, UI files and audio (ASSETORG2; the engine reads them over the game, kit and engine layers; `games/emberwake/assets/README.md`). |
| `docs/` | Game design: brief, roadmap, decisions, specs, balance reports, preferences, workflow. |
| `legacy/` | The frozen Godot build: the whole project of `main` at 7fa10da with its own data copy ([legacy/LEGACY.md](legacy/LEGACY.md)). Branch `legacy` resumes Godot work if ever needed. |
| `kits/` | Shared kits: `kits/rpg/` (the RPG kit: genre rules in `rpgkit_*` crates and their `schema.json` since KIT1-M, plus reusable RPG art and sound, ASSETORG2; [kits/rpg/README.md](kits/rpg/README.md)). |
| `tools/` | `fc.sh` (gate, tests, content, sim, parity), art and music generators, worktree helpers. |

## Playing

```bat
run.bat            :: pulls main, builds the engine (release) and opens the Scene Runner
run.bat dev        :: the same for develop, in the sibling folder <checkout>-dev
run.bat opt a      :: a design-review option build (see the header of run.bat)
```

Or directly from the repo root: `cargo run --release -p emberwake_gui` (main menu) or `-- screen=runner`. Saves and settings
live in `%APPDATA%/Aegis/emberwake` (copied from `%APPDATA%/Aegis` on the first start after SPLIT1-8); Godot saves load in the engine unchanged. Controls, screens and the agent link (port 47800):
[engine/README.md](engine/README.md).

## Checks

- `check_project.ps1 -Fast -Quiet` (desktop) or `tools/fc.sh gate`: builds every crate and runs every test (rules,
  battle, world, data loader and writer, Godot parity). CI runs the same on pull requests into `main` and pushes to
  `main` (`.github/workflows/gate.yml`, required check `gate`), after recording the Godot reference from `legacy/`.
- `tools/fc.sh docs`: doc size and markdown lint.
- `tools/fc.sh parity`: re-records the full Godot reference (battle traces, world battles, journeys) from `legacy/`
  with Godot 4.7.2; the parity suites read it from `games/emberwake/parity/out`.

## Editing content

Content changes go through the engine's writer, which validates them and writes Godot's storage format (a file saved
without changes stays byte for byte the same):

```bash
cargo run -q -p emberwake_host -- content check
cargo run -q -p emberwake_host -- content upsert units '{"id": "goblin_scout", "name": "Goblin Scout", "class_id": "ranger", "side": "enemy"}'
cargo run -q -p emberwake_host -- content upsert lore @entry.json world=ashlands
cargo run -q -p emberwake_host -- content remove units goblin_scout
cargo run -q -p emberwake_host -- content apply games/emberwake/data/seeds/<task>.json   # {"steps": [{"op": "upsert", "kind", "entry", "world"?}, ...]}
```

Kinds: the catalog kinds, `maps`, `quests` and `services`, and every world kind (with `world=<id>`). Entries are given in
their stored shape (ranked skills, items as family + modifiers). A change is kept only when the whole data folder still
checks cleanly: `content check` runs the Forge's content checks (every catalog and world rule, ported from Godot; its
warnings print as `warning:` lines and never fail), and a write that adds a problem is put back. `tools/fc.sh content
...` is the same.

## History

The Godot feature log and per-system notes of the Godot build are in [legacy/README.md](legacy/README.md); release notes in
[PATCH_NOTES.md](PATCH_NOTES.md); the engine's port, parity status and transition in
[engine/docs/PORT_NOTES.md](engine/docs/PORT_NOTES.md) and [engine/docs/ENGINE_SPEC.md](engine/docs/ENGINE_SPEC.md).
