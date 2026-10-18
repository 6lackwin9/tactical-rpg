---
class: SkillCatalog
extends: RefCounted
description: "Tactical RPG - Skill Catalog (the command tiers available for planning) Built from the ContentCatalog \"skills\" entries (edit them in the Content Forge). One family per command kind, tiered by level 1-3 today (4-5 reserved for progression). A unit may use every owned tier whose unlock level it has reached (Combatant.tier_unlocked); AiProfile.skill_cap is only an explicit AP ceiling. Move and Rest are single skills: Move is one block at 1 AP per tile (up to Combatant.move_per_block() tiles), Rest is one free block that recovers AP. Skills come from the content catalog (res://data/catalog/skills.json), in file order: the first skill of a kind is its \"lowest\" (Strike, Move...)."
---

# Properties
* `out`: Array[String] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[SkillDef] = []
* `result`: Array[String] = []
* `out`: Array[Dictionary] = []
* `fam`: String = s.family_id()
* `group`: Dictionary = {}
* `tiers`: Array[SkillDef] = []
* `list`: Array[SkillDef] = group["tiers"]
* `list`: Array[SkillDef] = g["tiers"]

# Methods

### `static _build() -> void`
```logic
_by_id.clear()
_tier_one.clear()
for entry: Dictionary in ContentCatalog.entries("skills"):
	var s: SkillDef = ContentFactory.skill(entry)
	_all.append(s)
	if not _by_id.has(s.id):
		_by_id[s.id] = s
	if not _tier_one.has(s.family_id()):
		_tier_one[s.family_id()] = s
	else:
		var low: SkillDef = _tier_one[s.family_id()]
		if s.tier < low.tier:
			_tier_one[s.family_id()] = s
```

### `static clear_cache() -> void`
```logic
_all.clear()
_by_id.clear()
_tier_one.clear()
```

### `static tier_one(family: String) -> SkillDef`
```logic
all()
if not _tier_one.has(family):
	return null
var s: SkillDef = _tier_one[family]
return s
```

### `static is_system(skill_id: String) -> bool`
```logic
var s: SkillDef = find(skill_id)
return s != null and s.system
```

### `static is_learned_only(skill_id: String) -> bool`
```logic
var s: SkillDef = find(skill_id)
return s != null and s.learned_only
```

### `static find(skill_id: String) -> SkillDef`
```logic
all()
if not _by_id.has(skill_id):
	return null
var s: SkillDef = _by_id[skill_id]
return s
```

### `static lowest(kind: SkillDef.Kind) -> SkillDef`
```logic
for s: SkillDef in all():
	if s.kind == kind:
		return s
return null
```

### `static family_of(skill_id: String) -> String`
```logic
var s: SkillDef = find(skill_id)
return skill_id if s == null else s.family_id()
```

### `static family_name(family: String) -> String`
```logic
var tiers: Array[SkillDef] = family_tiers(family)
return family if tiers.is_empty() else tiers[0].display_name
```

### `static is_basic_family(family: String) -> bool`
```logic
for s: SkillDef in family_tiers(family):
	if s.basic:
		return true
return false
```

### `static is_always_active(skill_id: String) -> bool`
```logic
var s: SkillDef = find(skill_id)
return s != null and (s.system or is_basic_family(s.family_id()))
```

### `static roman(n: int) -> String`
```logic
return ["", "I", "II", "III", "IV", "V"][clampi(n, 0, 5)]
```

### `static group_label(group: Dictionary) -> String`
```logic
var tiers: Array[SkillDef] = group["tiers"]
if tiers.size() <= 1:
	return str(group["name"]) if tiers.is_empty() else tiers[0].display_name
var parts: Array[String] = []
for s: SkillDef in tiers:
	parts.append(roman(s.tier))
return "%s %s" % [str(group["name"]), " / ".join(parts)]
```

