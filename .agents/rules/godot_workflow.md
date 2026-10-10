# Godot 4 Workflow & Verification Rules

## 1. Static Typing & Code Standards
- All GDScript files must use strict static typing (`-> void`, `: int`, etc.).
- Use snake_case for file names and PascalCase for `class_name` definitions.
- Keep combat/game domain logic strictly decoupled in `RefCounted` classes.

## 2. File Creation, Deletion & Rename Protocol
Whenever creating new script files, deleting files, or modifying `class_name` definitions:
1. **Regenerate Class Cache**: Run `godot --headless --editor --quit` to ensure `.godot/global_script_class_cache.cfg` and UIDs are updated.
2. **Recycle Headless LSP Daemon**: If a headless Godot LSP process is running (`Get-Process godot`), terminate it (`Stop-Process -Name godot -Force`). The IDE extension will automatically restart it with the fresh symbol registry so the IDE Problems tab has zero false diagnostics.
3. **Audit Diagnostics**: Run headless integration tests and verify that the active LSP reports 0 errors and 0 warnings.
