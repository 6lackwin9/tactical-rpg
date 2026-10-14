class_name TacticalAI
extends RefCounted

# ==============================================================================
# Tactical RPG - Tactical AI (round planner for computer-controlled units)
# Drafts one unit's whole round as a PlanStep sequence from the state at round
# start, growing a scratch CombatPlan one step at a time so every step is
# validated (stance flow, AP, blocks, paths) exactly like a player's input.
#   1. Withdraw: at low HP, leave through an exit (never for a reckless unit).
#   2. SUPPORT role: heal the most wounded ally (at or below profile.heal_below).
#   3. Pick a target: a reachable taunter first, else aggressors the nearest,
#      others the weakest (reckless units always the nearest).
#   4. Role pattern: guard / focus (only if it does not waste the turn) / kite to
#      maximum range (ranged) / Engage (auto-moves + strike).
#   5. No idle turns: when no target can be struck this round the unit still
#      advances along its route; it only rests when AP is the problem.
# Targets may move, so plans can fizzle: that is the planning-phase gamble.
# Used for enemies each round and for the player's AI presets (the player's own
# planner is never forced: taunt only steers TacticalAI).
# ==============================================================================

const MAX_STEPS: int = 16
const MAX_HEALS: int = 2

# With shared, the draft is written into that plan (replacing the actor's steps), so it
# sees - and avoids - the moves its friends have already planned there.
static func build_steps(actor: Combatant, ctx: PlanContext, profile: AiProfile, shared: CombatPlan = null) -> Array[PlanStep]:
	var plan: CombatPlan = shared if shared != null else CombatPlan.new(ctx)
	plan.clear(actor)
	var grid: CombatGrid = ctx.grid
	var foes: Array[Combatant] = ctx.foes_of(actor)
	if actor == null or not actor.is_active() or not grid.has_combatant(actor):
		return plan.steps_of(actor)
	var cap: int = ctx.skill_cap_of(actor)
	var guarded: bool = false
	var focused: bool = false
	var fled: bool = false
	var healed: Array[Combatant] = []   # friends already healed this round (the plan doesn't track their HP)
	var rests: int = 0
	var basic: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
	var band: Vector2i = basic.reach_for(actor)
	var reach: int = band.y

	for iteration: int in range(MAX_STEPS):
		var st: CombatPlan.SimState = plan.compile(actor)
		if st.blocks_left() < 1 or st.withdrawn:
			break

		var hp_ratio: float = float(actor.hp) / float(maxi(1, actor.max_hp))
		if profile.flee_ratio > 0.0 and not profile.reckless and hp_ratio <= profile.flee_ratio and not fled:
			fled = true
			if plan.add_step(actor, PlanStep.withdraw()).is_empty():
				continue

		if profile.role == AiProfile.Role.SUPPORT and healed.size() < MAX_HEALS:
			var patient: Combatant = _try_heal(plan, actor, profile, cap, healed)
			if patient != null:
				healed.append(patient)
				continue

		var ranked: Array[Combatant] = _rank_targets(plan, actor, st, foes, grid, profile)
		if ranked.is_empty():
			break
		var target: Combatant = ranked[0]
		var dist: int = grid.get_distance(st.pos, grid.get_position(target))
		var low_hp: bool = hp_ratio <= profile.defend_below and not profile.reckless

		# Buff (Focus) first only when the strike still fits afterwards: the buff is an
		# action, so a unit that has to walk to its target must not cast before moving.
		if not focused and (profile.role == AiProfile.Role.SUPPORT or actor.attack_range > 1):
			focused = true
			if _can_strike_in_place(actor, cap, dist, profile) and _buff_then_engage(plan, actor, ranked, cap, profile):
				continue

		if low_hp and not guarded and dist <= reach + 1:
			guarded = true
			if _try_skills(plan, actor, SkillDef.Kind.DEFEND, cap, profile, Callable()):
				continue

		if not profile.reckless:
			match profile.role:
				AiProfile.Role.SNIPER, AiProfile.Role.SUPPORT:
					# Kite: step back to maximum range, then shoot (only if the shot still fits).
					if dist < reach and _kite_then_engage(plan, actor, st, foes, ranked, band, cap, profile):
						continue
				AiProfile.Role.SKIRMISHER:
					if hp_ratio <= profile.defend_below and dist <= 1:
						var back: Vector2i = _retreat_dir(plan, actor, st.pos, foes, reach, grid.get_position(target))
						if back != Vector2i.ZERO and plan.add_step(actor, PlanStep.move(SkillCatalog.lowest(SkillDef.Kind.MOVE), back, 1), false).is_empty():
							continue

		# Best target first; if it can't be reached this round (one move only), try the next.
		if _engage_any(plan, actor, ranked, cap, profile):
			continue
		# Nothing can be struck this round, but a target is out there: close the distance
		# instead of idling (the unit starts next round already in contact).
		if _advance_toward(plan, actor, st, ranked):
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
	var skills: Array[SkillDef] = SkillCatalog.single_attacks(actor, cap) if kind == SkillDef.Kind.ATTACK else SkillCatalog.available(kind, cap, actor)
	skills.reverse()  # highest level first
	if profile.prefer_level > 0:
		skills.sort_custom(func(a: SkillDef, b: SkillDef) -> bool:
			return absi(a.level - profile.prefer_level) < absi(b.level - profile.prefer_level)
		)
	# The off-hand jab is a last resort for ranged units (only when nothing else reaches).
	var ordered: Array[SkillDef] = []
	for skill: SkillDef in skills:
		if not skill.offhand:
			ordered.append(skill)
	for skill: SkillDef in skills:
		if skill.offhand:
			ordered.append(skill)
	for skill: SkillDef in ordered:
		var step: PlanStep
		if make_step.is_valid():
			var raw: Variant = make_step.call(skill)
			step = raw if raw is PlanStep else null
		else:
			step = PlanStep.self_skill(skill)
		if step != null and plan.add_step(actor, step, false).is_empty():
			return true
	return false

# Attacks the first target in `ranked` the unit can strike this round.
static func _engage_any(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool:
	for candidate: Combatant in ranked:
		var engage: Callable = func(skill: SkillDef) -> PlanStep: return PlanStep.engage(skill, candidate)
		if _try_skills(plan, actor, SkillDef.Kind.ATTACK, cap, profile, engage):
			return true
	return false

# True when a strike is possible from where the unit stands (no move needed): the basic
# reach band, or - for the SUPPORT role, which opens with a buff and then jabs - any
# single-target attack the unit knows (e.g. the ranged off-hand jab at point-blank).
static func _can_strike_in_place(actor: Combatant, cap: int, dist: int, profile: AiProfile) -> bool:
	var basic: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor)
	if dist >= basic.x and dist <= basic.y:
		return true
	if profile.role != AiProfile.Role.SUPPORT:
		return false
	for skill: SkillDef in SkillCatalog.single_attacks(actor, cap):
		var reach_band: Vector2i = skill.reach_for(actor)
		if skill.dash_tiles <= 0 and dist >= reach_band.x and dist <= reach_band.y:
			return true
	return false

# Casts a Focus-type buff and then attacks, trying the buffs best-first; a buff whose cost
# would leave no strike is taken back out (so the buff never wastes the turn).
static func _buff_then_engage(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool:
	var buffs: Array[SkillDef] = SkillCatalog.available(SkillDef.Kind.FOCUS, cap, actor)
	buffs.reverse()   # highest level first
	for buff: SkillDef in buffs:
		var mark: int = plan.steps_of(actor).size()
		if not plan.add_step(actor, PlanStep.self_skill(buff), false).is_empty():
			continue
		if _engage_any(plan, actor, ranked, cap, profile):
			return true
		plan.remove_step(actor, mark)
	return false

# SUPPORT: heals the most wounded friend (at or below profile.heal_below) with an
# ally-target healing skill, highest level first. The caster itself is a candidate
# when the skill allows self-targeting.
static func _try_heal(plan: CombatPlan, actor: Combatant, profile: AiProfile, cap: int, already: Array[Combatant]) -> Combatant:
	var skills: Array[SkillDef] = []
	for s: SkillDef in SkillCatalog.available(SkillDef.Kind.SUPPORT, cap, actor):
		if s.heals():
			skills.append(s)
	if skills.is_empty():
		return null
	skills.reverse()
	var wounded: Array[Combatant] = []
	for friend: Combatant in plan.context.friends_of(actor):
		if already.has(friend) or not friend.is_active() or not plan.context.grid.has_combatant(friend) or friend.hp >= friend.max_hp:
			continue
		if float(friend.hp) / float(maxi(1, friend.max_hp)) <= profile.heal_below:
			wounded.append(friend)
	wounded.sort_custom(func(a: Combatant, b: Combatant) -> bool:
		var ra: float = float(a.hp) / float(maxi(1, a.max_hp))
		var rb: float = float(b.hp) / float(maxi(1, b.max_hp))
		return ra < rb if not is_equal_approx(ra, rb) else a.hp < b.hp
	)
	for ally: Combatant in wounded:
		for skill: SkillDef in skills:
			var step: PlanStep = null
			if ally == actor:
				if skill.target_mode != SkillDef.TargetMode.ALLY:
					step = PlanStep.self_skill(skill)
			elif skill.targets_ally():
				step = PlanStep.ally(skill, ally)
			if step != null and plan.add_step(actor, step, false).is_empty():
				return ally
	return null

# Ranged units: step back to the tile at maximum range from the target (or the farthest
# tile of the band within one move that keeps clear of other foes), then strike from
# there. The step is dropped again if no strike fits afterwards, so kiting never wastes the turn.
static func _kite_then_engage(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, foes: Array[Combatant], ranked: Array[Combatant], band: Vector2i, cap: int, profile: AiProfile) -> bool:
	var grid: CombatGrid = plan.context.grid
	var target: Combatant = ranked[0]
	var target_pos: Vector2i = grid.get_position(target)
	var blocked: Dictionary = plan.blocked_cells(actor)
	var limit: int = mini(plan.max_move_tiles(actor), maxi(0, st.ap_left))
	if limit < 1 or st.rooted or st.acted:
		return false
	var cur_near: int = _nearest_foe(grid, st.pos, foes)
	for want: int in range(band.y, band.x - 1, -1):
		if want <= grid.get_distance(st.pos, target_pos):
			break   # a spot at or below the current distance isn't kiting
		var goals: Dictionary = {}
		for dx: int in range(-want, want + 1):
			var dy: int = want - absi(dx)
			for sign_y: int in ([1, -1] if dy != 0 else [1]):
				var cell: Vector2i = target_pos + Vector2i(dx, dy * sign_y)
				if cell == st.pos or not grid.is_valid_coord(cell) or not grid.is_walkable(cell) or grid.is_exit_tile(cell) or blocked.has(cell):
					continue
				if _nearest_foe(grid, cell, foes) < maxi(band.x, mini(cur_near, band.y)):
					continue   # not away from the others
				goals[cell] = true
		if goals.is_empty():
			continue
		var path: Array[Vector2i] = GridPath.find_path(grid, st.pos, goals, blocked, false)
		if path.is_empty() or path.size() > limit:
			continue
		var dest: Vector2i = st.pos
		for d: Vector2i in path:
			dest += d
		var mark: int = plan.steps_of(actor).size()
		var straight: bool = true
		for d: Vector2i in path:
			straight = straight and d == path[0]
		var move_skill: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
		var kite_step: PlanStep = PlanStep.move(move_skill, path[0], path.size()) if straight else PlanStep.go_to(move_skill, dest)
		if not plan.add_step(actor, kite_step, false).is_empty():
			continue
		if _engage_any(plan, actor, ranked, cap, profile):
			return true
		plan.remove_step(actor, mark)
	return false

# Walks as far as the AP and the move allow along the route to the nearest reachable target.
static func _advance_toward(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, ranked: Array[Combatant]) -> bool:
	var tiles: int = mini(plan.max_move_tiles(actor), maxi(0, st.ap_left))
	if tiles < 1 or st.rooted or st.acted:
		return false
	for candidate: Combatant in ranked:
		var route: Array[Vector2i] = plan.approach_path(actor, st.pos, candidate)
		if route.is_empty():
			continue
		var dest: Vector2i = st.pos
		for i: int in range(mini(route.size(), tiles)):
			dest += route[i]
		if plan.add_step(actor, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), dest), false).is_empty():
			return true
	return false

# Foes ordered best-first: a taunting foe the unit can reach comes first; then aggressors
# prefer the nearest, everyone else the weakest (reckless units always the nearest);
# targets that are far or hard to reach are penalised by the route they would need.
static func _rank_targets(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, foes: Array[Combatant], grid: CombatGrid, profile: AiProfile) -> Array[Combatant]:
	var scored: Array[Dictionary] = []
	var nearest_first: bool = profile.reckless or profile.target_pref == AiProfile.TargetPref.NEAREST
	var attack_reach: int = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor).y
	for f: Combatant in foes:
		if not f.is_active() or not grid.has_combatant(f) or DictUtil.int_of(st.est_hp, f) <= 0:
			continue
		var d: int = grid.get_distance(st.pos, grid.get_position(f))
		var hp_left: float = float(DictUtil.int_of(st.est_hp, f))
		var score: float = float(d) * 100.0 + hp_left if nearest_first else hp_left + float(d) * 4.0
		var reachable: bool = d <= attack_reach
		if d > attack_reach:
			var route: Array[Vector2i] = plan.approach_path(actor, st.pos, f)
			reachable = not route.is_empty()
			score += 400.0 if route.is_empty() else float(route.size()) * 8.0
		if f.has_status(StatusEffects.TAUNT) and reachable:
			score -= 100000.0   # forced: a reachable taunter outranks every other choice
		scored.append({"foe": f, "score": score})
	scored.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return DictUtil.to_float(a["score"]) < DictUtil.to_float(b["score"]))
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
