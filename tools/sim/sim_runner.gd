extends RefCounted

# Balance sim runner (BAL2_SIM.md sections 1-3): sandbox units with variants, one fight with the
# round cap / preset / start distance, the binary and HP-margin scores, and the
# side-swapped matchup average. Loaded with preload (no class_name); writes only the sandbox fork.

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimVariants = preload("res://tools/sim/sim_variants.gd")
const SimObserver = preload("res://tools/sim/sim_observer.gd")
const SimRoles = preload("res://tools/sim/sim_roles.gd")

const MAX_STEPS_PER_ROUND: int = 600   # safety stop for one round's step loop

var args: SimArgs
var defs: Dictionary = {}           # unit id -> {"class_id", "level", "tier", "side", "party"}
var _written: Dictionary = {}       # unit id -> spend variant last upserted ("" = not yet)
var spend_variant: String = "class"
var preset: String = "balanced"
var observer: SimObserver = null    # commit 1b: the metrics collector
var not_moved_total: int = 0        # distance: units left in place, summed over fights
var fights: int = 0
var plan_fails: int = 0             # fights stopped because confirm_plan() refused the preset plan
var enemy_block: Vector2i = Vector2i.ZERO   # BAL3 3.2: columns x rows of the clumped enemy block (ZERO = none)
var block_misses: int = 0           # fights where the enemy block found no room (enemies stay)

func _init(p_args: SimArgs) -> void:
	args = p_args

# Declares sandbox unit `id`. `party` = the party side (the `loadout` argument and the spend
# variants apply). Options: "loadout" (explicit family list, wins over the argument), "plain" (true:
# a party reference unit with class spend and default loadout), "copy_of" (catalog unit id copied
# with `level`; class_id and tier come from it).
func define(id: String, class_id: String, level: int, tier: String, party: bool, opts: Dictionary = {}) -> void:
	var d: Dictionary = {"class_id": class_id, "level": level, "tier": tier, "side": "party" if party else "enemy", "party": party,
		"loadout": DictUtil.array_of(opts.get("loadout", [])), "plain": DictUtil.to_bool(opts.get("plain", false)), "copy_of": str(opts.get("copy_of", ""))}
	if DictUtil.dict_of(defs.get(id, {})) == d:
		return   # unchanged: the sandbox entry already written stays valid
	defs[id] = d
	_written.erase(id)

# Upserts unit `id` for the current spend variant (only when it changed). False = refused (printed).
# Ids that are not declared but exist in the catalog (catalog encounter rosters) are used as they are.
func _write(id: String) -> bool:
	var d: Dictionary = DictUtil.dict_of(defs.get(id, {}))
	if d.is_empty():
		if ContentCatalog.has("units", id):
			return true
		out("SIM unknown unit %s" % id)
		return false
	var party: bool = DictUtil.to_bool(d["party"])
	var plain: bool = DictUtil.to_bool(d["plain"])
	var variant: String = spend_variant if party and not plain else "class"
	if str(_written.get(id, "")) == variant:
		return true
	var class_id: String = str(d["class_id"])
	var level: int = DictUtil.to_int(d["level"], 1)
	var entry: Dictionary = {"id": id, "name": id, "side": str(d["side"]), "class_id": class_id, "level": level, "tier": str(d["tier"]), "flee_ratio": 0.0}
	var copy_of: String = str(d["copy_of"])
	if not copy_of.is_empty():
		entry = ContentCatalog.get_entry("units", copy_of).duplicate(true)
		entry["id"] = id
		entry["level"] = level
	var explicit: Array = DictUtil.array_of(d["loadout"])
	if not explicit.is_empty():
		entry["loadout"] = explicit
	if party and not plain:
		entry["spend"] = SimVariants.spend_for(variant, class_id, level)
		if explicit.is_empty() and args.loadout == "newest":
			entry["loadout"] = SimVariants.newest_loadout(class_id, level)
		elif explicit.is_empty() and args.loadout == "role":
			entry["loadout"] = SimRoles.role_loadout(class_id, SimRoles.primary_role(class_id), level)
	var errors: Array[String] = ContentCatalog.upsert("units", entry)
	if not errors.is_empty():
		out("SIM unit error %s (spend %s, loadout %s): %s" % [id, variant, str(entry.get("loadout", args.loadout)), str(errors)])
		_written.erase(id)
		return false
	_written[id] = variant
	return true

# Builds the Combatant of declared unit `id` for the current variant (stats mode). Null if refused.
func build(id: String) -> Combatant:
	if not _write(id):
		return null
	return ContentFactory.unit(ContentCatalog.get_entry("units", id))

# One battle of `left` (party) vs `right` (enemies). `base`: encounter fields that replace the
# generated defaults (zone, arena, exits, blocks, AP, wave_units / wave_round / wave_block); party,
# enemies, max_rounds (= cap) and rng_seed are always the sim's. Record keys: l, r (HP fractions),
# bin, hp (scores of the left side), rounds, cleared (every enemy down, waves included),
# survivors, party_size, ok (false = a unit was refused).
func fight(left: Array[String], right: Array[String], seed_value: int, base: Dictionary = {}) -> Dictionary:
	var waves: Array[String] = []
	for raw: Variant in DictUtil.array_of(base.get("wave_units", [])):
		waves.append(str(raw))
	for id: String in left + right + waves:
		if not _write(id):
			return {"ok": false}
	var entry: Dictionary = {"id": "sim_fight", "name": "Sim", "zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12, "ap_regen": 4}
	for key: Variant in base:
		entry[key] = base[key]
	var party_ids: Array[String] = left
	var jitter_rng: RandomNumberGenerator = null
	if args.jitter:
		jitter_rng = SimVariants.jitter_rng(seed_value)
		party_ids = SimVariants.shuffled(left, jitter_rng)
	entry.merge({"party": party_ids, "enemies": right, "max_rounds": args.cap, "rng_seed": seed_value}, true)
	var session := CombatSession.new(CatalogEncounter.from_entry(entry))
	session.start()
	_prepare(session, jitter_rng)
	if observer != null:
		observer.on_start(session)
	var rounds: int = 0
	var plan_failed: bool = false
	for round_num: int in range(args.cap):
		rounds = round_num + 1
		session.plan_apply_preset_all(preset)
		if not session.confirm_plan():
			_report_plan_fail(session, round_num + 1)
			plan_failed = true
			break
		for i: int in range(MAX_STEPS_PER_ROUND):
			if session.phase == CombatSession.Phase.REPLAY:
				break
			session.step_next()
			if observer != null:
				observer.on_step(session)
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
		if observer != null:
			observer.on_round(session)
	var l: float = side_hp_frac(session.allies)
	var r: float = side_hp_frac(session.enemies)
	var survivors: int = 0
	for c: Combatant in session.allies:
		if c.is_active():
			survivors += 1
	var won: bool = session.outcome == EncounterObjectives.WON   # OBJ1 rule 11: a hold, escape or slay win scores 1 like a KO win
	var result: Dictionary = {"ok": true, "l": l, "r": r, "bin": 1.0 if won else binary_score(l, r), "hp": 1.0 if won else hp_score(l, r), "rounds": rounds, "cleared": r <= 0.0,
		"survivors": survivors, "party_size": session.allies.size(), "plan_fail": plan_failed}
	if observer != null:
		observer.on_end(session, result)
	session.dispose()
	fights += 1
	return result

# A refused confirm_plan() ends the fight early; it is counted and printed (never silent):
# one `PLAN FAIL <unit> <reason>` line per invalid ally, with its position and steps.
func _report_plan_fail(session: CombatSession, round_num: int) -> void:
	plan_fails += 1
	for a: Combatant in session.allies:
		if not a.is_active():
			continue
		var st: CombatPlan.SimState = session.plan.compile(a)
		if st.error_index < 0:
			continue
		var steps: Array[String] = []
		var holders: Array[String] = []
		var all_steps: Array[PlanStep] = session.plan.steps_of(a)
		for s: PlanStep in all_steps:
			steps.append("%s%s" % [s.title(), (" @" + str(s.cell)) if s.cell != CombatPlan.NO_CELL else ""])
		var bad_cell: Vector2i = all_steps[st.error_index].cell
		for other: Combatant in session.allies + session.enemies:
			if other != a and other.is_active() and session.grid.has_combatant(other) and bad_cell != CombatPlan.NO_CELL:
				if session.grid.get_position(other) == bad_cell or session.plan.planned_position(other) == bad_cell:
					holders.append("%s %s->%s" % [other.name, str(session.grid.get_position(other)), str(session.plan.planned_position(other))])
		out("PLAN FAIL %s %s (round %d, preset %s, at %s, step %d of [%s]; tile held by [%s])" % [a.name, st.error_text, round_num, preset,
			str(session.grid.get_position(a)), st.error_index + 1, ", ".join(steps), ", ".join(holders)])

# After start(): enemy presets, the seeded start jitter (`jitter_rng`, null = off), the start
# distance and the enemy block; then the round-1 enemy plan is redrafted (start() already
# scheduled it with the original profiles and positions).
func _prepare(session: CombatSession, jitter_rng: RandomNumberGenerator = null) -> void:
	var changed: bool = false
	if preset != "balanced":
		for e: Combatant in session.enemies:
			var raw: Variant = session.encounter.ai_profiles.get(e, null)
			if raw is AiProfile:
				var base: AiProfile = raw
				session.encounter.ai_profiles[e] = PlanPresets.profile_for(preset, base)
		changed = true
	if jitter_rng != null:
		SimVariants.jitter_start(session.grid, session.allies, session.enemies, jitter_rng)
		changed = true
	if args.distance > 0:
		not_moved_total += SimVariants.apply_distance(session.grid, session.allies, session.enemies, args.distance)
		changed = true
	if enemy_block != Vector2i.ZERO:
		var shift: int = jitter_rng.randi_range(-1, 1) if jitter_rng != null else 0
		if not SimVariants.apply_block(session.grid, session.allies, session.enemies, enemy_block, args.distance, shift):
			block_misses += 1
		changed = true
	if changed:
		session.resolver.scheduled_actions.clear()
		session.plan = CombatPlan.new(session.encounter.plan_context())
		session.encounter.schedule_round(session.resolver, session.current_round)

# Remaining HP / max HP summed over a side (inactive units count 0; wave units count from the start).
static func side_hp_frac(units: Array[Combatant]) -> float:
	var hp: float = 0.0
	var max_hp: float = 0.0
	for c: Combatant in units:
		hp += float(c.hp if c.is_active() else 0)
		max_hp += float(c.max_hp)
	return hp / maxf(1.0, max_hp)

# Section 2, binary: KO win 1, KO loss 0, else the HP fraction lead with a 0.02 dead band.
static func binary_score(l: float, r: float) -> float:
	if r <= 0.0 and l > 0.0:
		return 1.0
	if l <= 0.0 and r > 0.0:
		return 0.0
	return 1.0 if l > r + 0.02 else (0.0 if r > l + 0.02 else 0.5)

# Section 2, HP margin: KO win 1, KO loss 0, both down 0.5, else clamp(0.5 + (l - r) / 2, 0, 1).
static func hp_score(l: float, r: float) -> float:
	if r <= 0.0 and l > 0.0:
		return 1.0
	if l <= 0.0 and r > 0.0:
		return 0.0
	if l <= 0.0 and r <= 0.0:
		return 0.5
	return clampf(0.5 + (l - r) / 2.0, 0.0, 1.0)

# Mean scores of `a` vs `b` over spend variants x presets x seeds, sides swapped:
# {"bin", "hp", "n"} (n = fights counted; 0 when every variant was refused).
func matchup(a_ids: Array[String], b_ids: Array[String]) -> Dictionary:
	var bin_sum: float = 0.0
	var hp_sum: float = 0.0
	var n: int = 0
	for sv: String in args.spend_variants():
		spend_variant = sv
		for pv: String in args.preset_variants():
			preset = pv
			for s: int in args.seeds():
				var ab: Dictionary = fight(a_ids, b_ids, s)
				var ba: Dictionary = fight(b_ids, a_ids, s)
				if not DictUtil.to_bool(ab.get("ok", false)) or not DictUtil.to_bool(ba.get("ok", false)):
					continue
				bin_sum += DictUtil.to_float(ab["bin"]) + 1.0 - DictUtil.to_float(ba["bin"])
				hp_sum += DictUtil.to_float(ab["hp"]) + 1.0 - DictUtil.to_float(ba["hp"])
				n += 2
	spend_variant = "class"
	preset = "balanced"
	return {"bin": bin_sum / maxf(1.0, float(n)), "hp": hp_sum / maxf(1.0, float(n)), "n": n}

# A score cell: "bin/hp" in percent (or one of them per the `score` key); "n/a" when nothing ran.
func cell(res: Dictionary) -> String:
	if DictUtil.to_int(res.get("n", 0)) <= 0:
		return "n/a"
	var b: int = roundi(DictUtil.to_float(res["bin"]) * 100.0)
	var h: int = roundi(DictUtil.to_float(res["hp"]) * 100.0)
	if not args.show_hp():
		return "%d%%" % b
	if not args.show_binary():
		return "%d" % h
	return "%d/%d" % [b, h]

# Distance note printed under a table.
func distance_note() -> void:
	if args.distance > 0:
		out("distance %d: %d units not moved (summed over %d fights)" % [args.distance, not_moved_total, fights])

static func out(text: String) -> void:
	print("§ " + text)
