# Known issues archive

Rows moved from `.claude/skills/godot-debug-checklist/SKILL.md` because the cause was fixed or the API no longer exists (2026-10-17). Text unchanged apart from table escaping.

| Symptom | Cause | Fix | Why archived |
| --- | --- | --- | --- |
| Off-hand-only items (`wpn_dagger`, tome) rejected in MAIN_HAND | `off_hand_category` set | Use a main-hand item (e.g. `wpn_stiletto`) | ITEM1: dagger is main-hand, one hand category per family |
| Authored `power` unit does double scaling | Tier `power_scale` must not apply to authored power | Authored power is absolute (DamageCalculator) | Fixed in the engine long ago |
| Test calls removed API (`stats.mastery`, `allowed_gear_types`, `masteries`, `class_to_entry(..., desc)`) | Mastery replaced by `Proficiency` + `Passives` | Use `stats.proficiency`, `ContentFactory.apply_passives`, `class.make_proficiency()` | Removed API |
| IDE shows UNSAFE warnings referencing removed classes (`MasterySystem`) | Stale language-server state | Ignore until the editor scan; trust `--check-only` | Removed class |
| A Content Forge / content test fails once and passes on re-run | Two agents ran suites at the same time and the content tests forked to fixed `user://` sandbox names | Fixed in CLN-8 (`TestHelpers.sandbox`) | Fixed |
| `tools/fc.sh parse` reports OK but new `.gd` files were never parsed | `git status --short` collapsed an untracked folder into one line | Fixed in BAL2-L (`git status --short -uall`) | Fixed |
