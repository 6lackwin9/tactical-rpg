class_name DialogueTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Dialogue Test Suite (S11, spec DLG1-L)
# Commit 1 (schema): dialogue trees, choice conditions and effects (DialogueRules), shops (ShopRules), the base service
# lines (ServiceCatalog) and manifest service_lines / start_gold, quest reward gold and chest gold, character
# gone_flags, the gone-NPC quest lint, and reference scans / renames through the new json fields (checks 1-6).
# Fixtures: a content fork (one test-only QUEST item), a sandbox quest file and service file, and a hand-built world
# in an EMPTY sandbox world root; the shipped data is only read (ids picked by rule) and never written.
# ==============================================================================

const TAG: String = "Dialogue"
static var SANDBOX: String = TestHelpers.sandbox("dialogue")
const W: String = "dg_base"
const QUEST_ITEM: String = "qa_dlg_quest_token"
const GONE: Dictionary = {"key": "dg.mara_gone", "value": true}

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

# At least one problem contains `needle`.
static func _has(errors: Array[String], needle: String) -> bool:
	if not TestHelpers.any_contains(errors, needle):
		print("      [%s] expected '%s' in %s" % [TAG, needle, str(errors)])
		return false
	return true

# The node `id` of tree `t` (the dictionary itself, so callers can mutate it).
static func _node(t: Dictionary, id: String) -> Dictionary:
	for n: Variant in DictUtil.array_of(t["nodes"]):
		var node: Dictionary = DictUtil.dict_of(n)
		if str(node.get("id", "")) == id:
			return node
	return {}

static func _choices(t: Dictionary, id: String) -> Array:
	return DictUtil.array_of(_node(t, id).get("choices", []))

# ------------------------------------------------------------------------------

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING DIALOGUE TEST (DLG1-L schema)           ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	TestHelpers.restore_services()
	var ok: bool = true
	ok = _test_shipped() and ok
	TestHelpers.remove_sandbox(SANDBOX)
	var forked: bool = TestHelpers.fork(SANDBOX + "/catalog")
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	QuestCatalog.use_path(SANDBOX + "/quests.json")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	var sandboxed: bool = forked and TestHelpers.sandbox_services(SANDBOX) and QuestCatalog.file_path().begins_with("user://") and WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty()
	ok = _report("Sandbox (content fork, empty quest / service files and world root under user://)", {"sandboxed": sandboxed}) and ok
	if sandboxed:
		var fx: Dictionary = _content_fixtures()
		var fx_ok: bool = _report("Content fixtures picked by rule (+ one test-only QUEST item on the fork)", {"two items": JsonUtil.strings(fx["items"]).size() >= 2, "a passive": not str(fx["passive"]).is_empty(), "a unit": not str(fx["unit"]).is_empty(), "quest item": DictUtil.to_bool(fx["quest_item_ok"])})
		ok = fx_ok and ok
		if fx_ok and _build_world(fx):
			ok = _test_trees(fx) and ok
			ok = _test_conditions(fx) and ok
			ok = _test_effects(fx) and ok
			ok = _test_shops(fx) and ok
			ok = _test_gold_gone_services(fx) and ok
			ok = _test_lint() and ok
			ok = _test_ref_scans() and ok
			ok = _test_persistence() and ok
		else:
			ok = false
	TestHelpers.restore_services()
	QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	TestHelpers.cleanup(SANDBOX + "/catalog")
	TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox
	return ok

static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["new kinds"] = WorldSchema.KINDS.has("dialogues") and WorldSchema.KINDS.has("shops")
	checks["shipped worlds validate"] = WorldCatalog.validate_all().is_empty()
	checks["shipped services load"] = ServiceCatalog.load_errors().is_empty() and not ServiceCatalog.save_blocked()
	checks["shipped services validate"] = ServiceCatalog.validate_all().is_empty()
	var known: bool = true
	for id: String in ServiceCatalog.ids():
		known = known and DialogueRules.SERVICE_IDS.has(id)
	checks["shipped service ids known"] = known
	checks["every service id has a default line"] = DialogueRules.DEFAULT_LINES.size() == DialogueRules.SERVICE_IDS.size() and DialogueRules.SERVICE_IDS.all(func(id: String) -> bool: return DialogueRules.DEFAULT_LINES.has(id))
	var default_tokens_ok: bool = true
	for id: Variant in DialogueRules.DEFAULT_LINES:
		default_tokens_ok = default_tokens_ok and DialogueRules.line_errors(str(id), DialogueRules.DEFAULT_LINES[id], "Default").is_empty()
	checks["default lines valid"] = default_tokens_ok
	var m: Dictionary = WorldSchema.defaults("world")
	checks["manifest defaults"] = DictUtil.to_int(m["start_gold"]) == -1 and DictUtil.dict_of(m["service_lines"]).is_empty()
	var c: Dictionary = WorldSchema.defaults("characters")
	checks["character defaults"] = str(c["dialogue"]).is_empty() and DictUtil.array_of(c["gone_flags"]).is_empty()
	var d: Dictionary = WorldSchema.defaults("dialogues")
	checks["dialogue defaults"] = DictUtil.to_bool(d["services"]) and DictUtil.array_of(d["nodes"]).is_empty() and str(d["name"]) == "New Dialogue"
	var s: Dictionary = WorldSchema.defaults("shops")
	checks["shop defaults"] = JsonUtil.strings(s["buys"]) == ShopRules.BUY_TYPES and is_equal_approx(DictUtil.to_float(s["buy_mult"]), 1.0) and is_equal_approx(DictUtil.to_float(s["sell_mult"]), 0.5) and str(s["name"]) == "New Shop"
	checks["chest gold default"] = DictUtil.to_int(DictUtil.dict_of(MapRules.MARKER_KINDS["chest"]).get("gold", -1)) == 0
	checks["gold reward key"] = QuestRules.REWARD_KEYS.has("gold")
	return _report("Shipped data validates with the new kinds and fields; defaults", checks)

# Read-only picks from the shipped catalog (by rule) plus one QUEST item upserted on the fork.
static func _content_fixtures() -> Dictionary:
	var items: Array[String] = []
	for row: Dictionary in ContentCatalog.entries("items"):
		if str(row.get("item_type", "")) != "QUEST" and items.size() < 2:
			items.append(str(row["id"]))
	var passives: Array[String] = ContentCatalog.ids("passives")
	var units: Array[String] = ContentCatalog.ids("units")
	var added: Array[String] = TestHelpers.add_item_of_type(QUEST_ITEM, "QUEST")
	var quest_item_ok: bool = added.is_empty() and str(ContentCatalog.get_entry("items", QUEST_ITEM).get("item_type", "")) == "QUEST"
	if not added.is_empty():
		print("      [%s] quest item: %s" % [TAG, str(added)])
	return {"items": items, "passive": passives[0] if not passives.is_empty() else "", "unit": units[0] if not units.is_empty() else "", "quest_item_ok": quest_item_ok}

# A valid tree using every condition kind, every effect and both node kinds of moves (6 choices = MAX_CHOICES).
static func _tree(fx: Dictionary) -> Dictionary:
	var item0: String = JsonUtil.strings(fx["items"])[0]
	return {"id": "d_mara", "name": "Mara talk", "speaker": "mara", "start": "hello", "nodes": [
		{"id": "hello", "text": "Hello, {npc}.", "choices": [
			{"text": "Tell me", "to": "info", "show_locked": true,
				"requires": [{"kind": "quest", "quest": "q_main", "status": "active"}, {"kind": "items", "item": item0, "count": 1}, {"kind": "gold", "min": 5}, {"kind": "stat", "stat": "KNOW", "min": 14}, {"key": "dg.flag"}],
				"unless": [{"key": "dg.told", "value": true}], "effects": [{"effect": "set_flag", "key": "dg.told"}]},
			{"text": "Trade", "to": "", "effects": [{"effect": "take", "items": {item0: 1}}, {"effect": "give", "gold": 20}]},
			{"text": "Shop", "effects": [{"effect": "open_shop", "shop": "shop_a"}]},
			{"text": "Fight", "effects": [{"effect": "set_flag", "key": "dg.fought"}, {"effect": "fight", "encounter": "enc_a"}]},
			{"text": "Start", "to": "bye", "effects": [{"effect": "start_quest", "quest": "q_main"}]},
			{"text": "Turn in", "effects": [{"effect": "turn_in", "quest": "q_main"}, {"effect": "set_flag", "key": "dg.turned"}, {"effect": "give", "gold": 5}]},
		]},
		{"id": "info", "speaker": "tev", "text": "Info.", "next": "bye", "effects": [{"effect": "give", "items": {item0: 1}}, {"effect": "set_flag", "key": "dg.info", "value": 2}]},
		{"id": "bye", "text": "Bye.", "end": true},
	]}

static func _shop(fx: Dictionary) -> Dictionary:
	var items: Array[String] = JsonUtil.strings(fx["items"])
	return {"id": "shop_a", "name": "Shop A", "keeper": "mara",
		"stock": [{"item": items[0], "count": -1, "price": -1, "requires_flags": []}, {"item": items[1], "count": 2, "price": 40, "requires_flags": [{"key": "dg.open"}]}],
		"loot": [{"loot": "l_item", "price": 150}], "buy_mult": 1.15, "sell_mult": 0.85, "buys": ["CONSUMABLE", "WEAPON"], "requires_flags": []}

static func _build_world(fx: Dictionary) -> bool:
	var unit: String = str(fx["unit"])
	var item0: String = JsonUtil.strings(fx["items"])[0]
	var errors: Array[String] = []
	errors.append_array(QuestCatalog.upsert({"id": "t_flag", "name": "Flag", "objectives": [{"kind": "flag"}], "rewards": {"xp": 5}}))
	errors.append_array(WorldCatalog.create_world(W, "Dialogue QA"))
	errors.append_array(WorldCatalog.upsert(W, "locations", {"id": "loc_a", "name": "Loc A"}))
	errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "hero", "name": "Hero", "role": "party", "base_unit": unit}))
	errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": unit}))
	errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "mara", "name": "Mara", "base_unit": unit, "dialogue": "d_mara"}))
	errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "tev", "name": "Tev", "base_unit": unit}))
	errors.append_array(WorldCatalog.upsert(W, "encounters", {"id": "enc_a", "name": "Enc A", "location": "loc_a", "party": ["hero"], "enemies": ["foe"]}))
	errors.append_array(WorldCatalog.upsert(W, "quests", {"id": "q_main", "name": "Main", "template": "t_flag", "giver": "mara", "steps": [{"text": "Raise the signal", "flag": "dg.signal"}]}))
	errors.append_array(WorldCatalog.upsert(W, "quests", {"id": "q_lore", "name": "Lore only"}))
	errors.append_array(WorldCatalog.upsert(W, "loot", {"id": "l_item", "name": "Item loot", "reward_kind": "item", "ref": item0, "count": 1}))
	errors.append_array(WorldCatalog.upsert(W, "loot", {"id": "l_passive", "name": "Passive loot", "reward_kind": "passive", "ref": str(fx["passive"]), "count": 1}))
	errors.append_array(WorldCatalog.upsert(W, "shops", _shop(fx)))
	errors.append_array(WorldCatalog.upsert(W, "dialogues", _tree(fx)))
	var problems: Dictionary = WorldCatalog.validate_world(W)
	var checks: Dictionary = {}
	checks["fixtures stored"] = errors.is_empty()
	checks["fixture world validates"] = problems.is_empty()
	checks["valid tree has no warnings"] = DialogueRules.tree_warnings(WorldCatalog.get_own(W, "dialogues", "d_mara")).is_empty() and DialogueRules.dialogue_warnings(W).is_empty()
	checks["no lint without gone flags"] = DialogueRules.quest_warnings(W).is_empty()
	if not errors.is_empty() or not problems.is_empty():
		print("      [%s] fixture problems: %s %s" % [TAG, str(errors), str(problems)])
	return _report("Fixture world (every condition kind and effect, a shop, gold-free quest) validates", checks)

# ------------------------------------------------------------------------------
# Check 1: trees
# ------------------------------------------------------------------------------

static func _test_trees(fx: Dictionary) -> bool:
	var checks: Dictionary = {}
	var t: Dictionary = _tree(fx)
	t["start"] = "nope"
	checks["missing start"] = _one(_errs("dialogues", t), "Start node 'nope' does not exist.")
	t = _tree(fx)
	DictUtil.dict_of(_choices(t, "hello")[0])["to"] = "nowhere"
	checks["dangling to"] = _one(_errs("dialogues", t), "to 'nowhere' is not a node")
	t = _tree(fx)
	_node(t, "info")["next"] = "nowhere"
	checks["dangling next"] = _one(_errs("dialogues", t), "next 'nowhere' is not a node")
	t = _tree(fx)
	_node(t, "hello")["end"] = true
	checks["choices and end"] = _one(_errs("dialogues", t), "needs exactly one of choices, next or end")
	t = _tree(fx)
	var many: Array = []
	for i: int in range(DialogueRules.MAX_NODES + 1):
		many.append({"id": "n%d" % i, "text": "Line %d" % i, "end": true})
	t["nodes"] = many
	t["start"] = "n0"
	t["services"] = false
	checks["65 nodes"] = _one(_errs("dialogues", t), "A dialogue holds 1..%d nodes." % DialogueRules.MAX_NODES)
	t["nodes"] = []
	checks["0 nodes"] = _has(_errs("dialogues", t), "A dialogue holds 1..%d nodes." % DialogueRules.MAX_NODES)
	t = _tree(fx)
	_choices(t, "hello").append({"text": "More", "to": "bye"})
	checks["7 choices"] = _one(_errs("dialogues", t), "choices (has %d)" % (DialogueRules.MAX_CHOICES + 1))
	t = _tree(fx)
	_node(t, "bye")["mood"] = "calm"
	checks["unknown node key"] = _one(_errs("dialogues", t), "Node 'bye': unknown key 'mood'.")
	t = _tree(fx)
	_node(t, "bye")["text"] = " "
	checks["empty text"] = _one(_errs("dialogues", t), "needs text")
	t = _tree(fx)
	_node(t, "bye")["text"] = "x".repeat(DialogueRules.MAX_TEXT + 1)
	checks["text too long"] = _one(_errs("dialogues", t), "longer than")
	t = _tree(fx)
	DictUtil.array_of(_node(t, "info")["effects"]).append({"effect": "take", "gold": 1})
	checks["node effect take"] = _one(_errs("dialogues", t), "Node effects may only be set_flag or give.")
	t = _tree(fx)
	t["start"] = "bye"
	checks["services on, end start node"] = _one(_errs("dialogues", t), "With services on, the start node needs choices.")
	t["services"] = false
	checks["services off, end start node ok"] = _errs("dialogues", t).is_empty()
	t = _tree(fx)
	DictUtil.array_of(t["nodes"]).append({"id": "info", "text": "Again.", "end": true})
	checks["duplicate node id"] = _one(_errs("dialogues", t), "Node id 'info' is used twice.")
	t = _tree(fx)
	_node(t, "bye")["speaker"] = "nobody"
	checks["unknown node speaker"] = _one(_errs("dialogues", t), "unknown character 'nobody'")
	t = _tree(fx)
	DictUtil.array_of(t["nodes"]).append({"id": "orphan", "text": "Nobody comes here.", "end": true})
	var warnings: Array[String] = DialogueRules.tree_warnings(t)
	checks["unreachable node warns only"] = _errs("dialogues", t).is_empty() and warnings.size() == 1 and warnings[0].contains("'orphan'")
	return _report("Check 1: trees (one error per bad case, unreachable node warns)", checks)

# ------------------------------------------------------------------------------
# Check 2: conditions
# ------------------------------------------------------------------------------

static func _test_conditions(fx: Dictionary) -> bool:
	var item0: String = JsonUtil.strings(fx["items"])[0]
	var checks: Dictionary = {}
	var valid: Array = [{"kind": "flag", "key": "dg.a", "value": 3}, {"kind": "quest", "quest": "q_main", "status": "ready"}, {"kind": "items", "item": item0}, {"kind": "gold", "min": 0}, {"kind": "stat", "stat": "STR", "min": DialogueRules.STAT_MAX}]
	checks["valid kinds pass"] = DialogueRules.conditions_errors(W, valid, "Requires").is_empty()
	for status: String in DialogueRules.QUEST_STATUSES:
		checks["status " + status] = DialogueRules.conditions_errors(W, [{"kind": "quest", "quest": "q_main", "status": status}], "Requires").is_empty()
	var bad: Dictionary = {
		"unknown kind": [{"kind": "mood", "key": "dg.a"}, "unknown condition kind 'mood'"],
		"status done": [{"kind": "quest", "quest": "q_main", "status": "done"}, "status 'done'"],
		"unknown quest": [{"kind": "quest", "quest": "q_nope", "status": "active"}, "unknown quest 'q_nope'"],
		"unknown item": [{"kind": "items", "item": "no_such_item_qa"}, "unknown base item 'no_such_item_qa'"],
		"count 0": [{"kind": "items", "item": item0, "count": 0}, "count must be a whole number 1..%d" % QuestRules.ITEM_COUNT_MAX],
		"gold without min": [{"kind": "gold"}, "min must be a whole number 0..%d" % DialogueRules.GOLD_CONDITION_MAX],
		"gold above max": [{"kind": "gold", "min": DialogueRules.GOLD_CONDITION_MAX + 1}, "min must be"],
		"stat LUCK": [{"kind": "stat", "stat": "LUCK", "min": 10}, "stat 'LUCK'"],
		"min 31": [{"kind": "stat", "stat": "KNOW", "min": DialogueRules.STAT_MAX + 1}, "min must be a whole number %d..%d" % [DialogueRules.STAT_MIN, DialogueRules.STAT_MAX]],
		"stat min 0": [{"kind": "stat", "stat": "KNOW", "min": 0}, "min must be"],
		"extra key": [{"key": "dg.a", "extra": 1}, "unknown key 'extra'"],
		"bad flag key": [{"key": "Bad Key"}, "flag key 'Bad Key'"],
	}
	for label: Variant in bad:
		var pair: Array = DictUtil.array_of(bad[label])
		checks[str(label)] = _one(DialogueRules.conditions_errors(W, [pair[0]], "Requires"), str(pair[1]))
	checks["not a list"] = _one(DialogueRules.conditions_errors(W, {"key": "dg.a"}, "Requires"), "must be a list")
	# Through a whole tree (choice `unless`).
	var t: Dictionary = _tree(fx)
	DictUtil.dict_of(_choices(t, "hello")[0])["unless"] = [{"kind": "stat", "stat": "LUCK", "min": 10}]
	checks["in a tree"] = _one(_errs("dialogues", t), "stat 'LUCK'")
	return _report("Check 2: conditions (unknown kind, status, item, count, stat, min, extra key: errors)", checks)

# ------------------------------------------------------------------------------
# Check 3: effects
# ------------------------------------------------------------------------------

static func _test_effects(fx: Dictionary) -> bool:
	var item0: String = JsonUtil.strings(fx["items"])[0]
	var turn: Dictionary = {"effect": "turn_in", "quest": "q_main"}
	var fight: Dictionary = {"effect": "fight", "encounter": "enc_a"}
	var shop: Dictionary = {"effect": "open_shop", "shop": "shop_a"}
	var flag: Dictionary = {"effect": "set_flag", "key": "dg.x"}
	var checks: Dictionary = {}
	var bad: Dictionary = {
		"unknown effect": [[{"effect": "dance"}], "unknown effect 'dance'"],
		"fight not last": [[fight, flag], "fight must be the last effect"],
		"open_shop not last": [[shop, flag], "open_shop must be the last effect"],
		"fight + turn_in": [[turn, fight], "fight can't share a choice with turn_in or open_shop"],
		"fight + open_shop": [[shop, fight], "fight can't share a choice"],
		"turn_in second": [[flag, turn], "turn_in must come first; only set_flag or give may follow."],
		"take after turn_in": [[turn, {"effect": "take", "gold": 1}], "turn_in must come first"],
		"give with nothing": [[{"effect": "give"}], "needs items or gold"],
		"count 100": [[{"effect": "give", "items": {item0: QuestRules.ITEM_COUNT_MAX + 1}}], "count of '%s'" % item0],
		"gold above max": [[{"effect": "take", "gold": DialogueRules.EFFECT_GOLD_MAX + 1}], "gold must be"],
		"unknown give item": [[{"effect": "give", "items": {"no_such_item_qa": 1}}], "unknown base item"],
		"unknown shop": [[{"effect": "open_shop", "shop": "shop_nope"}], "unknown shop 'shop_nope'"],
		"unknown encounter": [[{"effect": "fight", "encounter": "enc_nope"}], "unknown encounter 'enc_nope'"],
		"lore-only start_quest": [[{"effect": "start_quest", "quest": "q_lore"}], "has no template"],
		"bad set_flag key": [[{"effect": "set_flag", "key": "Bad"}], "flag key 'Bad'"],
		"extra effect key": [[{"effect": "fight", "encounter": "enc_a", "who": "x"}], "unknown key 'who'"],
	}
	for label: Variant in bad:
		var pair: Array = DictUtil.array_of(bad[label])
		checks[str(label)] = _has(DialogueRules.effects_errors(W, pair[0], "Choice", false), str(pair[1]))
	var nine: Array = []
	for i: int in range(DialogueRules.MAX_EFFECTS + 1):
		nine.append({"effect": "set_flag", "key": "dg.f%d" % i})
	checks["9 effects"] = _one(DialogueRules.effects_errors(W, nine, "Choice", false), "at most %d effects" % DialogueRules.MAX_EFFECTS)
	checks["turn_in then set_flag and give ok"] = DialogueRules.effects_errors(W, [turn, flag, {"effect": "give", "gold": 3}], "Choice", false).is_empty()
	checks["flag then shop ok"] = DialogueRules.effects_errors(W, [flag, shop], "Choice", false).is_empty()
	checks["node level refuses start_quest"] = _one(DialogueRules.effects_errors(W, [{"effect": "start_quest", "quest": "q_main"}], "Node 'x'", true), "Node effects may only be set_flag or give.")
	return _report("Check 3: effects (unknown, order rules, give/take bounds, refs: errors)", checks)

# ------------------------------------------------------------------------------
# Check 4: shops
# ------------------------------------------------------------------------------

static func _test_shops(fx: Dictionary) -> bool:
	var items: Array[String] = JsonUtil.strings(fx["items"])
	var checks: Dictionary = {}
	checks["valid shop"] = _errs("shops", _shop(fx)).is_empty()
	var cases: Dictionary = {}
	var s: Dictionary = _shop(fx)
	s["keeper"] = "nobody"
	cases["unknown keeper"] = [s, "unknown character 'nobody'"]
	s = _shop(fx)
	s["keeper"] = ""
	cases["no keeper"] = [s, "Choose the shop keeper."]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["stock"])[0])["item"] = QUEST_ITEM
	cases["QUEST item in stock"] = [s, "Quest items can't be stocked."]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["stock"])[0])["count"] = 0
	cases["count 0"] = [s, "count must be -1 (unlimited) or 1..%d" % ShopRules.COUNT_MAX]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["stock"])[0])["price"] = 0
	cases["price 0"] = [s, "price must be -1 (derived) or 1..%d" % ShopRules.PRICE_MAX]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["stock"])[0])["item"] = "no_such_item_qa"
	cases["unknown stock item"] = [s, "unknown base item 'no_such_item_qa'"]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["stock"])[0])["bonus"] = 1
	cases["extra stock key"] = [s, "unknown key 'bonus'"]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["loot"])[0])["loot"] = "l_passive"
	cases["passive loot entry"] = [s, "Shops sell item loot only."]
	s = _shop(fx)
	DictUtil.dict_of(DictUtil.array_of(s["loot"])[0])["price"] = 0
	cases["loot price 0"] = [s, "price must be 1..%d" % ShopRules.PRICE_MAX]
	s = _shop(fx)
	DictUtil.array_of(s["loot"]).append({"loot": "l_item", "price": 10})
	cases["loot twice"] = [s, "lists 'l_item' twice"]
	s = _shop(fx)
	s["buy_mult"] = 0.4
	cases["buy_mult 0.4"] = [s, "Buy multiplier must be"]
	s = _shop(fx)
	s["buy_mult"] = 1.234
	cases["buy_mult 1.234"] = [s, "at most two decimals"]
	s = _shop(fx)
	s["sell_mult"] = 1.5
	cases["sell_mult 1.5"] = [s, "Sell multiplier must be"]
	s = _shop(fx)
	s["buys"] = ["QUEST"]
	cases["buys QUEST"] = [s, "'QUEST' is not an item type a shop buys"]
	s = _shop(fx)
	s["buys"] = ["WEAPON", "WEAPON"]
	cases["buys twice"] = [s, "lists 'WEAPON' twice"]
	s = _shop(fx)
	DictUtil.array_of(s["stock"]).append({"item": items[0], "count": 1, "price": 5, "requires_flags": []})
	cases["duplicate item"] = [s, "Stock lists '%s' twice." % items[0]]
	s = _shop(fx)
	s["requires_flags"] = [{"key": "Bad Key"}]
	cases["bad open flag"] = [s, "flag key 'Bad Key'"]
	for label: Variant in cases:
		var pair: Array = DictUtil.array_of(cases[label])
		checks[str(label)] = _one(_errs("shops", DictUtil.dict_of(pair[0])), str(pair[1]))
	s = _shop(fx)
	s["sell_mult"] = 0.0
	s["buy_mult"] = 3.0
	checks["edges 0.0 and 3.0 ok"] = _errs("shops", s).is_empty()
	return _report("Check 4: shops (keeper, QUEST stock, count, price, passive loot, multipliers, buys, duplicates)", checks)

# ------------------------------------------------------------------------------
# Check 5: gold, gone_flags, service lines
# ------------------------------------------------------------------------------

static func _test_gold_gone_services(fx: Dictionary) -> bool:
	var item0: String = JsonUtil.strings(fx["items"])[0]
	var checks: Dictionary = {}
	# Template rewards.gold
	var t_over: Dictionary = {"id": "t_gold", "name": "Gold", "objectives": [{"kind": "flag"}], "rewards": {"gold": QuestRules.GOLD_MAX + 1}}
	checks["gold above max errors"] = _has(QuestCatalog.validate_entry(t_over), "Rewards gold must be a whole number 0..%d." % QuestRules.GOLD_MAX)
	var t_soft: Dictionary = {"id": "t_gold", "name": "Gold", "objectives": [{"kind": "flag"}], "rewards": {"gold": QuestRules.SOFT_MAX_GOLD + 1}}
	checks["gold above soft cap warns"] = QuestCatalog.validate_entry(t_soft).is_empty() and QuestCatalog.warnings_for(t_soft).size() == 1 and QuestCatalog.warnings_for(t_soft)[0].contains("gold")
	var t_cap: Dictionary = {"id": "t_gold", "name": "Gold", "objectives": [{"kind": "flag"}], "rewards": {"gold": QuestRules.SOFT_MAX_GOLD}}
	checks["gold at soft cap is quiet"] = QuestCatalog.validate_entry(t_cap).is_empty() and QuestCatalog.warnings_for(t_cap).is_empty()
	checks["gold not whole errors"] = _has(QuestCatalog.validate_entry({"id": "t_gold", "name": "Gold", "objectives": [{"kind": "flag"}], "rewards": {"gold": 2.5}}), "Rewards gold")
	# Chest gold (a hand-built template: one spawn and one chest anchor).
	var tmpl: Dictionary = {"anchors": [{"id": MapRules.SPAWN, "x": 1, "y": 1}, {"id": "a_chest", "x": 2, "y": 2}], "paths": {}}
	var chest: Dictionary = {"id": "box", "kind": "chest", "anchor": "a_chest"}
	var gold_only: Dictionary = chest.merged({"gold": 25})
	checks["gold-only chest valid"] = MapRules.markers_errors(W, {}, tmpl, [gold_only]).is_empty()
	checks["chest gold 10000 errors"] = _one(MapRules.markers_errors(W, {}, tmpl, [chest.merged({"gold": MapRules.CHEST_GOLD_MAX + 1})]), "gold must be a whole number 0..%d" % MapRules.CHEST_GOLD_MAX)
	checks["empty chest still errors"] = _one(MapRules.markers_errors(W, {}, tmpl, [chest.merged({"gold": 0})]), "holds nothing")
	checks["items chest unchanged"] = MapRules.markers_errors(W, {}, tmpl, [chest.merged({"items": {item0: 2}})]).is_empty()
	# Character gone_flags
	var mara: Dictionary = WorldCatalog.get_own(W, "characters", "mara")
	checks["gone_flags valid"] = _errs("characters", mara.merged({"gone_flags": [GONE]}, true)).is_empty()
	checks["gone_flags bad key"] = _one(_errs("characters", mara.merged({"gone_flags": [{"key": "Bad Key"}]}, true)), "Gone flags 1: flag key 'Bad Key'")
	checks["unknown dialogue ref"] = _one(_errs("characters", mara.merged({"dialogue": "d_nope"}, true)), "unknown dialogue 'd_nope'")
	# Service file (sandbox)
	checks["unknown id"] = _has(ServiceCatalog.upsert({"id": "greet", "text": "Hi."}), "unknown service line 'greet'")
	checks["unknown token"] = _has(ServiceCatalog.upsert({"id": "greeting", "text": "Hi {name}."}), "Unknown token 'name'.")
	checks["empty text"] = _has(ServiceCatalog.upsert({"id": "greeting", "text": " "}), "needs text")
	checks["extra field"] = _has(ServiceCatalog.upsert({"id": "greeting", "text": "Hi.", "mood": "x"}), "Unknown service line field 'mood'")
	checks["refused upserts store nothing"] = ServiceCatalog.ids().is_empty()
	checks["empty file warns for every id"] = DictUtil.array_of(ServiceCatalog.warnings_all().get("services", [])).size() == DialogueRules.SERVICE_IDS.size()
	checks["valid line"] = ServiceCatalog.upsert({"id": "greeting", "text": "{npc} nods; you hold {gold}."}).is_empty() and ServiceCatalog.text_of("greeting") == "{npc} nods; you hold {gold}."
	checks["missing id warns"] = DictUtil.array_of(ServiceCatalog.warnings_all().get("services", [])).size() == DialogueRules.SERVICE_IDS.size() - 1
	checks["missing id falls back"] = ServiceCatalog.text_of("farewell") == str(DialogueRules.DEFAULT_LINES["farewell"])
	checks["duplicate id refused"] = not ServiceCatalog.upsert({"id": "greeting", "text": "Again."}, "farewell").is_empty()
	var before: Array[Dictionary] = ServiceCatalog.entries()
	checks["save"] = ServiceCatalog.save() == OK and not ServiceCatalog.is_dirty()
	ServiceCatalog.reload_from_disk()
	checks["reload equal"] = JsonUtil.deep_equal(before, ServiceCatalog.entries()) and ServiceCatalog.validate_all().is_empty()
	var broken: String = SANDBOX + "/broken_services.json"
	var f := FileAccess.open(broken, FileAccess.WRITE)
	f.store_string("{\"kind\": \"dialogue_services\", \"entries\": [{\"id\": \"greeting\", \"text\": \"Hi\", \"voice\": \"x\"}]}")
	f.close()
	ServiceCatalog.use_path(broken)
	checks["unknown stored field blocks save"] = ServiceCatalog.save_blocked() and ServiceCatalog.save() != OK and not ServiceCatalog.validate_all().is_empty()
	TestHelpers.sandbox_services(SANDBOX)
	# Manifest start_gold and service_lines
	checks["manifest valid"] = WorldCatalog.update_manifest(W, {"start_gold": 50, "service_lines": {"greeting": "{npc} waves."}}).is_empty() and WorldCatalog.manifest_errors(W).is_empty()
	checks["start_gold stored"] = DictUtil.to_int(WorldCatalog.manifest(W)["start_gold"]) == 50
	WorldCatalog.update_manifest(W, {"start_gold": -2})
	checks["start_gold -2"] = _one(WorldCatalog.manifest_errors(W), "Start gold must be -1 (inherited) or 0..99999.")
	WorldCatalog.update_manifest(W, {"start_gold": 100000})
	checks["start_gold 100000"] = _one(WorldCatalog.manifest_errors(W), "Start gold")
	WorldCatalog.update_manifest(W, {"start_gold": -1, "service_lines": {"greeting": "{who} waves."}})
	checks["service line token"] = _one(WorldCatalog.manifest_errors(W), "Unknown token 'who'.")
	WorldCatalog.update_manifest(W, {"service_lines": {"hello": "Hi."}})
	checks["service line id"] = _one(WorldCatalog.manifest_errors(W), "unknown service line 'hello'")
	WorldCatalog.update_manifest(W, {"service_lines": {}})
	checks["manifest restored"] = WorldCatalog.manifest_errors(W).is_empty()
	return _report("Check 5: rewards.gold cap and soft cap, chest gold, gone_flags, service file and manifest lines", checks)

# ------------------------------------------------------------------------------
# Check 6: gone-NPC lint
# ------------------------------------------------------------------------------

static func _set_quest(variants: Array, turn_in: String = "") -> void:
	var q: Dictionary = WorldCatalog.get_own(W, "quests", "q_main")
	q["variants"] = variants
	q["turn_in"] = turn_in
	WorldCatalog.upsert(W, "quests", q)

static func _set_gone(id: String, flags: Array) -> void:
	var c: Dictionary = WorldCatalog.get_own(W, "characters", id)
	c["gone_flags"] = flags
	WorldCatalog.upsert(W, "characters", c)

static func _test_lint() -> bool:
	var checks: Dictionary = {}
	var other: Dictionary = {"key": "dg.other", "value": true}
	_set_gone("mara", [GONE])
	var warned: Array[String] = DialogueRules.quest_warnings(W)
	checks["gone giver, no variant warns"] = warned.size() == 1 and warned[0].contains("Quest '%s:q_main': giver 'mara' can be gone (dg.mara_gone = true) and no variant re-routes it." % W)
	_set_quest([{"id": "v_giver", "when_flags": [GONE], "set": {"giver": "tev"}}])
	checks["variant setting giver clears it"] = DialogueRules.quest_warnings(W).is_empty()
	_set_quest([{"id": "v_off", "when_flags": [GONE], "set": {"available": false}}])
	checks["available false clears it"] = DialogueRules.quest_warnings(W).is_empty()
	_set_quest([{"id": "v_other", "when_flags": [other], "set": {"giver": "tev"}}])
	checks["variant on another flag does not"] = DialogueRules.quest_warnings(W).size() == 1
	_set_quest([{"id": "v_value", "when_flags": [{"key": GONE["key"], "value": false}], "set": {"giver": "tev"}}])
	checks["same key, other value does not"] = DialogueRules.quest_warnings(W).size() == 1
	_set_quest([{"id": "v_turn", "when_flags": [GONE], "set": {"turn_in": "tev"}}])
	checks["re-routing only turn_in leaves the giver"] = DialogueRules.quest_warnings(W).size() == 1
	# Explicit turn-in: its own gone flags need a turn_in re-route.
	_set_gone("mara", [])
	_set_gone("tev", [other])
	_set_quest([{"id": "v_giver", "when_flags": [other], "set": {"giver": "mara"}}], "tev")
	warned = DialogueRules.quest_warnings(W)
	checks["gone turn-in, giver variant only warns"] = warned.size() == 1 and warned[0].contains("turn-in 'tev'")
	_set_quest([{"id": "v_turn", "when_flags": [other], "set": {"turn_in": "mara"}}], "tev")
	checks["turn_in variant clears it"] = DialogueRules.quest_warnings(W).is_empty()
	_set_gone("tev", [])
	_set_quest([])
	checks["fixture restored"] = DialogueRules.quest_warnings(W).is_empty() and WorldCatalog.validate_world(W).is_empty()
	return _report("Check 6: gone-NPC lint (giver / turn-in gone flags need a re-routing variant or available false)", checks)

# ------------------------------------------------------------------------------
# Reference scans and renames through the dialogue and shop json fields
# ------------------------------------------------------------------------------

# The keys of the matches held by entry `id` of `kind`.
static func _keys_from(matches: Array[Dictionary], kind: String, id: String) -> Array[String]:
	var out: Array[String] = []
	for m: Dictionary in matches:
		if str(m["kind"]) == kind and str(m["id"]) == id:
			out.append(str(m["key"]))
	return out

static func _test_ref_scans() -> bool:
	var checks: Dictionary = {}
	# q_main: one condition (choice 1 requires), start_quest and turn_in effects.
	checks["quest refs in the tree"] = _keys_from(WorldCatalog.references_to(W, "quests", "q_main"), "dialogues", "d_mara") == ["nodes", "nodes", "nodes"]
	checks["speaker refs"] = _keys_from(WorldCatalog.references_to(W, "characters", "mara"), "dialogues", "d_mara") == ["speaker"] and _keys_from(WorldCatalog.references_to(W, "characters", "tev"), "dialogues", "d_mara") == ["nodes"]
	checks["keeper ref"] = _keys_from(WorldCatalog.references_to(W, "characters", "mara"), "shops", "shop_a") == ["keeper"]
	checks["shop ref"] = _keys_from(WorldCatalog.references_to(W, "shops", "shop_a"), "dialogues", "d_mara") == ["nodes"]
	checks["encounter ref"] = _keys_from(WorldCatalog.references_to(W, "encounters", "enc_a"), "dialogues", "d_mara") == ["nodes"]
	checks["loot ref"] = _keys_from(WorldCatalog.references_to(W, "loot", "l_item"), "shops", "shop_a") == ["loot"]
	checks["dialogue ref"] = _keys_from(WorldCatalog.references_to(W, "dialogues", "d_mara"), "characters", "mara") == ["dialogue"]
	checks["referenced entries can't be removed"] = not WorldCatalog.remove(W, "characters", "tev").is_empty() and not WorldCatalog.remove(W, "shops", "shop_a").is_empty() and not WorldCatalog.remove(W, "dialogues", "d_mara").is_empty()
	# Renames rewrite every path-addressed ref.
	var renamed: bool = true
	for pair: Array in [["quests", "q_main", "q_main2"], ["characters", "tev", "tev2"], ["shops", "shop_a", "shop_b"], ["encounters", "enc_a", "enc_b"], ["loot", "l_item", "l_item2"], ["dialogues", "d_mara", "d_mara2"]]:
		var kind: String = str(pair[0])
		var e: Dictionary = WorldCatalog.get_own(W, kind, str(pair[1]))
		e["id"] = str(pair[2])
		renamed = WorldCatalog.upsert(W, kind, e, str(pair[1])).is_empty() and renamed
	checks["renames stored"] = renamed
	var t: Dictionary = WorldCatalog.get_own(W, "dialogues", "d_mara2")
	var hello: Array = _choices(t, "hello")
	checks["condition quest renamed"] = str(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(hello[0])["requires"])[0])["quest"]) == "q_main2"
	checks["effect refs renamed"] = str(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(hello[2])["effects"])[0])["shop"]) == "shop_b" and str(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(hello[3])["effects"])[1])["encounter"]) == "enc_b" and str(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(hello[4])["effects"])[0])["quest"]) == "q_main2" and str(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(hello[5])["effects"])[0])["quest"]) == "q_main2"
	checks["node speaker renamed"] = str(_node(t, "info")["speaker"]) == "tev2"
	checks["shop loot renamed"] = str(DictUtil.dict_of(DictUtil.array_of(WorldCatalog.get_own(W, "shops", "shop_b")["loot"])[0])["loot"]) == "l_item2"
	checks["character dialogue renamed"] = str(WorldCatalog.get_own(W, "characters", "mara")["dialogue"]) == "d_mara2"
	var problems: Dictionary = WorldCatalog.validate_world(W)
	checks["world still validates"] = problems.is_empty()
	if not problems.is_empty():
		print("      [%s] after renames: %s" % [TAG, str(problems)])
	return _report("Reference scans and renames (speakers, condition / effect refs, keeper, shop loot, character dialogue)", checks)

static func _test_persistence() -> bool:
	var checks: Dictionary = {}
	var tree: Dictionary = WorldCatalog.get_own(W, "dialogues", "d_mara2")
	var shop: Dictionary = WorldCatalog.get_own(W, "shops", "shop_b")
	checks["save"] = WorldCatalog.save_world(W) == OK
	checks["files written"] = FileAccess.file_exists(WorldCatalog.path_for(W, "dialogues")) and FileAccess.file_exists(WorldCatalog.path_for(W, "shops"))
	WorldCatalog.reload_from_disk()
	checks["tree round-trips"] = JsonUtil.deep_equal(tree, WorldCatalog.get_own(W, "dialogues", "d_mara2"))
	checks["shop round-trips"] = JsonUtil.deep_equal(shop, WorldCatalog.get_own(W, "shops", "shop_b"))
	checks["valid after reload"] = WorldCatalog.validate_world(W).is_empty() and WorldCatalog.load_errors().is_empty()
	return _report("Persistence (dialogues.json and shops.json written, reload equal and valid)", checks)
