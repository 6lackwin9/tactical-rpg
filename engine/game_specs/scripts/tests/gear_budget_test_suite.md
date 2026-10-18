---
class: GearBudgetTestSuite
extends: RefCounted
description: "Tactical RPG - Gear Budget Test Suite (milestone M2d) GearBudget weights, burden / block-cost penalties, loadout sums (pouch ignored), special items reported separately, soft warnings (never errors) from ContentCatalog, the special schema fields round-trip, and the Forge preview budget lines. Expectations are computed from GearBudget's constants. Works on a sandbox copy of the catalog."
---

# Properties
* `TAG`: String = "GearBudget" (const)
* `M2B_RANGES`: Dictionary = { (const)
* `M2B_SPECIAL`: Array[String] = ["wpn_glass_wand"] (const)
* `M2B_CLOTH`: Array[String] = ["arm_robe", "arm_cleric_vestment", "arm_mage_robe"] (const)

# Methods

### `static _near(a: float, b: float) -> bool`
```logic
return absf(a - b) < 0.001
```

### `static _gear(id: String, extra: Dictionary) -> Dictionary`
```logic
var e: Dictionary = {"id": id, "name": id, "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 10}
e.merge(extra, true)
return ContentSchema.normalize("items", e)
```

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING GEAR BUDGET TEST (M2d)                  ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = true
ok = _test_item_weights() and ok
if not TestHelpers.fork(SANDBOX):
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "sandbox fork", {"forked": false})
ok = _test_shipped() and ok
ok = _test_m2b_items() and ok
ok = _test_loadouts() and ok
ok = _test_warnings() and ok
ok = _test_schema_round_trip() and ok
ok = _test_forge_preview() and ok
TestHelpers.cleanup(SANDBOX)
return ok
```

### `static _test_item_weights() -> bool`
```logic
var c: Dictionary = {}
var full: Dictionary = _gear("t_full", {"base_power": 12, "mitigation": 6, "stat_bonuses": {"STR": 2, "DEX": 1}, "burden": 3, "block_cost": 3})
var expected: float = 12 * GearBudget.W_POWER + 6 * GearBudget.W_MITIGATION + 3 * GearBudget.W_STAT_POINT + 3 * GearBudget.W_BURDEN + 2 * GearBudget.W_EXTRA_BLOCK
c["weighted sum"] = _near(GearBudget.item_value(full), expected)
var plain: Dictionary = _gear("t_plain", {"base_power": 12})
c["plain = power only"] = _near(GearBudget.item_value(plain), 12 * GearBudget.W_POWER)
c["burden lowers value"] = _near(GearBudget.item_value(_gear("t_b", {"base_power": 12, "burden": 2})), GearBudget.item_value(plain) + 2 * GearBudget.W_BURDEN)
c["burden penalty is negative weight"] = GearBudget.W_BURDEN < 0.0
c["extra block cost lowers value"] = _near(GearBudget.item_value(_gear("t_c", {"base_power": 12, "block_cost": 2})), GearBudget.item_value(plain) + GearBudget.W_EXTRA_BLOCK)
c["base block cost 1 is free"] = _near(GearBudget.item_value(_gear("t_d", {"base_power": 12, "block_cost": 1})), GearBudget.item_value(plain))
c["mitigation weighted"] = _near(GearBudget.item_value(_gear("t_m", {"item_type": "ARMOR", "slot": "BODY", "base_power": 0, "mitigation": 8})), 8 * GearBudget.W_MITIGATION)
var potion: Dictionary = ContentSchema.normalize("items", {"id": "t_p", "item_type": "CONSUMABLE", "heal": 20})
c["consumables are worth 0"] = _near(GearBudget.item_value(potion), 0.0)
c["band brackets target"] = GearBudget.band_low() < GearBudget.TARGET_BUDGET and GearBudget.band_high() > GearBudget.TARGET_BUDGET
return TestHelpers.report(TAG, "item weights and penalties", c)
```

### `static _test_shipped() -> bool`
```logic
var c: Dictionary = {}
c["validate_all stays empty (warnings are separate)"] = ContentCatalog.validate_all().is_empty()
var passing: int = 0
for e: Dictionary in ContentCatalog.entries("classes"):
	if GearBudget.in_band(GearBudget.loadout_value(e)):
		passing += 1
	print("      class %s loadout %.1f / %.1f%s" % [str(e["id"]), GearBudget.loadout_value(e), GearBudget.TARGET_BUDGET, "" if GearBudget.in_band(GearBudget.loadout_value(e)) else " OUT OF BAND"])
c["most shipped classes in band"] = passing * 2 > ContentCatalog.entries("classes").size()
return TestHelpers.report(TAG, "shipped catalog", c)
```

### `static _test_m2b_items() -> bool`
```logic
var c: Dictionary = {}
var all_ids: Array[String] = M2B_SPECIAL.duplicate()
for id: Variant in M2B_RANGES:
	all_ids.append(str(id))
var present: bool = true
var in_range: bool = true
var not_stash: bool = true
var no_warn: bool = true
for id: String in all_ids:
	if not ContentCatalog.has("items", id):
		present = false
		continue
	var e: Dictionary = ContentCatalog.get_entry("items", id)
	not_stash = not_stash and not DictUtil.to_bool(e["starter_stash"])
	no_warn = no_warn and ContentCatalog.warnings_for("items", e).is_empty()
	if M2B_RANGES.has(id):
		var r: Array = M2B_RANGES[id]
		var v: float = GearBudget.item_value(e)
		print("      m2b %s value %.1f (range %.0f-%.0f)" % [id, v, DictUtil.to_float(r[0]), DictUtil.to_float(r[1])])
		in_range = in_range and v >= DictUtil.to_float(r[0]) and v <= DictUtil.to_float(r[1]) and (id == "shd_tower" or not GearBudget.is_special(e))
c["all M2b items present"] = present
c["values inside slot ranges, non-special"] = in_range
c["not in the starter stash"] = not_stash
c["no item warnings"] = no_warn
c["validate_all stays empty"] = ContentCatalog.validate_all().is_empty()
var special_ok: bool = true
for id: String in M2B_SPECIAL:
	var e: Dictionary = ContentCatalog.get_entry("items", id)
	special_ok = special_ok and GearBudget.is_special(e) and not str(e["special_note"]).strip_edges().is_empty()
	special_ok = special_ok and GearBudget.item_value(e) > 20.0
c["special items carry a note"] = special_ok
# Tower shield trades its mitigation for burden, block cost and a STR requirement versus the kite shield.
var tower: Dictionary = ContentCatalog.get_entry("items", "shd_tower")
var kite: Dictionary = ContentCatalog.get_entry("items", "shd_kite")
c["tower shield not strictly better than kite"] = DictUtil.to_int(tower["mitigation"]) > DictUtil.to_int(kite["mitigation"]) and (DictUtil.to_int(tower["burden"]) > DictUtil.to_int(kite["burden"]) or DictUtil.to_int(tower["block_cost"]) > DictUtil.to_int(kite["block_cost"]))
c["tower shield is a documented special (M2c-1 tank warrior)"] = GearBudget.is_special(tower) and not str(tower["special_note"]).strip_edges().is_empty()
# Cloth armor is gated by the proficiency dict: the warrior cannot wear it, a cloth-trained class can.
var warrior: ClassArchetype = ClassArchetype.create("warrior")
var cloth_user: ClassArchetype = null
for ce: Dictionary in ContentCatalog.entries("classes"):
	var arch: ClassArchetype = ContentFactory.class_archetype(ce)
	var probe: Equipment = ContentFactory.item(ContentCatalog.get_entry("items", "arm_robe")) as Equipment
	if probe != null and arch.can_equip_gear(probe) and cloth_user == null:
		cloth_user = arch
var cloth_gate: bool = cloth_user != null
for id: String in M2B_CLOTH:
	var eq: Equipment = ContentFactory.item(ContentCatalog.get_entry("items", id)) as Equipment
	cloth_gate = cloth_gate and eq != null and eq.weapon_type == "cloth" and not warrior.can_equip_gear(eq) and cloth_user != null and cloth_user.can_equip_gear(eq)
c["cloth armor needs cloth proficiency"] = cloth_gate
# Weapons: wand ranged one-handed INT, mage staff two-handed INT, cleric mace melee one-handed.
var wand: Equipment = ContentFactory.item(ContentCatalog.get_entry("items", "wpn_wand")) as Equipment
var mstaff: Equipment = ContentFactory.item(ContentCatalog.get_entry("items", "wpn_mage_staff")) as Equipment
var cmace: Equipment = ContentFactory.item(ContentCatalog.get_entry("items", "wpn_cleric_mace")) as Equipment
c["weapon shapes"] = wand != null and mstaff != null and cmace != null and wand.weapon_type == "wand" and not wand.is_two_handed and wand.base_range >= 2 and mstaff.is_two_handed and cmace.base_range == 1 and not cmace.is_two_handed
c["icons resolve per gear type"] = wand != null and mstaff != null and ItemArt.icon_key(wand) == "wand" and ItemArt.icon_key(mstaff) == "staff" and ItemArt.icon_key(ContentFactory.item(ContentCatalog.get_entry("items", "arm_robe"))) == "robe" and ItemArt.icon_key(ContentFactory.item(tower)) == "shield" and ItemArt.icon(wand) != null
# Placeholder art keys render for every team.
var art_ok: bool = true
for key: String in ["priest", "mage", "rogue"]:
	art_ok = art_ok and UnitArt.CLASS_KEYS.has(key) and UnitArt.normalize_class(key) == key
	for team: int in [UnitArt.Team.FRIENDLY, UnitArt.Team.HOSTILE, UnitArt.Team.NEUTRAL]:
		art_ok = art_ok and UnitArt.token(key, team as UnitArt.Team) != null and UnitArt.portrait(key, team as UnitArt.Team) != null
c["priest/mage/rogue art renders"] = art_ok
return TestHelpers.report(TAG, "M2b items and art", c)
```

### `static _seed_items() -> void`
```logic
ContentCatalog.upsert("items", _gear("t_sword", {"base_power": 14}))
ContentCatalog.upsert("items", _gear("t_plate", {"item_type": "ARMOR", "slot": "BODY", "weapon_type": "plate", "base_power": 0, "mitigation": 6, "burden": 2}))
ContentCatalog.upsert("items", _gear("t_ring", {"item_type": "ACCESSORY", "slot": "ACCESSORY_1", "weapon_type": "", "base_power": 0, "stat_bonuses": {"STR": 2}}))
ContentCatalog.upsert("items", _gear("t_god", {"base_power": 80, "special": true, "special_note": "Huge power but two block cost.", "block_cost": 2}))
ContentCatalog.upsert("items", _gear("t_god_bare", {"base_power": 80, "special": true}))
ContentCatalog.upsert("items", ContentSchema.normalize("items", {"id": "t_potion", "item_type": "CONSUMABLE", "heal": 20}))
```

### `static _class_with(slots: Dictionary) -> Dictionary`
```logic
var e: Dictionary = ContentCatalog.get_entry("classes", "warrior").duplicate(true)
for slot: String in ContentSchema.LOADOUT_SLOTS:
	e[slot] = ""
e["pouch"] = []
e.merge(slots, true)
return e
```

### `static _test_loadouts() -> bool`
```logic
_seed_items()
var c: Dictionary = {}
var cls: Dictionary = _class_with({"main_hand": "t_sword", "body": "t_plate", "accessory_1": "t_ring", "pouch": ["t_potion"]})
var expected: float = 14 * GearBudget.W_POWER + (6 * GearBudget.W_MITIGATION + 2 * GearBudget.W_BURDEN) + 2 * GearBudget.W_STAT_POINT
c["loadout = sum of slots, pouch ignored"] = _near(GearBudget.loadout_value(cls), expected)
c["breakdown lists 3 slots"] = GearBudget.loadout_breakdown(cls).size() == 3
c["no special reported"] = _near(GearBudget.special_value(cls), 0.0)
var with_special: Dictionary = _class_with({"main_hand": "t_god", "body": "t_plate"})
var god_value: float = 80 * GearBudget.W_POWER + GearBudget.W_EXTRA_BLOCK
c["special value reported separately"] = _near(GearBudget.special_value(with_special), god_value)
c["total includes special"] = _near(GearBudget.loadout_value(with_special), god_value + GearBudget.item_value(ContentCatalog.get_entry("items", "t_plate")))
var flagged: int = 0
for row: Dictionary in GearBudget.loadout_breakdown(with_special):
	if DictUtil.to_bool(row["special"]):
		flagged += 1
c["exactly one special row"] = flagged == 1
return TestHelpers.report(TAG, "loadout sums and special reporting", c)
```

### `static _test_warnings() -> bool`
```logic
var c: Dictionary = {}
var cheap: Dictionary = _class_with({"main_hand": "t_ring"})
cheap["id"] = "t_cheap_class"
var mid: Dictionary = _class_with({"main_hand": "t_sword", "body": "t_plate", "accessory_1": "t_ring"})
# Make a clearly over-budget loadout by pairing the bare special (no documented note).
var over: Dictionary = _class_with({"main_hand": "t_god_bare", "body": "t_plate"})
var over_warn: Array[String] = _class_warnings(over)
c["over band warns"] = over_warn.size() == 1
c["warning names value and budget"] = over_warn.size() == 1 and over_warn[0].contains("%.1f" % GearBudget.loadout_value(over)) and over_warn[0].contains("%.1f" % GearBudget.TARGET_BUDGET)
c["warning names top item"] = over_warn.size() == 1 and over_warn[0].contains("t_god_bare")
c["over band is not an error"] = ContentCatalog.validate_entry("classes", over, "warrior").filter(func(m: String) -> bool: return m.contains("Gear budget")).is_empty()
var under_warn: Array[String] = _class_warnings(cheap)
c["under band warns"] = under_warn.size() == 1 and under_warn[0].contains("below")
var documented: Dictionary = _class_with({"main_hand": "t_god", "body": "t_plate"})
c["documented special may exceed band"] = _class_warnings(documented).is_empty()
c["special without note warns"] = ContentCatalog.warnings_for("items", ContentCatalog.get_entry("items", "t_god_bare")).size() == 1
c["special with note is clean"] = ContentCatalog.warnings_for("items", ContentCatalog.get_entry("items", "t_god")).is_empty()
c["normal item is clean"] = ContentCatalog.warnings_for("items", ContentCatalog.get_entry("items", "t_sword")).is_empty()
c["in-band class is clean"] = GearBudget.in_band(GearBudget.loadout_value(mid)) and _class_warnings(mid).is_empty()
var all_w: Dictionary = ContentCatalog.warnings_all()
c["warnings_all lists the bare special"] = all_w.has("items:t_god_bare")
c["validate_all still empty"] = ContentCatalog.validate_all().is_empty()
return TestHelpers.report(TAG, "soft warnings", c)
```

### `static _test_schema_round_trip() -> bool`
```logic
var c: Dictionary = {}
var entry: Dictionary = ContentCatalog.get_entry("items", "t_god")
var rebuilt: Dictionary = ContentFactory.item_to_entry(ContentFactory.item(entry))
c["special survives"] = DictUtil.to_bool(rebuilt["special"])
c["note survives"] = str(rebuilt["special_note"]) == str(entry["special_note"])
var plain: Dictionary = ContentFactory.item_to_entry(ContentFactory.item(ContentCatalog.get_entry("items", "t_sword")))
c["defaults false / empty"] = not DictUtil.to_bool(plain["special"]) and str(plain["special_note"]).is_empty()
var shown_for_consumable: int = 0
var shown_for_weapon: int = 0
for f: Dictionary in ContentSchema.fields("items"):
	if str(f["key"]) == "special" or str(f["key"]) == "special_note":
		if ContentSchema.is_visible(f, {"item_type": "CONSUMABLE"}):
			shown_for_consumable += 1
		if ContentSchema.is_visible(f, {"item_type": "WEAPON"}):
			shown_for_weapon += 1
c["special fields only for equipment"] = shown_for_consumable == 0 and shown_for_weapon == 2
return TestHelpers.report(TAG, "schema round trip", c)
```

### `static _test_forge_preview() -> bool`
```logic
var c: Dictionary = {}
var cls: Dictionary = _class_with({"main_hand": "t_god", "body": "t_plate"})
var text: String = ForgePreview.describe("classes", cls)
c["class preview has budget line"] = text.contains("Gear budget")
c["class preview has target"] = text.contains("%.1f" % GearBudget.TARGET_BUDGET)
c["class preview shows special items"] = text.contains("Special items")
var over: Dictionary = _class_with({"main_hand": "t_god_bare", "body": "t_plate"})
c["out of band marked"] = ForgePreview.describe("classes", over).contains("out of band")
var item_text: String = ForgePreview.describe("items", ContentCatalog.get_entry("items", "t_god"))
c["item preview has value"] = item_text.contains("Gear value") and item_text.contains("%.1f" % GearBudget.item_value(ContentCatalog.get_entry("items", "t_god")))
c["item preview shows special"] = item_text.contains("SPECIAL")
return TestHelpers.report(TAG, "forge preview", c)
```

