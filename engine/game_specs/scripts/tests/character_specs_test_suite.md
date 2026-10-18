---
class: CharacterSpecsTestSuite
extends: RefCounted
description: "TEST 3: CHARACTER SPECS, STATS, GEAR & INVENTORY TEST"
---

# Methods

### `static run(_tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING CHARACTER SPECS & INVENTORY TEST       ===")
print("=======================================================")

# 1. Base initialization from archetype
var warrior_arch := ClassArchetype.create_warrior()
var stats := CharacterStats.new(warrior_arch, 1)

var base_stats_ok: bool = (
	stats.str_stat == warrior_arch.get_base_stat("STR") and
	stats.dex_stat == warrior_arch.get_base_stat("DEX") and
	stats.int_stat == warrior_arch.get_base_stat("INT") and
	stats.cons_stat == warrior_arch.get_base_stat("CONS") and
	stats.perc_stat == warrior_arch.get_base_stat("PERC") and
	stats.know_stat == warrior_arch.get_base_stat("KNOW") and
	StatRules.total_of(warrior_arch.base_stats) == StatRules.CLASS_BUDGET and
	stats.unallocated_attribute_points == StatRules.spend_pool(1) and
	stats.level == 1
)

# 2. Derived Max HP & Speed
# Max HP = 60 + CONS * 8 + STR * 2; Speed = 8 + DEX; bare-hand power is DamageCalculator's
# (unit power STR + DEX plus any trained-fist weapon power, times the tier scale)
var bare := Combatant.new("Bare", 100, 10, 1, stats)
var bare_parts: Dictionary = DamageCalculator.attack_power(bare, false)
var bare_unit: float = DictUtil.to_float(bare_parts["unit_power"])
var bare_expected: int = maxi(1, roundi((DictUtil.to_float(bare_parts["weapon_power"]) + bare_unit) * bare.power_scale))
var derived_stats_ok: bool = (
	stats.get_max_hp() == 60 + stats.cons_stat * 8 + stats.str_stat * 2 and
	stats.get_initiative_speed() == 8 + stats.dex_stat and
	bare_unit == float(stats.str_stat + stats.dex_stat) and
	bare.get_effective_power(false) == bare_expected
)
var before_str: int = stats.str_stat
var before_cons: int = stats.cons_stat
var before_perc: int = stats.perc_stat
var before_int: int = stats.int_stat
var before_dex: int = stats.dex_stat
var before_know: int = stats.know_stat

# 3. Guaranteed level up growths (Level 1 -> 2)
# Growth comes from the class (warrior: CONS +3, STR +1, PERC +1)
var lvl_ok: bool = stats.level_up()
var growths_ok: bool = (
	lvl_ok and
	stats.level == 2 and
	stats.str_stat == before_str + warrior_arch.get_growth_for_stat("STR") and
	stats.cons_stat == before_cons + warrior_arch.get_growth_for_stat("CONS") and
	stats.perc_stat == before_perc + warrior_arch.get_growth_for_stat("PERC") and
	stats.unallocated_attribute_points == StatRules.spend_pool(1) + StatRules.FREE_PER_LEVEL
)

# 4. Allocate 1 free point from level up
var alloc_lvl_point_ok: bool = stats.allocate_attribute_point("INT") and stats.int_stat == before_int + 1 and stats.unallocated_attribute_points == StatRules.spend_pool(2) - 1

# 5. Outside Event Stat Points (e.g. Ancient Shrine)
stats.add_free_attribute_points(2)
var alloc_event_points_ok: bool = (
	stats.unallocated_attribute_points == StatRules.spend_pool(2) + 1 and
	stats.allocate_attribute_point("DEX") and
	stats.allocate_attribute_point("KNOW") and
	stats.dex_stat == before_dex + 1 and
	stats.know_stat == before_know + 1 and
	stats.unallocated_attribute_points == StatRules.spend_pool(2) - 1
)

# 6. Proficiency (class gear training) and passives (granted by lore sources, never by the class)
var sword_power_bonus: int = stats.proficiency.get_weapon_power_bonus("sword")
var sword_discount: int = stats.proficiency.get_weapon_requirement_discount("sword")
var sage := Combatant.new("Sage", 100, 12, 1, stats)
ContentFactory.apply_passives(sage, ["ruin_lore"])
var ruins_context: Dictionary = {"biome": "ruins"}
var mastery_ok: bool = (sword_power_bonus == 3 and sword_discount == 6 and stats.proficiency.level_of("bow") == 0
	and stats.get_effective_stat("KNOW", ruins_context) == stats.know_stat + 3 and stats.get_effective_stat("KNOW") == stats.know_stat and sage.passives.has("ruin_lore"))

# 7. Equipment Stat Requirements & Proficiency Threshold Discount
# Sword proficiency 3 grants a 6-point requirement discount: the sword asks exactly STR + 6 (the discounted STR meets tier 1, not tier 2)
var tier_one: int = stats.str_stat + sword_discount
var greatsword := Equipment.new(
	"iron_greatsword",
	"Iron Greatsword",
	Equipment.EquipSlot.MAIN_HAND,
	"sword",
	Item.ItemType.WEAPON,
	18,
	{"STR": tier_one},
	[
		{"stat": "STR", "threshold": tier_one, "bonus_power": 4, "desc": "Tier 2: Crushing Strike"},
		{"stat": "STR", "threshold": tier_one + 6, "bonus_power": 8, "desc": "Tier 3: Sunder Guard"}
	]
)

var meets_req_ok: bool = greatsword.meets_requirements(stats, stats.proficiency)

# 8. Threshold Tiers Unlocked (STR + discount 6 meets the Tier 2 threshold)
# Base 18 + Tier 4 + sword proficiency 3 = 25
var active_tiers: Array[Dictionary] = greatsword.get_active_threshold_tiers(stats, stats.proficiency)
var tiers_ok: bool = (active_tiers.size() == 1 and greatsword.get_effective_equipment_power(stats, stats.proficiency) == 25)

# 9. Deficit Wield Penalty Check (Character with 11 DEX tries to wield Longbow requiring 24 DEX)
var longbow := Equipment.new(
	"yew_longbow",
	"Yew Longbow",
	Equipment.EquipSlot.MAIN_HAND,
	"bow",
	Item.ItemType.WEAPON,
	16,
	{"DEX": 24}
)
var bow_penalty: Dictionary = longbow.get_wield_penalty(stats, stats.proficiency)
var penalty_ok: bool = (
	not longbow.meets_requirements(stats, stats.proficiency) and
	bow_penalty["is_penalized"] == true and
	bow_penalty["deficit"] > 0 and
	longbow.get_effective_equipment_power(stats, stats.proficiency) < 16 and
	longbow.get_effective_block_cost(stats, stats.proficiency) > 1
)

# 10. Combatant Integration, Inventory & Battle Pouch Consumable
var test_unit := Combatant.new("Sir Gareth", 200, 18, 1, stats)
test_unit.inventory.equip(greatsword)
test_unit.sync_from_stats_and_gear()

var potion := Consumable.new("health_potion", "Healing Draught", Consumable.ConsumableType.HEALTH_POTION, 40)
test_unit.inventory.assign_pouch_item(0, potion)

test_unit.hp = 120
var use_outcome: Dictionary = test_unit.inventory.use_pouch_item(0, test_unit)
var pouch_used_ok: bool = (
	use_outcome.get("healed", 0) == 40 and
	test_unit.hp == 160 and
	test_unit.inventory.battle_pouch.is_empty()
)

print("\n=======================================================")
print("=== CHARACTER SPECS & INVENTORY TEST ASSERTIONS     ===")
print("=======================================================")
print("  [Specs] Archetype 6-stat baselines initialized: %s" % ("PASS" if base_stats_ok else "FAIL"))
print("  [Specs] Derived Max HP & Initiative Speed calculated: %s" % ("PASS" if derived_stats_ok else "FAIL"))
print("  [LevelUp] Level 2 guaranteed growths applied: %s" % ("PASS" if growths_ok else "FAIL"))
print("  [LevelUp] Free attribute points allocated: %s" % ("PASS" if alloc_lvl_point_ok else "FAIL"))
print("  [Event] Outside event points awarded & spent: %s" % ("PASS" if alloc_event_points_ok else "FAIL"))
print("  [Proficiency] Gear training and passive stat bonuses: %s" % ("PASS" if mastery_ok else "FAIL"))
print("  [Gear] Stat requirement & proficiency discount check: %s" % ("PASS" if meets_req_ok else "FAIL"))
print("  [Gear] Threshold tiers active boosts unlocked: %s" % ("PASS" if tiers_ok else "FAIL"))
print("  [Gear] Deficit wield penalty applied: %s" % ("PASS" if penalty_ok else "FAIL"))
print("  [Inventory] Battle pouch item consumed in combat: %s" % ("PASS" if pouch_used_ok else "FAIL"))

return (base_stats_ok and derived_stats_ok and growths_ok and alloc_lvl_point_ok and
	alloc_event_points_ok and mastery_ok and meets_req_ok and tiers_ok and penalty_ok and pouch_used_ok)
```

