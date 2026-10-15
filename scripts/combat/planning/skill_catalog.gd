class_name SkillCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Catalog (the command tiers available for planning)
# Built from the ContentCatalog "skills" entries (edit them in the Content Forge).
# One family per command kind, tiered by level 1-3 today (4-5 reserved for
# progression). A unit may use every skill up to its unlocked level
# (AiProfile.skill_cap stands in for real skill progression for now).
# Move and Rest are single skills: Move is one block at 1 AP per tile (up to
# Combatant.move_per_block() tiles), Rest is one free block that recovers AP.
# ==============================================================================

static var _all: Array[SkillDef] = []

static func all() -> Array[SkillDef]:
	if _all.is_empty():
		_build()
	return _all

# Skills come from the content catalog (res://data/catalog/skills.json), in file
# order: the first skill of a kind is its "lowest" (Strike, Move...).
static func _build() -> void:
	for entry: Dictionary in ContentCatalog.entries("skills"):
		_all.append(ContentFactory.skill(entry))

# Drops the static cache. Call on shutdown: a static Array holding instances of a
# script keeps that script (and everything it references) alive past exit.
static func clear_cache() -> void:
	_all.clear()

static func is_system(skill_id: String) -> bool:
	var s: SkillDef = find(skill_id)
	return s != null and s.system

static func is_learned_only(skill_id: String) -> bool:
	var s: SkillDef = find(skill_id)
	return s != null and s.learned_only

static func find(skill_id: String) -> SkillDef:
	for s: SkillDef in all():
		if s.id == skill_id:
			return s
	return null

# Skills of `kind` the unit has unlocked, lowest level first.
static func available(kind: SkillDef.Kind, skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in all():
		if s.kind == kind and s.level <= skill_cap and (c == null or c.can_use(s)):
			result.append(s)
	return result

# Single-target attacks for c (no area skills; the off-hand jab only for ranged units).
static func single_attacks(c: Combatant, skill_cap: int) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap, c):
		if s.is_area() or (s.offhand and c.attack_range <= 1):
			continue
		result.append(s)
	return result

static func area_attacks(skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap, c):
		if s.is_area():
			result.append(s)
	return result

static func lowest(kind: SkillDef.Kind) -> SkillDef:
	for s: SkillDef in all():
		if s.kind == kind:
			return s
	return null

# ------------------------------------------------------------------------------
# Skill families (tiers of one skill, chosen at planning time)
# ------------------------------------------------------------------------------

# The family of a skill (its own id when it stands alone or is unknown).
static func family_of(skill_id: String) -> String:
	var s: SkillDef = find(skill_id)
	return skill_id if s == null else s.family_id()

# Every tier of `family`, lowest tier first (ties keep file order).
static func family_tiers(family: String) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in all():
		if s.family_id() == family:
			result.append(s)
	result.sort_custom(func(a: SkillDef, b: SkillDef) -> bool: return a.tier < b.tier)
	return result

# Every family id, in file order of its first skill.
static func families() -> Array[String]:
	var result: Array[String] = []
	for s: SkillDef in all():
		if not result.has(s.family_id()):
			result.append(s.family_id())
	return result

# Display name of a family: its lowest tier ("Mend" for Mend I / II / III).
static func family_name(family: String) -> String:
	var tiers: Array[SkillDef] = family_tiers(family)
	return family if tiers.is_empty() else tiers[0].display_name

# A basic family (any tier flagged basic) is available to every class and costs no loadout slot.
static func is_basic_family(family: String) -> bool:
	for s: SkillDef in family_tiers(family):
		if s.basic:
			return true
	return false

# Basic or engine skill: always passes the active-loadout filter.
static func is_always_active(skill_id: String) -> bool:
	var s: SkillDef = find(skill_id)
	return s != null and (s.system or is_basic_family(s.family_id()))

# `kind` skills the unit can plan, grouped by family with the affordable tiers (tier <= cap by AP level)
# lowest first: [{"family": id, "name": text, "tiers": Array[SkillDef]}], families in file order.
static func grouped(kind: SkillDef.Kind, skill_cap: int, c: Combatant = null) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for s: SkillDef in available(kind, skill_cap, c):
		var fam: String = s.family_id()
		var group: Dictionary = {}
		for g: Dictionary in out:
			if str(g["family"]) == fam:
				group = g
		if group.is_empty():
			var tiers: Array[SkillDef] = []
			group = {"family": fam, "name": family_name(fam), "tiers": tiers}
			out.append(group)
		var list: Array[SkillDef] = group["tiers"]
		list.append(s)
	for g: Dictionary in out:
		var list: Array[SkillDef] = g["tiers"]
		list.sort_custom(func(a: SkillDef, b: SkillDef) -> bool: return a.tier < b.tier)
	return out

static func roman(n: int) -> String:
	return ["", "I", "II", "III", "IV", "V"][clampi(n, 0, 5)]

# "Mend I / II / III" for a group from grouped() (single tier: just the name).
static func group_label(group: Dictionary) -> String:
	var tiers: Array[SkillDef] = group["tiers"]
	if tiers.size() <= 1:
		return str(group["name"]) if tiers.is_empty() else tiers[0].display_name
	var parts: Array[String] = []
	for s: SkillDef in tiers:
		parts.append(roman(s.tier))
	return "%s %s" % [str(group["name"]), " / ".join(parts)]
