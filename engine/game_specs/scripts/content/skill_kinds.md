---
class: SkillKinds
extends: RefCounted
description: "Tactical RPG - Skill Kinds (one descriptor table per SkillDef.Kind, FORGE1 section 4) Which Forge sections a kind has, whether it can be authored, its stance rule, AP rule, cast target and icon. SkillDef, the schema (`when` lists), the validator, SkillRanks and the Forge read this table; nothing else restates the kind lists. Keys are the kind NAMES (\"ATTACK\") so this file does not depend on the SkillDef enum. Skills the engine and the planner look up by id (validate_all invariant; never deleted)."
---

# Properties
* `REQUIRED_IDS`: Array[String] = ["strike", "guard", "move", "rest", "switch_target", "offhand_jab"] (const)
* `KINDS`: Dictionary = { (const)
* `SECTION_FIELDS`: Dictionary = { (const)
* `COMMON_FIELDS`: Array[String] = ["id", "name", "kind", "level", "blocks", "description", "template", "system", "basic", "learned_only", "grade", "requires", "origin", "unlock_offset", "family", "tier"] (const)
* `KIND_ONLY_FIELDS`: Dictionary = {"stance": ["ATTACK"]} (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[String] = []
* `raw`: Variant = info(kind).get("sections", [])
* `list`: Array = raw
* `out`: Array[String] = []
* `raw`: Variant = SECTION_FIELDS.get(section, [])
* `list`: Array = raw
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []

# Methods

### `static info(kind: String) -> Dictionary`
```logic
var raw: Variant = KINDS.get(kind, {})
var d: Dictionary = raw if raw is Dictionary else {}
return d
```

### `static is_authorable(kind: String) -> bool`
```logic
return DictUtil.to_bool(info(kind).get("authorable", false))
```

### `static has_field(kind: String, field: String) -> bool`
```logic
if COMMON_FIELDS.has(field):
	return true
if KIND_ONLY_FIELDS.has(field):
	var only: Array = KIND_ONLY_FIELDS[field]
	return only.has(kind)
return not sections_with(kind, field).is_empty()
```

### `static plans_on_self(kind: String) -> bool`
```logic
if kind == "SUPPORT":
	return true
return str(info(kind).get("cast", "")) == "SELF" and (is_authorable(kind) or kind == "REST")
```

### `static label_of(kind: String) -> String`
```logic
return str(info(kind).get("label", kind.capitalize()))
```

### `static icon_of(kind: String) -> String`
```logic
return str(info(kind).get("icon", "💤"))
```

