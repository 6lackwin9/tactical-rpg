class_name SkillCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Catalog (the command tiers available for planning)
# One family per command kind, tiered by level 1-3 today (4-5 reserved for
# progression). A unit may use every skill up to its unlocked level
# (AiProfile.skill_cap stands in for real skill progression for now).
# Per-tile AP is flat across move tiers, so higher move levels buy time (fewer
# blocks), not distance for free.
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

	_all.append(_move("step", "Step", 1, 1, "Move 1 tile."))
	_all.append(_move("dash", "Dash", 2, 2, "Move 2 tiles in a line, in one block."))
	_all.append(_move("sprint", "Sprint", 3, 3, "Move 3 tiles in a line, in one block."))

	_all.append(SkillDef.create("reset_stance", "Reset Stance", SkillDef.Kind.NEUTRAL, 1, 1, "Return to Neutral so any action is open again."))

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

static func _move(id: String, p_name: String, level: int, steps: int, text: String) -> SkillDef:
	var skill: SkillDef = SkillDef.create(id, p_name, SkillDef.Kind.MOVE, level, 1, text)
	skill.steps = steps
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
