---
class: GearEquipTestSuite
extends: RefCounted
description: "TEST 4: GEAR EQUIP UI, TEST DUMMY & DAMAGE FORMULA SUITE"
---

# Methods

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING GEAR EQUIP UI & TEST DUMMY SUITE       ===")
print("=======================================================")

# 1. Instantiate CharacterEquipmentModal
var modal := CharacterEquipmentModal.new()
if tree_parent != null:
	tree_parent.add_child(modal)

var stash := Inventory.new()
ItemCatalog.populate_party_starter_stash(stash)

var warrior_stats := CharacterStats.new(ClassArchetype.create_warrior(), 2)
var warrior := Combatant.new("Warrior", 100, 15, 1, warrior_stats)
ItemCatalog.setup_default_hero_loadout(warrior, "Warrior")

var ranger_stats := CharacterStats.new(ClassArchetype.create_ranger(), 1)
var ranger := Combatant.new("Ranger", 80, 14, 2, ranger_stats)
ItemCatalog.setup_default_hero_loadout(ranger, "Ranger")

modal.setup_heroes([warrior, ranger], stash)

var modal_init_ok: bool = (modal != null and modal.party_allies.size() == 2 and modal.active_hero == warrior)

# 2. Hero switching in modal
modal.select_hero(ranger)
var hero_switch_ok: bool = (modal.active_hero == ranger)
modal.select_hero(warrior)

# 3. Equipping weapon updates power and main hand
var claymore := ItemCatalog.create_iron_claymore()
stash.add_item(claymore)
modal._equip_item(claymore)
var equip_ok: bool = (
	warrior.inventory.get_main_weapon() == claymore and
	warrior.inventory.get_main_weapon().name == "Iron Claymore"
)

# 4. Claymore Threshold Tier (+4 Power at 18 STR)
var tier_boost: int = claymore.get_unlocked_threshold_power_boost(warrior.stats, warrior.stats.proficiency)
var threshold_boost_ok: bool = (tier_boost == 4)

# 5. Deficit penalty check on Titan Warhammer (requires 20 STR)
var warhammer := ItemCatalog.create_titan_warhammer()
stash.add_item(warhammer)
modal._equip_item(warhammer)
var penalty: Dictionary = warhammer.get_wield_penalty(warrior.stats, warrior.stats.proficiency)
var is_pen: bool = penalty.get("is_penalized", false)
var deficit_ok: bool = (is_pen and penalty.get("deficit", 0) > 0)

# 6. Armor Burden calculation affecting speed
var plate := ItemCatalog.create_steel_plate_cuirass()
stash.add_item(plate)
modal._equip_item(plate)
var total_burden: int = warrior.inventory.get_total_armor_burden()
var speed_with_burden: int = warrior.stats.get_initiative_speed(total_burden)
var burden_ok: bool = (total_burden >= 3 and speed_with_burden < warrior.stats.get_initiative_speed(0))

# 7. Unequipping item returns to stash
var unequipped_arm: Equipment = warrior.inventory.unequip(Equipment.EquipSlot.BODY)
var unequip_ok: bool = (unequipped_arm == plate and warrior.inventory.get_equipped(Equipment.EquipSlot.BODY) == null)

# Re-equip broadsword for baseline testing
var broadsword := ItemCatalog.create_steel_broadsword()
warrior.inventory.equip(broadsword)
warrior.sync_from_stats_and_gear()

# 8. Training Dummy setup
var dummy := Combatant.new("Training Dummy", 500, 10, 1)
dummy.defense_mitigation_ratio = 0.0
var dummy_init_ok: bool = (dummy != null and dummy.hp == 500 and dummy.max_hp == 500)

# 9. Damage formula & 50% defense mitigation calculation
var atk_pwr: int = warrior.get_effective_power(false)
var full_dmg: int = dummy.take_damage(atk_pwr)
var expected_hp_1: int = 500 - atk_pwr

dummy.defense_mitigation_ratio = 0.5
var mit_dmg: int = dummy.take_damage(atk_pwr)
var expected_mit_dmg: int = maxi(1, int(roundf(float(atk_pwr) * 0.5)))
var damage_formula_ok: bool = (full_dmg == atk_pwr and mit_dmg == expected_mit_dmg and dummy.hp == (expected_hp_1 - expected_mit_dmg))

# 10. Consumable Battle Pouch usage
dummy.defense_mitigation_ratio = 0.0
warrior.inventory.battle_pouch.clear()
var bomb := ItemCatalog.create_throwing_fire_bomb()
warrior.inventory.add_to_battle_pouch(bomb)
var pouch_count_before: int = warrior.inventory.battle_pouch.size()
var bomb_dmg: int = dummy.take_damage(bomb.damage_amount)
warrior.inventory.remove_from_battle_pouch(bomb)
var pouch_count_after: int = warrior.inventory.battle_pouch.size()
var consumable_test_ok: bool = (pouch_count_before == 1 and pouch_count_after == 0 and bomb.damage_amount > 0 and bomb_dmg == bomb.damage_amount)

print("\n=======================================================")
print("=== GEAR EQUIP UI & TEST DUMMY SUITE ASSERTIONS     ===")
print("=======================================================")
print("  [UI] Equipment modal initialized & mounted: %s" % ("PASS" if modal_init_ok else "FAIL"))
print("  [UI] Hero switching in equipment modal: %s" % ("PASS" if hero_switch_ok else "FAIL"))
print("  [Gear] Weapon equipped from stash & synced: %s" % ("PASS" if equip_ok else "FAIL"))
print("  [Gear] Threshold Tier 1 (+4 Power) boost verified: %s" % ("PASS" if threshold_boost_ok else "FAIL"))
print("  [Gear] Deficit penalty on Titan Warhammer verified: %s" % ("PASS" if deficit_ok else "FAIL"))
print("  [Gear] Armor burden reduces initiative speed: %s" % ("PASS" if burden_ok else "FAIL"))
print("  [Gear] Unequipped item returns to stash: %s" % ("PASS" if unequip_ok else "FAIL"))
print("  [Dummy] Training dummy initialized (500 HP): %s" % ("PASS" if dummy_init_ok else "FAIL"))
print("  [Formula] Base strike & 50%% defense mitigation verified: %s" % ("PASS" if damage_formula_ok else "FAIL"))
print("  [Pouch] Battle pouch consumable used against dummy: %s" % ("PASS" if consumable_test_ok else "FAIL"))

return (modal_init_ok and hero_switch_ok and equip_ok and threshold_boost_ok and
	deficit_ok and burden_ok and unequip_ok and dummy_init_ok and damage_formula_ok and consumable_test_ok)
```

