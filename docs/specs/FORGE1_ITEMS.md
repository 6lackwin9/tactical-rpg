# FORGE1_ITEMS: Item tab of the Content Forge (item family template, base item = family + modifiers, tier thresholds)

Status: proposed (design only; nothing here is approved; supersedes the ITEMS part, section 4 and the I1/I2 comparison, of `FORGE1_DESIGN.md`)
Depends on: SKILL1-L commit 2 (shared typed controls, live problems bar, disabled Save) for the Forge UI commit only; the data layer commit has no dependency (SK2 is done)
Lane: logic (ITEM1-L, schema first, three commits), then one small asset task (ITEM1-A)
Touches (when approved): `scripts/content/content_schema.gd` (`_item_fields`, new `_item_family_fields`), `content_catalog.gd` (`_item_errors`, new family and tier validation, version map, load/save), `content_factory.gd` (`item`, `item_to_entry`), new `scripts/core/specs/item_families.gd`, `item_modifiers.gd`, `item_text.gd` (pure RefCounted), `scripts/core/inventory/equipment.gd` (tier bonuses), `scripts/core/combat/combatant.gd` (`sync_from_stats_and_gear`, `rebuild_gear_effects`), `scripts/core/specs/gear_budget.gd`, `passives.gd` (`effect_specs()`, shared with the passives tab), `scripts/ui/forge/*`, new `data/catalog/item_families.json`, `data/catalog/items.json` (version 2), `data/seeds/item1_families.gd`, tests in `FORGE1_ITEMS_MATRIX.md` section 4. Companion: `docs/specs/FORGE1_ITEMS_MATRIX.md` (family rows, 31-item migration).

HISTORY.md check: nothing here re-proposes a deprecated idea. "Hard-coded items -> JSON catalog + Forge" is extended, not reversed. "Gear proficiency as a plain allowed list" stays deprecated (proficiency dict untouched). One replaced idea of FORGE1_DESIGN is logged: its `either_main` hand and "copy" storage (I1), reasons in 0 and 3.1. DECISIONS kept: gear budget weights (soft, special items with notes), hands main_1h/main_2h/off, orb/tome off-hand only, item effects use the passive grammar restricted to item types, stat bonuses never negative, weapons never scale from KNOW, named items = base + tuning, edits only through `ContentCatalog.upsert`.

## Goal

Split items into ITEM FAMILY (template: item type, hands, slots, base numbers, soft bands, allowed modifier kinds, closed engine rules) and BASE ITEM (identity + family reference + a generic list of modifiers + a tier-threshold list). Same pattern as the skills decision (family = template, rank/item = derived by modifiers, resolved against the family with no cascade, derived text). Storage: families in a new `item_families.json`; items on disk as family + modifiers; flat resolved rows in memory so engine, budget, world layer and most tests see today's shape.

## 0. What the code does today (checked)

| Fact | Consequence |
| --- | --- |
| 31 items: 18 WEAPON, 3 SHIELD, 5 ARMOR (all BODY), 2 ACCESSORY, 3 CONSUMABLE; no MATERIAL/QUEST. Per `weapon_type`: sword 3, axe 2, mace 2, hammer 1, fist 1, dagger 2, bow 1, staff 2, wand 2, orb 1, tome 1, shield 3, plate 1, leather 1, cloth 3; crossbow 0 | 20 families cover everything (matrix section 2) |
| `Proficiency.gear_types()` = 13 weapon tokens (incl. `shield`) + plate/leather/cloth. The token drives: class `proficiency` keys, `ClassArchetype.can_equip` (`can_use(weapon_type)`; empty type = always usable), scaling/power/crit/burden/block-cost training, `Combatant.wielded_type()` for skill `weapon_types` gating (main-hand type or `unarmed`), `StashFilter`, `ItemArt` | family id of a gear family MUST equal the token |
| Hand is NOT a family property: sword has a 2H (claymore), a 1H (broadsword) and an off-hand (shortsword, `off_hand_category` shortsword); dagger has 1H (stiletto) and off (wpn_dagger); fist (knuckles) is off; mace scaling is STR (flanged) or INT (cleric mace); staff and bow are 2H. `Equipment.hand()` is derived from `is_two_handed` + `off_hand_category`. A plain 1H weapon fits MAIN_HAND or OFF_HAND (dual wield); an off-category weapon fits OFF_HAND only | FORGE1_DESIGN's single `hand` per family (`either_main`) is wrong. Family needs a SET of allowed hands, the item picks one (replaced idea) |
| F1 latent bug: wpn_shortsword, wpn_knuckles, wpn_dagger store `slot: MAIN_HAND` but are off-hand only (`can_equip_in_slot`). Callers that equip into `eq.slot` (`character_equipment_modal`, `dojo_stash_panel`, `world_factory` named items) would target the wrong slot. The validator accepts it (`Weapons go in MAIN_HAND or OFF_HAND`) | Deriving `slot` from the hand fixes it; migration changes those 3 stored slots to OFF_HAND (intended, tested) |
| Hard-coded type tokens in code: `Equipment.FOCUS_TYPES` (orb, tome), `is_ranged_weapon` (bow, crossbow, sling; `sling` is not a gear type), `is_shield` (SHIELD or category shield), `SkillDef` spell gate (staff or wand), `Inventory` lines 87/111 | These are the "engine rules": 4 names (`focus`, `always_ranged`, `shield`, `spell_source`), see 1.3 |
| Tier thresholds: `{stat, threshold, bonus_power, desc}`; criterion = `get_effective_stat + prof.level x 2 (LEVEL_DISCOUNT)` vs threshold; only `bonus_power` is read (`Equipment.get_effective_equipment_power`, `DamageCalculator`, `ui_theme`); validator checks `stat` and `threshold` exist only; 5 items use it (claymore, warhammer, shortbow, wpn_dagger, wpn_staff), all power-only | The section exists as a power stub; criteria and bonuses are generalised in 4 |
| `GearBudget.item_value` reads the flat row (power, mitigation, stat bonuses, burden, block cost, three effect types) and ignores `threshold_tiers`; `shd_tower` and `wpn_glass_wand` are `special` | Budget keeps reading the resolved flat row; numbers must not move (acceptance) |
| World named item = `base_item` + `tuning` {power, mitigation, burden} + extra `stat_bonuses` + `grants` (`WorldFactory.named_item` copies `get_entry`, edits numbers, calls `ContentFactory.item`); 1 named item shipped (ashlands) | Works unchanged if `get_entry` returns the resolved flat row (5) |
| `effects` and `threshold_tiers` are raw JSON text boxes in the Forge; `required_stats`, `stat_bonuses` use the stats widget | Replaced by typed tables (6) |

## 1. Item families (deliverable a)

### 1.1 Family record (new kind `item_families`, file `data/catalog/item_families.json`, version 1)

| Field | Type | Allowed / range | Default | Rule |
| --- | --- | --- | --- | --- |
| id | id | snake_case, unique; a GEAR family (item_type WEAPON, SHIELD, ARMOR) must equal a token of `Proficiency.gear_types()` and every token must have a family (invariant, like skills REQUIRED_IDS) | "" | read-only after create when items use it |
| name, description | string 1-32, text 0-240 | | "New family" | identity |
| item_type | enum | WEAPON, SHIELD, ARMOR, ACCESSORY, CONSUMABLE, MATERIAL, QUEST | WEAPON | locked while items use it |
| hands | multi_pick | subset of main_1h, main_2h, off; WEAPON non-empty; SHIELD exactly [off]; other types empty | [main_1h] | item `hand` must be one of these |
| default_hand | enum | one of `hands` | main_1h | |
| off_hand_category | enum | "", shield, shortsword, knuckles, dagger, knife, orb, tome; non-empty iff `off` in hands | "" | replaces the per-item key; derived onto the resolved row (used by `ItemArt`, `Equipment`) |
| slots | multi_pick | ARMOR: subset of HEAD, BODY; ACCESSORY: ACCESSORY_1, ACCESSORY_2; weapons/shields: derived from hands (main_* -> MAIN_HAND, main_1h also OFF_HAND as dual wield, off -> OFF_HAND) and not stored | [] | item `slot` must be in it |
| scaling_stats | multi_pick | WEAPON: non-empty subset of STR, DEX, INT (never KNOW, decided M2b) | [STR] | item `scaling_stat` in it; first = default |
| base | five ints | power 0-200 (WEAPON), range 1-6 (WEAPON), block_cost 1-6, burden 0-10, mitigation 0-100 (SHIELD, ARMOR); fields not valid for the type are hidden and stored 0/1 | per matrix | the template numbers |
| bands | per field {min, max} | min <= max, inside the schema range, `base` inside | per matrix rule 2 | resolved item numbers outside the band = warning |
| adjustable | multi_pick | subset of power, range, block_cost, burden, mitigation valid for the type | all valid | which fields an `adjust` modifier may change |
| modifier_kinds | multi_pick | subset of adjust, stat_bonus, requirement, effect | per type | closed list, see 2.3 |
| max_modifiers, max_tiers | int | 0-12, 0-4 | 8, 3 weapons | |
| engine_rules | multi_pick | closed list 1.3 | [] | |
| damage_type | enum | "", slash, pierce, blunt, magic, fire | "" | default of items |
| art | string | `ItemArt` key | "" | default icon (item `icon_id` overrides) |

### 1.2 Family list (verified against code and data; numbers in the matrix)

Weapons: sword, axe, mace, hammer, fist, dagger, bow, crossbow (no item yet), staff, wand, orb, tome. Shield: shield. Armor: plate, leather, cloth. Accessory: accessory. Consumable: consumable. Placeholders: material, quest. Total 20. Hands: sword {1h, 2h, off}, axe {1h, 2h}, mace {1h}, hammer {2h}, fist {1h, off}, dagger {1h, off}, bow/crossbow {2h}, staff {2h, 1h}, wand {1h}, orb/tome {off}, shield {off}. Hands present in data are facts; the rest (mace/hammer/wand 1H-or-2H limits, staff 1H, crossbow) are proposals (question 1).

### 1.3 Engine rules (closed list, code-owned)

| Rule | Meaning today | Families | Where it lives now |
| --- | --- | --- | --- |
| focus | off-hand only, never 2H, a 2H main refuses it, equipping a 2H main stows it | orb, tome | `Equipment.FOCUS_TYPES`, `conflicts_with_main`, `dojo_session` 672/869/873 |
| always_ranged | ranged even at range 1 | bow, crossbow | `Equipment.is_ranged_weapon` |
| shield | counts as a shield (blocks 2H pairing and dual wield) | shield | `Equipment.is_shield`, `Inventory` 111 |
| spell_source | main-hand type that opens spell skills | staff, wand | `SkillDef` line ~167 |

First commit keeps the code constants; `item_families_test_suite` asserts the rule lists equal the constants (no drift). Later commit may read the families instead. A new rule is a code change, stated in the Forge help text; validation rejects unknown rule names. Rule consistency: `focus` requires hands == [off] and WEAPON; `shield` requires SHIELD; `always_ranged`/`spell_source` require WEAPON.

## 2. Base item (deliverable b)

### 2.1 Stored record (items.json version 2, one entry per item)

Identity: `id` (snake_case, unique, fixed prefix not enforced), `name` (1-32), `description` (0-240, tokens allowed, 2.6), `icon_id` (picker of `ItemArt.UNIQUE_ICONS` or ""), `rarity`, `value` 0-99999, `weight` 0.0-50.0, `starter_stash`. Family: `family` (ref, required, picker). Build: `hand` (WEAPON, SHIELD: one of family hands, default family default), `slot` (ARMOR, ACCESSORY only: one of family slots; weapons/shields derive it), `scaling_stat` (WEAPON, default family default), `damage_type` (default family). Lists: `modifiers` (2.2), `tiers` (4). Special: `special`, `special_note`. Consumable payload fields unchanged (3). Everything derived (`item_type`, `weapon_type`, `slot` for weapons, `two_handed`, `off_hand_category`, base numbers) is NOT stored.

### 2.2 Resolved flat row (in memory, what every consumer reads)

`ContentCatalog.entries/get_entry/validate_entry` keep returning today's flat keys: `item_type, slot, weapon_type (= family id for gear families, else ""), two_handed (= hand main_2h), off_hand_category, base_power, base_range, scaling_stat, block_cost, burden, mitigation, required_stats, stat_bonuses, effects, threshold_tiers` plus the new `family, hand, modifiers, tiers`. Resolution (`ItemModifiers.resolve(entry, family)`, pure): numbers = family.base + sum of `adjust` deltas; `stat_bonus` modifiers -> `stat_bonuses` dict; `requirement` -> `required_stats`; `effect` -> `effects` list; `tiers` -> `threshold_tiers` (legacy-compatible shape, 4.3). Inverse `collapse(flat_row, family)` for the saver and for legacy fixtures: a flat row without `family` derives it from `weapon_type` (else `accessory`/`consumable` by item_type) and its modifiers from the differences. `upsert` accepts either shape.

### 2.3 Modifier list (generic; 1 or 12 without engine change)

`modifiers` is an ordered list of `{kind, ...params}`. The engine only reads the resolved sums, so adding a modifier never needs engine code; a NEW KIND needs a descriptor row plus one resolve branch (code). Kinds are a closed list in `ItemModifiers.KINDS` (single source for validator, Forge widgets, budget, text).

| Kind | Params | Range | Unique per | Resolves to | GearBudget value (existing weights) |
| --- | --- | --- | --- | --- | --- |
| adjust | field (power, range, block_cost, burden, mitigation), delta | delta -100..100; resolved value inside the schema range (error) and family band (warning) | field | the number | power x1.0, mitigation x1.5, burden x-2.0, block_cost x-3.0 per step above 1, range 0 |
| stat_bonus | stat (STR DEX INT CONS PERC KNOW), value | 1-8, never negative | stat | `stat_bonuses[stat]` | value x2.0 |
| requirement | stat, value | 1-30 | stat | `required_stats[stat]` | 0 (wield penalty, not value) |
| effect | effect (dict from the passive grammar, item types only) | `Passives.ITEM_TYPES`: skill_potency, status_potency 1..25, spell_reach 1..2, resist; validated by `Passives.validate_effects(items_only=true)` | identical effect twice | `effects` entry | skill_potency x20 per 1.0, status_potency x0.2, spell_reach x3 |

Limits: total <= `family.max_modifiers` (hard cap 12), kinds in `family.modifier_kinds`, `adjust.field` in `family.adjustable`. `GearBudget.modifier_value(mod)` is a pure function; the Forge shows one value per modifier and the sum equals `item_value(resolved row)` (acceptance). The existing soft rule stays: `special` needs `special_note`. Extending the list later (e.g. `mitigation_vs`, `on_hit`) = new descriptor row + resolve branch + budget weight.

### 2.4 Description tokens and derived text (decision "derived names, tokens in descriptions" applied)

Names stay authored (identity). Description may contain tokens resolved at display by `ItemText.describe(entry)`: `{power} {range} {burden} {mitigation} {block_cost} {hand} {family}`, `{req:STR}` (requirement value), `{bon:STR}` (stat bonus), `{tier1.value} {tier1.power}` (tier N). Unknown token or a token whose value does not exist = validation error naming the token; a literal number in the text that equals a numeric field is a soft warning ("use {power}"). Auto-generated lines (not stored): modifier summary ("+2 STR, +2 CONS; Req STR 14"), tier text ("Sunder Guard: +4 Power" from `label` + bonus), hand label. Legacy descriptions are untouched (no tokens = shown as is).

## 3. Copy versus inherit (deliverable c)

| Option | Gameplay | Cost | Risk |
| --- | --- | --- | --- |
| C. Copy: items keep full numbers, family seeds new items and validates bands (FORGE1_DESIGN I1) | none | no engine change; items.json unchanged in shape | family stops being "the template": changing a base never reaches existing items; hands/rules duplicated per item (the F1 bug class stays) |
| I. Inherit: structural facts (type, slot, hand set, weapon token, category, rules) and base numbers resolved from the family at load; items store hand/scaling/modifiers only | editing a family base moves every item that did not override | `ItemModifiers.resolve/collapse`, loader/saver, version 2 file, migration seed (31 items, mechanical), tests | silent budget drift when a base changes |
| H. Hybrid: structure inherited, numbers always explicit | none | as I | modifiers become absolute, loses the "template + deltas" the user asked for |

Recommendation: I. Rails against drift: (1) the Forge family editor shows an impact panel before Apply ("12 items change: wpn_x power 18 -> 20; class loadout totals warrior 23.0 -> 25.0") using `GearBudget`; (2) numbers of the family base are one batch edit through `ContentCatalog.upsert_many(item_families, ...)`; (3) the migration seed verifies `resolve(collapse(row)) == row` and equal `item_value` for all 31; (4) bands warn when a base edit pushes an item outside. Migration cost: 31 items collapse mechanically (matrix section 3); shipped diff is items.json only. Rank "no cascade": an item resolves against its family once, families never inherit from families.

## 4. Tier thresholds (deliverable b, 3.4.3)

### 4.1 Tier record (`tiers`, ordered, up to `family.max_tiers`)

`{label, when, bonus, desc}`: `label` string 0-24 ("Sunder Guard"); `desc` string 0-120 (empty = auto: label + ": " + bonus text); `when` and `bonus` below. Unique `when` per item; ascending by value within a stat; a tier needs at least one non-zero bonus.

### 4.2 Closed lists

| `when.type` | Params | Range | Evaluated by (engine) |
| --- | --- | --- | --- |
| stat_at_least (exists) | stat (STR DEX INT CONS PERC KNOW), value | 1-40 | `get_effective_stat(stat, {weapon}) + proficiency.level x LEVEL_DISCOUNT` >= value (current rule, incl. static gear bonuses, excluding tier bonuses) |
| proficiency_at_least | gear (default own family; any gear token), level | 1-3 (`Proficiency.MAX_LEVEL`) | `proficiency.level_of(gear)` |
| unit_level_at_least | value | 1-10 (`StatRules` levels) | `CharacterStats.level` |

| `bonus` key | Allowed on | Range | Engine status |
| --- | --- | --- | --- |
| power | WEAPON | 0-50 | works today (`get_unlocked_threshold_power_boost`) |
| mitigation | ARMOR, SHIELD | 0-20 | MUST ADD: `Equipment` effective mitigation = base + active tiers, read where armor rating sums `eq.mitigation` (builder greps) |
| stat_bonuses | any gear | stats 1-5 per stat, <= 3 stats | MUST ADD: `Combatant.sync_from_stats_and_gear` adds a second modifier source `gear_tier` after static gear bonuses |
| effects | any gear | item grammar, <= 2 entries | MUST ADD: `rebuild_gear_effects` also applies the effects of active tiers with source `gear:<id>:tier<n>` |

Evaluation order (no circularity, single pass): static gear modifiers -> evaluate every tier against stats WITHOUT tier modifiers -> add tier stat modifiers -> rebuild gear effects incl. tier effects. This reorders the current call (effects are rebuilt first today); acceptance: items without tiers or with power-only tiers produce byte-identical stats and effects. Legacy `{stat, threshold, bonus_power, desc}` rows load as `when stat_at_least` + `bonus power`; the resolved `threshold_tiers` keeps `stat, threshold, bonus_power, desc` for those, so `Equipment`, `DamageCalculator`, `ui_theme` and `character_specs` tests are untouched; extended tiers add `when`/`bonus` keys.

Budget (question 5): today tiers are worth 0. Default of this spec: still 0 in `item_value`; a separate "potential" line in the preview counts active-at-cap value x0.5, informational.

## 5. World named-item layer (deliverable e)

Unchanged in ITEM1: `base_item` (ref `base:items`) + `tuning` {power, mitigation, burden} + `stat_bonuses` + `grants`. `WorldFactory.named_item` reads `get_entry` (resolved flat row), applies the deltas and builds through `ContentFactory.item`, so a base item's modifiers and tiers carry over and base edits propagate (already true). Forge/World preview: `references_to("items", id)` and the family impact panel count named items. Guard: `WorldCatalog._item_errors` additionally requires the base to resolve (family exists). Later option (ITEM2, not now): named items get their own `modifiers` list using the same `ItemModifiers` kinds, so lore items can add an effect; it needs `world_schema` `_item_fields` and `named_item` changes (note: `named_item` does not cap `block_cost` or re-validate the family band; accepted).

## 6. Guard rails, form layout (deliverable f)

Rule: no raw JSON or comma text for list or dict values; every number is a spinbox with schema min/max/step (dynamic limits: delta so the resolved value stays in range; stat value 1-8); enums are dropdowns of the allowed set only; refs are filtered pickers; free text only for id, name, description, label, special_note. Save and Apply are disabled while problems exist; the preview header shows "Cannot save: N problems" with one line per reason (field and row named), offending controls outlined; warnings (bands, budget, tokens, special note) are yellow and never block. Reuses SKILL1-L controls (`multi_pick`, dynamic limits, problems bar); new here: `modifier_table` (kind dropdown swaps the parameter widgets, built from `ItemModifiers.KINDS` and `Passives.effect_specs()`), `tier_table`. Validation returns `{field, index, message}` rows (same pattern as `validate_skill_entry`); `validate_entry` maps them to strings.

Item form sections: 1 Identity (id, name, description + token help, icon, rarity, value, weight, starter_stash); 2 Family and build (family picker; read-only chips item type, gear token, slot; hand; scaling stat; slot for armor/accessory; damage type); 3 Base values (read-only grid with "family base + adjustments = result" per number and band marker); 4 Modifiers (table, add disabled at the cap with the reason); 5 Tier thresholds (table); 6 Special; 7 Consumable payload (consumables only); 8 Preview (resolved summary, per-modifier budget values, loadouts and named items that use it, problems, warnings). Family form: Identity, Type and hands, Template numbers and bands, Modifier policy, Engine rules (checkbox per rule with its help), impact panel, "Used by" (items).

Rule-derived validation (hard unless marked soft): family of an item exists; hand in family hands; scaling in family list; slot in family slots; `adjust` field/kind allowed, unique, resolved value within the schema range; band violation (soft); modifier counts; stat and requirement unique per stat; effects pass the item grammar; tier criteria, bonuses and counts per 4; removing a family with items is refused; changing a family `item_type` or `hands` revalidates all its items and is refused with the failing ids; consumable fields only for CONSUMABLE; a WEAPON with `focus` has power band max <= 10 (soft); `weapon_type` token of gear families equals the family id.

## 7. Consumables (deliverable d)

Defer. ITEM1 gives consumables a single `consumable` family (so every item has a family) with no modifiers; the payload fields (`consumable_type`, `heal`, `damage`, `buff_stat`, `buff_pct`, `duration`, `use_blocks`, `max_stack`) stay on the item with typed controls and existing ranges. Family-per-`consumable_type` (potion, throwable, brew) and modifier-style buffs wait for the consumables rework (open question 6 of FORGE1_DESIGN stays open). `material` and `quest` families exist only as placeholders.

## 8. Storage, migration, tests (deliverables g, h)

Storage: new kind `item_families` (20 rows) + `items.json` version 2 (family + modifiers + tiers). `ContentCatalog` gets a `VERSIONS` map shared with SKILL1-L (items 2, item_families 1); version-1 flat files still load (collapse on load, legacy fixture rows keep working). The seed `data/seeds/item1_families.gd` (idempotent) writes the 20 families, then re-saves items through `collapse`. Acceptance: validate_all empty; 31 items; for every item the resolved flat row equals the pre-migration row on all engine keys except the 3 intended slot changes (F1); `item_value` identical; every class and unit loadout total identical; arena (202), runner, world, progression, save suites pass unchanged.

New suites (one `results.append` each): `item_families_test_suite` (record validation, invariants, rule list equals the code constants, remove/retype refusals), `item_modifiers_test_suite` (round trip over every shipped item, sandbox edits: family base change moves only inheriting items, kind/cap/uniqueness errors, budget sum equals `item_value`, legacy flat upsert), `item_tiers_test_suite` (each criterion and bonus: stat, proficiency, unit level; power, mitigation, stat_bonuses, effects; no circularity; ordering; byte-identical when no tiers), `item_form_model_test_suite` (headless: sections, disabled-Save reasons, dynamic limits, token resolution). Updates to existing suites: matrix section 4. Rule: derive ids/counts from the catalog, never hard-code.

## 9. Milestone plan and ordering (deliverable i)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| ITEM1-L commit 1 | logic | `ItemFamilies`, `ItemModifiers`, `ItemText`, schema (`item_families` kind, items v2 fields), loader/saver/VERSIONS, family/item/tier validation, `GearBudget.modifier_value`, new tests 1-2, updated tests. Schema first: asset task released after this commit | - |
| ITEM1-L commit 2 | logic | Tier engine: `mitigation`, `stat_bonuses`, `effects` bonuses, `proficiency_at_least`, `unit_level_at_least`, evaluation order, `Passives.effect_specs()` (shared with passives tab), test 3 | commit 1 (shape), question 5 |
| ITEM1-L commit 3 | logic | Forge: family form, item sections, modifier and tier tables, impact panel, problems bar, filter chips (family, hand, rarity, special, has tiers, has effects, used by class/unit/named item), model test 4 | commit 1, SKILL1-L commit 2 |
| ITEM1-A | asset | Run the migration seed, confirm the NEW family proposals, optionally convert the descriptions that quote numbers (claymore, warhammer "22 STR") to tokens | commit 1 |

Hub files shared with the other passes: `content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `forge_form.gd`, `content_forge.gd`, `forge_preview.gd`, `passives.gd` (skills edits `skill_def`, classes edit `class_archetype`, `class_forge_model`). Hub conflicts, so sequential. Recommended order: SKILL1-L (commits 1, 2) -> ITEM1-L commit 1 + ITEM1-A -> CLASS1-L (its loadout pickers can then group by family; it needs no item schema otherwise) -> ITEM1-L commits 2 and 3. Commit 2 touches only `equipment.gd`, `combatant.gd`, `gear_budget.gd`, `passives.gd` (no content hub) so it may run in parallel with CLASS1-L. Safe anytime: balance sims, M4 quest runtime.

## 10. Out of scope

Engine kind or rule additions beyond the 4 existing rules; new gameplay values or rebalancing (migration is value-neutral); named-item modifiers (ITEM2); consumable rework; making families inherit families; art; changing proficiency, class or skill schema; reading gear tokens from families inside `Proficiency` (constants stay, invariant test guards drift); any implementation.

## 11. Open questions (for the user)

1. Confirm the proposed (not data-backed) family hands and the NEW rows: mace/wand 1H only, hammer 2H only, staff also 1H, crossbow (copy of bow, no item), material and quest placeholders.
2. Confirm Option I (inherit; family base edits propagate, with the impact panel) over C (copy). Numbers as deltas (modifiers `adjust`) or keep absolute numbers on the item (H)?
3. Plain 1H weapons currently fit either hand (dual wield). Keep that for every `main_1h` family, or restrict dual wield to a family flag?
4. Band violations: soft warning (proposed until the balance pass) or hard error now?
5. Tier bonuses: implement stat_bonuses, effects, mitigation, proficiency and unit-level criteria now (commit 2), and should GearBudget count tiers (proposal: not in `item_value`, informational potential at x0.5)?
6. Descriptions: adopt tokens (2.4) now, or leave descriptions numberless and fully authored?
