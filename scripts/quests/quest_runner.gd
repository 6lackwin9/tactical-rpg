class_name QuestRunner
extends RefCounted

# ==============================================================================
# Tactical RPG - Quest runner (S8, spec M4-L): the quest state machine over GameState.
# Stored per quest (GameState.quests, key = the resolved entry's qualified id): {state: active|ready|completed,
# objective: index}. Everything else (variants, availability, offers) re-resolves from flags + loot_claimed on every
# read, so it is deterministic and never saved. Refs resolve from gs.world_id. Mutators return problems or results
# and never throw; the game loop calls on_location_reached / on_encounter_won / set_flag explicitly.
# ==============================================================================

static func _entry(gs: GameState, ref: String) -> Dictionary:
	return WorldCatalog.resolve(gs.world_id, "quests", ref)

static func _runnable(entry: Dictionary) -> bool:
	var template_id: String = str(entry.get("template", ""))
	return not entry.is_empty() and not template_id.is_empty() and QuestCatalog.has(template_id)

static func _stored(gs: GameState, qid: String) -> Dictionary:
	return DictUtil.dict_of(gs.quests.get(qid, {}))

static func _count(resolved_quest: Dictionary) -> int:
	return DictUtil.array_of(resolved_quest.get("objectives", [])).size()

# ------------------------------------------------------------------------------
# Reading
# ------------------------------------------------------------------------------

# invalid | completed | blocked | ready | active | unavailable | available | locked (spec M4-L status table).
static func status(gs: GameState, ref: String) -> String:
	var entry: Dictionary = _entry(gs, ref)
	if not _runnable(entry):
		return "invalid"
	var stored: Dictionary = _stored(gs, str(entry["_qid"]))
	var state: String = str(stored.get("state", ""))
	if state == "completed":
		return "completed"
	var available: bool = DictUtil.to_bool(QuestRules.resolve(gs, entry).get("available", true))
	if not state.is_empty():
		return state if available else "blocked"
	if not available:
		return "unavailable"
	for req: String in JsonUtil.strings(entry.get("requires", [])):
		var other: Dictionary = _entry(gs, req)
		if other.is_empty() or str(_stored(gs, str(other["_qid"])).get("state", "")) != "completed":
			return "locked"
	for c: Variant in DictUtil.array_of(entry.get("requires_flags", [])):
		if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return "locked"
	return "available"

# The resolved quest ({} if invalid): giver, turn_in, links, texts, objectives, rewards, loot_pool...
static func resolved(gs: GameState, ref: String) -> Dictionary:
	var entry: Dictionary = _entry(gs, ref)
	return QuestRules.resolve(gs, entry) if _runnable(entry) else {}

# Qids of the world's merged quests in `wanted` status, catalog order.
static func quests_with(gs: GameState, wanted: String) -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in WorldCatalog.merged_entries(gs.world_id, "quests"):
		if status(gs, str(e["_qid"])) == wanted:
			out.append(str(e["_qid"]))
	return out

# {index, kind, text, target} of the current objective (target = location / encounter qid or flag key); {} unless active.
static func current_objective(gs: GameState, ref: String) -> Dictionary:
	if status(gs, ref) != "active":
		return {}
	var entry: Dictionary = _entry(gs, ref)
	var q: Dictionary = QuestRules.resolve(gs, entry)
	var index: int = DictUtil.to_int(_stored(gs, str(entry["_qid"])).get("objective", 0))
	var objectives: Array = DictUtil.array_of(q.get("objectives", []))
	if index < 0 or index >= objectives.size():
		return {}
	var kind: String = str(DictUtil.dict_of(objectives[index]).get("kind", ""))
	var steps: Array = DictUtil.array_of(q.get("steps", []))
	var step: Dictionary = DictUtil.dict_of(steps[index]) if index < steps.size() else {}
	var target: String = ""
	match kind:
		"reach_location":
			target = str(WorldCatalog.resolve(gs.world_id, "locations", str(step.get("location", ""))).get("_qid", ""))
		"win_encounter":
			target = str(WorldCatalog.resolve(gs.world_id, "encounters", str(step.get("encounter", ""))).get("_qid", ""))
		"flag":
			target = str(step.get("flag", ""))
	return {"index": index, "kind": kind, "text": str(step.get("text", "")), "target": target, "value": step.get("value", true)}

# Loot qids the quest offers now: resolved loot_pool minus loot_claimed minus entries whose requires_flags fail, in
# pool order. Same rule in any status (UI preview); only complete() claims.
static func offers(gs: GameState, ref: String) -> Array[String]:
	var out: Array[String] = []
	var q: Dictionary = resolved(gs, ref)
	for lref: String in JsonUtil.strings(q.get("loot_pool", [])):
		var loot: Dictionary = WorldCatalog.resolve(gs.world_id, "loot", lref)
		if loot.is_empty():
			continue
		var lqid: String = str(loot["_qid"])
		if out.has(lqid) or gs.loot_claimed.has(lqid):
			continue
		var ok: bool = true
		for c: Variant in DictUtil.array_of(JsonUtil.normalize(loot.get("requires_flags", []))):
			ok = ok and QuestRules.condition_met(gs, DictUtil.dict_of(c))
		if ok:
			out.append(lqid)
	return out

static func picks_allowed(gs: GameState, ref: String) -> int:
	return mini(DictUtil.to_int(resolved(gs, ref).get("loot_picks", 0)), offers(gs, ref).size())   # resolve() fills the schema default; {} = invalid

# Warnings only (nothing changes): unknown quest qids, indexes beyond the template, unknown claimed loot ids.
static func check(gs: GameState) -> Array[String]:
	var out: Array[String] = []
	for key: Variant in gs.quests:
		var qid: String = str(key)
		var entry: Dictionary = _entry(gs, qid)
		if not _runnable(entry) or str(entry.get("_qid", "")) != qid:
			out.append("Unknown quest '%s' in the save." % qid)
			continue
		var index: int = DictUtil.to_int(_stored(gs, qid).get("objective", 0))
		var count: int = _count(QuestRules.resolve(gs, entry))
		if index > count:
			out.append("Quest '%s' is at objective %d but its template has %d." % [qid, index, count])
	for lqid: String in gs.loot_claimed:
		if str(WorldCatalog.resolve(gs.world_id, "loot", lqid).get("_qid", "")) != lqid:
			out.append("Unknown claimed loot '%s' in the save." % lqid)
	return out

# ------------------------------------------------------------------------------
# Changing state
# ------------------------------------------------------------------------------

static func start(gs: GameState, ref: String) -> Array[String]:
	var errors: Array[String] = []
	var st: String = status(gs, ref)
	if st != "available":
		errors.append("Quest '%s' can't start (it is %s)." % [ref, st])
		return errors
	gs.quests[str(_entry(gs, ref)["_qid"])] = {"state": "active", "objective": 0}
	refresh(gs)
	return errors

static func abandon(gs: GameState, ref: String) -> Array[String]:
	var errors: Array[String] = []
	var st: String = status(gs, ref)
	if not ["active", "ready", "blocked"].has(st):
		errors.append("Quest '%s' can't be abandoned (it is %s)." % [ref, st])
		return errors
	gs.quests.erase(str(_entry(gs, ref)["_qid"]))
	return errors

static func on_location_reached(gs: GameState, location_ref: String) -> Array[String]:
	return _on_event(gs, "reach_location", str(WorldCatalog.resolve(gs.world_id, "locations", location_ref).get("_qid", "")))

static func on_encounter_won(gs: GameState, encounter_ref: String) -> Array[String]:
	return _on_event(gs, "win_encounter", str(WorldCatalog.resolve(gs.world_id, "encounters", encounter_ref).get("_qid", "")))

# One event advances at most one objective per active (not blocked) quest bound to the same target.
static func _on_event(gs: GameState, kind: String, target: String) -> Array[String]:
	var advanced: Array[String] = []
	if target.is_empty():
		return advanced
	for qid: String in quests_with(gs, "active"):
		var cur: Dictionary = current_objective(gs, qid)
		if str(cur.get("kind", "")) == kind and str(cur.get("target", "")) == target:
			_advance(gs, qid)
			advanced.append(qid)
	refresh(gs)
	return advanced

static func _advance(gs: GameState, qid: String) -> void:
	var stored: Dictionary = _stored(gs, qid)
	gs.quests[qid] = {"state": "active", "objective": DictUtil.to_int(stored.get("objective", 0)) + 1}

# Error text as GameState.set_flag ("" = set, then refresh).
static func set_flag(gs: GameState, key: String, value: Variant) -> String:
	var err: String = gs.set_flag(key, value)
	if err.is_empty():
		refresh(gs)
	return err

# Repeats until nothing changes: met flag objectives advance, quests at their objective count become ready, available
# auto_start quests start. Returns the qids that changed. Bounded (variants change no stored data, so no loop).
static func refresh(gs: GameState) -> Array[String]:
	var changed: Array[String] = []
	var quests: Array[Dictionary] = WorldCatalog.merged_entries(gs.world_id, "quests")
	var passes: int = quests.size() * (QuestRules.MAX_OBJECTIVES + 1) + 1
	for _pass: int in range(passes):
		var any: bool = false
		for e: Dictionary in quests:
			var qid: String = str(e["_qid"])
			var st: String = status(gs, qid)
			if st == "available" and DictUtil.to_bool(e.get("auto_start", false)):
				gs.quests[qid] = {"state": "active", "objective": 0}
				st = "active"
				any = true
				if not changed.has(qid):
					changed.append(qid)
			if st != "active":
				continue
			var index: int = DictUtil.to_int(_stored(gs, qid).get("objective", 0))
			if index >= _count(QuestRules.resolve(gs, e)):
				gs.quests[qid] = {"state": "ready", "objective": index}
				any = true
				if not changed.has(qid):
					changed.append(qid)
				continue
			var cur: Dictionary = current_objective(gs, qid)
			if str(cur.get("kind", "")) == "flag" and QuestRules.condition_met(gs, {"key": cur["target"], "value": cur["value"]}):
				_advance(gs, qid)
				any = true
				if not changed.has(qid):
					changed.append(qid)
		if not any:
			break
	return changed

# Only from ready. `picks`: unique, each in offers(), 0..picks_allowed(). Grants the resolved generic rewards plus the
# picked loot (RewardGrant), claims the picks, sets complete_flags in order, stores completed, then refresh.
# All-or-nothing. Returns {ok, errors, granted, notes}.
static func complete(gs: GameState, ref: String, recipient_id: String = "", picks: Array[String] = []) -> Dictionary:
	var errors: Array[String] = []
	var fail: Dictionary = {"ok": false, "errors": errors, "granted": {}, "notes": [] as Array[String]}
	if status(gs, ref) != "ready":
		errors.append("quest is not ready")
		return fail
	var entry: Dictionary = _entry(gs, ref)
	var qid: String = str(entry["_qid"])
	var q: Dictionary = QuestRules.resolve(gs, entry)
	var offered: Array[String] = offers(gs, ref)
	var allowed: int = picks_allowed(gs, ref)
	var loot: Array[Dictionary] = []
	var seen: Dictionary = {}
	if picks.size() > allowed:
		errors.append("At most %d loot picks are allowed (got %d)." % [allowed, picks.size()])
	for p: String in picks:
		if seen.has(p):
			errors.append("Loot '%s' is picked twice." % p)
		seen[p] = true
		if not offered.has(p):
			errors.append("Loot '%s' is not offered by this quest." % p)
		else:
			loot.append(WorldCatalog.resolve(gs.world_id, "loot", p))
	var flags_to_set: Array[Dictionary] = []
	for c: Variant in DictUtil.array_of(q.get("complete_flags", [])):
		var cond: Dictionary = DictUtil.dict_of(c)
		var key: String = str(cond.get("key", ""))
		var value: Variant = cond.get("value", true)
		if not GameState.flag_key_ok(key) or not GameState.flag_value_ok(value):
			errors.append("Complete flag '%s' is invalid." % key)
		flags_to_set.append({"key": key, "value": value})
	if not errors.is_empty():
		return fail
	var result: Dictionary = RewardGrant.apply(gs, DictUtil.dict_of(q.get("rewards", {})), recipient_id, loot)
	if not DictUtil.to_bool(result["ok"]):   # RewardGrant validates first and changes nothing on error
		errors.assign(DictUtil.array_of(result["errors"]))
		return fail
	for p: String in picks:
		gs.loot_claimed.append(p)
	for f: Dictionary in flags_to_set:
		gs.set_flag(str(f["key"]), f["value"])
	gs.quests[qid] = {"state": "completed", "objective": _count(q)}
	refresh(gs)
	return {"ok": true, "errors": errors, "granted": result["granted"], "notes": result["notes"]}
