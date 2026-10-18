---
class: UnarmedTestSuite
extends: RefCounted
description: "Tactical RPG - Unarmed damage and gear budget (M2c-2a2 + BAL2-L Rules 3 and 5; split from EnginePiecesTestSuite by CLN-12). Bare hands use STR+DEX, trained fists add the fist base and training, GearBudget counts an empty main hand as UNARMED_VALUE; the shipped classes raise no budget warning. Expectations come from DamageCalculator and GearBudget constants."
---

# Properties
* `TAG`: String = "EnginePieces" (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING UNARMED AND GEAR BUDGET TEST (M2c-2a2)  ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = true
ok = _test_unarmed() and ok
ok = _test_shipped_gear_warnings() and ok
return ok
```

### `static _test_unarmed() -> bool`
```logic
var checks: Dictionary = {}
# Fixture: the brawler armed with a flanged mace in memory (unit slot override / class copy), so the
# checks hold whether the shipped brawler carries a weapon or not (BAL2-L Rule 3).
var fixture_weapon: String = "wpn_flanged_mace"
var fighter: Combatant = ContentFactory.unit({"id": "t_ep_brawler", "name": "Fist", "class_id": "brawler", "level": 4, "main_hand": fixture_weapon})
var armed_name: String = str(DamageCalculator.attack_power(fighter)["stat_name"])
fighter.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
var power: Dictionary = DamageCalculator.attack_power(fighter)
var str_val: int = fighter.stats.get_effective_stat("STR", {})
var dex_val: int = fighter.stats.get_effective_stat("DEX", {})
checks["armed unit does not use the bare-hand path"] = armed_name != "STR+DEX"
checks["no weapon: stat is STR+DEX"] = str(power["stat_name"]) == "STR+DEX" and power["weapon"] == null
# Trained fists (BAL2-L Rule 5): fist level >= TRAINED_FIST_MIN_LEVEL adds UNARMED_BASE_POWER + fist training.
var fist_level: int = fighter.stats.proficiency.level_of("fist")
var fist_power: int = DamageCalculator.UNARMED_BASE_POWER + fighter.stats.proficiency.get_weapon_power_bonus("fist")
checks["fixture brawler has trained fists (fist %d)" % fist_level] = fist_level >= DamageCalculator.TRAINED_FIST_MIN_LEVEL and DictUtil.to_bool(power["trained_fists"])
checks["unit power = STR + DEX, weapon power = fist base + training"] = DictUtil.to_float(power["unit_power"]) == float(str_val + dex_val) and DictUtil.to_float(power["weapon_power"]) == float(fist_power)
checks["attack power = (fist base + training + STR + DEX) x power scale"] = DictUtil.to_int(power["attack_power"]) == maxi(1, roundi(float(fist_power + str_val + dex_val) * fighter.power_scale))
var untrained: Combatant = ContentFactory.unit({"id": "t_ep_mage", "name": "Bare", "class_id": "mage", "level": 4})
untrained.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
var plain: Dictionary = DamageCalculator.attack_power(untrained)
var m_str: int = untrained.stats.get_effective_stat("STR", {})
var m_dex: int = untrained.stats.get_effective_stat("DEX", {})
checks["fist level < 2: plain bare hands, (STR+DEX) x power scale"] = untrained.stats.proficiency.level_of("fist") < DamageCalculator.TRAINED_FIST_MIN_LEVEL \
	and not DictUtil.to_bool(plain["trained_fists"]) and DictUtil.to_float(plain["weapon_power"]) == 0.0 \
	and DictUtil.to_int(plain["attack_power"]) == maxi(1, roundi(float(m_str + m_dex) * untrained.power_scale))
checks["DamageCalculator.compute is the formula (damage > 0)"] = int(DamageCalculator.compute(fighter, ContentFactory.unit({"id": "t_ep_dummy", "name": "Dummy", "class_id": "warrior", "level": 1}), false, false, 1.0, false, null)["final_dmg"]) > 0
# GearBudget: an empty main hand counts UNARMED_VALUE.
checks["UNARMED_VALUE sits within a one-handed weapon's range"] = GearBudget.UNARMED_VALUE > 0.0 and GearBudget.UNARMED_VALUE < GearBudget.TARGET_BUDGET
var armed: Dictionary = ContentCatalog.get_entry("classes", "brawler").duplicate(true)
armed["main_hand"] = fixture_weapon
checks["fixture weapon exists"] = ContentCatalog.has("items", fixture_weapon)
var bare: Dictionary = armed.duplicate(true)
bare["main_hand"] = ""
var armed_total: float = GearBudget.loadout_value(armed)
var main_item: float = GearBudget.item_value(ContentCatalog.get_entry("items", str(armed["main_hand"])))
checks["bare loadout = armed loadout - main weapon + UNARMED_VALUE"] = absf(GearBudget.loadout_value(bare) - (armed_total - main_item + GearBudget.UNARMED_VALUE)) < 0.001
var rows: Array[Dictionary] = GearBudget.loadout_breakdown(bare)
var has_row: bool = false
for r: Dictionary in rows:
	has_row = has_row or str(r["name"]) == "Unarmed"
checks["breakdown lists an 'Unarmed' row"] = has_row
# Not read as under budget just for lacking a weapon: it counts like a weapon of UNARMED_VALUE, so an
# otherwise-equal loadout is only as far from the band as that value implies.
checks["a class with no gear at all stays unjudged (no phantom row)"] = GearBudget.loadout_breakdown({"id": "t_empty"}).is_empty() and GearBudget.check_class({"id": "t_empty"}).is_empty()
return TestHelpers.report(TAG, "Unarmed damage and gear budget", checks)
```

### `static _test_shipped_gear_warnings() -> bool`
```logic
var checks: Dictionary = {}
var all_clear: bool = true
for entry: Dictionary in ContentCatalog.entries("classes"):
	all_clear = all_clear and GearBudget.check_class(entry).is_empty()
checks["warnings for the shipped classes stay empty"] = all_clear
return TestHelpers.report(TAG, "Shipped classes: no gear budget warnings", checks)
```

