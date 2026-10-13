class_name PlanContext
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Context (who is on which side, and how each unit plans)
# The read-only world a CombatPlan / TacticalAI works against: the grid, both
# sides and each unit's AiProfile (unlocked skill level, role, thresholds).
# ==============================================================================

var grid: CombatGrid = null
var allies: Array[Combatant] = []
var enemies: Array[Combatant] = []
var profiles: Dictionary = {}   # Combatant -> AiProfile
var total_blocks: int = 6       # blocks in a round

static func create(p_grid: CombatGrid, p_allies: Array[Combatant], p_enemies: Array[Combatant], p_profiles: Dictionary = {}, p_total_blocks: int = 6) -> PlanContext:
	var ctx := PlanContext.new()
	ctx.total_blocks = p_total_blocks
	ctx.grid = p_grid
	ctx.allies = p_allies
	ctx.enemies = p_enemies
	ctx.profiles = p_profiles
	return ctx

func profile_of(c: Combatant) -> AiProfile:
	var raw: Variant = profiles.get(c, null)
	if raw is AiProfile:
		var stored: AiProfile = raw
		return stored
	var is_ranged: bool = c.attack_range > 1
	return AiProfile.create(AiProfile.Role.SNIPER if is_ranged else AiProfile.Role.AGGRESSOR)

# The unit's attack power for its basic attack (DamageCalculator).
func power_of(c: Combatant) -> int:
	return DamageCalculator.power_of(c, c.attack_range > 1)

func skill_cap_of(c: Combatant) -> int:
	return profile_of(c).skill_cap

func foes_of(c: Combatant) -> Array[Combatant]:
	if allies.has(c):
		return enemies
	if enemies.has(c):
		return allies
	var none: Array[Combatant] = []
	return none

func friends_of(c: Combatant) -> Array[Combatant]:
	if allies.has(c):
		return allies
	if enemies.has(c):
		return enemies
	var none: Array[Combatant] = []
	return none
