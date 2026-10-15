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
	var smart: bool = profile.smart_skills
	var urgent_cleanse_tried: bool = false   # AI1: at most one cleanse per round (each attempt is made once)
	var late_cleanse_tried: bool = false
	var disengage_tried: bool = false
	var precast_done: bool = false
	var riposte_tried: bool = false
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

		# AI1: a cleanse-only skill goes first when an ally is locked down (stun / root).
		if smart and not urgent_cleanse_tried:
			urgent_cleanse_tried = true
			if _try_cleanse(plan, actor, cap, true):
				late_cleanse_tried = true
				continue

		if profile.role == AiProfile.Role.SUPPORT and healed.size() < MAX_HEALS:
			var patient: Combatant = _try_heal(plan, actor, profile, cap, healed)
			if patient != null:
				healed.append(patient)
				continue

		if smart and not late_cleanse_tried:
			late_cleanse_tried = true
			if _try_cleanse(plan, actor, cap, false):
				continue

		var ranked: Array[Combatant] = _rank_targets(plan, actor, st, foes, grid, profile)
		if ranked.is_empty():
			break
		var target: Combatant = ranked[0]
		var dist: int = grid.get_distance(st.pos, grid.get_position(target))
		var low_hp: bool = hp_ratio <= profile.defend_below and not profile.reckless

		# AI1: a disengage must be the unit's first action (it is the round's move), so it is tried
		# before any buff: ranged / support units break melee contact, a hurt skirmisher falls back.
		if smart and not disengage_tried and not profile.reckless:
			disengage_tried = true
			var adjacent: bool = _nearest_foe(grid, st.pos, foes) <= 1
			var ranged_role: bool = profile.role == AiProfile.Role.SNIPER or profile.role == AiProfile.Role.SUPPORT
			if adjacent and ranged_role and _try_disengage(plan, actor, cap, profile, ranked, true):
				continue
			if profile.role == AiProfile.Role.SKIRMISHER and hp_ratio <= profile.defend_below and dist <= 1 and _try_disengage(plan, actor, cap, profile, ranked, false):
				continue

		# AI1: once per round, taunt / ally buff / debuff attack before the strike (each only if the strike still fits).
		if smart and not precast_done:
			precast_done = true
			if _precast(plan, actor, ranked, cap, profile):
				continue

		# Buff (Focus) first only when the strike still fits afterwards: the buff is an
		# action, so a unit that has to walk to its target must not cast before moving.
		if not focused and (profile.role == AiProfile.Role.SUPPORT or actor.attack_range > 1):
			focused = true
			if _can_strike_in_place(actor, cap, dist, profile) and _buff_then_engage(plan, actor, ranked, cap, profile):
				continue

		if low_hp and not guarded and dist <= reach + 1:
			guarded = true
			if smart:
				if _guard_smart(plan, actor, cap, profile, foes, st.pos):
					continue
			elif _try_skills(plan, actor, SkillDef.Kind.DEFEND, cap, profile, Callable()):
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
			# AI1: close the round in a riposte stance (a later strike would cancel it, so planning stops there).
			if smart and not riposte_tried:
				riposte_tried = true
				if _riposte_finisher(plan, actor, cap, profile, foes, hp_ratio):
					break
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
static func _try_skills(plan: CombatPlan, actor: Combatant, kind: SkillDef.Kind, cap: int, profile: AiProfile, make_step: Callable, filter: Callable = Callable()) -> bool:
	# Attacks: single-target ones (area attacks go through _try_area; ranged units add the off-hand jab).
	var skills: Array[SkillDef] = _strikes(actor, cap, profile) if kind == SkillDef.Kind.ATTACK else SkillCatalog.available(kind, cap, actor)
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
		if filter.is_valid() and not DictUtil.to_bool(filter.call(skill)):
			continue
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
	if profile.smart_skills and _try_area(plan, actor, cap, profile):
		return true
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
	for skill: SkillDef in _strikes(actor, cap, profile):
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
		if already.has(friend) or not friend.is_active() or not plan.context.grid.has_combatant(friend):
			continue
		if _needs_heal(friend, profile) or (profile.smart_skills and _locked_down(friend)):
			wounded.append(friend)
	wounded.sort_custom(func(a: Combatant, b: Combatant) -> bool:
		var ra: float = float(a.hp) / float(maxi(1, a.max_hp))
		var rb: float = float(b.hp) / float(maxi(1, b.max_hp))
		return ra < rb if not is_equal_approx(ra, rb) else a.hp < b.hp
	)
	for ally: Combatant in wounded:
		var hurt: bool = _needs_heal(ally, profile)
		for skill: SkillDef in skills:
			if not hurt and not _frees_lock(ally, skill):
				continue   # only above heal_below because it is stunned / rooted: needs a heal that also cleanses that
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

# ------------------------------------------------------------------------------
# AI1: smart skill use (all of it off when profile.smart_skills is false)
# ------------------------------------------------------------------------------

# Single-target attacks the unit may plan as "the strike"; with smart skills on, zero-power
# (debuff-only) attacks are left out: they are cast by _precast, never as the damage dealer.
static func _strikes(actor: Combatant, cap: int, profile: AiProfile) -> Array[SkillDef]:
	var list: Array[SkillDef] = SkillCatalog.single_attacks(actor, cap)
	if not profile.smart_skills:
		return list
	var out: Array[SkillDef] = []
	for skill: SkillDef in list:
		if skill.deals_damage():
			out.append(skill)
	return out

# Highest level first (the AI's tier choice); prefer_level still overrides.
static func _ordered(skills: Array[SkillDef], profile: AiProfile) -> Array[SkillDef]:
	var out: Array[SkillDef] = skills.duplicate()
	out.reverse()
	if profile.prefer_level > 0:
		out.sort_custom(func(a: SkillDef, b: SkillDef) -> bool:
			return absi(a.level - profile.prefer_level) < absi(b.level - profile.prefer_level)
		)
	return out

# Optional extras (taunt, buffs, debuffs, area attacks, the riposte finisher) respect a low
# prefer_level (the economy preset): skills above it are not used for them.
static func _optional(skills: Array[SkillDef], profile: AiProfile) -> Array[SkillDef]:
	if profile.prefer_level <= 0:
		return skills
	var out: Array[SkillDef] = []
	for skill: SkillDef in skills:
		if skill.level <= profile.prefer_level:
			out.append(skill)
	return out

static func _needs_heal(friend: Combatant, profile: AiProfile) -> bool:
	return friend.hp < friend.max_hp and float(friend.hp) / float(maxi(1, friend.max_hp)) <= profile.heal_below

# Stunned or rooted (active or pending): cannot act normally.
static func _locked_down(c: Combatant) -> bool:
	return StatusEffects.has_any(c, StatusEffects.STUN) or StatusEffects.has_any(c, StatusEffects.ROOT)

static func _frees_lock(c: Combatant, skill: SkillDef) -> bool:
	var list: Array[String] = SupportRules.cleansable(c, skill)
	return list.has(StatusEffects.STUN) or list.has(StatusEffects.ROOT)

static func _nearest_foe_unit(grid: CombatGrid, pos: Vector2i, foes: Array[Combatant]) -> Combatant:
	var best: Combatant = null
	var best_dist: int = 99
	for f: Combatant in foes:
		if f.is_active() and grid.has_combatant(f):
			var d: int = grid.get_distance(pos, grid.get_position(f))
			if d < best_dist:
				best_dist = d
				best = f
	return best

# Disengage (the round's first action). With `require_strike` a strike must still fit afterwards,
# otherwise the jump is taken back out (the caller falls back to the old kite).
static func _try_disengage(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile, ranked: Array[Combatant], require_strike: bool) -> bool:
	for skill: SkillDef in _ordered(SkillCatalog.available(SkillDef.Kind.DISENGAGE, cap, actor), profile):
		var mark: int = plan.steps_of(actor).size()
		# The planner's own landing pick first (the farthest tile from the threat).
		if plan.add_step(actor, PlanStep.self_skill(skill), false).is_empty():
			if not require_strike or _engage_any(plan, actor, ranked, cap, profile):
				return true
			plan.remove_step(actor, mark)
		if not require_strike or ranked.is_empty():
			continue
		# The farthest tile may be out of weapon range: try landings that keep the target in reach.
		for cell: Vector2i in _landing_cells(plan, actor, skill, ranked[0]):
			var jump: PlanStep = PlanStep.self_skill(skill)
			jump.cell = cell
			if not plan.add_step(actor, jump, false).is_empty():
				continue
			if _engage_any(plan, actor, ranked, cap, profile):
				return true
			plan.remove_step(actor, mark)
	return false

# Up to four jump landings farther from `target` than the unit stands but still inside its basic
# attack band, farthest first, then the shortest jump, then top-left.
static func _landing_cells(plan: CombatPlan, actor: Combatant, skill: SkillDef, target: Combatant) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	var st: CombatPlan.SimState = plan.compile(actor)
	var grid: CombatGrid = plan.context.grid
	var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor)
	var target_pos: Vector2i = grid.get_position(target)
	var here: int = grid.get_distance(st.pos, target_pos)
	var blocked: Dictionary = plan.blocked_cells(actor)
	var reachable: Dictionary = plan.jump_tiles_from(st.pos, skill.jump_tiles)
	var scored: Array[Dictionary] = []
	for key: Variant in reachable:
		var cell: Vector2i = key
		var d: int = grid.get_distance(cell, target_pos)
		if blocked.has(cell) or d <= here or d < band.x or d > band.y:
			continue
		scored.append({"cell": cell, "d": d, "steps": DictUtil.int_of(reachable, cell)})
	scored.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if DictUtil.to_int(a["d"]) != DictUtil.to_int(b["d"]):
			return DictUtil.to_int(a["d"]) > DictUtil.to_int(b["d"])
		if DictUtil.to_int(a["steps"]) != DictUtil.to_int(b["steps"]):
			return DictUtil.to_int(a["steps"]) < DictUtil.to_int(b["steps"])
		var ca: Vector2i = a["cell"]
		var cb: Vector2i = b["cell"]
		return ca.y < cb.y if ca.y != cb.y else ca.x < cb.x
	)
	for entry: Dictionary in scored:
		if out.size() >= 4:
			break
		var chosen: Vector2i = entry["cell"]
		out.append(chosen)
	return out

# Cleanse-only SUPPORT skill, at most once per round. `urgent`: only for a target whose stun / root
# the skill can remove. Target: most cleansable debuffs, then lowest HP ratio. Tier: the lowest that
# covers everything cleansable on the target, else the highest that fits.
static func _try_cleanse(plan: CombatPlan, actor: Combatant, cap: int, urgent: bool) -> bool:
	var skills: Array[SkillDef] = SkillCatalog.cleanse_only(cap, actor)
	if skills.is_empty():
		return false
	var grid: CombatGrid = plan.context.grid
	var friends: Array[Combatant] = plan.context.friends_of(actor)
	var scored: Array[Dictionary] = []
	var index: int = 0
	for friend: Combatant in friends:
		index += 1
		if not friend.is_active() or not grid.has_combatant(friend):
			continue
		var needed: Array[String] = []
		for skill: SkillDef in skills:
			if not SupportRules.target_problem(actor, skill, friend, friends).is_empty():
				continue
			for status: String in SupportRules.cleansable(friend, skill):
				if not needed.has(status):
					needed.append(status)
		if needed.is_empty() or (urgent and not (needed.has(StatusEffects.STUN) or needed.has(StatusEffects.ROOT))):
			continue
		scored.append({"unit": friend, "n": needed.size(), "ratio": float(friend.hp) / float(maxi(1, friend.max_hp)), "idx": index})
	scored.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if DictUtil.to_int(a["n"]) != DictUtil.to_int(b["n"]):
			return DictUtil.to_int(a["n"]) > DictUtil.to_int(b["n"])
		if not is_equal_approx(DictUtil.to_float(a["ratio"]), DictUtil.to_float(b["ratio"])):
			return DictUtil.to_float(a["ratio"]) < DictUtil.to_float(b["ratio"])
		return DictUtil.to_int(a["idx"]) < DictUtil.to_int(b["idx"])
	)
	for entry: Dictionary in scored:
		var patient: Combatant = entry["unit"]
		var total: int = DictUtil.to_int(entry["n"])
		var eligible: Array[SkillDef] = []
		for skill: SkillDef in skills:
			if SupportRules.target_problem(actor, skill, patient, friends).is_empty() and not SupportRules.cleansable(patient, skill).is_empty():
				eligible.append(skill)
		eligible.sort_custom(func(a: SkillDef, b: SkillDef) -> bool:
			if a.tier != b.tier:
				return a.tier < b.tier
			if a.level != b.level:
				return a.level < b.level
			return a.id < b.id
		)
		var order: Array[SkillDef] = []
		var others: Array[SkillDef] = []
		for skill: SkillDef in eligible:
			if SupportRules.cleansable(patient, skill).size() >= total:
				order.append(skill)
			else:
				others.append(skill)
		others.reverse()
		order.append_array(others)
		for skill: SkillDef in order:
			var step: PlanStep = PlanStep.self_skill(skill) if patient == actor else PlanStep.ally(skill, patient)
			if plan.add_step(actor, step, false).is_empty():
				return true
	return false

# Casts that come before the strike, each tried once per round and kept only when a strike still
# fits afterwards: taunt, an ally buff (haste), a debuff attack (intimidate). The trial strike is
# taken out again; the normal flow plans the real one.
static func _precast(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool:
	var added: bool = false
	for kind: int in range(3):
		var options: Array[PlanStep]
		match kind:
			0:
				options = _taunt_options(plan, actor, cap, profile)
			1:
				options = _buff_options(plan, actor, cap, profile)
			_:
				options = _debuff_options(plan, actor, cap, profile)
		for step: PlanStep in options:
			var mark: int = plan.steps_of(actor).size()
			if not plan.add_step(actor, step, false).is_empty():
				continue
			if _engage_any(plan, actor, ranked, cap, profile):
				plan.remove_step(actor, mark + 1)
				added = true
				break
			plan.remove_step(actor, mark)
	return added

# Taunt: when an ally (not the caster) stands within 3 tiles of a foe and the caster is not taunting.
static func _taunt_options(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile) -> Array[PlanStep]:
	var out: Array[PlanStep] = []
	if StatusEffects.has_any(actor, StatusEffects.TAUNT):
		return out
	var grid: CombatGrid = plan.context.grid
	var foes: Array[Combatant] = plan.context.foes_of(actor)
	var exposed: bool = false
	for ally: Combatant in plan.context.friends_of(actor):
		if ally != actor and ally.is_active() and grid.has_combatant(ally) and _nearest_foe(grid, plan.planned_position(ally), foes) <= 3:
			exposed = true
	if not exposed:
		return out
	for skill: SkillDef in _ordered(_optional(SkillCatalog.granting(SkillDef.Kind.SUPPORT, StatusEffects.TAUNT, cap, actor), profile), profile):
		if skill.target_mode != SkillDef.TargetMode.ALLY:
			out.append(PlanStep.self_skill(skill))
	return out

# Ally buffs (haste...): the ally without the status that is farthest from the foes, but beyond
# its attack reach + 1 (a buff on someone about to fight is wasted); the caster itself when allowed.
static func _buff_options(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile) -> Array[PlanStep]:
	var out: Array[PlanStep] = []
	var grid: CombatGrid = plan.context.grid
	var foes: Array[Combatant] = plan.context.foes_of(actor)
	for skill: SkillDef in _ordered(_optional(SkillCatalog.ally_buffs(StatusEffects.TAUNT, cap, actor), profile), profile):
		var cands: Array[Dictionary] = []
		var index: int = 0
		for friend: Combatant in plan.context.friends_of(actor):
			index += 1
			if not friend.is_active() or not grid.has_combatant(friend):
				continue
			if (friend == actor and skill.target_mode == SkillDef.TargetMode.ALLY) or (friend != actor and skill.target_mode == SkillDef.TargetMode.SELF):
				continue
			if StatusEffects.has_any(friend, skill.grant_status):
				continue
			var d: int = _nearest_foe(grid, plan.planned_position(friend), foes)
			if d <= SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(friend).y + 1:
				continue
			cands.append({"unit": friend, "d": d, "idx": index})
		cands.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
			if DictUtil.to_int(a["d"]) != DictUtil.to_int(b["d"]):
				return DictUtil.to_int(a["d"]) > DictUtil.to_int(b["d"])
			return DictUtil.to_int(a["idx"]) < DictUtil.to_int(b["idx"])
		)
		for entry: Dictionary in cands:
			var target: Combatant = entry["unit"]
			out.append(PlanStep.self_skill(skill) if target == actor else PlanStep.ally(skill, target))
	return out

# Debuff attacks (zero-power, inflicts a status): the foe with the highest attack power that does
# not already carry (or await) the status.
static func _debuff_options(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile) -> Array[PlanStep]:
	var out: Array[PlanStep] = []
	var skills: Array[SkillDef] = _ordered(_optional(SkillCatalog.debuff_attacks(cap, actor), profile), profile)
	if skills.is_empty():
		return out
	var grid: CombatGrid = plan.context.grid
	var st: CombatPlan.SimState = plan.compile(actor)
	var scored: Array[Dictionary] = []
	var index: int = 0
	for foe: Combatant in plan.context.foes_of(actor):
		index += 1
		if not foe.is_active() or not grid.has_combatant(foe) or DictUtil.int_of(st.est_hp, foe) <= 0:
			continue
		scored.append({"unit": foe, "power": DamageCalculator.power_of(foe, foe.attack_range > 1), "idx": index})
	scored.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if DictUtil.to_int(a["power"]) != DictUtil.to_int(b["power"]):
			return DictUtil.to_int(a["power"]) > DictUtil.to_int(b["power"])
		return DictUtil.to_int(a["idx"]) < DictUtil.to_int(b["idx"])
	)
	for skill: SkillDef in skills:
		for entry: Dictionary in scored:
			var foe: Combatant = entry["unit"]
			if not StatusEffects.has_any(foe, skill.status_effect):
				out.append(PlanStep.engage(skill, foe))
	return out

# Low-HP guard: against a melee foe prefer a riposte stance over a plain guard, against a ranged
# foe (or while already riposting) the plain guard first.
static func _guard_smart(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile, foes: Array[Combatant], pos: Vector2i) -> bool:
	var near: Combatant = _nearest_foe_unit(plan.context.grid, pos, foes)
	var melee: bool = near != null and near.attack_range <= 1
	var holding: bool = StatusEffects.has_any(actor, StatusEffects.RIPOSTE)
	var riposte_only: Callable = func(s: SkillDef) -> bool: return s.grant_status == StatusEffects.RIPOSTE
	var plain_only: Callable = func(s: SkillDef) -> bool: return s.grant_status != StatusEffects.RIPOSTE
	var order: Array[Callable] = []
	if holding:
		order = [plain_only]
	elif melee:
		order = [riposte_only, plain_only]
	else:
		order = [plain_only, riposte_only]
	for filter: Callable in order:
		if _try_skills(plan, actor, SkillDef.Kind.DEFEND, cap, profile, Callable(), filter):
			return true
	return false

# After the first strike: end the round in a riposte stance when the unit is healthy enough not to
# be guarding, is not reckless, and a melee foe stands next to it. Returns true when it was added.
static func _riposte_finisher(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile, foes: Array[Combatant], hp_ratio: float) -> bool:
	if profile.reckless or profile.defend_below <= 0.0 or hp_ratio <= profile.defend_below or StatusEffects.has_any(actor, StatusEffects.RIPOSTE):   # defend_below 0 = never guards (all-out preset)
		return false
	var st: CombatPlan.SimState = plan.compile(actor)
	var grid: CombatGrid = plan.context.grid
	var near: Combatant = _nearest_foe_unit(grid, st.pos, foes)
	if st.withdrawn or near == null or near.attack_range > 1 or grid.get_distance(st.pos, grid.get_position(near)) != 1:
		return false
	for skill: SkillDef in _ordered(_optional(SkillCatalog.granting(SkillDef.Kind.DEFEND, StatusEffects.RIPOSTE, cap, actor), profile), profile):
		if plan.add_step(actor, PlanStep.self_skill(skill), false).is_empty():
			return true
	return false

# Area attack: chosen when it hits at least profile.area_threshold() foes and no ally (planned
# positions, the shape code of SkillDef.area_cells that the planner and AreaAttackCommand use).
# Highest expected damage (power x foes hit) first, then the higher tier.
static func _try_area(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile) -> bool:
	var skills: Array[SkillDef] = []
	for skill: SkillDef in _optional(SkillCatalog.area_attacks(cap, actor), profile):
		if skill.deals_damage():
			skills.append(skill)
	if skills.is_empty():
		return false
	var st: CombatPlan.SimState = plan.compile(actor)
	if st.withdrawn or st.pos == CombatPlan.NO_CELL:
		return false
	var grid: CombatGrid = plan.context.grid
	var foes: Array[Combatant] = plan.context.foes_of(actor)
	var friends: Array[Combatant] = plan.context.friends_of(actor)
	var cands: Array[Dictionary] = []
	for skill: SkillDef in skills:
		var band: Vector2i = skill.reach_for(actor)
		for aim: Vector2i in _aim_cells(grid, st.pos, skill, band):
			var cells: Array[Vector2i] = []
			for cell: Vector2i in skill.area_cells(st.pos, aim):
				if grid.is_valid_coord(cell) and cell != st.pos:
					cells.append(cell)
			var foes_hit: int = 0
			for foe: Combatant in foes:
				if foe.is_active() and grid.has_combatant(foe) and DictUtil.int_of(st.est_hp, foe) > 0 and cells.has(grid.get_position(foe)):
					foes_hit += 1
			var allies_hit: int = 0
			for friend: Combatant in friends:
				if friend != actor and friend.is_active() and cells.has(plan.planned_position(friend)):
					allies_hit += 1
			if allies_hit > 0 or foes_hit < profile.area_threshold():
				continue
			cands.append({"skill": skill, "aim": aim, "dmg": skill.power_for(actor) * float(foes_hit)})
	cands.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if not is_equal_approx(DictUtil.to_float(a["dmg"]), DictUtil.to_float(b["dmg"])):
			return DictUtil.to_float(a["dmg"]) > DictUtil.to_float(b["dmg"])
		var sa: SkillDef = a["skill"]
		var sb: SkillDef = b["skill"]
		if sa.level != sb.level:
			return sa.level > sb.level
		var aa: Vector2i = a["aim"]
		var ab: Vector2i = b["aim"]
		return aa.y < ab.y if aa.y != ab.y else aa.x < ab.x
	)
	for cand: Dictionary in cands:
		var chosen: SkillDef = cand["skill"]
		var aim_cell: Vector2i = cand["aim"]
		if plan.add_step(actor, PlanStep.area_at(chosen, aim_cell), false).is_empty():
			return true
	return false

# Tiles an area skill can be aimed at from `pos`: SWEEP the four diagonal neighbours, LINE the
# four neighbours, PLUS / BURST every tile inside the skill's reach band.
static func _aim_cells(grid: CombatGrid, pos: Vector2i, skill: SkillDef, band: Vector2i) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	if skill.area == SkillDef.Area.SWEEP:
		for d: Vector2i in [Vector2i(1, 1), Vector2i(1, -1), Vector2i(-1, 1), Vector2i(-1, -1)]:
			if grid.is_valid_coord(pos + d):
				out.append(pos + d)
		return out
	if skill.area == SkillDef.Area.LINE:
		for d: Vector2i in CombatPlan.DIRS:
			if grid.is_valid_coord(pos + d):
				out.append(pos + d)
		return out
	for dx: int in range(-band.y, band.y + 1):
		for dy: int in range(-band.y, band.y + 1):
			var cell: Vector2i = pos + Vector2i(dx, dy)
			var d: int = grid.get_distance(pos, cell)
			if grid.is_valid_coord(cell) and d >= band.x and d <= band.y:
				out.append(cell)
	return out
