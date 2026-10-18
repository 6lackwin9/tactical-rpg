---
class: ClassArchetype
extends: RefCounted
description: "Tactical RPG - Class Archetype (the six base classes; data in the content catalog) A class is stats + gear + skills, lore-free: base_stats / growth_per_level   follow StatRules (equal budget, equal growth) proficiency                     {gear type: {\"level\": 1-3, \"scaling\": 1.0-2.0}} what the class can wield and how well (Proficiency) skill_path                      {family id: unlock level} (insertion order = entry position; FORGE1_CLASSES 3.5) loadout                         default gear per slot (item ids); units may override art / ai_role                   token & portrait key, default AI role Units (class + level + side + tier + spend) are built from it by ContentFactory; passives (from world sources) are never part of a class."
---

# Properties
* `id`: String = "warrior"
* `display_name`: String = "Warrior"
* `description`: String = ""
* `archetype_name`: String:
* `base_stats`: Dictionary = {}         # stat (String) -> level-1 base value (int)
* `growth_per_level`: Dictionary = {}   # stat (String) -> guaranteed gain per level (int)
* `proficiency`: Dictionary = {}        # gear type -> {"level": int, "scaling": float}
* `skill_path`: Dictionary = {}         # family head id (String) -> unlock level (int), in entry order
* `loadout`: Dictionary = {}            # slot key (main_hand ...) -> item id
* `art`: String = "warrior"
* `ai_role`: String = "AGGRESSOR"
* `STANDARD_PATH_LEVELS`: Array[int] = [1, 3, 4, 5, 7] (const)
* `rows`: Array[Dictionary] = []
* `position`: int = 0
* `la`: int = DictUtil.to_int(a["level"])
* `lb`: int = DictUtil.to_int(b["level"])
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []

# Methods

### `_init(p_id: String = "warrior", p_name: String = "Warrior") -> void`
```logic
id = p_id
display_name = p_name
```

### `get_base_stat(stat_name: String) -> int`
```logic
return DictUtil.int_of(base_stats, stat_name.to_upper(), 10)
```

### `get_growth_for_stat(stat_name: String) -> int`
```logic
return DictUtil.int_of(growth_per_level, stat_name.to_upper(), 0)
```

### `make_proficiency() -> Proficiency`
```logic
return Proficiency.from_dict(proficiency)
```

### `static path_from_list(steps: Array, drop_basics: bool = true) -> Dictionary`
```logic
var out: Dictionary = {}
for raw: Variant in steps:
	var step: Dictionary = DictUtil.dict_of(raw)
	var skill_id: String = str(step.get("skill", ""))
	if skill_id.is_empty():
		continue
	if drop_basics and DictUtil.to_bool(ContentCatalog.get_entry("skills", skill_id).get("basic", false)):
		continue
	var level: Variant = whole_level(step.get("level", 1))
	if not out.has(skill_id):
		out[skill_id] = level
	elif level is int and out[skill_id] is int and DictUtil.to_int(level) < DictUtil.to_int(out[skill_id]):
		out[skill_id] = level
return out
```

### `static whole_level(raw: Variant) -> Variant`
```logic
if raw is float:
	var x: float = raw
	if x == floorf(x):
		return roundi(x)
return raw
```

### `can_equip_gear(eq: Equipment) -> bool`
```logic
if eq == null:
	return false
if eq.item_type == Item.ItemType.ACCESSORY or eq.slot in [Equipment.EquipSlot.ACCESSORY_1, Equipment.EquipSlot.ACCESSORY_2]:
	return true
if eq.weapon_type.is_empty() or proficiency.is_empty():
	return true
return make_proficiency().can_use(eq.weapon_type)
```

### `static create(class_id: String) -> ClassArchetype`
```logic
if ContentCatalog.has("classes", class_id):
	return ContentFactory.class_archetype(ContentCatalog.get_entry("classes", class_id))
push_warning("ClassArchetype: unknown class '%s'" % class_id)
return ClassArchetype.new(class_id, class_id.capitalize())
```

### `static create_warrior() -> ClassArchetype`
```logic
return create("warrior")
```

### `static create_ranger() -> ClassArchetype`
```logic
return create("ranger")
```

