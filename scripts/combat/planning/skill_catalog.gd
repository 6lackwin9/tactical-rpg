class_name SkillCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Catalog (the command tiers available for planning)
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

static func _build() -> void:
	_all.append(_attack("strike", "Strike", 1, 2, 1.0, "A measured blow."))
	_all.append(_attack("power_strike", "Power Strike", 2, 2, 1.5, "A committed blow: +50% power."))
	_all.append(_attack("heavy_blow", "Heavy Blow", 3, 3, 2.4, "A slow, crushing blow: +140% power."))

	_all.append(_defend("guard", "Guard", 1, 2, 0.3, "Raise a light guard: -30% damage."))
	_all.append(_defend("brace", "Brace", 2, 2, 0.5, "Brace for impact: -50% damage."))
	_all.append(_defend("fortress", "Fortress", 3, 3, 0.7, "Dig in: -70% damage."))

	_all.append(_focus("focus", "Focus", 2, 1.5, 2, "+50% attack for 2 rounds."))
	_all.append(_focus("deep_focus", "Deep Focus", 3, 1.75, 3, "+75% attack for 3 rounds."))

	_all.append(SkillDef.create("move", "Move", SkillDef.Kind.MOVE, 1, 1, "Move up to your speed in tiles in one block: 1 AP per tile. Keeps your stance."))
	var rest: SkillDef = SkillDef.create("rest", "Rest", SkillDef.Kind.REST, 1, 1, "Catch your breath: recover 2 AP and return to Neutral stance, free.")
	rest.recover_ap = 2
	_all.append(rest)

static func _attack(id: String, p_name: String, level: int, blocks: int, mult: float, text: String) -> SkillDef:
	var skill: SkillDef = SkillDef.create(id, p_name, SkillDef.Kind.ATTACK, level, blocks, text)
	skill.power_mult = mult
	return skill

static func _defend(id: String, p_name: String, level: int, blocks: int, mitigation: float, text: String) -> SkillDef:
	var skill: SkillDef = SkillDef.create(id, p_name, SkillDef.Kind.DEFEND, level, blocks, text)
	skill.mitigation = mitigation
	return skill

static func _focus(id: String, p_name: String, level: int, boost: float, rounds: int, text: String) -> SkillDef:
	var skill: SkillDef = SkillDef.create(id, p_name, SkillDef.Kind.FOCUS, level, 2, text)
	skill.boost = boost
	skill.boost_rounds = rounds
	return skill

static func find(skill_id: String) -> SkillDef:
	for s: SkillDef in all():
		if s.id == skill_id:
			return s
	return null

# Skills of `kind` the unit has unlocked, lowest level first.
static func available(kind: SkillDef.Kind, skill_cap: int) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in all():
		if s.kind == kind and s.level <= skill_cap:
			result.append(s)
	return result

static func lowest(kind: SkillDef.Kind) -> SkillDef:
	for s: SkillDef in all():
		if s.kind == kind:
			return s
	return null
