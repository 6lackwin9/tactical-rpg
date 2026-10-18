# Project Aegis: AI-Native Modular Game Engine
## Architectural Proposal & Milestone Roadmap

> **Superseded 2026-10-18** by [docs/ENGINE_SPEC.md](docs/ENGINE_SPEC.md). Reason: Markdown as a programming language was dropped. A custom language is the hardest part to build and the least natural one for agents. Logic is now plain Rust, content stays in data files, and rules stay in readable spec tables. This file is kept for history.

**Branch:** `engine`  
**Engine Language:** Rust (`stable-x86_64-pc-windows-gnu`, rustc 1.99.1+)  
**Native Source Language:** **Structured Markdown (`.md`)**  
**Date:** 2026-10-18  

---

## 1. Executive Summary & Core Philosophy

**The core thesis of Project Aegis:**  
> **Markdown (`.md`) is the native programming language of the engine.**

Traditional game engines force developers into specialized script files (`.gd`, `.cs`, `.cpp`) and visual scene files (`.tscn`, `.unity`, `.prefab`). For AI agents, structured Markdown is the most natural, expressive, and inspectable medium for reasoning, authoring, and diffing.

In Project Aegis:
1. **Native Game Authoring is Markdown $\rightarrow$ AST**: For any new game, developers and agents write structured `.md` files describing entities, logic, rules, state machines, math formulas, and data tables. The engine parses `.md` directly into a typed **Abstract Syntax Tree (AST)** and executes it.
2. **Legacy GDScript as an Import Source**: The existing `.gd` and `.tscn` files in this repository exist only because the prototype started in Godot. Rather than making the engine dependent on GDScript parsing long-term, the engine includes a **transpiler/converter that converts `.gd` and `.tscn` into canonical `.md` files**.
3. **Unified Engine Core**: The engine runtime (`aegis_vm`) only knows how to execute the typed AST produced from `.md`.

```mermaid
graph TD
    subgraph Legacy / Reference Import
        GD[Existing .gd & .tscn Files] -->|gd2md Converter| MD[Structured .md Game Specs]
    end

    subgraph Native Agent Workflow
        Author[Human / Agent Design Intent] -->|Direct Authoring| MD
    end

    subgraph Aegis Engine Core
        MD -->|aegis_parser (MD Parser)| AST[Typed AST / Intermediate Representation]
        AST --> VM[aegis_vm Runtime & Rules Engine]
        Core[aegis_core: IVec2, RNG, Signals, Variant] --> VM
        VFS[aegis_vfs: Sandboxed I/O & JSON Catalogs] --> VM
    end

    subgraph Execution & Validation
        VM --> Sim[aegis_sim: Headless Invariant Tests & Balance Sweep]
        VM --> Render[aegis_app: WGPU 2D Batching, SVG & Audio]
    end
```

---

## 2. What Does "Code in Markdown" Look Like?

Aegis Markdown files are clean, human-readable, and agent-friendly. They combine structured metadata, rule tables, and logic blocks:

### Example: `combatant.md` (Entity Specification)
````markdown
---
class: Combatant
extends: RefCounted
tags: [combat, entity]
---

# Properties
* `name`: String = ""
* `team`: Enum[allies, enemies] = allies
* `max_hp`: int = 100
* `current_hp`: int = 100
* `position`: IVec2 = (-1, -1)

# Signals
* `hp_changed(current: int, max: int)`
* `defeated()`

# Methods

### `take_damage(amount: int, is_crit: bool) -> int`
Applies incoming damage reduced by active armor rating.
```logic
var actual_damage = max(1, amount - self.armor)
self.current_hp = max(0, self.current_hp - actual_damage)
emit hp_changed(self.current_hp, self.max_hp)

if self.current_hp == 0:
    emit defeated()

return actual_damage
```

### `heal(amount: int) -> int`
Restores hit points up to `max_hp`.
```logic
var restored = min(self.max_hp - self.current_hp, amount)
self.current_hp += restored
emit hp_changed(self.current_hp, self.max_hp)
return restored
```
````

### Why this is superior for agents:
* **Zero Syntax Boilerplate**: Agents don't struggle with esoteric engine macro decorators or language-specific quirks.
* **Unified Documentation & Code**: The docstring, the specification, and the executable logic live in the exact same file.
* **Table-Driven Rules**: Complex status interactions, item tiers, and loot drops can be authored as standard Markdown tables that parse directly into structured memory.

---

## 3. High-Level Workspace Architecture

The engine is structured as a modular Cargo workspace in `engine/`:

```
engine/
├── Cargo.toml                  # Workspace manifest
├── ENGINE_PROPOSAL.md          # Architectural blueprint (this document)
├── README.md                   # Workspace guide
├── crates/
│   ├── aegis_core/             # Math (IVec2), Seeded RNG, Variant, Signals/Slots, Dynamic Value
│   ├── aegis_vfs/              # Virtual File System (res://, user://) and JSON catalogs
│   ├── aegis_md_parser/        # Native Markdown -> Typed AST Parser
│   ├── aegis_gd_bridge/        # GDScript & TSCN -> Markdown Converter (Legacy migration)
│   ├── aegis_vm/               # AST execution runtime & Godot-compatible Standard Library shims
│   ├── aegis_sim/              # Headless test runner & Monte Carlo balance harness
│   └── aegis_app/              # 2D Presentation shell (WGPU / Pixels, UI layout, Audio)
```

---

## 4. Component Deep Dive

### 4.1. `aegis_md_parser` (The Core Engine Parser)
* Parses `.md` files using an AST-aware Markdown tokenizer (powered by `pulldown-cmark` or custom block parser).
* Extracts:
  * Frontmatter (class name, inheritance, traits).
  * Typed properties and default values.
  * Signal definitions.
  * Logic blocks (tokenized and parsed into AST expressions).
  * Data tables (parsed into structured catalog rows).

### 4.2. `aegis_gd_bridge` (The Migration Tool)
* Converts the current 345 `.gd` files and 9 `.tscn` files into canonical `.md` files.
* Allows the existing tactical RPG codebase to be converted into the Aegis native `.md` format in one automated pass.
* Once converted, the `.gd` files can be retired and the game is driven 100% by `.md`.

### 4.3. `aegis_core` & `aegis_vfs`
* **`IVec2` / `Vector2i`**: Discrete grid coordinates, Manhattan distance, raycasting on 2D grids.
* **`Variant` / `Value`**: Dynamic typed value system (`Nil`, `Bool`, `Int`, `Float`, `String`, `Array`, `Dict`, `Object`, `Callable`).
* **`Signal`**: Event bus supporting pub/sub event emission.
* **Seeded PRNG**: Deterministic random number generator for reproducible battle replays.
* **VFS**: Transparent file mapping between virtual paths and OS filesystem with sandboxed permissions.

### 4.4. `aegis_vm` (Runtime & Standard Library)
* Evaluates the typed AST.
* Provides the standard shims currently relied upon by the tactical RPG:
  * Memory model: `RefCounted` (Rust `Rc<RefCell<T>>` / `Arc<RwLock<T>>`).
  * Scene/UI Base types: `Node`, `Control`, `Container`, `Label`, `Button`, `PanelContainer`.
  * Timers & Tweens: `Timer`, `create_tween()`.

### 4.5. `aegis_sim` (Simulation & CI Test Harness)
* Runs test suites headlessly in milliseconds.
* Monte Carlo campaign balance simulations: executes thousands of full battles across party compositions to verify win-rate and balance invariants.

### 4.6. `aegis_app` (Presentation Shell)
* `wgpu` 2D hardware-accelerated batch rendering.
* `taffy` UI Flexbox/Grid layout engine.
* `resvg` / `tiny-skia` for dynamic SVG rasterization.
* `miniaudio` / `cpal` for music and SFX.

---

## 5. Revised Milestone Roadmap

| Milestone | Deliverables | Verification Criteria |
| :--- | :--- | :--- |
| **M1: Core Primitives & Workspace** | Setup workspace, `aegis_core`, `aegis_vfs`, `IVec2`, `Variant`, `Signal`, PRNG. | Unit tests pass for all foundational types in Rust. |
| **M2: Native Markdown Parser (`aegis_md_parser`)** | Parser capable of turning `.md` entity and rule specs into a typed AST. | Unit tests verifying that Markdown classes, properties, signals, and methods produce clean ASTs. |
| **M3: GDScript $\rightarrow$ Markdown Converter (`aegis_gd_bridge`)** | Tooling to transpile existing `.gd` and `.tscn` files into canonical `.md` files. | Automatically converts reference game scripts into valid `.md` files that parse into AST. |
| **M4: Headless VM & Combat Logic** | AST runtime executing combat simulation, timeline blocks, and commands. | Headless combat tests (`bal2_rules`, `battle_flow`) pass using `.md` source files. |
| **M5: Headless Test Suite (Full Green)** | Headless runner executing all 93 test suites from `.md`. | All test suites pass in < 2 seconds in CLI. |
| **M6: 2D Presentation & UI Shell** | Windowing, WGPU 2D rendering, Taffy UI layout, and SVG rasterization. | Full game is playable in a native desktop window. |
| **M7: Pure Agent Authoring** | Agents create new game features, classes, and quests strictly via `.md` authoring. | New content added without touching Rust or GDScript code. |
