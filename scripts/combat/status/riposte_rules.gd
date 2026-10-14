class_name RiposteRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Riposte Rules (reactive counter damage, milestone M2a)
# A unit with the `riposte` status (granted by a DEFEND skill, see DefendCommand) counters
# an attacker once per attacker per round when ALL of these hold:
#   - the attack is melee (not a ranged shot; an off-hand jab at distance 1 is melee)
#   - the attacker stands adjacent (distance 1) to the defender when the hit lands
#   - the defender survived the hit and the attacker is still on the field
# The counter is a normal DamageCalculator.compute hit with skill_mult = COUNTER_MULT
# (50% of the defender's attack power; Focus, stance matchup, crit, guard and armor apply
# as always). Riposte is removed by any stance change (Combatant.set_stance).
# ==============================================================================

const COUNTER_MULT: float = 0.5
const GUARD_MITIGATION: float = 0.15   # the example skill's guard; always below Guard (0.3)

# True when `defender` would counter `attacker`'s melee hit right now.
static func can_counter(attacker: Combatant, defender: Combatant, grid: CombatGrid, ranged: bool) -> bool:
	if ranged or attacker == null or defender == null or grid == null or attacker == defender:
		return false
	if not defender.has_status(StatusEffects.RIPOSTE) or not defender.is_alive() or not attacker.is_active():
		return false
	if defender.riposted.has(attacker):
		return false
	if not grid.has_combatant(attacker) or not grid.has_combatant(defender):
		return false
	return grid.get_distance(grid.get_position(attacker), grid.get_position(defender)) == 1

# Resolves the counter if it applies. Returns {} or
# {defender, attacker, damage, crit, breakdown, defeated, log}.
static func counter(attacker: Combatant, defender: Combatant, grid: CombatGrid, ranged: bool, rng: RandomNumberGenerator) -> Dictionary:
	if not can_counter(attacker, defender, grid, ranged):
		return {}
	defender.riposted[attacker] = true
	var res: Dictionary = DamageCalculator.compute(defender, attacker, false, false, COUNTER_MULT, false, rng)
	var taken: int = attacker.take_damage(DictUtil.to_int(res["final_dmg"]), true)
	var crit: bool = DictUtil.to_bool(res["is_crit"])
	var defeated: bool = not attacker.is_alive()
	var text: String = "[Riposte] %s counters %s for %d damage%s%s" % [
		defender.name, attacker.name, taken, " (CRIT)" if crit else "",
		(" -> %s DEFEATED!" % attacker.name) if defeated else (" (Target HP: %d/%d)" % [attacker.hp, attacker.max_hp])
	]
	return {"defender": defender, "attacker": attacker, "damage": taken, "crit": crit, "breakdown": DamageCalculator.describe(res), "defeated": defeated, "log": text}
