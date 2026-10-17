class_name SkillKinds
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Kinds (one descriptor table per SkillDef.Kind, FORGE1 section 4)
# Which Forge sections a kind has, whether it can be authored, its stance rule, AP rule, cast
# target and icon. SkillDef, the schema (`when` lists), the validator, SkillRanks and the
# Forge read this table; nothing else restates the kind lists.
# Keys are the kind NAMES ("ATTACK") so this file does not depend on the SkillDef enum.
# ==============================================================================

# Skills the engine and the planner look up by id (validate_all invariant; never deleted).
const REQUIRED_IDS: Array[String] = ["strike", "guard", "move", "rest", "switch_target", "offhand_jab"]

# stance: OFFENSIVE / DEFENSIVE / TACTICAL / NEUTRAL, KEEP (keeps the unit's stance), KEEP_UNLESS_SET (SUPPORT: keeps it unless
# the row sets a stance override). ap: LEVEL (AP = level), PER_TILE (Move), FREE. cast: whom the skill is aimed at.
const KINDS: Dictionary = {
	"ATTACK": {"label": "Attack", "authorable": true, "sections": ["damage", "inflict", "reach", "area", "gate"], "stance": "OFFENSIVE", "ap": "LEVEL", "cast": "FOE", "icon": "⚔"},
	"DEFEND": {"label": "Defend", "authorable": true, "sections": ["grant", "guard", "gate"], "stance": "DEFENSIVE", "ap": "LEVEL", "cast": "SELF", "icon": "🛡"},
	"FOCUS": {"label": "Focus", "authorable": true, "sections": ["focus"], "stance": "TACTICAL", "ap": "LEVEL", "cast": "SELF", "icon": "✦"},
	"SUPPORT": {"label": "Support", "authorable": true, "sections": ["reach", "support", "grant", "gate"], "stance": "KEEP_UNLESS_SET", "ap": "LEVEL", "cast": "ALLY", "icon": "+"},
	"DISENGAGE": {"label": "Disengage", "authorable": true, "sections": ["jump"], "stance": "KEEP", "ap": "LEVEL", "cast": "SELF", "icon": "⤴"},
	"REST": {"label": "Rest", "authorable": false, "sections": ["rest"], "stance": "TACTICAL", "ap": "FREE", "cast": "SELF", "icon": "💤"},
	"MOVE": {"label": "Move", "authorable": false, "sections": [], "stance": "KEEP", "ap": "PER_TILE", "cast": "TILE", "icon": "➜"},
	"TARGET": {"label": "Switch target", "authorable": false, "sections": [], "stance": "KEEP", "ap": "FREE", "cast": "FOE", "icon": "🎯"},
	"STATUS": {"label": "Status block", "authorable": false, "sections": [], "stance": "KEEP", "ap": "FREE", "cast": "SELF", "icon": "✖"},
}

# Forge section id -> the skill fields it holds (a field may sit in two sections: status_magnitude, status_rounds).
const SECTION_FIELDS: Dictionary = {
	"damage": ["power_mult", "weapon_hand", "dash_tiles"],
	"inflict": ["status_effect", "status_potency", "status_magnitude", "status_rounds"],
	"reach": ["reach_min", "reach_max"],
	"area": ["area"],
	"support": ["target_mode", "heal_stat", "heal_base", "heal_mult", "cleanse_statuses"],
	"grant": ["grant_status", "status_magnitude", "status_rounds"],
	"guard": ["mitigation"],
	"focus": ["boost", "boost_rounds"],
	"jump": ["jump_tiles"],
	"rest": ["recover_ap"],
	"gate": ["weapon_types", "min_proficiency"],
}

# Fields every kind has (no section needed).
const COMMON_FIELDS: Array[String] = ["id", "name", "kind", "level", "blocks", "description", "template", "system", "basic", "learned_only", "grade", "requires", "origin", "unlock_offset", "family", "tier"]

# Kind-specific fields that sit outside the sections (the stance override is an ATTACK-only authoring choice).
const KIND_ONLY_FIELDS: Dictionary = {"stance": ["ATTACK"]}

static func names() -> Array[String]:
	var out: Array[String] = []
	for key: Variant in KINDS:
		out.append(str(key))
	return out

static func info(kind: String) -> Dictionary:
	var raw: Variant = KINDS.get(kind, {})
	var d: Dictionary = raw if raw is Dictionary else {}
	return d

static func authorable() -> Array[String]:
	var out: Array[String] = []
	for k: String in names():
		if DictUtil.to_bool(info(k).get("authorable", false)):
			out.append(k)
	return out

static func is_authorable(kind: String) -> bool:
	return DictUtil.to_bool(info(kind).get("authorable", false))

static func sections_of(kind: String) -> Array[String]:
	var out: Array[String] = []
	var raw: Variant = info(kind).get("sections", [])
	if raw is Array:
		var list: Array = raw
		for s: Variant in list:
			out.append(str(s))
	return out

static func section_fields(section: String) -> Array[String]:
	var out: Array[String] = []
	var raw: Variant = SECTION_FIELDS.get(section, [])
	if raw is Array:
		var list: Array = raw
		for f: Variant in list:
			out.append(str(f))
	return out

# Sections of `kind` that hold `field` (empty = the kind has no section for it).
static func sections_with(kind: String, field: String) -> Array[String]:
	var out: Array[String] = []
	for s: String in sections_of(kind):
		if section_fields(s).has(field):
			out.append(s)
	return out

# All section-bound field keys (every field some kind section holds).
static func sectioned_fields() -> Array[String]:
	var out: Array[String] = []
	for key: Variant in SECTION_FIELDS:
		for f: String in section_fields(str(key)):
			if not out.has(f):
				out.append(f)
	return out

# True when `kind` shows `field`: a common field, a field of one of its sections, or a kind-only field.
static func has_field(kind: String, field: String) -> bool:
	if COMMON_FIELDS.has(field):
		return true
	if KIND_ONLY_FIELDS.has(field):
		var only: Array = KIND_ONLY_FIELDS[field]
		return only.has(kind)
	return not sections_with(kind, field).is_empty()

# Kinds that show `field` (generates the schema `when` lists).
static func kinds_with(field: String) -> Array[String]:
	var out: Array[String] = []
	for k: String in names():
		if has_field(k, field):
			out.append(k)
	return out

# The fields of `kind` the Forge shows, section by section: [{"section": id, "fields": [...]}].
static func layout(kind: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for s: String in sections_of(kind):
		out.append({"section": s, "fields": section_fields(s)})
	return out

# True when the planner offers the kind when the caster clicks itself: kinds cast on SELF that a player can plan (authorable
# kinds and Rest; not engine status blocks) and SUPPORT (the skill's own target mode decides).
static func plans_on_self(kind: String) -> bool:
	if kind == "SUPPORT":
		return true
	return str(info(kind).get("cast", "")) == "SELF" and (is_authorable(kind) or kind == "REST")

static func label_of(kind: String) -> String:
	return str(info(kind).get("label", kind.capitalize()))

static func icon_of(kind: String) -> String:
	return str(info(kind).get("icon", "💤"))

# The label of the weapon-gate kinds, for messages: "ATTACK, DEFEND, SUPPORT".
static func gate_kinds() -> Array[String]:
	return kinds_with("weapon_types")
