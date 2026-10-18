---
class: TacticalAI
extends: RefCounted
description: "Tactical RPG - Tactical AI (round planner for computer-controlled units) Drafts one unit's whole round as a PlanStep sequence from the state at round start, growing a scratch CombatPlan one step at a time so every step is validated (stance flow, AP, blocks, paths) exactly like a player's input. 1. Withdraw: at low HP, leave through an exit (never for a reckless unit). 2. SUPPORT role: heal the most wounded ally (at or below profile.heal_below). 3. Pick a target: a reachable taunter first, else aggressors the nearest, others the weakest (reckless units always the nearest). 4. Role pattern: guard / focus (only if it does not waste the turn) / kite to maximum range (ranged) / Engage (auto-moves + strike). 5. No idle turns: when no target can be struck this round the unit still advances along its route; it only rests when AP is the problem. Terrain (S12T-L rule 8): AI routes pay AI_HAZARD_PENALTY extra per hazard tile entered, and ranged units prefer, among equally ranked firing / kiting cells, the one with the most cover against the nearest ranged foe. Covered targets rank lower through the planner's estimate (it includes cover). Combos (S12C-L rule 6): the strike order puts a stance-break skill first against a Defensive or riposting foe, and an exploit skill first only when its status is active on the target (else after the plain strikes). BAL3-AI levers (docs/specs/BAL3.md section 4), all on incoming_damage (the damage the foes that would pick a unit are expected to deal it by a given block): B: Focus goes before a cast only while the damage expected before that cast resolves is below FOCUS_PRESSURE_RATIO of the unit's current HP (else the cast goes first). C: a unit expected to take at least its current HP this round takes the highest-damage option that resolves by FAST_BLOCK (a lower tier or the plain strike) over a slower higher tier (_fast_strike); the choice is by damage, so under pressure it may override the combo order above. Heal reservation: a SUPPORT unit whose draft has no heal keeps one heal (its lowest heal tier) open at the end of the round when a friend is projected at or below heal_below by the round's end (_reserve_heal). D taunt pull: a unit whose top target is a taunter it can strike or reach within its one move this round commits to it: the taunter is its only target (it walks to it when the strike does not fit), and two steps that would cost the move are skipped: a hurt skirmisher's disengage, and the low-HP guard while the taunter is out of reach. BAL5 levers (docs/specs/BAL5_LEVERS.md sections 2-4; symmetric, enemies use them too): T2 early taunt: taunt also goes up when any foe would pick an ally this round (_likely_target), not only when an ally stands within 3 tiles of a foe; when it cannot go up in place before a strike, the unit walks toward the nearest such diver and taunts after the walk (_early_taunt_walk). S1 healer self-care: a SUPPORT unit expecting at least SELF_CARE_RATIO of its current HP as damage this round heals itself first (lowest heal tier that fits), else steps back one tile (_self_care). A2 friendly-fire tolerance: an area cast may hit one ally when it hits AREA_FF_MIN_FOES foes and costs that ally less than AREA_FF_HP_RATIO of its current HP (_friendly_fire_ok). Targets may move, so plans can fizzle: that is the planning-phase gamble. Used for enemies each round and for the player's AI presets (the player's own planner is never forced: taunt only steers TacticalAI)."
---

# Properties
* `MAX_STEPS`: int = 16 (const)
* `MAX_HEALS`: int = 2 (const)
* `AI_HAZARD_PENALTY`: int = 3   # PLACEHOLDER (S12T-L): extra route cost per hazard tile an AI move enters (const)
* `FOCUS_PRESSURE_RATIO`: float = 0.5   # BAL3-AI lever B: Focus first only while the pre-cast incoming damage is below this share of HP (const)
* `FAST_BLOCK`: int = 2                 # BAL3-AI lever C: a unit likely to die prefers options that resolve by this block (const)
* `ITEM_HEAL_RATIO`: float = 0.35       # ITEMUSE1 rule 9: drink a potion when the round would leave it at or below this HP share (const)
* `SELF_CARE_RATIO`: float = 0.5        # BAL5 lever S1: a SUPPORT unit expecting at least this share of its current HP as damage this round cares for itself first (const)
* `SELF_CARE_NONE`: int = 0             # _self_care results (const)
* `SELF_CARE_HEALED`: int = 1 (const)
* `SELF_CARE_RETREATED`: int = 2 (const)
* `AREA_FF_MIN_FOES`: int = 3          # BAL5 lever A2: an area cast may hit one ally only when it hits at least this many foes (const)
* `AREA_FF_HP_RATIO`: float = 0.25      # BAL5 lever A2: ... and the expected damage on that ally is below this share of its current HP (const)
* `plan`: CombatPlan = shared if shared != null else CombatPlan.new(ctx)
* `grid`: CombatGrid = ctx.grid
* `foes`: Array[Combatant] = ctx.foes_of(actor)
* `prefer`: Callable = cover_preference(grid, foes)
* `cap`: int = ctx.skill_cap_of(actor)
* `guarded`: bool = false
* `focused`: bool = false
* `fled`: bool = false
* `healed`: Array[Combatant] = []   # friends already healed this round (the plan doesn't track their HP)
* `rests`: int = 0
* `urgent_cleanse_tried`: bool = false   # AI1: at most one cleanse per round (each attempt is made once)
* `late_cleanse_tried`: bool = false
* `disengage_tried`: bool = false
* `precast_done`: bool = false
* `riposte_tried`: bool = false
* `self_care_tried`: bool = false   # BAL5 lever S1: checked once, at the start of the draft
* `basic`: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
* `band`: Vector2i = basic.reach_for(actor)
* `reach`: int = band.y
* `st`: CombatPlan.SimState = plan.compile(actor)
* `hp_ratio`: float = float(actor.hp) / float(maxi(1, actor.max_hp))
* `care`: int = _self_care(plan, actor, profile, cap, st, foes, reach)
* `patient`: Combatant = _try_heal(plan, actor, profile, cap, healed)
* `ranked`: Array[Combatant] = _rank_targets(plan, actor, st, foes, grid, profile)
* `pulled`: Combatant = _taunt_pull(plan, actor, st, ranked)
* `only_taunter`: Array[Combatant] = [pulled]
* `target`: Combatant = ranked[0]
* `dist`: int = grid.get_distance(st.pos, grid.get_position(target))
* `low_hp`: bool = hp_ratio <= profile.defend_below and not profile.reckless
* `adjacent`: bool = _nearest_foe(grid, st.pos, foes) <= 1
* `ranged_role`: bool = profile.role == AiProfile.Role.SNIPER or profile.role == AiProfile.Role.SUPPORT
* `back`: Vector2i = _retreat_dir(plan, actor, st.pos, foes, reach, grid.get_position(target))
* `skills`: Array[SkillDef] = _strikes(actor, cap) if kind == SkillDef.Kind.ATTACK else SkillCatalog.available(kind, cap, actor)
* `ordered`: Array[SkillDef] = []
* `guarded`: bool = target.current_stance == CombatTypes.StanceType.DEFENSIVE or StatusEffects.has_any(target, StatusEffects.RIPOSTE) or _plans_riposte(plan, target)
* `first`: Array[SkillDef] = []
* `middle`: Array[SkillDef] = []
* `last`: Array[SkillDef] = []
* `breaks`: bool = guarded and skill.disrupt == SkillDef.Disrupt.STANCE_BREAK
* `exploit_applies`: bool = not str(SkillDef.exploit_match(skill.exploit, target)["status"]).is_empty()
* `scored`: Array[Dictionary] = []
* `nearest_first`: bool = profile.reckless or profile.target_pref == AiProfile.TargetPref.NEAREST
* `attack_reach`: int = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor).y
* `d`: int = grid.get_distance(st.pos, grid.get_position(f))
* `hp_left`: float = float(DictUtil.int_of(st.est_hp, f))
* `score`: float = float(d) * 100.0 + hp_left if nearest_first else hp_left + float(d) * 4.0
* `reachable`: bool = d <= attack_reach
* `route`: Array[Vector2i] = plan.approach_path(actor, st.pos, f)
* `result`: Array[Combatant] = []
* `foe`: Combatant = entry["foe"]
* `list`: Array[SkillDef] = SkillCatalog.single_attacks(actor, cap)
* `out`: Array[SkillDef] = []
* `out`: Array[SkillDef] = skills.duplicate()
* `out`: Array[SkillDef] = []
* `out`: Array[Vector2i] = []
* `st`: CombatPlan.SimState = plan.compile(actor)
* `grid`: CombatGrid = plan.context.grid
* `band`: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor)
* `target_pos`: Vector2i = grid.get_position(target)
* `here`: int = grid.get_distance(st.pos, target_pos)
* `blocked`: Dictionary = plan.blocked_cells(actor)
* `reachable`: Dictionary = plan.jump_tiles_from(st.pos, skill.jump_tiles)
* `scored`: Array[Dictionary] = []
* `cell`: Vector2i = key
* `d`: int = grid.get_distance(cell, target_pos)
* `ca`: Vector2i = a["cell"]
* `cb`: Vector2i = b["cell"]
* `chosen`: Vector2i = entry["cell"]
* `grid`: CombatGrid = plan.context.grid
* `where`: Dictionary = plan.planned_positions(plan.context.friends_of(actor))
* `out`: Array[Combatant] = []
* `out`: Array[PlanStep] = []
* `grid`: CombatGrid = plan.context.grid
* `foes`: Array[Combatant] = plan.context.foes_of(actor)
* `where`: Dictionary = plan.planned_positions(plan.context.friends_of(actor))
* `exposed`: bool = false
* `ally_at`: Vector2i = plan.planned_position(ally)
* `out`: Array[PlanStep] = []
* `grid`: CombatGrid = plan.context.grid
* `foes`: Array[Combatant] = plan.context.foes_of(actor)
* `picked`: Array[Combatant] = _fortify_targets(plan, actor, skill) if skill.grant_status == StatusEffects.FORTIFY else _regen_targets(plan, actor, skill, profile)
* `cands`: Array[Dictionary] = []
* `index`: int = 0
* `d`: int = _nearest_foe(grid, plan.planned_position(friend), foes)
* `target`: Combatant = entry["unit"]
* `out`: Array[Combatant] = []
* `grid`: CombatGrid = plan.context.grid
* `foes`: Array[Combatant] = plan.context.foes_of(actor)
* `cands`: Array[Dictionary] = []
* `index`: int = 0
* `pos`: Vector2i = plan.planned_position(friend)
* `adjacent`: int = 0
* `taunting`: int = 1 if StatusEffects.has_any(friend, StatusEffects.TAUNT) else 0
* `out`: Array[Combatant] = []
* `unit`: Combatant = entry["unit"]
* `out`: Array[Combatant] = []
* `ra`: float = float(a.hp) / float(maxi(1, a.max_hp))
* `rb`: float = float(b.hp) / float(maxi(1, b.max_hp))
* `grid`: CombatGrid = plan.context.grid
* `first`: Array[Combatant] = []
* `fragile`: Combatant = _fragile_friend(plan, actor)
* `target_pos`: Vector2i = plan.planned_position(fragile)
* `actor_pos`: Vector2i = grid.get_position(actor)
* `basic`: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
* `threat`: int = plan.max_move_tiles(foe) + (basic.reach_for(foe).y if basic != null else 1)
* `da`: int = grid.get_distance(actor_pos, grid.get_position(a))
* `db`: int = grid.get_distance(actor_pos, grid.get_position(b))
* `out`: Array[Combatant] = first.duplicate()
* `out`: Array[PlanStep] = []
* `skills`: Array[SkillDef] = _ordered(_optional(SkillCatalog.debuff_attacks(cap, actor), profile), profile)
* `grid`: CombatGrid = plan.context.grid
* `st`: CombatPlan.SimState = plan.compile(actor)
* `scored`: Array[Dictionary] = []
* `index`: int = 0
* `by_power`: Array[Combatant] = []
* `unit`: Combatant = entry["unit"]
* `out`: Array[Vector2i] = []
* `cell`: Vector2i = pos + Vector2i(dx, dy)
* `d`: int = grid.get_distance(pos, cell)

# Methods

### `static _try_skills(plan: CombatPlan, actor: Combatant, kind: SkillDef.Kind, cap: int, profile: AiProfile, make_step: Callable, filter: Callable = Callable(), target: Combatant = null) -> bool`
```logic
for skill: SkillDef in _skill_order(plan, actor, kind, cap, profile, target):
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
```

### `static _engage_any(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool`
```logic
if plan.ai_pressure.has(actor) and _fast_strike(plan, actor, ranked, cap, profile):
	return true
if _try_area(plan, actor, cap, profile):
	return true
for candidate: Combatant in ranked:
	var engage: Callable = func(skill: SkillDef) -> PlanStep: return PlanStep.engage(skill, candidate)
	if _try_skills(plan, actor, SkillDef.Kind.ATTACK, cap, profile, engage, Callable(), candidate):
		return true
return false
```

### `static _plans_riposte(plan: CombatPlan, foe: Combatant) -> bool`
```logic
for step: PlanStep in plan.steps_of(foe):
	if step.skill != null and step.skill.kind == SkillDef.Kind.DEFEND and step.skill.grant_status == StatusEffects.RIPOSTE:
		return true
return false
```

### `static _can_strike_in_place(actor: Combatant, cap: int, dist: int, profile: AiProfile) -> bool`
```logic
var basic: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor)
if dist >= basic.x and dist <= basic.y:
	return true
if profile.role != AiProfile.Role.SUPPORT:
	return false
for skill: SkillDef in _strikes(actor, cap):
	var reach_band: Vector2i = skill.reach_for(actor)
	if skill.dash_tiles <= 0 and dist >= reach_band.x and dist <= reach_band.y:
		return true
return false
```

### `static _buff_then_engage(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool`
```logic
var buffs: Array[SkillDef] = SkillCatalog.available(SkillDef.Kind.FOCUS, cap, actor)
buffs.reverse()   # highest level first
for buff: SkillDef in buffs:
	var mark: int = plan.steps_of(actor).size()
	if not plan.add_step(actor, PlanStep.self_skill(buff), false).is_empty():
		continue
	if _engage_any(plan, actor, ranked, cap, profile):
		if _focus_safe(plan, actor):
			return true
		_truncate(plan, actor, mark)   # lever B: under pressure the cast goes first (the normal flow plans it)
		continue
	plan.remove_step(actor, mark)
return false
```

### `static _focus_safe(plan: CombatPlan, actor: Combatant) -> bool`
```logic
var cast: PlanEntry = _last_attack(plan.compile(actor))
if cast == null:
	return true
var incoming: int = incoming_damage(plan, actor, cast.end_block(), cast.pos_before)
return float(incoming) < FOCUS_PRESSURE_RATIO * float(actor.hp)
```

### `static _last_attack(st: CombatPlan.SimState) -> PlanEntry`
```logic
for i: int in range(st.entries.size() - 1, -1, -1):
	var entry: PlanEntry = st.entries[i]
	if entry.skill != null and entry.skill.kind == SkillDef.Kind.ATTACK:
		return entry
return null
```

### `static _truncate(plan: CombatPlan, actor: Combatant, mark: int) -> void`
```logic
var steps: Array[PlanStep] = plan.steps_of(actor)
if steps.size() > mark:
	steps.resize(maxi(0, mark))
	plan.set_steps(actor, steps)
```

### `static _fast_strike(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool`
```logic
var mark: int = plan.steps_of(actor).size()
var best: PlanStep = null
var best_damage: int = -1
if _try_area(plan, actor, cap, profile):
	var area_entry: PlanEntry = _last_attack(plan.compile(actor))
	if area_entry != null and area_entry.end_block() <= FAST_BLOCK:
		best = plan.steps_of(actor)[mark]
		best_damage = area_entry.est_damage
	_truncate(plan, actor, mark)
for candidate: Combatant in ranked:
	var strikable: bool = false
	for skill: SkillDef in _skill_order(plan, actor, SkillDef.Kind.ATTACK, cap, profile, candidate):
		var step: PlanStep = PlanStep.engage(skill, candidate)
		if not plan.add_step(actor, step, false).is_empty():
			continue
		strikable = true
		var entry: PlanEntry = _last_attack(plan.compile(actor))
		if entry != null and entry.end_block() <= FAST_BLOCK and entry.est_damage > best_damage:
			best = step
			best_damage = entry.est_damage
		_truncate(plan, actor, mark)
	if strikable:
		break
if best == null:
	return false
return plan.add_step(actor, best, false).is_empty()
```

### `static _try_item(plan: CombatPlan, actor: Combatant, ctx: PlanContext) -> bool`
```logic
var grid: CombatGrid = ctx.grid
var pos: Vector2i = grid.get_position(actor)
var potion: Consumable = null
var bomb: Consumable = null
for item_id: String in ItemCommand.pouch_ids(actor):
	var con: Consumable = ItemCommand.pouch_item(actor, item_id)
	if not ItemCommand.offered(con) or plan.items_left(actor, item_id) <= 0:
		continue
	if potion == null and con.heal_amount > 0 and not ItemCommand.aims_at_foes(con):
		potion = con
	elif bomb == null and ItemCommand.aims_at_foes(con):
		bomb = con
if potion != null:
	var after: int = actor.hp - incoming_damage(plan, actor, ctx.total_blocks, pos)
	if float(after) <= ITEM_HEAL_RATIO * float(actor.max_hp) and not _heal_planned_on(plan, actor, ctx):
		if plan.add_step(actor, PlanStep.use_item(potion.id, actor), false).is_empty():
			return true
if bomb != null:
	var best: Combatant = null
	for foe: Combatant in ctx.foes_of(actor):
		if not foe.is_active() or not grid.has_combatant(foe) or foe.hp > bomb.damage_amount:
			continue
		if grid.get_distance(pos, grid.get_position(foe)) > bomb.reach():
			continue
		if best == null or foe.hp < best.hp:
			best = foe
	if best != null and plan.add_step(actor, PlanStep.use_item(bomb.id, best), false).is_empty():
		return true
return false
```

### `static _heal_planned_on(plan: CombatPlan, actor: Combatant, ctx: PlanContext) -> bool`
```logic
for friend: Combatant in ctx.friends_of(actor):
	if friend == actor:
		continue
	for e: PlanEntry in plan.entries_for(friend):
		if e.target == actor and ((e.item != null and e.item.heal_amount > 0) or (e.item == null and e.skill != null and e.skill.heals())):
			return true
return false
```

### `static _try_heal(plan: CombatPlan, actor: Combatant, profile: AiProfile, cap: int, already: Array[Combatant]) -> Combatant`
```logic
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
	if _needs_heal(friend, profile) or _locked_down(friend):
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
```

### `static _self_care(plan: CombatPlan, actor: Combatant, profile: AiProfile, cap: int, st: CombatPlan.SimState, foes: Array[Combatant], reach: int) -> int`
```logic
if float(incoming_damage(plan, actor, plan.context.total_blocks, st.pos)) < SELF_CARE_RATIO * float(actor.hp):
	return SELF_CARE_NONE
if actor.hp < actor.max_hp:
	var heals: Array[SkillDef] = []
	for skill: SkillDef in SkillCatalog.available(SkillDef.Kind.SUPPORT, cap, actor):
		if skill.heals() and skill.target_mode != SkillDef.TargetMode.ALLY:
			heals.append(skill)
	heals.sort_custom(func(a: SkillDef, b: SkillDef) -> bool:
		if a.level != b.level:
			return a.level < b.level
		if a.tier != b.tier:
			return a.tier < b.tier
		return a.id < b.id
	)
	for skill: SkillDef in heals:
		if plan.add_step(actor, PlanStep.self_skill(skill), false).is_empty():
			return SELF_CARE_HEALED
var grid: CombatGrid = plan.context.grid
var ranked: Array[Combatant] = _rank_targets(plan, actor, st, foes, grid, profile)
if ranked.is_empty():
	return SELF_CARE_NONE
var back: Vector2i = _retreat_dir(plan, actor, st.pos, foes, reach, grid.get_position(ranked[0]))
if back != Vector2i.ZERO and plan.add_step(actor, PlanStep.move(SkillCatalog.lowest(SkillDef.Kind.MOVE), back, 1), false).is_empty():
	return SELF_CARE_RETREATED
return SELF_CARE_NONE
```

### `static _kite_then_engage(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, foes: Array[Combatant], ranked: Array[Combatant], band: Vector2i, cap: int, profile: AiProfile) -> bool`
```logic
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
	var path: Array[Vector2i] = GridPath.find_path(grid, st.pos, goals, blocked, false, hazard_penalty(grid), cover_preference(grid, foes))
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
```

### `static hazard_penalty(grid: CombatGrid) -> Callable`
```logic
return func(cell: Vector2i) -> int: return AI_HAZARD_PENALTY if grid.has_hazard(cell) else 0
```

### `static cover_preference(grid: CombatGrid, foes: Array[Combatant]) -> Callable`
```logic
var shooters: Array[Combatant] = []
for f: Combatant in foes:
	if f.is_active() and grid.has_combatant(f) and f.attack_range > 1:
		shooters.append(f)
if shooters.is_empty():
	return Callable()
return func(cell: Vector2i) -> float:
	var nearest: Combatant = null
	var best: int = 1 << 20
	for f: Combatant in shooters:
		var d: int = grid.get_distance(cell, grid.get_position(f))
		if d < best:
			best = d
			nearest = f
	return CoverRules.cover_for(grid, grid.get_position(nearest), cell, true)
```

### `static _advance_toward(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, ranked: Array[Combatant]) -> bool`
```logic
var tiles: int = mini(plan.max_move_tiles(actor), maxi(0, st.ap_left))
if tiles < 1 or st.rooted or st.acted:
	return false
for candidate: Combatant in ranked:
	var route: Array[Vector2i] = plan.approach_path(actor, st.pos, candidate)
	if route.is_empty():
		continue
	var reachable: Array[Vector2i] = GridPath.fit_blocks(plan.context.grid, st.pos, route, actor.move_per_block(), CombatPlan.MAX_MOVE_BLOCKS)   # cost-chunked (S12T-L)
	var dest: Vector2i = st.pos
	for i: int in range(mini(reachable.size(), tiles)):
		dest += reachable[i]
	if plan.add_step(actor, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), dest), false).is_empty():
		return true
return false
```

### `static _retreat_dir(plan: CombatPlan, actor: Combatant, pos: Vector2i, foes: Array[Combatant], reach: int, target_pos: Vector2i) -> Vector2i`
```logic
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
```

### `static _nearest_foe(grid: CombatGrid, pos: Vector2i, foes: Array[Combatant]) -> int`
```logic
var best: int = 99
for f: Combatant in foes:
	if f.is_active() and grid.has_combatant(f):
		best = mini(best, grid.get_distance(pos, grid.get_position(f)))
return best
```

### `static _needs_heal(friend: Combatant, profile: AiProfile) -> bool`
```logic
return friend.hp < friend.max_hp and float(friend.hp) / float(maxi(1, friend.max_hp)) <= profile.heal_below
```

### `static _locked_down(c: Combatant) -> bool`
```logic
return StatusEffects.has_any(c, StatusEffects.STUN) or StatusEffects.has_any(c, StatusEffects.ROOT)
```

### `static _frees_lock(c: Combatant, skill: SkillDef) -> bool`
```logic
var list: Array[String] = SupportRules.cleansable(c, skill)
return list.has(StatusEffects.STUN) or list.has(StatusEffects.ROOT)
```

### `static _nearest_foe_unit(grid: CombatGrid, pos: Vector2i, foes: Array[Combatant]) -> Combatant`
```logic
var best: Combatant = null
var best_dist: int = 99
for f: Combatant in foes:
	if f.is_active() and grid.has_combatant(f):
		var d: int = grid.get_distance(pos, grid.get_position(f))
		if d < best_dist:
			best_dist = d
			best = f
return best
```

### `static _try_disengage(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile, ranked: Array[Combatant], require_strike: bool) -> bool`
```logic
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
```

### `static _try_cleanse(plan: CombatPlan, actor: Combatant, cap: int, urgent: bool) -> bool`
```logic
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
```

### `static _precast(plan: CombatPlan, actor: Combatant, ranked: Array[Combatant], cap: int, profile: AiProfile) -> bool`
```logic
var added: bool = false
for kind: int in range(3):
	var options: Array[PlanStep]
	match kind:
		0:
			options = _taunt_options(plan, actor, cap, profile)
		1:
			options = _buff_options(plan, actor, cap, profile)
		_:
			options = _debuff_options(plan, actor, cap, profile, ranked)
	var kind_added: bool = false
	for step: PlanStep in options:
		var mark: int = plan.steps_of(actor).size()
		if not plan.add_step(actor, step, false).is_empty():
			continue
		if _engage_any(plan, actor, ranked, cap, profile):
			plan.remove_step(actor, mark + 1)
			added = true
			kind_added = true
			break
		plan.remove_step(actor, mark)
	if kind == 0 and not kind_added and not options.is_empty() and _early_taunt_walk(plan, actor, options):
		added = true
return added
```

### `static _early_taunt_walk(plan: CombatPlan, actor: Combatant, taunts: Array[PlanStep]) -> bool`
```logic
var st: CombatPlan.SimState = plan.compile(actor)
if st.moved or st.acted or st.rooted:
	return false
var grid: CombatGrid = plan.context.grid
var diver: Combatant = null
var best: int = 1 << 20
for foe: Combatant in _divers(plan, actor):
	var d: int = grid.get_distance(st.pos, grid.get_position(foe))
	if d < best:
		best = d
		diver = foe
if diver == null:
	return false
var route: Array[Vector2i] = plan.approach_path(actor, st.pos, diver)
if route.is_empty():
	return false
var reachable: Array[Vector2i] = GridPath.fit_blocks(grid, st.pos, route, actor.move_per_block(), CombatPlan.MAX_MOVE_BLOCKS)
var tiles: int = mini(reachable.size(), mini(plan.max_move_tiles(actor), maxi(0, st.ap_left)))
var move_skill: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)
var mark: int = plan.steps_of(actor).size()
for n: int in range(tiles, 0, -1):
	var dest: Vector2i = st.pos
	for i: int in range(n):
		dest += reachable[i]
	if not plan.add_step(actor, PlanStep.go_to(move_skill, dest), false).is_empty():
		continue
	for taunt: PlanStep in taunts:
		if plan.add_step(actor, taunt, false).is_empty():
			return true
	_truncate(plan, actor, mark)
return false
```

### `static _picked_by_a_foe(plan: CombatPlan, ally: Combatant, at: Vector2i, foes: Array[Combatant], where: Dictionary) -> bool`
```logic
var grid: CombatGrid = plan.context.grid
for foe: Combatant in foes:
	if foe.is_active() and grid.has_combatant(foe) and _likely_target(plan, foe, ally, at, where):
		return true
return false
```

### `static _debuff_open(foe: Combatant, status: String) -> bool`
```logic
if StatusRegistry.stacking_of(status) == StatusRegistry.STACK_POTENCY:
	var cap: int = StatusRegistry.cap_of(status)
	return cap <= 0 or DictUtil.int_of(foe.status_magnitudes, status) + DictUtil.int_of(foe.pending_status_magnitudes, status) < cap
return not StatusEffects.has_any(foe, status)
```

### `static _fragile_friend(plan: CombatPlan, actor: Combatant) -> Combatant`
```logic
var best: Combatant = null
for friend: Combatant in plan.context.friends_of(actor):
	if friend.is_active() and plan.context.grid.has_combatant(friend) and (best == null or friend.max_hp < best.max_hp):
		best = friend
return best
```

### `static _guard_smart(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile, foes: Array[Combatant], pos: Vector2i) -> bool`
```logic
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
```

### `static _riposte_finisher(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile, foes: Array[Combatant], hp_ratio: float) -> bool`
```logic
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
```

### `static _try_area(plan: CombatPlan, actor: Combatant, cap: int, profile: AiProfile) -> bool`
```logic
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
		var hit_friends: Array[Combatant] = []
		for friend: Combatant in friends:
			if friend != actor and friend.is_active() and cells.has(plan.planned_position(friend)):
				hit_friends.append(friend)
		if foes_hit < profile.area_threshold() or not _friendly_fire_ok(actor, skill, hit_friends, foes_hit):
			continue
		cands.append({"skill": skill, "aim": aim, "dmg": skill.power_for(actor) * float(foes_hit), "allies": hit_friends.size()})
cands.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
	if not is_equal_approx(DictUtil.to_float(a["dmg"]), DictUtil.to_float(b["dmg"])):
		return DictUtil.to_float(a["dmg"]) > DictUtil.to_float(b["dmg"])
	if DictUtil.to_int(a["allies"]) != DictUtil.to_int(b["allies"]):
		return DictUtil.to_int(a["allies"]) < DictUtil.to_int(b["allies"])
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
```

### `static _friendly_fire_ok(actor: Combatant, skill: SkillDef, hit_friends: Array[Combatant], foes_hit: int) -> bool`
```logic
if hit_friends.is_empty():
	return true
if hit_friends.size() > 1 or foes_hit < AREA_FF_MIN_FOES:
	return false
var ally: Combatant = hit_friends[0]
return float(DamageCalculator.expected(actor, ally, skill)) < AREA_FF_HP_RATIO * float(ally.hp)
```

### `static incoming_damage(plan: CombatPlan, target: Combatant, by_block: int, at: Vector2i, positions: Dictionary = {}) -> int`
```logic
var grid: CombatGrid = plan.context.grid
var total: int = 0
if at == CombatPlan.NO_CELL:
	return total
var where: Dictionary = positions if not positions.is_empty() else plan.planned_positions(plan.context.friends_of(target))
for foe: Combatant in plan.context.foes_of(target):
	if foe.is_active() and grid.has_combatant(foe) and _likely_target(plan, foe, target, at, where):
		total += _strike_chain(plan, foe, target, at, by_block)
return total
```

### `static _likely_target(plan: CombatPlan, foe: Combatant, target: Combatant, at: Vector2i, where: Dictionary) -> bool`
```logic
var grid: CombatGrid = plan.context.grid
var foe_pos: Vector2i = grid.get_position(foe)
var threat: int = plan.max_move_tiles(foe) + SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(foe).y
var foe_profile: AiProfile = plan.context.profile_of(foe)
var nearest_first: bool = foe_profile.reckless or foe_profile.target_pref == AiProfile.TargetPref.NEAREST
var best: float = INF
var mine: float = INF
for unit: Combatant in plan.context.foes_of(foe):
	if not unit.is_active() or not grid.has_combatant(unit):
		continue
	var raw_pos: Variant = where.get(unit, CombatPlan.NO_CELL)
	var pos: Vector2i = at if unit == target else (raw_pos if raw_pos is Vector2i else CombatPlan.NO_CELL)
	if pos == CombatPlan.NO_CELL:
		continue
	var d: int = grid.get_distance(foe_pos, pos)
	if d > threat:
		continue
	var score: float = float(d) * 100.0 + float(unit.hp) if nearest_first else float(unit.hp) + float(d) * 4.0
	if unit.has_status(StatusEffects.TAUNT):
		score -= 100000.0
	best = minf(best, score)
	if unit == target:
		mine = score
return mine < INF and mine <= best
```

### `static _strike_chain(plan: CombatPlan, foe: Combatant, target: Combatant, at: Vector2i, by_block: int) -> int`
```logic
var d: int = plan.context.grid.get_distance(plan.context.grid.get_position(foe), at)
var best: int = 0
for skill: SkillDef in _strikes(foe, plan.context.skill_cap_of(foe)):
	var band: Vector2i = skill.reach_for(foe)
	var tiles: int = 0
	if d > band.y:
		tiles = d - band.y
	elif d < band.x:
		tiles = band.x - d
	var move_blocks: int = 0 if tiles == 0 else ceili(float(tiles) / float(maxi(1, foe.move_per_block())))
	if move_blocks > CombatPlan.MAX_MOVE_BLOCKS:
		continue
	var hits: int = floori(float(by_block - move_blocks) / float(maxi(1, skill.blocks)))
	var cost: int = skill.ap_cost(foe)
	if cost > 0:
		hits = mini(hits, floori(float(foe.ap - tiles) / float(cost)))
	if hits > 0:
		best = maxi(best, hits * DamageCalculator.expected(foe, target, skill))
return best
```

### `static _taunt_pull(plan: CombatPlan, actor: Combatant, st: CombatPlan.SimState, ranked: Array[Combatant]) -> Combatant`
```logic
if ranked.is_empty() or not ranked[0].has_status(StatusEffects.TAUNT):
	return null
var taunter: Combatant = ranked[0]
var grid: CombatGrid = plan.context.grid
var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(actor)
var d: int = grid.get_distance(st.pos, grid.get_position(taunter))
if d >= band.x and d <= band.y:
	return taunter
if st.rooted or st.acted or st.moved:
	return null
var route: Array[Vector2i] = plan.approach_path(actor, st.pos, taunter)
return taunter if not route.is_empty() and plan.fits_move(actor, st.pos, route) else null
```

### `static _reserve_heal(plan: CombatPlan, actor: Combatant, profile: AiProfile, cap: int) -> Combatant`
```logic
var heals: Array[SkillDef] = []
for skill: SkillDef in SkillCatalog.available(SkillDef.Kind.SUPPORT, cap, actor):
	if skill.heals() and skill.ap_cost(actor) <= actor.ap:
		heals.append(skill)
if heals.is_empty():
	return null
heals.sort_custom(func(a: SkillDef, b: SkillDef) -> bool:
	if a.level != b.level:
		return a.level < b.level
	if a.tier != b.tier:
		return a.tier < b.tier
	return a.id < b.id
)
var heal: SkillDef = heals[0]
var grid: CombatGrid = plan.context.grid
var cands: Array[Dictionary] = []
var index: int = 0
var where: Dictionary = plan.planned_positions(plan.context.friends_of(actor))   # one pass for every friend's estimate
for friend: Combatant in plan.context.friends_of(actor):
	index += 1
	if not friend.is_active() or not grid.has_combatant(friend):
		continue
	if (friend == actor and heal.target_mode == SkillDef.TargetMode.ALLY) or (friend != actor and not heal.targets_ally()):
		continue
	var raw_at: Variant = where.get(friend, CombatPlan.NO_CELL)
	var friend_at: Vector2i = raw_at if raw_at is Vector2i else CombatPlan.NO_CELL
	var projected: int = friend.hp - incoming_damage(plan, friend, plan.context.total_blocks, friend_at, where)
	var ratio: float = float(projected) / float(maxi(1, friend.max_hp))
	if ratio <= profile.heal_below:
		cands.append({"unit": friend, "ratio": ratio, "idx": index})
cands.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
	if not is_equal_approx(DictUtil.to_float(a["ratio"]), DictUtil.to_float(b["ratio"])):
		return DictUtil.to_float(a["ratio"]) < DictUtil.to_float(b["ratio"])
	return DictUtil.to_int(a["idx"]) < DictUtil.to_int(b["idx"])
)
var saved: Array[PlanStep] = plan.steps_of(actor)
var keep: int = _move_floor(plan.compile(actor))
for entry: Dictionary in cands:
	var patient: Combatant = entry["unit"]
	var steps: Array[PlanStep] = saved.duplicate()
	while true:
		var attempt: Array[PlanStep] = steps.duplicate()
		plan.set_steps(actor, attempt)
		var step: PlanStep = PlanStep.self_skill(heal) if patient == actor else PlanStep.ally(heal, patient)
		if plan.add_step(actor, step, false).is_empty():
			if not _step_moves(plan.compile(actor), steps.size()):
				return patient
			plan.set_steps(actor, steps.duplicate())
		if steps.size() <= keep:
			break
		steps.pop_back()
plan.set_steps(actor, saved)
return null
```

### `static _move_floor(st: CombatPlan.SimState) -> int`
```logic
var floor_steps: int = 0
for entry: PlanEntry in st.entries:
	if entry.skill != null and entry.step_index >= 0 and (entry.skill.kind == SkillDef.Kind.MOVE or entry.skill.kind == SkillDef.Kind.DISENGAGE or entry.pos_after != entry.pos_before):
		floor_steps = maxi(floor_steps, entry.step_index + 1)
return floor_steps
```

### `static _step_moves(st: CombatPlan.SimState, index: int) -> bool`
```logic
for entry: PlanEntry in st.entries:
	if entry.step_index == index and entry.skill != null and entry.skill.kind == SkillDef.Kind.MOVE:
		return true
return false
```

