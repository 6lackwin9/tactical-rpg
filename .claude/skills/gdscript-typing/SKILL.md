---
name: gdscript-typing
description: GDScript strict-typing and warning-prevention patterns for this project (UNSAFE_* warnings, Variant narrowing with DictUtil, typed loops and arrays, casts, stale LSP diagnostics). Use when writing or reviewing GDScript (logic-builder, change-reviewer).
---

# GDScript strict typing

To prevent IDE warnings from ever appearing, adhere strictly to these patterns:

### A. Preventing `UNSAFE_CAST`

- **Root Cause**: Occurs when downcasting a `Variant` to a concrete type using `as TargetType` (e.g. `dict[k] as Control` or `dict.get(k) as Control`), because untyped collections return `Variant` and `as` is treated by GDScript as an unchecked dynamic cast.
- **Rule**: NEVER use bare `... as TargetType` on expressions returning `Variant` when assigning to statically typed variables.
- **Enums**: Casting `Variant` to an enum via `as SomeEnum` also triggers `UNSAFE_CAST`. Always use safe fallback extraction:

  ```gdscript
  var raw_slot: Variant = dict.get("slot", Equipment.EquipSlot.MAIN_HAND)
  var slot_val: Equipment.EquipSlot = raw_slot if raw_slot is Equipment.EquipSlot else Equipment.EquipSlot.MAIN_HAND
  ```

- **Solution Pattern (Type-Safe Accessor / Runtime Guard)**: Encapsulate dictionary or collection lookups in dedicated helper methods using `is` type verification:

  ```gdscript
  func get_token_for(combatant: Combatant) -> Control:
      if combatant == null or not unit_token_nodes.has(combatant):
          return null
      var node: Variant = unit_token_nodes[combatant]
      if node is Control:
          return node
      return null
  ```

  If an inline lookup is unavoidable, guard with runtime type verification before assignment:

  ```gdscript
  var token: Control = null
  if dict.has(key):
      var val: Variant = dict[key]
      if val is Control:
          token = val
  ```

### B. Preventing `UNTYPED_DECLARATION` on Loops

- **Root Cause**: `for` loops where the iterator variable does not have an explicit static type (e.g. `for k in ["STR", "DEX"]:` or `for w in dict.keys():`).
- **Rule**: ALWAYS declare the explicit static type on the iterator variable:

  ```gdscript
  for k: String in ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]:
  for w_type: String in masteries.weapon_masteries:
  for def: Dictionary in slot_definitions:
  ```

### C. Preventing `UNSAFE_CALL_ARGUMENT`

- **Root Cause**: Passing values directly from untyped `Dictionary` lookups (`dict["key"]` or `dict.get("key")`) into function parameters expecting concrete types (such as `String`, `Color`, or `int`).
- **Rule**: Never pass bare dictionary lookup expressions into statically typed arguments. Always extract into strongly-typed local variables or use safe conversion wrappers:

  ```gdscript
  var s_key: String = str(s.get("key", ""))
  var s_name: String = str(s.get("name", ""))
  var s_col: Color = s.get("color", Color.WHITE)
  var s_tip: String = str(s.get("tip", ""))
  var card := _create_stat_badge(s_key, s_name, s_col, s_tip)
  ```

### D. Preventing `UNSAFE_METHOD_ACCESS` & `UNSAFE_PROPERTY_ACCESS` on `Variant`

- **Root Cause**: GDScript 4 does not perform flow-sensitive type narrowing on `Variant` variables inside `if val is TargetType:`. Calling a method or property on `val` directly will warn that the method is not present on the inferred type `Variant`.
- **Rule**: ALWAYS declare a typed local variable immediately after the `is` check:

  ```gdscript
  var btn_raw: Variant = filter_btn_group[f]
  if btn_raw is Button:
      var btn: Button = btn_raw
      btn.add_theme_color_override("font_color", Color.CYAN)
  ```

### E. Handling Stale LSP Cache Diagnostics (`The method "..." is not present on inferred type`, desktop IDE only)

- **Root Cause**: When new methods or properties are added to existing `class_name` scripts (`CharacterStats`, `Equipment`, `Inventory`, etc.), the running background Godot Language Server process retains old parsed ASTs in memory. Callers of the newly added methods report false `UNSAFE_METHOD_ACCESS` or `UNSAFE_PROPERTY_ACCESS` warnings.
- **Remediation Protocol**:
  1. Only when no agent is running suites or sims, stop the editor LSP process by its PID (never `Stop-Process -Name godot`: it kills other agents' runs).
  2. Regenerate the project class cache: `godot --headless --import` (desktop: `& "C:\Program Files\Godot\godot.exe" ...`).
  3. The IDE extension will automatically restart the LSP with the fresh symbol registry, clearing all false method/property warnings.

### F. Preventing `UNUSED_VARIABLE` & `UNUSED_PARAMETER`

- **Rule**: Any parameter or lambda argument that is intentionally unused MUST be prefixed with an underscore (`_delta: float`, `_card: Control`, `_idx: int`, `_icon: String`).
- Avoid creating temporary variables that are not subsequently read.

### G. Preventing `NARROWING_CONVERSION` & `INTEGER_DIVISION`

- **Rule**: Explicitly convert floats to integers using `int(...)`, `roundi(...)`, `floori(...)`, or `ceili(...)` instead of implicit truncation.
- For integer coordinates (`Vector2i`) derived from `Vector2`, use `Vector2i(roundi(v.x), roundi(v.y))` or `Vector2i(int(v.x), int(v.y))`.

### H. Preventing `RETURN_VALUE_DISCARDED`

- **Rule**: When invoking methods that return an error code or discarded object (such as `connect()` or `tween_property(...)`), ensure chains are utilized cleanly, or use `@warning_ignore("return_value_discarded")` only if intentionally unhandled.

### I. Preventing `SHADOWED_VARIABLE_BASE_CLASS`

- **Root Cause**: Naming a function parameter, local variable, or property with an identifier that exists on an inherited engine class (such as `is_visible`, `position`, `name`, `size`, `scale` on `CanvasItem`, `Node`, or `Control`).
- **Rule**: Never use names of base class methods or properties for parameters. Prefix function parameters with `p_` (e.g. `p_visible: bool`, `p_name: String`, `p_pos: Vector2`) or use descriptive contextual names.

### J. Preventing `CONFUSABLE_LOCAL_DECLARATION`

- **Root Cause**: Re-declaring a local variable with `var` in a child block (e.g. inside an `if` block) when a variable with the same name is declared later in the outer function scope or in another branch.
- **Rule**: Declare the variable once at the top of the function scope and re-assign without `var`, or use distinct descriptive variable names across branches.

### K. Preventing `UNUSED_SIGNAL`

- **Root Cause**: Declaring a `signal` definition in a class without any call to `.emit(...)` within that class.
- **Rule**: Every signal declared in a script must be explicitly emitted within the class when the corresponding event or state transition occurs (e.g. `gear_modal_requested.emit(hero)`). If a signal is deprecated or unused, remove it.

### L. Safe Float & Integer Extraction from `Variant` (Preventing `UNSAFE_CAST` & `UNSAFE_CALL_ARGUMENT`)

- **Root Cause**:
  1. Invoking the constructor `float(val)` or `int(val)` on expressions that return `Variant` triggers `UNSAFE_CALL_ARGUMENT`.
  2. Writing `val as float` or `val as int` from a `Variant` triggers `UNSAFE_CAST` (`Casting "Variant" to "float"/"int" is unsafe`).
- **Rule**: ALWAYS use direct ternary extraction WITHOUT the `as` keyword:

  ```gdscript
  var raw_val: Variant = dict.get("power_penalty_ratio", 0.0)
  var ratio: float = raw_val if raw_val is float else 0.0

  var raw_int: Variant = dict.get("bonus", 0)
  var bonus: int = raw_int if raw_int is int else 0
  ```

