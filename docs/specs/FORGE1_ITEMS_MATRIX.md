# FORGE1_ITEMS_MATRIX: family seed table, migration of the 31 items, test list

Companion of `docs/specs/FORGE1_ITEMS.md` (read that first). Numbers below are DERIVED from the shipped `data/catalog/items.json` (31 entries; rechecked on main after BAL2-A, 2026-10-17) by the rules in section 1; the asset builder recomputes them from the file with `ContentCatalog.entries("items")` and must not trust this table over the data. Rows marked `NEW` have no shipped item; the user confirmed them (crossbow, material, quest) together with the proposed hands.

## 1. Derivation rules (mechanical, so the asset task needs no judgement)

1. `base[hand].<field>` of a family = the LOWER median of that field over its shipped items OF THAT HAND (one item = that item, so its modifiers shrink to none). A hand with no shipped item (NEW hands) copies the family's default hand. Section 2 below lists the default-hand numbers; per-hand values are recomputed by the seed. Worked examples after round 4 (one hand category per family; shortsword, knuckles and wpn_dagger are main_1h): sword `main_1h` 14 (lower median of broadsword 18 and shortsword 14), `main_2h` 24 (claymore); dagger `main_1h` 15 (lower median of stiletto 15 and wpn_dagger 16); fist `main_1h` 12. Per-hand bases exist only for sword, axe and staff (families with both main hands); every other family has one base.
2. `bands[hand].<field>` = [floor(0.75 x shipped min), ceil(1.25 x shipped max)] over that hand's items, clamped to the schema range; zero-only fields get [0, 10] (power) or [0, 0] (others); `bonus_value` = [0, 4] for main families, [0, 6] for off families (shield, orb, tome; they are support items with little or no power, so their power band is low and their bonus band high; seed defaults). The old dagger 16-vs-15 finding no longer exists (both daggers are main_1h; 16 sits inside the band 11-20). Bands are SOFT (warning) until after the balance pass (user answer); every family and item row below is seeded `origin: system`.
3. `hands` = hands present among shipped items, plus hands the spec lists as `NEW`. `scaling_stats` likewise, default = the most common.
4. A modifier `adjust` is written for every numeric field where the item differs from the family base (delta may be negative); `stat_bonus`, `requirement`, `effect` modifiers are copied one per stat / effect.
5. Round trip is the acceptance: `ItemModifiers.resolve(entry, family)` must equal the pre-migration flat row on every key the engine reads (`item_type, slot, weapon_type, base_power, base_range, two_handed, off_hand_category, scaling_stat, block_cost, burden, mitigation, required_stats, stat_bonuses, effects, threshold_tiers`), and `GearBudget.item_value` must be identical for all 31.

## 2. Family rows (20)

Columns: hands (default first) | scaling (default first) | base power/range/block/burden/mitigation | soft bands (power; other) | engine rules | shipped items. `slots` is derived from the hand for weapons and shields (see main spec 1.1) and listed explicitly only for armor and accessories.

| Family | item_type | Hands | Scaling | Base p/r/b/bd/m | Bands | Engine rules | Items |
| --- | --- | --- | --- | --- | --- | --- | --- |
| sword | WEAPON | main_1h, main_2h | STR | 14 (1h) / 24 (2h) /1/1/0/- | p 10-23 (1h), 18-30 (2h) | - | 3 |
| axe | WEAPON | main_1h, main_2h | STR | 16/1/1/0/- | p 12-34 | - | 2 |
| mace | WEAPON | main_1h | STR, INT | 13/1/1/0/- | p 9-22 | - | 2 |
| hammer | WEAPON | main_2h | STR | 34/1/1/3/- | p 25-43 | - | 1 |
| fist | WEAPON | main_1h | STR | 12/1/1/0/- | p 9-15 | - | 1 |
| dagger | WEAPON | main_1h | DEX | 15/1/1/0/- | p 11-20 | - | 2 |
| bow | WEAPON | main_2h | DEX | 20/3/1/0/- | p 15-25, r 2-6 | always_ranged | 1 |
| crossbow (NEW) | WEAPON | main_2h | DEX | 20/3/1/0/- (copy of bow) | p 15-25, r 2-6 | always_ranged | 0 |
| staff | WEAPON | main_2h, main_1h | INT | 22/2/1/0/- | p 16-32, r 1-3 | spell_source | 2 |
| wand | WEAPON | main_1h | INT | 15/3/1/0/- | p 11-38, r 2-4 | spell_source | 2 |
| orb | WEAPON | off (cat `orb`) | INT | 0/1/1/0/- | p 0-10 | focus | 1 |
| tome | WEAPON | off (cat `tome`) | INT | 0/1/1/0/- | p 0-10 | focus | 1 |
| shield | SHIELD | off (cat `shield`) | - | -/-/1/2/6 | m 3-20, bd 0-4, b 1-3 | shield | 3 |
| plate | ARMOR | slots HEAD, BODY | - | -/-/1/3/8 | m 6-10, bd 2-4 | - | 1 |
| leather | ARMOR | slots HEAD, BODY | - | -/-/1/1/4 | m 3-5, bd 0-2 | - | 1 |
| cloth | ARMOR | slots HEAD, BODY | - | -/-/1/0/2 | m 1-4, bd 0-1 | - | 3 |
| accessory | ACCESSORY | slots ACCESSORY_1, ACCESSORY_2 | - | -/-/1/0/0 | bd 0-2 | - | 2 |
| consumable | CONSUMABLE | - | - | payload stays on the item | - | - | 3 |
| material (NEW) | MATERIAL | - | - | - | - | - | 0 |
| quest (NEW) | QUEST | - | - | - | - | - | 0 |

Common to all weapon families: `block_cost` band 1-3 (glass wand 3), `burden` band 0-4, `adjustable` = power, range, block_cost, burden. Armor and shield: `adjustable` = mitigation, burden, block_cost. Accessory: burden. Modifier kinds: weapons and armor `adjust, stat_bonus, requirement, effect`; accessory `stat_bonus, effect, requirement`; consumable, material, quest none. `max_modifiers`: weapons 8, armor 6, shield 6, accessory 4 (schema cap 12). `max_tiers`: weapons 3, armor and shield 2, accessory 0. Orb and tome carry power 0: their value is in `effect` modifiers.

One hand category per family (main spec 8.1): `main_1h` and `main_2h` fit MAIN_HAND only; `off` items (orb, tome, shields) fit OFF_HAND only. The legacy code flag `allow_plain_dual_wield` (default false; set by the dojo) alone lets a `main_1h` item into OFF_HAND. Migration retypes three items: wpn_shortsword, wpn_knuckles and wpn_dagger go from hand `off` to `main_1h` (their stored slot MAIN_HAND was already main-hand); no other item changes hand.

## 3. Migration of the 31 items (what `collapse` must produce)

`family` = the item's `weapon_type` (non-empty), else accessory / consumable by `item_type`. D = deltas vs the family base of the DEFAULT hand (section 2); with per-hand bases the seed recomputes them against `base[item.hand]` (e.g. claymore power +6 becomes 0; with sword main_1h base 14, broadsword becomes power +4 and shortsword 0), the resolved rows stay identical. Req = `requirement`, Bon = `stat_bonus`. Stored `slot` is derived from the hand; the 3 retyped items keep slot MAIN_HAND (finding F1 disappears, main spec section 0), and `off_hand_category` becomes "" for them (icons via `icon_id`, main spec 1.1 `art`).

| Item | Family | Hand | Scaling | Modifiers (adjust deltas, Req, Bon, effect) | Tiers |
| --- | --- | --- | --- | --- | --- |
| wpn_claymore | sword | main_2h | STR | power +6, burden +1, Req STR 14 | STR>=18 power +4 "Sunder Guard" |
| wpn_broadsword | sword | main_1h | STR | power +4 (vs base 14), Req STR 10 | - |
| wpn_shortsword | sword | main_1h (was off) | STR | power -4, Req STR 10 | - |
| wpn_hand_axe | axe | main_1h | STR | Req STR 10 | - |
| wpn_battle_axe | axe | main_2h | STR | power +11, burden +2, Req STR 15 | - |
| wpn_flanged_mace | mace | main_1h | STR | power +4, burden +1, Req STR 11 | - |
| wpn_cleric_mace | mace | main_1h | INT | Req INT 10, Bon KNOW +1 | - |
| wpn_warhammer | hammer | main_2h | STR | Req STR 22 | STR>=26 power +6 "Crushing Impact" |
| wpn_knuckles | fist | main_1h (was off) | STR | Req STR 10 | - |
| wpn_dagger | dagger | main_1h (was off) | DEX | power +1, Req DEX 12 | DEX>=15 power +4 "Puncture" |
| wpn_stiletto | dagger | main_1h | DEX | Req DEX 11 | - |
| wpn_shortbow | bow | main_2h | DEX | Req DEX 13 | DEX>=16 power +3 "True Flight" |
| wpn_staff | staff | main_2h | INT | Req INT 14 | INT>=18 power +5 "Arcane Surge" |
| wpn_mage_staff | staff | main_2h | INT | power +3, burden +2, Req INT 15 | - |
| wpn_wand | wand | main_1h | INT | Req INT 11 | - |
| wpn_glass_wand | wand | main_1h | INT | power +15, block_cost +2, Req INT 14, special | - |
| wpn_orb | orb | off | INT | Req INT 12, effect skill_potency spell +0.1 | - |
| wpn_tome | tome | off | INT | Req KNOW 11, Bon KNOW +1, effect skill_potency heal +0.1 | - |
| shd_kite | shield | off | - | Req STR 12 | - |
| shd_buckler | shield | off | - | burden -2, mitigation -2, Req DEX 10 | - |
| shd_tower | shield | off | - | burden +1, block_cost +1, mitigation +10, Req STR 15, special | - |
| arm_plate | plate | - (BODY) | - | Req CONS 12 | - |
| arm_leather | leather | - (BODY) | - | Req DEX 10 | - |
| arm_robe | cloth | - (BODY) | - | - | - |
| arm_cleric_vestment | cloth | - (BODY) | - | mitigation +1, Bon KNOW +1 | - |
| arm_mage_robe | cloth | - (BODY) | - | Bon INT +2 | - |
| acc_bear_ring | accessory | - (ACCESSORY_1) | - | Bon STR +2, Bon CONS +2 | - |
| acc_swift_amulet | accessory | - (ACCESSORY_2) | - | Bon DEX +3 | - |
| con_heal_potion, con_focus_elixir, con_fire_bomb | consumable | - | - | none (payload fields unchanged) | - |

Check against the file: wpn_staff deltas are 0 because the lower median of 22 and 25 is 22; wpn_mage_staff burden 2 and power 25. Shield base mitigation 6 is the lower median of 4, 6, 16 and plate base 8 its single item (BAL2-A values, DECISIONS items-gear "BAL2 gear budget"; updated 2026-10-17). `off_hand_category`: `shd_tower` stores "shield", `shd_kite` and `shd_buckler` store ""; all three resolve to "shield" (main spec 8 acceptance (b)). Tier `label` = text before ":" in the legacy `desc`; auto desc = `<label>: +<n> Power` reproduces all 5 legacy strings exactly (acceptance).

## 4. Tests to update (rule-derived; detail of main spec section 8)

| Suite | Hits | What changes |
| --- | --- | --- |
| `focus_gear_test_suite.gd` | 18 refs to `off_hand_category`, `weapon_type`, `two_handed` | Orb/tome rules move from "stored category must match the gear type" (a mismatch can no longer be expressed) to family rules: `focus` engine rule forces hands = [off]; a family or item that breaks it is the error; keep assertions on `Equipment.hand()`, slot OFF_HAND, never 2H, 2H main refusal (engine behaviour unchanged) |
| `foundation_test_suite.gd` | 15 (hand table line ~234, conflict test line ~294 sets `off_hand_category = "dagger"` on a 2H item) | Hand table derived from `family.hands` + item `hand`; the conflict case becomes `hand = main_2h` on a family without it (error "hand not allowed by family") and the legacy-row case (flat row with 2H and a category) |
| `gear_budget_test_suite.gd` | 5 | Numbers unchanged (acceptance); add: modifier breakdown sums to `item_value`; `tier_potential` = 0.5 x tier value and never inside `item_value` |
| `character_specs_test_suite.gd` | 4 (greatsword tiers) | Fixture uses a flat row; add extended tiers (stat_bonuses, effects, criteria); no dual-wield reference |
| `scenes/dojo/dojo_assertions.gd` Dojo-13, 13b, 21, 21c (~423), 28; `scripts/dojo/dojo_loadouts.gd` presets `dual_wield`, `claymore_backup`, `duelist`, `archer`; dojo session | shortsword / dagger / broadsword in OFF_HAND | Per section 5 table: the dojo sets the legacy flag so these keep running; strict-rule passes (flag false) added; Dojo-28 reversed for the retyped items |
| `foundation_test_suite.gd` `HAND_TABLE` (line 234), hand-conflict test (294), `_test_two_handed_offhand` (324) | shortsword / knuckles / dagger as `off` | Three table entries become `main_1h`; conflict case uses category `tome`; dagger fixture sets the flag; add flag-both-ways case for a plain `main_1h` item |
| `art_terrain_test_suite.gd` ~122 | shortsword icon | Unchanged, icon via `icon_id` |
| Origin (inside the new suites) | - | system item key-field edit refused, user item deletable, system item delete refused, promote then delete refused, v1 rows load as system |
| `content_test_suite.gd` (claymore, line ~64), `gear_equip`, `ai_skills`, `combat_extensions`, `progression`, `save` | 1-4 each | Flat fixtures with `weapon_type` keep working (family derived); fixtures that put an impossible combination (e.g. a `fist` with INT scaling, a sword with range 3) now fail family validation: update the fixture to a legal combination, never loosen the rule |
| `world_test_suite.gd` | named item | Unchanged (named items resolve through `get_entry`); add one case per tuning key on a base with modifiers |
| NEW `item_families_test_suite.gd` (1) | - | record validation, invariants, rule lists equal the code constants, remove/retype refusals; the 8.1 cases of section 5 below |
| NEW `item_modifiers_test_suite.gd` (2) | - | round trip over every shipped item; sandbox edits: a family base change moves only inheriting items; kind/cap/uniqueness errors; budget sum equals `item_value`; legacy flat upsert |
| NEW `item_tiers_test_suite.gd` (3) | - | each criterion (stat, proficiency, unit level) and bonus (power, mitigation, stat_bonuses, effects); no circularity; ordering; byte-identical when no tiers |
| NEW `item_form_model_test_suite.gd` (4) | - | headless: sections, disabled-Save reasons, dynamic limits, token resolution |

## 5. Strict hand typing: checks to update (detail of main spec 8.1; moved from the main spec 2026-10-17, unchanged)

Checks that rely on shortsword, knuckles or dagger being off-hand weapons, and the rule-derived update of each (logic task; listed so none is missed). The dojo session sets the flag, so its dual-wield checks keep running; strict-rule checks are added next to them. Line numbers are approximate (builder greps).

| Where | Relied on | Update |
| --- | --- | --- |
| Dojo session setup (`dojo_session`) | - | NEW: set `allow_plain_dual_wield = true` at session start, restore on exit; the loadouts below then equip as before |
| Dojo-13 (`dojo_assertions.gd` ~153-163) off-hand shortsword STR scaling | shortsword to OFF_HAND | Expectation unchanged (scaling of the held item is engine behaviour); runs under the dojo flag; relabel "Legacy dual wield (flag): off-hand Shortsword STR scaling" |
| Dojo-13b (~165-174) off-hand knife DEX scaling | wpn_dagger to OFF_HAND | Same: unchanged under the flag, relabel "Legacy dual wield (flag): off-hand Dagger DEX scaling" |
| Dojo-13c (shield) | - | NO change. Add Dojo-13d: with the flag false, shortsword and dagger are refused in OFF_HAND, tome and orb accepted |
| Dojo-21 (~286-301) 2H main refuses shield, accepts off-hand weapon (stash shortsword) | shortsword is an off item | Under the flag the pairing is the legacy rule and expectation is unchanged; add the strict pass: flag false, a 2H main refuses shield, focus item and plain 1H (nothing fits the off hand) |
| Dojo-21c (~323-343) and ~423 (Dual Strike combo, preset `dual_wield`) | broadsword + shortsword | Unchanged under the flag (Dual Strike needs a 1H melee main plus an offensive off-hand weapon); add: flag false, Quick Setup skips the shortsword and reports it (the Dojo-20 mechanism) |
| Dojo-28 (~495-517) slot rules | `shortsword.can_equip_in_slot(off)` true, main false; stash shortsword refused in main | Reverse for the retyped items: shortsword is main_1h, so in MAIN_HAND it is accepted; flag false: off refused; flag true: off accepted. Retarget the "off item refused in main" half to the tome (refused in main, accepted in off); buckler/claymore expectations unchanged; label "off-hand items (shield, orb, tome) off-hand only; main-hand weapons main-hand only unless the legacy flag" |
| Presets `dual_wield`, `claymore_backup`, `duelist` (Blade & Dagger), `archer` (`dojo_loadouts.gd` lines 42, 64, 76, 88) | a main_1h item in OFF_HAND | Item lists unchanged; they need the dojo flag (set at session start). Descriptions updated to "legacy dual wield (dojo)" |
| `foundation_test_suite` `HAND_TABLE` (line 234) | wpn_shortsword, wpn_knuckles, wpn_dagger = "off" | Change those three to "main_1h" (derived hand follows the cleared category); wpn_tome, wpn_orb, shields stay "off". Hand-conflict test (line 294) sets `off_hand_category = "dagger"` on the claymore: use "tome" (a 2H item with any off category is still the error). Add: flag false -> `broadsword.can_equip_in_slot(off)` false, flag true -> true |
| `foundation_test_suite` `_test_two_handed_offhand` (~324-328) rogue with wpn_dagger in OFF_HAND | dagger off-hand swing | Set the flag in that fixture (and restore); expectation unchanged. Contrast case may also use bare hands |
| `focus_gear_test_suite` (orb, tome fixtures), `gear_equip`, `character_specs` (greatsword tiers) | `character_specs` and `gear_equip` have no shortsword, knuckles or dagger reference (grep) | NO change; the builder re-greps `create_iron_shortsword`, `create_spiked_knuckles`, `create_shadow_dagger` and sets the flag in any fixture that equips them off-hand |
| `art_terrain_test_suite` ~122 | `icon_key(shortsword) == "shortsword"` | Unchanged (icon via `icon_id`, main spec 1.1 `art`) |
| New in `item_families_test_suite` | - | a family with `off` plus a main hand is an error; slot derivation with the flag both ways; stash `OFF_HAND` filter hides plain 1H by default; shipped retyped items have hand main_1h and `off_hand_category` "" |

## 6. Form layout (detail of main spec 6 and commit 3; moved 2026-10-17, unchanged)

Item form sections: 1 Identity (id, name, description + token help, icon, rarity, value, weight, starter_stash); 2 Family and build (family picker; read-only chips item type, gear token, slot; hand; scaling stat; slot for armor/accessory; damage type); 3 Base values (read-only grid with "family base + adjustments = result" per number and band marker); 4 Modifiers (table, add disabled at the cap with the reason); 5 Tier thresholds (table); 6 Special; 7 Consumable payload (consumables only); 8 Preview (resolved summary, per-modifier budget values, loadouts and named items that use it, problems, warnings).

Family form: Identity, Type and hands, Template numbers and bands, Modifier policy, Engine rules (checkbox per rule with its help), impact panel, "Used by" (items).

Item list filter chips: family, hand, rarity, special, has tiers, has effects, origin, used by class/unit/named item.
