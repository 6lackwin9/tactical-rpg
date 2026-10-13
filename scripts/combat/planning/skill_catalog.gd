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

static func find(skill_id: String) -> SkillDef:
	for s: SkillDef in all():
		if s.id == skill_id:
			return s
	return null

# Skills of `kind` the unit has unlocked, lowest level first.
static func available(kind: SkillDef.Kind, skill_cap: int, c: Combatant = null) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in all():
		if s.kind == kind and s.level <= skill_cap and (c == null or c.knows_skill(s.id)):
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
