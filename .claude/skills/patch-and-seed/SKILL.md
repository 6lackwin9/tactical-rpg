---
name: patch-and-seed
description: Safe editing recipe for this Godot project. Use when changing code in several places or changing catalog/world data (classes, units, items, skills, passives).
---

# Patch and seed

## Code edits
- Use the Edit tool for exact single/multi-site changes; Write for new files. Do not use sed line ranges or shell heredocs containing GDScript strings (newline/escape corruption).
- If many sites need the same change, a patch script (`extends SceneTree`, run via `tools/fc.sh run script.gd`) is fine, but every `find()` must be checked for -1 before slicing and the script must print `MISSING:` for unmatched text. Verify with `git diff --stat`.
- After edits: `tools/fc.sh parse`, then `tools/fc.sh arena` or `dojo`.

## Data edits (catalog or worlds)
- Write a seed script in the scratchpad that calls `ContentCatalog.reload_from_disk()`, `ContentCatalog.upsert(kind, entry)` per entry in dependency order (items, skills, passives, classes, units, encounters), prints any returned errors, then `validate_all()` (must print `{}`) and `save_all()` (0).
- Upsert is idempotent (replaces same id). Class/unit rules (`StatRules`, proficiency, skill cap) are enforced: read the error, fix the data, do not loosen the rule.
- World data: use `WorldCatalog` APIs the same way; JSON hand edits only for trivial text and re-run the world suite.

## Tests after data changes
- Numbers in tests derive from rules; print real values with a scratch script before changing an expectation. Dojo numbers (Dojo-2/3/4/16/23) encode the dojo warrior's damage (currently 53).
