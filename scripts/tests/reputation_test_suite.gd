class_name ReputationTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Reputation Test Suite (S11, spec REP1-L)
# Schema (checks 1-4): condition operators in the shared grammar (QuestRules.op_errors), the rep value rule and the
# rep key lint (RepRules), the dialogue add_flag effect, the world quest field `reputation` and its faction rename.
# Fixtures: a content fork, a sandbox quest template file and a hand-built world in an EMPTY sandbox world root; the
# shipped data is only read.
# ==============================================================================

const TAG: String = "Reputation"
static var SANDBOX: String = TestHelpers.sandbox("reputation")
const W: String = "rp_base"
const FA: String = "f_wardens"
const FB: String = "f_hollow"

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _errs(kind: String, entry: Dictionary) -> Array[String]:
	return WorldCatalog.validate_entry(W, kind, entry, str(entry.get("id", "")))

# Exactly one problem, and it contains `needle`.
static func _one(errors: Array[String], needle: String) -> bool:
	if errors.size() != 1 or not errors[0].contains(needle):
		print("      [%s] expected one '%s', got %s" % [TAG, needle, str(errors)])
		return false
	return true

static func _none(errors: Array[String]) -> bool:
	if not errors.is_empty():
		print("      [%s] expected no problem, got %s" % [TAG, str(errors)])
		return false
	return true

static func _rep_key(local: String) -> String:
	return RepRules.key(WorldCatalog.qid(W, local))

# A valid templated quest of the fixture world with `extra` merged in.
static func _quest(extra: Dictionary) -> Dictionary:
	var q: Dictionary = {"id": "q_rep", "name": "Rep quest", "template": "t_flag", "steps": [{"text": "Raise it", "flag": "rp.signal"}]}
	q.merge(extra, true)
	return q

# A one-node dialogue whose choice carries `effects` and `requires`.
static func _tree(effects: Array, requires: Array = []) -> Dictionary:
	return {"id": "d_rep", "name": "Rep talk", "start": "hello", "services": false, "nodes": [
		{"id": "hello", "text": "Hello.", "choices": [{"text": "Go", "to": "", "requires": requires, "effects": effects}]},
	]}

# ------------------------------------------------------------------------------

static func run(_tree_parent: Node = null) -> bool:
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
		else:
			ok = false
	QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	TestHelpers.cleanup(SANDBOX + "/catalog")
	TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox
	return ok

static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["shipped worlds validate"] = WorldCatalog.validate_all().is_empty()
	checks["quest reputation default {}"] = DictUtil.dict_of(WorldSchema.defaults("quests").get("reputation", null)).is_empty() and WorldSchema.defaults("quests").get("reputation", null) is Dictionary
	checks["reputation is not a variant field"] = not QuestRules.VARIANT_FIELDS.has("reputation")
	var ops: Array[String] = ["==", "!=", ">=", "<=", ">", "<"]
	checks["ops"] = QuestRules.OPS == ops
	checks["rep key"] = RepRules.key("ashlands:ember_wardens") == "rep.ashlands.ember_wardens" and GameState.flag_key_ok(RepRules.key("ashlands:ember_wardens"))
	checks["rep key round trip"] = RepRules.faction_qid_of(RepRules.key("a_b:c_d")) == "a_b:c_d" and RepRules.faction_qid_of("rep.x").is_empty()
	return _report("Shipped data validates with the reputation field; constants", checks)

static func _build_world() -> bool:
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

# Check 1: operators in the shared condition grammar.
static func _test_operators() -> bool:
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

# Check 2: the rep value rule and the rep key lint.
static func _test_rep_values() -> bool:
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
	checks["marker sets_flags 101"] = TestHelpers.any_contains(QuestRules.set_flags_errors([{"key": k, "value": 101}], "Marker 'm' sets_flags"), msg)
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
	checks["foreign world key warns"] = RepRules.key_resolves(W, k) and not RepRules.key_resolves(W, "rep.nowhere.f_wardens") and not RepRules.key_resolves(W, "rep.%s" % W)
	checks["fixture restored"] = WorldCatalog.remove(W, "quests", "q_stray").is_empty() and RepRules.key_warnings(W).is_empty()
	return _report("Rep values -100..100 where flags are set or bounded; unknown faction keys warn (check 2)", checks)

# Check 3: the dialogue add_flag effect.
static func _test_add_flag() -> bool:
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

# Check 4: the world quest field `reputation`.
static func _test_quest_reputation() -> bool:
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
