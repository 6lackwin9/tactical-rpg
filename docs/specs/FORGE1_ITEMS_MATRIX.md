# FORGE1_ITEMS_MATRIX: family seed table, migration of the 31 items, test list

Companion of `docs/specs/FORGE1_ITEMS.md` (read that first). Numbers below are DERIVED from the shipped `data/catalog/items.json` (31 entries) by the rules in section 1; the asset builder recomputes them from the file with `ContentCatalog.entries("items")` and must not trust this table over the data. Rows marked `NEW` have no shipped item; the user confirmed them (crossbow, material, quest) together with the proposed hands.

## 1. Derivation rules (mechanical, so the asset task needs no judgement)

1. `base.<field>` of a family = the LOWER median of that field over its shipped items (one item = that item, so its modifiers shrink to none).
2. `band.<field>` = [floor(0.75 x shipped min), ceil(1.25 x shipped max)], clamped to the schema range; zero-only fields get [0, 10] (power) or [0, 0] (others). Bands are SOFT (warning) until after the balance pass (user answer); every family and item row below is seeded `origin: system`.
3. `hands` = hands present among shipped items, plus hands the spec lists as `NEW`. `scaling_stats` likewise, default = the most common.
4. A modifier `adjust` is written for every numeric field where the item differs from the family base (delta may be negative); `stat_bonus`, `requirement`, `effect` modifiers are copied one per stat / effect.
5. Round trip is the acceptance: `ItemModifiers.resolve(entry, family)` must equal the pre-migration flat row on every key the engine reads (`item_type, slot, weapon_type, base_power, base_range, two_handed, off_hand_category, scaling_stat, block_cost, burden, mitigation, required_stats, stat_bonuses, effects, threshold_tiers`), and `GearBudget.item_value` must be identical for all 31.

## 2. Family rows (20)

Columns: hands (default first) | scaling (default first) | base power/range/block/burden/mitigation | soft bands (power; other) | engine rules | shipped items. `slots` is derived from the hand for weapons and shields (see main spec 2.2) and listed explicitly only for armor and accessories.

| Family | item_type | Hands | Scaling | Base p/r/b/bd/m | Bands | Engine rules | Items |
| --- | --- | --- | --- | --- | --- | --- | --- |
| sword | WEAPON | main_1h, main_2h, off (cat `shortsword`) | STR | 18/1/1/0/- | p 10-30 | - | 3 |
| axe | WEAPON | main_1h, main_2h | STR | 16/1/1/0/- | p 12-34 | - | 2 |
| mace | WEAPON | main_1h | STR, INT | 13/1/1/0/- | p 9-22 | - | 2 |
| hammer | WEAPON | main_2h | STR | 34/1/1/3/- | p 25-43 | - | 1 |
| fist | WEAPON | main_1h, off (cat `knuckles`) | STR | 12/1/1/0/- | p 9-15 | - | 1 |
| dagger | WEAPON | main_1h, off (cat `dagger`) | DEX | 15/1/1/0/- | p 11-20 | - | 2 |
| bow | WEAPON | main_2h | DEX | 20/3/1/0/- | p 15-25, r 2-6 | always_ranged | 1 |
| crossbow (NEW) | WEAPON | main_2h | DEX | 20/3/1/0/- (copy of bow) | p 15-25, r 2-6 | always_ranged | 0 |
| staff | WEAPON | main_2h, main_1h | INT | 22/2/1/0/- | p 16-32, r 1-3 | spell_source | 2 |
| wand | WEAPON | main_1h | INT | 15/3/1/0/- | p 11-38, r 2-4 | spell_source | 2 |
| orb | WEAPON | off (cat `orb`) | INT | 0/1/1/0/- | p 0-10 | focus | 1 |
| tome | WEAPON | off (cat `tome`) | INT | 0/1/1/0/- | p 0-10 | focus | 1 |
| shield | SHIELD | off (cat `shield`) | - | -/-/1/2/8 | m 3-20, bd 0-4, b 1-3 | shield | 3 |
| plate | ARMOR | slots HEAD, BODY | - | -/-/1/3/10 | m 7-13, bd 2-4 | - | 1 |
| leather | ARMOR | slots HEAD, BODY | - | -/-/1/1/4 | m 3-5, bd 0-2 | - | 1 |
| cloth | ARMOR | slots HEAD, BODY | - | -/-/1/0/2 | m 1-4, bd 0-1 | - | 3 |
| accessory | ACCESSORY | slots ACCESSORY_1, ACCESSORY_2 | - | -/-/1/0/0 | bd 0-2 | - | 2 |
| consumable | CONSUMABLE | - | - | payload stays on the item | - | - | 3 |
| material (NEW) | MATERIAL | - | - | - | - | - | 0 |
| quest (NEW) | QUEST | - | - | - | - | - | 0 |

Common to all weapon families: `block_cost` band 1-3 (glass wand 3), `burden` band 0-4, `adjustable` = power, range, block_cost, burden. Armor and shield: `adjustable` = mitigation, burden, block_cost. Accessory: burden. Modifier kinds: weapons and armor `adjust, stat_bonus, requirement, effect`; accessory `stat_bonus, effect, requirement`; consumable, material, quest none. `max_modifiers`: weapons 8, armor 6, shield 6, accessory 4 (schema cap 12). `max_tiers`: weapons 3, armor and shield 2, accessory 0. Orb and tome carry power 0: their value is in `effect` modifiers.

A `main_1h` item fits MAIN_HAND only while `Equipment.allow_plain_dual_wield` is false (default, main spec 8.1); with the flag true it also fits OFF_HAND (the old dual wield). A hand `off` item (shortsword, knuckles, dagger, orb, tome, shields) fits OFF_HAND only, always allowed. Migration does not change which hand any shipped item has.

## 3. Migration of the 31 items (what `collapse` must produce)

`family` = the item's `weapon_type` (non-empty), else accessory / consumable by `item_type`. D = deltas vs the family base (section 2). Req = `requirement`, Bon = `stat_bonus`. Stored `slot` is derived from the hand; the 3 off-category items change from MAIN_HAND to OFF_HAND (finding F1, main spec section 0).

| Item | Family | Hand | Scaling | Modifiers (adjust deltas, Req, Bon, effect) | Tiers |
| --- | --- | --- | --- | --- | --- |
| wpn_claymore | sword | main_2h | STR | power +6, burden +1, Req STR 14 | STR>=18 power +4 "Sunder Guard" |
| wpn_broadsword | sword | main_1h | STR | Req STR 10 | - |
| wpn_shortsword | sword | off | STR | power -4, Req STR 10 | - |
| wpn_hand_axe | axe | main_1h | STR | Req STR 10 | - |
| wpn_battle_axe | axe | main_2h | STR | power +11, burden +2, Req STR 15 | - |
| wpn_flanged_mace | mace | main_1h | STR | power +4, burden +1, Req STR 11 | - |
| wpn_cleric_mace | mace | main_1h | INT | Req INT 10, Bon KNOW +1 | - |
| wpn_warhammer | hammer | main_2h | STR | Req STR 22 | STR>=26 power +6 "Crushing Impact" |
| wpn_knuckles | fist | off | STR | Req STR 10 | - |
| wpn_dagger | dagger | off | DEX | power +1, Req DEX 12 | DEX>=15 power +4 "Puncture" |
| wpn_stiletto | dagger | main_1h | DEX | Req DEX 11 | - |
| wpn_shortbow | bow | main_2h | DEX | Req DEX 13 | DEX>=16 power +3 "True Flight" |
| wpn_staff | staff | main_2h | INT | Req INT 14 | INT>=18 power +5 "Arcane Surge" |
| wpn_mage_staff | staff | main_2h | INT | power +3, burden +2, Req INT 15 | - |
| wpn_wand | wand | main_1h | INT | Req INT 11 | - |
| wpn_glass_wand | wand | main_1h | INT | power +15, block_cost +2, Req INT 14, special | - |
| wpn_orb | orb | off | INT | Req INT 12, effect skill_potency spell +0.1 | - |
| wpn_tome | tome | off | INT | Req KNOW 11, Bon KNOW +1, effect skill_potency heal +0.1 | - |
| shd_kite | shield | off | - | Req STR 12 | - |
| shd_buckler | shield | off | - | burden -2, mitigation -4, Req DEX 10 | - |
| shd_tower | shield | off | - | burden +1, block_cost +1, mitigation +8, Req STR 15, special | - |
| arm_plate | plate | - (BODY) | - | Req CONS 12 | - |
| arm_leather | leather | - (BODY) | - | Req DEX 10 | - |
| arm_robe | cloth | - (BODY) | - | - | - |
| arm_cleric_vestment | cloth | - (BODY) | - | mitigation +1, Bon KNOW +1 | - |
| arm_mage_robe | cloth | - (BODY) | - | Bon INT +2 | - |
| acc_bear_ring | accessory | - (ACCESSORY_1) | - | Bon STR +2, Bon CONS +2 | - |
| acc_swift_amulet | accessory | - (ACCESSORY_2) | - | Bon DEX +3 | - |
| con_heal_potion, con_focus_elixir, con_fire_bomb | consumable | - | - | none (payload fields unchanged) | - |

Check against the file: wpn_staff deltas are 0 because the lower median of 22 and 25 is 22; wpn_mage_staff burden 2 and power 25. Shield base mitigation 8 is the lower median of 4, 8, 16. Tier `label` = text before ":" in the legacy `desc`; auto desc = "<label>: +<n> Power" reproduces all 5 legacy strings exactly (acceptance).

## 4. Tests to update (rule-derived; detail of main spec section 8)

| Suite | Hits | What changes |
| --- | --- | --- |
| `focus_gear_test_suite.gd` | 18 refs to `off_hand_category`, `weapon_type`, `two_handed` | Orb/tome rules move from "stored category must match the gear type" (a mismatch can no longer be expressed) to family rules: `focus` engine rule forces hands = [off]; a family or item that breaks it is the error; keep assertions on `Equipment.hand()`, slot OFF_HAND, never 2H, 2H main refusal (engine behaviour unchanged) |
| `foundation_test_suite.gd` | 15 (hand table line ~234, conflict test line ~294 sets `off_hand_category = "dagger"` on a 2H item) | Hand table derived from `family.hands` + item `hand`; the conflict case becomes `hand = main_2h` on a family without it (error "hand not allowed by family") and the legacy-row case (flat row with 2H and a category) |
| `gear_budget_test_suite.gd` | 5 | Numbers unchanged (acceptance); add: modifier breakdown sums to `item_value`; `tier_potential` = 0.5 x tier value and never inside `item_value` |
| `character_specs_test_suite.gd` | 4 (greatsword tiers) | Fixture uses a flat row; add extended tiers (stat_bonuses, effects, criteria); no dual-wield reference |
| `scenes/dojo/dojo_assertions.gd` Dojo-28 (and re-run Dojo-21c, ~423) | broadsword in OFF_HAND expected true | Per main spec 8.1: default flag expects false, second pass with the flag true expects true; 21c unchanged (shortsword is hand `off`) |
| `foundation_test_suite.gd` hand table | see above | Add flag-both-ways case for a plain `main_1h` item |
| Origin (inside the new suites) | - | system item key-field edit refused, user item deletable, system item delete refused, promote then delete refused, v1 rows load as system |
| `content_test_suite.gd` (claymore, line ~64), `gear_equip`, `ai_skills`, `combat_extensions`, `progression`, `save` | 1-4 each | Flat fixtures with `weapon_type` keep working (family derived); fixtures that put an impossible combination (e.g. a `fist` with INT scaling, a sword with range 3) now fail family validation: update the fixture to a legal combination, never loosen the rule |
| `world_test_suite.gd` | named item | Unchanged (named items resolve through `get_entry`); add one case per tuning key on a base with modifiers |
| NEW `item_families_test_suite.gd`, `item_modifiers_test_suite.gd`, `item_tiers_test_suite.gd`, `item_form_model_test_suite.gd` | - | See main spec section 8 |
