class_name LevelUp
extends RefCounted

# ==============================================================================
# Tactical RPG - Level-up rules (S7): award XP, confirm a spend draft, automatic spend.
# Confirmed points are permanent (no respec). Pure functions over UnitState / PartyState.
# ==============================================================================

# Adds `xp` (>= 0) to the state. Returns {levels_gained, old_level, new_level, unspent}.
static func award(state: UnitState, xp: int) -> Dictionary:
	var old_level: int = state.level()
	state.total_xp += maxi(0, xp)
	var new_level: int = state.level()
	return {"levels_gained": new_level - old_level, "old_level": old_level, "new_level": new_level, "unspent": state.unspent()}

# Full encounter XP to every ACTIVE member (a downed one included, benched members get none).
# `enemies`: [{"level": int, "tier": String}]. Returns member id -> award result (+ "xp").
static func award_encounter(party: PartyState, enemies: Array[Dictionary]) -> Dictionary:
	var out: Dictionary = {}
	for m: UnitState in party.active_members():
		var xp: int = XpCurve.encounter_xp(enemies, m.level())
		var res: Dictionary = award(m, xp)
		res["xp"] = xp
		out[m.id] = res
	return out

# Adds `draft` (stat -> points) to the state's spend. Errors leave the state untouched.
static func confirm(state: UnitState, draft: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var sum: int = 0
	for key: Variant in draft:
		if not StatRules.STATS.has(str(key)):
			errors.append("Unknown stat '%s'." % str(key))
		elif not (draft[key] is int) or DictUtil.to_int(draft[key]) < 0:
			errors.append("Draft %s must be a whole number >= 0." % str(key))
		else:
			sum += DictUtil.to_int(draft[key])
	if sum > state.unspent():
		errors.append("Draft spends %d, only %d unspent." % [sum, state.unspent()])
	if not errors.is_empty():
		return errors
	for key: Variant in draft:
		state.spend[str(key)] = DictUtil.to_int(state.spend.get(str(key), 0)) + DictUtil.to_int(draft[key])
	return errors

# Spends every unspent point by the class growth weights (sims, presets, builder).
static func auto(state: UnitState, class_growth: Dictionary) -> void:
	var left: int = state.unspent()
	if left <= 0:
		return
	var extra: Dictionary = StatRules.auto_spend(class_growth, left)
	for key: Variant in extra:
		state.spend[str(key)] = DictUtil.to_int(state.spend.get(str(key), 0)) + DictUtil.to_int(extra[key])

# `auto` with the growth of the state's own class.
static func auto_for(state: UnitState) -> void:
	var arch: ClassArchetype = state.archetype()
	auto(state, arch.growth_per_level if arch != null else {})
