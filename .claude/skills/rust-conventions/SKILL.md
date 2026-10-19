---
name: rust-conventions
description: Code patterns of the Aegis engine (Rust, engine/crates): crate roles, port comments, Godot Variant semantics over serde_json, ordered maps, determinism, errors, tests against the Godot reference. Use when writing or reviewing engine code (logic-builder, change-reviewer).
---

# Rust conventions (Aegis engine)

These describe how the engine is written today (transition 2026-10-19); follow them so new code reads like the code around it. Typing and gates: `CLAUDE.md`, `.agents/rules/engine_workflow.md`.

## Crates and where code goes

| Crate | Holds | Never |
| --- | --- | --- |
| `aegis_core` | engine core: Godot RNG, sort and `hash()`, Variant helpers (`json`), lenient field readers (`value`) | anything game-specific |
| `aegis_content` | engine content layer: file format, schema normalize and storage, content store and world graph, upsert / remove over the game's `Hooks` | anything game-specific |
| `aegis_save` | engine save framework: slots, envelope, atomic write, migration runner (the game gives its format and steps) | anything game-specific |
| `emberwake_data` | typed catalog rows (`UnitDef`, `SkillDef`, `ItemDef`, ...) built from resolved JSON | game rules |
| `emberwake_rules` | pure rules: stats, gear, unit building, damage, statuses | battle state, IO |
| `emberwake_battle` | grid, paths, commands, resolver, planner, AI, encounter rules, session, sim | world state, drawing |
| `emberwake_content` | the game's data loader (`source`) and writer (`store`) on `aegis_content` | drawing, game state |
| `emberwake_world` | game state, saves, exploration, quests, dialogue, shops, travel, battle bridge | drawing |
| `aegis_host` | engine command API protocol: request routing, history, step counter over the game's `Commands` | anything game-specific |
| `emberwake_host` | the game's battle command set and CLI commands; the `aegis` binary | rules of its own |
| `aegis_cli` | engine command line shell: arguments, serve, link, content over the game's `Game` | anything game-specific |
| `aegis_ui` | engine UI: drawing, widget kit, art, live tuning, agent link, screen stack (`Cx<G>`), window shell | anything game-specific |
| `emberwake_gui` | the game's screens and dev tools on `aegis_ui`; every battle action goes through `emberwake_host` requests | game rules |

A rule lives in exactly one place; the GUI, the CLI and the tests call it (the Godot build's "rules engines own logic, scenes only render" rule).

## Comments

- Every module starts with `//!` naming what it is and the Godot file it ports, for example `//! CombatResolver (scripts/combat/arena/combat_resolver.gd): ...`. New engine-only modules say what they are and why.
- A function that mirrors a Godot function names it in its doc comment (`/// SkillRanks.expand.`, `/// ItemCommand.side_problem: ...`), so a parity divergence can be traced to both sides.
- Comment density is light: one doc line per public item, a short comment where an order or a constant is not obvious (RNG draw order, a Godot quirk kept on purpose).

## Godot semantics kept on purpose

- JSON-shaped data (world entries, game state, saves, data files) stays `serde_json::Value` / `Dict` (`aegis_core::json`), read with the Variant helpers there (`to_int`, `to_float_or`, `gstr` for GDScript `str()`, `s` / `i` / `b` / `f` / `dget` / `aget`), so ports read line by line like the originals.
- Object order is insertion order (`serde_json` `preserve_order`): erase with `shift_remove`, never `remove`; use `IndexMap` / `IndexSet` where Godot iterated a Dictionary.
- Godot quirks are reproduced, not "fixed": `roundi` rounds half away from zero, `JSON.parse` makes every number a float, `JSON.stringify` writes 14 significant digits, `hash()` is murmur3 over Variants, RNG is PCG32 (`aegis_core::rng::GodotRng`) with draws in Godot's order. Changing one of these is a rule change (CLAUDE.md "Godot parity").
- Determinism: no wall clock, no thread timing, no `HashMap` iteration order in anything a battle, save or trace depends on.

## Style

- The workspace is not rustfmt-formatted: match the surrounding layout (long single-line struct literals and calls are normal; about 160 columns, some longer). Do not run `cargo fmt` on existing files.
- No compiler warnings. Clippy is advisory (`tools/fc.sh clippy`): fix lints new code adds; old ones are cleanup tasks.
- Errors: `anyhow::Result` with `.context(...)` at IO boundaries (loading files, CLI, GUI start). Rules code returns plain values or a reason `String` ("" = fine), like the Godot planner (`add_step` returns the problem text). `unwrap()` only where an earlier check guarantees the value.
- Data problems are reported, not fatal: the loader collects `errors` (Godot's `load_errors`) and skips the bad entry.

## Tests

- Integration tests live in `crates/<crate>/tests/*.rs`; most compare the engine with the Godot reference in `games/emberwake/parity/out` (recorded from `legacy/`) and return early with a message when it is missing.
- New behaviour gets a test whose expectation comes from a rule or from the Godot reference, never a number nobody printed.
- A test that changes data works on a scratch copy (`std::env::temp_dir()`), never on the shipped `games/emberwake/data/` (see `tests/store.rs`).
