# Project Aegis Engine

This directory contains the source code for the **Aegis Engine**, an agent-native game engine written in Rust that runs in parallel with the Godot build and aims for feature parity with it.

**Current direction:** [docs/ENGINE_SPEC.md](docs/ENGINE_SPEC.md) (draft, 2026-10-18): headless deterministic sim, one command API shared by GUI and CLI, feel as data, parity with Godot (exact for rules, statistical for battles). All engine work stays under `engine/`; `.gdignore` keeps Godot from scanning it.

* **Original proposal (Markdown-as-language, superseded by the spec):** [ENGINE_PROPOSAL.md](ENGINE_PROPOSAL.md)
* **Active Branch:** `engine`
* **Toolchain:** Rust (`stable-x86_64-pc-windows-gnu`)

## Workspace Structure

Current crates as of commit `57a168d`. The spec (Section 7) lists which ones are kept, reworked or retired.

* `crates/aegis_core/` — Core math, vectors (`IVec2`), Variants, Signals/event bus, and seeded PRNG.
* `crates/aegis_vfs/` — Virtual file system (`res://`, `user://`), JSON serialization, and sandboxed I/O.
* `crates/aegis_md_parser/` — Native Markdown (`.md`) to Typed AST Parser.
* `crates/aegis_gd_bridge/` — Tool to convert legacy Godot `.gd` scripts and `.tscn` scenes into canonical `.md` game files.
* `crates/aegis_vm/` *(planned)* — AST execution engine and rules runtime.
* `crates/aegis_sim/` *(planned)* — Headless test suite and Monte Carlo balance simulation runner.
* `crates/aegis_app/` *(planned)* — WGPU 2D batch renderer, Taffy UI layout, and window presentation.
