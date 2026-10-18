# Project Aegis Engine

This directory contains the source code for the **Aegis Engine**, a high-performance, modular, AI-native game engine written in Rust.

**Core Philosophy:** Structured Markdown (`.md`) is the native programming language of the engine. Existing `.gd` scripts in the parent directory serve as a reference implementation to be converted into canonical `.md` specs.

* **Full Architecture Proposal:** [ENGINE_PROPOSAL.md](file:///c:/Users/tusha/OneDrive/Project/Godot/tactical-rpg/new-game-project/engine/ENGINE_PROPOSAL.md)
* **Active Branch:** `engine`
* **Toolchain:** Rust (`stable-x86_64-pc-windows-gnu`)

## Workspace Structure

* `crates/aegis_core/` — Core math, vectors (`IVec2`), Variants, Signals/event bus, and seeded PRNG.
* `crates/aegis_vfs/` — Virtual file system (`res://`, `user://`), JSON serialization, and sandboxed I/O.
* `crates/aegis_md_parser/` — Native Markdown (`.md`) to Typed AST Parser.
* `crates/aegis_gd_bridge/` — Tool to convert legacy Godot `.gd` scripts and `.tscn` scenes into canonical `.md` game files.
* `crates/aegis_vm/` *(planned)* — AST execution engine and rules runtime.
* `crates/aegis_sim/` *(planned)* — Headless test suite and Monte Carlo balance simulation runner.
* `crates/aegis_app/` *(planned)* — WGPU 2D batch renderer, Taffy UI layout, and window presentation.
