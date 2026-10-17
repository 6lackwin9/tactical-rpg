class_name EnginePiecesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Engine Pieces Test Suite (milestone M2c-2a2)
# Cleanse (SUPPORT removes active and pending debuffs, never buffs), Disengage (a jump away
# from an enemy: the unit's move for the round), the SWEEP area (an L of three tiles in the
# aimed quadrant). Expectations come from the rules (StatusEffects, SupportRules), not magic
# numbers. Unarmed damage and the gear budget checks moved to UnarmedTestSuite (CLN-12).
# ==============================================================================

const TAG: String = "EnginePieces"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING ENGINE PIECES TEST (M2c-2a2)            ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_cleanse_rules() and ok
	ok = _test_cleanse_combined() and ok
	ok = _test_cleanse_planner() and ok
	ok = _test_cleanse_validation() and ok
	ok = _test_disengage_planning() and ok
	ok = _test_disengage_rules() and ok
	ok = _test_disengage_execution() and ok
	ok = _test_sweep_pattern() and ok
	ok = _test_sweep_planning() and ok
	return ok

class Rig:
	extends RefCounted
	var grid: CombatGrid = null
	var allies: Array[Combatant] = []
	var foes: Array[Combatant] = []
	var ctx: PlanContext = null
	var plan: CombatPlan = null

# A 9x9 closed grid with `units` placed at the given tiles; the first array holds the allies.
static func _setup(ally_cells: Array[Vector2i], foe_cells: Array[Vector2i]) -> Rig:
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var allies: Array[Combatant] = []
	var foes: Array[Combatant] = []
	for i: int in range(ally_cells.size()):
		var a := Combatant.new("Ally%d" % i, 200, 10, 1)
		grid.place_combatant(a, ally_cells[i])
		allies.append(a)
	for i: int in range(foe_cells.size()):
		var f := Combatant.new("Foe%d" % i, 200, 10, 1)
		grid.place_combatant(f, foe_cells[i])
		foes.append(f)
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, foes, 5)
	var rig := Rig.new()
	rig.grid = grid
	rig.allies = allies
	rig.foes = foes
	rig.ctx = ctx
	rig.plan = CombatPlan.new(ctx)
	return rig

static func _support(id: String, heal_base: int, cleanse: Array[String], grant: String) -> SkillDef:
	var s: SkillDef = SkillDef.create(id, id, SkillDef.Kind.SUPPORT, 2, 1, "")
	s.heal_base = heal_base
	s.cleanse_statuses = cleanse
	s.grant_status = grant
	s.learned_only = false
	return s

# The purify tier that cleanses every debuff (lower tiers cleanse a named subset).
static func _purify_all() -> SkillDef:
	for tier: SkillDef in SkillCatalog.family_tiers("purify"):
		if tier.cleanse_statuses.has(SkillDef.CLEANSE_ALL):
			return tier
	return null

static func _afflict(c: Combatant) -> void:
	c.statuses[StatusEffects.STAGGER] = true
	c.status_rounds_left[StatusEffects.STAGGER] = 2
	c.statuses[StatusEffects.HASTE] = true
	c.status_rounds_left[StatusEffects.HASTE] = 2
	StatusEffects.queue(c, StatusEffects.ROOT, 1)
	StatusEffects.queue(c, StatusEffects.BLEED, 2)
	StatusEffects.queue(c, StatusEffects.TAUNT, 1)

# ------------------------------------------------------------------------------

static func _test_cleanse_rules() -> bool:
	var checks: Dictionary = {}
	var purify: SkillDef = _purify_all()
	checks["purify exists, is SUPPORT, learned_only"] = purify != null and purify.kind == SkillDef.Kind.SUPPORT and purify.learned_only
	checks["'all' expands to every debuff, in order"] = purify.cleanse_list() == StatusEffects.debuffs() and purify.cleanses() and not purify.heals()
	checks["purify validates with no heal or grant"] = ContentCatalog.validate_all().is_empty()
	var caster := Combatant.new("Caster", 100, 10, 1)
	var target := Combatant.new("Target", 100, 10, 1)
	_afflict(target)
	var before_hp: int = target.hp
	var seen: Array[String] = SupportRules.cleansable(target, purify)
	checks["cleansable lists active and pending debuffs only"] = seen == [StatusEffects.STAGGER, StatusEffects.ROOT, StatusEffects.BLEED]
	var result: Dictionary = SupportRules.apply(caster, target, purify, purify.status_rounds)
	checks["apply reports what it removed"] = ContentFactory._strings(result["cleansed"]) == [StatusEffects.STAGGER, StatusEffects.ROOT, StatusEffects.BLEED]
	var gone: bool = true
	for status: String in StatusEffects.debuffs():
		gone = gone and not StatusEffects.has_any(target, status) and not target.status_rounds_left.has(status) and not target.pending_status_rounds.has(status)
	checks["no debuff is left, active or pending"] = gone
	checks["buffs stay (active haste, pending taunt)"] = target.statuses.has(StatusEffects.HASTE) and target.pending_statuses.has(StatusEffects.TAUNT) and int(target.status_rounds_left[StatusEffects.HASTE]) == 2
	checks["no heal from a cleanse-only skill"] = target.hp == before_hp and int(result["healed"]) == 0
	# A list cleanses only what it names.
	var only_bleed: SkillDef = _support("t_only_bleed", 0, ["bleed"], "")
	var t2 := Combatant.new("T2", 100, 10, 1)
	_afflict(t2)
	SupportRules.apply(caster, t2, only_bleed, 1)
	checks["a named list removes only those"] = not StatusEffects.has_any(t2, StatusEffects.BLEED) and t2.statuses.has(StatusEffects.STAGGER) and t2.pending_statuses.has(StatusEffects.ROOT)
	# Nothing to cleanse: no change, no error.
	var clean := Combatant.new("Clean", 100, 10, 1)
	var empty_result: Dictionary = SupportRules.apply(caster, clean, purify, 1)
	checks["nothing to cleanse is a quiet no-op"] = ContentFactory._strings(empty_result["cleansed"]).is_empty() and SupportRules.cleansable(clean, purify).is_empty()
	# Cleansing the pending bleed stops its round-end tick; roll_over no longer applies the pending root.
	var t3 := Combatant.new("T3", 100, 10, 1)
	StatusEffects.queue(t3, StatusEffects.ROOT, 1)
	SupportRules.apply(caster, t3, purify, 1)
	StatusEffects.roll_over(t3, 0)
	checks["a cleansed pending debuff never becomes active"] = not t3.has_status(StatusEffects.ROOT)
	return TestHelpers.report(TAG, "Cleanse rules", checks)

static func _test_cleanse_combined() -> bool:
	var checks: Dictionary = {}
	var greater: SkillDef = _support("t_greater_mend", 12, ["all"], StatusEffects.HASTE)
	var caster := Combatant.new("Caster", 100, 10, 1)
	var target := Combatant.new("Target", 100, 10, 1)
	target.hp = 50
	_afflict(target)
	var result: Dictionary = SupportRules.apply(caster, target, greater, 2)
	checks["heal lands (fixed 12)"] = target.hp == 62 and int(result["healed"]) == 12
	checks["debuffs gone"] = not StatusEffects.has_any(target, StatusEffects.STAGGER) and not StatusEffects.has_any(target, StatusEffects.ROOT) and not StatusEffects.has_any(target, StatusEffects.BLEED)
	checks["buff queued for 2 rounds"] = target.pending_statuses.has(StatusEffects.HASTE) and int(target.pending_status_rounds[StatusEffects.HASTE]) == 2
	checks["existing buffs untouched"] = target.statuses.has(StatusEffects.HASTE)
	# Through a real command + resolver round: SupportCommand logs the cleanse.
	var setup: Rig = _setup([Vector2i(3, 4), Vector2i(4, 4)], [Vector2i(8, 8)])
	var allies: Array[Combatant] = setup.allies
	allies[1].hp = 100
	_afflict(allies[1])
	var plan: CombatPlan = setup.plan
	var problem: String = plan.add_step(allies[0], PlanStep.ally(greater, allies[1]))
	checks["planner accepts the combined skill"] = problem.is_empty()
	var messages: Array[String] = TestHelpers.run_steps(allies[0], setup.ctx, [PlanStep.ally(greater, allies[1])])
	var logged: bool = false
	for m: String in messages:
		logged = logged or (m.begins_with("[Support]") and "cleansed" in m)
	checks["resolver round cleanses the ally and logs it"] = logged and not StatusEffects.has_any(allies[1], StatusEffects.BLEED)
	return TestHelpers.report(TAG, "Greater Mend style: heal + cleanse + buff", checks)

static func _test_cleanse_planner() -> bool:
	var checks: Dictionary = {}
	var purify: SkillDef = _purify_all()
	var setup: Rig = _setup([Vector2i(3, 4), Vector2i(4, 4)], [Vector2i(8, 8)])
	var allies: Array[Combatant] = setup.allies
	allies[0].learn_skill(purify.id)
	var plan: CombatPlan = setup.plan
	var healthy_options: Array[PlanOption] = plan.target_options(allies[0], allies[1])
	var clean_option: PlanOption = null
	for o: PlanOption in healthy_options:
		if o.step.skill == purify:
			clean_option = o
	checks["purify is offered on an ally"] = clean_option != null and clean_option.valid
	checks["warns when there is nothing to cleanse"] = clean_option != null and "Nothing to cleanse" in clean_option.warning and clean_option.cleanses.is_empty()
	_afflict(allies[1])
	var sick_option: PlanOption = null
	for o: PlanOption in plan.target_options(allies[0], allies[1]):
		if o.step.skill == purify:
			sick_option = o
	checks["option lists what it removes"] = sick_option != null and sick_option.cleanses == [StatusEffects.STAGGER, StatusEffects.ROOT, StatusEffects.BLEED]
	checks["cost text names them"] = sick_option != null and "cleanses Staggered, Rooted, Bleeding" in sick_option.cost_text() and sick_option.warning.is_empty()
	checks["costs AP = level"] = sick_option != null and sick_option.ap == purify.ap_cost(allies[0])
	var self_options: Array[PlanOption] = plan.target_options(allies[0], allies[0])
	var self_option: PlanOption = null
	for o: PlanOption in self_options:
		if o.step.skill == purify:
			self_option = o
	checks["also castable on yourself (ALLY_OR_SELF)"] = self_option != null and self_option.valid
	return TestHelpers.report(TAG, "Cleanse planner options", checks)

static func _test_cleanse_validation() -> bool:
	var checks: Dictionary = {}
	var base: Dictionary = {"id": "t_bad", "name": "Bad", "kind": "SUPPORT", "level": 2, "blocks": 1}
	var bad_id: Dictionary = base.duplicate()
	bad_id["cleanse_statuses"] = ["haste"]
	checks["a buff id is rejected"] = not ContentCatalog.validate_entry("skills", ContentSchema.normalize("skills", bad_id)).is_empty()
	var empty_support: Dictionary = base.duplicate()
	checks["a SUPPORT skill with no effect is rejected"] = not ContentCatalog.validate_entry("skills", ContentSchema.normalize("skills", empty_support)).is_empty()
	var good: Dictionary = base.duplicate()
	good["cleanse_statuses"] = ["bleed", "root"]
	checks["a debuff list is accepted"] = ContentCatalog.validate_entry("skills", ContentSchema.normalize("skills", good)).is_empty()
	var attack: Dictionary = {"id": "t_bad2", "name": "Bad2", "kind": "ATTACK", "level": 1, "blocks": 1, "cleanse_statuses": ["all"]}
	checks["only SUPPORT skills may cleanse"] = not ContentCatalog.validate_entry("skills", ContentSchema.normalize("skills", attack)).is_empty()
	var entry: Dictionary = ContentFactory.skill_to_entry(SkillCatalog.find("purify"))
	checks["round-trips through the factory"] = ContentFactory.skill_to_entry(ContentFactory.skill(entry)) == entry
	return TestHelpers.report(TAG, "Cleanse schema and validation", checks)

# ------------------------------------------------------------------------------

static func _test_disengage_planning() -> bool:
	var checks: Dictionary = {}
	var backflip: SkillDef = SkillCatalog.find("backflip")
	checks["backflip: DISENGAGE, learned_only, validates"] = backflip != null and backflip.kind == SkillDef.Kind.DISENGAGE and backflip.learned_only and backflip.jump_tiles > 0
	var setup: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var grid: CombatGrid = setup.grid
	var hero: Combatant = setup.allies[0]
	var foe: Combatant = setup.foes[0]
	hero.learn_skill("backflip")
	var plan: CombatPlan = setup.plan
	var step: PlanStep = PlanStep.self_skill(backflip)
	var problem: String = plan.add_step(hero, step)
	checks["a disengage is planned"] = problem.is_empty()
	var st: CombatPlan.SimState = plan.compile(hero)
	var entry: PlanEntry = st.entries[st.entries.size() - 1]
	var from: Vector2i = Vector2i(4, 4)
	var dest: Vector2i = entry.pos_after
	checks["costs AP = skill level, one block, keeps the stance"] = entry.skill_ap == backflip.ap_cost(hero) and entry.blocks() == backflip.blocks and entry.stance_after == entry.stance_before and entry.switch_ap == 0
	# Rule-derived expectation: nobody can end farther from the foe than the best free tile within the jump.
	var foe_pos: Vector2i = grid.get_position(foe)
	var best: int = 0
	for x: int in range(CombatGrid.GRID_SIZE):
		for y: int in range(CombatGrid.GRID_SIZE):
			var cell := Vector2i(x, y)
			if grid.get_distance(from, cell) <= backflip.jump_tiles and not grid.is_occupied(cell):
				best = maxi(best, grid.get_distance(cell, foe_pos))
	checks["lands on a tile maximising the distance from the foe"] = grid.get_distance(dest, foe_pos) == best and best > grid.get_distance(from, foe_pos)
	checks["within the jump and free"] = grid.get_distance(from, dest) <= backflip.jump_tiles and not grid.is_occupied(dest)
	checks["the chosen tile is stored on the step (stable re-compile)"] = step.cell == dest and plan.compile(hero).pos == dest
	# Deterministic: a fresh plan picks the same tile.
	var again: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var hero2: Combatant = again.allies[0]
	hero2.learn_skill("backflip")
	var plan2: CombatPlan = again.plan
	plan2.add_step(hero2, PlanStep.self_skill(backflip))
	checks["deterministic tie-break"] = plan2.compile(hero2).pos == dest
	checks["the unit's position is planned for allies"] = plan.planned_position(hero) == dest
	# The jump is the move: no second move, no jump after another move or an attack.
	var second: String = plan.add_step(hero, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), Vector2i(4, 3)))
	checks["a move after the jump is refused"] = not second.is_empty() and "first" in second
	var twice: String = plan.add_step(hero, PlanStep.self_skill(backflip))
	checks["a second jump is refused"] = not twice.is_empty()
	var other: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var hero3: Combatant = other.allies[0]
	hero3.learn_skill("backflip")
	var plan3: CombatPlan = other.plan
	var foe3: Combatant = other.foes[0]
	plan3.add_step(hero3, PlanStep.engage(SkillCatalog.find("strike"), foe3))
	checks["a jump after attacking is refused (move comes first)"] = not plan3.add_step(hero3, PlanStep.self_skill(backflip)).is_empty()
	var option_text: String = ""
	var fresh: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var hero4: Combatant = fresh.allies[0]
	hero4.learn_skill("backflip")
	for o: PlanOption in fresh.plan.target_options(hero4, hero4):
		if o.step.skill == backflip:
			option_text = o.cost_text()
			checks["planner option shows the destination"] = o.valid and o.jump_to == dest and ("jumps to %s" % str(dest)) in option_text
	checks["self options include the disengage"] = option_text != ""
	return TestHelpers.report(TAG, "Disengage planning", checks)

static func _test_disengage_rules() -> bool:
	var checks: Dictionary = {}
	var backflip: SkillDef = SkillCatalog.find("backflip")
	# Rooted: refused.
	var setup: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var hero: Combatant = setup.allies[0]
	hero.learn_skill("backflip")
	hero.statuses[StatusEffects.ROOT] = true
	var plan: CombatPlan = setup.plan
	var rooted_problem: String = plan.add_step(hero, PlanStep.self_skill(backflip))
	checks["rooted: refused"] = not rooted_problem.is_empty() and "rooted" in rooted_problem
	# No enemy.
	var lonely: Rig = _setup([Vector2i(4, 4)], [])
	var solo: Combatant = lonely.allies[0]
	solo.learn_skill("backflip")
	checks["no enemy: refused"] = not lonely.plan.add_step(solo, PlanStep.self_skill(backflip)).is_empty()
	# Choosing the tile (GOTO) and the foe (ENGAGE).
	var s2: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5), Vector2i(0, 0)])
	var h2: Combatant = s2.allies[0]
	var foes2: Array[Combatant] = s2.foes
	h2.learn_skill("backflip")
	var p2: CombatPlan = s2.plan
	var near_dest: Vector2i = Vector2i(4, 3)   # one tile farther from the (4,5) foe
	checks["clicked tile farther away is accepted"] = p2.add_step(h2, PlanStep.go_to(backflip, near_dest)).is_empty() and p2.compile(h2).pos == near_dest
	var s3: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var h3: Combatant = s3.allies[0]
	h3.learn_skill("backflip")
	var p3: CombatPlan = s3.plan
	checks["a tile that is not farther is refused"] = not p3.add_step(h3, PlanStep.go_to(backflip, Vector2i(5, 5))).is_empty()
	checks["a tile beyond the jump is refused"] = not p3.add_step(h3, PlanStep.go_to(backflip, Vector2i(4, 4 - backflip.jump_tiles - 1))).is_empty()
	checks["the unit's own tile is refused"] = not p3.add_step(h3, PlanStep.go_to(backflip, Vector2i(4, 4))).is_empty()
	# Chosen foe: jump away from foes2[1] at (0,0) even though foes2[0] is nearer.
	var s4: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5), Vector2i(0, 0)])
	var h4: Combatant = s4.allies[0]
	h4.learn_skill("backflip")
	var p4: CombatPlan = s4.plan
	var g4: CombatGrid = s4.grid
	var far_foe: Combatant = s4.foes[1]
	checks["jump away from a chosen foe"] = p4.add_step(h4, PlanStep.engage(backflip, far_foe)).is_empty() and g4.get_distance(p4.compile(h4).pos, Vector2i(0, 0)) > g4.get_distance(Vector2i(4, 4), Vector2i(0, 0))
	checks["default threat is the nearest foe"] = foes2.size() == 2 and p2.compile(h2).pos.y < 4
	# Walls block the jump, units don't; occupied tiles are never landing tiles.
	var s5: Rig = _setup([Vector2i(4, 4), Vector2i(4, 3)], [Vector2i(4, 5)])
	var g5: CombatGrid = s5.grid
	var p5: CombatPlan = s5.plan
	var reach_units: Dictionary = p5.jump_tiles_from(Vector2i(4, 4), 2)
	checks["leaps over a unit (tile behind an ally is reachable)"] = reach_units.has(Vector2i(4, 2)) and reach_units.has(Vector2i(4, 3))
	g5.set_terrain(Vector2i(4, 3), Terrain.Type.WALL)
	var reach_wall: Dictionary = p5.jump_tiles_from(Vector2i(4, 4), 2)
	checks["a wall is neither entered nor passed straight through"] = not reach_wall.has(Vector2i(4, 3)) and not reach_wall.has(Vector2i(4, 2))
	var h5: Combatant = s5.allies[0]
	h5.learn_skill("backflip")
	checks["landing never on an occupied or wall tile"] = p5.add_step(h5, PlanStep.self_skill(backflip)).is_empty() and p5.compile(h5).pos != Vector2i(4, 3) and not g5.is_occupied(p5.compile(h5).pos)
	# Cornered: no tile farther away -> refused with a reason.
	var s6: Rig = _setup([Vector2i(0, 0)], [Vector2i(1, 1)])
	var h6: Combatant = s6.allies[0]
	h6.learn_skill("backflip")
	var r6: String = s6.plan.add_step(h6, PlanStep.self_skill(backflip))
	checks["a corner still finds a tile farther away, or says why not"] = r6.is_empty() or "No free tile" in r6
	return TestHelpers.report(TAG, "Disengage rules", checks)

static func _test_disengage_execution() -> bool:
	var checks: Dictionary = {}
	var backflip: SkillDef = SkillCatalog.find("backflip")
	var setup: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var grid: CombatGrid = setup.grid
	var hero: Combatant = setup.allies[0]
	hero.learn_skill("backflip")
	var plan: CombatPlan = setup.plan
	plan.add_step(hero, PlanStep.self_skill(backflip))
	var dest: Vector2i = plan.compile(hero).pos
	var ap_before: int = hero.ap
	var messages: Array[String] = TestHelpers.run_steps(hero, setup.ctx, [PlanStep.self_skill(backflip)])
	var jumped: bool = false
	for m: String in messages:
		jumped = jumped or m.begins_with("[Disengage]")
	checks["the round resolves: unit stands on the planned tile"] = grid.get_position(hero) == dest and jumped
	checks["AP charged = skill level"] = ap_before - hero.ap == backflip.ap_cost(hero)
	# Command-level failures.
	var s2: Rig = _setup([Vector2i(4, 4)], [Vector2i(4, 5)])
	var g2: CombatGrid = s2.grid
	var h2: Combatant = s2.allies[0]
	var blocker := Combatant.new("Blocker", 50, 10, 1)
	g2.place_combatant(blocker, Vector2i(4, 2))
	var blocked_cmd := DisengageCommand.new(h2, g2, Vector2i(4, 2), backflip.jump_tiles, backflip.blocks)
	blocked_cmd.execute()
	checks["landing on an occupied tile fails and stays put"] = not blocked_cmd.landed and g2.get_position(h2) == Vector2i(4, 4) and "Failed" in blocked_cmd.execution_log
	var far_cmd := DisengageCommand.new(h2, g2, Vector2i(4, 4 - backflip.jump_tiles - 1), backflip.jump_tiles, backflip.blocks)
	far_cmd.execute()
	checks["a landing beyond the jump fails"] = not far_cmd.landed and g2.get_position(h2) == Vector2i(4, 4)
	h2.statuses[StatusEffects.ROOT] = true
	var root_cmd := DisengageCommand.new(h2, g2, Vector2i(4, 3), backflip.jump_tiles, backflip.blocks)
	root_cmd.execute()
	checks["a rooted unit can't jump"] = not root_cmd.landed and g2.get_position(h2) == Vector2i(4, 4)
	h2.statuses.erase(StatusEffects.ROOT)
	var ok_cmd := DisengageCommand.new(h2, g2, Vector2i(4, 3), backflip.jump_tiles, backflip.blocks)
	ok_cmd.execute()
	checks["a clear landing succeeds"] = ok_cmd.landed and ok_cmd.moved_tiles == 1 and g2.get_position(h2) == Vector2i(4, 3)
	return TestHelpers.report(TAG, "Disengage execution", checks)

# ------------------------------------------------------------------------------

static func _test_sweep_pattern() -> bool:
	var checks: Dictionary = {}
	var kick: SkillDef = SkillCatalog.find("sweep_kick")
	checks["sweep_kick: ATTACK, SWEEP, learned_only"] = kick != null and kick.kind == SkillDef.Kind.ATTACK and kick.area == SkillDef.Area.SWEEP and kick.learned_only and kick.is_area()
	var origin := Vector2i(4, 4)
	var quadrants: Array[Vector2i] = [Vector2i(1, 1), Vector2i(-1, 1), Vector2i(-1, -1), Vector2i(1, -1)]
	var union: Dictionary = {}
	var ok_all: bool = true
	for q: Vector2i in quadrants:
		var cells: Array[Vector2i] = kick.area_cells(origin, origin + q)
		var expected: Array[Vector2i] = [origin + Vector2i(q.x, 0), origin + q, origin + Vector2i(0, q.y)]
		ok_all = ok_all and cells.size() == 3 and cells == expected and not cells.has(origin)
		for cell: Vector2i in cells:
			union[cell] = true
	checks["each quadrant is the L (1,0) (1,1) (0,1) rotated"] = ok_all
	checks["the four quadrants cover the 8 neighbours exactly"] = union.size() == 8
	var base: Array[Vector2i] = kick.area_cells(origin, origin + Vector2i(1, 1))
	checks["the (1,1) aim is the unrotated base L"] = base == [Vector2i(5, 4), Vector2i(5, 5), Vector2i(4, 5)]
	checks["reach is melee (aim tile is diagonal)"] = kick.reach_for(Combatant.new("x", 10, 10, 1)) == Vector2i(1, 1)
	checks["validates"] = ContentCatalog.validate_all().is_empty()
	checks["other areas unchanged (BURST 9, PLUS 5, LINE 3)"] = _count_area(SkillDef.Area.BURST) == 9 and _count_area(SkillDef.Area.PLUS) == 5 and _count_area(SkillDef.Area.LINE) == 3
	return TestHelpers.report(TAG, "Sweep pattern", checks)

static func _count_area(area: SkillDef.Area) -> int:
	var s: SkillDef = SkillDef.create("t_area", "A", SkillDef.Kind.ATTACK, 1, 1, "")
	s.area = area
	return s.area_cells(Vector2i(4, 4), Vector2i(4, 5) if area != SkillDef.Area.SWEEP else Vector2i(5, 5)).size()

static func _test_sweep_planning() -> bool:
	var checks: Dictionary = {}
	var kick: SkillDef = SkillCatalog.find("sweep_kick")
	# Hero (4,4); foe on (5,4) and (5,5) in the aimed quadrant, an ally on (4,5), bystander at (3,3).
	var setup: Rig = _setup([Vector2i(4, 4), Vector2i(4, 5)], [Vector2i(5, 4), Vector2i(5, 5), Vector2i(3, 3)])
	var allies: Array[Combatant] = setup.allies
	var foes: Array[Combatant] = setup.foes
	allies[0].learn_skill("sweep_kick")
	var plan: CombatPlan = setup.plan
	var aimed: PlanStep = PlanStep.area_at(kick, Vector2i(5, 5))
	checks["aiming at a diagonal neighbour is valid"] = plan.add_step(allies[0], aimed).is_empty()
	var entry: PlanEntry = plan.compile(allies[0]).entries[0]
	checks["entry carries the L as hit-tile data"] = entry.cells == kick.area_cells(Vector2i(4, 4), Vector2i(5, 5))
	checks["friendly fire is counted (ally on (4,5))"] = entry.hits_allies == 1
	var expected_dmg: int = plan.estimate_damage(allies[0], kick, entry.stance_after, foes[0]) + plan.estimate_damage(allies[0], kick, entry.stance_after, foes[1])
	checks["estimate covers the two foes in the L"] = entry.est_damage == expected_dmg and expected_dmg > 0
	var orth: String = CombatPlan.new(setup.ctx).add_step(allies[0], PlanStep.area_at(kick, Vector2i(5, 4)))
	checks["an orthogonal aim is refused"] = not orth.is_empty() and "diagonal" in orth
	var far: String = CombatPlan.new(setup.ctx).add_step(allies[0], PlanStep.area_at(kick, Vector2i(6, 6)))
	checks["a far aim is refused"] = not far.is_empty()
	# Options: a foe on a diagonal tile yields a valid sweep option with the friendly-fire warning.
	var diag_options: Array[PlanOption] = CombatPlan.new(setup.ctx).target_options(allies[0], foes[1])
	var sweep_option: PlanOption = null
	for o: PlanOption in diag_options:
		if o.step.skill == kick:
			sweep_option = o
	checks["planner offers it on a diagonal foe, with friendly-fire warning"] = sweep_option != null and sweep_option.valid and "Friendly fire" in sweep_option.warning
	var side_option: PlanOption = null
	for o: PlanOption in CombatPlan.new(setup.ctx).target_options(allies[0], foes[0]):
		if o.step.skill == kick:
			side_option = o
	checks["it is invalid on an orthogonal foe (no quadrant)"] = side_option != null and not side_option.valid
	# Resolution: every unit in the L is hit, the bystander and the caster are not.
	var hp_before: Dictionary = {}
	for c: Combatant in allies + foes:
		hp_before[c] = c.hp
	TestHelpers.run_steps(allies[0], setup.ctx, [aimed])
	checks["foes and the ally in the L take damage"] = foes[0].hp < int(hp_before[foes[0]]) and foes[1].hp < int(hp_before[foes[1]]) and allies[1].hp < int(hp_before[allies[1]])
	checks["caster and bystander untouched"] = allies[0].hp == int(hp_before[allies[0]]) and foes[2].hp == int(hp_before[foes[2]])
	var dmg: int = int(hp_before[foes[0]]) - foes[0].hp
	checks["each victim takes its own DamageCalculator hit (equal here)"] = dmg == int(hp_before[foes[1]]) - foes[1].hp
	return TestHelpers.report(TAG, "Sweep planning and resolution", checks)
