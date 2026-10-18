---
class: QuestRunner
extends: RefCounted
description: "Tactical RPG - Quest runner (S8, spec M4-L): the quest state machine over GameState. Stored per quest (GameState.quests, key = the resolved entry's qualified id): {state: active|ready|completed, objective: index}. Everything else (variants, availability, offers) re-resolves from flags + loot_claimed on every read, so it is deterministic and never saved. Refs resolve from gs.world_id. Mutators return problems or results and never throw; the game loop calls on_location_reached / on_encounter_won / set_flag explicitly."
---

# Properties
* `out`: Array[Dictionary] = WorldCatalog.merged_entries(gs.world_id, "quests")
* `seen`: Dictionary = {}
* `qid`: String = str(key)
* `extra`: Dictionary = _entry(gs, qid)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `q`: Dictionary = resolved(gs, ref)
* `loot`: Dictionary = WorldCatalog.resolve(_world_of(gs, q), "loot", lref)
* `lqid`: String = str(loot["_qid"])
* `ok`: bool = true
* `out`: Array[String] = []
* `qid`: String = str(key)
* `entry`: Dictionary = _entry(gs, qid)
* `index`: int = DictUtil.to_int(_stored(gs, qid).get("objective", 0))
* `count`: int = _count(QuestRules.resolve(gs, entry))
* `lworld`: String = WorldCatalog.split_qid(lqid)[0] if lqid.contains(":") and WorldCatalog.split_qid(lqid).size() == 2 else gs.world_id
* `base`: String = gs.world_id if WorldCatalog.lineage(gs.world_id).has(lworld) else lworld
* `errors`: Array[String] = []
* `st`: String = status(gs, ref)
* `errors`: Array[String] = []
* `st`: String = status(gs, ref)
* `qid`: String = str(_entry(gs, ref)["_qid"])
* `advanced`: Array[String] = []
* `cur`: Dictionary = current_objective(gs, qid)
* `changed`: Array[String] = []
* `quests`: Array[Dictionary] = _scope(gs)
* `passes`: int = quests.size() * (QuestRules.MAX_OBJECTIVES + 1) + 1
* `any`: bool = false
* `qid`: String = str(e["_qid"])
* `st`: String = status(gs, qid)
* `index`: int = DictUtil.to_int(_stored(gs, qid).get("objective", 0))
* `cur`: Dictionary = current_objective(gs, qid)

# Methods

### `static _entry(gs: GameState, ref: String) -> Dictionary`
```logic
return _entry_from(gs, gs.world_id, ref)
```

### `static _entry_from(gs: GameState, world_id: String, ref: String) -> Dictionary`
```logic
var found: Dictionary = WorldCatalog.resolve(world_id, "quests", ref)
if found.is_empty() or not world_visible(gs.world_id, str(found.get("_world", world_id))):
	return {}
return found
```

### `static world_visible(current: String, quest_world: String) -> bool`
```logic
if WorldCatalog.can_reach_world(current, quest_world):
	return true
var here: Array[String] = WorldCatalog.lineage(current)
for w: String in WorldCatalog.lineage(quest_world):
	for target: String in JsonUtil.strings(WorldCatalog.manifest(w).get("connects", [])):
		if here.has(target):
			return true
return false
```

### `static _world_of(gs: GameState, entry: Dictionary) -> String`
```logic
var w: String = str(entry.get("_world", gs.world_id))
return gs.world_id if WorldCatalog.lineage(gs.world_id).has(w) else w
```

### `static _runnable(entry: Dictionary) -> bool`
```logic
var template_id: String = str(entry.get("template", ""))
return not entry.is_empty() and not template_id.is_empty() and QuestCatalog.has(template_id)
```

### `static _stored(gs: GameState, qid: String) -> Dictionary`
```logic
return DictUtil.dict_of(gs.quests.get(qid, {}))
```

### `static ready_flag(qid: String) -> String`
```logic
return "quest.%s.ready" % MapRules.dotted(qid)
```

### `static _count(resolved_quest: Dictionary) -> int`
```logic
return DictUtil.array_of(resolved_quest.get("objectives", [])).size()
```

### `static status(gs: GameState, ref: String) -> String`
```logic
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
```

### `static resolved(gs: GameState, ref: String) -> Dictionary`
```logic
var entry: Dictionary = _entry(gs, ref)
return QuestRules.resolve(gs, entry) if _runnable(entry) else {}
```

### `static current_objective(gs: GameState, ref: String) -> Dictionary`
```logic
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
```

### `static picks_allowed(gs: GameState, ref: String) -> int`
```logic
return mini(DictUtil.to_int(resolved(gs, ref).get("loot_picks", 0)), offers(gs, ref).size())   # resolve() fills the schema default; {} = invalid
```

### `static _advance(gs: GameState, qid: String) -> void`
```logic
var stored: Dictionary = _stored(gs, qid)
gs.quests[qid] = {"state": "active", "objective": DictUtil.to_int(stored.get("objective", 0)) + 1}
```

### `static set_flag(gs: GameState, key: String, value: Variant) -> String`
```logic
var err: String = gs.set_flag(key, value)
if err.is_empty():
	refresh(gs)
return err
```

### `static add_flag(gs: GameState, key: String, delta: int, lo: Variant = null, hi: Variant = null) -> String`
```logic
var res: Dictionary = QuestRules.added_flag(gs, key, delta, lo, hi)
if not str(res["error"]).is_empty():
	return str(res["error"])
return set_flag(gs, key, res["value"])
```

### `static complete(gs: GameState, ref: String, recipient_id: String = "", picks: Array[String] = []) -> Dictionary`
```logic
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
```

