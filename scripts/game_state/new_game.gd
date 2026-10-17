class_name NewGame
extends RefCounted

# ==============================================================================
# Tactical RPG - New game (S13): a fresh GameState from a world and base party units.
# Members start at the base unit's authored level (XP = that level's threshold), empty spend
# (= automatic), default gear. Unknown units or worlds are refused.
# ==============================================================================

# {"state": GameState or null, "errors": Array[String]}. `p_seed` >= 0 is the save's rng_seed as given (clamped to the
# GameState range); -1 = from the system clock (EXP1-L: one deterministic random stream per save).
static func create(world_id: String, base_unit_ids: Array[String], p_seed: int = -1) -> Dictionary:
	var errors: Array[String] = []
	if not WorldCatalog.has_world(world_id):
		errors.append("Unknown world '%s'." % world_id)
	if base_unit_ids.is_empty():
		errors.append("A new game needs at least one party unit.")
	var party := PartyState.new()
	for unit_id: String in base_unit_ids:
		if not ContentCatalog.has("units", unit_id):
			errors.append("Unknown unit '%s'." % unit_id)
			continue
		var entry: Dictionary = ContentCatalog.get_entry("units", unit_id)
		var member_id: String = unit_id
		var n: int = 2
		while party.member(member_id) != null:
			member_id = "%s_%d" % [unit_id, n]
			n += 1
		party.add_member(UnitState.at_level(unit_id, DictUtil.to_int(entry.get("level", 1), 1), member_id))
	if not errors.is_empty():
		return {"state": null, "errors": errors}
	var state: GameState = GameState.create(world_id, party)
	state.rng_seed = mini(p_seed, GameState.SEED_MAX) if p_seed >= 0 else (int(Time.get_unix_time_from_system() * 1000.0) ^ Time.get_ticks_usec()) & GameState.SEED_MAX
	state.party.gold = DialogueRules.start_gold(world_id)   # DLG1-L: the nearest set manifest start_gold, else 0
	errors = state.validate()
	if not errors.is_empty():
		return {"state": null, "errors": errors}
	return {"state": state, "errors": errors}
