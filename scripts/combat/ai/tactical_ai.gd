class_name TacticalAI
extends RefCounted

# ==============================================================================
# Tactical RPG - Tactical AI (round planner for computer-controlled units)
# Drafts one unit's whole round as a PlanStep sequence from the state at round
# start, growing a scratch CombatPlan one step at a time so every step is
# validated (stance flow, AP, blocks, paths) exactly like a player's input.
#   1. Withdraw: at low HP, leave through an exit.
#   2. Pick a target (aggressors the nearest, others the weakest).
#   3. Role pattern: guard / focus / back away / Engage (auto-moves + strike).
# Targets may move, so plans can fizzle: that is the planning-phase gamble.
# Used for enemies each round and for the player's AI presets.
# ==============================================================================

const MAX_STEPS: int = 16

static func build_steps(actor: Combatant, ctx: PlanContext, profile: AiProfile) -> Array[PlanStep]:
	var plan := CombatPlan.new(ctx)
	var grid: CombatGrid = ctx.grid
	var foes: Array[Combatant] = ctx.foes_of(actor)
	if actor == null or not actor.is_active() or not grid.has_combatant(actor):
		return plan.steps_of(actor)
	var cap: int = ctx.skill_cap_of(actor)
	var guarded: bool = false
	var focused: bool = false
	var fled: bool = false
	var rests: int = 0
	var reach: int = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor).y

	for iteration: int in range(MAX_STEPS):
		var st: CombatPlan.SimState = plan.compile(actor)
		if st.blocks_left() < 1 or st.withdrawn:
			break

		var hp_ratio: float = float(actor.hp) / float(maxi(1, actor.max_hp))
		if profile.flee_ratio > 0.0 and hp_ratio <= profile.flee_ratio and not fled:
			fled = true
			if plan.add_step(actor, PlanStep.withdraw()).is_empty():
				continue

		if profile.role == AiProfile.Role.SUPPORT and not focused:
			focused = true
			if _try_skills(plan, actor, SkillDef.Kind.FOCUS, cap, profile, Callable()):
				continue

		var ranked: Array[Combatant] = _rank_targets(plan, actor, st, foes, grid, profile)
		if ranked.is_empty():
			break
		var target: Combatant = ranked[0]
		var dist: int = grid.get_distance(st.pos, grid.get_position(target))
		var low_hp: bool = hp_ratio <= profile.defend_below

		if low_hp and not guarded and dist <= reach + 1:
			guarded = true
			if _try_skills(plan, actor, SkillDef.Kind.DEFEND, cap, profile, Callable()):
				continue

		var retreat: bool = false
		match profile.role:
			AiProfile.Role.SNIPER, AiProfile.Role.SUPPORT:
				retreat = dist < reach
			AiProfile.Role.SKIRMISHER:
				retreat = hp_ratio <= profile.defend_below and dist <= 1
		if retreat:
			var back: Vector2i = _retreat_dir(plan, actor, st.pos, foes, reach, grid.get_position(target))
			if back != Vector2i.ZERO and plan.add_step(actor, PlanStep.move(SkillCatalog.lowest(SkillDef.Kind.MOVE), back, 1), false).is_empty():
				continue

		# Best target first; if it can't be reached this round (one move only), try the next.
		var engaged: bool = false
		for candidate: Combatant in ranked:
			var engage: Callable = func(skill: SkillDef) -> PlanStep: return PlanStep.engage(skill, candidate)
			if _try_skills(plan, actor, SkillDef.Kind.ATTACK, cap, profile, engage):
				engaged = true
				break
		if engaged:
			continue
		# Can't afford anything useful: catch a breath to recover AP, then try again.
		if rests < 2 and st.blocks_left() >= 2 and st.ap_left < SkillCatalog.lowest(SkillDef.Kind.ATTACK).ap_cost(actor) + 1:
			rests += 1
			if plan.add_step(actor, PlanStep.self_skill(SkillCatalog.find("rest"))).is_empty():
				continue
		break

	return plan.steps_of(actor)

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

# Tries skills of `kind` in preference order; returns true when one was added.
# `make_step` builds the step from a skill (default: a self step).
static func _try_skills(plan: CombatPlan, actor: Combatant, kind: SkillDef.Kind, cap: int, profile: AiProfile, make_step: Callable) -> bool:
	# Attacks: single-target ones (the AI doesn't aim area attacks yet; ranged units add the off-hand jab).
	var skills: Array[SkillDef] = SkillCatalog.single_attacks(actor, cap) if kind == SkillDef.Kind.ATTACK else SkillCatalog.available(kind, cap)
	skills.reverse()  # highest level first
	if profile.prefer_level > 0:
		skills.sort_custom(func(a: SkillDef, b: SkillDef) -> bool:
			return absi(a.level - profile.prefer_level) < absi(b.level - profile.prefer_level)
		)
	for skill: SkillDef in skills:
		var step: PlanStep
		if make_step.is_valid():
			var raw: Variant = make_step.call(skill)
			step = raw if raw is PlanStep else null
		else:
			step = PlanStep.self_skill(skill)
		if step != null and plan.add_step(actor, step, false).is_empty():
			return true
	return false

# Foes ordered best-first: aggressors prefer the nearest, everyone else the weakest;
# targets that are far or hard to reach are penalised by the route they would need.
static func _rank_targets(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, foes: Array[Combatant], grid: CombatGrid, profile: AiProfile) -> Array[Combatant]:
	var scored: Array[Dictionary] = []
	for f: Combatant in foes:
		if not f.is_active() or not grid.has_combatant(f) or DictUtil.int_of(st.est_hp, f) <= 0:
			continue
		var d: int = grid.get_distance(st.pos, grid.get_position(f))
		var hp_left: float = float(DictUtil.int_of(st.est_hp, f))
		var score: float = float(d) * 100.0 + hp_left if profile.target_pref == AiProfile.TargetPref.NEAREST else hp_left + float(d) * 4.0
		if d > SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor).y:
			var route: Array[Vector2i] = plan.approach_path(actor, st.pos, f)
			score += 400.0 if route.is_empty() else float(route.size()) * 8.0
		scored.append({"foe": f, "score": score})
	scored.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return float(a["score"]) < float(b["score"]))
	var result: Array[Combatant] = []
	for entry: Dictionary in scored:
		var foe: Combatant = entry["foe"]
		result.append(foe)
	return result

# One step that gains distance from the nearest foe but stays within weapon range
# of the current target (so a ranged unit can keep shooting).
static func _retreat_dir(plan: CombatPlan, actor: Combatant, pos: Vector2i, foes: Array[Combatant], reach: int, target_pos: Vector2i) -> Vector2i:
	var grid: CombatGrid = plan.context.grid
	var blocked: Dictionary = plan.blocked_cells(actor)
	var best: Vector2i = Vector2i.ZERO
	var best_gain: int = 0
	var cur_near: int = _nearest_foe(grid, pos, foes)
	for d: Vector2i in CombatPlan.DIRS:
		var nxt: Vector2i = pos + d
		if not grid.is_valid_coord(nxt) or not grid.is_walkable(nxt) or grid.is_exit_tile(nxt) or blocked.has(nxt):
			continue
		if grid.get_distance(nxt, target_pos) > reach:
			continue
		var gain: int = _nearest_foe(grid, nxt, foes) - cur_near
		if gain > best_gain:
			best_gain = gain
			best = d
	return best

static func _nearest_foe(grid: CombatGrid, pos: Vector2i, foes: Array[Combatant]) -> int:
	var best: int = 99
	for f: Combatant in foes:
		if f.is_active() and grid.has_combatant(f):
			best = mini(best, grid.get_distance(pos, grid.get_position(f)))
	return best
