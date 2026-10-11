# Godot 4 Workflow & Verification Rules

## 1. Static Typing & Code Standards

- All GDScript files must use strict static typing (`-> void`, `: int`, etc.).
- Use snake_case for file names and PascalCase for `class_name` definitions.
- Keep combat/game domain logic strictly decoupled in `RefCounted` classes.

## 2. File Creation, Deletion & Rename Protocol

Whenever creating new script files, deleting files, or modifying `class_name` definitions:

1. **Regenerate Class Cache**: Run `godot --headless --editor --quit-after 50` to ensure `.godot/global_script_class_cache.cfg` and UIDs are updated.
2. **Recycle Headless LSP Daemon**: If a headless Godot LSP process is running (`Get-Process godot`), terminate it (`Stop-Process -Name godot -Force`). The IDE extension will automatically restart it with the fresh symbol registry so the IDE Problems tab has zero false diagnostics.
3. **Audit Diagnostics**: Run headless integration tests and verify that the active LSP reports 0 errors and 0 warnings.

## 3. Mandatory Post-Code Error Resolution Protocol

After making any code edits or additions to scripts (`*.gd`), scenes (`*.tscn`), or configuration files:

1. **Run Automated Project Check**: Immediately execute `powershell -NoProfile -ExecutionPolicy Bypass -File .\check_project.ps1` (or `.\check_project.bat --no-pause`).
2. **Verify 3 Check Stages**:
   - All GDScript files must pass Godot parser syntax check (`--check-only`) with 0 errors.
   - Global class cache and imports must compile cleanly (`--editor --quit-after 50`).
   - Headless integration test suite (`test_arena.tscn`) must execute and pass 100% of assertions with exit code 0.
3. **Auto-Remediate All Current Problems**: If any script errors, warnings, LSP diagnostics (IDE Problems tab), or test assertion failures occur, the agent MUST diagnose the root cause, fix the code, and rerun the checks until **all current problems are completely resolved (0 errors, 0 warnings, 0 assertion failures)** before proceeding to documentation or committing.

## 4. Mandatory Documentation & Commit Protocol

Only after all current problems and verification checks have passed cleanly:

1. **Synchronize `README.md`**: The agent must always review and update `README.md` after any code change to reflect:
   - Any new, extracted, or relocated files in the codebase architecture directory breakdown.
   - Newly implemented features, mechanics, or UI capabilities.
   - Updated assertion counts and test coverage in the testing section.
   - Updated roadmap status (marking completed milestones and adding future placeholders).
2. **Commit Only Once All Current Problems Are Resolved**:
   - Never commit broken code, failing tests, or unaddressed diagnostics.
   - Stage and commit code changes together with the updated `README.md` only once all current problems are 100% resolved and verified.

## 5. GDScript Warning Prevention & Safe Typing Standards

To prevent IDE warnings from ever appearing, adhere strictly to these patterns:

### A. Preventing `UNSAFE_CAST`

- **Root Cause**: Occurs when downcasting a `Variant` to a concrete type using `as TargetType` (e.g. `dict[k] as Control` or `dict.get(k) as Control`), because untyped collections return `Variant` and `as` is treated by GDScript as an unchecked dynamic cast.
- **Rule**: NEVER use bare `... as TargetType` on expressions returning `Variant` when assigning to statically typed variables.
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

### B. Preventing `UNSAFE_CALL_ARGUMENT` & `UNSAFE_PROPERTY_ACCESS`

- **Root Cause**: Passing an unverified `Variant` into a statically typed function parameter, or accessing properties on a `Variant`.
- **Rule**: Always extract the value into a typed local variable with an `is` guard, or use strongly-typed data structures before calling methods or accessing properties.

### C. Preventing `UNUSED_VARIABLE` & `UNUSED_PARAMETER`

- **Rule**: Any parameter or lambda argument that is intentionally unused MUST be prefixed with an underscore (`_delta: float`, `_card: Control`, `_idx: int`).
- Avoid creating temporary variables that are not subsequently read.

### D. Preventing `NARROWING_CONVERSION` & `INTEGER_DIVISION`

- **Rule**: Explicitly convert floats to integers using `int(...)`, `roundi(...)`, `floori(...)`, or `ceili(...)` instead of implicit truncation.
- For integer coordinates (`Vector2i`) derived from `Vector2`, use `Vector2i(roundi(v.x), roundi(v.y))` or `Vector2i(int(v.x), int(v.y))`.

### E. Preventing `RETURN_VALUE_DISCARDED`

- **Rule**: When invoking methods that return an error code or discarded object (such as `connect()` or `tween_property(...)`), ensure chains are utilized cleanly, or use `@warning_ignore("return_value_discarded")` only if intentionally unhandled.

## 6. Markdown (.md) Formatting Standards (markdownlint Compliance)

Whenever creating or modifying markdown files (`README.md`, rules, docs), strictly observe the following formatting rules:

1. **Headings (MD022)**: Every heading (`#`, `##`, `###`, etc.) must be preceded by exactly 1 blank line and followed by exactly 1 blank line.
2. **Lists (MD032)**: All bulleted or numbered lists must have a blank line before the first list item and a blank line after the last list item.
3. **Fenced Code Blocks (MD031 & MD040)**:
   - Must be preceded by a blank line and followed by a blank line.
   - Inside list items, indent sub-content and maintain surrounding blank lines.
   - Every fenced code block must declare an explicit language identifier (`gdscript`, `bash`, `powershell`, `text`, etc.).
4. **Spacing & EOF (MD009, MD010, MD047)**:
   - No trailing whitespace at the end of lines.
   - No hard tabs (use spaces).
   - Every file must terminate with a single trailing newline.
