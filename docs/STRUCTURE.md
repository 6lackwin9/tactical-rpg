# Project structure: current and reserved

Builders put code where this file says; if a path is missing from here, ask the planner before creating it. Update this
file when a module, crate or folder is added. Since 2026-10-19 the game is the Aegis engine (Rust); the Godot-era layout
(`scripts/`, `scenes/`, GDScript tools) is frozen in `legacy/` and its version of this file is
`legacy/docs/STRUCTURE.md`. Which Godot file each engine module ports: `engine/docs/PORT_NOTES.md`.

## Engine (`engine/`; the Cargo workspace is the repo root: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`, SPLIT1-1)

| Path | Purpose | Status |
| --- | --- | --- |
| `engine/crates/aegis_core/src/` | engine core, game-agnostic (SPLIT1-2): `rng.rs` (Godot PCG32), `ghash.rs` (Godot `hash()`), `sort.rs` (Godot sort), `json.rs` (Godot Variant helpers), `value.rs` (lenient JSON field readers) | done |
| `engine/crates/aegis_data/src/` | `lib.rs`: typed catalog rows (`ClassDef`, `UnitDef`, `SkillDef`, `ItemDef`, `PassiveDef`) and `Catalog` from resolved JSON | done |
| `engine/crates/aegis_rules/src/` | pure rules: `stats.rs` (StatRules), `gear.rs` (equipment, inventory, pouch), `unit.rs` (unit building, skills, passives), `damage.rs` (DamageCalculator), `status.rs` (status table and stacking) | done |
| `engine/crates/aegis_battle/src/` | battle: `grid.rs`, `path.rs`, `world.rs` (units, sides, AI profiles), `skill.rs` (skill rules and queries), `command.rs` (commands incl. items, riposte, support, hazard, cover), `resolver.rs` (block timeline), `plan.rs` (planner), `ai.rs` (TacticalAI), `encounter.rs` (objectives, boss phases, morale), `session.rs` (CombatSession, encounters, guests), `sim.rs` (balance-sim fight) | done |
| `engine/crates/aegis_world/src/` | game loop: `source.rs` (data loader), `store.rs` (data writer), `data.rs` (catalogs, worlds, lineage, resolve), `state.rs`, `new_game.rs`, `save.rs` (Godot save format, every migration), `progression.rs`, `explore.rs`, `quest.rs`, `dialogue.rs`, `shop.rs`, `travel.rs`, `purse.rs`, `rolls.rs`, `battle.rs` (battle bridge, world roster) | done |
| `engine/crates/aegis_host/src/lib.rs` | the battle command API (JSON request in, JSON reply out) used by the CLI, the GUI and tests | done |
| `engine/crates/aegis_cli/src/main.rs` | `aegis`: `sim duel`, `battle`, `serve`, `link`, `content` | done |
| `engine/crates/aegis_gui/src/` | `aegis-gui`: every screen (`menus.rs`, `region.rs`, `worldmap.rs`, `hub.rs`, `party.rs`, `battle.rs`) and dev tool (`runner.rs` Scene Runner, `forge.rs` Content and World Forge viewers, `dojo.rs`), drawing (`canvas.rs`, `painter.rs`, `ui.rs`, `art.rs`), `bridge.rs` (agent link, port 47800), `tuning.rs` | done (look and feel after the transition: `engine/docs/SCREENS.md`) |
| `engine/crates/<crate>/tests/` | integration tests; most replay the Godot reference: `aegis_battle/tests/trace_parity.rs`, `aegis_world/tests/{journey,saves,world_traces,data_source,store}.rs`, `aegis_rules/tests/parity.rs`, `aegis_host/tests/replay.rs` | done |
| `engine/data/` | `schema.json` (the field tables the loader and writer use; a copy of Godot's ContentSchema / WorldSchema), `tuning.json` (GUI feel values, live reload) | done |
| `engine/parity/` | `export.sh`, `trace.sh`, `journey.sh` and `godot/*.gd`: record the frozen Godot build's reference from `legacy/` into `parity/out/` (gitignored) | done |
| `engine/docs/` | `ENGINE_SPEC.md` (direction, milestones, transition), `PORT_NOTES.md` (port map, parity status), `SCREENS.md` (screens, gaps), `screens/` (Godot reference shots) | done |
| `engine/assets/fonts/` | the GUI font (Open Sans, OFL) | done |
| `engine/web/` | an early JavaScript combat prototype; parked (ENGINE_SPEC "web/ player") | parked |

## Content and assets

| Path | Purpose | Status |
| --- | --- | --- |
| `data/catalog/` | classes, units, skills (ranked, v2), items (family + modifiers, v2), item families, passives, encounters, terrain | in use |
| `data/worlds/<id>/` | `world.json` manifest and one file per world kind (regions, locations, factions, characters, items, disciplines, quests, lore, encounters, loot, dialogues, shops, routes) | in use |
| `data/maps/`, `data/quests/`, `data/dialogue/`, `data/progression/` | region map templates, quest templates, dialogue service lines, the XP curve | in use |
| `data/assets/manifest.json` | art and sound file rows (Godot-era `res://` paths; not read or written by the engine yet: redefined with the engine's sprite work) | parked |
| `data/seeds/` | JSON seeds of tasks in flight (`aegis content apply`); `README.md` indexes executed seeds; Godot-era `.gd` seeds are in `legacy/data/seeds/` | in use |
| `assets/art/`, `assets/audio/`, `assets/fonts/` | art read by the engine (`assets/art/**/*.png`), audio and fonts (no audio in the engine yet) | in use |

## Tools, CI, process

| Path | Purpose | Status |
| --- | --- | --- |
| `check_project.ps1`, `check_project.bat` | the desktop commit gate: build every crate, run every test | done |
| `run.bat`, `run_dev.bat`, `run_opt.bat` | launch the Scene Runner from main, develop or a design option (pull, build, save backups) | done |
| `tools/fc.sh` | `gate`, `build`, `test`, `clippy`, `docs`, `content`, `sim`, `parity` | done |
| `tools/md_lint.awk`, `tools/overlap.sh`, `tools/wt_remove.ps1`, `tools/patch_notes.py`, `tools/gen_art.py`, `tools/gen_music.py` | markdown lint subset, worktree overlap, worktree removal, patch notes, placeholder art and music generators | done |
| `tools/sprites/` | SPRITE1-A pixel-art rig (`rig.py`), class animations (`classes.py`), token and portrait export (`export_units.py`; `python3 tools/sprites/export_units.py`) | in use (strips and battle playback: SPRITE1-A2 / -L / -U) |
| `.github/workflows/gate.yml` | CI: docs lint, Godot reference from `legacy/`, then `tools/fc.sh gate` on PRs into `main` (required check `gate`) and pushes to `main` | done |
| `.github/workflows/branch-cleanup.yml` | daily deletion of remote task branches already on `develop` (never `main`, `develop`, `engine`, `legacy`) | done |
| `.claude/` (agents, skills, hooks), `.agents/rules/engine_workflow.md`, `CLAUDE.md` | agent setup and rules | done |
| `docs/` | design and process docs (this index: `CLAUDE.md`); `docs/workflow/` desktop vs cloud; `docs/specs/` task specs; `docs/decisions/` settled decisions | in use |
| `legacy/` | the frozen Godot build (main at 7fa10da, `legacy/LEGACY.md`); never edited | frozen |

Rules: game rules live in the library crates (`aegis_rules`, `aegis_battle`, `aegis_world`); `aegis_gui` only draws and
sends command-API requests; data lives in `data/**` and changes only through `aegis content`; tests live in each crate's
`tests/` folder. Code patterns: skill `rust-conventions`.

## Code conventions

- Names: files and modules `snake_case`; types and traits `PascalCase`; functions and variables `snake_case`; constants
  `UPPER_SNAKE_CASE`.
- Ports keep the Godot names of the rules they port (`Session::confirm_plan` for `CombatSession.confirm_plan`), so a
  rule can be found on both sides.
- Commands follow plan then resolve: planning queues commands and never changes state; resolution runs them in the
  resolver's deterministic order. Game logic never depends on frame timing or the window.
