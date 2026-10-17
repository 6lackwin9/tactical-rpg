class_name CombatExtKit
extends RefCounted

# Shared helpers of the combat extensions suites (CombatExtensions, CombatExtAttacks, CombatExtAi):
# hand-built units (fixed strike power 15, no armor), plan contexts and a one-round play on a bare resolver.

static func new_grid(exits: Array[Vector2i] = []) -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, exits)

static func unit(unit_name: String, hp: int, speed: int, attack_range: int, grid: CombatGrid, pos: Vector2i, learn: Array[String] = []) -> Combatant:
	return TestHelpers.unit(unit_name, grid, pos, hp, speed, attack_range, 15, learn)   # strike power 15, no armor

static func new_ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant], profiles: Dictionary = {}) -> PlanContext:
	var all_profiles: Dictionary = profiles.duplicate()
	for c: Combatant in allies + enemies:
		if not all_profiles.has(c):
			all_profiles[c] = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.0, 3)
	return PlanContext.create(grid, allies, enemies, all_profiles)

static func profile(role: AiProfile.Role, flee: float = 0.0) -> AiProfile:
	return AiProfile.create(role, flee, 3)

# Commits the given steps and plays one full round on a bare resolver.
class Played:
	extends RefCounted
	var resolver: CombatResolver = null
	var messages: Array[String] = []
	var actions: Array[CombatResolver.ScheduledAction] = []

static func play(ctx: PlanContext, plans: Dictionary) -> Played:
	var played := Played.new()
	played.resolver = CombatResolver.new()
	played.resolver.grid = ctx.grid
	played.resolver.log_emitted.connect(func(msg: String) -> void: played.messages.append(msg))
	var cp := CombatPlan.new(ctx)
	for key: Variant in plans:
		if key is Combatant:
			var actor: Combatant = key
			var steps: Array[PlanStep] = []
			var raw: Variant = plans[key]
			if raw is Array:
				var raw_list: Array = raw
				steps.assign(raw_list)
			cp.set_steps(actor, steps)
	cp.commit(played.resolver)
	played.actions = played.resolver.scheduled_actions.duplicate()
	played.resolver.resolve_round()
	return played

static func has_message(played: Played, prefix: String) -> bool:
	for m: String in played.messages:
		if m.begins_with(prefix):
			return true
	return false

static func any_skill_kind(steps: Array[PlanStep], kind: SkillDef.Kind) -> bool:
	for s: PlanStep in steps:
		if s.skill != null and s.skill.kind == kind:
			return true
	return false

static func any_mode(steps: Array[PlanStep], mode: PlanStep.Mode) -> bool:
	for s: PlanStep in steps:
		if s.mode == mode:
			return true
	return false

static func base_move(c: Combatant) -> int:
	return clampi(floori(float(c.speed + 2) / 5.0), 1, 4)
