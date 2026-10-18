---
class: ReputationTestSuite
extends: RefCounted
description: "Tactical RPG - Reputation Test Suite (S11, spec REP1-L) Schema (checks 1-4): condition operators in the shared grammar (QuestRules.op_errors), the rep value rule and the rep key lint (RepRules, also in WorldCatalog.world_warnings), the dialogue add_flag effect, the world quest field `reputation` and its faction rename. Runtime (checks 5-13): operator evaluation, the shared grammar (quests, variants, markers), add_flag and its clamps, quest completion applying reputation, bands, shop price factors, hostile closure, no ripple, saves; the dialogue and shop panels show the band and the rep prefix. Fixtures: a content fork (one test-only CONSUMABLE of value 40), a sandbox quest template file and save folder, and a hand-built world in an EMPTY sandbox world root; the shipped data is only read."
---

# Properties
* `TAG`: String = "Reputation" (const)
* `W`: String = "rp_base" (const)
* `FA`: String = "f_wardens" (const)
* `FB`: String = "f_hollow" (const)
* `FC`: String = "f_ash"             # FB's enemy; FB is FA's ally (relations only, no ripple) (const)
* `ITEM40`: String = "qa_rep_value40" (const)
* `FIXED`: int = 33 (const)

# Methods

### `static _report(label: String, checks: Dictionary) -> bool`
```logic
return TestHelpers.report(TAG, label, checks)
```

### `static _one(errors: Array[String], needle: String) -> bool`
```logic
if errors.size() != 1 or not errors[0].contains(needle):
	print("      [%s] expected one '%s', got %s" % [TAG, needle, str(errors)])
	return false
return true
```

### `static _none(errors: Array[String]) -> bool`
```logic
if not errors.is_empty():
	print("      [%s] expected no problem, got %s" % [TAG, str(errors)])
	return false
return true
```

### `static _rep_key(local: String) -> String`
```logic
return RepRules.key(WorldCatalog.qid(W, local))
```

### `static _quest(extra: Dictionary) -> Dictionary`
```logic
var q: Dictionary = {"id": "q_rep", "name": "Rep quest", "template": "t_flag", "steps": [{"text": "Raise it", "flag": "rp.signal"}]}
q.merge(extra, true)
return q
```

### `static _tree(effects: Array, requires: Array = []) -> Dictionary`
```logic
return {"id": "d_rep", "name": "Rep talk", "start": "hello", "services": false, "nodes": [
	{"id": "hello", "text": "Hello.", "choices": [{"text": "Go", "to": "", "requires": requires, "effects": effects}]},
]}
```

### `static run(_tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING REPUTATION TEST (REP1-L)                ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
var ok: bool = true
ok = _test_shipped() and ok
TestHelpers.remove_sandbox(SANDBOX)
var forked: bool = TestHelpers.fork(SANDBOX + "/catalog")
DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
QuestCatalog.use_path(SANDBOX + "/quests.json")
WorldCatalog.use_root(SANDBOX + "/worlds")
var sandboxed: bool = forked and QuestCatalog.file_path().begins_with("user://") and WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty()
ok = _report("Sandbox (content fork, empty quest file and world root under user://)", {"sandboxed": sandboxed}) and ok
if sandboxed:
	if _build_world():
		ok = _test_operators() and ok
		ok = _test_rep_values() and ok
		ok = _test_add_flag() and ok
		ok = _test_quest_reputation() and ok
		SaveGame.set_base_dir(SANDBOX + "/saves")
		if _build_runtime():
			ok = _test_evaluation() and ok
			ok = _test_shared_grammar() and ok
			ok = _test_add_flag_runtime() and ok
			ok = _test_completion() and ok
			ok = _test_bands() and ok
			ok = _test_prices() and ok
			ok = _test_hostile() and ok
			ok = _test_no_ripple() and ok
			ok = _test_saves() and ok
			ok = _test_panels() and ok
		else:
			ok = false
		SaveGame.reset_base_dir()
	else:
		ok = false
QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
TestHelpers.cleanup(SANDBOX + "/catalog")
TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox
return ok
```

### `static _test_shipped() -> bool`
```logic
var checks: Dictionary = {}
checks["shipped worlds validate"] = WorldCatalog.validate_all().is_empty()
checks["quest reputation default {}"] = DictUtil.dict_of(WorldSchema.defaults("quests").get("reputation", null)).is_empty() and WorldSchema.defaults("quests").get("reputation", null) is Dictionary
checks["reputation is not a variant field"] = not QuestRules.VARIANT_FIELDS.has("reputation")
var ops: Array[String] = ["==", "!=", ">=", "<=", ">", "<"]
checks["ops"] = QuestRules.OPS == ops
checks["rep key"] = RepRules.key("ashlands:ember_wardens") == "rep.ashlands.ember_wardens" and GameState.flag_key_ok(RepRules.key("ashlands:ember_wardens"))
checks["rep key round trip"] = RepRules.faction_qid_of(RepRules.key("a_b:c_d")) == "a_b:c_d" and RepRules.faction_qid_of("rep.x").is_empty()
# Check 13 (part): no save version bump; the newest committed fixture still decodes against the shipped worlds.
checks["save version unchanged"] = SaveGame.VERSION >= 5   # REP1 added no step; 6 since TRAV2-L
var v5: Dictionary = SaveGame.decode(FileAccess.get_file_as_string(SaveCompatTestSuite.fixture_path(SaveGame.VERSION)))
checks["v5 save loads unchanged"] = DictUtil.to_bool(v5["ok"]) and GameState.of(v5["state"]) != null
return _report("Shipped data validates with the reputation field; constants", checks)
```

### `static _build_world() -> bool`
```logic
var errors: Array[String] = []
errors.append_array(QuestCatalog.upsert({"id": "t_flag", "name": "Flag", "objectives": [{"kind": "flag"}], "rewards": {"xp": 5}}))
errors.append_array(WorldCatalog.create_world(W, "Reputation QA"))
errors.append_array(WorldCatalog.upsert(W, "factions", {"id": FA, "name": "Wardens"}))
errors.append_array(WorldCatalog.upsert(W, "factions", {"id": FB, "name": "Hollow"}))
errors.append_array(WorldCatalog.upsert(W, "quests", _quest({})))
var problems: Dictionary = WorldCatalog.validate_world(W)
var ok: bool = errors.is_empty() and problems.is_empty()
if not ok:
	print("      [%s] fixture: %s %s" % [TAG, str(errors), str(problems)])
return _report("Fixture world (two factions, one templated quest) validates", {"built": ok})
```

### `static _test_operators() -> bool`
```logic
var checks: Dictionary = {}
var k: String = _rep_key(FA)
checks["op =~"] = _one(QuestRules.condition_errors([{"key": k, "op": "=~", "value": 1}], "Requires flags"), "Requires flags 1: op '=~' must be one of == != >= <= > <.")
checks[">= with true"] = _one(QuestRules.condition_errors([{"key": "rp.count", "op": ">=", "value": true}], "Requires flags"), "op '>=' needs a whole-number value.")
checks["< with \"10\""] = _one(QuestRules.condition_errors([{"key": "rp.count", "op": "<", "value": "10"}], "Requires flags"), "op '<' needs a whole-number value.")
checks["missing value with >"] = _one(QuestRules.condition_errors([{"key": "rp.count", "op": ">"}], "Requires flags"), "needs a whole-number value")
checks["op not a String"] = _one(QuestRules.condition_errors([{"key": "rp.count", "op": 3, "value": 1}], "Requires flags"), "must be one of")
checks["extra key"] = _one(QuestRules.condition_errors([{"key": "rp.count", "op": "==", "value": 1, "bound": 2}], "Requires flags"), "unknown key 'bound'")
checks["!= with a String"] = _none(QuestRules.condition_errors([{"key": "rp.name", "op": "!=", "value": "mara"}], "Requires flags"))
var every: Array = []
for op: String in QuestRules.OPS:
	every.append({"key": "rp.count", "op": op, "value": 3})
checks["every op with an int"] = _none(QuestRules.condition_errors(every, "Requires flags"))
checks["legacy conditions unchanged"] = _none(QuestRules.condition_errors([{"key": "rp.a"}, {"key": "rp.b", "value": false}, {"key": "rp.c", "value": "x"}, {"key": "rp.d", "value": 2}], "Requires flags"))
# Every list of the shared grammar accepts op: quest requires, variants, loot, markers, routes, gone flags, shops.
checks["quest requires_flags op"] = _none(_errs("quests", _quest({"requires_flags": [{"key": k, "op": ">=", "value": 10}]})))
checks["quest requires_flags bad op"] = _one(_errs("quests", _quest({"requires_flags": [{"key": k, "op": "=>", "value": 10}]})), "op '=>'")
checks["variant when_flags op"] = _none(_errs("quests", _quest({"variants": [{"id": "v_low", "when_flags": [{"key": k, "op": "<=", "value": -10}], "set": {"description": "Cold."}}]})))
checks["gone_flags op"] = _one(_errs("characters", {"id": "c_x", "name": "X", "base_unit": ContentCatalog.ids("units")[0], "gone_flags": [{"key": k, "op": "~", "value": 1}]}), "op '~'")
checks["loot requires_flags op"] = _one(QuestRules.loot_entry_errors({"reward_kind": "item", "ref": ContentCatalog.ids("items")[0], "count": 1, "requires_flags": [{"key": k, "op": ">", "value": "x"}]}), "needs a whole-number value")
# Dialogue flag conditions delegate to QuestRules.
checks["dialogue flag op ok"] = _none(DialogueRules.conditions_errors(W, [{"kind": "flag", "key": k, "op": ">=", "value": 10}], "Choice requires"))
checks["dialogue flag op bad"] = _one(DialogueRules.conditions_errors(W, [{"key": k, "op": "=~", "value": 10}], "Choice requires"), "op '=~' must be one of")
checks["dialogue flag op needs int"] = _one(DialogueRules.conditions_errors(W, [{"key": "rp.count", "op": "<", "value": "10"}], "Choice requires"), "op '<' needs a whole-number value.")
return _report("Condition operators: validation in every condition list (check 1)", checks)
```

### `static _test_rep_values() -> bool`
```logic
var checks: Dictionary = {}
var k: String = _rep_key(FA)
var msg: String = "Reputation '%s' must be a whole number from -100 to 100." % k
checks["complete_flags 101"] = _one(_errs("quests", _quest({"complete_flags": [{"key": k, "value": 101}]})), msg)
checks["complete_flags \"high\""] = _one(_errs("quests", _quest({"complete_flags": [{"key": k, "value": "high"}]})), msg)
checks["complete_flags without value"] = _one(_errs("quests", _quest({"complete_flags": [{"key": k}]})), msg)
checks["complete_flags -100 and 100"] = _none(_errs("quests", _quest({"complete_flags": [{"key": k, "value": -100}, {"key": _rep_key(FB), "value": 100}]})))
checks["non-rep complete_flags keep any value"] = _none(_errs("quests", _quest({"complete_flags": [{"key": "rp.done"}, {"key": "rp.n", "value": 500}]})))
checks["dialogue set_flag -101"] = _one(_errs("dialogues", _tree([{"effect": "set_flag", "key": k, "value": -101}])), msg)
checks["dialogue set_flag 20"] = _none(_errs("dialogues", _tree([{"effect": "set_flag", "key": k, "value": 20}])))
checks["condition rep >= 150"] = _one(QuestRules.condition_errors([{"key": k, "op": ">=", "value": 150}], "Requires flags"), msg)
checks["condition rep >= -100"] = _none(QuestRules.condition_errors([{"key": k, "op": ">=", "value": -100}], "Requires flags"))
checks["condition rep == 150 (no ordering op)"] = _none(QuestRules.condition_errors([{"key": k, "value": 150}], "Requires flags"))
checks["condition non-rep >= 150"] = _none(QuestRules.condition_errors([{"key": "rp.count", "op": ">=", "value": 150}], "Requires flags"))
checks["marker sets_flags 101"] = TestHelpers.any_contains(QuestRules.flag_list_errors([{"key": k, "value": 101}], "Marker 'm' sets_flags", "sets_flags"), msg)
checks["requires_flags list is a condition list"] = QuestRules.flag_list_errors([{"key": k, "value": 101}], "Marker 'm' requires_flags", "requires_flags").is_empty()
var loc_errs: Array[String] = MapRules.markers_errors(W, {}, {}, [{"id": "m_set", "kind": "point", "anchor": "a", "sets_flags": [{"key": k, "value": 101}]}])
checks["marker sets_flags in location validation"] = TestHelpers.any_contains(loc_errs, msg)
var route_errs: Array[String] = TravelRules.route_errors(W, WorldSchema.normalize("routes", {"id": "r_x", "name": "X", "scripted": [{"id": "s_set", "encounter": "e_none", "sets_flags": [{"key": k, "value": 101}]}]}))
checks["scripted sets_flags in route validation"] = TestHelpers.any_contains(route_errs, msg)
checks["dialogue condition rep < 101"] = _one(DialogueRules.conditions_errors(W, [{"key": k, "op": "<", "value": 101}], "Choice requires"), msg)
# The lint: a rep key naming no faction of the world is a warning, never an error.
var stray: String = "rep.%s.f_gone" % W
var before: int = RepRules.key_warnings(W).size()
var q_stray: Dictionary = _quest({"id": "q_stray", "requires_flags": [{"key": stray, "op": ">=", "value": 10}], "complete_flags": [{"key": k, "value": 5}]})
checks["stray key is no error"] = _none(_errs("quests", q_stray))
var stored: bool = WorldCatalog.upsert(W, "quests", q_stray).is_empty()
var warned: Array[String] = RepRules.key_warnings(W)
checks["no warnings before"] = before == 0
checks["stray key warns once"] = stored and warned.size() == 1 and warned[0].contains("Reputation key '%s' names no faction of this world." % stray) and warned[0].begins_with("%s:q_stray" % W)
checks["world warnings carry it"] = TestHelpers.any_contains(WorldCatalog.world_warnings(W), "Reputation key '%s'" % stray) and TestHelpers.any_contains(JsonUtil.strings(WorldCatalog.warnings_all().get(W, [])), "Reputation key '%s'" % stray)
checks["foreign world key warns"] = RepRules.key_resolves(W, k) and not RepRules.key_resolves(W, "rep.nowhere.f_wardens") and not RepRules.key_resolves(W, "rep.%s" % W)
checks["fixture restored"] = WorldCatalog.remove(W, "quests", "q_stray").is_empty() and RepRules.key_warnings(W).is_empty()
return _report("Rep values -100..100 where flags are set or bounded; unknown faction keys warn (check 2)", checks)
```

### `static _test_add_flag() -> bool`
```logic
var checks: Dictionary = {}
var k: String = _rep_key(FA)
checks["valid in a choice"] = _none(_errs("dialogues", _tree([{"effect": "add_flag", "key": k, "delta": 15}, {"effect": "add_flag", "key": "rp.count", "delta": -3, "min": 0, "max": 9}])))
var node_tree: Dictionary = _tree([])
DictUtil.dict_of(DictUtil.array_of(node_tree["nodes"])[0])["effects"] = [{"effect": "add_flag", "key": k, "delta": 5}]
checks["valid as a node effect"] = _none(_errs("dialogues", node_tree))
checks["delta 0"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": k, "delta": 0}])), "delta must be a whole number from -100 to 100, not 0.")
checks["delta 101"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": k, "delta": 101}])), "delta must be a whole number from -100 to 100, not 0.")
checks["delta missing"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": k}])), "delta must be")
checks["min 5 > max 1"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": "rp.count", "delta": 1, "min": 5, "max": 1}])), "min must not be above max.")
checks["min not a number"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": "rp.count", "delta": 1, "min": "low"}])), "min must be a whole number.")
checks["bad key"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": "Rep Bad", "delta": 1}])), "flag key 'Rep Bad' must be non-empty")
checks["unknown key"] = _one(_errs("dialogues", _tree([{"effect": "add_flag", "key": k, "delta": 1, "step": 2}])), "unknown key 'step'")
checks["may follow turn_in"] = DialogueRules.effects_errors(W, [{"effect": "turn_in", "quest": "q_rep"}, {"effect": "add_flag", "key": k, "delta": 5}], "Choice", false).is_empty()
return _report("Dialogue add_flag effect validation (check 3)", checks)
```

### `static _test_quest_reputation() -> bool`
```logic
var checks: Dictionary = {}
checks["valid"] = _none(_errs("quests", _quest({"reputation": {FA: 10, "%s:%s" % [W, FB]: -50}})))
checks["unknown faction"] = _one(_errs("quests", _quest({"reputation": {"f_none": 5}})), "Reputation: unknown faction 'f_none'.")
checks["delta 0"] = _one(_errs("quests", _quest({"reputation": {FA: 0}})), "delta must be a whole number from -50 to 50, not 0.")
checks["delta 51"] = _one(_errs("quests", _quest({"reputation": {FA: 51}})), "delta must be a whole number from -50 to 50, not 0.")
checks["delta \"5\""] = _one(_errs("quests", _quest({"reputation": {FA: "5"}})), "delta must be")
var five: Dictionary = {}
var extra_errs: Array[String] = []
for i: int in range(5):
	var fid: String = "f_extra_%d" % i
	extra_errs.append_array(WorldCatalog.upsert(W, "factions", {"id": fid, "name": "Extra %d" % i}))
	five[fid] = 1
checks["5 keys"] = extra_errs.is_empty() and _one(_errs("quests", _quest({"reputation": five})), "Reputation lists at most 4 factions (has 5).")
var removed: String = ""
for i: int in range(5):
	removed += WorldCatalog.remove(W, "factions", "f_extra_%d" % i)
checks["extra factions removed"] = removed.is_empty()
checks["no template"] = _one(_errs("quests", {"id": "q_lore", "name": "Lore", "reputation": {FA: 5}}), "Reputation needs a template.")
checks["not an object (blocking)"] = TestHelpers.any_contains(WorldCatalog.upsert(W, "quests", _quest({"id": "q_bad", "reputation": [FA]})), "Reputation must be an object")
checks["variants can't set it"] = _one(_errs("quests", _quest({"variants": [{"id": "v", "when_flags": [{"key": "rp.x"}], "set": {"reputation": {FA: 5}}}]})), "'reputation' can't be overridden")
# A faction rename rewrites the quest's reputation key; a referenced faction can't be removed.
var stored: bool = WorldCatalog.upsert(W, "quests", _quest({"reputation": {FA: 10, FB: -5}})).is_empty()
var refs: Array[Dictionary] = WorldCatalog.references_to(W, "factions", FA)
checks["reputation key is a reference"] = stored and refs.any(func(r: Dictionary) -> bool: return str(r.get("key", "")) == "reputation" and str(r.get("id", "")) == "q_rep")
checks["referenced faction can't be removed"] = not WorldCatalog.remove(W, "factions", FA).is_empty()
var renamed: bool = WorldCatalog.upsert(W, "factions", {"id": "f_wardens2", "name": "Wardens"}, FA).is_empty()
var rep: Dictionary = DictUtil.dict_of(WorldCatalog.get_own(W, "quests", "q_rep").get("reputation", {}))
checks["rename rewrites the key"] = renamed and rep.has("f_wardens2") and not rep.has(FA) and DictUtil.to_int(rep["f_wardens2"]) == 10 and DictUtil.to_int(rep.get(FB, 0)) == -5
checks["renamed world validates"] = WorldCatalog.validate_world(W).is_empty()
var back: bool = WorldCatalog.upsert(W, "factions", {"id": FA, "name": "Wardens"}, "f_wardens2").is_empty()
checks["rename back"] = back and DictUtil.dict_of(WorldCatalog.get_own(W, "quests", "q_rep").get("reputation", {})).has(FA)
checks["fixture restored"] = WorldCatalog.upsert(W, "quests", _quest({})).is_empty() and WorldCatalog.validate_world(W).is_empty()
return _report("World quest reputation field: validation, references, rename (check 4)", checks)
```

### `static _keeper_tree() -> Dictionary`
```logic
return {"id": "d_keeper", "name": "Keeper talk", "start": "hello", "nodes": [
	{"id": "hello", "text": "Welcome.", "choices": [
		{"text": "Praise", "to": "hello", "effects": [{"effect": "add_flag", "key": _rep_key(FA), "delta": 15}]},
		{"text": "Count", "to": "hello", "effects": [{"effect": "add_flag", "key": "rp.count", "delta": 5, "max": 12}]},
		{"text": "Mixed", "to": "hello", "effects": [{"effect": "give", "gold": 5}, {"effect": "add_flag", "key": "rp.word", "delta": 1}]},
		{"text": "Trusted", "to": "hello", "show_locked": true, "requires": [{"key": _rep_key(FA), "op": ">=", "value": 10}]},
	]},
]}
```

### `static _build_runtime() -> bool`
```logic
for e: Dictionary in ContentCatalog.entries("units"):
	if _unit.is_empty() and str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null:
		_unit = str(e["id"])
var errors: Array[String] = TestHelpers.add_item_of_type(ITEM40, "CONSUMABLE", 40, {"heal": 10})
var fixed_item: String = ""
for row: Dictionary in ContentCatalog.entries("items"):
	if fixed_item.is_empty() and str(row.get("item_type", "")) == "CONSUMABLE" and str(row["id"]) != ITEM40:
		fixed_item = str(row["id"])
errors.append_array(WorldCatalog.upsert(W, "factions", {"id": FC, "name": "Ash"}))
errors.append_array(WorldCatalog.upsert(W, "factions", {"id": FB, "name": "Hollow", "relations": {FA: "ally", FC: "enemy"}}))
errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "keeper", "name": "Keeper", "base_unit": _unit, "faction": FA, "dialogue": "d_keeper"}))
errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "loner", "name": "Loner", "base_unit": _unit}))
errors.append_array(WorldCatalog.upsert(W, "dialogues", _keeper_tree()))
errors.append_array(WorldCatalog.upsert(W, "shops", {"id": "s_keep", "name": "Keeper's stall", "keeper": "keeper", "buy_mult": 1.0, "sell_mult": 0.5, "buys": ["CONSUMABLE"],
	"stock": [{"item": ITEM40, "count": -1, "price": -1, "requires_flags": []}, {"item": fixed_item, "count": -1, "price": FIXED, "requires_flags": []}]}))
errors.append_array(WorldCatalog.upsert(W, "shops", {"id": "s_free", "name": "Loner's stall", "keeper": "loner", "buy_mult": 1.0, "sell_mult": 0.5, "buys": ["CONSUMABLE"],
	"stock": [{"item": ITEM40, "count": -1, "price": -1, "requires_flags": []}]}))
errors.append_array(WorldCatalog.upsert(W, "quests", _quest({"id": "q_gain", "steps": [{"text": "Go", "flag": "rp.go"}], "complete_flags": [{"key": "rp.done"}, {"key": _rep_key(FA), "value": 5}], "reputation": {FA: 15, FB: -10}})))
errors.append_array(WorldCatalog.upsert(W, "quests", _quest({"id": "q_gated", "steps": [{"text": "Wait", "flag": "rp.never"}], "requires_flags": [{"key": _rep_key(FA), "op": ">=", "value": 20}]})))
errors.append_array(WorldCatalog.upsert(W, "quests", _quest({"id": "q_auto", "auto_start": true, "steps": [{"text": "Wait", "flag": "rp.never2"}], "requires_flags": [{"key": _rep_key(FA), "op": ">=", "value": 20}]})))
errors.append_array(WorldCatalog.upsert(W, "quests", _quest({"id": "q_ten", "steps": [{"text": "Wait", "flag": "rp.never3"}], "requires_flags": [{"key": _rep_key(FB), "op": ">=", "value": 10}],
	"variants": [{"id": "v_cold", "when_flags": [{"key": _rep_key(FB), "op": "<=", "value": -10}], "set": {"description": "Cold."}}]})))
var problems: Dictionary = WorldCatalog.validate_world(W)
var ok: bool = not _unit.is_empty() and not fixed_item.is_empty() and errors.is_empty() and problems.is_empty()
if not ok:
	print("      [%s] runtime fixture: unit '%s' item '%s' %s %s" % [TAG, _unit, fixed_item, str(errors), str(problems)])
return _report("Runtime fixtures (keeper with a faction, a keeper without one, rep-gated quests, a shop)", {"built": ok})
```

### `static _gs() -> GameState`
```logic
var gs: GameState = GameState.of(NewGame.create(W, [_unit] as Array[String], 7)["state"])
gs.party.gold = 1000
return gs
```

### `static _cond(op: String, value: Variant, flag_key: String = "rp.v") -> Dictionary`
```logic
return {"key": flag_key, "op": op, "value": value}
```

### `static _test_evaluation() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
checks["missing flag >= 0 holds"] = QuestRules.condition_met(gs, _cond(">=", 0))
checks["missing flag > 0 fails"] = not QuestRules.condition_met(gs, _cond(">", 0))
checks["missing flag < 1 holds"] = QuestRules.condition_met(gs, _cond("<", 1))
gs.set_flag("rp.v", 7)
checks["7 >= 7, <= 7, > 6, < 8"] = QuestRules.condition_met(gs, _cond(">=", 7)) and QuestRules.condition_met(gs, _cond("<=", 7)) and QuestRules.condition_met(gs, _cond(">", 6)) and QuestRules.condition_met(gs, _cond("<", 8))
checks["7 > 7 and < 7 fail"] = not QuestRules.condition_met(gs, _cond(">", 7)) and not QuestRules.condition_met(gs, _cond("<", 7))
gs.set_flag("rp.v", "9")
checks["String flag with >= fails"] = not QuestRules.condition_met(gs, _cond(">=", 0)) and not QuestRules.condition_met(gs, _cond("<=", 100))
gs.set_flag("rp.v", true)
checks["bool flag with < fails"] = not QuestRules.condition_met(gs, _cond("<", 5))
var mirror: bool = true
for have: Variant in [null, true, false, 1, 0, "a", ""]:
	if have == null:
		gs.clear_flag("rp.v")
	else:
		gs.set_flag("rp.v", have)
	for want: Variant in [true, false, 1, 0, "a"]:
		mirror = mirror and QuestRules.condition_met(gs, _cond("!=", want)) == not QuestRules.condition_met(gs, _cond("==", want))
		mirror = mirror and QuestRules.condition_met(gs, {"key": "rp.v", "value": want}) == QuestRules.condition_met(gs, _cond("==", want))
checks["!= mirrors == (and == is the default) for every type"] = mirror
gs.set_flag("rp.v", 1)
checks["1 == true still fails"] = not QuestRules.condition_met(gs, _cond("==", true)) and QuestRules.condition_met(gs, _cond("!=", true))
return _report("Operator evaluation (check 5)", checks)
```

### `static _test_shared_grammar() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var kb: String = _rep_key(FB)
gs.set_flag(kb, 9)
checks["quest locked at 9"] = QuestRunner.status(gs, "q_ten") == "locked"
gs.set_flag(kb, 10)
checks["available at 10"] = QuestRunner.status(gs, "q_ten") == "available"
gs.set_flag(kb, -9)
checks["variant off at -9"] = not JsonUtil.strings(QuestRunner.resolved(gs, "q_ten").get("applied_variants", [])).has("v_cold")
gs.set_flag(kb, -10)
var q: Dictionary = QuestRunner.resolved(gs, "q_ten")
checks["variant applies at -10"] = JsonUtil.strings(q.get("applied_variants", [])).has("v_cold") and str(q.get("description", "")) == "Cold."
var marker: Dictionary = {"id": "m_rep", "kind": "point", "anchor": "a", "unless_flags": [{"key": kb, "op": "<", "value": 0}]}
gs.set_flag(kb, 0)
checks["marker present at 0"] = ExploreRunner.presence(gs, WorldCatalog.qid(W, "l_any"), marker) == "present"
gs.set_flag(kb, -1)
checks["marker gone at -1"] = ExploreRunner.presence(gs, WorldCatalog.qid(W, "l_any"), marker) == "absent"
return _report("Shared grammar: quests, variants and markers gate on rep (check 6)", checks)
```

### `static _test_add_flag_runtime() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var ka: String = _rep_key(FA)
var r: DialogueRunner = DialogueRunner.open(gs, "keeper")
checks["opened"] = r.errors.is_empty()
checks["from missing to +15"] = DictUtil.to_bool(r.choose(_find(r.view(), "Praise"))["ok"]) and gs.get_flag(ka, null) is int and DictUtil.to_int(gs.get_flag(ka, 0)) == 15
var a: bool = DictUtil.to_bool(r.choose(_find(r.view(), "Count"))["ok"])
var b: bool = DictUtil.to_bool(r.choose(_find(r.view(), "Count"))["ok"])
checks["clamps at max"] = a and b and DictUtil.to_int(gs.get_flag("rp.count", 0)) == 10 and DictUtil.to_bool(r.choose(_find(r.view(), "Count"))["ok"]) and DictUtil.to_int(gs.get_flag("rp.count", 0)) == 12
gs.set_flag(ka, 95)
checks["rep clamps at 100"] = QuestRunner.add_flag(gs, ka, 20).is_empty() and DictUtil.to_int(gs.get_flag(ka, 0)) == RepRules.MAX
gs.set_flag(ka, -95)
checks["rep clamps at -100"] = QuestRunner.add_flag(gs, ka, -20).is_empty() and DictUtil.to_int(gs.get_flag(ka, 0)) == RepRules.MIN
checks["a non-rep key is not clamped to 100"] = QuestRunner.add_flag(gs, "rp.big", 100).is_empty() and QuestRunner.add_flag(gs, "rp.big", 100).is_empty() and DictUtil.to_int(gs.get_flag("rp.big", 0)) == 200
gs.set_flag(ka, 0)
gs.set_flag("rp.word", "hello")
var gold0: int = gs.party.gold
var refused: Dictionary = r.choose(_find(r.view(), "Mixed"))
checks["non-int flag refuses the whole choice"] = not DictUtil.to_bool(refused["ok"]) and TestHelpers.any_contains(JsonUtil.strings(refused["errors"]), "Flag 'rp.word' is not a number.") and gs.party.gold == gold0 and str(gs.get_flag("rp.word", "")) == "hello"
checks["QuestRunner.add_flag refuses too"] = QuestRunner.add_flag(gs, "rp.word", 1) == "Flag 'rp.word' is not a number."
return _report("add_flag: adds, clamps, refuses non-numbers all-or-nothing (check 7)", checks)
```

### `static _find(v: Dictionary, needle: String) -> int`
```logic
var choices: Array = DictUtil.array_of(v.get("choices", []))
for i: int in range(choices.size()):
	if str(DictUtil.dict_of(choices[i]).get("text", "")).contains(needle):
		return i
return -1
```

### `static _test_completion() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var ka: String = _rep_key(FA)
var kb: String = _rep_key(FB)
var started: bool = QuestRunner.start(gs, "q_gain").is_empty() and QuestRunner.set_flag(gs, "rp.go", true).is_empty()
checks["ready"] = started and QuestRunner.status(gs, "q_gain") == "ready"
checks["gated quests locked before"] = QuestRunner.status(gs, "q_gated") == "locked" and QuestRunner.status(gs, "q_auto") == "locked"
var res: Dictionary = QuestRunner.complete(gs, "q_gain")
# complete_flags set rep FA to 5 first, then +15 lands on it: 20 (not 15, not 5).
checks["completed"] = DictUtil.to_bool(res["ok"]) and QuestRunner.status(gs, "q_gain") == "completed" and gs.get_flag("rp.done", false) == true
checks["delta after complete_flags"] = DictUtil.to_int(gs.get_flag(ka, 0)) == 20 and DictUtil.to_int(gs.get_flag(kb, 0)) == -10
var echoed: Dictionary = DictUtil.dict_of(DictUtil.dict_of(res["granted"]).get("reputation", {}))
checks["granted.reputation echoes the change"] = DictUtil.to_int(echoed.get(WorldCatalog.qid(W, FA), 0)) == 15 and DictUtil.to_int(echoed.get(WorldCatalog.qid(W, FB), 0)) == -10 and echoed.size() == 2
checks["gated quest available in the same call"] = QuestRunner.status(gs, "q_gated") == "available"
checks["auto_start quest started by the one refresh"] = QuestRunner.status(gs, "q_auto") == "active"
var again: Dictionary = QuestRunner.complete(gs, "q_gain")
checks["second complete changes nothing"] = not DictUtil.to_bool(again["ok"]) and DictUtil.to_int(gs.get_flag(ka, 0)) == 20 and DictUtil.to_int(gs.get_flag(kb, 0)) == -10
var clamp_gs: GameState = _gs()
var _s: Array[String] = QuestRunner.start(clamp_gs, "q_gain")
var _f: String = QuestRunner.set_flag(clamp_gs, "rp.go", true)
clamp_gs.set_flag(kb, -95)
var clamped: Dictionary = QuestRunner.complete(clamp_gs, "q_gain")
checks["rep clamp on completion, echo after the clamp"] = DictUtil.to_bool(clamped["ok"]) and DictUtil.to_int(clamp_gs.get_flag(kb, 0)) == RepRules.MIN and DictUtil.to_int(DictUtil.dict_of(DictUtil.dict_of(clamped["granted"])["reputation"]).get(WorldCatalog.qid(W, FB), 0)) == -5
return _report("Quest completion applies reputation once (check 8)", checks)
```

### `static _test_bands() -> bool`
```logic
var checks: Dictionary = {}
var want: Dictionary = {-100: "hostile", -50: "hostile", -49: "unfriendly", -10: "unfriendly", -9: "neutral", 0: "neutral", 9: "neutral", 10: "friendly", 49: "friendly", 50: "honoured", 100: "honoured"}
for v: Variant in want:
	checks["%d is %s" % [DictUtil.to_int(v), str(want[v])]] = RepRules.band(DictUtil.to_int(v)) == str(want[v])
var gs: GameState = _gs()
checks["standing reads 0 when missing"] = RepRules.standing(gs, FA) == 0
gs.set_flag(_rep_key(FA), 33)
checks["standing reads the flag by ref or qid"] = RepRules.standing(gs, FA) == 33 and RepRules.standing(gs, WorldCatalog.qid(W, FA)) == 33
return _report("Bands (check 9)", checks)
```

### `static _price_of(gs: GameState, shop: String, item: String) -> int`
```logic
for o: Dictionary in ShopRules.offers(gs, shop):
	if str(o["item"]) == item:
		return DictUtil.to_int(o["price"])
return -1
```

### `static _sell_of(gs: GameState, shop: String, item: String) -> int`
```logic
for r: Dictionary in ShopRules.sell_list(gs, shop):
	if str(r["item"]) == item:
		return DictUtil.to_int(r["price"])
return -1
```

### `static _test_prices() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.add_to_stash(ITEM40, 3)
var ka: String = _rep_key(FA)
var fixed_item: String = str(DictUtil.dict_of(DictUtil.array_of(WorldCatalog.resolve(W, "shops", "s_keep")["stock"])[1])["item"])
# value 40, buy_mult 1.0 / sell_mult 0.5: buy = ceil(40 x 100 x rb / 10000), sell = floor(40 x 50 x rs / 10000).
var rows: Array[Array] = [[-30, "unfriendly", 46, 17], [0, "neutral", 40, 20], [30, "friendly", 38, 21], [70, "honoured", 36, 22]]
for row: Array in rows:
	gs.set_flag(ka, DictUtil.to_int(row[0]))
	var tag: String = str(row[1])
	checks["band " + tag] = ShopRules.keeper_band(gs, WorldCatalog.resolve(W, "shops", "s_keep")) == tag
	checks["buy " + tag] = _price_of(gs, "s_keep", ITEM40) == DictUtil.to_int(row[2])
	checks["sell " + tag] = _sell_of(gs, "s_keep", ITEM40) == DictUtil.to_int(row[3])
	checks["fixed price unchanged " + tag] = _price_of(gs, "s_keep", fixed_item) == FIXED
	checks["keeper without a faction is neutral " + tag] = _price_of(gs, "s_free", ITEM40) == 40 and _sell_of(gs, "s_free", ITEM40) == 20
gs.set_flag(ka, 30)
var gold0: int = gs.party.gold
var bought: Dictionary = ShopRules.buy(gs, "s_keep", "item", ITEM40, 2)
checks["buy charges the band price"] = DictUtil.to_bool(bought["ok"]) and DictUtil.to_int(bought["price"]) == 76 and gs.party.gold == gold0 - 76
var sold: Dictionary = ShopRules.sell(gs, "s_keep", ITEM40, 1)
checks["sell pays the band price"] = DictUtil.to_bool(sold["ok"]) and DictUtil.to_int(sold["price"]) == 21
return _report("Shop price factors by band; fixed prices and faction-less keepers unchanged (check 10)", checks)
```

### `static _test_hostile() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.add_to_stash(ITEM40, 1)
var ka: String = _rep_key(FA)
var shop_line: String = DialogueRules.service_line(W, "shop")
var r: DialogueRunner = DialogueRunner.open(gs, "keeper")
checks["service shown while neutral"] = r.errors.is_empty() and _find(r.view(), shop_line) >= 0
gs.set_flag(ka, -50)
checks["closed"] = not ShopRules.is_open(gs, "s_keep")
checks["no offers, nothing to sell"] = ShopRules.offers(gs, "s_keep").is_empty() and ShopRules.sell_list(gs, "s_keep").is_empty()
checks["buy refused"] = not DictUtil.to_bool(ShopRules.buy(gs, "s_keep", "item", ITEM40, 1)["ok"])
checks["sell refused"] = not DictUtil.to_bool(ShopRules.sell(gs, "s_keep", ITEM40, 1)["ok"])
checks["service choice not generated"] = _find(r.view(), shop_line) < 0
checks["other keepers unaffected"] = ShopRules.is_open(gs, "s_free")
gs.set_flag(ka, -49)
checks["reopens at -49"] = ShopRules.is_open(gs, "s_keep") and _find(r.view(), shop_line) >= 0 and _price_of(gs, "s_keep", ITEM40) == 46
return _report("Hostile keeper: shop closed, service hidden, reopens above -50 (check 11)", checks)
```

### `static _test_no_ripple() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
checks["+30 applied"] = QuestRunner.add_flag(gs, _rep_key(FA), 30).is_empty() and RepRules.standing(gs, FA) == 30
checks["ally and enemy stay at 0"] = RepRules.standing(gs, FB) == 0 and RepRules.standing(gs, FC) == 0 and not gs.has_flag(_rep_key(FB)) and not gs.has_flag(_rep_key(FC))
return _report("No ripple through faction relations (check 12)", checks)
```

### `static _test_saves() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.set_flag(_rep_key(FA), -37)
gs.set_flag(_rep_key(FB), 100)
var written: bool = SaveGame.write(gs, 1).is_empty()
var read: Dictionary = SaveGame.read(1)
var back: GameState = GameState.of(read["state"])
checks["round trip"] = written and DictUtil.to_bool(read["ok"]) and back != null and back.get_flag(_rep_key(FA), null) is int and DictUtil.to_int(back.get_flag(_rep_key(FA), 0)) == -37 and DictUtil.to_int(back.get_flag(_rep_key(FB), 0)) == 100
checks["standing after load"] = back != null and RepRules.standing(back, FA) == -37 and RepRules.band(RepRules.standing(back, FA)) == "unfriendly"
checks["written at the current version"] = SaveGame.VERSION >= 5 and DictUtil.to_int(SaveGame.peek(1).get("version", 0)) == SaveGame.VERSION
return _report("Saves: rep flags round-trip, no version bump (check 13)", checks)
```

### `static _test_panels() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.set_flag(_rep_key(FA), 20)
var r: DialogueRunner = DialogueRunner.open(gs, "keeper")
var v: Dictionary = r.view()
checks["view carries faction and band"] = str(v.get("faction", "")) == WorldCatalog.qid(W, FA) and str(v.get("faction_name", "")) == "Wardens" and str(v.get("band", "")) == "friendly"
checks["rep prefix"] = _find(v, "[Wardens friendly] Trusted") >= 0
checks["no prefix on ungated choices"] = _find(v, "] Praise") < 0 and _find(v, "Praise") >= 0
var loner: DialogueRunner = DialogueRunner.open(gs, "loner")
checks["no band without a faction"] = str(loner.view().get("band", "x")).is_empty()
var dp := DialoguePanel.new()
dp.show_runner(r)
checks["dialogue panel shows the band"] = dp.speaker_text().contains(RepRules.standing_text("Wardens", "friendly"))
dp.free()
var sp := ShopPanel.new()
sp.show_shop(gs, WorldCatalog.qid(W, "s_keep"))
checks["shop panel shows the band"] = sp.standing_text() == RepRules.standing_text("Wardens", "friendly")
sp.show_shop(gs, WorldCatalog.qid(W, "s_free"))
checks["no band line without a faction"] = sp.standing_text().is_empty()
sp.free()
gs.set_flag(_rep_key(FA), 5)
checks["locked below the bound, prefix kept"] = _find(r.view(), "[Wardens friendly] Trusted") >= 0 and DictUtil.to_bool(DictUtil.dict_of(DictUtil.array_of(r.view()["choices"])[_find(r.view(), "Trusted")]).get("locked", false))
return _report("Display: band in the dialogue and shop panels, rep choice prefix (REP1-L 2.4)", checks)
```

