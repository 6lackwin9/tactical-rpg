class_name CombatExtensionsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Extensions Test Suite (S12-A + S4-A, milestone M1)
# Ally-target commands (heal / buff), the haste / taunt durations, round reset and the catalog
# fields. Gap-close and bleed: CombatExtAttacksTestSuite; the AI fixes: CombatExtAiTestSuite;
# shared helpers: CombatExtKit (split by CLN-12). Units are hand-built (fixed strike power, no
# armor) and learn the example skills (mend, rally, taunt_shout, lunge, bleeding_cut) explicitly:
# those are `learned_only`, so no existing unit or number changes. Expectations are
# derived from the rules (SupportRules, StatusEffects, SkillDef), not magic numbers.
# ==============================================================================

const TAG: String = "CombatExt"

static var SANDBOX: String = TestHelpers.sandbox("combat_ext_test")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT EXTENSIONS TEST (S12-A / S4-A)  ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_heal_and_clamp() and ok
	ok = _test_ally_validation() and ok
	ok = _test_buffs_and_haste() and ok
	ok = _test_reset_round_clears_queued_statuses() and ok
	ok = _test_catalog_and_schema() and ok
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return ok

# ------------------------------------------------------------------------------
# Heal / ally-target commands
# ------------------------------------------------------------------------------

static func _test_heal_and_clamp() -> bool:
	var checks: Dictionary = {}
	var mend: SkillDef = SkillCatalog.find("mend")
	var grid: CombatGrid = CombatExtKit.new_grid()
	var healer: Combatant = CombatExtKit.unit("Healer", 100, 10, 1, grid, Vector2i(2, 4), ["mend"])
	var friend: Combatant = CombatExtKit.unit("Friend", 100, 10, 1, grid, Vector2i(4, 4))
	var foe: Combatant = CombatExtKit.unit("Foe", 100, 10, 1, grid, Vector2i(8, 8))
	var allies: Array[Combatant] = [healer, friend]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = CombatExtKit.new_ctx(grid, allies, enemies)
	checks["mend is a SUPPORT ally heal at AP = level"] = mend != null and mend.kind == SkillDef.Kind.SUPPORT and mend.heals() and mend.targets_ally() and mend.ap_cost() == mend.level

	# The heal comes from SupportRules (stat x multiplier), not DamageCalculator.
	var know: int = healer.stats.get_effective_stat("KNOW", healer.get_weapon_context())
	var expected_amount: int = mend.heal_base + roundi(float(know) * mend.heal_mult)
	var amount: int = SupportRules.heal_amount(healer, mend)
	checks["heal = base + round(KNOW x mult)"] = amount == expected_amount and amount > 0
	var sage: Combatant = Combatant.new("Sage", 100, 10, 1)
	sage.stats.add_modifier(StatModifier.new("KNOW", 6, 0.0, "test", "always", -1))
	checks["a higher KNOW heals more"] = SupportRules.heal_amount(sage, mend) > amount and SupportRules.heal_amount(sage, mend) == mend.heal_base + roundi(float(know + 6) * mend.heal_mult)

	friend.hp = 40
	var missing: int = friend.max_hp - friend.hp
	var plan := CombatPlan.new(ctx)
	var problem: String = plan.add_step(healer, PlanStep.ally(mend, friend))
	var st: CombatPlan.SimState = plan.compile(healer)
	var entry: PlanEntry = st.entries[st.entries.size() - 1] if not st.entries.is_empty() else null
	checks["plan accepts a heal in reach"] = problem.is_empty() and entry != null and entry.skill == mend and entry.target == friend
	checks["forecast: expected heal is clamped to missing HP"] = entry != null and entry.est_heal == mini(amount, missing)
	checks["support keeps the stance (no switch cost)"] = entry != null and entry.switch_ap == 0 and entry.stance_after == entry.stance_before and entry.skill_ap == mend.level
	var played: CombatExtKit.Played = CombatExtKit.play(ctx, {healer: plan.steps_of(healer)})
	checks["heal applied (hp + min(heal, missing))"] = friend.hp == 40 + mini(amount, missing)
	checks["support logs a line"] = CombatExtKit.has_message(played, "[Support]")
	var heal_action: CombatResolver.ScheduledAction = null
	for a: CombatResolver.ScheduledAction in played.actions:
		if a.command is SupportCommand:
			heal_action = a
	var log_entry: ActionLogEntry = CombatLogBuilder.from_action(heal_action, 1, 1, allies, grid) if heal_action != null else null
	checks["replay entry renders the heal"] = log_entry != null and log_entry.category == "BUFF" and log_entry.outcome_tag == "+%d HP" % (friend.hp - 40) and log_entry.target_name == "Friend"

	# Clamp: a heal never exceeds max HP.
	friend.hp = friend.max_hp - 2
	var clamp_plan := CombatPlan.new(ctx)
	clamp_plan.add_step(healer, PlanStep.ally(mend, friend))
	var clamp_st: CombatPlan.SimState = clamp_plan.compile(healer)
	checks["forecast clamps at 2"] = not clamp_st.entries.is_empty() and clamp_st.entries[clamp_st.entries.size() - 1].est_heal == 2
	CombatExtKit.play(ctx, {healer: clamp_plan.steps_of(healer)})
	checks["heal clamps to max HP"] = friend.hp == friend.max_hp

	# Full HP: valid but flagged; no revive of the dead.
	var full_plan := CombatPlan.new(ctx)
	var options: Array[PlanOption] = full_plan.target_options(healer, friend)
	var mend_option: PlanOption = null
	for o: PlanOption in options:
		if o.step.skill == mend:
			mend_option = o
	checks["full-HP target: valid option with a warning, no heal"] = mend_option != null and mend_option.valid and mend_option.heal == 0 and mend_option.warning.contains("full HP")
	friend.hp = 40
	var fresh: Array[PlanOption] = CombatPlan.new(ctx).target_options(healer, friend)
	var shown: PlanOption = null
	for o: PlanOption in fresh:
		if o.step.skill == mend:
			shown = o
	checks["preview text shows the heal"] = shown != null and shown.heal == mini(amount, missing) and shown.cost_text().contains("+%d HP" % shown.heal)
	friend.hp = 0
	checks["Combatant.heal never revives"] = (func() -> bool:
		friend.heal(50)
		return friend.hp == 0).call()
	return TestHelpers.report(TAG, "Heal amount, forecast, clamp, log", checks)

static func _test_ally_validation() -> bool:
	var checks: Dictionary = {}
	var mend: SkillDef = SkillCatalog.find("mend")
	var rally: SkillDef = SkillCatalog.find("rally")
	var shout: SkillDef = SkillCatalog.find("taunt_shout")
	var grid: CombatGrid = CombatExtKit.new_grid()
	var healer: Combatant = CombatExtKit.unit("Healer", 100, 10, 1, grid, Vector2i(2, 4), ["mend", "rally", "taunt_shout"])
	var friend: Combatant = CombatExtKit.unit("Friend", 100, 10, 1, grid, Vector2i(3, 6))
	var far_friend: Combatant = CombatExtKit.unit("FarFriend", 100, 10, 1, grid, Vector2i(8, 4))
	var foe: Combatant = CombatExtKit.unit("Foe", 100, 10, 1, grid, Vector2i(6, 0))
	var novice: Combatant = CombatExtKit.unit("Novice", 100, 10, 1, grid, Vector2i(0, 8))
	friend.hp = 50
	far_friend.hp = 50
	var allies: Array[Combatant] = [healer, friend, far_friend, novice]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = CombatExtKit.new_ctx(grid, allies, enemies)
	var plan := CombatPlan.new(ctx)
	checks["enemy is not a legal heal target"] = not plan.add_step(healer, PlanStep.ally(mend, foe)).is_empty()
	checks["a SELF skill can't be aimed at an ally"] = not plan.add_step(healer, PlanStep.ally(shout, friend)).is_empty()
	checks["an ALLY-only skill can't be cast on yourself"] = not plan.add_step(healer, PlanStep.self_skill(rally)).is_empty() and not plan.add_step(healer, PlanStep.ally(rally, healer)).is_empty()
	checks["ALLY_OR_SELF can heal yourself"] = plan.add_step(healer, PlanStep.self_skill(mend)).is_empty()
	plan.clear(healer)
	checks["a SELF skill is fine on yourself"] = plan.add_step(healer, PlanStep.self_skill(shout)).is_empty()
	plan.clear(healer)
	checks["an unlearned unit can't use it"] = not CombatPlan.new(ctx).add_step(novice, PlanStep.ally(mend, friend)).is_empty() and not novice.knows_skill("mend")

	# Out of reach: the healer walks into the skill's reach first (unless rooted / too far).
	var band: Vector2i = mend.support_reach(healer)
	var walk := CombatPlan.new(ctx)
	var walk_problem: String = walk.add_step(healer, PlanStep.ally(mend, far_friend))
	var walk_st: CombatPlan.SimState = walk.compile(healer)
	var moved: bool = false
	for e: PlanEntry in walk_st.entries:
		moved = moved or e.skill.kind == SkillDef.Kind.MOVE
	checks["out of reach: walks into reach, then heals"] = walk_problem.is_empty() and moved and grid.get_distance(walk_st.pos, grid.get_position(far_friend)) <= band.y
	healer.statuses[StatusEffects.ROOT] = true
	checks["rooted healer can't walk into reach"] = not CombatPlan.new(ctx).add_step(healer, PlanStep.ally(mend, far_friend)).is_empty()
	checks["rooted healer still heals in reach"] = CombatPlan.new(ctx).add_step(healer, PlanStep.ally(mend, friend)).is_empty()
	healer.statuses.clear()

	var option_list: Array[PlanOption] = CombatPlan.new(ctx).target_options(healer, friend)
	var offered: Array[String] = []
	for o: PlanOption in option_list:
		if o.valid:
			offered.append(o.step.skill.id)
	checks["clicking an ally offers mend and rally, not the self-only shout"] = offered.has("mend") and offered.has("rally") and not offered.has("taunt_shout")
	var foe_options: Array[PlanOption] = CombatPlan.new(ctx).target_options(healer, foe)
	var foe_support: bool = false
	for o: PlanOption in foe_options:
		foe_support = foe_support or o.step.skill.kind == SkillDef.Kind.SUPPORT
	checks["clicking an enemy offers no support skills"] = not foe_support
	var all_options: Array[PlanOption] = CombatPlan.new(ctx).options(healer)
	var ally_option: bool = false
	for o: PlanOption in all_options:
		ally_option = ally_option or (o.step.mode == PlanStep.Mode.ALLY and o.step.target == friend)
	checks["options() includes ally-target steps"] = ally_option
	friend.hp = 0
	var dead_problem: String = CombatPlan.new(ctx).add_step(healer, PlanStep.ally(mend, friend))
	checks["a defeated ally can't be targeted"] = not dead_problem.is_empty()

	# Resolution: if the ally is gone or out of reach when the heal lands, nothing happens.
	friend.hp = 50
	var cmd := SupportCommand.new(healer, far_friend, mend, grid, mend.blocks)
	cmd.execute()
	checks["the command refuses an ally out of reach"] = cmd.failed and far_friend.hp == 50 and cmd.execution_log.begins_with("[Support Failed]")
	return TestHelpers.report(TAG, "Ally-target validation and options", checks)

# ------------------------------------------------------------------------------
# Buffs: haste and taunt durations
# ------------------------------------------------------------------------------

static func _test_buffs_and_haste() -> bool:
	var checks: Dictionary = {}
	var rally: SkillDef = SkillCatalog.find("rally")
	var shout: SkillDef = SkillCatalog.find("taunt_shout")
	var grid: CombatGrid = CombatExtKit.new_grid()
	var caller: Combatant = CombatExtKit.unit("Caller", 100, 10, 1, grid, Vector2i(2, 4), ["rally", "taunt_shout"])
	var runner: Combatant = CombatExtKit.unit("Runner", 100, 3, 1, grid, Vector2i(3, 4))
	var foe: Combatant = CombatExtKit.unit("Foe", 100, 10, 1, grid, Vector2i(8, 8))
	var allies: Array[Combatant] = [caller, runner]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = CombatExtKit.new_ctx(grid, allies, enemies)
	var plan := CombatPlan.new(ctx)
	checks["rally plans on an ally"] = plan.add_step(caller, PlanStep.ally(rally, runner)).is_empty()
	plan.add_step(caller, PlanStep.self_skill(shout))
	var before_tiles: int = runner.move_per_block()
	CombatExtKit.play(ctx, {caller: plan.steps_of(caller)})
	checks["buffs queue for next round (not active now)"] = runner.pending_statuses.has(StatusEffects.HASTE) and not runner.has_status(StatusEffects.HASTE) and caller.pending_statuses.has(StatusEffects.TAUNT)
	checks["status rounds recorded from the skill"] = DictUtil.int_of(caller.pending_status_rounds, StatusEffects.TAUNT) == shout.status_rounds and DictUtil.int_of(runner.pending_status_rounds, StatusEffects.HASTE) == rally.status_rounds

	var slow_cap: int = CombatPlan.new(ctx).max_move_tiles(runner)
	var goto_far: Vector2i = Vector2i(3 + slow_cap + 1, 4)
	checks["before haste: a slow unit can't move that far"] = not CombatPlan.new(ctx).add_step(runner, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), goto_far)).is_empty()
	StatusEffects.roll_over(runner, 4)
	StatusEffects.roll_over(caller, 4)
	checks["haste active next round: +HASTE_TILES per move block"] = runner.has_status(StatusEffects.HASTE) and runner.move_per_block() == CombatExtKit.base_move(runner) + StatusEffects.HASTE_TILES and before_tiles == CombatExtKit.base_move(runner)
	checks["haste widens the planner's move"] = CombatPlan.new(ctx).add_step(runner, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), goto_far)).is_empty()
	checks["taunt active next round"] = caller.has_status(StatusEffects.TAUNT)
	StatusEffects.roll_over(runner, 4)
	StatusEffects.roll_over(caller, 4)
	checks["haste lasts its one round"] = not runner.has_status(StatusEffects.HASTE) and runner.move_per_block() == CombatExtKit.base_move(runner)
	checks["a 2-round taunt is still up the second round"] = caller.has_status(StatusEffects.TAUNT)
	StatusEffects.roll_over(caller, 4)
	checks["and gone after"] = not caller.has_status(StatusEffects.TAUNT) and caller.status_rounds_left.is_empty()
	# Existing one-round statuses keep their behaviour.
	StatusEffects.try_inflict(runner, StatusEffects.STAGGER, 200, null)
	StatusEffects.roll_over(runner, 4)
	var staggered: bool = runner.has_status(StatusEffects.STAGGER)
	StatusEffects.roll_over(runner, 4)
	checks["stagger still lasts exactly one round"] = staggered and not runner.has_status(StatusEffects.STAGGER)
	return TestHelpers.report(TAG, "Haste and taunt statuses", checks)

# ------------------------------------------------------------------------------
# Catalog + schema
# ------------------------------------------------------------------------------

static func _test_catalog_and_schema() -> bool:
	var checks: Dictionary = {}
	var ids: Array[String] = ["mend", "rally", "taunt_shout", "lunge", "bleeding_cut"]
	var present: bool = true
	var round_trip: bool = true
	for id: String in ids:
		var entry: Dictionary = ContentCatalog.get_entry("skills", id)
		present = present and not entry.is_empty() and ContentCatalog.validate_entry("skills", entry, id).is_empty()
		var rebuilt: Dictionary = ContentFactory.skill_to_entry(ContentFactory.skill(entry))
		round_trip = round_trip and rebuilt == ContentSchema.normalize("skills", entry)
	checks["the five example skills validate"] = present
	checks["fields survive entry -> SkillDef -> entry"] = round_trip
	checks["whole catalog validates"] = ContentCatalog.validate_all().is_empty()
	var in_a_path: bool = false
	for cls: Dictionary in ContentCatalog.entries("classes"):
		for fam: Variant in DictUtil.dict_of(cls.get("skill_path", {})):
			in_a_path = in_a_path or ids.has(str(fam))
	checks["example skills sit in class paths (M2c-1), still learned-only"] = in_a_path
	checks["learned_only: hand-built units don't know them until they learn them"] = (func() -> bool:
		var c := Combatant.new("U", 10, 10, 1)
		var before: bool = c.knows_skill("mend") or c.knows_skill("lunge")
		c.learn_skill("mend")
		return not before and c.knows_skill("mend") and not c.knows_skill("lunge") and c.knows_skill("strike")).call()

	# Class-derived units get them through extra_skills.
	var with_extra: Combatant = ContentFactory.unit({"id": "t_healer", "name": "T", "class_id": "mage", "level": 3, "side": "party", "extra_skills": ["mend"]})
	var without: Combatant = ContentFactory.unit({"id": "t_plain", "name": "P", "class_id": "mage", "level": 3, "side": "party"})
	checks["extra_skills grants a learned-only skill; others lack it"] = with_extra.can_use(SkillCatalog.find("mend")) and not without.can_use(SkillCatalog.find("mend"))

	# Status resist keys and passive validation include bleed (and refuse buffs).
	var keys_raw: Variant = ContentSchema.field("units", "status_resist").get("keys", [])
	var keys: Array = keys_raw if keys_raw is Array else []
	checks["status_resist keys include bleed"] = keys.has(StatusEffects.BLEED) and keys.has(StatusEffects.STAGGER)
	var exists: Callable = func(_id: String) -> bool: return true
	checks["passive resist accepts bleed, refuses haste"] = Passives.validate_effects([{"type": "resist", "status": "bleed", "value": 25}], exists).is_empty() and not Passives.validate_effects([{"type": "resist", "status": "haste", "value": 25}], exists).is_empty()
	checks["status option lists: debuffs for attacks, buffs for support"] = ContentSchema.options_of({"options_from": "status"}).has("bleed") and not ContentSchema.options_of({"options_from": "status"}).has("haste") and ContentSchema.options_of({"options_from": "status_buff"}).has("haste") and ContentSchema.options_of({"options_from": "status_buff"}).has("taunt")

	# Validation refuses bad data (sandbox copy; nothing is saved).
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		checks["heal per stat needs a stat"] = not ContentCatalog.upsert("skills", {"id": "bad_heal", "name": "x", "kind": "SUPPORT", "heal_mult": 1.0}).is_empty()
		checks["a support skill needs an effect"] = not ContentCatalog.upsert("skills", {"id": "bad_empty", "name": "x", "kind": "SUPPORT"}).is_empty()
		checks["grants must be a buff"] = not ContentCatalog.upsert("skills", {"id": "bad_grant", "name": "x", "kind": "SUPPORT", "grant_status": "stun"}).is_empty()
		checks["taunt must target SELF"] = not ContentCatalog.upsert("skills", {"id": "bad_taunt", "name": "x", "kind": "SUPPORT", "grant_status": "taunt", "target_mode": "ALLY"}).is_empty()
		checks["attack can't grant or heal"] = not ContentCatalog.upsert("skills", {"id": "bad_atk", "name": "x", "kind": "ATTACK", "heal_base": 5}).is_empty()
		checks["attack can only inflict debuffs"] = not ContentCatalog.upsert("skills", {"id": "bad_buffatk", "name": "x", "kind": "ATTACK", "status_effect": "haste", "status_potency": 50}).is_empty()
		checks["area attacks can't be gap-closes"] = not ContentCatalog.upsert("skills", {"id": "bad_dash", "name": "x", "kind": "ATTACK", "area": "PLUS", "dash_tiles": 2}).is_empty()
		checks["only attacks gap-close"] = not ContentCatalog.upsert("skills", {"id": "bad_dash2", "name": "x", "kind": "SUPPORT", "grant_status": "haste", "dash_tiles": 2}).is_empty()
		checks["a valid heal is accepted"] = ContentCatalog.upsert("skills", {"id": "good_heal", "name": "Good", "kind": "SUPPORT", "heal_stat": "INT", "heal_mult": 2.0, "heal_base": 3}).is_empty()
		var good: SkillDef = SkillCatalog.find("good_heal")
		checks["and builds a SkillDef with its heal"] = good != null and good.heals() and good.heal_stat == "INT"
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Catalog and schema for the new fields", checks)

# Undoing a round (round 2+; round 1 restarts the battle) discards statuses and durations queued during it.
static func _test_reset_round_clears_queued_statuses() -> bool:
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
		session.plan_apply_preset_all("balanced")
		session.confirm_plan()
		var victim: Combatant = session.enemies[0]
		victim.pending_statuses[StatusEffects.BLEED] = true
		victim.pending_status_rounds[StatusEffects.BLEED] = 5
		session.reset_round()
		checks["queued statuses cleared"] = victim.pending_statuses.is_empty()
		checks["queued durations cleared"] = victim.pending_status_rounds.is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "Round reset discards queued statuses and their durations", checks)
