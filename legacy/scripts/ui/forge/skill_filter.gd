class_name SkillFilter
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Filter (FORGE1 appendix J): search, filter and sort of the skill list.
# Pure predicates over RANKED skill entries (ContentCatalog.skill_entries()) and ContentCatalog.references_to, so the Forge
# list and the tests use the same code. A filter set is a Dictionary; missing keys mean "any":
#   text (substring of id / name / description template, case-insensitive), kinds (Array of kind names),
#   stance ("any" or a stance_of label), learned_only ("any" | "yes" | "no"),
#   grade ("any" | basic | novice | advanced | expert; FORGE-VIEW: "basic" is a view grouping of the basic skills and the
#   engine rows such as Move and Rest, which the other grades leave out; the data keeps their grade), weapon_type,
#   inflicts, grants (status ids), usage ("any" | "class:<id>" | "unit_extra" | "passive" | "requires" | "unused").
# Problems are not a list filter: the validators run as tests over all shipped data (TOOLS-RO).
# ==============================================================================

const SORTS: Array[String] = ["name", "kind", "ap"]
const BASIC_GRADE: String = "basic"
const KEEPS_STANCE: String = "Keeps stance"

static func default_filters() -> Dictionary:
	return {"text": "", "kinds": [], "stance": "any", "learned_only": "any", "grade": "any",
		"weapon_type": "", "inflicts": "", "grants": "", "usage": "any"}

# The grade choices of the list: any, basic (view grouping), then the data grades.
static func grade_options() -> Array[String]:
	return ["any", BASIC_GRADE, "novice", "advanced", "expert"]

# The grade the list groups a family under: "basic" for basic skills and engine rows, else its data grade.
static func view_grade(entry: Dictionary) -> String:
	var base: Dictionary = SkillRanks.effective(entry, 1)
	if DictUtil.to_bool(base.get("basic", false)) or DictUtil.to_bool(base.get("system", false)):
		return BASIC_GRADE
	return str(base.get("grade", "novice"))

# The stance the family fights in (the engine's SkillDef.stance), or KEEPS_STANCE for kinds that keep the current one.
static func stance_of(entry: Dictionary) -> String:
	var s: SkillDef = ContentFactory.skill(SkillRanks.effective(entry, 1))
	return KEEPS_STANCE if s.keeps_stance() else CombatTypes.stance_to_string(s.stance())

# The stance choices present in `entries` (sorted; the list offers "any" before them).
static func stance_options(entries: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for entry: Dictionary in entries:
		var stance: String = stance_of(entry)
		if not out.has(stance):
			out.append(stance)
	out.sort()
	return out

static func matches(entry: Dictionary, filters: Dictionary) -> bool:
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

static func _tri(mode: String, value: bool) -> bool:
	return mode == "any" or (mode == "yes" and value) or (mode == "no" and not value)

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
		_:
			list.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return str(a.get("name", "")).to_lower() < str(b.get("name", "")).to_lower())
