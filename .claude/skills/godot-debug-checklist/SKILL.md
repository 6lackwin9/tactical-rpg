---
name: godot-debug-checklist
description: Pre/post-edit checklist and known-issue lookup for this Godot tactical RPG (GDScript, headless tests). Use before and after EVERY code or data edit, and consult the known-issues table before debugging any failure as if it were new.
---

# Godot tactical RPG: edit checklist and known issues

Rule: when something fails, look it up in **Known issues** first. Only debug from scratch if nothing matches, then add the new cause to the table.

## Every edit: process

1. **Before editing**: read the file's current content (it may have changed on disk). Prefer the Edit/Write tools; use patch scripts for multi-site edits. Avoid `sed` line-range edits.
2. **New or renamed `class_name` scripts**: refresh the class cache before anything else:
   `godot --headless --editor --quit-after 50` (IDE diagnostics such as "Identifier X not declared" are stale until this runs).
3. **Parse check**: `godot --headless --path . --check-only --quit` (or `check_project.ps1`, which does cache refresh, per-file parse, then both test scenes).
4. **Tests**: `godot --headless res://scenes/test_arena.tscn > /tmp/arena.out 2>&1` then `grep -n "FAIL\|SCRIPT ERROR" /tmp/arena.out`; Dojo suite: `res://scenes/gear_dojo.tscn`. Use `timeout 280` and `run_in_background` for long runs.
5. **Strict typing pass**: no `UNSAFE_*` warnings, no untyped loop vars, no unused params, no unsafe casts (use `DictUtil.array_of/dict_of/to_int/...`).
6. **Data edits**: change catalog JSON only through `ContentCatalog.upsert` (validates, rewrites refs) in a seed script, then `validate_all()` must be `{}` and `save_all()` returns 0.
7. **README** updated for every change; commit/push only when `check_project.ps1` is clean (trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`).

## Debug method (cheap first)

- Reproduce with a tiny `extends SceneTree` script in the scratchpad (`godot --headless --path . -s script.gd`), print the actual numbers, then compare with what the test assumes. Do not guess at numbers.
- Test flags are often combined into one boolean: split them with prints before theorising.
- Filter engine noise: `grep -v "^ERROR\|^   at\|WARNING\|Please"` (RID leak / thread warnings at exit are harmless).

## Known issues

| Symptom | Cause | Fix |
|---|---|---|
| `Identifier "StatRules/Proficiency/..." not declared`, "Could not find type" | Global class cache stale after adding a `class_name` file | Run the editor scan (step 2) |
| A script run hangs with no output | Compile errors in autoload-reachable scripts | Run `--check-only`, fix errors, retry |
| Whole file truncated after a patch script | `String.find()` returned -1 and `substr(0, -1)`/`substr(-1)` was used | Check every `find()` result before slicing; verify with `git diff --stat` |
| Patch `MISSING:` printed | `\n` inside GDScript `"""` strings is a real newline; tabs/escapes differ | Match exact text; use the Edit tool for single sites |
| `printf`/heredoc wrote a literal newline inside a string literal | Shell expanded `\n` | Fix with Edit; write GDScript via Write/Edit, not shell |
| `sed` line edits corrupt match blocks | Line numbers shifted | Never use sed ranges; use Edit |
| Unit stat/number tests fail after class/data changes | Test hard-codes old values (class budgets, speeds, power) | Print real values, derive expectations from rules (`StatRules`, `Proficiency`) not magic numbers |
| Skill missing for a class unit (`heavy_blow` etc.) | `known_skills` = class path up to level; skill cap = `ceil(level/2)`; `min_proficiency` gating | Raise unit level or add `extra_skills`; check `Combatant.can_use` |
| Catalog refuses a class/unit | `StatRules`: budget 62, growth 5 (max 3/stat), spend exactly `4 + (level-1)` or empty, skill level <= cap of its path step, loadout gear must be trained and fit the slot | Read the validation message; fix data, not the rule |
| Off-hand-only items (`wpn_dagger`, tome) rejected in MAIN_HAND | `off_hand_category` set | Use a main-hand item (e.g. `wpn_stiletto`) |
| Authored `power` unit does double scaling | Tier `power_scale` must not apply to authored power | Authored power is absolute (DamageCalculator) |
| Unit art empty/wrong | Unit `art` empty means "class art" | Use `ContentFactory.art_of(entry)` |
| Planner: friend can't enter a tile a faster friend vacates | `CombatPlan._ally_moves` evaluates in sequence-creation order | Evaluate faster units first (speed desc) |
| Speeds/tiles-per-block differ wildly between units | Speed = 8 + DEX - burden; ranger/scout are much faster than warriors | Do not assume the warrior is faster in tests; compare `speed` |
| Test calls removed API (`stats.mastery`, `allowed_gear_types`, `masteries`, `class_to_entry(..., desc)`) | Mastery replaced by `Proficiency` + `Passives` | Use `stats.proficiency`, `ContentFactory.apply_passives`, `class.make_proficiency()` |
| Passive adds nothing at max level | Proficiency level clamps to 3 | Test passives on gear below level 3 or on stat/resist effects |
| IDE shows UNSAFE warnings referencing removed classes (`MasterySystem`) | Stale language-server state | Ignore until the editor scan; trust `--check-only` |
