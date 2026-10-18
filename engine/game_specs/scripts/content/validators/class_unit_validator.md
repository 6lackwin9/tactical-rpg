---
class: ClassUnitValidator
extends: RefCounted
description: "Class, unit and loadout validation rules (moved from ContentCatalog, CLN-9); ContentCatalog.validate_entry dispatches here. Class rules: equal stat budget and growth (StatRules), valid proficiency, a skill path whose skills fit the level cap, and default gear the class can wield."
---

# Properties
* `errors`: Array[String] = StatRules.validate_class(ContentFactory._ints(e["base_stats"]), ContentFactory._ints(e["growth"]))
* `prof_raw`: Variant = e["proficiency"]
* `weapons`: int = 0
* `prof`: Dictionary = prof_raw
* `gear`: String = str(key)
* `entry`: Dictionary = DictUtil.dict_of(prof[key])
* `scaling`: float = DictUtil.to_float(entry.get("scaling", Proficiency.DEFAULT_SCALING), -1.0)
* `errors`: Array[String] = []
* `path`: Dictionary = path_raw
* `skill_id`: String = str(key)
* `raw_level`: Variant = path[key]
* `level`: int = raw_level
* `skill`: Dictionary = ContentCatalog.get_entry("skills", skill_id)
* `errors`: Array[String] = []
* `item_id`: String = str(e.get(slot, ""))
* `item`: Dictionary = ContentCatalog.get_entry("items", item_id)
* `main`: Dictionary = ContentCatalog.get_entry("items", str(e.get("main_hand", "")))
* `off`: Dictionary = ContentCatalog.get_entry("items", str(e.get("off_hand", "")))
* `DUPLICATE_ACCESSORY_ERROR`: String = "The same accessory cannot fill both accessory slots." (const)
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []
* `path`: Dictionary = DictUtil.dict_of(e.get("skill_path", {}))
* `families`: Array[String] = []
* `prof`: Proficiency = Proficiency.from_dict(DictUtil.dict_of(e.get("proficiency", {})))
* `base_level`: int = DictUtil.to_int(path[fam], 1)
* `opens`: int = base_level + row.tier_offset()
* `head`: SkillDef = SkillCatalog.tier_one(fam)
* `arch`: ClassArchetype = ContentFactory.class_of(e)
* `none`: Array[Dictionary] = []
* `role`: String = str(e.get("ai_role", ""))
* `families`: Array[String] = []
* `main`: String = str(e.get("main_hand", ""))
* `out`: Array[Dictionary] = []
* `item`: Dictionary = ContentCatalog.get_entry("items", main_hand)
* `reach`: int = DictUtil.to_int(item.get("base_range", 1), 1) if not item.is_empty() else 1
* `errors`: Array[String] = []
* `fixed`: bool = str(e["stat_mode"]) == "fixed"
* `arch`: ClassArchetype = ContentFactory.class_of(e)
* `prof`: Proficiency = arch.make_proficiency() if arch != null else Proficiency.new()
* `errors`: Array[String] = []
* `owned`: Array[String] = []
* `extras`: Array[String] = ContentFactory._strings(e.get("extra_skills", []))
* `checked`: Array[String] = []
* `skill`: Dictionary = ContentCatalog.get_entry("skills", skill_id)
* `errors`: Array[String] = []
* `listed`: Array[String] = ContentFactory._strings(e.get("loadout", []))
* `level`: int = DictUtil.to_int(e.get("level", 1), 1)
* `owned_ids`: Array[String] = arch.skills_at(level)
* `owned`: Array[String] = ContentFactory.families_of(owned_ids)
* `seen`: Array[String] = []
* `slots`: Dictionary = ContentFactory.loadout_of(e, arch, level)
* `class_count`: int = ContentFactory._strings(slots["class"]).size()
* `extra_count`: int = ContentFactory._strings(slots["extra"]).size()

# Methods

### `static is_duplicate_accessory(first: String, second: String) -> bool`
```logic
if StatRules.allow_duplicate_accessory or first.is_empty() or first == ContentSchema.NO_ITEM:
	return false
return first == second
```

### `static _warn(section: String, text: String) -> Dictionary`
```logic
return {"section": section, "text": text}
```

### `static _path_has_support(families: Array[String]) -> bool`
```logic
for fam: String in families:
	var head: SkillDef = SkillCatalog.tier_one(fam)
	if head != null and head.kind == SkillDef.Kind.SUPPORT:
		return true
return false
```

### `static _gate_reachable(tokens: Array[String], prof: Proficiency) -> bool`
```logic
for token: String in tokens:
	var gear: String = SkillDef.UNARMED_PROFICIENCY if token == SkillDef.UNARMED_TOKEN else token
	if prof.can_use(gear):
		return true
return false
```

