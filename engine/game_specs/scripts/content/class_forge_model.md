---
class: ClassForgeModel
extends: RefCounted
description: "Tactical RPG - Class Forge Model (FORGE1_CLASSES 2-5) The headless brain of the class tab of the Content Forge viewer (and the unit AI group): pure static functions over a class entry. Layout (the five sections from the schema `group`), each control's state (ranges equal to the rule constants, disabled options and the reason), a fresh valid class, the problems of an entry (errors) and the warnings, the skill path table (ordering, standard / custom levels, family picker, tier ladder, default-loadout preview), the stat, gear budget and proficiency coverage panels, and the gear pickers grouped by item family (dual-wield guard, duplicate-accessory flag). The controls only render what this returns."
---

# Properties
* `SECTION_TITLES`: Dictionary = { (const)
* `ID_FIXED_REASON`: String = "The id is fixed after the first save (rename through the catalog API rewrites references)." (const)
* `ROLE_HELP`: Dictionary = { (const)
* `PROFICIENCY_STEP`: float = 0.05 (const)
* `CUSTOM_LEVEL`: int = -1   # level selector entry "Custom level..." (reveals a spinner 1..MAX_LEVEL) (const)
* `order`: Array[String] = []
* `by_group`: Dictionary = {}
* `group`: String = str(f.get("group", "identity" if kind == "classes" else "general"))
* `key`: String = str(f["key"])
* `keys`: Array = by_group[group]
* `preferred`: Array[String] = ["general", ClassUnitFields.UNIT_AI_GROUP, ClassUnitFields.UNIT_AI_ADVANCED_GROUP, "loadout"]
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `messages`: Array[String] = ContentCatalog.validate_entry("classes", entry, stored_id)
* `old`: Dictionary = ContentCatalog.get_entry("classes", stored_id if not stored_id.is_empty() else str(entry.get("id", "")))
* `floor_level`: int = advanced_floor(family)
* `out`: Array[int] = []
* `path`: Dictionary = DictUtil.dict_of(entry.get("skill_path", {}))
* `active`: Array[String] = default_loadout(entry, StatRules.MAX_LEVEL)
* `out`: Array[Dictionary] = []
* `fam`: String = str(key)
* `level`: int = DictUtil.to_int(path[key], 1)
* `head`: Dictionary = ContentCatalog.get_entry("skills", fam)
* `requires`: Array[String] = ContentFactory._strings(head.get("requires", []))
* `requires_ok`: bool = true
* `path`: Dictionary = DictUtil.dict_of(entry.get("skill_path", {}))
* `out`: Array[Dictionary] = []
* `id`: String = str(row.get("id", ""))
* `requires`: Array[String] = ContentFactory._strings(row.get("requires", []))
* `missing`: Array[String] = []
* `used`: bool = path.has(id) and id != current
* `arch`: ClassArchetype = ContentFactory.class_archetype(entry)
* `slots`: Dictionary = ContentFactory.loadout_of({}, arch, level)
* `e`: Dictionary = ContentSchema.normalize("classes", entry)
* `out`: Array[Dictionary] = []
* `prof`: Proficiency = Proficiency.from_dict(DictUtil.dict_of(e["proficiency"]))
* `used`: Array[String] = []
* `base`: Dictionary = ContentFactory._ints(e["base_stats"])
* `item`: Dictionary = ContentCatalog.get_entry("items", str(e[slot]))
* `gear`: String = str(item.get("weapon_type", ""))
* `req`: Dictionary = ContentFactory._ints(DictUtil.dict_of(item.get("required_stats", {})))
* `have`: int = DictUtil.int_of(base, str(stat), StatRules.STAT_FLOOR)
* `armor`: bool = false
* `prof`: Dictionary = DictUtil.dict_of(entry.get("proficiency", {}))
* `out`: Array[String] = []
* `prof`: Proficiency = Proficiency.from_dict(DictUtil.dict_of(entry.get("proficiency", {})))
* `check_training`: bool = kind == "classes"
* `arch`: ClassArchetype = ContentFactory.class_of(entry)
* `other`: String = ""
* `groups`: Array[String] = []
* `by_group`: Dictionary = {}
* `group`: String = item_group(item)
* `id`: String = str(item.get("id", ""))
* `gear`: String = str(item.get("weapon_type", ""))
* `reason`: String = ""
* `list`: Array = by_group[group]
* `out`: Array[Dictionary] = []

# Methods

### `static group_of(kind: String, key: String) -> String`
```logic
return str(ContentSchema.field(kind, key).get("group", ""))
```

### `static state(entry: Dictionary, key: String, stored_id: String = "") -> Dictionary`
```logic
var out: Dictionary = {"enabled": true, "reason": "", "min": null, "max": null, "step": null, "options": [], "disabled_options": {}}
var f: Dictionary = ContentSchema.field("classes", key)
if f.is_empty():
	return out
if DictUtil.to_bool(f.get("readonly", false)):
	out["enabled"] = false
if key == "id" and not stored_id.is_empty():
	out["enabled"] = false
	out["reason"] = ID_FIXED_REASON
match key:
	"ai_role":
		out["reason"] = str(ROLE_HELP.get(str(entry.get("ai_role", "")), ""))
	"main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2":
		_slot_state(out, entry, key, "classes")
	"pouch":
		out["max"] = pouch_limit()
return out
```

### `static _slot_state(out: Dictionary, entry: Dictionary, key: String, kind: String) -> void`
```logic
var disabled: Dictionary = out["disabled_options"]
var choices: Array[Dictionary] = slot_choices(entry, key, kind)
out["choices"] = choices
for choice: Dictionary in choices:
	if not DictUtil.to_bool(choice["enabled"]):
		disabled[str(choice["id"])] = str(choice["reason"])
if key == "off_hand":
	var main_id: String = str(entry.get("main_hand", ""))
	if main_id.is_empty() and kind == "units":
		var arch: ClassArchetype = ContentFactory.class_of(entry)
		main_id = str(arch.loadout.get("main_hand", "")) if arch != null else ""
	var main: Dictionary = ContentCatalog.get_entry("items", main_id)
	if DictUtil.to_bool(main.get("two_handed", false)):
		out["enabled"] = false
		out["reason"] = "The main hand holds a two-handed weapon: the off hand stays empty."
```

### `static unit_state(entry: Dictionary, key: String) -> Dictionary`
```logic
var out: Dictionary = {"enabled": true, "reason": "", "min": null, "max": null, "step": null, "options": [], "disabled_options": {}}
match key:
	"main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2":
		_slot_state(out, entry, key, "units")
	"pouch":
		out["max"] = pouch_limit()
	"ai_role":
		var arch: ClassArchetype = ContentFactory.class_of(entry)
		out["empty_label"] = "Class role: %s" % (arch.ai_role if arch != null else "AGGRESSOR")
		out["reason"] = str(ROLE_HELP.get(effective_role(entry), ""))
return out
```

### `static ranges() -> Dictionary`
```logic
return {
	"base_stats": {"min": StatRules.STAT_FLOOR, "max": StatRules.STAT_CEILING, "total": StatRules.CLASS_BUDGET},
	"growth": {"min": 0, "max": StatRules.GROWTH_STAT_MAX, "total": StatRules.GROWTH_TOTAL},
	"proficiency_level": {"min": 1, "max": Proficiency.MAX_LEVEL},
	"proficiency_scaling": {"min": Proficiency.MIN_SCALING, "max": Proficiency.MAX_SCALING, "step": PROFICIENCY_STEP, "default": Proficiency.DEFAULT_SCALING},
	"path_level": {"min": 1, "max": StatRules.MAX_LEVEL, "standard": ClassArchetype.STANDARD_PATH_LEVELS, "advanced_min": StatRules.ADVANCED_MIN_LEVEL},
	"name": {"max_len": ClassUnitFields.NAME_MAX_LEN},
	"description": {"max_len": ClassUnitFields.DESCRIPTION_MAX_LEN},
	"id": {"min_len": ClassUnitFields.ID_MIN_LEN, "max_len": ClassUnitFields.ID_MAX_LEN},
	"pouch": {"max": pouch_limit()},
	"heal_below": {"min": ClassUnitFields.HEAL_BELOW_MIN, "max": ClassUnitFields.HEAL_BELOW_MAX, "step": 0.05},
}
```

### `static pouch_limit() -> int`
```logic
return Inventory.new().max_pouch_slots
```

### `static remaining(entry: Dictionary, key: String) -> int`
```logic
var stats: Dictionary = ContentFactory._ints(DictUtil.dict_of(entry.get(key, {})))
if key == "growth":
	return StatRules.GROWTH_TOTAL - StatRules.total_of(stats)
var total: int = 0
for s: String in StatRules.STATS:
	total += DictUtil.int_of(stats, s, StatRules.STAT_FLOOR)
return StatRules.CLASS_BUDGET - total
```

### `static new_entry(id: String) -> Dictionary`
```logic
var e: Dictionary = ContentSchema.defaults("classes")
e["id"] = id
e["origin"] = ContentSchema.ORIGIN_USER
var base: Dictionary = {}
for s: String in StatRules.STATS:
	base[s] = StatRules.STAT_FLOOR
var left: int = StatRules.CLASS_BUDGET - StatRules.STAT_FLOOR * StatRules.STATS.size()
var i: int = 0
while left > 0:
	var stat: String = StatRules.STATS[i % StatRules.STATS.size()]
	if DictUtil.to_int(base[stat]) < StatRules.STAT_CEILING:
		base[stat] = DictUtil.to_int(base[stat]) + 1
		left -= 1
	i += 1
e["base_stats"] = base
var growth: Dictionary = {}
var g_left: int = StatRules.GROWTH_TOTAL
var j: int = 0
while g_left > 0:
	var g_stat: String = StatRules.STATS[j % StatRules.STATS.size()]
	if DictUtil.int_of(growth, g_stat) < StatRules.GROWTH_STAT_MAX:
		growth[g_stat] = DictUtil.int_of(growth, g_stat) + 1
		g_left -= 1
	j += 1
e["growth"] = growth
e["proficiency"] = {"sword": {"level": 1, "scaling": Proficiency.DEFAULT_SCALING}}
e["skill_path"] = {}
return e
```

### `static save_state(entry: Dictionary, stored_id: String = "") -> Dictionary`
```logic
var problems: Array[Dictionary] = draft_problems(entry, stored_id)
var reasons: Array[String] = []
for p: Dictionary in problems:
	reasons.append(str(p["message"]))
return {"blocked": not problems.is_empty(), "problems": problems, "reasons": reasons, "warnings": ClassUnitValidator.class_warnings(ContentSchema.normalize("classes", entry))}
```

### `static field_of_message(message: String, entry: Dictionary = {}) -> String`
```logic
if message.begins_with("Skill path") or message.begins_with("The skill path"):
	return "skill_path"
if message.begins_with("Base "):
	return "base_stats"
if message.begins_with("Growth"):
	return "growth"
if message.begins_with("Proficiency") or message.contains("gear proficiency"):
	return "proficiency"
if message.begins_with("Art "):
	return "art"
if message.begins_with("ID ") or message.begins_with("ID:"):
	return "id"
if message.begins_with("Name"):
	return "name"
if message.begins_with("Description"):
	return "description"
if message.contains("accessory"):
	return "accessory_2"
if message.contains("two-handed"):
	return "off_hand"
if message.contains("key field"):
	for f: Dictionary in ContentSchema.fields("classes"):
		if message.contains("'%s'" % str(f.get("label", ""))):
			return str(f["key"])
for slot: String in ContentSchema.LOADOUT_SLOTS + ["pouch"]:
	var label: String = str(ContentSchema.field("classes", slot).get("label", slot))
	if message.contains(label) or message.contains(slot.capitalize()):
		return slot
if message.contains("no training") or message.contains("can't be equipped"):
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item_name: String = str(ContentCatalog.get_entry("items", str(entry.get(slot, ""))).get("name", ""))
		if not item_name.is_empty() and (message.contains("(%s)" % item_name) or message.begins_with(item_name)):
			return slot
return ""
```

### `static advanced_floor(family: String) -> int`
```logic
var head: Dictionary = ContentCatalog.get_entry("skills", family)
return StatRules.ADVANCED_MIN_LEVEL if str(head.get("grade", "novice")) != "novice" else 1
```

### `static is_custom_level(level: int) -> bool`
```logic
return not ClassArchetype.STANDARD_PATH_LEVELS.has(level)
```

### `static path_counter(entry: Dictionary) -> String`
```logic
var n: int = DictUtil.dict_of(entry.get("skill_path", {})).size()
return "%d %s, first %d active by default" % [n, "family" if n == 1 else "families", StatRules.MAX_CLASS_SLOTS]
```

### `static ladder(family: String, level: int) -> String`
```logic
var parts: Array[String] = []
for row: SkillDef in SkillCatalog.family_tiers(family):
	var opens: int = level + row.tier_offset()
	parts.append("T%d Lv %d%s" % [row.tier, mini(opens, StatRules.MAX_LEVEL), " (capped)" if opens > StatRules.MAX_LEVEL else ""])
return " / ".join(parts) if not parts.is_empty() else "T1 Lv %d" % level
```

### `static set_path_level(entry: Dictionary, family: String, level: int) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var path: Dictionary = DictUtil.dict_of(out.get("skill_path", {})).duplicate()
path[family] = level
out["skill_path"] = path
return out
```

### `static add_path_family(entry: Dictionary, family: String, level: int = -1) -> Dictionary`
```logic
if DictUtil.dict_of(entry.get("skill_path", {})).has(family):
	return entry.duplicate(true)
return set_path_level(entry, family, level if level > 0 else maxi(1, advanced_floor(family)))
```

### `static remove_path_family(entry: Dictionary, family: String) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var path: Dictionary = DictUtil.dict_of(out.get("skill_path", {})).duplicate()
path.erase(family)
out["skill_path"] = path
return out
```

### `static replace_path_family(entry: Dictionary, old_family: String, new_family: String) -> Dictionary`
```logic
var out: Dictionary = entry.duplicate(true)
var path: Dictionary = DictUtil.dict_of(out.get("skill_path", {}))
if not path.has(old_family) or (path.has(new_family) and new_family != old_family):
	return out
out["skill_path"] = ContentCatalog._renamed_key(path, old_family, new_family)
return out
```

### `static stat_panel(entry: Dictionary) -> Dictionary`
```logic
var arch: ClassArchetype = ContentFactory.class_archetype(entry)
var rows: Array[Dictionary] = []
var base_total: int = 0
for s: String in StatRules.STATS:
	var base: int = DictUtil.int_of(arch.base_stats, s, StatRules.STAT_FLOOR)
	var growth: int = DictUtil.int_of(arch.growth_per_level, s)
	base_total += base
	var bad: String = ""
	if base < StatRules.STAT_FLOOR or base > StatRules.STAT_CEILING:
		bad = "base outside %d-%d" % [StatRules.STAT_FLOOR, StatRules.STAT_CEILING]
	elif growth < 0 or growth > StatRules.GROWTH_STAT_MAX:
		bad = "growth outside 0-%d" % StatRules.GROWTH_STAT_MAX
	rows.append({"stat": s, "base": base, "growth": growth, "lv5": base + growth * 4, "lv10": base + growth * (StatRules.MAX_LEVEL - 1), "bad": bad})
var vitals: Array[Dictionary] = []
for lvl: int in [1, 5, StatRules.MAX_LEVEL]:
	var st := CharacterStats.new(arch, lvl)
	vitals.append({"level": lvl, "hp": st.get_max_hp({}), "speed": st.get_initiative_speed(0, {})})
return {"base_total": base_total, "budget": StatRules.CLASS_BUDGET, "growth_total": StatRules.total_of(arch.growth_per_level), "growth_budget": StatRules.GROWTH_TOTAL,
	"rows": rows, "vitals": vitals, "note": "Units add %d + (level - 1) free points (StatRules.SPEND_BASE)." % StatRules.SPEND_BASE}
```

### `static budget_panel(entry: Dictionary) -> Dictionary`
```logic
var rows: Array[Dictionary] = GearBudget.loadout_breakdown(entry)
var total: float = GearBudget.loadout_value(entry)
var specials: Array[String] = []
for row: Dictionary in rows:
	if DictUtil.to_bool(row["special"]):
		specials.append("%s (%s)" % [str(row["name"]), str(row["note"]) if not str(row["note"]).is_empty() else "no note"])
var in_band: bool = GearBudget.in_band(total)
return {"rows": rows, "total": total, "target": GearBudget.TARGET_BUDGET, "low": GearBudget.band_low(), "high": GearBudget.band_high(),
	"in_band": in_band, "specials": specials, "excused": not in_band and total > GearBudget.band_high() and GearBudget.has_documented_special(entry),
	"unarmed_value": GearBudget.UNARMED_VALUE}
```

### `static can_remove_proficiency(entry: Dictionary, gear: String) -> Dictionary`
```logic
var prof: Dictionary = DictUtil.dict_of(entry.get("proficiency", {}))
if Proficiency.WEAPON_TYPES.has(gear):
	var weapons: int = 0
	for key: Variant in prof:
		if Proficiency.WEAPON_TYPES.has(str(key)):
			weapons += 1
	if weapons <= 1:
		return {"allowed": false, "reason": "A class needs at least one weapon type."}
for slot: String in ContentSchema.LOADOUT_SLOTS:
	var item: Dictionary = ContentCatalog.get_entry("items", str(entry.get(slot, "")))
	if str(item.get("weapon_type", "")) == gear:
		return {"allowed": false, "reason": "The default gear uses it (%s)." % str(item.get("name", ""))}
return {"allowed": true, "reason": ""}
```

### `static item_group(item: Dictionary) -> String`
```logic
var family: Dictionary = ContentCatalog.family_of(item)
if not family.is_empty():
	return str(family.get("name", family.get("id", "")))
var gear: String = str(item.get("weapon_type", ""))
return gear.capitalize() if not gear.is_empty() else str(item.get("item_type", "")).capitalize()
```

### `static effective_role(unit: Dictionary) -> String`
```logic
var own: String = str(unit.get("ai_role", ""))
if not own.is_empty():
	return own
var arch: ClassArchetype = ContentFactory.class_of(unit)
return arch.ai_role if arch != null else ""
```

### `static unit_field_visible(unit: Dictionary, key: String) -> bool`
```logic
match key:
	"flee_ratio", "defend_below":
		return not DictUtil.to_bool(unit.get("reckless", false))
	"heal_below":
		return effective_role(unit) == "SUPPORT"
var f: Dictionary = ContentSchema.field("units", key)
return f.is_empty() or ContentSchema.is_visible(f, unit)
```

