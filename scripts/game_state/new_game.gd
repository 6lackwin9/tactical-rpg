class_name NewGame
extends RefCounted

# ==============================================================================
# Tactical RPG - New game (S13): a fresh GameState from a world and base party units or world party characters
# (RECRUIT1-L rule 3: a character entry becomes a hero by hero_from_character, at its base unit's level).
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
		if not ContentCatalog.has("units", unit_id) and not WorldCatalog.resolve(world_id, "characters", unit_id).is_empty():
			var hero: UnitState = hero_from_character(world_id, unit_id)
			if hero == null:
				errors.append("Character '%s' can't join the party." % unit_id)
			elif party.member(hero.id) != null:
				errors.append("Character '%s' is listed twice." % unit_id)
			else:
				party.add_member(hero)
			continue
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
	state.party.supplies = TravelRules.start_supplies(world_id)   # TRAV2-L: nearest set start_supplies, else 10
	errors = state.validate()
	if not errors.is_empty():
		return {"state": null, "errors": errors}
	return {"state": state, "errors": errors}

# A party member from world character `character_ref` (RECRUIT1-L rule 1): id = the character's unqualified id,
# base_unit = its base unit, lore_ref = its qualified id, passives = WorldFactory.character_passives (its own plus faction
# grants), empty spend, default gear, XP = the threshold of the base unit's authored level. null when the character
# does not resolve or WorldCatalog.party_character_errors refuses it.
static func hero_from_character(world_id: String, character_ref: String) -> UnitState:
	var c: Dictionary = WorldCatalog.resolve(world_id, "characters", character_ref)
	if c.is_empty() or not WorldCatalog.party_character_errors(world_id, character_ref).is_empty():
		return null
	var base_unit: String = str(c.get("base_unit", ""))
	var level: int = DictUtil.to_int(ContentCatalog.get_entry("units", base_unit).get("level", 1), 1)
	var hero: UnitState = UnitState.at_level(base_unit, level, str(c.get("id", "")))
	hero.lore_ref = str(c.get("_qid", ""))
	hero.passives = WorldFactory.character_passives(c, world_id)
	return hero

# Problems that keep `world_id` from starting a new game (UI1-L): a world is PLAYABLE when its resolved `start_party`
# and its resolved `start_location` (EXP1-L) are both non-empty. Empty = playable.
static func playable_errors(world_id: String) -> Array[String]:
	var errors: Array[String] = []
	if not WorldCatalog.has_world(world_id):
		errors.append("Unknown world '%s'." % world_id)
		return errors
	if WorldCatalog.start_party(world_id).is_empty():
		errors.append("World '%s' is not playable (no start party)." % world_id)
	if MapRules.start_location(world_id).is_empty():
		errors.append("World '%s' is not playable (no start location)." % world_id)
	return errors

# Every playable world id, in WorldCatalog order.
static func playable_worlds() -> Array[String]:
	var out: Array[String] = []
	for w: String in WorldCatalog.world_ids():
		if playable_errors(w).is_empty():
			out.append(w)
	return out

# A new game in `world_id` with its fixed start party (UI1-L): {"state", "errors"} like create().
static func create_for_world(world_id: String, p_seed: int = -1) -> Dictionary:
	var errors: Array[String] = playable_errors(world_id)
	if not errors.is_empty():
		return {"state": null, "errors": errors}
	return create(world_id, WorldCatalog.start_party(world_id), p_seed)
