class_name SkillEngineTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Engine Gaps Test Suite (slice M2C2B-L)
# X1: area attacks roll the skill's status once per unit hit (allies included).
# X2: a DEFEND skill may grant a support-grant buff (taunt), queued for next round,
#     while riposte stays active at once. Expectations are derived from the rules.
# ==============================================================================

const SURE: int = 500   # potency far above any resistance: the chance clamps to 100

static func _report(label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [SkillEngine] %s: %s%s" % [label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SKILL ENGINE TEST (M2C2B-L)             ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_area_status() and ok
	ok = _test_area_resistance() and ok
	ok = _test_area_plan_entry() and ok
	ok = _test_defend_taunt() and ok
	ok = _test_validation() and ok
	return ok

# ------------------------------------------------------------------------------

static func _rng(seed_value: int) -> RandomNumberGenerator:
	var r := RandomNumberGenerator.new()
	r.seed = seed_value
	return r

static func _unit(unit_name: String, grid: CombatGrid, pos: Vector2i) -> Combatant:
	var c := Combatant.new(unit_name, 400, 10, 1)
	c.power_override = 20
	c.armor_override = 0
	grid.place_combatant(c, pos)
	return c

static func _area(source: Combatant, grid: CombatGrid, cells: Array[Vector2i], status: String, potency: int, seed_value: int) -> AreaAttackCommand:
	var cmd := AreaAttackCommand.new(source, grid, cells, 1.0, 2)
	cmd.status_effect = status
	cmd.status_potency = potency
	cmd.status_rounds = 2
	cmd.rng = _rng(seed_value)
	return cmd

static func _test_area_status() -> bool:
	var checks: Dictionary = {}
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var caster: Combatant = _unit("Caster", grid, Vector2i(3, 3))
	var foe_a: Combatant = _unit("FoeA", grid, Vector2i(4, 3))
	var foe_b: Combatant = _unit("FoeB", grid, Vector2i(3, 4))
	var ally: Combatant = _unit("Ally", grid, Vector2i(2, 3))
	var bystander: Combatant = _unit("Bystander", grid, Vector2i(6, 6))
	var cells: Array[Vector2i] = [Vector2i(3, 3), Vector2i(4, 3), Vector2i(3, 4), Vector2i(2, 3)]
	var cmd: AreaAttackCommand = _area(caster, grid, cells, StatusEffects.STAGGER, SURE, 11)
	cmd.execute()
	checks["every unit on the area is hit"] = cmd.last_hits == 3
	checks["every unit hit has the status queued for next round"] = foe_a.pending_statuses.has(StatusEffects.STAGGER) and foe_b.pending_statuses.has(StatusEffects.STAGGER)
	checks["friendly fire rolls too"] = ally.pending_statuses.has(StatusEffects.STAGGER)
	checks["the caster and a unit outside the area get nothing"] = not caster.pending_statuses.has(StatusEffects.STAGGER) and not bystander.pending_statuses.has(StatusEffects.STAGGER)
	checks["the queued duration is the command's status rounds"] = DictUtil.int_of(foe_a.pending_status_rounds, StatusEffects.STAGGER) == 2
	checks["the status is not active this round"] = not foe_a.statuses.has(StatusEffects.STAGGER)
	checks["the log shows one status note per target"] = cmd.execution_log.count("next round") == 3

	# A weaken status carries its magnitude through the area path.
	var grid2 := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var caster2: Combatant = _unit("Caster2", grid2, Vector2i(3, 3))
	var foe2: Combatant = _unit("Foe2", grid2, Vector2i(4, 3))
	var weaken_cmd: AreaAttackCommand = _area(caster2, grid2, [Vector2i(4, 3)], StatusEffects.WEAKEN, SURE, 5)
	weaken_cmd.status_magnitude = 15
	weaken_cmd.execute()
	checks["weaken magnitude is carried by the applied instance"] = DictUtil.int_of(foe2.pending_status_magnitudes, StatusEffects.WEAKEN) == 15

	# No status on the command: nothing is queued, damage unchanged.
	var grid3 := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var caster3: Combatant = _unit("Caster3", grid3, Vector2i(3, 3))
	var foe3: Combatant = _unit("Foe3", grid3, Vector2i(4, 3))
	var plain := AreaAttackCommand.new(caster3, grid3, [Vector2i(4, 3)], 1.0, 2)
	plain.rng = _rng(2)
	plain.execute()
	checks["no status on the command queues nothing"] = foe3.pending_statuses.is_empty() and plain.last_hits == 1
	return _report("Area attacks roll the status per unit hit", checks)

static func _test_area_resistance() -> bool:
	var checks: Dictionary = {}
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var caster: Combatant = _unit("Caster", grid, Vector2i(3, 3))
	var immune: Combatant = _unit("Immune", grid, Vector2i(4, 3))
	var normal: Combatant = _unit("Normal", grid, Vector2i(3, 4))
	immune.status_resist[StatusEffects.STAGGER] = 100
	var cmd: AreaAttackCommand = _area(caster, grid, [Vector2i(4, 3), Vector2i(3, 4)], StatusEffects.STAGGER, 100 + ceili(normal.debuff_resistance()), 9)
	cmd.execute()
	checks["a fully resistant target is never afflicted"] = not immune.pending_statuses.has(StatusEffects.STAGGER)
	checks["its neighbour still is (independent rolls)"] = normal.pending_statuses.has(StatusEffects.STAGGER)
	checks["the resisted line names the chance"] = cmd.execution_log.find("resisted %s (0%%)" % StatusEffects.display_name(StatusEffects.STAGGER).to_lower()) >= 0
	# Zero potency never applies.
	var grid2 := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var c2: Combatant = _unit("C2", grid2, Vector2i(3, 3))
	var t2: Combatant = _unit("T2", grid2, Vector2i(4, 3))
	var zero: AreaAttackCommand = _area(c2, grid2, [Vector2i(4, 3)], StatusEffects.STAGGER, 0, 4)
	zero.execute()
	checks["potency 0 never applies"] = not t2.pending_statuses.has(StatusEffects.STAGGER)
	# Seeded, repeatable: the same seed gives the same outcome on a 50% roll.
	var outcomes: Array[bool] = []
	for i: int in 2:
		var g: CombatGrid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
		var src: Combatant = _unit("S", g, Vector2i(3, 3))
		var victim: Combatant = _unit("V", g, Vector2i(4, 3))
		_area(src, g, [Vector2i(4, 3)], StatusEffects.STAGGER, 50, 77).execute()
		outcomes.append(victim.pending_statuses.has(StatusEffects.STAGGER))
	checks["the same seed gives the same roll"] = outcomes[0] == outcomes[1]
	return _report("Area status honours resistance and the seeded rng", checks)

static func _test_area_plan_entry() -> bool:
	var checks: Dictionary = {}
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var caster: Combatant = _unit("Caster", grid, Vector2i(3, 3))
	var foe: Combatant = _unit("Foe", grid, Vector2i(4, 3))
	var skill: SkillDef = SkillDef.create("t_burst", "Burst", SkillDef.Kind.ATTACK, 2, 2, "")
	skill.area = SkillDef.Area.PLUS
	skill.status_effect = StatusEffects.ROOT
	skill.status_potency = SURE
	skill.status_rounds = 2
	var entry := PlanEntry.new()
	entry.skill = skill
	entry.cells = [Vector2i(4, 3)]
	var built: BattleCommand = entry.build_command(caster, grid)
	checks["the plan entry builds an area command"] = built is AreaAttackCommand
	checks["it carries the skill's status, potency and rounds"] = built.status_effect == StatusEffects.ROOT and built.status_potency >= SURE and built is AreaAttackCommand and (built as AreaAttackCommand).status_rounds == 2
	built.rng = _rng(3)
	built.execute()
	checks["executing it queues the status for the target"] = foe.pending_statuses.has(StatusEffects.ROOT) and DictUtil.int_of(foe.pending_status_rounds, StatusEffects.ROOT) == 2
	return _report("Plan entry feeds the area command", checks)

static func _defend_skill(grant: String, rounds: int) -> SkillDef:
	var s: SkillDef = SkillDef.create("t_wall", "Wall", SkillDef.Kind.DEFEND, 2, 2, "")
	s.mitigation = 0.4
	s.grant_status = grant
	s.status_rounds = rounds
	return s

static func _test_defend_taunt() -> bool:
	var checks: Dictionary = {}
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var tank: Combatant = _unit("Tank", grid, Vector2i(3, 3))
	var skill: SkillDef = _defend_skill(StatusEffects.TAUNT, 2)
	var entry := PlanEntry.new()
	entry.skill = skill
	var cmd: BattleCommand = entry.build_command(tank, grid)
	var note: String = cmd.on_start()
	cmd.execute()
	checks["a taunt defend is not active this round"] = not tank.has_status(StatusEffects.TAUNT)
	checks["it is queued for next round"] = tank.pending_statuses.has(StatusEffects.TAUNT)
	checks["its duration is the skill's status rounds"] = DictUtil.int_of(tank.pending_status_rounds, StatusEffects.TAUNT) == skill.status_rounds
	checks["the mitigation still applies this round"] = is_equal_approx(tank.defense_mitigation_ratio, skill.mitigation)
	checks["a start line is logged"] = not note.is_empty()
	StatusEffects.roll_over(tank, 4)
	checks["after the roll-over taunt is active"] = tank.has_status(StatusEffects.TAUNT)

	var tank2: Combatant = _unit("Tank2", grid, Vector2i(5, 5))
	var rip_entry := PlanEntry.new()
	rip_entry.skill = _defend_skill(StatusEffects.RIPOSTE, 2)
	var rip: BattleCommand = rip_entry.build_command(tank2, grid)
	rip.on_start()
	checks["a riposte defend is still active at once, not queued"] = tank2.has_status(StatusEffects.RIPOSTE) and not tank2.pending_statuses.has(StatusEffects.RIPOSTE)

	var tank3: Combatant = _unit("Tank3", grid, Vector2i(1, 1))
	var plain_entry := PlanEntry.new()
	plain_entry.skill = _defend_skill("", 1)
	var plain: BattleCommand = plain_entry.build_command(tank3, grid)
	checks["a plain defend queues nothing"] = plain.on_start().is_empty() and tank3.pending_statuses.is_empty()
	return _report("DEFEND skill granting taunt", checks)

static func _errors(entry: Dictionary) -> String:
	return " | ".join(ContentCatalog.validate_entry("skills", entry, str(entry.get("id", ""))))

static func _test_validation() -> bool:
	var checks: Dictionary = {}
	var taunt: Dictionary = {"id": "t_wall", "name": "Wall", "kind": "DEFEND", "grant_status": "taunt", "mitigation": 0.4}
	checks["a DEFEND skill granting taunt validates"] = _errors(taunt).is_empty()
	var riposte: Dictionary = taunt.duplicate()
	riposte["grant_status"] = "riposte"
	checks["a DEFEND skill granting riposte still validates"] = _errors(riposte).is_empty()
	var haste: Dictionary = taunt.duplicate()
	haste["grant_status"] = "haste"
	var expect_haste: bool = StatusEffects.support_buffs().has("haste")
	checks["any support-grant buff is allowed, derived from the registry"] = _errors(haste).is_empty() == expect_haste
	var bad: Dictionary = taunt.duplicate()
	bad["grant_status"] = "stagger"
	checks["a debuff is still rejected as a defend grant"] = _errors(bad).find("is not a defend or support buff") >= 0
	var area_zero: Dictionary = {"id": "t_az", "name": "AZ", "kind": "ATTACK", "area": "PLUS", "power_mult": 0.0, "status_effect": "weaken", "status_potency": 50, "status_magnitude": 10}
	checks["a no-damage area attack stays rejected (D1 rule kept)"] = _errors(area_zero).find("no damage can't be an area") >= 0
	var area_status: Dictionary = {"id": "t_as", "name": "AS", "kind": "ATTACK", "area": "BURST", "reach_min": 2, "reach_max": 2, "power_mult": 0.6, "status_effect": "root", "status_potency": 50}
	checks["a damaging area attack with a status validates"] = _errors(area_status).is_empty()
	checks["the shipped catalog still validates"] = ContentCatalog.validate_all().is_empty()
	return _report("Validation", checks)
