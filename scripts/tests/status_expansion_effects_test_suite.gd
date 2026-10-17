class_name StatusExpansionEffectsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Expansion Effects Test Suite (S12S-L commit 2)
# Rules 1-7: slow tiles per move block, expose / fortify armor rating (compute and planner estimate),
# the round-end order bleed -> poison -> burn -> regen with its log lines, heals and per-step rows,
# poison stacking, granted magnitudes and sources (SupportRules, DefendCommand), resist rolls,
# magnitude defaults and status chips, the Forge magnitude range. Expectations are derived from the
# registry rows and the spec rules; no shipped content is read or written.
# ==============================================================================

const TAG: String = "StatusEffectsS12S"
const SURE: int = 500   # potency far above any resistance: the chance clamps to 100

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING STATUS EXPANSION EFFECTS TEST (S12S-L)  ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_slow() and ok
	ok = _test_armor() and ok
	ok = _test_round_end() and ok
	ok = _test_round_end_defeat() and ok
	ok = _test_poison_stacking() and ok
	ok = _test_support_grant() and ok
	ok = _test_defend_grant() and ok
	ok = _test_resist_and_defaults() and ok
	ok = _test_forge_range() and ok
	return ok

# ------------------------------------------------------------------------------

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])

static func _rng(seed_value: int) -> RandomNumberGenerator:
	var r := RandomNumberGenerator.new()
	r.seed = seed_value
	return r

# Base tiles per block for a speed (Combatant.move_per_block before statuses): clamp(floor((speed + 2) / 5), 1, 4).
static func _speed_for_tiles(tiles: int) -> int:
	return tiles * 5 - 2

static func _active(c: Combatant, status: String, magnitude: int = 0, source: String = "") -> void:
	StatusEffects.queue(c, status, 2, magnitude, source)
	StatusEffects.roll_over(c, -1)

# Acceptance 3 (rule 1): slow takes SLOW_TILES off (floor 1), haste and slow cancel.
static func _test_slow() -> bool:
	var checks: Dictionary = {}
	var one := Combatant.new("One", 100, _speed_for_tiles(1), 1)
	checks["fixture: base 1 tile per block"] = one.move_per_block() == 1
	_active(one, StatusEffects.SLOW)
	checks["a slowed unit with base 1 still moves 1"] = one.move_per_block() == 1
	var three := Combatant.new("Three", 100, _speed_for_tiles(3), 1)
	var base: int = three.move_per_block()
	checks["fixture: base 3 tiles per block"] = base == 3
	_active(three, StatusEffects.SLOW)
	checks["base 3 slowed gives base - SLOW_TILES (2)"] = three.move_per_block() == base - StatusEffects.SLOW_TILES and three.move_per_block() == 2
	var both := Combatant.new("Both", 100, _speed_for_tiles(3), 1)
	StatusEffects.queue(both, StatusEffects.SLOW, 1)
	StatusEffects.queue(both, StatusEffects.HASTE, 1)
	StatusEffects.roll_over(both, -1)
	checks["hasted and slowed gives the base"] = both.move_per_block() == base
	var ctx: PlanContext = TestHelpers.hand_ctx(_grid(), [three], [], 3)
	var plan := CombatPlan.new(ctx)
	checks["the plan's move preview uses the slowed tiles per block"] = plan.max_move_tiles(three) == three.move_per_block() * CombatPlan.MAX_MOVE_BLOCKS
	StatusEffects.queue(three, StatusEffects.SLOW, 1)
	checks["a queued slow does not change this round's tiles"] = three.move_per_block() == 2
	return TestHelpers.report(TAG, "Slow: tiles per move block (acceptance 3)", checks)

# Acceptance 4 (rule 2): armor_rating = max(0, base + fortify - expose); compute and the planner estimate agree.
static func _test_armor() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var atk: Combatant = TestHelpers.unit("Atk", grid, Vector2i(3, 4))
	var foe: Combatant = TestHelpers.unit("Foe", grid, Vector2i(4, 4))
	foe.armor_override = 10
	checks["fixture: base rating 10"] = DamageCalculator.armor_rating(foe) == 10 and DamageCalculator.base_armor_rating(foe) == 10
	_active(foe, StatusEffects.EXPOSE, 15, "Atk")
	checks["expose above the base rating gives 0"] = DamageCalculator.armor_rating(foe) == 0 and DamageCalculator.base_armor_rating(foe) == 10
	var strike: SkillDef = SkillCatalog.find("strike")
	var stance: CombatTypes.StanceType = strike.stance()
	var plan := CombatPlan.new(TestHelpers.hand_ctx(grid, [atk], [foe], 3))
	var est_exposed: int = plan.estimate_damage(atk, strike, stance, foe)
	var res_exposed: Dictionary = DamageCalculator.compute(atk, foe, false, false, strike.power_for(atk), false, null, stance, false)
	checks["compute reports the exposed rating"] = DictUtil.to_int(res_exposed["armor_rating"]) == 0
	checks["planner estimate = compute before crit (exposed)"] = est_exposed == DictUtil.to_int(res_exposed["final_dmg"])
	StatusEffects.remove(foe, StatusEffects.EXPOSE)
	var est_base: int = plan.estimate_damage(atk, strike, stance, foe)
	_active(foe, StatusEffects.FORTIFY, 8, "Ally")
	checks["fortify 8 on rating 10 gives 18"] = DamageCalculator.armor_rating(foe) == 18
	var est_fort: int = plan.estimate_damage(atk, strike, stance, foe)
	var res_fort: Dictionary = DamageCalculator.compute(atk, foe, false, false, strike.power_for(atk), false, null, stance, false)
	checks["compute reports the fortified rating"] = DictUtil.to_int(res_fort["armor_rating"]) == 18
	checks["planner estimate = compute before crit (fortified)"] = est_fort == DictUtil.to_int(res_fort["final_dmg"])
	checks["exposed hits harder and fortified softer than the base"] = est_exposed > est_base and est_fort < est_base
	var cmd := AttackCommand.new(atk, foe, strike.power_for(atk), 1, 0, grid)
	cmd.rng = _rng(3)
	cmd.execute()
	var hit: int = DictUtil.to_int(DamageCalculator.compute(atk, foe, false, false, strike.power_for(atk), false, _rng(3))["final_dmg"])
	checks["a resolved attack uses the fortified rating"] = cmd.last_damage == hit
	var queued: Combatant = TestHelpers.unit("Queued", grid, Vector2i(6, 6))
	queued.armor_override = 10
	StatusEffects.queue(queued, StatusEffects.EXPOSE, 2, 5, "Atk")
	checks["a queued expose does not change this round's rating"] = DamageCalculator.armor_rating(queued) == 10
	return TestHelpers.report(TAG, "Expose / fortify armor rating (acceptance 4)", checks)

# A resolver with `units` registered and a log collector.
static func _resolver(units: Array[Combatant], log_lines: Array[String]) -> CombatResolver:
	var r := CombatResolver.new()
	for c: Combatant in units:
		r.register_combatant(c)
	r.log_emitted.connect(func(msg: String) -> void: log_lines.append(msg))
	return r

static func _index_of_prefix(lines: Array[String], prefix: String) -> int:
	for i: int in range(lines.size()):
		if lines[i].begins_with(prefix):
			return i
	return -1

# Acceptance 5 (rule 3): bleed, poison 6, burn 5, regen 5 on a 200 max HP unit at 150 HP.
static func _test_round_end() -> bool:
	var checks: Dictionary = {}
	var c := Combatant.new("Tick", 200, 10, 1)
	c.hp = 150
	StatusEffects.queue(c, StatusEffects.BLEED, 1, 0, "Cutter")
	StatusEffects.queue(c, StatusEffects.POISON, 2, 6, "Viper")
	StatusEffects.queue(c, StatusEffects.BURN, 2, 5, "Torch")
	StatusEffects.queue(c, StatusEffects.REGEN, 2, 5, "Healer")
	StatusEffects.roll_over(c, -1)
	var bleed: int = StatusEffects.bleed_damage(c)
	var burn: int = maxi(1, roundi(200.0 * 5.0 / 100.0))
	var regen: int = maxi(1, roundi(200.0 * 5.0 / 100.0))
	var lines: Array[String] = []
	var r: CombatResolver = _resolver([c], lines)
	var dealt: Dictionary = r.apply_round_end_effects()
	checks["rule-derived amounts: bleed 12, poison 6, burn 10, regen 10"] = bleed == 12 and burn == 10 and regen == 10
	checks["HP after = 150 - 12 - 6 - 10 + 10 = 132"] = c.hp == 150 - bleed - 6 - burn + regen and c.hp == 132
	checks["the return value is bleed + poison + burn"] = DictUtil.int_of(dealt, c) == bleed + 6 + burn
	checks["last_round_end_heals holds the regen"] = DictUtil.int_of(r.last_round_end_heals, c) == regen
	var i_bleed: int = _index_of_prefix(lines, "[Bleed]")
	var i_poison: int = _index_of_prefix(lines, "[Poison]")
	var i_burn: int = _index_of_prefix(lines, "[Burn]")
	var i_regen: int = _index_of_prefix(lines, "[Regen]")
	checks["log lines appear in the order bleed, poison, burn, regen"] = i_bleed >= 0 and i_bleed < i_poison and i_poison < i_burn and i_burn < i_regen
	checks["poison log line is exact"] = lines.has("[Poison] Tick takes 6 poison damage (HP %d/200)" % (150 - bleed - 6))
	checks["burn log line is exact"] = lines.has("[Burn] Tick burns for %d damage (HP %d/200)" % [burn, 150 - bleed - 6 - burn])
	checks["regen log line is exact"] = lines.has("[Regen] Tick regenerates %d HP (HP 132/200)" % regen)
	var steps: Array[String] = []
	var sources: Array[String] = []
	var amounts: Array[int] = []
	for row: Dictionary in r.last_round_end_steps:
		steps.append(str(row["status"]))
		sources.append(str(row["source"]))
		amounts.append(DictUtil.to_int(row["amount"]))
	checks["last_round_end_steps lists each step in order with its source and amount"] = steps == ["bleed", "poison", "burn", "regen"] and sources == ["Cutter", "Viper", "Torch", "Healer"] and amounts == [bleed, 6, burn, regen]
	# Regen is capped at missing HP; a full unit records nothing.
	var near := Combatant.new("Near", 200, 10, 1)
	near.hp = 197
	_active(near, StatusEffects.REGEN, 5, "Healer")
	var full := Combatant.new("Full", 200, 10, 1)
	_active(full, StatusEffects.REGEN, 5, "Healer")
	var lines2: Array[String] = []
	var r2: CombatResolver = _resolver([near, full], lines2)
	r2.apply_round_end_effects()
	checks["regen is capped at missing HP"] = near.hp == 200 and DictUtil.int_of(r2.last_round_end_heals, near) == 3
	checks["a unit at full HP regenerates nothing (no heal, no log)"] = not r2.last_round_end_heals.has(full) and _index_of_prefix(lines2, "[Regen] Full") < 0
	checks["each call refills the heal and step records"] = r2.last_round_end_steps.size() == 1
	return TestHelpers.report(TAG, "Round end: bleed -> poison -> burn -> regen (acceptance 5)", checks)

static func _test_round_end_defeat() -> bool:
	var checks: Dictionary = {}
	var c := Combatant.new("Frail", 200, 10, 1)
	c.hp = 5
	StatusEffects.queue(c, StatusEffects.POISON, 2, 6, "Viper")
	StatusEffects.queue(c, StatusEffects.BURN, 2, 5, "Torch")
	StatusEffects.queue(c, StatusEffects.REGEN, 2, 5, "Healer")
	StatusEffects.roll_over(c, -1)
	var lines: Array[String] = []
	var r: CombatResolver = _resolver([c], lines)
	var dealt: Dictionary = r.apply_round_end_effects()
	checks["poison reduces the unit to 0 (the return value is the step damage, as for bleed)"] = c.hp == 0 and DictUtil.int_of(dealt, c) == 6
	checks["the poison line carries the DEFEATED suffix"] = lines.has("[Poison] Frail takes 6 poison damage (HP 0/200) -> Frail DEFEATED!")
	checks["the step row records the HP actually lost (5)"] = r.last_round_end_steps.size() == 1 and DictUtil.to_int(r.last_round_end_steps[0]["amount"]) == 5
	checks["a defeated unit neither burns nor regenerates"] = _index_of_prefix(lines, "[Burn]") < 0 and _index_of_prefix(lines, "[Regen]") < 0 and r.last_round_end_heals.is_empty()
	var gone := Combatant.new("Gone", 200, 10, 1)
	gone.hp = 100
	_active(gone, StatusEffects.POISON, 6, "Viper")
	gone.withdraw()
	var r2: CombatResolver = _resolver([gone], [] as Array[String])
	r2.apply_round_end_effects()
	checks["a withdrawn unit takes no round-end steps"] = gone.hp == 100 and r2.last_round_end_steps.is_empty()
	return TestHelpers.report(TAG, "Round end: a defeated unit does not regenerate", checks)

# Acceptance 6 (rule 4): poison 10 + 25 queued in one round -> one active instance of 30 (cap); +10 next round keeps 30.
static func _test_poison_stacking() -> bool:
	var checks: Dictionary = {}
	var p: String = StatusEffects.POISON
	var cap: int = StatusRegistry.cap_of(p)
	var c := Combatant.new("Pois", 200, 10, 1)
	StatusEffects.queue(c, p, 2, 10, "A")
	StatusEffects.queue(c, p, 3, 25, "B")
	StatusEffects.roll_over(c, -1)
	checks["cap is 30 (registry)"] = cap == 30
	checks["10 + 25 queued together become one active instance at the cap"] = c.has_status(p) and DictUtil.int_of(c.status_magnitudes, p) == mini(cap, 35)
	checks["rounds take the longer (3)"] = DictUtil.int_of(c.status_rounds_left, p) == 3
	StatusEffects.queue(c, p, 1, 10, "C")
	StatusEffects.roll_over(c, -1)
	checks["a third of 10 next round keeps the cap"] = DictUtil.int_of(c.status_magnitudes, p) == cap
	checks["rounds stay the longer (3 - 1 carried vs 1 queued)"] = DictUtil.int_of(c.status_rounds_left, p) == 2
	checks["the newest applier is the source"] = str(c.status_sources.get(p, "")) == "C"
	var d := Combatant.new("Stack", 200, 10, 1)
	_active(d, p, 4, "A")
	StatusEffects.queue(d, p, 2, 5, "B")
	StatusEffects.roll_over(d, -1)
	checks["below the cap, active 4 + pending 5 = 9"] = DictUtil.int_of(d.status_magnitudes, p) == 9
	var e := Combatant.new("Burnt", 200, 10, 1)
	_active(e, StatusEffects.BURN, 4, "A")
	StatusEffects.queue(e, StatusEffects.BURN, 2, 3, "B")
	StatusEffects.roll_over(e, -1)
	checks["burn (refresh_max_duration) keeps the stronger (4), not the sum"] = DictUtil.int_of(e.status_magnitudes, StatusEffects.BURN) == 4
	return TestHelpers.report(TAG, "Poison stacks to its cap (acceptance 6)", checks)

static func _support_skill(status: String, magnitude: int, rounds: int) -> SkillDef:
	var skill: SkillDef = SkillDef.create("t_sx_grant_%s" % status, "Grant", SkillDef.Kind.SUPPORT, 1, 1, "")
	skill.grant_status = status
	skill.status_magnitude = magnitude
	skill.status_rounds = rounds
	return skill

# Acceptance 7 (rule 5): a SUPPORT regen grant with magnitude 5 starts next round, lasts status_rounds, records the caster.
static func _test_support_grant() -> bool:
	var checks: Dictionary = {}
	var caster := Combatant.new("Mender", 100, 10, 1)
	var ally := Combatant.new("Ally", 200, 10, 1)
	ally.hp = 100
	var rounds: int = 2
	var skill: SkillDef = _support_skill(StatusEffects.REGEN, 5, rounds)
	var result: Dictionary = SupportRules.apply(caster, ally, skill, skill.status_rounds)
	checks["the grant is reported"] = str(result["status"]) == StatusEffects.REGEN
	checks["pending first: not active this round"] = not ally.has_status(StatusEffects.REGEN) and ally.pending_statuses.has(StatusEffects.REGEN)
	checks["pending carries magnitude 5 and the caster"] = DictUtil.int_of(ally.pending_status_magnitudes, StatusEffects.REGEN) == 5 and str(ally.pending_status_sources.get(StatusEffects.REGEN, "")) == "Mender"
	var lasted: int = 0
	for i: int in range(rounds + 2):
		StatusEffects.roll_over(ally, -1)
		if ally.has_status(StatusEffects.REGEN):
			lasted += 1
			checks["active round %d: source and magnitude" % (i + 1)] = str(ally.status_sources.get(StatusEffects.REGEN, "")) == "Mender" and DictUtil.int_of(ally.status_magnitudes, StatusEffects.REGEN) == 5
	checks["it lasts its status_rounds once active"] = lasted == rounds
	var fortify: SkillDef = _support_skill(StatusEffects.FORTIFY, 8, 2)
	var tank := Combatant.new("Tank", 200, 10, 1)
	tank.armor_override = 10
	SupportRules.apply(caster, tank, fortify, fortify.status_rounds)
	StatusEffects.roll_over(tank, -1)
	checks["a SUPPORT fortify grant raises the rating next round"] = DamageCalculator.armor_rating(tank) == 18 and str(tank.status_sources.get(StatusEffects.FORTIFY, "")) == "Mender"
	var option := PlanOption.new()
	option.step = PlanStep.new()
	option.step.skill = skill
	option.valid = true
	option.grants = StatusEffects.REGEN
	checks["the plan option names the grant with its magnitude"] = option.cost_text().contains("Regenerating 5% max HP")
	return TestHelpers.report(TAG, "SUPPORT grants carry magnitude and caster (acceptance 7)", checks)

# Rule 5: DefendCommand passes the skill's magnitude and the defender's name; fortify (a support-grant buff) queues.
static func _test_defend_grant() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = TestHelpers.unit("Bulwark", grid, Vector2i(2, 2))
	tank.armor_override = 10
	var skill: SkillDef = SkillDef.create("t_sx_def", "Hold", SkillDef.Kind.DEFEND, 1, 1, "")
	skill.mitigation = 0.3
	skill.grant_status = StatusEffects.FORTIFY
	skill.status_magnitude = 6
	skill.status_rounds = 2
	var entry: PlanEntry = PlanEntry.create(skill, 1, 0)
	var cmd: DefendCommand = entry.build_command(tank, grid) as DefendCommand
	checks["PlanEntry carries the magnitude to the DefendCommand"] = cmd != null and cmd.status_magnitude == 6
	var msg: String = cmd.on_start() if cmd != null else ""
	checks["a DEFEND fortify grant queues for next round"] = not tank.has_status(StatusEffects.FORTIFY) and tank.pending_statuses.has(StatusEffects.FORTIFY) and msg.contains("next round")
	checks["it carries the magnitude and the defender as source"] = DictUtil.int_of(tank.pending_status_magnitudes, StatusEffects.FORTIFY) == 6 and str(tank.pending_status_sources.get(StatusEffects.FORTIFY, "")) == "Bulwark"
	StatusEffects.roll_over(tank, -1)
	checks["next round the rating is base + 6"] = DamageCalculator.armor_rating(tank) == 16
	var riposte := DefendCommand.new(tank, 0.3, 1, 1)
	riposte.grant_status = StatusEffects.RIPOSTE
	riposte.status_rounds = 2
	riposte.on_start()
	checks["a riposte grant is active now with the defender as source"] = tank.has_status(StatusEffects.RIPOSTE) and str(tank.status_sources.get(StatusEffects.RIPOSTE, "")) == "Bulwark"
	return TestHelpers.report(TAG, "DEFEND grants carry magnitude and source", checks)

# Rule 6 and the magnitude default: resist rolls cover the new debuffs; magnitude 0 on a ranged status stores the minimum.
static func _test_resist_and_defaults() -> bool:
	var checks: Dictionary = {}
	var immune := Combatant.new("Immune", 100, 10, 1)
	var open := Combatant.new("Open", 100, 10, 1)
	var all_resist: bool = true
	var all_land: bool = true
	for id: String in [StatusEffects.SLOW, StatusEffects.EXPOSE, StatusEffects.POISON, StatusEffects.BURN]:
		immune.status_resist[id] = 100
		all_resist = all_resist and StatusEffects.apply_chance(immune, id, 100) == 0 and not StatusEffects.try_inflict(immune, id, 100, _rng(1), 1, StatusRegistry.magnitude_min(id), "Foe")
		all_land = all_land and StatusEffects.try_inflict(open, id, SURE, _rng(1), 1, StatusRegistry.magnitude_max(id), "Foe")
	checks["status_resist 100 blocks slow, expose, poison and burn"] = all_resist and immune.pending_statuses.is_empty()
	checks["sure potency lands each of them with the applier as source"] = all_land and str(open.pending_status_sources.get(StatusEffects.BURN, "")) == "Foe"
	var z := Combatant.new("Zero", 100, 10, 1)
	StatusEffects.grant_now(z, StatusEffects.FORTIFY, 2, 0, "Self")
	checks["grant_now with magnitude 0 on a ranged status stores the registry minimum"] = DictUtil.int_of(z.status_magnitudes, StatusEffects.FORTIFY) == StatusRegistry.magnitude_min(StatusEffects.FORTIFY)
	StatusEffects.queue(z, StatusEffects.EXPOSE, 2, 0)
	checks["queue with magnitude 0 on a ranged status stores the registry minimum"] = DictUtil.int_of(z.pending_status_magnitudes, StatusEffects.EXPOSE) == StatusRegistry.magnitude_min(StatusEffects.EXPOSE)
	StatusEffects.queue(z, StatusEffects.SLOW, 1, 0)
	checks["a status without a range stores no magnitude"] = not z.pending_status_magnitudes.has(StatusEffects.SLOW)
	checks["chip text adds the magnitude with its unit"] = StatusEffects.chip_text(z, StatusEffects.FORTIFY) == "Fortified %d armor" % StatusRegistry.magnitude_min(StatusEffects.FORTIFY) \
		and StatusEffects.label_with_magnitude(StatusEffects.POISON, 12) == "Poisoned 12 HP" and StatusEffects.label_with_magnitude(StatusEffects.SLOW, 0) == "Slowed"
	return TestHelpers.report(TAG, "Resist rolls, magnitude defaults, chips (rule 6)", checks)

# Reviewer item: the Forge magnitude field takes its range from the magnitude status's registry row.
static func _test_forge_range() -> bool:
	var checks: Dictionary = {}
	var expose: Dictionary = SkillFormModel.state({"kind": "ATTACK", "status_effect": StatusEffects.EXPOSE}, "status_magnitude")
	checks["expose: min / max from the registry"] = DictUtil.to_int(expose.get("min", -1)) == StatusRegistry.magnitude_min(StatusEffects.EXPOSE) and DictUtil.to_int(expose.get("max", -1)) == StatusRegistry.magnitude_max(StatusEffects.EXPOSE)
	var regen: Dictionary = SkillFormModel.state({"kind": "SUPPORT", "grant_status": StatusEffects.REGEN}, "status_magnitude")
	checks["a regen grant: min / max from the registry"] = DictUtil.to_int(regen.get("min", -1)) == StatusRegistry.magnitude_min(StatusEffects.REGEN) and DictUtil.to_int(regen.get("max", -1)) == StatusRegistry.magnitude_max(StatusEffects.REGEN)
	var stagger: Dictionary = SkillFormModel.state({"kind": "ATTACK", "status_effect": StatusEffects.STAGGER}, "status_magnitude")
	checks["stagger: max 0 with a reason naming the ranged statuses"] = DictUtil.to_int(stagger.get("max", -1)) == 0 and str(stagger.get("reason", "")).contains(", ".join(StatusRegistry.magnitude_ids()))
	return TestHelpers.report(TAG, "Forge magnitude range follows the registry", checks)
