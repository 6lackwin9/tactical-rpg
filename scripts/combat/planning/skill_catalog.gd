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

	var jab: SkillDef = _attack("offhand_jab", "Off-hand Jab", 1, 1, 0.5, "Melee jab with the off-hand weapon (or bare hands) at distance 1: ranged units' point-blank answer.")
	jab.offhand = true
	_all.append(jab)
	var bash: SkillDef = _attack("shield_bash", "Shield Bash", 2, 2, 0.8, "Defensive-stance strike: 80% power, may Stagger the target next round.")
	bash.stance_override = CombatTypes.StanceType.DEFENSIVE
	bash.status_effect = StatusEffects.STAGGER
	bash.status_potency = 70
	_all.append(bash)
	var hamstring: SkillDef = _attack("hamstring", "Hamstring", 2, 2, 0.8, "80% power, may Root the target (no moving) next round.")
	hamstring.status_effect = StatusEffects.ROOT
	hamstring.status_potency = 80
	_all.append(hamstring)
	var shockwave: SkillDef = _attack("shockwave", "Shockwave", 2, 2, 0.9, "Hits the 3 tiles in a line from you (aim at an adjacent tile). Friendly fire!")
	shockwave.area = SkillDef.Area.LINE
	_all.append(shockwave)
	var volley: SkillDef = _attack("volley", "Volley", 2, 2, 0.7, "Hits a tile 2-3 away and its 4 neighbours. Friendly fire!")
	volley.area = SkillDef.Area.PLUS
	volley.reach_min = 2
	volley.reach_max = 3
	_all.append(volley)
	var firebomb: SkillDef = _attack("firebomb", "Firebomb", 3, 3, 0.6, "Hits a 3x3 area around a tile 2 away. Friendly fire!")
	firebomb.area = SkillDef.Area.BURST
	firebomb.reach_min = 2
	firebomb.reach_max = 2
	_all.append(firebomb)

	_all.append(_defend("guard", "Guard", 1, 2, 0.3, "Raise a light guard: -30% damage."))
	_all.append(_defend("brace", "Brace", 2, 2, 0.5, "Brace for impact: -50% damage."))
	_all.append(_defend("fortress", "Fortress", 3, 3, 0.7, "Dig in: -70% damage."))

	_all.append(_focus("focus", "Focus", 2, 1.5, 2, "+50% attack for 2 rounds."))
	_all.append(_focus("deep_focus", "Deep Focus", 3, 1.75, 3, "+75% attack for 3 rounds."))

	_all.append(SkillDef.create("move", "Move", SkillDef.Kind.MOVE, 1, 1, "Move up to 2 blocks (speed-based tiles per block), once per round: 1 AP per tile. Keeps your stance."))
	_all.append(SkillDef.create("switch_target", "Switch Target", SkillDef.Kind.TARGET, 1, 1, "Lock onto a different enemy (no AP). Added automatically when an attack changes target."))
	_all.append(SkillDef.create("staggered", "Staggered", SkillDef.Kind.STATUS, 1, 1, "Block lost to Stagger."))
	_all.append(SkillDef.create("stunned", "Stunned", SkillDef.Kind.STATUS, 1, 1, "Block lost to Stun."))
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

# Drops the static cache. Call on shutdown: a static Array holding instances of a
# script keeps that script (and everything it references) alive past exit.
static func clear_cache() -> void:
	_all.clear()

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

# Single-target attacks for c (no area skills; the off-hand jab only for ranged units).
static func single_attacks(c: Combatant, skill_cap: int) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap):
		if s.is_area() or (s.offhand and c.attack_range <= 1):
			continue
		result.append(s)
	return result

static func area_attacks(skill_cap: int) -> Array[SkillDef]:
	var result: Array[SkillDef] = []
	for s: SkillDef in available(SkillDef.Kind.ATTACK, skill_cap):
		if s.is_area():
			result.append(s)
	return result

static func lowest(kind: SkillDef.Kind) -> SkillDef:
	for s: SkillDef in all():
		if s.kind == kind:
			return s
	return null
