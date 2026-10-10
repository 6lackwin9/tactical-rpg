# Project Context & Coding Standards: Tactical RPG

## 1. Engine & Environment Constraints
- **Target Engine:** Godot 4.x (GDScript 2.0).
- **Execution Mode:** Headless terminal support via `godot --headless`.
- **Operating Environment:** Windows PowerShell / Antigravity IDE.

---

## 2. GDScript 2.0 Strict Typing & Style Rules
- **Static Typing is Mandatory:** Always provide explicit type annotations for variables, constants, parameters, and function return types.
  - Correct: `var current_hp: int = 100`
  - Correct: `func calculate_damage(base_power: int, defense: int) -> int:`
  - Avoid: `var current_hp = 100` or untyped function arguments.
- **Global Classes:** Use `class_name <ClassName>` at the top of domain scripts to register types globally. Avoid unnecessary string-based `preload()` calls unless resolving circular class dependencies.
- **Naming Conventions:**
  - Files and directories: `snake_case` (e.g., `battle_command.gd`, `test_arena.tscn`).
  - Classes and custom Types: `PascalCase` (e.g., `Combatant`, `AttackCommand`).
  - Functions and variables: `snake_case` (e.g., `take_damage()`, `is_alive`).
  - Constants and Enums: `UPPER_SNAKE_CASE` (e.g., `MAX_QUEUE_SIZE`, `TurnState.PLANNING`).
  - Private/Internal helpers: Prefix with a single underscore (e.g., `func _resolve_speed_ties() -> void:`).
- **Modern Syntax Enforcements:**
  - Use `super()` instead of Godot 3's `._init()` or `.method()`.
  - Use `callable.connect(...)` instead of string-based `connect("signal", self, "func")`.
  - Use `Array[T]` for typed arrays (e.g., `var action_queue: Array[BattleCommand] = []`).
  - Use `maxi()`, `mini()`, `clampi()` for integers; avoid float conversions when computing stats.

---

## 3. Gameplay & Combat Architecture
- **Combat Paradigm:** Simultaneous turn resolution via a Two-Phase Command Pattern:
  1. **Planning Phase:** Players and AI queue action intentions into a battle manager (`action_queue.append(command)`). No state modifications occur here.
  2. **Resolution Phase:** Actions are sorted deterministically by initiative (`(priority * 1000) + speed`) and executed via `command.execute()`.
- **Decoupling Rules:**
  - Core mechanics, commands, and data containers MUST inherit from `RefCounted`, never `Node` or `Node2D`.
  - Scene nodes (`Node2D`, `Control`) are reserved strictly for rendering visuals, playing audio, handling user input, and managing tweens/animations.
  - Game logic must never depend directly on visual animation timings or scene trees to calculate battle outcomes.

---

## 4. Testing & Agent Verification Workflow
Whenever adding new mechanics, commands, or data fields:
1. **Never break the test harness:** Update `res://scenes/test_arena.gd` to test any newly created command or system.
2. **Execute Headless Verification:** Before declaring a task complete, run the test scene headlessly in the terminal:
   ```powershell
   godot --headless res://scenes/test_arena.tscn | Out-Host
   ```
3. **Handle Errors Autonomously:** Inspect standard error output. If an out-of-bounds, unhandled signal, or null-reference exception occurs, fix the root cause in the relevant .gd file before presenting the solution.