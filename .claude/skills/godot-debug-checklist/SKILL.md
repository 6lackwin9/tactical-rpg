---
name: godot-debug-checklist
description: Short edit checklist for this Godot tactical RPG (GDScript, headless tests): read, cache refresh, parse, tests, typing, data via upsert, markdown lint. Use before and after every code or data edit; on any failure load godot-known-issues.
---

# Godot tactical RPG: edit checklist and known issues

Rule: when something fails, load `godot-known-issues` and look the failure up there first.

## Every edit: process

1. **Before editing**: read the file's current content (it may have changed on disk). Prefer the Edit/Write tools; use patch scripts for multi-site edits. Avoid `sed` line-range edits.
2. **New or renamed `class_name` scripts**: refresh the class cache before anything else:
   `godot --headless --import` (IDE diagnostics such as "Identifier X not declared" are stale until this runs).
3. **Parse check**: `tools/fc.sh parse` after each edit (refreshes the class cache when it sees new `.gd` files).
4. **Tests**: `tools/fc.sh arena` / `tools/fc.sh dojo` (failures first, log path printed); before handing work back, `check_project.ps1 -Fast -Quiet` (about 30 s on desktop, 3-4 min in the cloud). The full `check_project.ps1` (about 3 min) runs only before a milestone merge. Use `run_in_background` for long runs.
5. **Strict typing pass**: no `UNSAFE_*` warnings, no untyped loop vars, no unused params, no unsafe casts (use `DictUtil.array_of/dict_of/to_int/...`).
6. **Data edits**: change catalog JSON only through `ContentCatalog.upsert` (validates, rewrites refs) in a seed script, then `validate_all()` must be `{}` and `save_all()` returns 0.
7. **Markdown check**: when editing or creating `.md` files, run `tools/fc.sh docs` (markdownlint subset) and, on desktop, verify 0 markdownlint warnings in the IDE Problems tab. Table delimiters need spaces around pipes (`| --- | --- |`, MD060), headings and lists need single blank lines around them (MD022, MD032), and no consecutive blank lines (MD012).

## Debug method (cheap first)

- Reproduce with a tiny `extends SceneTree` script in the scratchpad (`godot --headless --path . -s script.gd`), print the actual numbers, then compare with what the test assumes. Do not guess at numbers.
- Test flags are often combined into one boolean: split them with prints before theorising.
- Filter engine noise: `grep -v "^ERROR\|^   at\|WARNING\|Please"` (RID leak / thread warnings at exit are harmless).

## Known issues

On any failure, load the skill `godot-known-issues` (symptom / cause / fix table) before debugging, and add new recurring causes there.
