class_name WeakenTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Weaken Test Suite (slice D1-L)
# The weaken status (registry row, magnitude carried by the instance, cap and non-stacking),
# the outgoing-power hook (SkillDef.power_for, planner estimate, AttackCommand, riposte counter),
# zero-power debuff-only attacks, schema validation, cleanse, and the data-driven riposte tiers
# (the reverse and the ascending model built in memory). Units are hand-built with a fixed strike
# power and no armor; every expectation is derived from DamageCalculator.compute and the rules.
# ==============================================================================

const SANDBOX: String = "user://weaken_test"
const MAG: int = 20
const SURE: int = 500   # potency far above any resistance: the chance clamps to 100

static func _report(label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [Weaken] %s: %s%s" % [label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING WEAKEN TEST (D1-L)                      ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_registry() and ok
	ok = _test_stacking() and ok
	ok = _test_timing() and ok
	ok = _test_hook() and ok
	ok = _test_riposte_counter() and ok
	ok = _test_resistance() and ok
	ok = _test_zero_power() and ok
	ok = _test_validation() and ok
	ok = _test_cleanse() and ok
	ok = _test_undo() and ok
	ok = _test_riposte_data_models() and ok
	return ok

# ------------------------------------------------------------------------------

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])

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

# An active weaken of `magnitude` percent (as it stands after the round roll-over).
static func _weaken_now(c: Combatant, magnitude: int, rounds: int = 2) -> void:
	c.statuses[StatusEffects.WEAKEN] = true
	c.status_rounds_left[StatusEffects.WEAKEN] = rounds
	c.status_magnitudes[StatusEffects.WEAKEN] = magnitude

static func _attack_skill(id: String, power: float, status: String, magnitude: int) -> SkillDef:
	var s: SkillDef = SkillDef.create(id, id, SkillDef.Kind.ATTACK, 1, 1, "")
	s.power_mult = power
	s.status_effect = status
	s.status_potency = 100
	s.status_magnitude = magnitude
	s.status_rounds = 2
	return s

# ------------------------------------------------------------------------------

static func _test_registry() -> bool:
	var checks: Dictionary = {}
	var row: Dictionary = StatusRegistry.row(StatusEffects.WEAKEN)
	checks["row_errors() is empty"] = StatusRegistry.row_errors().is_empty()
	checks["row fields equal the spec table"] = str(row["kind"]) == "debuff" and DictUtil.to_bool(row["resistable"]) and DictUtil.to_int(row["default_rounds"]) == 2 and str(row["stacking"]) == StatusRegistry.REFRESH_MAX_DURATION and DictUtil.to_int(row["cap"]) == 25 and str(row["display"]) == "Weakened" and str(row["grant"]).is_empty()
	checks["it joins resistable() and debuffs() from the registry"] = StatusEffects.resistable().has(StatusEffects.WEAKEN) and StatusEffects.debuffs().has(StatusEffects.WEAKEN)
	checks["the schema offers it as an Inflicts option and a resist key"] = ContentSchema.options_of({"options_from": "status"}).has(StatusEffects.WEAKEN)
	checks["only weaken carries a magnitude"] = StatusEffects.has_magnitude(StatusEffects.WEAKEN) and not StatusEffects.has_magnitude(StatusEffects.BLEED)
	return _report("Registry row and derived lists", checks)

static func _test_stacking() -> bool:
	var checks: Dictionary = {}
	var w: String = StatusEffects.WEAKEN
	var cap: int = StatusRegistry.cap_of(w)
	checks["15 then 25 gives 25"] = StatusRegistry.stack(2, 15, 2, 25, w)["potency"] == 25
	checks["25 then 15 gives 25"] = StatusRegistry.stack(2, 25, 2, 15, w)["potency"] == 25
	checks["two 25s give 25 (never the sum)"] = StatusRegistry.stack(2, cap, 2, cap, w)["potency"] == cap
	checks["a new magnitude above the cap is clamped"] = StatusRegistry.stack(0, 0, 2, cap + 30, w)["potency"] == cap
	checks["rounds are the longer of the two"] = StatusRegistry.stack(3, 10, 2, 15, w)["rounds"] == 3 and StatusRegistry.stack(1, 10, 2, 15, w)["rounds"] == 2
	# Pending application on a unit, through the real queue.
	var c := Combatant.new("Stack", 100, 10, 1)
	StatusEffects.queue(c, w, 2, 15)
	StatusEffects.queue(c, w, 3, cap)
	StatusEffects.queue(c, w, 1, 10)
	checks["queue keeps the stronger magnitude and longer rounds"] = DictUtil.int_of(c.pending_status_magnitudes, w) == cap and DictUtil.int_of(c.pending_status_rounds, w) == 3
	StatusEffects.queue(c, StatusEffects.BLEED, 2, 40)
	checks["other statuses store no magnitude"] = not c.pending_status_magnitudes.has(StatusEffects.BLEED)
	# Active + pending meet at the round roll-over under the same rule.
	var d := Combatant.new("Meet", 100, 10, 1)
	_weaken_now(d, cap, 3)
	StatusEffects.queue(d, w, 1, 15)
	StatusEffects.roll_over(d, 4)
	checks["active 25 meets pending 15: stays 25, carried rounds kept"] = DictUtil.int_of(d.status_magnitudes, w) == cap and DictUtil.int_of(d.status_rounds_left, w) == 2 and d.pending_status_magnitudes.is_empty()
	var e := Combatant.new("Meet2", 100, 10, 1)
	_weaken_now(e, 15, 3)
	StatusEffects.queue(e, w, 2, cap)
	StatusEffects.roll_over(e, 4)
	checks["active 15 meets pending 25: becomes 25"] = DictUtil.int_of(e.status_magnitudes, w) == cap and DictUtil.int_of(e.status_rounds_left, w) == 2
	return _report("Stronger magnitude wins, capped, never summed", checks)

static func _test_timing() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var foe: Combatant = _unit("Foe", grid, Vector2i(4, 4))
	var rounds: int = 2
	var landed: bool = StatusEffects.try_inflict(foe, StatusEffects.WEAKEN, SURE, _rng(1), rounds, MAG)
	checks["applying succeeds with potency far above resistance"] = landed
	checks["this round the factor is unchanged (queued for next round)"] = is_equal_approx(foe.weaken_factor(), 1.0) and foe.pending_statuses.has(StatusEffects.WEAKEN)
	StatusEffects.roll_over(foe, 4)
	checks["next round it starts: factor = 1 - magnitude/100"] = is_equal_approx(foe.weaken_factor(), 1.0 - float(MAG) / 100.0)
	var lasted: int = 1
	for i: int in range(6):
		StatusEffects.roll_over(foe, 4)
		if not foe.has_status(StatusEffects.WEAKEN):
			break
		lasted += 1
	checks["it lasts exactly status_rounds once active"] = lasted == rounds
	checks["afterwards the factor is 1.0 and the magnitude is gone"] = is_equal_approx(foe.weaken_factor(), 1.0) and foe.status_magnitudes.is_empty()
	var over := Combatant.new("Over", 100, 10, 1)
	_weaken_now(over, 90)
	checks["a stored magnitude above the cap is clamped by weaken_factor"] = is_equal_approx(over.weaken_factor(), 1.0 - float(StatusRegistry.cap_of(StatusEffects.WEAKEN)) / 100.0)
	return _report("Queued for next round, counts down, clamped", checks)

static func _test_hook() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var atk: Combatant = _unit("Atk", grid, Vector2i(3, 4))
	var foe: Combatant = _unit("Foe", grid, Vector2i(4, 4))
	var strike: SkillDef = SkillCatalog.find("strike")
	var base_power: float = strike.power_for(atk)
	var reduced: float = base_power * (1.0 - float(MAG) / 100.0)
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, [atk], [foe], 3)
	var plan := CombatPlan.new(ctx)
	var stance: CombatTypes.StanceType = strike.stance()
	var est_base: int = plan.estimate_damage(atk, strike, stance, foe)
	checks["unweakened: power_for = power_mult x potency"] = is_equal_approx(base_power, strike.power_mult * atk.potency_for(strike.family_id()))
	_weaken_now(atk, MAG)
	checks["weakened: power_for = P x (1 - m/100)"] = is_equal_approx(strike.power_for(atk), reduced)
	var est_weak: int = plan.estimate_damage(atk, strike, stance, foe)
	var expected_est: int = DictUtil.to_int(DamageCalculator.compute(atk, foe, false, false, reduced, false, null, stance, false)["final_dmg"])
	checks["planner estimate follows the weakened power"] = est_weak == expected_est and est_weak < est_base
	var cmd := AttackCommand.new(atk, foe, strike.power_for(atk), 1, 0, grid)
	cmd.rng = _rng(7)
	cmd.execute()
	var expected_hit: int = DictUtil.to_int(DamageCalculator.compute(atk, foe, false, false, reduced, false, _rng(7))["final_dmg"])
	checks["resolved AttackCommand damage = compute() at the reduced multiplier"] = cmd.last_damage == expected_hit
	# Area attacks read the same power_for.
	var area_skill: SkillDef = _attack_skill("t_area", 1.0, "", 0)
	area_skill.area = SkillDef.Area.PLUS
	checks["area attacks follow it too (same power_for)"] = is_equal_approx(area_skill.power_for(atk), area_skill.power_mult * atk.potency_for(area_skill.family_id()) * (1.0 - float(MAG) / 100.0))
	var entry: PlanEntry = PlanEntry.create(strike, 1, 0)
	entry.target = foe
	var built: AttackCommand = entry.build_command(atk, grid) as AttackCommand
	checks["PlanEntry.build_command carries the weakened multiplier"] = built != null and is_equal_approx(built.skill_mult, reduced)
	# Heals are not affected.
	var mend: SkillDef = SkillDef.create("t_mend", "m", SkillDef.Kind.SUPPORT, 1, 1, "")
	mend.heal_base = 30
	var healer := Combatant.new("Healer", 100, 10, 1)
	var plain := Combatant.new("Plain", 100, 10, 1)
	_weaken_now(healer, MAG)
	checks["heals ignore weaken"] = SupportRules.heal_amount(healer, mend) == SupportRules.heal_amount(plain, mend)
	var unweakened := Combatant.new("Fresh", 100, 10, 1)
	checks["an unweakened unit is unchanged"] = is_equal_approx(strike.power_for(unweakened), strike.power_mult * unweakened.potency_for(strike.family_id()))
	return _report("Outgoing power: power_for, planner estimate, AttackCommand", checks)

static func _test_riposte_counter() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = _unit("Tank", grid, Vector2i(3, 4))
	var foe: Combatant = _unit("Foe", grid, Vector2i(4, 4))
	StatusEffects.grant_now(tank, StatusEffects.RIPOSTE, 2)
	_weaken_now(tank, MAG)
	var mult: float = RiposteRules.COUNTER_MULT * (1.0 - float(MAG) / 100.0)
	var expected: Dictionary = DamageCalculator.compute(tank, foe, false, false, mult, false, _rng(5))
	var result: Dictionary = RiposteRules.counter(foe, tank, grid, false, _rng(5))
	checks["a weakened riposter counters at 0.5 x (1 - m/100)"] = not result.is_empty() and DictUtil.to_int(result["damage"]) == DictUtil.to_int(expected["final_dmg"])
	var g2: CombatGrid = _grid()
	var tank2: Combatant = _unit("Tank2", g2, Vector2i(3, 4))
	var foe2: Combatant = _unit("Foe2", g2, Vector2i(4, 4))
	StatusEffects.grant_now(tank2, StatusEffects.RIPOSTE, 2)
	var full: Dictionary = RiposteRules.counter(foe2, tank2, g2, false, _rng(5))
	checks["unweakened it still counters at COUNTER_MULT"] = DictUtil.to_int(full["damage"]) == DictUtil.to_int(DamageCalculator.compute(tank2, foe2, false, false, RiposteRules.COUNTER_MULT, false, _rng(5))["final_dmg"]) and DictUtil.to_int(full["damage"]) >= DictUtil.to_int(result["damage"])
	return _report("The riposte counter is weakened too", checks)

static func _test_resistance() -> bool:
	var checks: Dictionary = {}
	var immune := Combatant.new("Immune", 100, 10, 1)
	immune.status_resist[StatusEffects.WEAKEN] = 100
	var open := Combatant.new("Open", 100, 10, 1)
	var rng: RandomNumberGenerator = _rng(11)
	var never: bool = true
	var always: bool = true
	for i: int in range(30):
		never = never and not StatusEffects.try_inflict(immune, StatusEffects.WEAKEN, 100, rng, 2, MAG)
		always = always and StatusEffects.try_inflict(open, StatusEffects.WEAKEN, SURE, rng, 2, MAG)
	checks["resist 100 never lands"] = never and not immune.pending_statuses.has(StatusEffects.WEAKEN)
	checks["no resistance and a sure potency always lands"] = always
	checks["the chance is potency minus resistance"] = StatusEffects.apply_chance(immune, StatusEffects.WEAKEN, 70) == 0 and StatusEffects.apply_chance(open, StatusEffects.WEAKEN, 70) == roundi(70.0 - open.debuff_resistance())
	return _report("Resistance (weaken key, 100 = immune)", checks)

static func _test_zero_power() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var warrior: Combatant = _unit("Warrior", grid, Vector2i(3, 4))
	var tank: Combatant = _unit("Tank", grid, Vector2i(4, 4))
	StatusEffects.grant_now(tank, StatusEffects.RIPOSTE, 2)
	var roar: SkillDef = _attack_skill("t_roar", 0.0, StatusEffects.WEAKEN, MAG)
	checks["a zero-power attack deals no damage by definition"] = not roar.deals_damage() and is_equal_approx(roar.power_for(warrior), 0.0)
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, [warrior], [tank], 3)
	var plan := CombatPlan.new(ctx)
	checks["the planner estimate is 0"] = plan.estimate_damage(warrior, roar, roar.stance(), tank) == 0
	var entry: PlanEntry = PlanEntry.create(roar, 1, 0)
	entry.target = tank
	var cmd: AttackCommand = entry.build_command(warrior, grid) as AttackCommand
	var hp_before: int = tank.hp
	cmd.rng = _rng(3)
	cmd.execute()
	checks["it deals 0 damage and calls no hit"] = cmd.last_damage == 0 and tank.hp == hp_before and not cmd.last_crit
	checks["it cannot trigger a riposte"] = cmd.reactions.is_empty() and warrior.hp == warrior.max_hp and tank.riposted.is_empty()
	checks["it still inflicts weaken with the skill's magnitude"] = tank.pending_statuses.has(StatusEffects.WEAKEN) and DictUtil.int_of(tank.pending_status_magnitudes, StatusEffects.WEAKEN) == MAG
	checks["the log shows the status line only"] = cmd.execution_log.begins_with("[Status]") and cmd.execution_log.find("damage") < 0
	# A weaken on a unit with a damaging attack: a normal hit still ripostes.
	var strike: SkillDef = SkillCatalog.find("strike")
	var hit := AttackCommand.new(warrior, tank, strike.power_for(warrior), 1, 0, grid)
	hit.rng = _rng(3)
	hit.execute()
	checks["a damaging attack on the same riposter is still countered"] = hit.reactions.size() == 1
	return _report("Zero-power attack: no damage, no counter, status still rolls", checks)

static func _errors(entry: Dictionary) -> String:
	return " | ".join(ContentCatalog.validate_entry("skills", entry, str(entry.get("id", ""))))

static func _test_validation() -> bool:
	var checks: Dictionary = {}
	var base: Dictionary = {"id": "t_roar", "name": "Roar", "kind": "ATTACK", "power_mult": 0.0, "status_effect": "weaken", "status_potency": 70, "status_magnitude": 15}
	checks["a zero-power weaken attack validates"] = _errors(base).is_empty()
	var no_status: Dictionary = base.duplicate()
	no_status["status_effect"] = ""
	no_status["status_magnitude"] = 0
	checks["power 0 without a status is rejected"] = _errors(no_status).find("A skill with no damage needs a status") >= 0
	var zero_mag: Dictionary = base.duplicate()
	zero_mag["status_magnitude"] = 0
	checks["weaken with magnitude 0 is rejected"] = _errors(zero_mag).find("weaken needs a magnitude 1-25") >= 0
	var high_mag: Dictionary = base.duplicate()
	high_mag["status_magnitude"] = StatusRegistry.cap_of(StatusEffects.WEAKEN) + 1
	checks["weaken above the cap is rejected"] = _errors(high_mag).find("weaken needs a magnitude 1-25") >= 0
	var other: Dictionary = {"id": "t_root", "name": "R", "kind": "ATTACK", "status_effect": "root", "status_potency": 50, "status_magnitude": 10}
	checks["a magnitude on another status is rejected"] = _errors(other).find("magnitude only applies to weaken") >= 0
	var plain: Dictionary = {"id": "t_plain", "name": "P", "kind": "ATTACK", "status_magnitude": 5}
	checks["a magnitude without a status is rejected"] = _errors(plain).find("magnitude only applies to weaken") >= 0
	var area: Dictionary = base.duplicate()
	area["area"] = "PLUS"
	checks["a no-damage area attack is rejected"] = _errors(area).find("no damage can't be an area") >= 0
	var damaging: Dictionary = {"id": "t_dmg", "name": "D", "kind": "ATTACK", "power_mult": 1.0, "status_effect": "weaken", "status_potency": 50, "status_magnitude": 10}
	checks["a damaging attack may weaken too"] = _errors(damaging).is_empty()
	var rt: Dictionary = ContentSchema.normalize("skills", base)
	checks["the new field round-trips through the factory"] = ContentFactory.skill_to_entry(ContentFactory.skill(rt)) == rt and ContentFactory.skill(rt).status_magnitude == 15
	checks["the schema field has the spec range (default 0, 0..95)"] = _field("status_magnitude", "default") == 0 and _field("status_magnitude", "min") == 0 and _field("status_magnitude", "max") == 95
	checks["power_mult may now be 0 in the schema"] = is_equal_approx(DictUtil.to_float(_field("power_mult", "min")), 0.0)
	checks["the shipped catalog still validates"] = ContentCatalog.validate_all().is_empty()
	return _report("Schema and validation", checks)

static func _field(key: String, prop: String) -> Variant:
	for f: Dictionary in ContentSchema.fields("skills"):
		if str(f["key"]) == key:
			return f.get(prop, null)
	return null

static func _test_cleanse() -> bool:
	var checks: Dictionary = {}
	var purify: SkillDef = SkillCatalog.find("purify")
	var caster := Combatant.new("Caster", 100, 10, 1)
	var active := Combatant.new("Active", 100, 10, 1)
	_weaken_now(active, MAG)
	var pending := Combatant.new("Pending", 100, 10, 1)
	StatusEffects.queue(pending, StatusEffects.WEAKEN, 2, MAG)
	checks["purify (all) lists weaken among the cleansable"] = purify.cleanse_list().has(StatusEffects.WEAKEN) and SupportRules.cleansable(active, purify) == [StatusEffects.WEAKEN] and SupportRules.cleansable(pending, purify) == [StatusEffects.WEAKEN]
	SupportRules.apply(caster, active, purify, purify.status_rounds)
	SupportRules.apply(caster, pending, purify, purify.status_rounds)
	checks["it removes an active weaken, magnitude included"] = not StatusEffects.has_any(active, StatusEffects.WEAKEN) and active.status_magnitudes.is_empty() and is_equal_approx(active.weaken_factor(), 1.0)
	checks["it removes a pending weaken, magnitude included"] = not StatusEffects.has_any(pending, StatusEffects.WEAKEN) and pending.pending_status_magnitudes.is_empty()
	var named: SkillDef = SkillDef.create("t_named", "n", SkillDef.Kind.SUPPORT, 1, 1, "")
	named.cleanse_statuses = [StatusEffects.WEAKEN]
	var target := Combatant.new("Target", 100, 10, 1)
	_weaken_now(target, MAG)
	StatusEffects.queue(target, StatusEffects.BLEED, 1)
	SupportRules.apply(caster, target, named, 1)
	checks["a named list may include weaken and leaves the rest"] = not StatusEffects.has_any(target, StatusEffects.WEAKEN) and target.pending_statuses.has(StatusEffects.BLEED)
	return _report("Cleanse removes active and pending weaken", checks)

static func _test_undo() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	session.plan_apply_preset_all("balanced")
	session.confirm_plan()
	for i: int in range(600):
		if session.phase == CombatSession.Phase.REPLAY:
			break
		session.step_next()
	checks["round 2 reachable"] = session.can_advance_round()
	if session.can_advance_round():
		session.advance_round()
		var unit: Combatant = session.allies[0]
		StatusEffects.queue(unit, StatusEffects.WEAKEN, 2, MAG)
		_weaken_now(unit, MAG)
		session.reset_round()
		checks["a weaken inflicted in the undone round is gone (pending and magnitude)"] = unit.pending_status_magnitudes.is_empty() and not unit.pending_statuses.has(StatusEffects.WEAKEN)
		checks["the round start state is restored (no active weaken or magnitude)"] = not unit.has_status(StatusEffects.WEAKEN) and unit.status_magnitudes.is_empty()
	session.dispose()
	return _report("Round undo rewinds weaken", checks)

# ------------------------------------------------------------------------------
# Riposte is data driven: AP = level, mitigation = mitigation, duration = status_rounds.
# Both tier models are built in memory; the planner, DefendCommand and the status follow the rows.

static func _riposte(id: String, tier: int, level: int, mitigation: float, rounds: int) -> SkillDef:
	var s: SkillDef = SkillDef.create(id, id, SkillDef.Kind.DEFEND, level, 2, "")
	s.mitigation = mitigation
	s.grant_status = StatusEffects.RIPOSTE
	s.status_rounds = rounds
	s.family = "t_riposte"
	s.tier = tier
	return s

static func _model(rows: Array[Array]) -> Array[SkillDef]:
	var out: Array[SkillDef] = []
	for r: Array in rows:
		out.append(_riposte("t_rip_%d" % int(r[0]), int(r[0]), int(r[1]), float(r[2]), int(r[3])))
	return out

static func _check_model(label: String, skills: Array[SkillDef], checks: Dictionary) -> void:
	var all_ok: bool = true
	for s: SkillDef in skills:
		var grid: CombatGrid = _grid()
		var tank: Combatant = _unit("Tank", grid, Vector2i(3, 4))
		var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, [tank], [], 3)
		var plan := CombatPlan.new(ctx)
		var steps: Array[PlanStep] = [PlanStep.self_skill(s)]
		plan.set_steps(tank, steps)
		var st: CombatPlan.SimState = plan.compile(tank)
		var ap_ok: bool = st.error_index < 0 and not st.entries.is_empty() and st.entries[0].skill_ap == s.level
		var option_ap: bool = s.ap_cost() == s.level
		var cmd: DefendCommand = st.entries[0].build_command(tank, grid) as DefendCommand if not st.entries.is_empty() else null
		var cmd_ok: bool = cmd != null and is_equal_approx(cmd.mitigation_ratio, s.mitigation) and cmd.status_rounds == s.status_rounds and cmd.grant_status == StatusEffects.RIPOSTE
		if cmd != null:
			cmd.on_start()
			cmd.execute()
		var applied: bool = cmd != null and is_equal_approx(tank.defense_mitigation_ratio, s.mitigation) and tank.has_status(StatusEffects.RIPOSTE) and DictUtil.int_of(tank.status_rounds_left, StatusEffects.RIPOSTE) == s.status_rounds
		all_ok = all_ok and ap_ok and option_ap and cmd_ok and applied
	checks["%s: planner AP, Guard mitigation and granted rounds follow the rows" % label] = all_ok

static func _test_riposte_data_models() -> bool:
	var checks: Dictionary = {}
	# rows: tier, level (AP), mitigation, status_rounds
	var reverse: Array[SkillDef] = _model([[1, 3, 0.15, 2], [2, 2, 0.05, 3], [3, 1, 0.0, 3]])
	var ascending: Array[SkillDef] = _model([[1, 1, 0.15, 2], [2, 2, 0.20, 3], [3, 3, 0.25, 3]])
	_check_model("reverse model (AP 3/2/1, mitigation 0.15/0.05/0.00)", reverse, checks)
	_check_model("ascending model (AP 1/2/3, mitigation 0.15/0.20/0.25)", ascending, checks)
	checks["the reverse model has descending AP and a zero-mitigation tier"] = reverse[0].ap_cost() > reverse[1].ap_cost() and reverse[1].ap_cost() > reverse[2].ap_cost() and is_equal_approx(reverse[2].mitigation, 0.0)
	checks["RiposteRules holds only the counter constant"] = RiposteRules.COUNTER_MULT == 0.5 and not (RiposteRules as GDScript).get_script_constant_map().has("GUARD_MITIGATION")
	# The same rows pass catalog validation (mitigation 0.0 on DEFEND, any AP order) in a sandbox fork.
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, SANDBOX))
	var forked: bool = ContentCatalog.fork_to(SANDBOX) == OK
	checks["sandbox fork"] = forked
	if forked:
		var accepted: bool = true
		var models: Array = [["t_rev", [[1, 3, 0.15, 2], [2, 2, 0.05, 3], [3, 1, 0.0, 3]]], ["t_asc", [[1, 1, 0.15, 2], [2, 2, 0.20, 3], [3, 3, 0.25, 3]]]]
		for model: Variant in models:
			var pair: Array = model
			var prefix: String = str(pair[0])
			for row: Variant in (pair[1] as Array):
				var r: Array = row
				var entry: Dictionary = {"id": "%s_%d" % [prefix, int(r[0])], "name": "%s %d" % [prefix, int(r[0])], "kind": "DEFEND", "family": prefix, "tier": int(r[0]), "level": int(r[1]), "mitigation": float(r[2]), "grant_status": "riposte", "status_rounds": int(r[3]), "blocks": 2, "learned_only": true}
				accepted = accepted and ContentCatalog.upsert("skills", entry).is_empty()
		checks["both models are accepted as data (AP in any order, mitigation 0.0)"] = accepted
		var back: SkillDef = ContentFactory.skill(ContentCatalog.get_entry("skills", "t_rev_3"))
		checks["the stored zero-mitigation row reads back exactly"] = is_equal_approx(back.mitigation, 0.0) and back.level == 1 and back.status_rounds == 3
		checks["a sandbox intimidate-style row (power 0, weaken) is accepted"] = ContentCatalog.upsert("skills", {"id": "t_intim", "name": "Intimidate", "kind": "ATTACK", "power_mult": 0.0, "stance": "DEFENSIVE", "status_effect": "weaken", "status_potency": 70, "status_magnitude": 15, "status_rounds": 2, "learned_only": true}).is_empty()
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return _report("Riposte tiers are data only (reverse and ascending models)", checks)
