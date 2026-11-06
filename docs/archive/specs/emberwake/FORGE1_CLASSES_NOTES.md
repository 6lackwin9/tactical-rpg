# FORGE1_CLASSES_NOTES: findings, storage rationale and the later units pass behind FORGE1_CLASSES

Companion of `games/emberwake/docs/specs/FORGE1_CLASSES.md` (the builder contract; read that first). Reference only: no rule here is new. Moved out of the main spec on 2026-10-17 to keep it under the agent-doc size limit; text unchanged except where marked "updated 2026-10-17". Brackets name the main-spec section.

## 1. Findings when the spec was written [main spec 1]

Checked 2026-10-16. Line numbers are pre-CLN-9 (the validators move to `scripts/content/rules/*`); the builder greps.

| Fact | Consequence |
| --- | --- |
| `_class_fields` is a flat list: identity, `base_stats`, `growth` (typed), `proficiency` and `skill_path` (type `json`, raw text), 6 gear refs + `pouch` | Two fields need new widgets |
| `_class_errors` validates stats, art, proficiency, path (order, level 1-10, skill exists, not system, tier 1, one step per family, grade >= advanced needs level >= 5, a level-1 step), loadout fit | Most rules exist; path rules change in section 3.5 |
| `ClassArchetype.skill_path` is `Array[Dictionary]` `[{level, skill}]`; consumers: `skills_at` (class_archetype, unit_state:57, factory 273/319, catalog 726, tests), `content_factory` 199-214 and 303 (`loadout_of`), `content_catalog` 305/333/641, `forge_preview` 153, `class_rules_test_suite` 35/169/183, `content_test_suite` 111-143, `skill_family_test_suite` 288/345, `skill_unlock_test_suite` 61/257/303-328, `combat_extensions_test_suite` 730, seeds `sk2.gd`, `m2c2b_lib.gd` | One accessor change covers most; see 3.5 |
| Shipped paths today list strike, guard (all six) and offhand_jab (ranger, brawler) at level 1 beside real skills; unlock levels used: 1, 3, 4, 5, 7 | Basics leave the path (3.3); standard levels come from this set |
| Default loadout = first 5 non-basic families by path order (`ContentFactory.loadout_of`); a 6th is inactive | New deterministic rule in 3.5 |
| Class AI today = `ai_role`; `target_pref`, `flee_ratio`, `defend_below`, `prefer_level`, `reckless`, `status_resist`, `skill_cap` are unit fields; `AiProfile.heal_below` (0.7) has no data path (`ContentFactory.ai_profile` never sets it) | AI stays on the unit; `heal_below` needs a unit field (section 4) |

Updated 2026-10-17 (checked on main after SKILL1 745962a and BAL2-A fa08ab6):

- SKILL1 already removed the basics from all six shipped paths, grants every basic from level 1 in the factory (`SkillCatalog.basic_ids()`, `content_factory.gd` ~277), rejects a basic in a path, enforces `requires` on the path (class side) and on unit `extra_skills` / `loadout` (`_unit_requires_errors`), and dropped the "level-1 step required" error. `sk2.gd` and `m2c2b_lib.gd` were archived by CLN-2.
- Still list-shaped and still validated as "ascending order" and "one step per family": `skill_path` (CLASS1 changes it).
- Not yet built: `heal_below` data path, duplicate-accessory flag, class key-field marks, `group` on class fields, the level-1 path warning (class warnings today are the gear budget only).
- Unlock levels used after BAL2-A A3: still exactly 1, 3, 4, 5, 7; every shipped path is ascending, so migration acceptance 7 holds.
- BAL2-A A4 gave the four L5 party units explicit `loadout`s; default-loadout checks must use a unit with an empty `loadout`.

## 2. Storage rationale [main spec 7]

C1: keep flat class JSON, add `group` to fields; the only shape change is `skill_path` (list -> dict). Nested class JSON (rewrite of factory, catalog, PartyBuilder, five suites) and a proficiency list (HISTORY: dict chosen) stay rejected. Dict for the path is the user's decision; its cost is the consumers listed in section 1.

## 3. HISTORY.md check [main spec header]

Nothing in FORGE1_CLASSES re-proposes a deprecated idea. Related rows kept: "Gear proficiency as a plain allowed list" (proficiency stays a dict), "Mastery" (no traits on classes), "Unequal class stat totals" (budget 62 / growth 5 stay hard), "Hard-coded classes -> JSON catalog + Forge". DECISIONS respected: soft gear budget, SK2 unlock rules, edits only through `ContentCatalog.upsert`. The "exactly six classes" line is superseded by the user answer (more classes allowed; main spec 2.0). The earlier recommendation to carry AI defaults on the class is DROPPED by the user (main spec 4).

## 4. Units extension (design only, later pass) [main spec 8]

| Unit group | Inherits from class | Override rule |
| --- | --- | --- |
| Identity | `art` (empty = class art) | own name/description/side |
| Build | stats, growth, proficiency | `level`, `tier`, `spend` (exactly 4 + (level-1) or empty = class weights) |
| Gear | six slots + pouch | empty slot = class item; a set slot replaces; `none` (`ContentSchema.NO_ITEM`, BAL2-L 3.1, exists) = deliberately empty, no class fallback (updated 2026-10-17); duplicate-accessory flag and strict hand typing apply |
| Skills | path up to level + all basics | `extra_skills`, `loadout` (owned non-basic families, max 5 + 3; basics never listed; every `requires` family must be owned) |
| AI | `ai_role` only (empty = class role) | `target_pref`, `flee_ratio`, `defend_below`, `reckless`, `heal_below` are plain unit values with the defaults of main spec 4; no inherit sentinels |
| Overrides | none | `ap`, `power`, `armor`, `skill_cap`, `status_resist` |

Unit `origin` follows the shared rules. A unit preview shows effective stats, gear budget, loadout and AI profile. The "Used by" panel lists units for a class. World files untouched.
