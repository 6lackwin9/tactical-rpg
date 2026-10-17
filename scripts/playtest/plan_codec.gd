class_name PlanCodec
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Codec (PLAYTEST1 section D: PlanStep <-> JSON)
# Step: {mode, skill, target, dir, tiles, cell}; unit ref: {side: ally | enemy, i, name}.
# FIELDS maps every PlanStep script property to its JSON key; the playtest suite fails when PlanStep
# gains a property without an entry here, so new step fields cannot be dropped silently.
# Decode errors name the round, unit and step (1-based); a name mismatch alone is a warning.
# ==============================================================================

const FIELDS: Dictionary = {"mode": "mode", "skill": "skill", "target": "target", "direction": "dir", "tiles": "tiles", "cell": "cell"}
const NO_CELL: Array = [-1, -1]

# ------------------------------------------------------------------------------
# Unit refs and roster
# ------------------------------------------------------------------------------

static func unit_ref(session: CombatSession, c: Combatant) -> Variant:
	if c == null:
		return null
	var i: int = session.allies.find(c)
	if i >= 0:
		return {"side": "ally", "i": i, "name": c.name}
	i = session.enemies.find(c)
	if i >= 0:
		return {"side": "enemy", "i": i, "name": c.name}
	return null

# The combatant a ref points at, or null (index out of range / unknown side).
static func resolve_ref(session: CombatSession, ref: Dictionary) -> Combatant:
	var side: String = str(ref.get("side", ""))
	var i: int = DictUtil.to_int(ref.get("i", -1), -1)
	var units: Array[Combatant] = []
	if side == "ally":
		units = session.allies
	elif side == "enemy":
		units = session.enemies
	return units[i] if i >= 0 and i < units.size() else null

static func ref_text(ref: Variant) -> String:
	var d: Dictionary = DictUtil.dict_of(ref)
	if d.is_empty():
		return "?"
	return "%s %d (%s)" % [str(d.get("side", "?")), DictUtil.to_int(d.get("i", -1), -1), str(d.get("name", ""))]

# The catalog unit (or world character) id a combatant was spawned from; "" for hand-built units.
static func catalog_id(session: CombatSession, c: Combatant) -> String:
	if session.encounter is CatalogEncounter:
		var enc: CatalogEncounter = session.encounter
		for unit_id: Variant in enc.units_by_id:
			if DictUtil.array_of(enc.units_by_id[unit_id]).has(c):
				return str(unit_id)
	return ""

# [{ref, catalog_id, name}] for allies then enemies, session order.
static func roster(session: CombatSession) -> Array:
	var out: Array = []
	for c: Combatant in session.allies + session.enemies:
		out.append({"ref": unit_ref(session, c), "catalog_id": catalog_id(session, c), "name": c.name})
	return out

# ------------------------------------------------------------------------------
# Encode
# ------------------------------------------------------------------------------

static func encode_step(session: CombatSession, step: PlanStep) -> Dictionary:
	return {
		"mode": mode_name(step.mode),
		"skill": step.skill.id if step.skill != null else "",
		"target": unit_ref(session, step.target),
		"dir": [step.direction.x, step.direction.y],
		"tiles": step.tiles,
		"cell": [step.cell.x, step.cell.y],
	}

static func encode_steps(session: CombatSession, steps: Array[PlanStep]) -> Array:
	var out: Array = []
	for step: PlanStep in steps:
		out.append(encode_step(session, step))
	return out

# [{unit: ref, steps: [...]}] in plan.sequences insertion order (planning order breaks tile-claim ties).
static func encode_plan(session: CombatSession, plan: CombatPlan) -> Array:
	var out: Array = []
	if plan == null:
		return out
	for key: Variant in plan.sequences:
		if not (key is Combatant):
			continue
		var c: Combatant = key
		out.append({"unit": unit_ref(session, c), "steps": encode_steps(session, plan.steps_of(c))})
	return out

# ------------------------------------------------------------------------------
# Decode
# ------------------------------------------------------------------------------

# PlanStep.Mode by key name (the enum's own value, whatever its numbering); SELF when unknown (check has() first).
static func mode_of(name_text: String) -> PlanStep.Mode:
	match name_text:
		"ENGAGE":
			return PlanStep.Mode.ENGAGE
		"MOVE":
			return PlanStep.Mode.MOVE
		"GOTO":
			return PlanStep.Mode.GOTO
		"AREA":
			return PlanStep.Mode.AREA
		"WITHDRAW":
			return PlanStep.Mode.WITHDRAW
		"ALLY":
			return PlanStep.Mode.ALLY
	return PlanStep.Mode.SELF

# The key name of a PlanStep.Mode value.
static func mode_name(mode: PlanStep.Mode) -> String:
	return str(PlanStep.Mode.find_key(mode))

# {ok, errors, warnings, units: Array[Combatant], steps: Array (of Array[PlanStep])} for C.plans.
static func decode_plans(session: CombatSession, plans: Array, roster_entries: Array, round_num: int) -> Dictionary:
	var errors: Array[String] = []
	var warnings: Array[String] = []
	var units: Array[Combatant] = []
	var step_lists: Array = []
	for raw: Variant in plans:
		var entry: Dictionary = DictUtil.dict_of(raw)
		var ref: Dictionary = DictUtil.dict_of(entry.get("unit", {}))
		var where: String = "round %d, unit %s" % [round_num, ref_text(ref)]
		var actor: Combatant = _check_ref(session, ref, roster_entries, "%s, all steps" % where, errors, warnings)
		if actor == null:
			continue
		var steps: Array[PlanStep] = []
		var raw_steps: Array = DictUtil.array_of(entry.get("steps", []))
		for k: int in range(raw_steps.size()):
			var step: PlanStep = decode_step(session, actor, DictUtil.dict_of(raw_steps[k]), roster_entries, "%s, step %d" % [where, k + 1], errors, warnings)
			if step != null:
				steps.append(step)
		units.append(actor)
		step_lists.append(steps)
	return {"ok": errors.is_empty(), "errors": errors, "warnings": warnings, "units": units, "steps": step_lists}

static func decode_step(session: CombatSession, actor: Combatant, d: Dictionary, roster_entries: Array, where: String, errors: Array[String], warnings: Array[String]) -> PlanStep:
	var name_text: String = str(d.get("mode", ""))
	if not PlanStep.Mode.has(name_text):
		errors.append("%s: unknown mode '%s'." % [where, name_text])
		return null
	var step := PlanStep.new()
	step.mode = mode_of(name_text)
	var skill_id: String = str(d.get("skill", ""))
	if not skill_id.is_empty():
		var skill: SkillDef = SkillCatalog.find(skill_id)
		if skill == null:
			errors.append("%s: unknown skill '%s'." % [where, skill_id])
			return null
		if not actor.can_use(skill):
			errors.append("%s: %s does not own skill '%s'." % [where, actor.name, skill_id])
			return null
		step.skill = skill
	var raw_target: Variant = d.get("target", null)
	if raw_target != null:
		var target: Combatant = _check_ref(session, DictUtil.dict_of(raw_target), roster_entries, "%s target" % where, errors, warnings)
		if target == null:
			return null
		step.target = target
	step.direction = _vec(d.get("dir", [0, 0]), Vector2i.ZERO)
	step.tiles = DictUtil.to_int(d.get("tiles", 1), 1)
	step.cell = _vec(d.get("cell", NO_CELL), Vector2i(-1, -1))
	return step

# Puts decoded plans on the session's live plan, in recorded order.
static func apply(session: CombatSession, decoded: Dictionary) -> void:
	session.plan.clear_all()
	var units: Array = DictUtil.array_of(decoded.get("units", []))
	var step_lists: Array = DictUtil.array_of(decoded.get("steps", []))
	for i: int in range(units.size()):
		var c: Combatant = units[i]
		var steps: Array[PlanStep] = []
		steps.assign(DictUtil.array_of(step_lists[i]))
		session.plan.set_steps(c, steps)

# First invalid active ally of the live plan as "unit: step k: reason" ("" = every sequence is valid).
static func invalid_text(session: CombatSession, round_num: int) -> String:
	for a: Combatant in session.allies:
		if a.is_active() and not session.plan.is_valid(a):
			var st: CombatPlan.SimState = session.plan.compile(a)
			return "round %d, unit %s, step %d: %s" % [round_num, ref_text(unit_ref(session, a)), st.error_index + 1, st.error_text]
	return ""

static func _check_ref(session: CombatSession, ref: Dictionary, roster_entries: Array, where: String, errors: Array[String], warnings: Array[String]) -> Combatant:
	var c: Combatant = resolve_ref(session, ref)
	if c == null:
		errors.append("%s: unit ref %s is out of range." % [where, ref_text(ref)])
		return null
	var slot: int = (session.allies.size() if str(ref.get("side", "")) == "enemy" else 0) + DictUtil.to_int(ref.get("i", 0))
	if slot < roster_entries.size():
		var expected: String = str(DictUtil.dict_of(roster_entries[slot]).get("catalog_id", ""))
		var actual: String = catalog_id(session, c)
		if expected != actual:
			errors.append("%s: unit is '%s' in this build but '%s' in the recording roster." % [where, actual, expected])
			return null
	if str(ref.get("name", c.name)) != c.name:
		warnings.append("%s: name '%s' in the recording, '%s' in this build." % [where, str(ref.get("name", "")), c.name])
	return c

static func _vec(v: Variant, fallback: Vector2i) -> Vector2i:
	var a: Array = DictUtil.array_of(v)
	if a.size() != 2:
		return fallback
	return Vector2i(DictUtil.to_int(a[0]), DictUtil.to_int(a[1]))
