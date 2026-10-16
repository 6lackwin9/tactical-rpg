# FORGE1_CLASSES: Class tab of the Content Forge (five modular sections, typed editors, live derived panels)

Status: approved (user 2026-10-16: plans and order SKILL1 -> ITEM1 -> CLASS1). Revised 2026-10-16 with DECISIONS "FORGE1 class answers", "FORGE1 skill answers 2", "FORGE1 item answers" and "Strict hand typing / no class skill loadout"; slimmed and refreshed 2026-10-17 against SKILL1, BAL2-A and CLN-9 (decided content unchanged).
Depends on: SKILL1-L (done: field types `table`, `multi_pick`, `dict_pick`, the `group` key, problems bar, `origin`; rules in `docs/specs/FORGE_SHARED.md`); CLN-9 (validator split); ITEM1-L commit 1 + ITEM1-A (pickers grouped by family; order in `FORGE1_ITEMS.md` 9)
Lane: logic (CLASS1-L schema first, CLASS1-M migration), small asset task (CLASS1-A)
Touches (when approved): `scripts/content/content_schema.gd` (`_class_fields`, `_unit_fields`; or their per-kind file if CLN-9's optional schema step lands); validators per CLN-9 split in `scripts/content/rules/class_unit_rules.gd` (planned name, not final: `_class_errors`, `_loadout_errors`, class warnings); `content_catalog.gd` only for `warnings_for` dispatch; `content_factory.gd` (`class_archetype`, `class_to_entry`, `loadout_of`, `ai_profile`), `scripts/core/specs/class_archetype.gd` (`skill_path`, `skills_at`, `path_steps`), `scripts/core/specs/stat_rules.gd` (rule flag, 3.6), `scripts/progression/unit_state.gd`, new `scripts/content/class_forge_model.gd` (pure view model), `scripts/ui/forge/{forge_form,content_forge,forge_preview}.gd`, `data/catalog/classes.json` and `units.json` (migration), tests in section 9
Companion: `docs/specs/FORGE1_CLASSES_NOTES.md` (findings incl. what SKILL1 and BAL2-A already built, storage rationale, HISTORY check: nothing here re-proposes a deprecated idea, units extension).

## Goal

Make a class authorable without typing JSON and without being able to save an invalid class. Five Forge sections with typed controls whose ranges are the rule constants, live derived panels (stat totals, gear budget, proficiency coverage, unlock ladder, read-only preview of the default active skills), and cross-field rules that name the reason and disable Save. System classes are protected; user classes are fully editable and deletable.

## 1. Findings (checked in code and data)

Full table and the 2026-10-17 recheck: NOTES section 1. In short: `skill_path` and `proficiency` are raw `json` fields (two new widgets); `skill_path` is a list `[{level, skill}]` read via `skills_at` and `loadout_of` (3.5); SKILL1 already handles basics and `requires` (3.3, 3.5); `AiProfile.heal_below` has no data path (4).

## 2. Section and field table (stored keys: `skill_path` changes shape, everything else unchanged; NEW marked)

### 2.0 Origin, locks and class count

`origin` follows `FORGE_SHARED.md` section 12 (shipped rows `system`, Forge-created `user`, delete only for `user`, Promote / Demote, SAFE DELETE; the shared text wins). Built in SKILL1: `ContentSchema.origin_field()` on every kind, schema flag `key_field: true` checked by `ContentCatalog.key_field_errors` in `upsert` / `upsert_many`, `remove` refusing system rows, `promote` / `demote`, `begin_system_edit()` for seeds, `tools/safe_delete.gd`. CLASS1 only marks the class key fields below.

- Class count is not enforced: the old "six classes" check and the Q1 warning are gone. "New class" is available; a new class starts valid (stats 6 each then adjusted, one sword row, empty path, empty loadout).
- Key fields (locked when `origin` = system; editable when `user`): `id`, `base_stats`, `growth`, `proficiency`, `ai_role`. Why: balance sims and the `StatRules` / proficiency tests read exactly these (sims also read `ai_role` as the default AI role). NOT locked (round 4): the six gear slots, `pouch` and `skill_path` are only defaults (units carry the real loadout and need not follow the class path), so they stay editable on system classes, as do `name`, `description`, `art`. Locked controls are shown disabled with the reason "system class: key field". The catalog enforces the same lock in `upsert` (the form is not the only guard): a changed key field on a system row is an error; gear, pouch and path edits apply but still run all fit, budget and path validation.
- `remove` is refused for `system` rows and for any class a unit references (existing rule). Rename of a `user` class rewrites unit `class_id` (existing rename-refs). System ids cannot be renamed.

### 2.1 Identity

| Key | Type | Allowed | Default | Control | Notes |
| --- | --- | --- | --- | --- | --- |
| `id` | id | `[a-z][a-z0-9_]*`, 2-24, unique | "" | line edit, live uniqueness | read-only after first Save; key field |
| `name` | string | 1-32 | "New Class" | line edit + counter | editable on system |
| `description` | text | 0-240 | "" | text area + counter | editable on system |
| `art` | enum | `UnitArt.CLASS_KEYS` | "warrior" | dropdown + thumbnail | editable on system |
| `ai_role` | enum | AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT | "AGGRESSOR" | dropdown + one-line role help | the unit's default role (unit empty = this); key field |

### 2.2 Stat behaviour

| Key | Type | Allowed | Default | Control |
| --- | --- | --- | --- | --- |
| `base_stats` | stats | each `STAT_FLOOR`..`STAT_CEILING` (6-18); total `CLASS_BUDGET` (62) | all 6 | six spinners, "Remaining: N" chip, Save disabled until 0 |
| `growth` | stats | each 0..`GROWTH_STAT_MAX` (3); total `GROWTH_TOTAL` (5) | all 0 | six spinners, "Remaining: N" chip |

### 2.3 Gear proficiency

| Key | Type | Allowed | Default | Control |
| --- | --- | --- | --- | --- |
| `proficiency` | dict gear type -> `{level, scaling}` | key in `Proficiency.gear_types()`, once each; >= 1 type in `Proficiency.WEAPON_TYPES` | new class: `{sword: {1, 1.5}}` | `dict_pick` table: type dropdown (unused types), level 1..`MAX_LEVEL`, scaling `MIN_SCALING`..`MAX_SCALING` step 0.05 (default 1.5); Remove disabled for the last weapon type or a type the loadout uses (reason shown) |

### 2.4 Skill progression

| Key | Type | Allowed | Default | Control |
| --- | --- | --- | --- | --- |
| `skill_path` | dict `{family id: unlock level}` | key = a FAMILY HEAD (tier 1, not `system`, not `basic`), each once by definition; value integer 1..`MAX_LEVEL`; NO limit on entries | `{}` | `table` (section 3.5): family picker + level selector; shipped data is migrated (3.5) |

### 2.5 Starting gear (the class's gear loadout; unchanged)

The class carries NO skill loadout: only the unlockable `skill_path` (2.4). The 5 + 3 active-skill loadout is set at the UNIT level (the unit `loadout` field, section 3.5); the class has no loadout-selection field, default-loadout field or picker.

| Key | Type | Allowed | Default | Control |
| --- | --- | --- | --- | --- |
| `main_hand` | ref items | fits MAIN_HAND, weapon type trained (or empty = unarmed; with `fist` level >= 2 that is trained fists, base 15, BAL2-L Rule 5; the shipped brawler) | "" | slot picker; untrained items greyed with "no training in X" |
| `off_hand` | ref items | fits OFF_HAND, trained | "" | disabled while main hand is 2H; only support items: shield, orb, tome (one hand category per family, FORGE1 items 8.1) |
| `head`, `body` | ref items | fits slot, armor type trained | "" | slot picker |
| `accessory_1`, `accessory_2` | ref items | fits slot; NOT the same item in both (3.6) | "" | slot picker; the item picked in one slot is greyed in the other while the flag is on |
| `pouch` | refs items | `CONSUMABLE` | `[]` | multi_pick, count limited by battle-pouch size |

AI behaviour: only `ai_role` (2.1). The Forge class form hides every other AI control (section 4).

## 3. Live derived panels and rule changes

### 3.1 Stat panel

Base total / 62, growth total / 5, per stat: base, growth, level 5 and 10 value, violation markers; level 1/5/10 HP and speed; note "units add 4 + (level-1) free points" (`StatRules.SPEND_BASE`).

### 3.2 Gear budget panel (soft)

Per slot value, total vs `GearBudget.TARGET_BUDGET` (23), band (18.4-27.6), unarmed `GearBudget.UNARMED_VALUE` (15, follows `DamageCalculator.UNARMED_BASE_POWER`; was 14 before BAL2-L commit 5), special items with notes, excused-over-band message. Shipped totals after BAL2-A: warrior 27.0, ranger 24.0, priest 26.5, brawler 19.0 (bare hands + leather), rogue 19.0, mage 27.0, all in band. Warnings never disable Save (DECISIONS; the items answer keeps band violations soft until after the balance pass).

### 3.3 Basics are not path content

Built by SKILL1 (keep it; NOTES 1): `_class_errors` rejects a path entry whose skill has `basic` = true; the factory grants every basic (`SkillCatalog.basic_ids()`) to every unit from level 1, always active, outside `MAX_CLASS_SLOTS`; shipped paths hold no basic. CLASS1: the picker never lists basics; the rule moves to the dict shape (3.5); `skills_at` returns path skills only and callers that need basics (`unit_state.gd`, the catalog) keep seeing them (`ClassArchetype.owned_skills_at(level)` or the factory grant; builder's choice while acceptance 5 holds). Regression test: every shipped unit at every level owns the same basics before and after CLASS1 (`owns_skill` per basic).

"Path needs a level-1 skill" is no longer an error (SKILL1); CLASS1 adds it as a warning (3.5): such a class has an empty class loadout at level 1.

### 3.4 Proficiency coverage panel

| Row | Severity |
| --- | --- |
| default-loadout item with `weapon_type` not trained | error (exists) |
| trained gear types no starter item uses | info |
| item requirement exceeds class STR/DEX/INT while class level is 1 | info |
| path skill with `weapon_types` that no trained type satisfies (`unarmed` needs `fist`) | warning |
| no armor type trained | info |

### 3.5 Skill path: standard levels, custom levels, ordering

Constant `ClassArchetype.STANDARD_PATH_LEVELS` (named list, from the shipped paths): `[1, 3, 4, 5, 7]` (every level used by a shipped path, rechecked after BAL2-A; 4 is used by intimidate, hamstring, backflip and focus; one constant, easy to prune). Advanced families offer only values >= `StatRules.ADVANCED_MIN_LEVEL` (5).

UI: a row per entry = family picker (heads only, non-basic, used families greyed, shows kind and grade badge, `requires` shown; a family whose `requires` are not on the path is flagged) + level selector dropdown listing the standard levels and a last entry "Custom level..." that reveals a spinner 1..`MAX_LEVEL`. A level not in the standard list is shown with a "custom" tag. No row limit; a counter shows "N families, first 5 active by default".

Schema contract: `skill_path` is `{String family id: int level}`. Stored in `classes.json` as a JSON object. Old list shape `[{level, skill}]` is accepted by `normalize` for one release (converted, basics dropped, then saved in the new shape) so old branches and tests do not break; after CLASS1-M a list is a validation error.

Skill loadout lives ONLY on the unit (the existing unit `loadout` field: owned non-basic families, max 5 class + 3). When a unit sets none, the default is derived by this (level, order) rule; the class form shows it as a read-only preview. Deterministic default (replaces "path order"): the unit's active class families = owned path families sorted by (unlock level ascending, then entry position in the dict ascending), first `MAX_CLASS_SLOTS` (5). Entry position = JSON object order. Order preservation: Godot `Dictionary` keeps insertion order and the catalog saves with `JSON.stringify(doc, "\t", false)` (sort_keys off), so file order round-trips; the Forge edits a row's level in place (key position unchanged), adds new families at the end, and removal keeps the others' order. A model helper `ClassForgeModel.ordered_path(dict) -> Array[Dictionary]` and `ClassArchetype.path_steps()` (sorted `[{level, skill}]`) are the only places that sort; consumers that today iterate `skill_path` as a list switch to `path_steps()`. Rename of a family (`upsert_skill`) rewrites the key in place.

`ClassArchetype`: `var skill_path: Dictionary` (family -> level, insertion order); `skills_at(level)` = families with value <= level in `path_steps()` order. `class_to_entry` writes the dict back in the same order (round-trip test).

Validation changes: remove "ascending level order" and "one step per family" (impossible by shape; duplicates in raw JSON are caught by the JSON parser keeping the last, so also reject duplicate keys at load if cheaply detectable, else rely on the picker); keep key exists / not system / tier 1 / level integer 1..10 and the grade floor (grade >= advanced needs level >= 5). The `requires` rule (FORGE_SHARED 11.3) already exists for the list shape and is ported unchanged: a path that lists a skill must also list EVERY family in its `requires` at an equal or lower level (error). The unit side (`extra_skills` / `loadout`, `_unit_requires_errors`) exists and is owned by the skills spec. Warnings (never errors): more than 5 non-basic families ("only the first 5 are active by default"), a tier unlocking beyond level 10 (capped), no level-1 entry.

Tier ladder per entry stays "T1 Lv L / T2 Lv L+off2 / T3 Lv L+off3" from `unlock_offset`, capped at 10.

### 3.6 Rule flag: no duplicate accessory

A rule flag `rules.allow_duplicate_accessory` (bool, default `false` = forbidden) lives with the other rule constants in `StatRules` (code constant `ALLOW_DUPLICATE_ACCESSORY: bool = false`, next to the dual-wield flag (a code constant, items 8.1); the logic-builder keeps both flags in one place, a `RuleFlags`-style section of `stat_rules.gd`; see `FORGE1_ITEMS.md` open question 2 on where that flag lives). When false: `_loadout_errors` errors "The same accessory cannot fill both accessory slots." for classes and units; pickers grey the duplicate. Empty slots ("") and `none` (`ContentSchema.NO_ITEM`, BAL2-L 3.1) are not items and never count as a duplicate. Revisit when stacking is designed; flipping the flag needs no data change. Test: with the flag on the same entry validates.

## 4. AI: role on the class, behaviour on the unit

Decision (DECISIONS): the class keeps `ai_role` only. The class-level defaults proposal (`target_pref`, `flee_ratio`, `defend_below`, `reckless`, sentinels, inherit toggles) is dropped; those stay unit fields with their current defaults, so no shipped unit changes.

Class form: `ai_role` + role help only. All other AI controls are absent from the class form (hidden, not disabled).

Unit AI section (unit extension; the unit tab shows these in an "AI" group):

| Key | Type | Allowed | Default | Control | Visibility |
| --- | --- | --- | --- | --- | --- |
| `ai_role` (exists) | enum | "", AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT | "" = the class's role | dropdown, empty shows "Class role: X" | always |
| `target_pref` (exists) | enum | ROLE, NEAREST, WEAKEST | ROLE | dropdown | always |
| `flee_ratio` (exists) | float | 0.0-1.0 step 0.05 | 0.25 | slider | hidden when `reckless` |
| `defend_below` (exists) | float | 0.0-1.0 step 0.05 | 0.4 | slider | hidden when `reckless` |
| `reckless` (exists) | bool | | false | checkbox | always |
| `heal_below` NEW | float | 0.1-1.0 step 0.05 | 0.7 (= `AiProfile.heal_below` default) | slider | shown when the effective role is SUPPORT |
| `prefer_level`, `skill_cap`, `status_resist` (exist) | unchanged | | | | advanced sub-group |

Data path: add `heal_below` to `_unit_fields` (group `ai`, default 0.7, min 0.1, max 1.0), normalize default-fills it; `ContentFactory.ai_profile` adds `profile.heal_below = DictUtil.to_float(e["heal_below"])`. `smart_skills` and `area_min_foes` stay engine-internal. Existing units get 0.7 by default fill, so `AiProfile` is identical (check in section 5).

Cross-field AI warnings (soft, shown on the class for `ai_role`, on the unit for the effective role): SUPPORT role without a SUPPORT-kind path skill; SNIPER with a base-range < 2 main hand; `reckless` with SUPPORT.

## 5. Rules and acceptance checks (rule-derived)

1. Every class field has a `group` in {identity, stats, proficiency, skills, loadout}; the class form has no AI group beyond `ai_role` (in identity).
2. Round trip: shipped class data after migration loads and saves byte-identical; `skill_path` dict order survives load -> `class_to_entry` -> save.
3. A class violating any error rule yields non-empty `draft_problems()`; Save is disabled; warnings never disable.
4. Control ranges equal rule constants (no duplicated literals).
5. A path containing a `basic` skill is an error; for every shipped unit at every level the basic skills owned equal the pre-migration set; every shipped class path contains no basic.
6. Default loadout: for each shipped class, the five families in the preview equal `active_class_families` of a built Lv 10 unit of that class with an empty `loadout` (the four L5 party units carry explicit loadouts since BAL2-A A4, so they are not the fixture), and equal the first five of `path_steps()` sorted by (level, position). Reordering two same-level entries swaps their order; changing a level re-sorts.
7. Migration: for each pre-migration shipped path, the new dict has exactly the old non-basic skills with the same levels; class default loadouts at levels 1-10 are identical to before (this holds because the old list was already ascending, so old path order equals (level, position)).
8. System lock: `upsert` of a `system` class with a changed key field is an error; changed `name`/`description`/`art` succeeds; `remove` of a system class fails; `remove` of a user class succeeds unless a unit references it; Promote makes a user row `system` and then delete fails.
9. Accessory flag: same item in both slots errors when `ALLOW_DUPLICATE_ACCESSORY` is false and validates when true; two empty or two `none` accessory slots never error.
10. `heal_below`: units default 0.7; `ContentFactory.ai_profile(unit).heal_below` equals the entry value; shipped units unchanged.
11. A class with 8 path families validates; 5 are active; the warning names 8 and 5.
12. A path with a skill whose `requires` are not ALL on the path at a level <= its own is an error; with all of them at a lower or equal level it validates (two-entry fixture).
13. Round 4 lock scope: `upsert` of a system class changing `main_hand`, `pouch` or `skill_path` succeeds (and is still validated); changing `base_stats`, `growth`, `proficiency` or `ai_role` is an error.

## 6. Validation matrix

Status as of main 2026-10-17 (after SKILL1); "exists" rules keep working after the dict change.

| Rule | Status | Level | Where |
| --- | --- | --- | --- |
| stats 6-18 total 62; growth 5, 0-3 | exists | error | `StatRules.validate_class` |
| art known; proficiency shape and ranges, >= 1 weapon | exists | error | `_class_errors` |
| loadout slot fit, training, 2H + off-hand | exists | error | `_loadout_errors` |
| id format, unique; name 1-32; description <= 240 | NEW | error | schema caps |
| `ai_role` enum; unit `heal_below` 0.1-1.0 | NEW | error | schema |
| `skill_path` is a dict, level int 1-10, skill exists / not system / tier 1 | changed | error | `_class_errors` |
| path contains a basic skill | exists (SKILL1), ported to dict | error | `_class_errors` |
| grade >= advanced level >= 5; every `requires` family on the path at <= level | exists (SKILL1), ported to dict | error | `_class_errors` |
| ascending order, one step per family | removed by CLASS1 | | |
| level-1 step required | error already removed (SKILL1); NEW warning | warning | `warnings_for` |
| duplicate accessory (flag off) | NEW | error | `_loadout_errors` |
| key field edited on a system class; delete of a system class | mechanism exists (`key_field_errors`, `remove`); NEW: class `key_field` marks | error | `upsert` / `remove` |
| more than 5 non-basic families; tier beyond level 10; weapon-gate unreachable; role mismatch | NEW | warning | `warnings_for` |
| gear budget band, special without note | exists | warning | `GearBudget` |
| class count != 6 | removed | | |

`_class_errors` and `_loadout_errors` live in the class/unit rules file after CLN-9. Warnings come back as typed records `{section, text}`; the string-list API stays.

## 7. Storage recommendation

C1: flat class JSON plus `group` on fields; the only shape change is `skill_path` (list -> dict). Rationale and rejected options: NOTES section 2.

## 8. Units extension (design only, later pass)

Moved to NOTES section 4 (inheritance table per unit group; unit gear `none` noted). Not part of CLASS1 except the unit fields named in sections 3.5, 3.6 and 4.

## 9. Tests to update or add

| Suite | Change |
| --- | --- |
| `content_test_suite.gd` (path cases use list shape) | convert to dict shape; add basic-in-path error, requires-not-met error, duplicate accessory flag (incl. `none`), 8-family warning, id/name/description caps; keep list-shape acceptance case for the one-release converter |
| `origin_test_suite.gd` (exists, SKILL1) | class system-lock (key fields only, acceptance 8, 13) and promote / demote cases |
| `class_rules_test_suite.gd` | iterate `path_steps()`; migration equality; no basic on shipped paths; `skills_at(1)` still non-empty for shipped classes |
| `skill_unlock_test_suite.gd` | build paths as dicts; ladder equals `Combatant.tier_unlocked`; (level, position) default-loadout check |
| `skill_family_test_suite.gd` | basics owned by a class-less unit; expected sets via `path_steps()` |
| `combat_extensions_test_suite.gd` | iterate dict; `ai` cases: `heal_below` flows into `AiProfile` |
| `progression_test_suite.gd` | owned basics before == after migration |
| new `class_form_model_test_suite.gd` (SKILL1 shipped `skill_form_model_test_suite.gd`; extending it instead is a builder choice) | grouping, disabled-Save reasons, control ranges, path ordering helper, standard-level list |
| dojo / world / save suites | pass unchanged |

Line hints of the pre-CLN-9 code: NOTES section 1.

## 10. Milestone plan

| Task | Content | Depends on |
| --- | --- | --- |
| CLASS1-L (commit 1, schema) | `group`; `skill_path` dict (schema, `ClassArchetype`, `path_steps`, factory, rules file, one-release list converter); basics rule and factory grant kept (3.3); `heal_below` unit field + `ai_profile`; duplicate-accessory flag; class `key_field` marks; new validation and typed warnings; `ClassForgeModel` + tests | SKILL1-L (done), CLN-9, ITEM1-L commit 1 |
| CLASS1-L (commit 2, UI) | five sections, path table with standard/custom level, panels, locked controls for system rows, new/delete/promote buttons, unit AI group | commit 1 |
| CLASS1-M (data, via `ContentCatalog.upsert` or seed script) | convert the six paths to dicts (they hold no basics since SKILL1; the seed still drops any it finds); shipped classes stay `system` (absent `origin` = system, FORGE_SHARED 12.1) | CLASS1-L commit 1 merged |
| CLASS1-A (optional) | none expected (heal_below default-fills) | CLASS1-L commit 1 |

Hub files shared with ITEM1: `content_schema.gd`, `content_factory.gd`, `scripts/ui/forge/{forge_form,content_forge,forge_preview}.gd`, `suite_registry.gd`; the validators no longer collide after CLN-9 (planner decides overlap). CLASS1-M must land in the same merge window as the converter removal (never leave main with a list-shaped path and no converter).

## 11. Out of scope

Changing stat budget, growth, proficiency semantics or gear budget numbers; stacking accessories (flag only); dual-wield enabling; world layer; skill editor internals (SKILL1); AI behaviour changes; class-level AI defaults; any class-level skill loadout or default-loadout selection; art.

## 12. Decided (user 2026-10-16)

All earlier questions are resolved and applied above: lock scope (2.0), standard levels `[1, 3, 4, 5, 7]` (3.5), level-1 path skill is a warning (3.3), `FORGE_SHARED.md` 12 wins on origin wording (2.0), no class-level skill loadout (2.5, 3.5), dual-wield flag is a code constant and strict hand typing applies to class and unit gear (2.5). Rows: `docs/decisions/content-forge.md` "FORGE1 class answers", "FORGE1 round 4 answers"; `items-gear.md` "Strict hand typing and no class skill loadout".

## 13. Open questions (2026-10-17 refresh)

1. Coverage panel (3.4) for an unarmed class: the brawler trains `fist` with an empty main hand (trained fists), so "trained gear types no starter item uses" would list `fist`. Should an empty main hand count as using `fist` when the class trains it at level >= 2? Recommendation: yes (info row suppressed), since bare hands are the fist weapon by rule.
2. Flag home (shared with `FORGE1_ITEMS.md` question 2): `stat_rules.gd` (3.6) or `equipment.gd` (items 8.1)?
