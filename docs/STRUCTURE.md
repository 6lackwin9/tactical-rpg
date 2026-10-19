# Project structure: current and reserved

Builders put code where this file says; if a path is missing from here, ask the planner before creating it. Update this
file when a module, crate or folder is added. Since 2026-10-19 the game is the Aegis engine (Rust); the Godot-era layout
(`scripts/`, `scenes/`, GDScript tools) is frozen in `legacy/` and its version of this file is
`legacy/docs/STRUCTURE.md`. Which Godot file each engine module ports: `engine/docs/PORT_NOTES.md`.

Layers (SPLIT1, `docs/specs/SPLIT1.md`): the engine (`engine/`) is game-agnostic and never names a game; each game lives
in `games/<id>/` (today one, `emberwake`) with its crates, content, assets and design docs; kits (`kits/`) come later,
when a second game shares rules. A task row's Layer (`docs/TASKS.md`) says which side its change lands on. The Cargo
workspace is the repo root (`Cargo.toml` members `engine/crates/*` and `games/*/crates/*`, `Cargo.lock`,
`rust-toolchain.toml`, `.cargo/config.toml`, SPLIT1-1).

## Engine (`engine/`)

| Path | Purpose | Status |
| --- | --- | --- |
| `engine/crates/aegis_core/src/` | engine core (SPLIT1-2): `rng.rs` (Godot PCG32), `ghash.rs` (Godot `hash()`), `sort.rs` (Godot sort), `json.rs` (Godot Variant helpers), `value.rs` (lenient JSON field readers) | done |
| `engine/crates/aegis_content/src/` | content layer (SPLIT1-3): `file.rs` (data file format: JSON.parse floats, entries, JSON.stringify), `schema.rs` (schema normalize, storage form, field and reference checks), `world.rs` (content store and world graph: lineage, merged, resolve, qualified ids, world loader), `store.rs` (upsert / remove / commit over the game's `Hooks`), `manifest.rs` (`game.json`: read, check, write, find the game folder; SPLIT1-8) | done |
| `engine/crates/aegis_save/src/lib.rs` | save framework (SPLIT1-4): slot files, envelope `{format, version, saved_at, game_state}`, atomic write, migration runner, slot headers; the game supplies its `Format` and steps; per-game user folder `%APPDATA%/Aegis/<id>` and the first-start copy (SPLIT1-8) | done |
| `engine/crates/aegis_ui/src/` | UI (SPLIT1-5): drawing (`canvas.rs`, `painter.rs` with the text-size factor, `ui.rs` widget kit incl. `select` with its choice list and the draggable `slider` (UIKIT1), drag and drop sources and targets (UIKIT2), `art.rs` loader of `<category>/<id>/<variant>.png`, ASSETORG1), `audio.rs` (sound and music on rodio: buses, voice pools, the crossfaded music track, volume previews), `tuning.rs` (live tuning file), `bridge.rs` (agent link, port 47800), `screen.rs` (screen stack, `Cx<G>` over the game's context, ask and notice, the settings store with the engine's keys and choices), `shell.rs` (window, event loop, agent commands, drag and drop (press, pick up, ghost, drop or click), applying the display, text-size and sound settings, playing the topmost screen's music) | done |
| `engine/crates/aegis_host/src/lib.rs` | command API protocol (SPLIT1-6): one JSON request in, one reply out, history and step counter over the game's `Commands` | done |
| `engine/crates/aegis_cli/src/lib.rs` | command line shell (SPLIT1-6): arguments, `serve`, `link`, `content` (incl. `manifest`); the game registers its own commands (`Game`) | done |
| `engine/crates/<crate>/tests/` | engine tests (unit tests sit beside the code: `aegis_core`, `aegis_save`, `aegis_content`) | done |
| `engine/docs/` | `ENGINE_SPEC.md` (direction, milestones, transition), `PORT_NOTES.md` (port map, parity status) | done |
| `engine/assets/fonts/` | the GUI font (Open Sans, OFL) | done |
| `engine/web/` | an early JavaScript combat prototype; parked (ENGINE_SPEC "web/ player") | parked |

## Game: Emberwake (`games/emberwake/`)

| Path | Purpose | Status |
| --- | --- | --- |
| `games/emberwake/game.json` | the game manifest (id, title, subtitle, version, kit, engine, worlds, start worlds, references; written by `aegis content manifest`, SPLIT1-8) | done |
| `games/emberwake/schema.json`, `games/emberwake/tuning.json` | the field tables the loader and writer use (a copy of Godot's ContentSchema / WorldSchema); the battle feel values (live reload) | done |
| `games/emberwake/crates/emberwake_data/src/` | `lib.rs`: typed catalog rows (`ClassDef`, `UnitDef`, `SkillDef`, `ItemDef`, `PassiveDef`) and `Catalog` from resolved JSON | done |
| `games/emberwake/crates/emberwake_rules/src/` | pure rules: `stats.rs` (StatRules), `gear.rs` (equipment, inventory, pouch), `unit.rs` (unit building, skills, passives), `damage.rs` (DamageCalculator), `status.rs` (status table and stacking) | done |
| `games/emberwake/crates/emberwake_battle/src/` | battle: `grid.rs`, `path.rs`, `world.rs` (units, sides, AI profiles), `skill.rs` (skill rules and queries), `command.rs` (commands incl. items, riposte, support, hazard, cover), `resolver.rs` (block timeline), `plan.rs` (planner), `ai.rs` (TacticalAI), `encounter.rs` (objectives, boss phases, morale), `session.rs` (CombatSession, encounters, guests), `sim.rs` (balance-sim fight) | done |
| `games/emberwake/crates/emberwake_content/src/` | the game's content rules (SPLIT1-7): `source.rs` (data loader: skill ranks, items, game-data files; SkillText and ItemText rendering), `store.rs` (data writer: storage rules, hooks, check incl. `game.json`) | done |
| `games/emberwake/crates/emberwake_world/src/` | game loop: `data.rs` (`Data` over the engine catalogs, party and character rules, item descriptions), `state.rs`, `new_game.rs`, `save.rs` (Godot save format and every migration step, on `aegis_save`), `progression.rs`, `explore.rs`, `quest.rs`, `dialogue.rs`, `shop.rs`, `travel.rs`, `purse.rs`, `rolls.rs`, `battle.rs` (battle bridge, world roster); `GAME_ID` | done |
| `games/emberwake/crates/emberwake_host/src/` | the game's command surface (SPLIT1-6): `lib.rs` (the battle command set on `aegis_host`), `cli.rs` (`sim duel`, `battle`, data folder, manifest, content hooks, serve host), `main.rs` (the `aegis` binary: the shell with Emberwake registered) | done |
| `games/emberwake/crates/emberwake_gui/src/` | `emberwake`: every screen (`menus.rs`, `region.rs`, `worldmap.rs`, `hub.rs`, `party.rs`, `battle.rs`) and dev tool (`runner.rs` Scene Runner, `forge.rs` Content and World Forge viewers, `dojo.rs`), `screens.rs` (the game half of the screen context, entering a game), `music.rs` (which track each screen plays: MUSIC1 context chain and scopes), `tuning.rs` (the battle feel values), `main.rs` (start-up), all on `aegis_ui` | done (look and feel after the transition: `games/emberwake/docs/SCREENS.md`) |
| `games/emberwake/crates/<crate>/tests/` | integration tests; most replay the Godot reference: `emberwake_battle/tests/trace_parity.rs`, `emberwake_world/tests/{journey,saves,world_traces,data_source,store}.rs`, `emberwake_rules/tests/parity.rs`, `emberwake_host/tests/replay.rs` | done |
| `games/emberwake/parity/` | `export.sh`, `trace.sh`, `journey.sh` and `godot/*.gd`: record the frozen Godot build's reference from `legacy/` into `parity/out/` (gitignored) | done |
| `games/emberwake/docs/` | the game's design docs (SPLIT1-9): `GAME_BRIEF.md`, `ROADMAP.md`, `BALANCE_REPORT.md`, `SCREENS.md` and `screens/` (Godot reference shots), `specs/` (game task specs), `decisions/` (the game systems' decision rows), `preferences/`, `design/` | in use |

## Content and assets (`games/emberwake/`)

| Path | Purpose | Status |
| --- | --- | --- |
| `games/emberwake/data/catalog/` | classes, units, skills (ranked, v2), items (family + modifiers, v2), item families, passives, encounters, terrain | in use |
| `games/emberwake/data/worlds/<id>/` | `world.json` manifest and one file per world kind (regions, locations, factions, characters, items, disciplines, quests, lore, encounters, loot, dialogues, shops, routes) | in use |
| `games/emberwake/data/maps/`, `games/emberwake/data/quests/`, `games/emberwake/data/dialogue/`, `games/emberwake/data/progression/` | region map templates, quest templates, dialogue service lines, the XP curve | in use |
| `games/emberwake/data/assets/manifest.json` | art and sound file rows (Godot-era `res://` paths; not read or written by the engine yet: redefined with the engine's sprite work) | parked |
| `games/emberwake/data/seeds/` | JSON seeds of tasks in flight (`aegis content apply`); `README.md` indexes executed seeds; Godot-era `.gd` seeds are in `legacy/data/seeds/` | in use |
| `games/emberwake/assets/art/`, `games/emberwake/assets/audio/`, `games/emberwake/assets/fonts/` | art read by the engine (`games/emberwake/assets/art/<category>/<id>/<variant>.png`, one folder per thing), audio (`music/<key>.ogg`, `sfx/<key>.wav`, played by `aegis_ui::audio`) and fonts | in use |

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
| `docs/` | studio process docs (this index: `CLAUDE.md`): `TASKS.md` (one queue for every layer), `AGENT_MAP.md`, `HISTORY.md`, `EFFICIENCY.md`, `WORKFLOW.md` and `workflow/` (desktop vs cloud), `DECISIONS.md` (index) with `decisions/workflow.md` and `decisions/archive.md`, `specs/` (engine and process specs: SPLIT1, RESCOPE1, CLEANUP), `ROADMAP.md` (a pointer to the game's roadmap) | in use |
| `legacy/` | the frozen Godot build (main at 7fa10da, `legacy/LEGACY.md`); never edited | frozen |

Rules: game rules live in the game's library crates (`emberwake_rules`, `emberwake_battle`, `emberwake_world`);
`emberwake_gui` only draws and sends command-API requests; engine crates know no game; data lives in
`games/emberwake/data/**` and changes only through `aegis content`; tests live in each crate's `tests/` folder. Code
patterns: skill `rust-conventions`.

## Code conventions

- Names: files and modules `snake_case`; types and traits `PascalCase`; functions and variables `snake_case`; constants
  `UPPER_SNAKE_CASE`.
- Ports keep the Godot names of the rules they port (`Session::confirm_plan` for `CombatSession.confirm_plan`), so a
  rule can be found on both sides.
- Commands follow plan then resolve: planning queues commands and never changes state; resolution runs them in the
  resolver's deterministic order. Game logic never depends on frame timing or the window.
