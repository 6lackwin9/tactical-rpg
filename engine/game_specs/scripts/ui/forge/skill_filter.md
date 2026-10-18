---
class: SkillFilter
extends: RefCounted
description: "Tactical RPG - Skill Filter (FORGE1 appendix J): search, filter and sort of the skill list. Pure predicates over RANKED skill entries (ContentCatalog.skill_entries()) and ContentCatalog.references_to, so the Forge list and the tests use the same code. A filter set is a Dictionary; missing keys mean \"any\": text (substring of id / name / description template, case-insensitive), kinds (Array of kind names), stance (\"any\" or a stance_of label), learned_only (\"any\" | \"yes\" | \"no\"), grade (\"any\" | basic | novice | advanced | expert; FORGE-VIEW: \"basic\" is a view grouping of the basic skills and the engine rows such as Move and Rest, which the other grades leave out; the data keeps their grade), weapon_type, inflicts, grants (status ids), usage (\"any\" | \"class:<id>\" | \"unit_extra\" | \"passive\" | \"requires\" | \"unused\"). Problems are not a list filter: the validators run as tests over all shipped data (TOOLS-RO)."
---

# Properties
* `SORTS`: Array[String] = ["name", "kind", "ap"] (const)
* `BASIC_GRADE`: String = "basic" (const)
* `KEEPS_STANCE`: String = "Keeps stance" (const)
* `out`: Array[String] = []
* `stance`: String = stance_of(entry)
* `out`: Array[Dictionary] = []

# Methods

### `static default_filters() -> Dictionary`
```logic
return {"text": "", "kinds": [], "stance": "any", "learned_only": "any", "grade": "any",
	"weapon_type": "", "inflicts": "", "grants": "", "usage": "any"}
```

### `static view_grade(entry: Dictionary) -> String`
```logic
var base: Dictionary = SkillRanks.effective(entry, 1)
if DictUtil.to_bool(base.get("basic", false)) or DictUtil.to_bool(base.get("system", false)):
	return BASIC_GRADE
return str(base.get("grade", "novice"))
```

### `static stance_of(entry: Dictionary) -> String`
```logic
var s: SkillDef = ContentFactory.skill(SkillRanks.effective(entry, 1))
return KEEPS_STANCE if s.keeps_stance() else CombatTypes.stance_to_string(s.stance())
```

### `static matches(entry: Dictionary, filters: Dictionary) -> bool`
```logic
var base: Dictionary = SkillRanks.effective(entry, 1)
var needle: String = str(filters.get("text", "")).strip_edges().to_lower()
if not needle.is_empty():
	var hay: String = "%s %s %s" % [str(base["id"]), str(base["name"]), str(base["description"])]
	if not hay.to_lower().contains(needle):
		return false
var kinds: Array = DictUtil.array_of(filters.get("kinds", []))
if not kinds.is_empty() and not kinds.has(str(base["kind"])):
	return false
var stance: String = str(filters.get("stance", "any"))
if stance != "any" and stance_of(entry) != stance:
	return false
if not _tri(str(filters.get("learned_only", "any")), DictUtil.to_bool(base["learned_only"])):
	return false
var grade: String = str(filters.get("grade", "any"))
if grade != "any" and view_grade(entry) != grade:
	return false
var weapon: String = str(filters.get("weapon_type", ""))
if not weapon.is_empty() and not ContentFactory._strings(base["weapon_types"]).has(weapon):
	return false
var inflicts: String = str(filters.get("inflicts", ""))
if not inflicts.is_empty() and str(base["status_effect"]) != inflicts:
	return false
var grants: String = str(filters.get("grants", ""))
if not grants.is_empty() and str(base["grant_status"]) != grants:
	return false
var usage: String = str(filters.get("usage", "any"))
if usage != "any" and not used_as(entry, usage):
	return false
return true
```

### `static _tri(mode: String, value: bool) -> bool`
```logic
return mode == "any" or (mode == "yes" and value) or (mode == "no" and not value)
```

### `static used_as(entry: Dictionary, usage: String) -> bool`
```logic
var head: String = str(entry.get("id", ""))
var ids: Array[String] = []
for rank: int in range(1, SkillRanks.rank_count(entry) + 1):
	ids.append(SkillRanks.rank_id(head, rank))
var users: Array[String] = []
for rank_id: String in ids:
	for user: String in ContentCatalog.references_to("skills", rank_id):
		if not users.has(user):
			users.append(user)
if usage == "unused":
	return users.is_empty()
if usage.begins_with("class:"):
	return users.has("classes:%s" % usage.trim_prefix("class:"))
if usage == "unit_extra":
	return users.any(func(u: String) -> bool: return u.begins_with("units:"))
if usage == "passive":
	return users.any(func(u: String) -> bool: return u.begins_with("passives:"))
if usage == "requires":
	return users.any(func(u: String) -> bool: return u.begins_with("skills:"))
return true
```

### `static sort_entries(list: Array[Dictionary], sort: String) -> void`
```logic
match sort:
	"kind":
		list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return str(a.get("kind", "")) + str(a.get("name", "")) < str(b.get("kind", "")) + str(b.get("name", "")))
	"ap":
		list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_int(a.get("level", 1), 1) < DictUtil.to_int(b.get("level", 1), 1) or (DictUtil.to_int(a.get("level", 1), 1) == DictUtil.to_int(b.get("level", 1), 1) and str(a.get("name", "")) < str(b.get("name", ""))))
	_:
		list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return str(a.get("name", "")).to_lower() < str(b.get("name", "")).to_lower())
```

