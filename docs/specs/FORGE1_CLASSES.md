# FORGE1_CLASSES: Class tab of the Content Forge (five modular sections, typed editors, live derived panels)

Status: proposed (design only; nothing here is approved)
Depends on: SK2 (done), FORGE1 skills (`docs/specs/FORGE1_SKILLS.md`, SKILL1-L): the class pass reuses its generic field types (`table`, `multi_pick`, `dict_pick`), the `group` schema key and the live problems bar / disabled Save
Lane: logic (CLASS1-L, schema first), optional small asset task (CLASS1-A)
Touches (when approved): `scripts/content/content_schema.gd` (`_class_fields`, `_unit_fields` AI sentinels), `content_catalog.gd` (`_class_errors`, `warnings_for`), `content_factory.gd` (`class_archetype`, `class_to_entry`, `ai_profile`), `scripts/core/specs/class_archetype.gd`, new `scripts/content/class_forge_model.gd` (pure RefCounted view model), `scripts/ui/forge/{forge_form,content_forge,forge_preview}.gd`, `data/catalog/classes.json` (new default fields only), tests in section 9

HISTORY.md check: nothing here re-proposes a deprecated idea. Related rows: "Gear proficiency as a plain allowed list" (the proficiency stays a dict `{type: {level, scaling}}`, only its editor changes), "Mastery" (no traits on classes: passives stay granted by sources), "Unequal class stat totals" (budget 62 / growth 5 stay hard rules), "Hard-coded classes -> JSON catalog + Forge" (kept). DECISIONS respected: six-class roster (see open question 1), gear budget is soft/warning level with special items allowed, SK2 unlock rules, class loadouts follow the budget, edits only through `ContentCatalog.upsert`.

## Goal

Make a class authorable without typing JSON and without being able to save an invalid class. The Forge shows a class as five sections, each with typed controls whose ranges are the rule constants (`StatRules`, `Proficiency`), live derived panels that show the consequences (stat totals, gear budget, proficiency coverage, unlock ladder, default five-slot loadout), and cross-field rules that name the reason in the preview and disable Save. The stored class JSON stays as it is (the grouping is a view).

## 1. Findings (checked in code and data)

| Fact | Consequence |
| --- | --- |
| `_class_fields` is a flat 17-field list: 5 identity, `base_stats`, `growth` (type `stats`, six spinners already), `proficiency` and `skill_path` (type `json`, raw TextEdit parsed on keystroke), 6 gear refs + `pouch` (`ref`/`refs`, already filtered by `item_type`) | Only two fields need new widgets; stats and gear already have typed controls |
| `_class_errors` already validates: `StatRules.validate_class` (per-stat 6-18, total 62, growth total 5, 0-3 per stat), art key, non-empty proficiency, gear type known, level 1-3, scaling 1.0-2.0, at least one weapon type, path order/level 1-10/skill exists/not system/tier 1/one step per family/advanced level >= 5/a level-1 step, and `_loadout_errors` (slot fit, training, 2H with off-hand) | Most rules exist; the gap is feedback (single string list) and a few cross-field checks (section 6) |
| `warnings_for("classes")` is only `GearBudget.check_class`; the preview shows the budget line, slot values and specials | The five-slot loadout, proficiency coverage and unlock ladder have no derived view or warning |
| Default five-slot loadout = first 5 non-basic path families by level (M2c-2a1); a 6th is silently inactive (known edge in DECISIONS). Path steps may list basics, validated like others | Path with more than 5 non-basic families is valid but needs a warning, not an error |
| Class AI today = one field, `ai_role` (enum AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT = `AiProfile.Role`). Everything else (`target_pref`, `flee_ratio`, `defend_below`, `prefer_level`, `reckless`, `status_resist`, `skill_cap`) is per UNIT with plain defaults (ROLE, 0.25, 0.4, 0, false); `AiProfile.heal_below` (0.7), `smart_skills`, `area_min_foes` exist in code but in no schema | A "class carries AI defaults" design needs sentinel values on units (section 4) |
| Shipped roles: warrior AGGRESSOR, ranger SNIPER, brawler AGGRESSOR, priest SUPPORT, rogue SKIRMISHER, mage SNIPER | Role semantics for the help text come from the `AiProfile.Role` comments |
| `Equipment.hand()` / `item_fits_slot` already encode the orb/tome off-hand rule and 2H vs off-hand | Gear pickers can filter by rule instead of relying on after-the-fact errors |
| Forge preview `_class` already shows levels 1/5/10 stats, budget, proficiency, path, gear, gear budget | Panels extend this function (or move it to the model) rather than replace it |

## 2. Section and field table (stored keys unchanged; new keys marked NEW)

Group = the Forge section. Control names use the shared FORGE1 field types.

### 2.1 Identity

| Key | Type | Allowed | Default | Control | Dependent / enforcement |
| --- | --- | --- | --- | --- | --- |
| `id` | id | `[a-z][a-z0-9_]*`, 2-24 chars, unique across classes | "" | line edit with live uniqueness check | read-only after the first Save (rename goes through the existing rename-refs action, which rewrites unit `class_id`) |
| `name` | string | 1-32 chars | "New Class" | line edit with counter | |
| `description` | text | 0-240 chars | "" | text area with counter | |
| `art` | enum | `UnitArt.CLASS_KEYS` | "warrior" | dropdown with token thumbnail | |
| `ai_role` | enum | AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT | "AGGRESSOR" | dropdown + one-line role help (section 4) | role drives the cross-field warnings in section 4 |

### 2.2 Stat behaviour

| Key | Type | Allowed | Default | Control | Dependent / enforcement |
| --- | --- | --- | --- | --- | --- |
| `base_stats` | stats (six keys) | each integer `StatRules.STAT_FLOOR`..`STAT_CEILING` (6-18); total exactly `CLASS_BUDGET` (62) | all 6 for a new class, then adjust (total 36) | six spinners with min 6 / max 18; stepper caps so no stat can exceed 18 | live "Remaining: N" chip: green at 0, red otherwise; Save disabled while not 62 |
| `growth` | stats (six keys) | each 0..`GROWTH_STAT_MAX` (3); total exactly `GROWTH_TOTAL` (5) | all 0 | six spinners with min 0 / max 3 | "Remaining: N" chip; a stat at 3 shows "max"; Save disabled while not 5 |

### 2.3 Gear proficiency

| Key | Type | Allowed | Default | Control | Dependent / enforcement |
| --- | --- | --- | --- | --- | --- |
| `proficiency` | dict gear type -> `{level, scaling}` | key in `Proficiency.gear_types()` (13 weapon types incl. shield, 3 armor types), each key at most once; at least one entry in `Proficiency.WEAPON_TYPES` | `{}` (new class: one weapon row `sword` level 1 scaling 1.5 pre-added so the class is valid from the start) | `dict_pick` table, one row per gear type: type dropdown (only unused types), level spinner 1..`Proficiency.MAX_LEVEL` (3), scaling spinner `MIN_SCALING`..`MAX_SCALING` (1.0-2.0) step 0.05 default `DEFAULT_SCALING` (1.5); Add row disabled when every type is used; Remove row disabled when it is the last weapon type, or when the type is used by the default loadout (reason shown) | tooltip per level: 1 = trained, 2 = +power/discount, 3 = block cost reduction (rules in the brief); row shows the Unit Power multiplier and the listed items the class can then wield |

### 2.4 Skill progression

| Key | Type | Allowed | Default | Control | Dependent / enforcement |
| --- | --- | --- | --- | --- | --- |
| `skill_path` | list of `{level, skill}` | `level` 1..`StatRules.MAX_LEVEL` (10); `skill` = a FAMILY HEAD (tier 1, not `system`); one step per family; ascending level (auto-sorted by the control); at least one level-1 step | `[]` (new class: one level-1 step `strike`... see open question 4 for whether basics belong on a path) | `table` with columns Level (spinner), Skill (family picker: heads only, search by name/kind, already-used families greyed, shows kind and advanced badge), derived read-only columns (unlock ladder, section 3.4); Add / Remove / move up-down | picker row for an `advanced` family sets the level spinner min to `ADVANCED_MIN_LEVEL` (5) and says so; deleting the only level-1 step is refused with the reason |

### 2.5 Default loadout

| Key | Type | Allowed | Default | Control | Dependent / enforcement |
| --- | --- | --- | --- | --- | --- |
| `main_hand` | ref items | item with `item_type` WEAPON/SHIELD..., fits MAIN_HAND, `weapon_type` trained by this class (or empty = unarmed) | "" | slot picker listing only fitting, trained items (untrained ones are shown greyed with "no training in X" so the user sees why) | picking a 2H weapon (`Equipment.hand` = main_2h) clears and disables Off hand with the reason |
| `off_hand` | ref items | fits OFF_HAND, trained | "" | slot picker, same filtering; orb/tome appear only here | disabled while main hand is 2H |
| `head`, `body` | ref items | fits slot, armor type trained | "" | slot picker | armor with no proficiency entry is greyed |
| `accessory_1`, `accessory_2` | ref items | fits slot | "" | slot picker | the same item may not fill both slots (new rule, open question 5) |
| `pouch` | refs items | `CONSUMABLE` items | `[]` | multi_pick (checklist), count limited by `Inventory` battle-pouch size | pouch ignored by the gear budget (existing) |

AI behaviour lives in section 2.1 (identity, role) plus the new AI defaults group in section 4. If the user wants a sixth visible section, "AI" is the natural split; the table above keeps five as requested.

## 3. Live derived panels (the preview side; each is a pure function in `ClassForgeModel`)

### 3.1 Stat panel

Rows: base total / 62, growth total / 5, per-stat max rules. Per stat: base value, growth value, level 5 and 10 value (`CharacterStats`), and a violation marker when a stat is outside 6-18 or growth is outside 0-3. Below: level 1/5/10 HP and speed (existing preview lines). Unit spend note: "units add 4 + (level-1) free points on top" (static text from `StatRules.SPEND_BASE`).

### 3.2 Gear budget panel (soft)

Per slot value (`GearBudget.loadout_breakdown`), total vs target `GearBudget.TARGET_BUDGET` (23) and band 18.4-27.6 (`band_low/high`), bar with the band marked, unarmed 14 shown when main hand is empty, special items listed with their notes (existing), excused-over-band message when a documented special item explains an over-band total, "special without note" warning. Warnings never disable Save (DECISIONS: soft only).

### 3.3 Proficiency coverage panel

| Row | Meaning | Severity |
| --- | --- | --- |
| each default-loadout item with `weapon_type` not in `proficiency` | cannot be equipped | error (exists, `_loadout_errors`) |
| trained gear types that no starter item uses | information: "trained, no default item" | info |
| loadout item whose class level for its type is 1 while an item requirement exceeds the class STR/DEX/INT | requirement discount missing | info (listed, not blocking) |
| each path skill with `weapon_types` where no listed proficiency type satisfies `min_proficiency` (the `unarmed` token needs `fist`) | the skill can never be cast by this class | warning (new) |
| armor types: none trained | class wears nothing | info |

### 3.4 Skill path panel (SK2 rules)

Per path step: family name, kind, advanced badge, path level, and the tier ladder "T1 Lv L / T2 Lv L+off2 / T3 Lv L+off3" read from each tier row's `unlock_offset` (default `TIER_UNLOCK_STEP` x (t-1)); a tier whose level exceeds `MAX_LEVEL` is shown "capped at Lv 10" (SK2-L clamps it) and is a warning (the tier is unreachable before the cap). Advanced families show "floor Lv 3 after reductions, needs path Lv >= 5". The panel also shows the default five-slot loadout preview: the first `MAX_CLASS_SLOTS` (5) non-basic families by level (path order breaks ties), basics shown separately as "always active", and the remainder as "owned, inactive by default (N)". Warning (new, not an error): "Path has N non-basic families; only the first 5 are active by default" when N > 5. Also warning when a family's tier 2 or 3 row is missing (family has fewer ranks than the class tier cap is not an error but is shown).

## 4. AI behaviour field design

Recommendation: the class carries `ai_role` plus a small set of defaults that units inherit; units override only what differs. Role is the dropdown; the other four are the per-class defaults that today live only on units.

| Key | Type | Allowed | Default | Control | Dependent visibility |
| --- | --- | --- | --- | --- | --- |
| `ai_role` | enum | AGGRESSOR, SKIRMISHER, SNIPER, SUPPORT | AGGRESSOR | dropdown with help text per role (closes distance; hunts weakest, falls back when hurt; holds max range; opens with Focus then sniper) | |
| `target_pref` NEW | enum | ROLE, NEAREST, WEAKEST | ROLE | dropdown | |
| `flee_ratio` NEW | float | 0.0-1.0 step 0.05 | 0.25 | slider + value | hidden when `reckless` (a reckless unit never flees or guards) |
| `defend_below` NEW | float | 0.0-1.0 step 0.05 | 0.4 | slider + value | hidden when `reckless` |
| `reckless` NEW | bool | | false | checkbox | |
| `heal_below` NEW (see Q3) | float | 0.1-1.0 step 0.05 | 0.7 | slider | shown only when `ai_role` is SUPPORT |

Units: `flee_ratio`, `defend_below` and `heal_below` become `-1 = use the class` (sentinel, hidden-to-inherit like `armor` -1 today); `target_pref` already has ROLE; `reckless` becomes a tri-state enum `inherit/yes/no`. Shipped class data gets the current literals (0.25 / 0.4 / ROLE / false / 0.7) so no unit's behaviour changes (acceptance check in section 5). `prefer_level`, `skill_cap`, `status_resist`, `smart_skills`, `area_min_foes` stay unit/engine-level (not class identity).

Cross-field AI warnings (soft): SUPPORT role without any SUPPORT-kind skill in the path; SNIPER role with a main-hand weapon of base range < 2 (derived from the item); SKIRMISHER/AGGRESSOR with only ranged weapons trained; `reckless` with SUPPORT role. These are warnings, never errors.

## 5. Rules and acceptance checks (rule-derived)

1. Section grouping is schema data: every class field has a `group` in {identity, stats, proficiency, skills, loadout, ai}; the Forge builds sections from it. Check: every key of `_class_fields` has a group; the form shows no ungrouped field.
2. Storage unchanged: `ContentSchema.normalize("classes", e)` round-trips shipped data byte-identical (existing round-trip test) after the new AI fields are added with defaults.
3. A class entry that violates any error rule in section 6 yields a non-empty `draft_problems()`; the preview lists each reason and Save/Apply are disabled (the FORGE1 problems bar). Warnings list below problems and do not disable.
4. Numeric controls cannot leave the schema range: spinner min/max/step come from the schema; the model clamps on load. Check: for every numeric field the control's range equals the schema `min`/`max`, and those equal the rule constants (`StatRules.STAT_FLOOR`, `GROWTH_STAT_MAX`, `Proficiency.MIN_SCALING`, ...), no duplicated literals.
5. Removing a proficiency row, or changing a main/off-hand item, is refused or reported when it breaks an invariant; the reason string is the same string `_class_errors` produces (one source: the model calls the validator).
6. Default loadout preview == `Combatant` default: for each shipped class the preview's five families equal `active_class_families` of a built Lv 10 unit of that class (derive from the engine, not literals).
7. Unit inheritance: a unit with `flee_ratio` -1 builds an `AiProfile` equal to the class's; a unit with an explicit value keeps it. Shipped units behave identically before and after (compare `AiProfile` fields for all shipped units).

## 6. Validation matrix

| Rule | Status | Level | Where |
| --- | --- | --- | --- |
| base stats 6-18, total 62; growth total 5, 0-3 per stat | exists | error | `StatRules.validate_class` |
| art key known | exists | error | `_class_errors` |
| proficiency non-empty, gear type known, level 1-3, scaling 1.0-2.0, >= 1 weapon type | exists | error | `_class_errors` |
| skill path: list shape, level 1-10, ascending, skill exists, not system, tier 1, one step per family, advanced >= 5, a level-1 step | exists | error | `_class_errors` |
| loadout slot fit, training, 2H + off-hand | exists | error | `_loadout_errors` |
| gear budget band, special without note | exists | warning | `GearBudget` |
| id format/length, unique; name 1-32; description <= 240 | NEW (shared FORGE1 length caps) | error | schema caps + `_class_errors` |
| `ai_role`, `target_pref` enum; `flee_ratio`/`defend_below`/`heal_below` ranges | NEW | error | schema |
| proficiency duplicate type; scaling not on a 0.05 step | NEW (control prevents; validator rejects raw JSON) | error | `_class_errors` |
| accessory slots hold the same item twice | NEW (Q5) | error | `_loadout_errors` |
| path skill with `weapon_types` unreachable by proficiency | NEW | warning | `warnings_for` |
| path has > 5 non-basic families (only first 5 active) | NEW | warning | `warnings_for` |
| tier unlock beyond level 10 (capped) | NEW | warning | `warnings_for` |
| role vs skills / weapon range mismatch | NEW | warning | `warnings_for` |
| class referenced by a unit cannot be removed; rename rewrites refs | exists | error | `remove` / `rename` |
| number of classes != 6 | NEW (Q1) | warning | `warnings_all` |

Warnings come back as typed records `{section, text}` so the preview can attach each to its section; the string list API of `warnings_for` stays for current callers.

## 7. Storage recommendation

| Option | Gameplay | Cost | Risk |
| --- | --- | --- | --- |
| C1. Keep the flat class JSON; add `group` to schema fields; sections are a Forge view; new AI keys added with defaults | none | schema `group` key (shared with skills), 5 new optional keys, normalize default fill; no data migration beyond defaults | none (round-trip test guards it) |
| C2. Nested JSON: `{identity:{}, stats:{}, proficiency:[], path:[], loadout:{}}` | none | rewrite `class_archetype`, `class_to_entry`, catalog load/save, `PartyBuilder` and 5 suites that read flat keys | high, no engine gain |
| C3. Proficiency stored as a list of rows instead of a dict | none | `Proficiency.from_dict`, `ClassArchetype`, GearBudget, tests | HISTORY: dict was chosen over a list; do not change |

Recommendation: C1. The dict-to-table conversion is done by the editor widget (ordered rows in, same dict out).

## 8. Units extension (design only, a later pass)

A unit entry inherits from its class and overrides:

| Unit group | Inherits from class | Override rule | Control |
| --- | --- | --- | --- |
| Identity | `art` (empty = class art) | own name/description/side | art dropdown with "class default" entry |
| Build | stats, growth, proficiency | `level` 1-10, `tier`, `spend` (exactly 4 + (level-1), or empty = class weights) | spend: six steppers with remaining-points chip, "follow class weights" button; `stat_mode: fixed` hides spend and shows vitals |
| Gear | six slots + pouch | an empty slot means the class's item; a set slot replaces it | pickers show the class item as ghost text; filter by the CLASS proficiency; 2H main disables off-hand |
| Skills | path up to level, default five-family loadout | `extra_skills` (multi_pick), `loadout` (ordered multi_pick of owned non-basic families, max 5 + 3, no basics) | picker lists only families owned at the unit's level; reasons for greyed ones |
| AI | role, target_pref, flee_ratio, defend_below, heal_below, reckless | sentinel = inherit (section 4) | each control has an "inherit class value" toggle showing the effective value |
| Overrides | none | `ap`, `power`, `armor`, `skill_cap`, `status_resist` | existing numeric controls with 0/-1 meaning "none" shown as an Off toggle |

A unit preview shows effective stats at its level, effective gear budget, effective loadout and AI profile. World characters stay `base_unit` + `tuning` + grants (unchanged): they reach classes only through the unit, so nothing in the class pass touches world files; the Forge "Used by" panel for a class lists units, and for a unit lists the characters using it. Passives are never granted by classes or base units (brief), so no class passive field is added.

## 9. Tests to update or add (rule-derived, no hard-coded class names)

| Suite | Change |
| --- | --- |
| `content_test_suite.gd` (lines 86-111: class round trip, budget, growth, proficiency, path cases) | keep; add: new AI fields round trip, range errors, id/name/description caps, duplicate proficiency type, 5+ families warning, unreachable weapon-gate warning, tier-capped warning, role warnings; derive ids from the catalog |
| `class_rules_test_suite.gd` | add per-stat caps and remaining-points helpers in `ClassForgeModel`; proficiency row bounds from `Proficiency` constants |
| `gear_budget_test_suite.gd` | assert the typed warning record keeps the same text; unchanged numbers |
| `skill_unlock_test_suite.gd` | add: panel ladder equals `Combatant.tier_unlocked` for every shipped path; default-loadout preview equals the built unit |
| units: `ai_profile` cases (content or ai suites) | inherit sentinel equals class value; explicit value kept; shipped units unchanged |
| new `forge_model_test_suite.gd` (shared with SKILL1-L; one `results.append` line in `suite_registry.gd`) | class section grouping, disabled-Save reasons, control ranges equal rule constants, dict_pick row add/remove rules |
| dojo/world/progression/save suites | must pass unchanged (storage unchanged) |

## 10. Milestone plan and conflicts

One logic task, schema first, plus at most one tiny asset task.

| Task | Content | Depends on |
| --- | --- | --- |
| CLASS1-L (commit 1, schema) | `group` on class fields, AI fields on class and sentinels on unit (default-filled), new validation and typed warnings, `ClassForgeModel` (pure) + its tests | SKILL1-L schema commit (needs `group`, `table`, `dict_pick`, `multi_pick`, problems bar) |
| CLASS1-L (commit 2, UI) | five sections in `forge_form`, derived panels in `forge_preview`, disabled Save with reasons for classes, unit AI inherit toggles (minimal) | commit 1 |
| CLASS1-A (optional) | write the AI default values into the six class rows with `ContentCatalog.upsert` (values equal to current unit behaviour, so only if the schema default fill does not already satisfy this) and the unit rows if any need explicit values | CLASS1-L commit 1 |

Hub files shared with the skills pass: `content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `forge_form.gd`, `content_forge.gd`, `forge_preview.gd`, `suite_registry.gd`. Recommended order: SKILL1-L first (it defines the generic field types, `group`, `filters` and the problems bar once); CLASS1-L after it merges, reusing them. Only the pure model `class_forge_model.gd` and its tests are conflict-free and could start earlier. Items (FORGE1 FG2) should come after CLASS1-L because class gear pickers will want the item family filter; until then they filter by `item_type`/`weapon_type`/`Equipment.hand` as today.

## 11. Out of scope

Changing the stat budget, growth, proficiency semantics or gear budget numbers; the JSON shape; unit templates; world layer; item families; new classes or class content; skills editor internals (SKILL1); AI behaviour changes (only where the defaults live); art.

## 12. Open questions

1. DECISIONS says exactly six classes. Should the Forge allow "New class" at all (only a warning at count != 6), or only edit the six?
2. AI defaults on the class (section 4): approve the sentinel design (class carries `target_pref`, `flee_ratio`, `defend_below`, `reckless`; units inherit with -1/inherit), or keep AI per unit and only `ai_role` on the class?
3. Expose `heal_below` (exists in `AiProfile`, no data path) as a class and unit field, or leave it engine-internal?
4. Should basics (strike, guard) be allowed on a class path? They are always active, so a path step for one only sets a pointless unlock level; the validator allows it today. Recommend a warning, or forbid.
5. Forbid the same item in both accessory slots?
6. Is "too many path families (more than 5)" a warning (recommended, extra families are legitimate for events or respec) or an error?
