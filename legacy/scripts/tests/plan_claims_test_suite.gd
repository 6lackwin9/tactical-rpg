class_name PlanClaimsTestSuite
extends RefCounted

# PLAN1: allies' planned approach tiles are claims that later drafts respect. A slower ally plans first
# and holds the only tile in reach of a foe; a faster ally planned after it (evaluated first in the
# fastest-first occupancy pass) must not take that tile, so the party plan stays valid and confirms.
# Hand-built grid and units (no catalog writes); expectations come from the plan rules.

const TAG: String = "PlanClaims"

# A hand-built battle: one foe walled in on three sides, so exactly one melee approach tile exists.
class ClaimEncounter:
	extends CombatEncounter

	var foe_cell: Vector2i = Vector2i(4, 4)
	var walls: Array[Vector2i] = [Vector2i(5, 4), Vector2i(4, 5), Vector2i(4, 3)]
	var slow_cell: Vector2i = Vector2i(3, 7)
	var fast_cell: Vector2i = Vector2i(1, 4)

	func build() -> void:
		max_rounds = 1
		grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
		for w: Vector2i in walls:
			grid.set_terrain(w, Terrain.Type.WALL)
		var slow: Combatant = TestHelpers.unit("Slow", grid, slow_cell, 400, 10)
		var fast: Combatant = TestHelpers.unit("Fast", grid, fast_cell, 400, 15)
		var foe: Combatant = TestHelpers.unit("Foe", grid, foe_cell, 400, 5)
		allies = [slow, fast]
		enemies = [foe]
		ai_profiles.clear()
		for c: Combatant in allies + enemies:
			ai_profiles[c] = AiProfile.create(AiProfile.Role.AGGRESSOR, 0.0, 3)

static func run(_tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== PLAN CLAIMS (allies' approach tiles) ASSERTIONS ===")
	print("=======================================================")
	var ok: bool = true
	ok = _hand_plan() and ok
	ok = _session_confirm() and ok
	ok = _preset_confirm() and ok
	ok = _ally_approach_pinned() and ok
	return ok

# The melee approach tiles of `foe` (distance 1, walkable, empty at the start).
static func _approach_tiles(grid: CombatGrid, foe: Combatant) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for d: Vector2i in CombatPlan.DIRS:
		var cell: Vector2i = grid.get_position(foe) + d
		if grid.is_walkable(cell) and not grid.is_exit_tile(cell) and not grid.is_occupied(cell):
			out.append(cell)
	return out

static func _fixture() -> ClaimEncounter:
	var enc := ClaimEncounter.new()
	enc.build()
	return enc

# Fixture sanity shared by the checks: one approach tile, both allies can reach it this round,
# and the ally planned second is the faster one (so it is evaluated first).
static func _fixture_checks(enc: ClaimEncounter, plan: CombatPlan) -> Dictionary:
	var slow: Combatant = enc.allies[0]
	var fast: Combatant = enc.allies[1]
	var tiles: Array[Vector2i] = _approach_tiles(enc.grid, enc.enemies[0])
	var tile: Vector2i = tiles[0] if tiles.size() == 1 else CombatPlan.NO_CELL
	return {
		"one approach tile": tiles.size() == 1,
		"slow ally reaches it": tile != CombatPlan.NO_CELL and enc.grid.get_distance(enc.grid.get_position(slow), tile) <= plan.max_move_tiles(slow),
		"fast ally reaches it": tile != CombatPlan.NO_CELL and enc.grid.get_distance(enc.grid.get_position(fast), tile) <= plan.max_move_tiles(fast),
		"second ally is faster": fast.speed > slow.speed,
	}

# Plan level: the slower ally claims the tile first; the faster ally's attack is refused (no free
# approach), the slower ally's plan stays valid and keeps its tile.
static func _hand_plan() -> bool:
	var enc: ClaimEncounter = _fixture()
	var plan := CombatPlan.new(enc.plan_context())
	var checks: Dictionary = _fixture_checks(enc, plan)
	var slow: Combatant = enc.allies[0]
	var fast: Combatant = enc.allies[1]
	var foe: Combatant = enc.enemies[0]
	var strike: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
	var tile: Vector2i = _approach_tiles(enc.grid, foe)[0]
	var slow_problem: String = plan.add_step(slow, PlanStep.engage(strike, foe))
	var fast_problem: String = plan.add_step(fast, PlanStep.engage(strike, foe))
	checks["slow ally plans the approach"] = slow_problem.is_empty()
	checks["fast ally can't take the claimed tile"] = not fast_problem.is_empty()
	checks["slow ally still valid"] = plan.is_valid(slow)
	checks["fast ally still valid"] = plan.is_valid(fast)
	checks["slow ally holds the tile"] = plan.planned_position(slow) == tile
	checks["fast ally not on the tile"] = plan.planned_position(fast) != tile
	return TestHelpers.report(TAG, "A later, faster ally's attack keeps off the tile an earlier ally's approach holds", checks)

# Session level with hand steps: confirm_plan() accepts the party plan.
static func _session_confirm() -> bool:
	var enc := ClaimEncounter.new()
	var session := CombatSession.new(enc)
	session.start()
	var checks: Dictionary = _fixture_checks(enc, session.plan)
	var foe: Combatant = session.enemies[0]
	var strike: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
	checks["slow ally plans the approach"] = session.plan_add_step(session.allies[0], PlanStep.engage(strike, foe)).is_empty()
	session.plan_add_step(session.allies[1], PlanStep.engage(strike, foe))   # refused or placed elsewhere
	checks["plan valid"] = session.plan_is_valid()
	checks["confirm accepted"] = session.confirm_plan(false)
	session.dispose()
	return TestHelpers.report(TAG, "Two allies wanting the same approach tile: confirm_plan accepted", checks)

# Preset path (the auto-plan): both allies drafted by the preset, every plan valid, no tile claimed
# twice, confirm accepted.
static func _preset_confirm() -> bool:
	var checks: Dictionary = {}
	for preset_id: String in PlanPresets.IDS:
		var session := CombatSession.new(ClaimEncounter.new())
		session.start()
		session.plan_apply_preset_all(preset_id)
		var claimed: Dictionary = {}
		var shared: bool = false
		for a: Combatant in session.allies:
			var cell: Vector2i = session.plan.planned_position(a)
			shared = shared or claimed.has(cell)
			claimed[cell] = true
		checks["%s: plan valid" % preset_id] = session.plan_is_valid()
		checks["%s: no tile claimed twice" % preset_id] = not shared
		checks["%s: confirm accepted" % preset_id] = session.confirm_plan(false)
		session.dispose()
	checks["presets exist"] = not PlanPresets.IDS.is_empty()
	return TestHelpers.report(TAG, "Preset auto-plan of the party is always confirmable (every preset)", checks)

# A support approach (walk into reach of the ally first) stores its destination like an Engage
# approach, so later drafts see the same claim the compile uses.
static func _ally_approach_pinned() -> bool:
	var support: SkillDef = null
	for s: SkillDef in SkillCatalog.all():
		if s.targets_ally():
			if support == null or s.level < support.level:
				support = s
	var checks: Dictionary = {"a support skill aimed at allies exists": support != null}
	if support == null:
		return TestHelpers.report(TAG, "Support approach destination is stored (step.cell)", checks)
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var healer: Combatant = TestHelpers.unit("Healer", grid, Vector2i(1, 1), 400, 10, 1, 20, [support.id])
	var patient: Combatant = TestHelpers.unit("Patient", grid, Vector2i(1, 7), 200, 10)
	patient.hp = 100
	var foe: Combatant = TestHelpers.unit("FarFoe", grid, Vector2i(7, 7), 400, 5)
	var allies: Array[Combatant] = [healer, patient]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, enemies, maxi(3, support.level))
	var plan := CombatPlan.new(ctx)
	var band: Vector2i = support.support_reach(healer)
	var dist: int = grid.get_distance(grid.get_position(healer), grid.get_position(patient))
	var needs_move: bool = dist > band.y and dist - band.y <= plan.max_move_tiles(healer)
	checks["fixture: out of reach but reachable this round"] = needs_move
	var step: PlanStep = PlanStep.ally(support, patient)
	var problem: String = plan.add_step(healer, step)
	checks["support step planned"] = problem.is_empty()
	checks["approach tile stored"] = step.cell != CombatPlan.NO_CELL
	checks["planned position is the stored tile"] = plan.planned_position(healer) == step.cell
	checks["pinned_cell reports it"] = plan.pinned_cell(healer) == step.cell
	if not problem.is_empty():
		print("    [%s] support step refused: %s" % [TAG, problem])
	return TestHelpers.report(TAG, "Support approach destination is stored (step.cell) and claimed", checks)
