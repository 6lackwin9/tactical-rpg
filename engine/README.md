# Project Aegis Engine

This directory contains the source code for the **Aegis Engine**, an agent-native game engine written in Rust that runs in parallel with the Godot build and aims for feature parity with it.

**Current direction:** [docs/ENGINE_SPEC.md](docs/ENGINE_SPEC.md) (draft, 2026-10-18): headless deterministic sim, one command API shared by GUI and CLI, feel as data, parity with Godot (exact for rules, statistical for battles). All engine work stays under `engine/`; `.gdignore` keeps Godot from scanning it.

* **Original proposal (Markdown-as-language, superseded by the spec):** [ENGINE_PROPOSAL.md](ENGINE_PROPOSAL.md)
* **Active Branch:** `engine`
* **Toolchain:** Rust `stable-x86_64-pc-windows-gnullvm`, pinned in `rust-toolchain.toml`. The MinGW on PATH is llvm-mingw, and the plain `gnu` toolchain fails to link against it (`unable to find library -lgcc_eh`).
* **Build output:** `.cargo/config.toml` puts it in `C:/Users/tusha/AppData/Local/Temp/aegis-target`, outside OneDrive (the folder is several GB). Binaries: `.../aegis-target/release/aegis.exe`.
* **Saves:** `user://` maps to `engine/`, so autosaves go to `engine/saves/` (ignored by git), never the Godot project root.

## Workspace

| Crate | Role |
| :--- | :--- |
| `crates/aegis_data/` | Typed content catalog (classes, units, skills, items, passives, encounters, terrain) |
| `crates/aegis_rules/` | Pure rules: stats, gear, unit building, damage, statuses, Godot RNG and sort |
| `crates/aegis_battle/` | Battle sim: grid, paths, commands, resolver, planner, AI, session, balance-sim fight |
| `crates/aegis_host/` | The command API (JSON request in, JSON reply out): state, skills, plan, preset, confirm, step, resolve, advance, autoplay, history |
| `crates/aegis_cli/` | `aegis` command line: `sim duel`, `battle <encounter>`, `serve` (the command API over stdin / stdout) |
| `crates/aegis_app/` | Old egui demo, kept until the GUI rework (spec E5) |
| `crates/aegis_core/` | Old shared types (used by `aegis_app` only) |

## Parity with Godot

The Godot game is the reference. `parity/` holds the tools that check the engine against it (new files only; no
Godot file is edited):

```bash
engine/parity/export.sh                      # catalog, unit snapshots, damage cases, RNG draws (~4 s)
engine/parity/trace.sh mode=duel level=5    # record Godot sim fights step by step (duel | group | teams | encounters)
cargo test --workspace                       # exact parity: units, damage, RNG, every saved battle trace
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
