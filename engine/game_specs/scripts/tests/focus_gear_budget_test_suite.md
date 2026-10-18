---
class: FocusGearBudgetTestSuite
extends: RefCounted
description: "Tactical RPG - Focus gear: GearBudget effect weights and the shipped C1 assets (C1-L; split from FocusGearTestSuite by CLN-12). Same sandbox fixtures (FocusGearTestSuite.seed_fixtures); expectations come from the fixture entries and the GearBudget weights, never from shipped numbers."
---

# Properties
* `TAG`: String = "FocusGear" (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING FOCUS GEAR BUDGET TEST (C1-L)           ===")
print("=======================================================")
if not TestHelpers.fork(SANDBOX):
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "sandbox fork", {"forked": false})
FocusGearTestSuite.seed_fixtures()
var ok: bool = true
ok = _test_gear_budget() and ok
ok = _test_shipped_assets() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _formula(entry: Dictionary) -> float`
```logic
var e: Dictionary = ContentSchema.normalize("items", entry)
var v: float = DictUtil.to_int(e["base_power"]) * GearBudget.W_POWER + DictUtil.to_int(e["mitigation"]) * GearBudget.W_MITIGATION
var bonuses: Dictionary = DictUtil.dict_of(e["stat_bonuses"])
for key: Variant in bonuses:
	v += DictUtil.to_int(bonuses[key]) * GearBudget.W_STAT_POINT
v += DictUtil.to_int(e["burden"]) * GearBudget.W_BURDEN + maxi(0, DictUtil.to_int(e["block_cost"]) - 1) * GearBudget.W_EXTRA_BLOCK
for raw: Variant in DictUtil.array_of(e["effects"]):
	var fx: Dictionary = DictUtil.dict_of(raw)
	var value: float = DictUtil.to_float(fx.get("value", 0.0))
	match str(fx.get("type", "")):
		"skill_potency":
			v += value * 20.0
		"status_potency":
			v += value * 0.2
		"spell_reach":
			v += value * 3.0
return v
```

### `static _test_gear_budget() -> bool`
```logic
var c: Dictionary = {}
c["weights are the spec's"] = FocusGearTestSuite.near(GearBudget.W_SKILL_POTENCY, 20.0) and FocusGearTestSuite.near(GearBudget.W_STATUS_POTENCY, 0.2) and FocusGearTestSuite.near(GearBudget.W_SPELL_REACH, 3.0)
for id: String in ["t_orb", "t_tome", "t_reach_ring"]:
	var entry: Dictionary = ContentCatalog.get_entry("items", id)
	c["%s value = formula" % id] = FocusGearTestSuite.near(GearBudget.item_value(entry), _formula(entry))
var bare: Dictionary = FocusGearTestSuite.focus("t_bare", "orb")
c["an item without effects is unchanged"] = FocusGearTestSuite.near(GearBudget.item_value(bare), 0.0)
c["+0.10 potency counts 2.0"] = FocusGearTestSuite.near(GearBudget.item_value(FocusGearTestSuite.focus("t_pot", "orb", {"effects": [{"type": "skill_potency", "group": "spell", "value": 0.10}]})), 2.0)
for id: String in ["wpn_orb", "wpn_tome"]:
	if ContentCatalog.has("items", id):
		var shipped: Dictionary = ContentCatalog.get_entry("items", id)
		c["%s shipped value = formula" % id] = FocusGearTestSuite.near(GearBudget.item_value(shipped), _formula(shipped))
var loadouts_ok: bool = true
for class_id: String in ["mage", "priest"]:
	loadouts_ok = loadouts_ok and GearBudget.check_class(ContentCatalog.get_entry("classes", class_id)).is_empty()
c["mage and priest default loadouts stay inside the band"] = loadouts_ok
return TestHelpers.report(TAG, "GearBudget effect weights", c)
```

### `static _test_shipped_assets() -> bool`
```logic
var c: Dictionary = {}
var priest: Dictionary = ContentCatalog.get_entry("classes", "priest")
var mage: Dictionary = ContentCatalog.get_entry("classes", "mage")
var priest_prof: Dictionary = DictUtil.dict_of(priest.get("proficiency", {}))
if not priest_prof.has("tome"):
	print("    [FocusGear] shipped C1 assets: SKIP (priest tome proficiency not in the catalog yet; C1-A1 pending)")
	return true
c["priest has no shield proficiency"] = not priest_prof.has("shield")
c["priest tome level 3, orb level 2"] = DictUtil.to_int(DictUtil.dict_of(priest_prof["tome"]).get("level", 0)) == 3 and DictUtil.to_int(DictUtil.dict_of(priest_prof.get("orb", {})).get("level", 0)) == 2
var mage_prof: Dictionary = DictUtil.dict_of(mage.get("proficiency", {}))
c["mage orb 3, tome 2"] = DictUtil.to_int(DictUtil.dict_of(mage_prof.get("orb", {})).get("level", 0)) == 3 and DictUtil.to_int(DictUtil.dict_of(mage_prof.get("tome", {})).get("level", 0)) == 2
var bad_unit: Array[String] = ContentCatalog.validate_entry("units", {"id": "t_shield_priest", "name": "x", "class_id": "priest", "off_hand": "shd_kite"})
c["a priest with a shield override fails the training check"] = FocusGearTestSuite.has_error(bad_unit, "no training in 'shield'")
for skill_id: String in ["arcane_bolt", "hex"]:
	var skill: SkillDef = SkillCatalog.find(skill_id)
	c["%s no longer names orb or tome" % skill_id] = skill != null and not skill.weapon_types.has("orb") and not skill.weapon_types.has("tome")
return TestHelpers.report(TAG, "shipped C1 assets", c)
```

