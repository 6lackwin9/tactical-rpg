class_name PartyCarry
extends RefCounted

# ==============================================================================
# Tactical RPG - Party Carry Mechanic (party-wide) vs Unit Burden (per-unit)
#
#   BURDEN  - per unit. Sum of the unit's equipped gear `burden`. Each point
#             lowers that unit's initiative speed by 1 (CharacterStats).
#   CARRY   - party-wide. One shared capacity for everything the party owns:
#               capacity = BASE_CAPACITY + sum over members of (STR * 3 + CONS)
#               load     = stash items + all equipped gear + all battle pouches
#             Light     (< 75% of capacity): no effect
#             Heavy     (75% - 100%):        warning only
#             Overloaded (> 100%):           EVERY unit loses 1 initiative speed
#                                            per 10 weight over capacity (rounded up)
# ==============================================================================

const BASE_CAPACITY: int = 30
const HEAVY_RATIO: float = 0.75
const OVERLOAD_STEP: float = 10.0

# Report keys:
#   capacity (int), load (float), stash_weight, equipped_weight, pouch_weight (float),
#   ratio (float), tier ("LIGHT" | "HEAVY" | "OVERLOADED"), speed_penalty (int),
#   members: Array[Dictionary] of {"name": String, "contribution": int}
static func evaluate(party: Array[Combatant], stash: Inventory) -> Dictionary:
	var members: Array[Dictionary] = []
	var capacity: int = BASE_CAPACITY
	var equipped_weight: float = 0.0
	var pouch_weight: float = 0.0
	var stash_weight: float = 0.0

	if stash != null:
		for item: Item in stash.items:
			stash_weight += item.weight

	for hero: Combatant in party:
		var contribution: int = 0
		if hero.stats != null:
			contribution = hero.stats.get_carry_contribution(hero.get_weapon_context())
		capacity += contribution
		members.append({"name": hero.name, "contribution": contribution})
		if hero.inventory == null:
			continue
		for eq: Equipment in hero.inventory._iter_equipped_gear():
			equipped_weight += eq.weight
		for con: Consumable in hero.inventory.battle_pouch:
			pouch_weight += con.weight
		# A hero's private bag counts too (unless it IS the stash, e.g. a lone arena hero).
		if hero.inventory != stash:
			for item: Item in hero.inventory.items:
				stash_weight += item.weight

	var total: float = stash_weight + equipped_weight + pouch_weight
	var ratio: float = total / float(maxi(1, capacity))
	var tier: String = "LIGHT"
	var penalty: int = 0
	if ratio > 1.0:
		tier = "OVERLOADED"
		penalty = ceili((total - float(capacity)) / OVERLOAD_STEP)
	elif ratio >= HEAVY_RATIO:
		tier = "HEAVY"
	return {
		"capacity": capacity,
		"load": total,
		"stash_weight": stash_weight,
		"equipped_weight": equipped_weight,
		"pouch_weight": pouch_weight,
		"ratio": ratio,
		"tier": tier,
		"speed_penalty": penalty,
		"members": members,
	}

# Evaluates and applies the overload penalty to every member, returning the report.
static func apply(party: Array[Combatant], stash: Inventory) -> Dictionary:
	var report: Dictionary = evaluate(party, stash)
	var penalty: int = 0
	var raw_penalty: Variant = report.get("speed_penalty", 0)
	if raw_penalty is int:
		penalty = raw_penalty
	for hero: Combatant in party:
		hero.carry_speed_penalty = penalty
		hero.sync_from_stats_and_gear()
	return report
