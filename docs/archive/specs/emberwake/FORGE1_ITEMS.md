# FORGE1_ITEMS: Item tab of the Content Forge (item family template, base item = family + modifiers, tier thresholds)

Status: approved (user answers 2026-10-16, see 11; plans approved 2026-10-16; slimmed and refreshed 2026-10-17 against SKILL1, BAL2-A, CLN-9, decided content unchanged). Supersedes the items part of the archived FORGE1_DESIGN. Shared Forge rules (former FORGE1_SKILLS 4.1 / 11 / 12): `games/emberwake/docs/specs/FORGE_SHARED.md`, same numbers.
Depends on: SKILL1-L (done), CLN-9 (validator split, before ITEM1)
Lane: logic (ITEM1-L, schema first, three commits), then one small asset task (ITEM1-A)
Touches (when approved): `scripts/content/content_schema.gd` (`_item_fields`, new `_item_family_fields`; or their per-kind file if CLN-9's optional schema step lands); item validators per CLN-9 split in `scripts/content/rules/item_rules.gd` (name planned in `docs/specs/CLEANUP.md` CLN-9, not final: `ItemValidator.item_errors`, family and tier validation); `content_catalog.gd` only for `VERSIONS`, load/save of the new kind and dispatch; `content_factory.gd` (`item`, `item_to_entry`); new `scripts/core/specs/item_families.gd`, `item_modifiers.gd`, `item_text.gd` (pure); `scripts/core/inventory/equipment.gd` (tiers, dual-wield flag); `scripts/core/combat/combatant.gd` (`sync_from_stats_and_gear`, `rebuild_gear_effects`); `scripts/core/specs/gear_budget.gd`, `passives.gd` (`effect_specs()`); `scripts/ui/forge/*`; new `data/catalog/item_families.json`; `data/catalog/items.json` (v2); `data/seeds/item1_families.gd`; tests per matrix sections 4-5.
Companions: `FORGE1_ITEMS_MATRIX.md` (family rows, 31-item migration, test updates incl. the 8.1 table, form layout); `FORGE1_ITEMS_NOTES.md` (findings, option record, rationale, HISTORY check: nothing here re-proposes a deprecated idea).

## Goal

Split items into ITEM FAMILY (template: item type, hands, slots, base numbers, soft bands, allowed modifier kinds, closed engine rules) and BASE ITEM (identity + family reference + a generic list of modifiers + a tier-threshold list). Same pattern as skills (family = template, item derived by modifiers against the family, no cascade, derived text). Families in a new `item_families.json`; items on disk as family + modifiers; flat resolved rows in memory so engine, budget, world layer and most tests see today's shape.

## Existing infrastructure (use it, do not re-specify)

| Piece | Where | ITEM1 does |
| --- | --- | --- |
| `origin` on every kind | `ContentSchema.origin_field()` appended by `fields()`; storage omits the default `system` (FORGE_SHARED 12.1) | nothing; the families kind gets it too |
| Key-field lock | schema flag `key_field: true` + `ContentCatalog.key_field_errors` (in `upsert` / `upsert_many`) | mark the 2.5 key fields |
| Seed scope | `ContentCatalog.begin_system_edit(allow_remove)` / `end_system_edit()` (12.4) | migration seed runs inside it |
| Delete, promote, demote, SAFE DELETE | `remove`, `promote`, `demote`, `tools/safe_delete.gd` (12.3, 12.5; scan already lists item `family`) | nothing new |
| Version map | `ContentCatalog.VERSIONS` (today `{"skills": 2}`) | add `items: 2`, `item_families: 1` |
| Token helpers | `SkillText` (`parse`, `escape`, `plural`, `format_percent`, `format_decimal`, `token_value` with `:n`) | `ItemText` reuses them (or extracts a shared helper) |
| Typed controls, problems bar | SKILL1-L Forge (`multi_pick`, `table`, `dict_pick`, dynamic limits; FORGE_SHARED appendix G) | reuse; add `modifier_table`, `tier_table` |

## 0. What the code does today

Moved to NOTES section 1. Data after BAL2-A (kite 6, plate 8, unarmed brawler): matrix sections 2-3.

## 1. Item families (deliverable a)

### 1.1 Family record (new kind `item_families`, file `data/catalog/item_families.json`, version 1)

| Field | Type | Allowed / range | Default | Rule |
| --- | --- | --- | --- | --- |
| id | id | snake_case, unique; a GEAR family (WEAPON, SHIELD, ARMOR) equals a `Proficiency.gear_types()` token and every token has a family (invariant, like skills REQUIRED_IDS) | "" | read-only after create when items use it |
| name, description | string 1-32, text 0-240 | | "New family" | |
| item_type | enum | WEAPON, SHIELD, ARMOR, ACCESSORY, CONSUMABLE, MATERIAL, QUEST | WEAPON | locked while items use it |
| hands | multi_pick | ONE CATEGORY: a non-empty subset of {main_1h, main_2h} (main, damage weapons) or exactly [off] (support items); mixing is an error. WEAPON non-empty; SHIELD [off]; other types empty | [main_1h] | item `hand` must be one of these |
| default_hand | enum | one of `hands` | main_1h | |
| off_hand_category | enum | derived, not authored: the family id for shield, orb, tome; "" for main families; schema options shrink to "", shield, orb, tome | "" | on the resolved row (`ItemArt`, `Equipment`) |
| slots | multi_pick | ARMOR: subset of HEAD, BODY; ACCESSORY: ACCESSORY_1, ACCESSORY_2; weapons/shields derive it from the hand, not stored (main_1h, main_2h -> MAIN_HAND; off -> OFF_HAND; main_1h also OFF_HAND only while `Equipment.allow_plain_dual_wield` is true, 8.1) | [] | item `slot` must be in it |
| origin | enum | system, user (2.5) | user | |
| scaling_stats | multi_pick | WEAPON: non-empty subset of STR, DEX, INT (never KNOW) | [STR] | item `scaling_stat` in it; first = default |
| base | per hand (key = each hand; non-weapons key `none`; two keys only for sword, axe, staff), five ints | power 0-200, range 1-6 (WEAPON), block_cost 1-6, burden 0-10, mitigation 0-100 (SHIELD, ARMOR); fields invalid for the type hidden, stored 0/1 | per matrix | an item resolves against `base[item.hand]` |
| bands | per hand, per field {min, max}; plus `bonus_value` {min, max} = GearBudget value of the non-power modifiers (stat_bonus + effect) | min <= max, inside the schema range, `base[hand]` inside | matrix rule 2; `bonus_value` main 0-4, off 0-6 (seed defaults) | outside = SOFT warning. Only relation warning (soft): `main_2h` base power below `main_1h` in a family with both |
| adjustable | multi_pick | subset of power, range, block_cost, burden, mitigation valid for the type | all valid | fields an `adjust` may change |
| modifier_kinds | multi_pick | subset of adjust, stat_bonus, requirement, effect | per type | 2.3 |
| max_modifiers, max_tiers | int | 0-12, 0-4 | 8, 3 (weapons) | |
| engine_rules | multi_pick | closed list 1.3 | [] | |
| damage_type | enum | "", slash, pierce, blunt, magic, fire | "" | default of items |
| art | string | `ItemArt` key | "" | default icon; item `icon_id` overrides (`ItemArt.icon_key` must honour it first: the retyped shortsword, knuckles, dagger keep their icons that way) |

### 1.2 Family list (numbers in the matrix)

20 families. Weapons: sword, axe, mace, hammer, fist, dagger, bow, crossbow (no item yet), staff, wand, orb, tome; shield; armor plate, leather, cloth; accessory; consumable; placeholders material, quest. Main (damage): sword {1h, 2h}, axe {1h, 2h}, mace {1h}, hammer {2h}, fist {1h}, dagger {1h}, bow/crossbow {2h}, staff {2h, 1h}, wand {1h}. Off (support): shield, orb, tome, each {off}. Rationale: NOTES section 2.

### 1.3 Engine rules (closed list, code-owned)

| Rule | Meaning | Families |
| --- | --- | --- |
| focus | off-hand only, never 2H, a 2H main refuses it, equipping a 2H main stows it | orb, tome |
| always_ranged | ranged even at range 1 | bow, crossbow |
| shield | counts as a shield (blocks 2H pairing and dual wield) | shield |
| spell_source | main-hand type that opens spell skills | staff, wand |

Code constants today: NOTES section 1 (`Equipment.FOCUS_TYPES`, `is_ranged_weapon`, `is_shield`, `SkillDef` spell gate).

Commit 1 keeps the code constants; `item_families_test_suite` asserts the rule lists equal them. A new rule is a code change; validation rejects unknown names. Consistency: `focus` requires hands == [off] and WEAPON; `shield` requires SHIELD; `always_ranged`, `spell_source` require WEAPON.

## 2. Base item (deliverable b)

### 2.1 Stored record (items.json version 2)

Identity: `origin` (2.5), `id` (snake_case, unique), `name` (1-32), `description` (0-240, tokens, 2.4), `icon_id` (`ItemArt.UNIQUE_ICONS` or ""), `rarity`, `value` 0-99999, `weight` 0.0-50.0, `starter_stash`. Family: `family` (ref, required). Build: `hand` (WEAPON, SHIELD: one of family hands, default family default), `slot` (ARMOR, ACCESSORY only), `scaling_stat` (WEAPON, default family default), `damage_type` (default family). Lists: `modifiers` (2.3), `tiers` (4). Special: `special`, `special_note`. Consumable payload unchanged (7). Derived, NOT stored: `item_type`, `weapon_type`, weapon `slot`, `two_handed`, `off_hand_category`, base numbers.

### 2.2 Resolved flat row (in memory, what every consumer reads)

`entries/get_entry/validate_entry` keep today's flat keys: `item_type, slot, weapon_type (= family id for gear families, else ""), two_handed (= hand main_2h), off_hand_category, base_power, base_range, scaling_stat, block_cost, burden, mitigation, required_stats, stat_bonuses, effects, threshold_tiers`, plus `family, hand, modifiers, tiers`. `ItemModifiers.resolve(entry, family)` (pure): numbers = family.base + sum of `adjust` deltas; `stat_bonus` -> `stat_bonuses`; `requirement` -> `required_stats`; `effect` -> `effects`; `tiers` -> `threshold_tiers` (legacy shape, 4.3). Inverse `collapse(flat_row, family)` for the saver and legacy fixtures: a row without `family` derives it from `weapon_type` (else `accessory`/`consumable` by item_type) and its modifiers from the differences. `upsert` accepts either shape.

### 2.3 Modifier list (generic)

`modifiers` = ordered list of `{kind, ...params}`. The engine reads only the resolved sums; a NEW KIND = descriptor row + resolve branch (code). Kinds: closed list `ItemModifiers.KINDS` (single source for validator, widgets, budget, text).

| Kind | Params | Range | Unique per | Resolves to | GearBudget value |
| --- | --- | --- | --- | --- | --- |
| adjust | field (power, range, block_cost, burden, mitigation), delta | -100..100; resolved value in the schema range (error), in the band (warning) | field | the number | power x1.0, mitigation x1.5, burden x-2.0, block_cost x-3.0 per step above 1, range 0 |
| stat_bonus | stat (STR DEX INT CONS PERC KNOW), value | 1-8, never negative | stat | `stat_bonuses[stat]` | x2.0 |
| requirement | stat, value | 1-30 | stat | `required_stats[stat]` | 0 |
| effect | passive-grammar dict, item types only | `Passives.ITEM_TYPES` (skill_potency, status_potency 1..25, spell_reach 1..2, resist); `Passives.validate_effects(..., items_only = true)` | identical effect | `effects` entry | skill_potency x20 per 1.0, status_potency x0.2, spell_reach x3 |

Limits: count <= `family.max_modifiers` (cap 12), kinds in `family.modifier_kinds`, `adjust.field` in `family.adjustable`. `GearBudget.modifier_value(mod)` is pure; per-modifier values sum to `item_value(resolved row)` (acceptance). `special` needs `special_note` (soft, exists).

### 2.4 Description tokens and derived text (adopted now)

Names stay authored. As `SkillText`: the description is a `{token}` template rendered with the RESOLVED values into the resolved row's `description`. `ItemText` (pure static) has `SkillText`'s API shape (`render`, `validate`, `warnings`, `tokens_for(item_type) -> Array[String]`) on the shared helpers. Tokens: `{name} {power} {range} {burden} {mitigation} {block_cost} {hand} {family}`, `{req:STR}`, `{bon:STR}`, `{tier1.value} {tier1.power}`. Validation: unknown token = error listing the valid tokens for the type; a token with no value for this item (`{req:STR}` without STR requirement, `{tier2.power}` with one tier) = error naming it; a literal number equal to a resolved numeric field = soft warning suggesting the token. Auto lines (not stored): modifier summary ("+2 STR, +2 CONS; Req STR 14"), tier text ("Sunder Guard: +4 Power"), hand label. Legacy descriptions without tokens show as is.

### 2.5 Origin (rules in FORGE_SHARED 12; item lists only here)

Items and families follow FORGE_SHARED 12 (lock, delete, promote/demote, SAFE DELETE, seed scope). Every shipped item and family is `system` (absent `origin` = system, 12.1).

- ITEM key fields (`key_field: true`): `id`, `family`, `hand`, `scaling_stat`, `slot` (armor, accessory), every `adjust`/`requirement`/`stat_bonus`/`effect` modifier the shipped data defines, `tiers` (criteria and bonuses); see question 1. Free: `name`, `description`, `icon_id`, `rarity`, `value`, `weight`, `starter_stash`, `special_note`.
- FAMILY key fields: `id`, `item_type`, `hands`, `off_hand_category`, `engine_rules`, `adjustable`, `modifier_kinds`. Free: name, description, `base`, `bands`, `max_*`, `art` (base edits go through the impact panel, 3).
- A system item cannot be removed; a family with items cannot be removed at all; a locked-field change via `upsert` is refused with the field named (`key_field_errors`).

## 3. Copy versus inherit (deliverable c): DECIDED, inherit with deltas

Items INHERIT from the family with `adjust` deltas (option record: NOTES 3). Required rails: (1) the family editor shows an impact panel before Apply ("12 items change: wpn_x power 18 -> 20; class loadout totals warrior 27.0 -> 29.0") via `GearBudget`; (2) base edits are one batch `ContentCatalog.upsert_many("item_families", ...)`; (3) the migration seed verifies `resolve(collapse(row)) == row` and equal `item_value` for all 31; (4) bands warn (soft) when a base edit pushes an item outside; (5) Apply is two-step (preview, confirm), refused only for hard errors (resolved value outside the schema range). No cascade: items resolve against their family once; families never inherit.

## 4. Tier thresholds (deliverable b)

### 4.1 Tier record (`tiers`, ordered, up to `family.max_tiers`)

`{label, when, bonus, desc}`: `label` 0-24 chars; `desc` 0-120 (empty = auto: label + ": " + bonus text). Unique `when` per item; ascending by value within a stat; at least one non-zero bonus.

### 4.2 Closed lists

| `when.type` | Params | Range | Evaluated by |
| --- | --- | --- | --- |
| stat_at_least (exists) | stat, value | 1-40 | `get_effective_stat(stat, {weapon}) + proficiency.level x LEVEL_DISCOUNT` >= value (static gear bonuses in, tier bonuses out) |
| proficiency_at_least | gear (default own family; any gear token), level | 1-3 (`Proficiency.MAX_LEVEL`) | `proficiency.level_of(gear)` |
| unit_level_at_least | value | 1-10 | `CharacterStats.level` |

| `bonus` key | Allowed on | Range | Engine |
| --- | --- | --- | --- |
| power | WEAPON | 0-50 | exists (`get_unlocked_threshold_power_boost`) |
| mitigation | ARMOR, SHIELD | 0-20 | ADD: effective mitigation = base + active tiers where armor rating sums `eq.mitigation` (builder greps) |
| stat_bonuses | any gear | 1-5 per stat, <= 3 stats | ADD: `sync_from_stats_and_gear` adds modifier source `gear_tier` after static gear bonuses |
| effects | any gear | item grammar, <= 2 | ADD: `rebuild_gear_effects` applies active tier effects with source `gear:<id>:tier<n>` |

### 4.3 Evaluation order, legacy rows, budget

Single pass, no circularity: static gear modifiers -> evaluate tiers on stats WITHOUT tier modifiers -> add tier stat modifiers -> rebuild gear effects incl. tier effects (effects are rebuilt first today). Acceptance: items without tiers or with power-only tiers give byte-identical stats and effects. Legacy `{stat, threshold, bonus_power, desc}` rows load as `stat_at_least` + `power`; resolved `threshold_tiers` keeps those keys, so `Equipment`, `DamageCalculator`, `ui_theme` and `character_specs` are untouched; extended tiers add `when`/`bonus`. All tier features are built NOW (user answer). `item_value` ignores tiers; `GearBudget.tier_potential(item)` is an informational preview line (all tier bonuses at the weights x0.5), never in class or loadout totals, never warns.

## 5. World named-item layer (deliverable e)

Unchanged: `base_item` + `tuning` {power, mitigation, burden} + `stat_bonuses` + `grants`; `WorldFactory.named_item` reads `get_entry` (resolved row), so modifiers and tiers carry over and base edits propagate. `references_to("items", id)` and the impact panel count named items. Guard: `WorldCatalog._item_errors` also requires the base to resolve (family exists). Later ITEM2: NOTES 5.

## 6. Guard rails (deliverable f; form layout in matrix section 6)

No raw JSON or comma text; numbers are spinboxes with schema min/max/step (dynamic: a delta keeps the resolved value in range; stat value 1-8); enums are dropdowns; refs are filtered pickers; free text only for id, name, description, label, special_note. Save and Apply are disabled while problems exist ("Cannot save: N problems", one line per reason, controls outlined); warnings (bands, budget, literal-number tokens, special note) are yellow, never block. Bands stay SOFT until after the balance pass. System rows: locked fields disabled with a lock tooltip, Delete disabled; user rows offer "Promote to system". `modifier_table` swaps parameter widgets by kind (from `ItemModifiers.KINDS`, `Passives.effect_specs()`). Validation returns `{field, index, message}` rows (as `validate_skill_entry`); `validate_entry` maps them to strings.

Rule-derived validation (hard unless soft): item family exists; weapon has exactly one hand, in family hands; bands per the item's hand (soft); scaling in family list; slot in family slots; `adjust` field/kind allowed, unique, resolved value in the schema range; modifier counts; stat bonus and requirement unique per stat; effects pass the item grammar; tier criteria, bonuses, counts per 4; removing a family with items refused; changing a family `item_type` or `hands` revalidates its items and is refused with the failing ids; consumable fields only on CONSUMABLE; WEAPON with `focus` has power band max <= 10 (soft); gear-family `weapon_type` equals the family id.

## 7. Consumables (deliverable d)

Deferred. One `consumable` family, no modifiers; payload fields (`consumable_type`, `heal`, `damage`, `buff_stat`, `buff_pct`, `duration`, `use_blocks`, `max_stack`) stay on the item with typed controls and existing ranges. Per-type families and modifier buffs wait for the consumables rework (FORGE1_DESIGN open question 6 stays open). `material` and `quest` are placeholders.

## 8. Storage, migration, tests (deliverables g, h)

New kind `item_families` (20 rows) + `items.json` v2; `VERSIONS` gains `items: 2`, `item_families: 1`; v1 flat files still load (collapse on load). Seed `data/seeds/item1_families.gd` (idempotent, inside the system-edit scope) writes the families, then re-saves items through `collapse`.

Acceptance: `validate_all` empty; 31 items; every resolved row equals the pre-migration row on all engine keys except (a) the retype of wpn_shortsword, wpn_knuckles, wpn_dagger (`off_hand_category` "", `hand()` main_1h, stored slot MAIN_HAND unchanged) and (b) `off_hand_category` of `shd_kite`, `shd_buckler` becoming "shield" (derived; stored "" today; engine-neutral, `is_shield` is true by `item_type`, NOTES 1); `item_value` identical for all 31; every class and unit loadout total identical; every `suite_registry` suite passes unchanged.

New suites 1-4 (`item_families`, `item_modifiers`, `item_tiers`, `item_form_model`; contents in matrix section 4) and existing-suite updates (matrix sections 4, 5). Derive ids and counts from the catalog.

### 8.1 Strict hand typing (user answer 2026-10-16)

Every weapon item has EXACTLY ONE `hand` (`main_1h`, `main_2h`, `off`); every family has ONE category: main (damage) or off (support: shield, orb, tome). Slot follows the hand: `main_1h`, `main_2h` fit MAIN_HAND only, `off` OFF_HAND only. Both daggers, shortsword (sword main_1h) and knuckles (fist main_1h) are main-hand damage weapons; mixing categories in a family is an error (1.1). Why: NOTES 2.

Flag `Equipment.allow_plain_dual_wield: bool` (code `static var`, default `false`, not data, not saved; tests flip and restore it): the legacy dojo switch. It lets a `main_1h` item ALSO into OFF_HAND (`can_equip_in_slot`, stash OFF_HAND filter, class/unit loadout validation), so dual wield (Dual Strike, off-hand attacks, `weapon_hand: both`, Switch Weapon) works as before round 4; nothing else (table). The dojo sets it for its session and restores it on exit. Such a weapon is budgeted by its main_1h numbers.

Single gate `Equipment.can_equip_in_slot` (stash filters, `session.equip_item`, equip modal, dojo; builder greps `.equip(` for bypasses):

| Case | Flag false (default) | Flag true |
| --- | --- | --- |
| `main_1h` or `main_2h` to MAIN_HAND | allowed | allowed |
| `off` item (tome, orb, shields) to OFF_HAND | allowed | allowed |
| `main_1h` to OFF_HAND | BLOCKED | allowed |
| `main_2h` to OFF_HAND; `off` item to MAIN_HAND | blocked | blocked |

Skills: a `weapon_hand: off` skill (offhand_jab) uses an off-hand weapon, else bare hands, so bare hands (STR+DEX; trained fists only on an empty MAIN hand, BAL2-L Rule 5) in every strict loadout; `both` needs the flag (FORGE_SHARED 4.1). Dual Strike, the `Inventory` off-hand switch and offhand jab stay. Worn gear is kept (loads do not call the gate; builder confirms); only NEW equips are gated.

Acceptance: no shipped class, unit or save fixture holds a `main_1h` item in OFF_HAND (scan with the flag false; report hits, do not fix silently). Shipped off hands (2026-10-17, after BAL2-A): warrior `shd_kite`, priest `wpn_tome`, mage `wpn_orb`; no class uses the three retyped items; brawler main hand empty. Every test check that relies on the old off-hand weapons, with its update: matrix section 5 (the logic task applies all of it).

## 9. Milestone plan and ordering (deliverable i)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| ITEM1-L commit 1 | logic | `ItemFamilies`, `ItemModifiers`, `ItemText`; schema (`item_families` kind, items v2, `key_field` marks); loader/saver, `VERSIONS`; family/item/tier validation (item rules file); `GearBudget.modifier_value`; dual-wield guard (8.1) and matrix-5 test updates; new suites 1-2. Releases the asset task | SKILL1-L (done), CLN-9 |
| ITEM1-L commit 2 | logic | Tier engine (mitigation, stat_bonuses, effects bonuses; proficiency and unit-level criteria; evaluation order), `GearBudget.tier_potential`, `Passives.effect_specs()`, suite 3 | commit 1 |
| ITEM1-L commit 3 | logic | Forge: family form, item sections, modifier and tier tables, impact panel, lock/promote/delete UI, problems bar, filter chips (matrix 6), suite 4 | commit 1 |
| ITEM1-A | asset | Run the migration seed (all rows system), create the NEW families; optionally convert number-quoting descriptions to tokens: claymore, warhammer ("22 STR"), `shd_buckler` ("4 mitigation"), and `shd_kite` ("+8"), `arm_plate` ("10 mitigation"), which quote pre-BAL2-A numbers (`{mitigation}` fixes them) | commit 1 |

Order: ITEM1-L commit 1 + ITEM1-A -> CLASS1-L (pickers group by family, honour the dual-wield guard and accessory flag) -> ITEM1-L commits 2, 3. Commit 2 (only `equipment`, `combatant`, `gear_budget`, `passives`) may run beside CLASS1-L. Shared hubs with CLASS1: `content_schema`, `content_factory`, `scripts/ui/forge/*`; validators no longer collide after CLN-9 (planner decides overlap). Safe anytime: balance sims, M4 quest runtime. Not scheduled: hard band enforcement after the balance pass.

## 10. Out of scope

Engine kinds or rules beyond the 4; new values or rebalancing (migration is value-neutral); named-item modifiers (ITEM2); consumable rework; families inheriting families; art; proficiency, class or skill schema; reading gear tokens from families inside `Proficiency`; fleshing out dual wield or `weapon_hand: both` (FORGE_SHARED 4.1); hard bands and tier scoring; any implementation.

## 11. Decided

All earlier questions are resolved: `games/emberwake/docs/decisions/items-gear.md` rows "FORGE1 item answers", "Strict hand typing and no class skill loadout", "FORGE1 round 5 answers", "BAL2 gear budget"; content-forge.md "FORGE1 round 4 answers".

## 12. Open questions (2026-10-17 refresh)

1. Lock granularity: `key_field_errors` compares whole fields, so `modifiers` and `tiers` as key fields lock the whole list on system items (no added modifier either), while 2.5 locks only the shipped modifiers. (a) whole-list lock, no extra code (recommended); (b) a per-entry check in the item rules file allowing appended modifiers.
2. Flag home: 8.1 puts `allow_plain_dual_wield` in `equipment.gd`; FORGE1_CLASSES 3.6 wants it beside `ALLOW_DUPLICATE_ACCESSORY` in `stat_rules.gd`. Recommended: `equipment.gd` (a test-flipped `static var`), with a pointer comment in `stat_rules.gd`.
