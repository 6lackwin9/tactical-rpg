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
	return _entry_from(gs, gs.world_id, ref)

# Resolves `ref` as written in `world_id`; {} when the quest's world is neither extended nor connected to the party's.
static func _entry_from(gs: GameState, world_id: String, ref: String) -> Dictionary:
	var found: Dictionary = WorldCatalog.resolve(world_id, "quests", ref)
	if found.is_empty() or not world_visible(gs.world_id, str(found.get("_world", world_id))):
		return {}
	return found

# QUESTX: is a quest of `quest_world` readable while the party is in `current`? Its own lineage, a world `current` (or an
# ancestor) connects to, or a world whose lineage connects to `current` (connections are followed both ways, one hop).
static func world_visible(current: String, quest_world: String) -> bool:
	if WorldCatalog.can_reach_world(current, quest_world):
		return true
	var here: Array[String] = WorldCatalog.lineage(current)
	for w: String in WorldCatalog.lineage(quest_world):
		for target: String in JsonUtil.strings(WorldCatalog.manifest(w).get("connects", [])):
			if here.has(target):
				return true
	return false

# The quests this runner tracks now: the world's merged quests, then stored quests of connected worlds (QUESTX), so a
# quest accepted in one world keeps progressing in a connected one. Offers (available / locked) stay per world.
static func _scope(gs: GameState) -> Array[Dictionary]:
	var out: Array[Dictionary] = WorldCatalog.merged_entries(gs.world_id, "quests")
	var seen: Dictionary = {}
	for e: Dictionary in out:
		seen[str(e["_qid"])] = true
	for key: Variant in gs.quests:
		var qid: String = str(key)
		if seen.has(qid):
			continue
		var extra: Dictionary = _entry(gs, qid)
		if _runnable(extra) and str(extra.get("_qid", "")) == qid:
			out.append(extra)
			seen[qid] = true
	return out

# Base world for refs written in a quest: the party's world for its own lineage (so child overrides apply), else the
# quest's own world (connected-world quests only).
static func _world_of(gs: GameState, entry: Dictionary) -> String:
	var w: String = str(entry.get("_world", gs.world_id))
	return gs.world_id if WorldCatalog.lineage(gs.world_id).has(w) else w

static func _runnable(entry: Dictionary) -> bool:
	var template_id: String = str(entry.get("template", ""))
	return not entry.is_empty() and not template_id.is_empty() and QuestCatalog.has(template_id)

static func _stored(gs: GameState, qid: String) -> Dictionary:
	return DictUtil.dict_of(gs.quests.get(qid, {}))

# QUEST2: flag set when the quest turns ready (all objectives done), kept through turn-in, cleared by abandon (the
# quest resets). Same qid-to-key convention as the exploration done flags. Map markers hide on it.
static func ready_flag(qid: String) -> String:
	return "quest.%s.ready" % MapRules.dotted(qid)

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
		var other: Dictionary = _entry_from(gs, _world_of(gs, entry), req)
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
	for e: Dictionary in _scope(gs):
		if status(gs, str(e["_qid"])) == wanted:
			out.append(str(e["_qid"]))
	return out

# {index, kind, text, target} of the current objective (target = location / encounter qid or flag key); {} unless active.
static func current_objective(gs: GameState, ref: String) -> Dictionary:
	if status(gs, ref) != "active":
		return {}
	var entry: Dictionary = _entry(gs, ref)
	var home: String = _world_of(gs, entry)
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
			target = str(WorldCatalog.resolve(home, "locations", str(step.get("location", ""))).get("_qid", ""))
		"win_encounter":
			target = str(WorldCatalog.resolve(home, "encounters", str(step.get("encounter", ""))).get("_qid", ""))
		"flag":
			target = str(step.get("flag", ""))
	return {"index": index, "kind": kind, "text": str(step.get("text", "")), "target": target, "value": step.get("value", true)}

# Loot qids the quest offers now: resolved loot_pool minus loot_claimed minus entries whose requires_flags fail, in
# pool order. Same rule in any status (UI preview); only complete() claims.
static func offers(gs: GameState, ref: String) -> Array[String]:
	var out: Array[String] = []
	var q: Dictionary = resolved(gs, ref)
	for lref: String in JsonUtil.strings(q.get("loot_pool", [])):
		var loot: Dictionary = WorldCatalog.resolve(_world_of(gs, q), "loot", lref)
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
		var lworld: String = WorldCatalog.split_qid(lqid)[0] if lqid.contains(":") and WorldCatalog.split_qid(lqid).size() == 2 else gs.world_id
		var base: String = gs.world_id if WorldCatalog.lineage(gs.world_id).has(lworld) else lworld
		if str(WorldCatalog.resolve(base, "loot", lqid).get("_qid", "")) != lqid:
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
	var qid: String = str(_entry(gs, ref)["_qid"])
	gs.quests.erase(qid)
	gs.clear_flag(ready_flag(qid))
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

# Error text matches GameState.set_flag ("" = set, then refresh).
static func set_flag(gs: GameState, key: String, value: Variant) -> String:
	var err: String = gs.set_flag(key, value)
	if err.is_empty():
		refresh(gs)
	return err

# add_flag (REP1-L 2.2): adds `delta` to an int flag (missing = 0), clamped to `lo` / `hi` when ints and to
# RepRules.MIN..MAX for `rep.` keys, then refresh. "" = done; a non-int flag changes nothing ("Flag '%s' is not a number.").
static func add_flag(gs: GameState, key: String, delta: int, lo: Variant = null, hi: Variant = null) -> String:
	var res: Dictionary = QuestRules.added_flag(gs, key, delta, lo, hi)
	if not str(res["error"]).is_empty():
		return str(res["error"])
	return set_flag(gs, key, res["value"])

# Repeats until nothing changes: met flag objectives advance, quests at their objective count become ready, available
# auto_start quests start. Returns the qids that changed. Bounded (variants change no stored data, so no loop).
static func refresh(gs: GameState) -> Array[String]:
	var changed: Array[String] = []
	var quests: Array[Dictionary] = _scope(gs)
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
				gs.set_flag(ready_flag(qid), true)
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
# picked loot (RewardGrant), claims the picks, sets complete_flags in order, applies `reputation` (REP1-L 2.2: per key
# in order, an add_flag on RepRules.key(faction qid) with the rep clamp), stores completed, then ONE refresh.
# All-or-nothing (a rep flag that is not a number refuses). Returns {ok, errors, granted, notes}; granted.reputation =
# {faction qid: the change actually applied after the clamp}.
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
			loot.append(WorldCatalog.resolve(_world_of(gs, entry), "loot", p))
	var flags_to_set: Array[Dictionary] = []
	for c: Variant in DictUtil.array_of(q.get("complete_flags", [])):
		var cond: Dictionary = DictUtil.dict_of(c)
		var key: String = str(cond.get("key", ""))
		var value: Variant = cond.get("value", true)
		if not GameState.flag_key_ok(key) or not GameState.flag_value_ok(value):
			errors.append("Complete flag '%s' is invalid." % key)
		flags_to_set.append({"key": key, "value": value})
	var rep_keys: Array[Dictionary] = []   # [{faction, key, delta}]
	var rep: Dictionary = DictUtil.dict_of(JsonUtil.normalize(entry.get("reputation", {})))
	for fref: Variant in rep:
		var faction: Dictionary = WorldCatalog.resolve(str(entry.get("_world", gs.world_id)), "factions", str(fref))
		if faction.is_empty() or not (rep[fref] is int):
			continue   # validation reports it; an unknown faction never blocks the turn-in
		var rkey: String = RepRules.key(str(faction["_qid"]))
		var after: Variant = gs.get_flag(rkey, 0)
		for f: Dictionary in flags_to_set:
			if str(f["key"]) == rkey:
				after = f["value"]
		if not (after is int):
			errors.append("Flag '%s' is not a number." % rkey)
		rep_keys.append({"faction": str(faction["_qid"]), "key": rkey, "delta": DictUtil.to_int(rep[fref])})
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
	var applied: Dictionary = {}
	for r: Dictionary in rep_keys:
		var rkey: String = str(r["key"])
		var before: int = DictUtil.to_int(gs.get_flag(rkey, 0))
		var added: Dictionary = QuestRules.added_flag(gs, rkey, DictUtil.to_int(r["delta"]))
		if str(added["error"]).is_empty():
			gs.set_flag(rkey, added["value"])
			applied[str(r["faction"])] = DictUtil.to_int(applied.get(str(r["faction"]), 0)) + DictUtil.to_int(added["value"]) - before
	gs.quests[qid] = {"state": "completed", "objective": _count(q)}
	refresh(gs)
	var granted: Dictionary = DictUtil.dict_of(result["granted"])
	granted["reputation"] = applied
	return {"ok": true, "errors": errors, "granted": granted, "notes": result["notes"]}
