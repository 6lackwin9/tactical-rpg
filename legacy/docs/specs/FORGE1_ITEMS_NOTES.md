# FORGE1_ITEMS_NOTES: findings, decision record and rationale behind FORGE1_ITEMS

Companion of `docs/specs/FORGE1_ITEMS.md` (the builder contract; read that first). Reference only: nothing here adds a rule. Moved out of the main spec on 2026-10-17 to keep it under the agent-doc size limit; text is unchanged except where marked "updated 2026-10-17". Section numbers in brackets name the main-spec section each part belongs to.

## 1. What the code did when the spec was written [main spec 0]

Checked 2026-10-16; line numbers are pre-CLN-9 and pre-BAL2 (builder greps).

| Fact | Consequence |
| --- | --- |
| 31 items: 18 WEAPON, 3 SHIELD, 5 ARMOR (all BODY), 2 ACCESSORY, 3 CONSUMABLE; no MATERIAL/QUEST. Per `weapon_type`: sword 3, axe 2, mace 2, hammer 1, fist 1, dagger 2, bow 1, staff 2, wand 2, orb 1, tome 1, shield 3, plate 1, leather 1, cloth 3; crossbow 0 | 20 families cover everything (matrix section 2) |
| `Proficiency.gear_types()` = 13 weapon tokens (incl. `shield`) + plate/leather/cloth. The token drives: class `proficiency` keys, `ClassArchetype.can_equip` (`can_use(weapon_type)`; empty type = always usable), scaling/power/crit/burden/block-cost training, `Combatant.wielded_type()` for skill `weapon_types` gating (main-hand type or `unarmed`), `StashFilter`, `ItemArt` | family id of a gear family MUST equal the token |
| Today hand is mixed inside families: sword has a 2H (claymore), a 1H (broadsword) and an off-hand (shortsword, `off_hand_category` shortsword); dagger has 1H (stiletto) and off (wpn_dagger); fist (knuckles) is off; mace scaling is STR (flanged) or INT (cleric mace); staff and bow are 2H. `Equipment.hand()` is derived from `is_two_handed` + `off_hand_category`. A plain 1H weapon fits MAIN_HAND or OFF_HAND today (dual wield; ends with strict typing, 8.1); an off-category weapon fits OFF_HAND only | FORGE1_DESIGN's single `hand` per family (`either_main`) was wrong. Round 4 resolves the mix differently: ONE hand CATEGORY per family (main or off), the item still picks main_1h or main_2h inside a main family (1.2, 8.1) |
| F1 latent bug: wpn_shortsword, wpn_knuckles, wpn_dagger store `slot: MAIN_HAND` but are off-hand only (`can_equip_in_slot`); callers that equip into `eq.slot` would target the wrong slot | Round 4 retypes those 3 items as main_1h, so the stored MAIN_HAND slot becomes correct: migration changes NO slot. `slot` is still derived from the hand and not stored |
| Hard-coded type tokens in code: `Equipment.FOCUS_TYPES` (orb, tome), `is_ranged_weapon` (bow, crossbow, sling; `sling` is not a gear type), `is_shield` (SHIELD or category shield), `SkillDef` spell gate (staff or wand), `Inventory` lines 87/111 | These are the "engine rules": 4 names (`focus`, `always_ranged`, `shield`, `spell_source`), see 1.3 |
| Tier thresholds: `{stat, threshold, bonus_power, desc}`; criterion = `get_effective_stat + prof.level x 2 (LEVEL_DISCOUNT)` vs threshold; only `bonus_power` is read (`Equipment.get_effective_equipment_power`, `DamageCalculator`, `ui_theme`); validator checks `stat` and `threshold` exist only; 5 items use it (claymore, warhammer, shortbow, wpn_dagger, wpn_staff), all power-only | The section exists as a power stub; criteria and bonuses are generalised in 4 |
| `GearBudget.item_value` reads the flat row (power, mitigation, stat bonuses, burden, block cost, three effect types) and ignores `threshold_tiers`; `shd_tower` and `wpn_glass_wand` are `special` | Budget keeps reading the resolved flat row; numbers must not move (acceptance) |
| World named item = `base_item` + `tuning` {power, mitigation, burden} + extra `stat_bonuses` + `grants` (`WorldFactory.named_item` copies `get_entry`, edits numbers, calls `ContentFactory.item`); 1 named item shipped (ashlands) | Works unchanged if `get_entry` returns the resolved flat row (5) |
| `effects` and `threshold_tiers` are raw JSON text boxes in the Forge; `required_stats`, `stat_bonuses` use the stats widget | Replaced by typed tables (6) |

Updated 2026-10-17: `shd_kite` and `shd_buckler` store `off_hand_category` "" while `shd_tower` stores "shield"; `is_shield` is true for all three by `item_type` SHIELD, and `ItemArt.icon_key` reads the category only for the shortsword icon, so deriving "shield" for all three (main spec 1.1) is engine-neutral.

## 2. Why one hand category per family [main spec 1.2]

From shipped counts: main-hand weapons go 13 -> 16 of 18 (shortsword, knuckles, wpn_dagger join); off-hand items go 8 -> 5 (orb, tome, 3 shields). Main coverage was the weaker side: fist had no main weapon (1 item, off), dagger 1 of 2, sword 2 of 3, while the off hand is already served by shield (3) and the focus pair, and the off category matches the engine rules `focus` and `shield`. Hands present in data are facts; the rest (mace/wand 1H only, hammer 2H only, staff also 1H, crossbow copying bow, material and quest placeholders) were CONFIRMED by the user (DECISIONS "FORGE1 item answers").

Bands [main spec 1.1 `bands`]: off-category families are budgeted LOWER on power and HIGHER on bonus value because off-hand weapons are SUPPORT items (utility and bonuses, little or no power: orb and tome have power 0, shields none); main families are damage weapons. A family has one category, so the old per-family relation warnings between `off` and `main_1h` (including the dagger 16-vs-15 finding) no longer exist.

Strict typing [main spec 8.1]: power is budgeted per hand; moving a weapon between hands would make it too weak or too strong.

## 3. Copy versus inherit: option record [main spec 3]

User answer: items INHERIT from the family template with deltas (modifiers `adjust`), with the impact panel as the drift rail. Options kept for the record:

| Option | Gameplay | Cost | Risk |
| --- | --- | --- | --- |
| C. Copy: items keep full numbers, family seeds new items and validates bands (FORGE1_DESIGN I1) | none | no engine change; items.json unchanged in shape | family stops being "the template": changing a base never reaches existing items; hands/rules duplicated per item (the F1 bug class stays) |
| I. Inherit: structural facts (type, slot, hand set, weapon token, category, rules) and base numbers resolved from the family at load; items store hand/scaling/modifiers only | editing a family base moves every item that did not override | `ItemModifiers.resolve/collapse`, loader/saver, version 2 file, migration seed (31 items, mechanical), tests | silent budget drift when a base changes |
| H. Hybrid: structure inherited, numbers always explicit | none | as I | modifiers become absolute, loses the "template + deltas" the user asked for |

Chosen: I (DECISIONS items-gear "FORGE1 item answers"). HISTORY.md row 2026-10-16 logs the replaced FORGE1_DESIGN `either_main` hand and "copy" storage (I1).

## 4. HISTORY.md check [main spec header]

Nothing in FORGE1_ITEMS re-proposes a deprecated idea. "Hard-coded items -> JSON catalog + Forge" is extended, not reversed. "Gear proficiency as a plain allowed list" stays deprecated (proficiency dict untouched). One replaced idea of FORGE1_DESIGN is logged: its `either_main` hand and "copy" storage (I1), reasons in section 1 and 3 above. Dual wield: DECISIONS "FORGE1 item answers" keeps the code path and adds a blocking guard (main spec 8.1); nothing in HISTORY.md touches it. DECISIONS kept: gear budget weights (soft, special items with notes), hands main_1h/main_2h/off, orb/tome off-hand only, item effects use the passive grammar restricted to item types, stat bonuses never negative, weapons never scale from KNOW, named items = base + tuning, edits only through `ContentCatalog.upsert`.

## 5. Later option: named-item modifiers (ITEM2, not now) [main spec 5]

Named items get their own `modifiers` list using the same `ItemModifiers` kinds, so lore items can add an effect; it needs `world_schema` `_item_fields` and `named_item` changes (note: `named_item` does not cap `block_cost` or re-validate the family band; accepted).

## 6. Modifier list extension and tier iteration [main spec 2.3, 4.3]

Extending the modifier list later (e.g. `mitigation_vs`, `on_hit`) = new descriptor row + resolve branch + budget weight. The extra tier features are built now and iterated after the balance pass (user answer).
