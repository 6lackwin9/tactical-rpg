# FORGE1_DESIGN, section 1 (SKILLS): archived

Archived 2026-10-16 from `docs/specs/FORGE1_DESIGN.md`. Superseded by `docs/specs/FORGE1_SKILLS.md` and `docs/specs/FORGE1_SKILLS_MATRIX.md` (storage S1 replaced by S3, family as template, derived rank names, description tokens). Kept for history only; do not implement from this file.

## Original section 1: SKILLS

> Superseded by `docs/specs/FORGE1_SKILLS.md` (kept below as history; the storage recommendation S1 in 1.9 is replaced by S3 there).

### 1.1 Skill kinds (user 1.1 / 1.4)

Engine `SkillDef.Kind`: ATTACK, DEFEND, FOCUS, MOVE, REST, TARGET, STATUS, SUPPORT, DISENGAGE. Non-test files referencing each: ATTACK 12, DEFEND 7, FOCUS 7, SUPPORT 6, DISENGAGE 6, MOVE 6, REST 4, TARGET 3, STATUS 2.

| Option | Gameplay effect | Cost | Risk |
| --- | --- | --- | --- |
| A. Keep the engine enum; narrow what AUTHORS see: authorable kinds ATTACK, SUPPORT, DEFEND, FOCUS, DISENGAGE; MOVE/TARGET/STATUS/REST become a read-only "engine" group (already `system`, already unauthorable); merge behaviour in the schema by shared modules (reach, status inflict/grant) rather than in the engine | none | Forge schema only | none |
| B. Merge FOCUS into SUPPORT (`grant_status: focus` as a registry buff) and DEFEND into "self" SUPPORT | fewer kinds, one "effect" skill | status registry entry for focus, `DefendCommand`/`FocusCommand`/planner/AI/log builder (7 non-test files each), 8+3 test files | high; DEFEND also sets stance and mitigation, FOCUS boosts the next attack; not a data-only merge |

Recommendation: A now. Honest result of the usage audit: no kind or field is dead. Everything authorable is used by at least one shipped row (FOCUS has 3 rows, DISENGAGE 6, `offhand` 1, `stance` DEFENSIVE 6). What is redundant is presentation: `level` is really "AP cost" (rename the label/key alias `ap` only in the Forge; storage key stays `level`), `system` is read-only, and `kind` MOVE/TARGET/STATUS/REST cannot be authored. "Deprecated" therefore means: hide from authoring and move to the read-only engine group, not delete. B stays an idea; re-raise only if the user wants fewer engine kinds for its own sake.

### 1.2 and 1.3 Field groups

Storage stays flat (see 1.9). The schema gains `group` per field; the Forge shows sections. Section "Modifier" is a view over the kind-specific fields (defaulted and omitted from storage by the existing `when` rule).

| Group (Forge section) | Fields | Applies to | Level |
| --- | --- | --- | --- |
| Identity | `id`, `name`, `description` | all | skill |
| Core | `kind`, `blocks`, `family` (see 1.5), `basic`, `learned_only`, `advanced` (SK2), `system` (read-only) | all | skill (shared by all ranks) |
| Rank (table, 1.6) | `level` (AP cost), `tier` (derived from row order), `unlock_offset` (SK2), plus the rank-varying numbers below | all | per rank |
| Modifier: Attack | `power_mult`*, `area`, `offhand`, `stance`, `dash_tiles`*, `weapon_types`, `min_proficiency` | ATTACK | skill (`*` rank) |
| Modifier: Reach | `reach_min`, `reach_max` | ATTACK, SUPPORT | skill |
| Modifier: Status (inflict) | `status_effect`, `status_potency`*, `status_magnitude`*, `status_rounds`* | ATTACK | skill (`*` rank) |
| Modifier: Support | `target_mode`, `heal_stat`, `heal_base`*, `heal_mult`*, `cleanse_statuses`*, `grant_status`*, `status_rounds`* | SUPPORT | skill (`*` rank) |
| Modifier: Defend | `mitigation`*, `grant_status`*, `status_rounds`* | DEFEND | `*` rank |
| Modifier: Focus | `boost`*, `boost_rounds`* | FOCUS | `*` rank |
| Modifier: Disengage | `jump_tiles`* | DISENGAGE | `*` rank |
| Modifier: Rest | `recover_ap` | REST (engine) | skill |

Rank-varying columns are marked in the schema with `"rank": true`. The builder derives the set from data rather than trusting this table: a script lists, per family, which keys differ across tiers (sample: Mend differs in `level`, `heal_base`, `heal_mult`, `description`, `name`); the validator then enforces "non-rank fields are identical across the ranks of a family" (extension of the existing `_family_errors`, which already requires equal kind and basic across tiers).

Defaults: every modifier field keeps its current schema default; a field outside the chosen kind's modifier is not shown and not stored (existing behaviour).

### 1.4 Removed / merged / renamed (all verified by grep)

| Item | Action | Why |
| --- | --- | --- |
| `status_rounds` shared by ATTACK/SUPPORT/DEFEND | stays one field, shown in whichever modifier section applies | merge of presentation, no data change |
| `reach_min/max` for ATTACK and SUPPORT | one "Reach" section | same |
| `tier` | derived from rank order in the Forge; stored as today | user edits the table, not the number |
| `family` free text | becomes a validated reference (1.5) | safety |
| `level` label | "AP cost" (Forge label; stored key unchanged) | clarity; renaming the key touches 9 suites |
| MOVE / TARGET / STATUS / REST kinds | hidden from the Kind dropdown (REST only for the existing rest row); engine group | already engine-only |
| `skill_cap` on units (SK2 changes meaning to an AP override) | out of this task | SK2 |
| Nothing else is unused | no deletions | audit above |

### 1.5 Skill id vs family

Today: `id` is the row id (`mend_2`); `family` is a free string equal to the head id (`mend`); empty = the row's own id. A family groups RANKS of one skill. It does NOT link basic and advanced versions: an advanced skill (Greater Mend) is a separate family (DECISIONS "Skill model v2"; SK2 flags it `advanced`). The loadout, `skill_potency` passives and unit `loadout` lists all key on the family id.

Proposal:

| Field | Type | Rule |
| --- | --- | --- |
| `family` | `ref` to skills (filter `tier == 1`, not system) or empty | Empty or equal to own id = this row is the head (rank 1). Otherwise must name an existing head row; the Forge fills it by picker, never by typing; free text is rejected. A rank row's id must be `<head>_<tier>` (new validation, shipped data already complies per the M2C2B convention; builder verifies with a script and lists exceptions) |
| `evolves_from` (new, optional) | `ref` to skills heads | Informational link "Greater Mend upgrades Mend" for UI and progression text. Not used by the engine, not required. Open question 1 |

The head id stays the family identity, so passives, loadouts and saves are untouched.

### 1.6 Ranks merged into one entry (view, not storage)

- The Forge list shows one entry per family (the head), with a "Ranks" table: one row per tier with columns = the `rank: true` fields for the skill's kind, plus `unlock_offset`. Add rank (up to 3 for class skills, 5 for basics today but basics are single-tier by decision) creates the row `<head>_<n>` through `ContentCatalog`; remove rank only for the HIGHEST rank (no gaps); the head cannot be removed while ranks exist.
- Core and modifier fields are edited once and fanned out to every row of the family on Apply; Name per rank is derived ("Mend" + roman numeral), and may be overridden (e.g. an authored distinct name).
- Apply is all-or-nothing: new `ContentCatalog.upsert_many(kind, entries)` validates the whole set first, writes none on any error.
- The engine, planner (`group_by_family`), tests and saves still see per-rank ids `mend`, `mend_2`, `mend_3`.

### 1.7 Search and filter

Forge list: text over id, name, description; chips for kind, advanced / basic / learned_only, weapon type, status inflicted, rank count; "used by" (class path, unit, nobody), "has problems"; sort by name / kind / AP. Declared per kind in the schema (`"filters": [...]`) so items, classes and units get the same bar.

### 1.8 Basics cannot be deleted

`ContentCatalog.remove` refuses `basic == true` and `system == true`, and also the head of a family that still has ranks, and any entry referenced elsewhere (existing). Basics are editable in numbers, name, description and modifier; their `id`, `kind`, `basic`, `tier`=1 and `family` are read-only (engine and tests reference `strike`, `guard`, `move`, `rest`, `switch_target`, `offhand_jab` by id) and validation rejects changing them. The Delete button is disabled with the reason as tooltip.

### 1.9 Storage: option comparison

| Option | Gameplay | Cost | Risk |
| --- | --- | --- | --- |
| S1. Keep flat per-rank rows; Forge shows the grouped view (groups, ranks table, fan-out writes) | none | schema `group`/`rank` keys, `upsert_many`, Forge form sections; zero data migration | duplicated core fields stay in the JSON (drift is prevented by the validator and the fan-out) |
| S2. New JSON: one entry per skill with `ranks: [...]` and nested `modifier`; `ContentCatalog` expands to per-rank rows at load, schema version 2 | none | migration of 127 rows, `ContentCatalog` load/save/entries/upsert/remove/rename-refs rewrite, `ContentFactory.skill`, 9 test suites and 6 seeds that build flat rows, all fixtures | high: every consumer of `entries("skills")` sees derived rows that cannot be upserted directly; expansion order and ids must stay stable for saves |

Recommendation: S1. It delivers every user-visible goal (1.2, 1.3, 1.6, 1.7, 1.9, 1.10) with no schema version bump. If the user later wants a smaller file, S2 can be built on top of S1 because the view model (head + ranks + modifier) is the same; only `ContentCatalog` load/save would change.

### 1.10 Guard rails (skills)

Validation stays in `ContentCatalog._skill_errors` / `_family_errors`; additions: non-rank fields identical across ranks, rank id convention, ranks contiguous from 1, `unlock_offset` rules (SK2), `evolves_from` must be a head and different family, tier counts (class 3, basic 1), value ranges already in the schema (min/max per field, enforced by SpinBox limits and `coerce`).
