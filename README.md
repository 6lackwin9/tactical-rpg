# Tactical RPG (Aegis engine)

A turn-planned tactical RPG: parties plan every unit's round on a shared block timeline, then the round resolves block by
block (stances, reach, cover, statuses, riposte, morale, boss phases) on a 12x9 grid, inside a world of regions, quests,
dialogue, shops and travel. Since 2026-10-19 the game runs on **Aegis**, its own Rust engine (`engine/`). The Godot 4
build it replaced is frozen in `legacy/` and stays the reference the engine is checked against.

## Layout

| Folder | What it holds |
| :--- | :--- |
| `engine/` | The Aegis engine: Rust workspace (rules, battle, world, data loader and writer, command API, CLI, GUI), parity tools, engine docs. Start at [engine/README.md](engine/README.md). |
| `data/` | The game content: catalog (classes, units, skills, items, item families, passives, encounters, terrain), worlds, region maps, quests, dialogue services, XP curve, asset manifest. Edited only through `aegis content` (below). |
| `assets/` | Art, audio and fonts (the engine reads the PNGs in `assets/art`). |
| `docs/` | Game design: brief, roadmap, decisions, specs, balance reports, preferences, workflow. |
| `legacy/` | The frozen Godot build: the whole project of `main` at 7fa10da with its own data copy ([legacy/LEGACY.md](legacy/LEGACY.md)). Branch `legacy` resumes Godot work if ever needed. |
| `tools/` | `fc.sh` (gate, tests, content, sim, parity), art and music generators, worktree helpers. |

## Playing

```bat
run.bat            :: pulls main, builds the engine (release) and opens the Scene Runner
run.bat dev        :: the same for develop, in the sibling folder <checkout>-dev
run.bat opt a      :: a design-review option build (see the header of run.bat)
```

Or directly: `cd engine` then `cargo run --release -p aegis_gui` (main menu) or `-- screen=runner`. Saves and settings
live in `%APPDATA%/Aegis`; Godot saves load in the engine unchanged. Controls, screens and the agent link (port 47800):
[engine/README.md](engine/README.md).

## Checks

- `check_project.ps1 -Fast -Quiet` (desktop) or `tools/fc.sh gate`: builds every crate and runs every test (rules,
  battle, world, data loader and writer, Godot parity). CI runs the same on pull requests into `main` and pushes to
  `main` (`.github/workflows/gate.yml`, required check `gate`), after recording the Godot reference from `legacy/`.
- `tools/fc.sh docs`: doc size and markdown lint.
- `tools/fc.sh parity`: re-records the full Godot reference (battle traces, world battles, journeys) from `legacy/`
  with Godot 4.7.2; the parity suites read it from `engine/parity/out`.

## Editing content

Content changes go through the engine's writer, which validates them and writes Godot's storage format (a file saved
without changes stays byte for byte the same):

```bash
cd engine
cargo run -q -p aegis_cli -- content check
cargo run -q -p aegis_cli -- content upsert units '{"id": "goblin_scout", "name": "Goblin Scout", "class_id": "ranger", "side": "enemy"}'
cargo run -q -p aegis_cli -- content upsert lore @entry.json world=ashlands
cargo run -q -p aegis_cli -- content remove units goblin_scout
cargo run -q -p aegis_cli -- content apply ../data/seeds/<task>.json   # {"steps": [{"op": "upsert", "kind", "entry", "world"?}, ...]}
```

Kinds: the catalog kinds, `maps`, `quests` and `services`, and every world kind (with `world=<id>`). Entries are given in
their stored shape (ranked skills, items as family + modifiers). A change is kept only when the whole data folder still
loads cleanly and its references resolve; otherwise the file is put back. `tools/fc.sh content ...` is the same.

## History

The Godot feature log and per-system notes of the Godot build are in [legacy/README.md](legacy/README.md); release notes in
[PATCH_NOTES.md](PATCH_NOTES.md); the engine's port, parity status and transition in
[engine/docs/PORT_NOTES.md](engine/docs/PORT_NOTES.md) and [engine/docs/ENGINE_SPEC.md](engine/docs/ENGINE_SPEC.md).
