---
class: ClassPathTestSuite
extends: RefCounted
description: "Tactical RPG - Class Path & Class Rules Test Suite (CLASS1-L commit 1, FORGE1_CLASSES 3-5) The class skill path is a dict {family: unlock level}: shape and one-release list converter, validation (basics, tier, level range, grade floor, every `requires` at an equal or lower level), soft warnings (path longer than the active slots, tier beyond the cap, no level-1 entry, unreachable weapon gate, role mismatches), the duplicate-accessory flag, the unit `heal_below` AI field, the class key-field locks and delete rules, and the CLASS1-M migration seed run on a list-shaped sandbox fork. Fixtures are test-only assets in sandboxes; shipped content is only read, picked by rule. Ids a demote must not find in this file are built by concatenation."
---

# Properties
* `TAG`: String = "ClassPath" (const)
* `SEED`: String = "res://data/seeds/class1_migrate.gd" (const)
* `out`: Array[int] = list.duplicate()
* `out`: Array[String] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING CLASS PATH TEST (CLASS1-L)              ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = true
ok = _test_shape() and ok
ok = _test_converter() and ok
ok = _test_validation() and ok
ok = _test_warnings() and ok
ok = _test_accessory_flag() and ok
ok = _test_heal_below() and ok
ok = _test_locks() and ok
TestHelpers.cleanup(SANDBOX)
ok = _test_migration() and ok
TestHelpers.cleanup(MIGRATE_SANDBOX)
return ok
```

### `static _class(id: String, path: Dictionary, extra: Dictionary = {}) -> Dictionary`
```logic
var e: Dictionary = ContentCatalog.get_entry("classes", ContentCatalog.ids("classes")[0])
e["id"] = id
e["name"] = "Fixture"
e["skill_path"] = path
e.erase("origin")
e.merge(extra, true)
return e
```

### `static _build_fixtures(gate_type: String) -> bool`
```logic
var ok: bool = true
for i: int in range(1, 9):
	ok = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_f%d" % i)).is_empty() and ok
ok = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_a")).is_empty() and ok
ok = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_b")).is_empty() and ok
ok = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_adv", {"grade": "advanced", "requires": ["t_cp_a", "t_cp_b"]})).is_empty() and ok
ok = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_gate", {"weapon_types": [gate_type], "min_proficiency": 1})).is_empty() and ok
ok = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_fist", {"weapon_types": [SkillDef.UNARMED_TOKEN], "min_proficiency": 1})).is_empty() and ok
return ok
```

### `static _untrained_weapon() -> String`
```logic
var prof: Dictionary = DictUtil.dict_of(ContentCatalog.get_entry("classes", ContentCatalog.ids("classes")[0])["proficiency"])
for gear: String in Proficiency.WEAPON_TYPES:
	if not prof.has(gear) and gear != SkillDef.UNARMED_PROFICIENCY:
		return gear
return ""
```

### `static _test_shape() -> bool`
```logic
var checks: Dictionary = {}
var dicts: bool = true
var no_basic: bool = true
var heads: bool = true
var level_one: bool = true
var round_trip: bool = true
var steps_sorted: bool = true
for e: Dictionary in ContentCatalog.entries("classes"):
	var path: Variant = e["skill_path"]
	dicts = dicts and path is Dictionary
	var arch: ClassArchetype = ContentFactory.class_archetype(e)
	for fam: Variant in DictUtil.dict_of(path):
		var head: Dictionary = ContentCatalog.get_entry("skills", str(fam))
		no_basic = no_basic and not DictUtil.to_bool(head.get("basic", false))
		heads = heads and not head.is_empty() and DictUtil.to_int(head.get("tier", 1), 1) == 1
	level_one = level_one and not arch.skills_at(1).is_empty()
	var back: Dictionary = ContentFactory.class_to_entry(arch)
	round_trip = round_trip and DictUtil.dict_of(back["skill_path"]).keys() == DictUtil.dict_of(path).keys() and DictUtil.dict_of(back["skill_path"]) == DictUtil.dict_of(path)
	var steps: Array[Dictionary] = arch.path_steps()
	for i: int in range(1, steps.size()):
		steps_sorted = steps_sorted and DictUtil.to_int(steps[i - 1]["level"]) <= DictUtil.to_int(steps[i]["level"])
checks["every shipped path is a dict in memory"] = dicts
checks["every shipped path holds no basic skill"] = no_basic
checks["every key is a known tier-1 head"] = heads
checks["skills_at(1) is non-empty for every shipped class"] = level_one
checks["class_to_entry keeps the dict and its key order"] = round_trip
checks["path_steps is sorted by unlock level"] = steps_sorted
checks["STANDARD_PATH_LEVELS is ascending within 1..MAX_LEVEL"] = ClassArchetype.STANDARD_PATH_LEVELS.all(func(l: int) -> bool: return l >= 1 and l <= StatRules.MAX_LEVEL) and ClassArchetype.STANDARD_PATH_LEVELS == _sorted(ClassArchetype.STANDARD_PATH_LEVELS)
var schema: Dictionary = ContentSchema.field("classes", "skill_path")
checks["the schema field is a table 1..MAX_LEVEL with an empty dict default"] = str(schema["type"]) == "table" and DictUtil.to_int(schema["min"]) == 1 and DictUtil.to_int(schema["max"]) == StatRules.MAX_LEVEL and DictUtil.dict_of(schema["default"]).is_empty()
return TestHelpers.report(TAG, "Shipped paths are dicts of tier-1 heads; order survives the factory", checks)
```

### `static _test_converter() -> bool`
```logic
var checks: Dictionary = {}
var heads: Array[String] = _plain_heads(2)
var basics: Array[String] = SkillCatalog.basic_ids()
checks["two plain shipped heads and a basic exist"] = heads.size() == 2 and not basics.is_empty()
if heads.size() < 2 or basics.is_empty():
	return TestHelpers.report(TAG, "List converter", checks)
var listed: Array = [{"level": 1, "skill": basics[0]}, {"level": 3, "skill": heads[1]}, {"level": 1, "skill": heads[0]}, {"level": 5, "skill": heads[1]}]
var saved: bool = ClassUnitFields.list_path_converter
ClassUnitFields.list_path_converter = true
var e: Dictionary = ContentSchema.normalize("classes", {"id": "t_cp_list", "name": "L", "skill_path": listed})
var path: Dictionary = DictUtil.dict_of(e["skill_path"])
checks["a list converts to a dict"] = e["skill_path"] is Dictionary
checks["...basics dropped"] = not path.has(basics[0])
checks["...list order kept (first position wins)"] = path.keys() == [heads[1], heads[0]]
checks["...a repeated family keeps its lowest level"] = DictUtil.to_int(path.get(heads[1], 0)) == 3 and DictUtil.to_int(path.get(heads[0], 0)) == 1
checks["JSON floats become whole levels"] = DictUtil.dict_of(ContentSchema.normalize("classes", {"skill_path": {heads[0]: 3.0}})["skill_path"])[heads[0]] is int
ClassUnitFields.list_path_converter = false
var raw: Dictionary = ContentSchema.normalize("classes", {"id": "t_cp_list", "name": "L", "skill_path": listed})
checks["converter off: the list is kept (never dropped)"] = raw["skill_path"] is Array
checks["converter off: a list is a validation error"] = TestHelpers.any_contains(_errors({"id": "t_cp_list", "name": "L", "skill_path": listed}), "must be a dictionary")
ClassUnitFields.list_path_converter = saved
checks["the converter is off since CLASS1-M migrated the shipped data"] = not ClassUnitFields.LIST_PATH_CONVERTER and not ClassUnitFields.list_path_converter
return TestHelpers.report(TAG, "One-release list converter (FORGE1_CLASSES 3.5)", checks)
```

### `static _test_validation() -> bool`
```logic
var checks: Dictionary = {}
checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
var gate: String = _untrained_weapon()
checks["fixtures"] = not gate.is_empty() and _build_fixtures(gate)
var adv_min: int = StatRules.ADVANCED_MIN_LEVEL
var basic: String = SkillCatalog.basic_ids()[0]
checks["a valid two-entry path validates"] = _errors(_class("t_cp_ok", {"t_cp_a": 1, "t_cp_b": 3})).is_empty()
checks["a basic in the path is an error naming it"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {basic: 1})), basic)
checks["an unknown family is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_nope": 1})), "unknown skill")
checks["a level outside 1..MAX_LEVEL is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_a": 0})), "outside") and TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_a": StatRules.MAX_LEVEL + 1})), "outside")
checks["a non-whole level is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_a": 2.5})), "whole number")
checks["a higher-tier key is an error"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_r", {"ranks": [{"level": 3, "power_mult": 1.2}]})).is_empty() and TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_r_2": 3})), "higher tier")
checks["grade advanced below the floor is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_a": 1, "t_cp_b": 1, "t_cp_adv": adv_min - 1})), "advanced")
checks["requires: none on the path is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_adv": adv_min})), "t_cp_a")
checks["requires: only one of two (ALL needed) is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_a": 1, "t_cp_adv": adv_min})), "t_cp_b")
checks["requires: one listed later is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {"t_cp_a": 1, "t_cp_b": adv_min + 1, "t_cp_adv": adv_min})), "only listed at level")
checks["requires: all at a lower or equal level validate"] = _errors(_class("t_cp_x", {"t_cp_a": 1, "t_cp_b": adv_min, "t_cp_adv": adv_min})).is_empty()
var eight: Dictionary = {}
for i: int in range(1, 9):
	eight["t_cp_f%d" % i] = 1 if i == 1 else ClassArchetype.STANDARD_PATH_LEVELS[mini(i, ClassArchetype.STANDARD_PATH_LEVELS.size() - 1)]
checks["8 families validate (no path length limit)"] = _errors(_class("t_cp_x", eight)).is_empty()
var arch: ClassArchetype = ContentFactory.class_archetype(_class("t_cp_x", eight))
checks["...MAX_CLASS_SLOTS of them are active by default"] = ContentFactory._strings(ContentFactory.loadout_of({}, arch, StatRules.MAX_LEVEL)["class"]).size() == StatRules.MAX_CLASS_SLOTS
checks["id caps: 1 character and 25 characters are errors"] = TestHelpers.any_contains(_errors(_class("t", {})), "min") and TestHelpers.any_contains(_errors(_class("t_cp_" + "x".repeat(ClassUnitFields.ID_MAX_LEN), {})), "max")
checks["name cap is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {}, {"name": "N".repeat(ClassUnitFields.NAME_MAX_LEN + 1)})), "max %d" % ClassUnitFields.NAME_MAX_LEN)
checks["description cap is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {}, {"description": "d".repeat(ClassUnitFields.DESCRIPTION_MAX_LEN + 1)})), "max %d" % ClassUnitFields.DESCRIPTION_MAX_LEN)
checks["an unknown ai_role is an error"] = TestHelpers.any_contains(_errors(_class("t_cp_x", {}, {"ai_role": "TANK"})), "TANK")
# Renaming a path family (upsert_skill with replace_id) rewrites the class path key in place.
var ren_ok: bool = ContentCatalog.upsert("classes", _class("t_cp_ren", {"t_cp_f1": 1, "t_cp_f2": 3, "t_cp_f3": 4})).is_empty()
var renamed: Dictionary = ContentCatalog.skill_entry("t_cp_f2")
renamed["id"] = "t_cp_f2b"
ren_ok = ren_ok and ContentCatalog.upsert_skill(renamed, "t_cp_f2").is_empty()
var ren_path: Dictionary = DictUtil.dict_of(ContentCatalog.get_entry("classes", "t_cp_ren")["skill_path"])
checks["renaming a family rewrites the path key at the same position and level"] = ren_ok and ren_path.keys() == ["t_cp_f1", "t_cp_f2b", "t_cp_f3"] and DictUtil.to_int(ren_path["t_cp_f2b"]) == 3
checks["...and references_to finds the class through the path"] = ContentCatalog.references_to("skills", "t_cp_f2b").has("classes:t_cp_ren")
# A rename onto a key the path already holds is refused (never a silent stale key). Only reachable with a file that
# loaded an unvalidated key, so the fork's classes.json gets one by hand.
checks["save before the hand edit"] = ContentCatalog.save_all() == OK
var doc: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(ContentCatalog.path_for("classes"))))
for raw: Variant in DictUtil.array_of(doc.get("entries", [])):
	var row: Dictionary = DictUtil.dict_of(raw)
	if str(row.get("id", "")) == "t_cp_ren":
		var hand_path: Dictionary = DictUtil.dict_of(row["skill_path"])
		hand_path["t_cp_ghost"] = 4
var file := FileAccess.open(ContentCatalog.path_for("classes"), FileAccess.WRITE)
file.store_string(JSON.stringify(doc, "\t", false) + "\n")
file.close()
ContentCatalog.reload_from_disk()
checks["the hand-edited path holds both keys"] = DictUtil.dict_of(ContentCatalog.get_entry("classes", "t_cp_ren")["skill_path"]).has("t_cp_ghost")
var onto: Dictionary = ContentCatalog.skill_entry("t_cp_f1")
onto["id"] = "t_cp_ghost"
checks["upsert_skill renaming onto an existing path key is refused"] = TestHelpers.any_contains(ContentCatalog.upsert_skill(onto, "t_cp_f1"), "lists both")
var flat: Dictionary = ContentCatalog.get_entry("skills", "t_cp_f1")
flat["id"] = "t_cp_ghost"
checks["flat upsert renaming onto an existing path key is refused"] = TestHelpers.any_contains(ContentCatalog.upsert("skills", flat, "t_cp_f1"), "lists both")
var kept: Dictionary = DictUtil.dict_of(ContentCatalog.get_entry("classes", "t_cp_ren")["skill_path"])
checks["...nothing changed (skill and path keys intact)"] = ContentCatalog.has("skills", "t_cp_f1") and kept.has("t_cp_f1") and kept.has("t_cp_ghost") and DictUtil.to_int(kept["t_cp_ghost"]) == 4
return TestHelpers.report(TAG, "Path validation: basics, tier, level, grade floor, requires (ALL, <= level), caps", checks)
```

### `static _test_warnings() -> bool`
```logic
var checks: Dictionary = {}
checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
var gate: String = _untrained_weapon()
checks["fixtures"] = not gate.is_empty() and _build_fixtures(gate)
var eight: Dictionary = {}
for i: int in range(1, 9):
	eight["t_cp_f%d" % i] = 1
var long_warn: Array[String] = _warnings(_class("t_cp_w", eight))
checks["more than MAX_CLASS_SLOTS families warns, naming 8 and 5"] = TestHelpers.any_contains(long_warn, "lists 8 families") and TestHelpers.any_contains(long_warn, "first %d" % StatRules.MAX_CLASS_SLOTS)
checks["...and is not an error"] = _errors(_class("t_cp_w", eight)).is_empty()
var five: Dictionary = {}
for i: int in range(1, StatRules.MAX_CLASS_SLOTS + 1):
	five["t_cp_f%d" % i] = 1
checks["exactly MAX_CLASS_SLOTS families does not warn about length"] = not TestHelpers.any_contains(_warnings(_class("t_cp_w", five)), "families; only")
checks["no level-1 entry warns"] = TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 3})), "level 1")
checks["a level-1 entry does not"] = not TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1})), "level 1")
var tiered: String = _three_tier_family()
checks["a shipped 3-tier novice family exists"] = not tiered.is_empty()
if not tiered.is_empty():
	var top: SkillDef = SkillCatalog.family_tiers(tiered).back()
	var late: int = StatRules.MAX_LEVEL - top.tier_offset() + 1
	checks["a tier unlocking past MAX_LEVEL warns (capped)"] = TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1, tiered: late})), "capped")
	checks["...not when it fits"] = not TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1, tiered: late - 1})), "capped")
checks["a weapon gate the class cannot train warns"] = TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1, "t_cp_gate": 1})), gate)
var fist_prof: Dictionary = {"sword": {"level": 1, "scaling": 1.5}, SkillDef.UNARMED_PROFICIENCY: {"level": 2, "scaling": 1.5}}
var bare: Dictionary = {"main_hand": "", "off_hand": "", "proficiency": fist_prof}
checks["an unarmed gate with fist trained does not warn"] = not TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1, "t_cp_fist": 1}, bare)), "trains none")
checks["SUPPORT without a SUPPORT path skill warns"] = TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1}, {"ai_role": "SUPPORT"})), "SUPPORT without")
checks["SNIPER with a melee main hand warns"] = TestHelpers.any_contains(_warnings(_class("t_cp_w", {"t_cp_f1": 1}, {"ai_role": "SNIPER", "main_hand": ""})), "SNIPER with a melee")
var support_unit: Dictionary = {"id": "t_cp_u", "name": "U", "class_id": ContentCatalog.ids("classes")[0], "ai_role": "SUPPORT", "reckless": true}
checks["a reckless SUPPORT unit warns"] = TestHelpers.any_contains(ContentCatalog.warnings_for("units", support_unit), "reckless SUPPORT")
var records: Array[Dictionary] = ClassUnitValidator.class_warnings(ContentSchema.normalize("classes", _class("t_cp_w", eight)))
var sections_ok: bool = not records.is_empty()
for r: Dictionary in records:
	sections_ok = sections_ok and ClassUnitFields.CLASS_GROUPS.has(str(r["section"])) and not str(r["text"]).is_empty()
checks["warnings come back as typed records {section, text} in the class groups"] = sections_ok
return TestHelpers.report(TAG, "Soft warnings: length, level 1, tier cap, weapon gate, roles", checks)
```

### `static _three_tier_family() -> String`
```logic
for fam: String in SkillCatalog.families():
	var tiers: Array[SkillDef] = SkillCatalog.family_tiers(fam)
	if tiers.size() >= 3 and not tiers[0].basic and not tiers[0].system and tiers[0].grade == SkillDef.Grade.NOVICE and tiers[0].requires.is_empty() and tiers.back().tier_offset() > 0:
		return fam
return ""
```

### `static _test_accessory_flag() -> bool`
```logic
var checks: Dictionary = {}
var ring: String = ""
for item: Dictionary in ContentCatalog.entries("items"):
	if str(item.get("item_type", "")) == "ACCESSORY" and ContentFactory.item_fits_slot(item, "ACCESSORY_1") and ContentFactory.item_fits_slot(item, "ACCESSORY_2"):
		ring = str(item["id"])
		break
checks["an accessory item exists"] = not ring.is_empty()
checks["the flag defaults to forbidden"] = not StatRules.ALLOW_DUPLICATE_ACCESSORY and not StatRules.allow_duplicate_accessory
var dup_class: Dictionary = _class("t_cp_acc", {}, {"accessory_1": ring, "accessory_2": ring})
var dup_unit: Dictionary = {"id": "t_cp_ua", "name": "U", "class_id": ContentCatalog.ids("classes")[0], "accessory_1": ring, "accessory_2": ring}
checks["the same accessory in both class slots is an error"] = TestHelpers.any_contains(_errors(dup_class), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR)
checks["...and in both unit slots"] = TestHelpers.any_contains(ContentCatalog.validate_entry("units", dup_unit), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR)
checks["one accessory alone is fine"] = not TestHelpers.any_contains(_errors(_class("t_cp_acc", {}, {"accessory_1": ring, "accessory_2": ""})), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR)
checks["two empty class slots never count"] = not TestHelpers.any_contains(_errors(_class("t_cp_acc", {}, {"accessory_1": "", "accessory_2": ""})), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR)
var none_unit: Dictionary = {"id": "t_cp_un", "name": "U", "class_id": ContentCatalog.ids("classes")[0], "accessory_1": ContentSchema.NO_ITEM, "accessory_2": ContentSchema.NO_ITEM}
checks["two 'none' unit slots never count"] = not TestHelpers.any_contains(ContentCatalog.validate_entry("units", none_unit), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR)
var saved: bool = StatRules.allow_duplicate_accessory
StatRules.allow_duplicate_accessory = true
checks["with the flag on the same entry validates"] = not TestHelpers.any_contains(_errors(dup_class), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR) and not TestHelpers.any_contains(ContentCatalog.validate_entry("units", dup_unit), ClassUnitValidator.DUPLICATE_ACCESSORY_ERROR)
StatRules.allow_duplicate_accessory = saved
return TestHelpers.report(TAG, "Duplicate accessory flag (StatRules.ALLOW_DUPLICATE_ACCESSORY)", checks)
```

### `static _test_heal_below() -> bool`
```logic
var checks: Dictionary = {}
var f: Dictionary = ContentSchema.field("units", "heal_below")
checks["unit field heal_below in the ai group, default = AiProfile default, range 0.1-1.0"] = not f.is_empty() and str(f["group"]) == ClassUnitFields.UNIT_AI_GROUP and is_equal_approx(DictUtil.to_float(f["default"]), AiProfile.new().heal_below) and is_equal_approx(DictUtil.to_float(f["min"]), ClassUnitFields.HEAL_BELOW_MIN) and is_equal_approx(DictUtil.to_float(f["max"]), ClassUnitFields.HEAL_BELOW_MAX)
checks["normalize default-fills it"] = is_equal_approx(DictUtil.to_float(ContentSchema.normalize("units", {"id": "t"})["heal_below"]), AiProfile.DEFAULT_HEAL_BELOW)
var unit: Dictionary = {"id": "t_cp_heal", "name": "H", "class_id": ContentCatalog.ids("classes")[0], "heal_below": 0.5}
var c: Combatant = ContentFactory.unit(unit, "", 12)
checks["ai_profile reads the entry value"] = is_equal_approx(ContentFactory.ai_profile(unit, c).heal_below, 0.5)
checks["out of range is clamped to the schema range"] = is_equal_approx(DictUtil.to_float(ContentSchema.normalize("units", {"heal_below": 0.0})["heal_below"]), ClassUnitFields.HEAL_BELOW_MIN)
var unchanged: bool = true
for u: Dictionary in ContentCatalog.entries("units"):
	unchanged = unchanged and is_equal_approx(ContentFactory.ai_profile(u, ContentFactory.unit(u, "", 12)).heal_below, AiProfile.new().heal_below)
checks["every shipped unit keeps the old AiProfile.heal_below"] = unchanged
return TestHelpers.report(TAG, "Unit AI field heal_below flows into AiProfile", checks)
```

### `static _test_locks() -> bool`
```logic
var checks: Dictionary = {}
checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
checks["fixtures"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_cp_f1")).is_empty()
var id: String = ContentCatalog.ids("classes")[0]
var base: Dictionary = ContentCatalog.get_entry("classes", id)
checks["shipped classes are system"] = ContentCatalog.is_system_origin(base)
var keys: Array[String] = []
for f: Dictionary in ContentSchema.fields("classes"):
	if DictUtil.to_bool(f.get("key_field", false)):
		keys.append(str(f["key"]))
checks["key fields are exactly id, ai_role, base_stats, growth, proficiency, origin"] = _same_set(keys, ["id", "ai_role", "base_stats", "growth", "proficiency", "origin"])
var free_edits: Dictionary = {"name": str(base["name"]) + "!", "description": "Edited.", "art": UnitArt.CLASS_KEYS[(UnitArt.CLASS_KEYS.find(str(base["art"])) + 1) % UnitArt.CLASS_KEYS.size()]}
for key: Variant in free_edits:
	var e: Dictionary = ContentCatalog.get_entry("classes", id)
	e[key] = free_edits[key]
	checks["changing %s on a system class succeeds" % str(key)] = ContentCatalog.upsert("classes", e).is_empty()
var main_edit: Dictionary = ContentCatalog.get_entry("classes", id)
main_edit["main_hand"] = ""
checks["changing main_hand on a system class succeeds (validated)"] = ContentCatalog.upsert("classes", main_edit).is_empty()
var pouch_edit: Dictionary = ContentCatalog.get_entry("classes", id)
pouch_edit["pouch"] = []
checks["changing pouch succeeds"] = ContentCatalog.upsert("classes", pouch_edit).is_empty()
var path_edit: Dictionary = ContentCatalog.get_entry("classes", id)
var path: Dictionary = DictUtil.dict_of(path_edit["skill_path"]).duplicate()
path["t_cp_f1"] = 1
path_edit["skill_path"] = path
checks["changing skill_path succeeds"] = ContentCatalog.upsert("classes", path_edit).is_empty()
var bad_path: Dictionary = ContentCatalog.get_entry("classes", id)
bad_path["skill_path"] = {SkillCatalog.basic_ids()[0]: 1}
checks["...but is still validated"] = not ContentCatalog.upsert("classes", bad_path).is_empty()
var stats_edit: Dictionary = ContentCatalog.get_entry("classes", id)
var stats: Dictionary = ContentFactory._ints(stats_edit["base_stats"])
var s0: String = StatRules.STATS[0]
var s1: String = StatRules.STATS[1]
var v0: int = DictUtil.int_of(stats, s0, StatRules.STAT_FLOOR)
stats[s0] = DictUtil.int_of(stats, s1, StatRules.STAT_FLOOR)
stats[s1] = v0
stats_edit["base_stats"] = stats
checks["the stat swap is a real change"] = stats != ContentFactory._ints(base["base_stats"])
checks["changing base_stats is a locked-field error"] = TestHelpers.any_contains(ContentCatalog.upsert("classes", stats_edit), "key field")
var growth_edit: Dictionary = ContentCatalog.get_entry("classes", id)
var growth: Dictionary = ContentFactory._ints(growth_edit["growth"])
growth_edit["growth"] = _moved_point(growth)
checks["changing growth is a locked-field error"] = TestHelpers.any_contains(ContentCatalog.upsert("classes", growth_edit), "key field")
var prof_edit: Dictionary = ContentCatalog.get_entry("classes", id)
var prof: Dictionary = DictUtil.dict_of(prof_edit["proficiency"]).duplicate(true)
var first_gear: String = str(prof.keys()[0])
var row: Dictionary = DictUtil.dict_of(prof[first_gear])
row["scaling"] = Proficiency.MAX_SCALING if not is_equal_approx(DictUtil.to_float(row.get("scaling", 0.0)), Proficiency.MAX_SCALING) else Proficiency.MIN_SCALING
prof[first_gear] = row
prof_edit["proficiency"] = prof
checks["changing proficiency is a locked-field error"] = TestHelpers.any_contains(ContentCatalog.upsert("classes", prof_edit), "key field")
var role_edit: Dictionary = ContentCatalog.get_entry("classes", id)
role_edit["ai_role"] = "SKIRMISHER" if str(role_edit["ai_role"]) != "SKIRMISHER" else "AGGRESSOR"
checks["changing ai_role is a locked-field error"] = TestHelpers.any_contains(ContentCatalog.upsert("classes", role_edit), "key field")
var renamed: Dictionary = ContentCatalog.get_entry("classes", id)
renamed["id"] = id + "_renamed"
checks["renaming a system class is refused"] = not ContentCatalog.upsert("classes", renamed, id).is_empty() and ContentCatalog.has("classes", id)
checks["remove of a system class is refused"] = ContentCatalog.remove("classes", id).contains("system asset")
# User classes (ids built by concatenation so the SAFE DELETE scan of this file never finds them).
var user_id: String = "t_cp_" + "user"
var user_class: Dictionary = ClassForgeModel.new_entry(user_id)
user_class.erase("origin")
checks["a new class is created as a user asset"] = ContentCatalog.upsert("classes", user_class).is_empty() and str(ContentCatalog.get_entry("classes", user_id)["origin"]) == ContentSchema.ORIGIN_USER
var user_stats: Dictionary = ContentCatalog.get_entry("classes", user_id)
user_stats["base_stats"] = _swapped(ContentFactory._ints(user_stats["base_stats"]))
checks["key fields of a user class are editable"] = ContentCatalog.upsert("classes", user_stats).is_empty()
var unit_id: String = "t_cp_" + "user_unit"
checks["a unit can use the user class"] = ContentCatalog.upsert("units", {"id": unit_id, "name": "U", "side": "party", "class_id": user_id}).is_empty()
checks["remove of a referenced user class is refused"] = ContentCatalog.remove("classes", user_id).contains("still used")
var moved: Dictionary = ContentCatalog.get_entry("classes", user_id)
moved["id"] = user_id + "_b"
checks["renaming a user class rewrites unit class_id"] = ContentCatalog.upsert("classes", moved, user_id).is_empty() and str(ContentCatalog.get_entry("units", unit_id)["class_id"]) == user_id + "_b"
checks["remove of an unreferenced user class succeeds"] = ContentCatalog.remove("units", unit_id).is_empty() and ContentCatalog.remove("classes", user_id + "_b").is_empty()
var promo_id: String = "t_cp_" + "promo"
checks["promote makes a user class system"] = ContentCatalog.upsert("classes", ClassForgeModel.new_entry(promo_id)).is_empty() and ContentCatalog.promote("classes", promo_id).is_empty() and ContentCatalog.is_system_origin(ContentCatalog.get_entry("classes", promo_id))
checks["...then delete fails"] = ContentCatalog.remove("classes", promo_id).contains("system asset")
checks["...demote makes it deletable again"] = ContentCatalog.demote("classes", promo_id).is_empty() and ContentCatalog.remove("classes", promo_id).is_empty()
return TestHelpers.report(TAG, "System class locks: key fields only; gear, pouch, path editable; delete and promote", checks)
```

### `static _same_set(a: Array[String], b: Array[String]) -> bool`
```logic
if a.size() != b.size():
	return false
for x: String in a:
	if not b.has(x):
		return false
return true
```

### `static _swapped(stats: Dictionary) -> Dictionary`
```logic
var out: Dictionary = stats.duplicate()
var s0: String = StatRules.STATS[0]
var s1: String = StatRules.STATS[1]
var v0: int = DictUtil.int_of(out, s0, StatRules.STAT_FLOOR)
var v1: int = DictUtil.int_of(out, s1, StatRules.STAT_FLOOR)
if v0 == v1:
	out[s0] = v0 + 1
	out[s1] = v1 - 1
else:
	out[s0] = v1
	out[s1] = v0
return out
```

### `static _moved_point(growth: Dictionary) -> Dictionary`
```logic
var out: Dictionary = growth.duplicate()
var from: String = ""
for s: String in StatRules.STATS:
	if DictUtil.int_of(out, s) > 0 and (from.is_empty() or DictUtil.int_of(out, s) > DictUtil.int_of(out, from)):
		from = s
for s: String in StatRules.STATS:
	if s != from and DictUtil.int_of(out, s) < StatRules.GROWTH_STAT_MAX:
		out[from] = DictUtil.int_of(out, from) - 1
		out[s] = DictUtil.int_of(out, s) + 1
		if DictUtil.int_of(out, from) == 0:
			out.erase(from)
		return out
return out
```

### `static _test_migration() -> bool`
```logic
var checks: Dictionary = {}
if not ResourceLoader.exists(SEED):
	return TestHelpers.report_ok(TAG, "CLASS1-M seed archived (migration already merged)", true)
# The seed reads list-shaped data through the converter, which is off since CLASS1-M; switch it on for this test only.
var conv_saved: bool = ClassUnitFields.list_path_converter
ClassUnitFields.list_path_converter = true
checks["sandbox fork"] = TestHelpers.fork(MIGRATE_SANDBOX)
var basic: String = SkillCatalog.basic_ids()[0]
checks["a list-shaped classes.json is written to the fork"] = _write_list_shaped(basic)
ContentCatalog.reload_from_disk()
var seed_script: GDScript = load(SEED)
checks["the fork stores lists"] = str(seed_script.call("file_shape")) == "list"
checks["the list-shaped fork loads and validates (converter)"] = ContentCatalog.validate_all().is_empty() and ContentCatalog.load_errors().is_empty()
var report: Dictionary = DictUtil.dict_of(seed_script.call("migrate"))
checks["migration report ok"] = DictUtil.to_bool(report.get("ok", false))
checks["...every class migrated"] = DictUtil.to_int(report.get("classes", 0)) == ContentCatalog.ids("classes").size()
checks["...the injected basic was dropped"] = str(report.get("basics_dropped", [])).contains(basic)
checks["...same skills and levels (no path diffs)"] = DictUtil.array_of(report.get("path_diffs", ["?"])).is_empty() and DictUtil.array_of(report.get("order_diffs", ["?"])).is_empty()
checks["...default loadouts identical at levels 1-10"] = DictUtil.array_of(report.get("loadout_diffs", ["?"])).is_empty()
checks["...every class-derived unit owns every basic at every level (SKILL1 grant rule)"] = DictUtil.array_of(report.get("basics_missing", ["?"])).is_empty()
checks["save_all"] = ContentCatalog.save_all() == OK
ContentCatalog.reload_from_disk()
checks["the file now stores dicts"] = str(seed_script.call("file_shape")) == "dict"
checks["validate_all empty after reload"] = ContentCatalog.validate_all().is_empty()
var first: String = FileAccess.get_file_as_string(ContentCatalog.path_for("classes"))
checks["a second save is byte-identical (round trip)"] = ContentCatalog.save_all() == OK and FileAccess.get_file_as_string(ContentCatalog.path_for("classes")) == first
checks["units.json carries heal_below"] = FileAccess.get_file_as_string(ContentCatalog.path_for("units")).contains("\"heal_below\"")
var again: Dictionary = DictUtil.dict_of(seed_script.call("migrate"))
checks["the seed is idempotent"] = DictUtil.to_bool(again.get("ok", false)) and str(again.get("file_shape_before", "")) == "dict"
ClassUnitFields.list_path_converter = conv_saved
return TestHelpers.report(TAG, "CLASS1-M seed on a list-shaped fork", checks)
```

### `static _write_list_shaped(basic: String) -> bool`
```logic
var path: String = ContentCatalog.path_for("classes")
var doc: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(path)))
var listed: Array = []
var first: bool = true
for raw: Variant in DictUtil.array_of(doc.get("entries", [])):
	var e: Dictionary = DictUtil.dict_of(raw)
	var steps: Array = []
	if first:
		steps.append({"level": 1, "skill": basic})
		first = false
	var d: Dictionary = DictUtil.dict_of(e.get("skill_path", {}))
	for key: Variant in d:
		steps.append({"level": d[key], "skill": str(key)})
	e["skill_path"] = steps
	listed.append(e)
doc["entries"] = listed
var file := FileAccess.open(path, FileAccess.WRITE)
if file == null:
	return false
file.store_string(JSON.stringify(doc, "\t", false) + "\n")
file.close()
return not listed.is_empty()
```

