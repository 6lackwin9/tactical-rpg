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

- Shipped data changes go in an idempotent seed `data/seeds/<task>.gd` (left in the repo for the planner to re-run after a rebase). The scratchpad is only for throwaway experiments, never for shipped data.
- The seed calls `ContentCatalog.reload_from_disk()`, `ContentCatalog.upsert(kind, entry)` per entry in dependency order (items, skills, passives, classes, units, encounters), prints any returned errors, then `validate_all()` (must print `{}`) and `save_all()` (0). System assets are edited inside `begin_system_edit()` / `end_system_edit()`.
- Upsert is idempotent (replaces same id). Class/unit rules (`StatRules`, proficiency, SK2 unlock levels) are enforced: read the error, fix the data, do not loosen the rule.
- World, quest, map and asset data use their own catalogs the same way (`WorldCatalog`, `QuestCatalog`, `MapCatalog`, `AssetCatalog`). Never hand-edit JSON.

## Tests after data changes

- Numbers in tests derive from rules; print real values with a scratch script before changing an expectation (the dojo checks derive the warrior's damage from `DamageCalculator`).
