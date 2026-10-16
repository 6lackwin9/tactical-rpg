class_name SkillFilter
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Filter (FORGE1 appendix J): search, filter and sort of the skill list.
# Pure predicates over RANKED skill entries (ContentCatalog.skill_entries()) and ContentCatalog.references_to, so the Forge
# list and the tests use the same code. A filter set is a Dictionary; missing keys mean "any":
#   text (substring of id / name / description template, case-insensitive), kinds (Array of kind names),
#   rank_count (0 = any), basic / learned_only ("any" | "yes" | "no"), grade ("any" | novice | advanced | expert),
#   show_system (false hides engine rows such as Move and Rest), weapon_type, inflicts, grants (status ids),
#   has_problems (bool), usage ("any" | "class:<id>" | "unit_extra" | "passive" | "requires" | "unused").
# ==============================================================================

const SORTS: Array[String] = ["name", "kind", "ap", "ranks", "problems"]

static func default_filters() -> Dictionary:
	return {"text": "", "kinds": [], "rank_count": 0, "basic": "any", "learned_only": "any", "grade": "any", "show_system": false,
		"weapon_type": "", "inflicts": "", "grants": "", "has_problems": false, "usage": "any"}

static func matches(entry: Dictionary, filters: Dictionary) -> bool:
	var base: Dictionary = SkillRanks.effective(entry, 1)
	if not DictUtil.to_bool(filters.get("show_system", false)) and DictUtil.to_bool(base.get("system", false)):
		return false
	var needle: String = str(filters.get("text", "")).strip_edges().to_lower()
	if not needle.is_empty():
		var hay: String = "%s %s %s" % [str(base["id"]), str(base["name"]), str(base["description"])]
		if not hay.to_lower().contains(needle):
			return false
	var kinds: Array = DictUtil.array_of(filters.get("kinds", []))
	if not kinds.is_empty() and not kinds.has(str(base["kind"])):
		return false
	var rank_count: int = DictUtil.to_int(filters.get("rank_count", 0))
	if rank_count > 0 and SkillRanks.rank_count(entry) != rank_count:
		return false
	if not _tri(str(filters.get("basic", "any")), DictUtil.to_bool(base["basic"])):
		return false
	if not _tri(str(filters.get("learned_only", "any")), DictUtil.to_bool(base["learned_only"])):
		return false
	var grade: String = str(filters.get("grade", "any"))
	if grade != "any" and str(base["grade"]) != grade:
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
	if DictUtil.to_bool(filters.get("has_problems", false)) and not has_problems(entry):
		return false
	var usage: String = str(filters.get("usage", "any"))
	if usage != "any" and not used_as(entry, usage):
		return false
	return true

static func _tri(mode: String, value: bool) -> bool:
	return mode == "any" or (mode == "yes" and value) or (mode == "no" and not value)

static func has_problems(entry: Dictionary) -> bool:
	return ContentCatalog.family_has_problems(str(entry.get("id", "")))

# Whether the family is used the way `usage` says (class path of one class, a unit's extra skills, a passive, a requires target, unused).
static func used_as(entry: Dictionary, usage: String) -> bool:
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

# The entries that pass `filters`, in `sort` order (one of SORTS).
static func apply(entries: Array[Dictionary], filters: Dictionary, sort: String = "name") -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for entry: Dictionary in entries:
		if matches(entry, filters):
			out.append(entry)
	sort_entries(out, sort)
	return out

static func sort_entries(list: Array[Dictionary], sort: String) -> void:
	match sort:
		"kind":
			list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return str(a.get("kind", "")) + str(a.get("name", "")) < str(b.get("kind", "")) + str(b.get("name", "")))
		"ap":
			list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_int(a.get("level", 1), 1) < DictUtil.to_int(b.get("level", 1), 1) or (DictUtil.to_int(a.get("level", 1), 1) == DictUtil.to_int(b.get("level", 1), 1) and str(a.get("name", "")) < str(b.get("name", ""))))
		"ranks":
			list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return SkillRanks.rank_count(a) > SkillRanks.rank_count(b) or (SkillRanks.rank_count(a) == SkillRanks.rank_count(b) and str(a.get("name", "")) < str(b.get("name", ""))))
		"problems":
			list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
				var pa: bool = has_problems(a)
				var pb: bool = has_problems(b)
				return (pa and not pb) or (pa == pb and str(a.get("name", "")) < str(b.get("name", ""))))
		_:
			list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return str(a.get("name", "")).to_lower() < str(b.get("name", "")).to_lower())
