class_name TestHelpers
extends RefCounted

# Shared statics for the headless suites (hand-built plan contexts, bare-resolver step runs, replay stepping).

static func resolve_until_replay(session: CombatSession, max_steps: int = 600) -> void:
	for i: int in range(max_steps):
		if session.phase == CombatSession.Phase.REPLAY:
			return
		session.step_next()

static func hand_ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant], cap: int = 3) -> PlanContext:
	var profiles: Dictionary = {}
	for c: Combatant in allies + enemies:
		var p: AiProfile = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.25, cap)
		if c.power_override <= 0:
			c.power_override = 15   # hand-built test units: fixed strike power
		if c.armor_override < 0:
			c.armor_override = 0    # and no armor, so expected numbers stay simple
		profiles[c] = p
	return PlanContext.create(grid, allies, enemies, profiles)

# Commits `steps` for `actor` and plays one full round on a bare resolver.
static func run_steps(actor: Combatant, ctx: PlanContext, steps: Array[PlanStep]) -> Array[String]:
	var resolver := CombatResolver.new()
	resolver.grid = ctx.grid
	var messages: Array[String] = []
	resolver.log_emitted.connect(func(msg: String) -> void: messages.append(msg))
	var cp := CombatPlan.new(ctx)
	cp.set_steps(actor, steps)
	cp.commit(resolver)
	resolver.resolve_round()
	return messages
