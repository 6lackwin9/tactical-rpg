# FORGE1_ITEMS: Item tab of the Content Forge (item family template, base item = family + modifiers, tier thresholds)

Status: spec (revised 2026-10-16 with the user's answers, DECISIONS rows "FORGE1 class answers", "FORGE1 skill answers 2", "FORGE1 item answers", "Strict hand typing"; not approved; supersedes the ITEMS part, section 4 and the I1/I2 comparison, of `FORGE1_DESIGN.md`)
Depends on: the skills pass (SKILL1-L: shared typed controls, `origin` rules, description token mechanism, `VERSIONS` map) for the Forge UI commit and the token/origin parts; the data layer otherwise has no dependency (SK2 is done)
Lane: logic (ITEM1-L, schema first, three commits), then one small asset task (ITEM1-A)
Touches (when approved): `scripts/content/content_schema.gd` (`_item_fields`, new `_item_family_fields`), `content_catalog.gd` (`_item_errors`, new family and tier validation, version map, load/save), `content_factory.gd` (`item`, `item_to_entry`), new `scripts/core/specs/item_families.gd`, `item_modifiers.gd`, `item_text.gd` (pure RefCounted), `scripts/core/inventory/equipment.gd` (tier bonuses), `scripts/core/combat/combatant.gd` (`sync_from_stats_and_gear`, `rebuild_gear_effects`), `scripts/core/specs/gear_budget.gd`, `passives.gd` (`effect_specs()`, shared with the passives tab), `scripts/ui/forge/*`, new `data/catalog/item_families.json`, `data/catalog/items.json` (version 2), `data/seeds/item1_families.gd`, tests in `FORGE1_ITEMS_MATRIX.md` section 4. Companion: `docs/specs/FORGE1_ITEMS_MATRIX.md` (family rows, 31-item migration).

HISTORY.md check: nothing here re-proposes a deprecated idea. "Hard-coded items -> JSON catalog + Forge" is extended, not reversed. "Gear proficiency as a plain allowed list" stays deprecated (proficiency dict untouched). One replaced idea of FORGE1_DESIGN is logged: its `either_main` hand and "copy" storage (I1), reasons in 0 and 3.1. Dual wield: DECISIONS "FORGE1 item answers" keeps the code path and adds a blocking guard (section 8); nothing in HISTORY.md touches it. DECISIONS kept: gear budget weights (soft, special items with notes), hands main_1h/main_2h/off, orb/tome off-hand only, item effects use the passive grammar restricted to item types, stat bonuses never negative, weapons never scale from KNOW, named items = base + tuning, edits only through `ContentCatalog.upsert`.

## Goal

Split items into ITEM FAMILY (template: item type, hands, slots, base numbers, soft bands, allowed modifier kinds, closed engine rules) and BASE ITEM (identity + family reference + a generic list of modifiers + a tier-threshold list). Same pattern as the skills decision (family = template, rank/item = derived by modifiers, resolved against the family with no cascade, derived text). Storage: families in a new `item_families.json`; items on disk as family + modifiers; flat resolved rows in memory so engine, budget, world layer and most tests see today's shape.

## 0. What the code does today (checked)

| Fact | Consequence |
| --- | --- |
| 31 items: 18 WEAPON, 3 SHIELD, 5 ARMOR (all BODY), 2 ACCESSORY, 3 CONSUMABLE; no MATERIAL/QUEST. Per `weapon_type`: sword 3, axe 2, mace 2, hammer 1, fist 1, dagger 2, bow 1, staff 2, wand 2, orb 1, tome 1, shield 3, plate 1, leather 1, cloth 3; crossbow 0 | 20 families cover everything (matrix section 2) |
| `Proficiency.gear_types()` = 13 weapon tokens (incl. `shield`) + plate/leather/cloth. The token drives: class `proficiency` keys, `ClassArchetype.can_equip` (`can_use(weapon_type)`; empty type = always usable), scaling/power/crit/burden/block-cost training, `Combatant.wielded_type()` for skill `weapon_types` gating (main-hand type or `unarmed`), `StashFilter`, `ItemArt` | family id of a gear family MUST equal the token |
| Hand is NOT a family property: sword has a 2H (claymore), a 1H (broadsword) and an off-hand (shortsword, `off_hand_category` shortsword); dagger has 1H (stiletto) and off (wpn_dagger); fist (knuckles) is off; mace scaling is STR (flanged) or INT (cleric mace); staff and bow are 2H. `Equipment.hand()` is derived from `is_two_handed` + `off_hand_category`. A plain 1H weapon fits MAIN_HAND or OFF_HAND today (dual wield; ends with strict typing, 8.1); an off-category weapon fits OFF_HAND only | FORGE1_DESIGN's single `hand` per family (`either_main`) is wrong. Family needs a SET of allowed hands, the item picks one (replaced idea) |
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
| slots | multi_pick | ARMOR: subset of HEAD, BODY; ACCESSORY: ACCESSORY_1, ACCESSORY_2; weapons/shields: derived from the item hand and not stored (main_1h and main_2h -> MAIN_HAND, off -> OFF_HAND; main_1h also OFF_HAND only while the code flag `Equipment.allow_plain_dual_wield` is true, 8.1) | [] | item `slot` must be in it |
| origin | enum | system, user (shared rules, section 2.5) | user | system families: id, item_type, hands, off_hand_category, engine_rules locked |
| scaling_stats | multi_pick | WEAPON: non-empty subset of STR, DEX, INT (never KNOW, decided M2b) | [STR] | item `scaling_stat` in it; first = default |
| base | per hand (key = each hand in `hands`; non-weapons use key `none`), five ints | power 0-200 (WEAPON), range 1-6 (WEAPON), block_cost 1-6, burden 0-10, mitigation 0-100 (SHIELD, ARMOR); fields not valid for the type are hidden and stored 0/1 | per matrix | the template numbers; an item resolves against `base[item.hand]` |
| bands | per hand, per field {min, max}; plus `bonus_value` {min, max} per hand = GearBudget value of the item's non-power modifiers (stat_bonus + effect) | min <= max, inside the schema range, `base[hand]` inside | per matrix rule 2; `bonus_value` main hands 0-4, `off` 0-6 (seed defaults, balance lane tunes) | resolved item outside its hand's band = warning. Off-hand weapons are budgeted LOWER on power and HIGHER on bonus value. Family warnings (soft): `off` power band max or base above the `main_1h` ones (family with both hands); `off` `bonus_value` max below the main one |
| adjustable | multi_pick | subset of power, range, block_cost, burden, mitigation valid for the type | all valid | which fields an `adjust` modifier may change |
| modifier_kinds | multi_pick | subset of adjust, stat_bonus, requirement, effect | per type | closed list, see 2.3 |
| max_modifiers, max_tiers | int | 0-12, 0-4 | 8, 3 weapons | |
| engine_rules | multi_pick | closed list 1.3 | [] | |
| damage_type | enum | "", slash, pierce, blunt, magic, fire | "" | default of items |
| art | string | `ItemArt` key | "" | default icon (item `icon_id` overrides) |

### 1.2 Family list (verified against code and data; numbers in the matrix)

Weapons: sword, axe, mace, hammer, fist, dagger, bow, crossbow (no item yet), staff, wand, orb, tome. Shield: shield. Armor: plate, leather, cloth. Accessory: accessory. Consumable: consumable. Placeholders: material, quest. Total 20. Hands: sword {1h, 2h, off}, axe {1h, 2h}, mace {1h}, hammer {2h}, fist {1h, off}, dagger {1h, off}, bow/crossbow {2h}, staff {2h, 1h}, wand {1h}, orb/tome {off}, shield {off}. Hands present in data are facts; the rest (mace/wand 1H only, hammer 2H only, staff also 1H, crossbow copying bow, material and quest placeholders) were CONFIRMED by the user (DECISIONS "FORGE1 item answers").

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

Identity: `origin` (system / user, default user; see 2.5), `id` (snake_case, unique, fixed prefix not enforced), `name` (1-32), `description` (0-240, tokens allowed, 2.4), `icon_id` (picker of `ItemArt.UNIQUE_ICONS` or ""), `rarity`, `value` 0-99999, `weight` 0.0-50.0, `starter_stash`. Family: `family` (ref, required, picker). Build: `hand` (WEAPON, SHIELD: exactly one of family hands, default family default; strict typing 8.1), `slot` (ARMOR, ACCESSORY only: one of family slots; weapons/shields derive it), `scaling_stat` (WEAPON, default family default), `damage_type` (default family). Lists: `modifiers` (2.2), `tiers` (4). Special: `special`, `special_note`. Consumable payload fields unchanged (3). Everything derived (`item_type`, `weapon_type`, `slot` for weapons, `two_handed`, `off_hand_category`, base numbers) is NOT stored.

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

### 2.4 Description tokens and derived text (ADOPTED now, user answer)

Names stay authored (identity). Same mechanism as the skills pass (`SkillText`, skills spec section 5): the description is a template with `{token}` placeholders rendered with the item's RESOLVED values; the resolved flat row carries the rendered text in `description` so UI and engine read plain text; `ItemText` is a pure static class with the same API shape (`render`, `validate`, `tokens_for(item_type) -> Array[String]`, `format_*`) and reuses the shared formatting helpers (counted tokens with plural unit, `:n` bare-number suffix, percent x100) rather than duplicating them. Tokens: `{name} {power} {range} {burden} {mitigation} {block_cost} {hand} {family}`, `{req:STR}` (requirement value), `{bon:STR}` (stat bonus), `{tier1.value} {tier1.power}` (tier N). Validation, same classes as skills: unknown token = error listing the valid tokens for the type; token whose value does not exist for this item (`{req:STR}` with no STR requirement, `{tier2.power}` with one tier) = error naming the token; a literal number in the text that equals a resolved numeric field = soft warning suggesting the token. Auto-generated lines (not stored): modifier summary ("+2 STR, +2 CONS; Req STR 14"), tier text ("Sunder Guard: +4 Power" from `label` + bonus), hand label. Legacy descriptions without tokens are shown as is (the asset task may convert the ones that quote numbers). If the skills pass names the shared helpers differently, this spec follows that naming.

### 2.5 Origin (shared rules, owned by the skills spec)

Items and item families carry `origin` exactly as defined for skills (DECISIONS "FORGE1 class answers"): `system` rows are protected, `user` rows are free. Rules (referenced, not redefined): key fields of a system row are LOCKED in the Forge; DELETE is allowed only for user rows; a user row that a game test uses is PROMOTED to system (Forge action "Promote to system", delete then locked); every shipped item and family is seeded `origin: system`. ITEM key fields (locked when system): item `id`, `family`, `hand`, `scaling_stat`, `slot` (armor, accessory), every `adjust`/`requirement`/`stat_bonus`/`effect` modifier that the shipped data defines, and `tiers` (criteria and bonuses). Free on system items: `name`, `description`, `icon_id`, `rarity`, `value`, `weight`, `starter_stash`, `special_note`. Family key fields (locked when system): `id`, `item_type`, `hands`, `off_hand_category`, `engine_rules`, `adjustable`, `modifier_kinds`; free: name, description, `base`, `bands`, `max_*`, `art` (base edits stay possible because they only go through the impact panel, section 3). A system item cannot be removed, and a family with items cannot be removed at all. Validation: changing a locked field of a system row via `upsert` is refused with the field named; shipped-data seeds use a `seed` flag to bypass. Save format: `origin` stored per row in `items.json` v2 and `item_families.json`; absent = `system` for rows migrated from v1 (they are the shipped content).

## 3. Copy versus inherit (deliverable c) - DECIDED: Option I (inherit), numbers as deltas

User answer: items INHERIT from the family template with deltas (modifiers `adjust`), with the impact panel as the drift rail. Options kept for the record:

| Option | Gameplay | Cost | Risk |
| --- | --- | --- | --- |
| C. Copy: items keep full numbers, family seeds new items and validates bands (FORGE1_DESIGN I1) | none | no engine change; items.json unchanged in shape | family stops being "the template": changing a base never reaches existing items; hands/rules duplicated per item (the F1 bug class stays) |
| I. Inherit: structural facts (type, slot, hand set, weapon token, category, rules) and base numbers resolved from the family at load; items store hand/scaling/modifiers only | editing a family base moves every item that did not override | `ItemModifiers.resolve/collapse`, loader/saver, version 2 file, migration seed (31 items, mechanical), tests | silent budget drift when a base changes |
| H. Hybrid: structure inherited, numbers always explicit | none | as I | modifiers become absolute, loses the "template + deltas" the user asked for |

Chosen: I. Rails against drift (the impact panel is REQUIRED, not optional): (1) the Forge family editor shows an impact panel before Apply ("12 items change: wpn_x power 18 -> 20; class loadout totals warrior 23.0 -> 25.0") using `GearBudget`; (2) numbers of the family base are one batch edit through `ContentCatalog.upsert_many(item_families, ...)`; (3) the migration seed verifies `resolve(collapse(row)) == row` and equal `item_value` for all 31; (4) bands warn (soft, section 6) when a base edit pushes an item outside; (5) Apply on a family edit is a two-step action (preview panel, then confirm) and is refused only for hard errors (a resolved value outside the schema range). Migration cost: 31 items collapse mechanically (matrix section 3); shipped diff is items.json only. Rank "no cascade": an item resolves against its family once, families never inherit from families.

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

All extra tier features of this section (mitigation, stat_bonuses, effects, proficiency and unit-level criteria) are built NOW (user answer; iterate after the balance pass). Budget: `item_value` still ignores tiers (shipped numbers must not move); `GearBudget.tier_potential(item)` is a separate informational line in the preview ("tier potential: +N.N") counting every tier's bonuses at the existing weights x0.5; it never feeds class or loadout totals and never warns.

## 5. World named-item layer (deliverable e)

Unchanged in ITEM1: `base_item` (ref `base:items`) + `tuning` {power, mitigation, burden} + `stat_bonuses` + `grants`. `WorldFactory.named_item` reads `get_entry` (resolved flat row), applies the deltas and builds through `ContentFactory.item`, so a base item's modifiers and tiers carry over and base edits propagate (already true). Forge/World preview: `references_to("items", id)` and the family impact panel count named items. Guard: `WorldCatalog._item_errors` additionally requires the base to resolve (family exists). Later option (ITEM2, not now): named items get their own `modifiers` list using the same `ItemModifiers` kinds, so lore items can add an effect; it needs `world_schema` `_item_fields` and `named_item` changes (note: `named_item` does not cap `block_cost` or re-validate the family band; accepted).

## 6. Guard rails, form layout (deliverable f)

Rule: no raw JSON or comma text for list or dict values; every number is a spinbox with schema min/max/step (dynamic limits: delta so the resolved value stays in range; stat value 1-8); enums are dropdowns of the allowed set only; refs are filtered pickers; free text only for id, name, description, label, special_note. Save and Apply are disabled while problems exist; the preview header shows "Cannot save: N problems" with one line per reason (field and row named), offending controls outlined; warnings (bands, budget, literal-number tokens, special note) are yellow and never block. Band violations are SOFT warnings only until after the balance pass; then reconsider hard enforcement with proper scoring (a later task, not part of this spec). Origin: locked fields of system rows are disabled with a lock tooltip, Delete is hidden or disabled on system rows, "Promote to system" is offered on user rows (section 2.5). Reuses SKILL1-L controls (`multi_pick`, dynamic limits, problems bar); new here: `modifier_table` (kind dropdown swaps the parameter widgets, built from `ItemModifiers.KINDS` and `Passives.effect_specs()`), `tier_table`. Validation returns `{field, index, message}` rows (same pattern as `validate_skill_entry`); `validate_entry` maps them to strings.

Item form sections: 1 Identity (id, name, description + token help, icon, rarity, value, weight, starter_stash); 2 Family and build (family picker; read-only chips item type, gear token, slot; hand; scaling stat; slot for armor/accessory; damage type); 3 Base values (read-only grid with "family base + adjustments = result" per number and band marker); 4 Modifiers (table, add disabled at the cap with the reason); 5 Tier thresholds (table); 6 Special; 7 Consumable payload (consumables only); 8 Preview (resolved summary, per-modifier budget values, loadouts and named items that use it, problems, warnings). Family form: Identity, Type and hands, Template numbers and bands, Modifier policy, Engine rules (checkbox per rule with its help), impact panel, "Used by" (items).

Rule-derived validation (hard unless marked soft): family of an item exists; weapon has exactly one hand, in family hands; band checks use the item's hand (soft); scaling in family list; slot in family slots; `adjust` field/kind allowed, unique, resolved value within the schema range; band violation (soft); modifier counts; stat and requirement unique per stat; effects pass the item grammar; tier criteria, bonuses and counts per 4; removing a family with items is refused; changing a family `item_type` or `hands` revalidates all its items and is refused with the failing ids; consumable fields only for CONSUMABLE; a WEAPON with `focus` has power band max <= 10 (soft); `weapon_type` token of gear families equals the family id.

## 7. Consumables (deliverable d)

Defer. ITEM1 gives consumables a single `consumable` family (so every item has a family) with no modifiers; the payload fields (`consumable_type`, `heal`, `damage`, `buff_stat`, `buff_pct`, `duration`, `use_blocks`, `max_stack`) stay on the item with typed controls and existing ranges. Family-per-`consumable_type` (potion, throwable, brew) and modifier-style buffs wait for the consumables rework (open question 6 of FORGE1_DESIGN stays open). `material` and `quest` families exist only as placeholders.

## 8. Storage, migration, tests (deliverables g, h)

Storage: new kind `item_families` (20 rows) + `items.json` version 2 (family + modifiers + tiers). `ContentCatalog` gets a `VERSIONS` map shared with SKILL1-L (items 2, item_families 1); version-1 flat files still load (collapse on load, legacy fixture rows keep working). The seed `data/seeds/item1_families.gd` (idempotent) writes the 20 families, then re-saves items through `collapse`. Acceptance: validate_all empty; 31 items; for every item the resolved flat row equals the pre-migration row on all engine keys except the 3 intended slot changes (F1); `item_value` identical; every class and unit loadout total identical; arena (202), runner, world, progression, save suites pass unchanged.

New suites (one `results.append` each): `item_families_test_suite` (record validation, invariants, rule list equals the code constants, remove/retype refusals), `item_modifiers_test_suite` (round trip over every shipped item, sandbox edits: family base change moves only inheriting items, kind/cap/uniqueness errors, budget sum equals `item_value`, legacy flat upsert), `item_tiers_test_suite` (each criterion and bonus: stat, proficiency, unit level; power, mitigation, stat_bonuses, effects; no circularity; ordering; byte-identical when no tiers), `item_form_model_test_suite` (headless: sections, disabled-Save reasons, dynamic limits, token resolution). Updates to existing suites: matrix section 4. Rule: derive ids/counts from the catalog, never hard-code.

### 8.1 Strict hand typing (user answer 2026-10-16; supersedes the dual-wield guard as the main rule)

Rule: every weapon item has EXACTLY ONE `hand` (`main_1h`, `main_2h`, `off`; required, no default for weapons beyond the family default). Slot follows the hand and is never chosen: `main_1h` and `main_2h` fit MAIN_HAND only, `off` fits OFF_HAND only (shields are `off`). Why: power is budgeted per hand (1.1 `base`/`bands` per hand); moving a weapon between hands would make off-hand weapons too weak as mains, or too strong as mains. Dagger, shortsword, knuckles, tome, orb and shields stay legal off-hand items (hand `off`); stiletto and other main_1h daggers are different items. Plain `main_1h` in the off hand stays blocked.

Flag `Equipment.allow_plain_dual_wield: bool` (code constant, `static var` in `equipment.gd`, DEFAULT `false`, not data, not saved; tests flip it and restore it). With strict typing it is mostly redundant. It gates ONE thing only: whether a `main_1h` item may ALSO be equipped in OFF_HAND (`can_equip_in_slot`, the stash OFF_HAND filter, class/unit loadout validation). It never lets an `off` item into MAIN_HAND, a `main_2h` item anywhere but MAIN_HAND, or changes shields and focus rules. Kept only as a future relaxation; such an item would be budgeted by its main_1h numbers, so enabling it needs its own design pass.

Single gate `Equipment.can_equip_in_slot` (stash filters, `session.equip_item`, equip modal, dojo; builder greps `.equip(` callers for any bypass):

| Case | Flag false (default) | Flag true |
| --- | --- | --- |
| `main_1h` or `main_2h` to MAIN_HAND | allowed | allowed |
| `off` item (dagger, knuckles, shortsword, tome, orb, shields) to OFF_HAND | allowed | allowed |
| `main_1h` to OFF_HAND | BLOCKED | allowed |
| `main_2h` to OFF_HAND; `off` item to MAIN_HAND | blocked | blocked |

Not removed: the dual-strike combo (`DojoSession` ~250), the `Inventory` off-hand switch logic, offhand jab. Gear already worn is kept: existing worn plain-1H-in-off-hand items stay equipped (loads do not call the gate; builder confirms and records it); only NEW equips are gated. Acceptance: no shipped class, unit, preset or save fixture holds a `main_1h` item in OFF_HAND (scan with the flag false; hits are reported, not fixed silently). Skills choose a hand through `weapon_hand` (FORGE1_SKILLS 4.1).

Checks that currently rely on dual wield and their rule-derived update (a code change in the logic task, listed here so none is missed):

| Where | Relies on | Update |
| --- | --- | --- |
| Dojo-28 (`scenes/dojo/dojo_assertions.gd` ~495-517) | `broadsword.can_equip_in_slot(off)` is true ("1H may dual wield") | Default flag: expect `not broadsword.can_equip_in_slot(off)`; add a second pass with `allow_plain_dual_wield = true` expecting true, then reset; shortsword/buckler/claymore expectations unchanged; rename the label to "off-hand weapons/shields off-hand only, plain 1H blocked from the off hand (guard)" |
| Dojo-21c (~323-343) and the assertion at ~423 | preset `dual_wield` = broadsword main + Iron Shortsword | NO change: the shortsword's hand is `off`, so it stays allowed; Dual Strike needs a 1H melee main plus an offensive off-hand weapon. Re-run to confirm |
| Presets `dual_wield` ("Dual Wield"), `duelist` ("Blade & Dagger"), `archer` (`scripts/dojo/dojo_loadouts.gd`) | broadsword + shortsword / Shadow Dagger, bow + dagger | NO change (both off items have hand `off`) |
| `foundation_test_suite` ~255-284 (hand table, `main_1h` needs `in_main` only) | none of OFF_HAND for main_1h | NO change; add: flag false -> `broadsword.can_equip_in_slot(off)` false, flag true -> true |
| `focus_gear_test_suite` (21 OFF_HAND refs), `gear_equip`, `character_specs` | only orb, tome, `off` fixtures; `character_specs` and `gear_equip` have no dual-wield reference (grep) | NO change; builder re-greps fixtures that build a synthetic plain 1H into OFF_HAND and, per rule, sets the flag in that fixture instead of changing the expectation |
| New in `item_families_test_suite` | - | slot derivation with the flag both ways; stash `OFF_HAND` filter hides plain 1H by default |

## 9. Milestone plan and ordering (deliverable i, aligned with the skills and class passes: skills FIRST)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| ITEM1-L commit 1 | logic | `ItemFamilies`, `ItemModifiers`, `ItemText` (on the shared token helpers), schema (`item_families` kind, items v2 fields incl. `origin`), loader/saver/VERSIONS, family/item/tier validation, origin locks, `GearBudget.modifier_value`, dual-wield guard (8.1) and its test updates, new tests 1-2. Schema first: asset task released after this commit | SKILL1-L commit 1 (VERSIONS, origin, token helpers) |
| ITEM1-L commit 2 | logic | Tier engine: `mitigation`, `stat_bonuses`, `effects` bonuses, `proficiency_at_least`, `unit_level_at_least`, evaluation order, `GearBudget.tier_potential`, `Passives.effect_specs()` (shared with passives tab), test 3 | commit 1 |
| ITEM1-L commit 3 | logic | Forge: family form, item sections, modifier and tier tables, impact panel, lock/promote/delete UI, problems bar, filter chips (family, hand, rarity, special, has tiers, has effects, origin, used by class/unit/named item), model test 4 | commit 1, SKILL1-L commit 2 |
| ITEM1-A | asset | Run the migration seed (all rows `origin: system`), create the NEW families, optionally convert descriptions that quote numbers (claymore, warhammer "22 STR") to tokens | commit 1 |

Hub files shared with the other passes: `content_schema.gd`, `content_catalog.gd`, `content_factory.gd`, `forge_form.gd`, `content_forge.gd`, `forge_preview.gd`, `passives.gd` (skills edits `skill_def`, classes edit `class_archetype`, `class_forge_model`). Hub conflicts, so sequential. Order across the three passes: SKILL1-L (commits 1, 2; defines origin, tokens, VERSIONS) -> ITEM1-L commit 1 + ITEM1-A -> CLASS1-L (its loadout pickers group by family, honour the dual-wield guard, and the accessory same-slot flag) -> ITEM1-L commits 2 and 3. Commit 2 touches only `equipment.gd`, `combatant.gd`, `gear_budget.gd`, `passives.gd` (no content hub) so it may run in parallel with CLASS1-L. Safe anytime: balance sims, M4 quest runtime. A post-balance follow-up (not scheduled): decide hard band enforcement.

## 10. Out of scope

Engine kind or rule additions beyond the 4 existing rules; new gameplay values or rebalancing (migration is value-neutral); named-item modifiers (ITEM2); consumable rework; making families inherit families; art; changing proficiency, class or skill schema; reading gear tokens from families inside `Proficiency` (constants stay, invariant test guards drift); fleshing out dual wield, the `weapon_hand: both` engine work (skills 4.1); hard band enforcement and tier scoring (post balance pass); any implementation.

## 11. Decided (all earlier questions resolved; DECISIONS "FORGE1 item answers", "Strict hand typing")

1. Hands and NEW rows as proposed; items inherit with deltas and the impact panel; band violations soft until after the balance pass; extra tier features now with an informational 50% line; description tokens now.
2. Strict hand typing (8.1): one hand per weapon, slot follows the hand, bands and base per hand; `off` items (dagger, shortsword, knuckles, tome, orb, shields) stay legal off-hand items; Dual Strike, presets `dual_wield`, `duelist` and Dojo-21c unchanged.
3. The dual-wield flag is a code constant (not a setting); it only gates `main_1h` into OFF_HAND; existing worn plain-1H-in-off-hand items are kept.
