# FORGE1: Content Forge structure and schema overhaul (skills, items, classes; units follow)

Status: proposed (design only: structure and schema first, no implementation approved)
Depends on: SK2 (its `advanced` / `unlock_offset` fields become part of the rank and core groups below; both touch the same hub files)
Lane: design now; later logic (schema first) then asset
Touches (when approved): `scripts/content/content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `scripts/ui/forge/*` (form, forge, preview, source), `scripts/core/specs/passives.gd` (effect grammar exposed as data), `scripts/core/inventory/equipment.gd` (threshold effects, FG2 only), `scripts/core/specs/gear_budget.gd` (FG2), new `data/catalog/item_families.json` (FG2), tests listed in section 7

Superseded in part (2026-10-16): the SKILLS part of this file (section 1, the skill rows of section 2 and the FG1 skill items in sections 6 and 7) is superseded by `docs/specs/FORGE1_SKILLS.md` (kind matrix, ranks in one entry with flat rows in memory instead of S1, typed guard rails). The original section 1 now lives in `docs/archive/FORGE1_DESIGN_SKILLS.md`. Classes, items and units sections stay as the starting point for their own later passes.

HISTORY.md check: nothing here re-proposes a deprecated idea. Relevant rows: "Hard-coded classes, items, skills in code -> JSON catalog + Forge" (we keep the catalog; we only change how it is presented and validated), "Gear proficiency as a plain allowed list" (the proficiency dict stays a dict), "Mastery" (not touched). DECISIONS respected: skill tiers stay separate skill ids grouped by `family`+`tier`, basics are the six single-tier skills, item `effects` use the passive grammar, GearBudget values items, hands main_1h/main_2h/off, special items with notes, tier numbers explicit per row, edits through `ContentCatalog.upsert`.

## 0. Findings that shape the proposal

| Fact (checked in code/data) | Consequence |
| --- | --- |
| Storage is already schema-driven with `when` conditions: `ContentSchema.to_storage` leaves out fields hidden by `kind`, so kind-specific fields are already "defaulted/hidden" per kind. The pain is the flat 36-field skill list and free-text fields, not dead data | A grouped VIEW needs one new schema key (`group`), not a new file shape |
| Skills: 127 rows, 78 are rank 2-3 rows. Merging ranks drops the list to about 49 entries (38-39 class families + 6 basics + 5 system). Kinds in data: ATTACK 88, SUPPORT 18, DEFEND 7, DISENGAGE 6, FOCUS 3, MOVE/TARGET/STATUS/REST are engine rows | Rank merge is the big declutter; kind reduction is small |
| Rank ids follow `<head>` / `<head>_2` / `<head>_3` (M2C2B: "tier ids are `<id>_2`, `<id>_3`"). `Combatant`, `Passives` (`skill_potency` family), unit `loadout`, `extra_skills`, class `skill_path` all key on the HEAD id (the family) or a row id | Keeping per-rank row ids and the head-as-family convention keeps every reference valid |
| At least 9 test suites and 6 seed scripts build skill rows as flat dictionaries through `upsert`/`normalize` (ai_skills, combat_extensions, content, engine_pieces, foundation, riposte, runner, skill_family, weaken; seeds c1a1, m2c2b_lib...). 31 test references to rank ids (`mend_2`, `riposte_2` ...). Worlds hold no skill ids directly (characters reach skills through `base_unit`); named items reference `base_item` ids | Changing the stored JSON shape would break a large test/seed surface for no engine gain |
| Item `threshold_tiers`: only `bonus_power` is applied by the engine (`Equipment.get_unlocked_threshold_power_boost`, `DamageCalculator`, 5 shipped uses); `desc` is display; the validator only checks `stat` and `threshold` exist | The tier-threshold section exists but is a power-only stub: generalise its schema, implement more bonus kinds later |
| Items carry both `item_type`, `slot`, `weapon_type`, `two_handed` and `off_hand_category`; the validator already forces `off_hand_category == weapon_type` for orb/tome and the hand is DERIVED (`Equipment.hand()`) | These are family-level facts stored per item: redundancy that an item family removes |
| World named items already layer on a base item (`base_item` + `tuning` + `grants`); characters on a base unit (`base_unit` + `tuning`) | Items become 3 layers: family -> base item -> named item. Units: class -> unit -> character |
| The Forge already blocks `apply()` while `draft_problems()` is non-empty and shows errors in a side list; the list marks invalid entries with a warning sign; `remove()` refuses `system` entries and referenced entries. Missing: basics deletion rule, live disabled Apply/Save with reasons in the preview, typed editors (`json` and comma `strings` widgets are raw text), search is a single text box (plus a unit side filter) | Guard rails are an extension of what exists, not a rebuild |

## 1. SKILLS

Moved to `docs/archive/FORGE1_DESIGN_SKILLS.md` (2026-10-16). Current skills design: `docs/specs/FORGE1_SKILLS.md` and `docs/specs/FORGE1_SKILLS_MATRIX.md`.

## 2. UI guard rails and typed editors (all kinds; user 1.9 / 1.10)

| Need | Design |
| --- | --- |
| Block saving, show the reason | `ContentForge` recomputes `draft_problems()` on every edit; while non-empty: Apply and Save buttons disabled, a red "Cannot apply: N problems" header at the top of the preview with each reason as a line (also the existing side list). Keyboard Apply is ignored. `commit_draft` on navigation keeps asking (stay on the draft) |
| Soft warnings (GearBudget etc.) | shown yellow under the problems; never disable Apply (existing separation `validate_*` vs `warnings_for`) |
| No raw JSON | New schema field types, rendered by `forge_form.gd`: `table` (`columns: [field defs]`, add/remove/reorder rows, per-cell typed widget, row validation), `multi_pick` (checklist from `options`/`options_from`, replaces comma `strings`), `dict_pick` (key picker + typed value for stats-like dicts, e.g. tuning, status_resist, proficiency), `effects` (a table whose "type" cell swaps the parameter widgets) |
| Single source for the effect grammar | `Passives` exposes `effect_specs()` (type -> parameters with kind, range, allowed values, and which types items may use); both `validate_effects` and the Forge editor read it, so UI and validator cannot drift |
| Pickers | refs use filtered pickers (family heads, gear types from the valid list, statuses from `StatusRegistry`, passives, items by slot); free text only for id, name, description |
| JSON escape hatch | A collapsed "Advanced: raw JSON" read-only view for debugging; edits only via typed widgets |

Fields converted: skills `weapon_types`, `cleanse_statuses`; items `required_stats`, `stat_bonuses`, `effects`, `threshold_tiers`; passives `effects`; classes `proficiency`, `skill_path`; units `spend`, `loadout`, `status_resist`; encounters `exits` (grid picker, later); world `tuning` (later, world schema).

## 3. CLASSES (user section 2)

Classes have one row each (6) and are low-volume; their pain is the raw JSON `proficiency` and `skill_path`. Storage stays flat (same reasoning as 1.9, plus `StatRules`, `GearBudget`, `PartyBuilder` and 5 suites read the flat keys).

| Section (Forge) | Fields | Editor | Validation (existing unless marked new) |
| --- | --- | --- | --- |
| Identity | `id`, `name`, `description`, `art`, `ai_role` | text, art picker, role dropdown | art key valid; `ai_role` enum |
| Stat behaviour | `base_stats`, `growth` | six spinners each with a live total ("62 / 62", "5 / 5, max 3 per stat" turns red) | `StatRules.validate_class` |
| Gear proficiency | `proficiency` | table: gear type (picker from `Proficiency.gear_types()` minus already used), level 1-3, scaling 1.0-2.0 step 0.05 | existing `_class_errors`; new: at least one weapon type (exists) |
| Skill progression | `skill_path` | table: level (1-10), family picker (heads only, non-system); basics allowed; ascending order auto-sorted; shows tiers that open (SK2: offset) | existing + SK2 (advanced floor, one step per family, tier-1 rows only) |
| Default loadout | `main_hand`, `off_hand`, `head`, `body`, `accessory_1`, `accessory_2`, `pouch` | item pickers filtered by slot and by what the class can wield (proficiency) | `_loadout_errors`; GearBudget warning shown live with the loadout total and the 23 +-20% band |

Nothing is removed. The user's modular list maps one to one onto field groups; `art` + `ai_role` + `description` form the identity group.

## 4. ITEMS (user section 3)

### 4.1 Model

Three layers: ITEM FAMILY (template/behaviour) -> BASE ITEM (catalog row) -> NAMED ITEM (world layer, existing `base_item` + `tuning`).

An item family corresponds to today's gear type token (`sword`, `axe`, `mace`, `hammer`, `fist`, `dagger`, `bow`, `crossbow`, `staff`, `wand`, `orb`, `tome`, `shield`, `plate`, `leather`, `cloth`) plus non-gear families (`accessory`, `potion`, `throwable`, `brew`, `material`, `quest`). Family id == gear type token, so `Proficiency`, skill `weapon_types`, and class proficiency keep working unchanged.

Family record (new catalog kind `item_families`, file `data/catalog/item_families.json`):

| Field | Type | Allowed | Meaning |
| --- | --- | --- | --- |
| `id` | id | snake_case; must equal a token in `Proficiency.gear_types()` for gear families | identity |
| `name`, `description` | string, text | | |
| `item_type` | enum | WEAPON, SHIELD, ARMOR, ACCESSORY, CONSUMABLE, MATERIAL, QUEST | replaces per-item `item_type` |
| `slots` | strings (pick) | MAIN_HAND, OFF_HAND, HEAD, BODY, ACCESSORY_1, ACCESSORY_2 | allowed slots (replaces the slot-vs-type chain in `_item_errors`) |
| `hand` | enum | main_1h, main_2h, either_main, off, none | replaces `two_handed` and `off_hand_category`; main 2H vs 1H becomes a family default, with per-item choice only for `either_main` families (sword, axe, mace, hammer, staff) |
| `default_scaling_stat` | enum | STR, DEX, INT | |
| `default_range` | int 1-6 | | `base_range` default (bow/crossbow/staff/wand 2-3) |
| `default_block_cost`, `default_burden` | int | 1-6, 0-10 | |
| `power_band` | {min, max} | ints 0-200 | validation band for weapon `base_power` (warning in FG2.0, error later) |
| `mitigation_band` | {min, max} | 0-100 | armor/shield |
| `allowed_modifiers` | strings (pick) | `stat_bonuses`, `effects`, `requirements`, `thresholds` | which modifier sections the family permits; `effects` further limited to the item grammar |
| `engine_rules` | strings (fixed enum, code-owned) | `off_hand_only`, `refuses_2h_main`, `unarmed_token`... | names of rules the engine already enforces (orb/tome off-hand only, 2H refuses off-hand focus); the data only names which family has them, the code implements them. New rules need code (stated plainly) |
| `damage_type` | enum | "", slash, pierce, blunt, magic, fire | default |
| `art` | string | ItemArt key | default icon |

Base item (the existing item row, regrouped):

| Group | Fields | Notes |
| --- | --- | --- |
| Identification | `id`, `name`, `description`, `icon_id` (artwork), `rarity`, `value`, `weight`, `starter_stash` | |
| Family | `family` (ref, required) | derives `item_type`, `slot` list, hand, gear type; the stored `item_type`/`weapon_type` stay in the JSON as written (S1) and are validated to MATCH the family |
| Base values (inherit family defaults) | `base_power`, `base_range`, `scaling_stat`, `block_cost`, `burden`, `mitigation`, `two_handed` (only for either_main families), `slot` (when the family allows more than one) | prefilled from the family on create; Forge shows "= family default" or a delta marker and a "reset to family" button; validated against bands |
| Modifiers (generic list) | `stat_bonuses`, `effects[]`, `required_stats` | any count (0 to N) with no engine change: `Equipment.effects` and `stat_bonuses` are already open-ended lists/dicts rebuilt in `rebuild_gear_effects`; GearBudget already weights each |
| Tier thresholds | `threshold_tiers[]` | see 4.3 |
| Special | `special`, `special_note` | unchanged |
| Consumable payload | `consumable_type`, `heal`, `damage`, `buff_stat`, `buff_pct`, `duration`, `use_blocks`, `max_stack` | family `potion`/`throwable`/`brew`; `consumable_type` stays an item field in FG2 (open question 4) |

### 4.2 Items storage: option comparison

| Option | Gameplay | Cost | Risk |
| --- | --- | --- | --- |
| I1. Flat rows keep full values; family only supplies defaults for new items, validation bands and Forge layout; explicit "re-apply family defaults" action | none | new kind `item_families` + schema/validation + Forge sections; no item migration (verify `family` per item by a seed script: item `family` = its `weapon_type`) | changing a family default does not propagate (by design; the action does it on request) |
| I2. Items store only deltas to the family; `ContentFactory.item` resolves | family edits propagate | `ContentCatalog.get_entry` returns raw deltas, so `GearBudget`, forge preview, factory, 5 suites and 4 seeds must call a new `resolve()` first; JSON migration of 31 items | medium-high: any missed consumer reads wrong numbers silently |

Recommendation: I1 in FG2, with `ItemFamilies.resolve(entry)` written as a pure helper so I2 is a later switch if the user wants propagation. Item count is only 31, so inheritance by copy is cheap to audit.

### 4.3 Tier thresholds (tiered affixes)

Existing: `{stat, threshold, bonus_power, desc}`, only `bonus_power` is engine-backed. Proposed generalisation (backwards compatible: old rows stay valid):

| Field | Type | Allowed | Default | Meaning |
| --- | --- | --- | --- | --- |
| `stat` | enum | STR DEX INT CONS PERC KNOW | required | unlock criterion stat (wielder's stat incl. gear) |
| `threshold` | int | 1-40 | required | stat value that unlocks the tier |
| `bonus_power` | int | 0-50 | 0 | existing, weapons |
| `stat_bonuses` | stats | allowed keys, positive | {} | extra stats while unlocked (FG2.1) |
| `effects` | list (item grammar) | same as item `effects` | [] | extra effects while unlocked (FG2.1) |
| `desc` | string | | "" | display; Forge auto-generates from the numbers when empty |

Rules: tiers sorted ascending by threshold per stat; thresholds must be unique per stat; a tier with no bonus is an error; a family must list `thresholds` in `allowed_modifiers`. Engine work: FG2.0 changes only schema/validation/Forge (`bonus_power` already works); FG2.1 (optional, separate decision) makes `rebuild_gear_effects` and the stat-bonus sum conditional on the tier being unlocked (touches `equipment.gd`, `combatant.gd`, `gear_budget.gd`: budget value counts the tier bonus at a discount, open question 5).

### 4.4 Removed / merged (items, verified)

| Field | Action | Why |
| --- | --- | --- |
| `off_hand_category` | derive from family `hand` + gear type; stop authoring it | validator already forces equality with `weapon_type` for orb/tome; shield/shortsword/knuckles/knife values are not read by `Equipment.hand()` beyond the focus check (builder greps before removal; keep the stored key until all readers use `hand`) |
| `two_handed` | author only for `either_main` families; otherwise derived | |
| `item_type`, `slot`, `weapon_type` | derived from family on create; validated to match | |
| `damage_type` | keep; family default | used by the stash filter and Dojo panel (display/filter, not combat) |
| `icon_id` | keep (2 uses) under Identification | |

## 5. Extension to UNITS (not in the first milestone)

Units today: identity (`id`, `name`, `side`, `description`, `art`), build (`class_id`, `level`, `tier`, `spend`, `stat_mode` + fixed vitals), gear overrides (loadout slots, `pouch`), skills (`extra_skills`, `loadout`, `skill_cap`), AI (`ai_role`, `target_pref`, `flee_ratio`, `defend_below`, `prefer_level`, `reckless`, `status_resist`), combat overrides (`ap`, `power`, `armor`). 15 rows.

| Option | Effect | Cost | Risk |
| --- | --- | --- | --- |
| U1. Same grouped view only (sections Identity, Build, Gear, Skills, AI, Overrides), typed editors for `spend`, `status_resist`, `loadout` | cleaner form | schema `group`, no data change | none |
| U2. Add kind `unit_templates` = class + tier + AI behaviour preset (e.g. "minion brawler, reckless"); a unit references `template` and keeps only name, level, spend, and explicit overrides; resolved like I1 | less repetition when there are many enemies | new kind, resolver, 3-layer inheritance (template -> unit -> world character tuning) | premature at 15 units; AI presets already exist in `plan_presets.gd` |

Recommendation: U1 inside FG1, U2 only when the unit count makes repetition painful (open question 6). World connection points stay as they are: `base_unit`, `base_item`, `base:passives`; the Forge `references_to` and `Used by` panel are extended with family and rank awareness (usage of a rank counts as usage of the family).

## 6. Migration plan (coarse milestones; schema-first inside the logic task)

| Milestone | Task | Content | Depends on |
| --- | --- | --- | --- |
| FG1 (one logic task, one small asset task) | FG1-L | (1) schema layer: `group`, `rank`, `filters`, field types `table`, `multi_pick`, `dict_pick`, `effects`; `Passives.effect_specs()`; (2) `upsert_many`, `remove` rules, family/rank validation, evolves_from; (3) Forge: sections, ranks table, live problems and disabled Apply/Save, filter bar, typed editors for skills, classes, passives, units; commit the schema first, then UI | SK2 merged |
| | FG1-A | seed pass only if the family/rank audit script finds exceptions (rename ids, set `evolves_from` on advanced skills); otherwise no asset task | FG1-L schema commit |
| FG2 (one logic task, one small asset task) | FG2-L | `item_families` kind, item `family` ref + derived/validated fields, band warnings, item Forge sections, generic modifier tables, threshold schema (FG2.0); optional FG2.1 conditional tier bonuses | FG1-L (shares schema/catalog/forge files) |
| | FG2-A | author the 22 family rows (table below), set `family` on 31 items, move threshold `desc` into the generated form | FG2-L schema commit |
| FG3 (idea) | units template | U2 if wanted | FG2 |

Hub files that conflict (run these milestones sequentially, not in parallel with each other or with SK2/C-series): `content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `forge_form.gd`, `content_forge.gd`, `forge_preview.gd`, `passives.gd`. Safe in parallel with FG1: AI work, balance sims, M4 quest runtime.

Family seed values for FG2-A (starting values, derived from the shipped items; the asset builder confirms against current rows rather than inventing numbers):

| Family | item_type | slots | hand | scaling | range | block cost | engine_rules |
| --- | --- | --- | --- | --- | --- | --- | --- |
| sword, axe, mace, hammer | WEAPON | MAIN_HAND | either_main | STR | 1 | 1 | |
| dagger | WEAPON | MAIN_HAND, OFF_HAND | main_1h | DEX | 1 | 1 | |
| fist | WEAPON | MAIN_HAND | main_1h | STR | 1 | 1 | |
| bow, crossbow | WEAPON | MAIN_HAND | main_2h | DEX | 2-3 | 1 | |
| staff | WEAPON | MAIN_HAND | either_main | INT | 2 | 1 | |
| wand | WEAPON | MAIN_HAND | main_1h | INT | 2 | 1 | |
| orb, tome | WEAPON | OFF_HAND | off | INT | 1 | 1 | off_hand_only, refuses_2h_main |
| shield | SHIELD | OFF_HAND | off | n/a | n/a | 1 | |
| plate, leather, cloth | ARMOR | HEAD, BODY | none | n/a | n/a | per item | |
| accessory | ACCESSORY | ACCESSORY_1, ACCESSORY_2 | none | n/a | n/a | n/a | |
| potion, throwable, brew, material, quest | CONSUMABLE / MATERIAL / QUEST | none | none | | | | |

Exact range/hand defaults per family must be read from the shipped items by the builder (e.g. which of staff/bow are 2H); the table above is the expected shape, not authoritative numbers.

## 7. Tests needing rule-derived updates

| Suite | Update |
| --- | --- |
| `content_test_suite.gd` | new cases: non-rank field mismatch across ranks, rank id convention, non-contiguous ranks, `upsert_many` all-or-nothing, basics undeletable, family ref validation, item family match, threshold rules; derive ids from the catalog |
| `skill_family_test_suite.gd` | family validation messages change from free text to ref errors; rank-add/remove through the model; no hard-coded family names |
| `class_rules_test_suite.gd` | `skill_path` table sort/validation helpers; proficiency table bounds (derive from `Proficiency.MAX_LEVEL`, `MIN/MAX_SCALING`) |
| `gear_budget_test_suite.gd`, `focus_gear_test_suite.gd`, `foundation_test_suite.gd` | orb/tome/hand rules asserted through `engine_rules` and `hand` of the family, not item literals; budget numbers unchanged (items keep full values under I1) |
| new `forge_model_test_suite.gd` | headless view-model tests: grouped fields per kind, ranks table fan-out, disabled-Apply reasons list, filter predicates, typed table coerce (no scene needed; one `results.append` line in `suite_registry.gd`) |
| `arena_test_suite` (202), `runner`, `world`, `progression`, `save` suites | must pass unchanged: storage shape and ids are unchanged under S1/I1 (this is the acceptance gate) |

## 8. Risks

| Risk | Mitigation |
| --- | --- |
| Fan-out writes make a rank edit touch several rows; a bug could desync ranks | `upsert_many` validates all rows first; validator re-checks non-rank equality on load (`validate_all`) |
| Family records are data but engine rules are code | `engine_rules` is a closed enum; adding a rule is a code change and the spec says so; validation rejects unknown rule names |
| Items inherit by copy, so a family default change does not propagate | explicit "re-apply defaults" action plus a drift report; I2 left open |
| Typed editors are Forge UI work (the largest cost, scene-free model tested headless) | build the view model in `RefCounted` classes first, widgets thin |
| Conflict with SK2 and any other schema work | FG1 starts after SK2-L merges |
| Removing `off_hand_category` authoring could break Dojo/stash code | keep the stored key until a grep shows no reader besides the focus check |

## 9. Out of scope

Engine kind merging (option B), changing skill ids or rank ids, the stored JSON shape of skills/classes/items/units, unit templates (FG3), world-layer schema changes (`tuning` editor), new item families beyond today's tokens, new gameplay values or rebalancing, art, any implementation.

## 10. Open questions (for the user)

1. `evolves_from`: add the optional link between an advanced family and its base family (UI text, progression), or keep advanced families fully independent as today?
2. Storage: confirm S1/I1 (flat storage plus grouped Forge view, no JSON migration). The alternative (S2/I2: nested shape with schema version 2) is a much bigger, riskier migration for the same visible result.
3. Item family as a data kind (`item_families.json`, editable in the Forge) versus a code registry like `StatusRegistry`. Data is recommended, but adding a family with new engine behaviour still needs code.
4. Should consumable behaviour (`consumable_type`) become family-level in FG2, or wait until consumables are reworked?
5. Tier thresholds: implement conditional stat/effect bonuses (FG2.1) now, and if so how should GearBudget value a bonus that is not always active (suggest 50%)?
6. Units: stay at grouped view (U1) until there are many enemies, or build templates (U2) in FG3?
7. Rank names: derive "Mend II" automatically, or keep each rank's name authored?
