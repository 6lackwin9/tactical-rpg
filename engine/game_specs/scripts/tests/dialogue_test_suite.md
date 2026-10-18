---
class: DialogueTestSuite
extends: RefCounted
description: "Tactical RPG - Dialogue Test Suite (S11, spec DLG1-L) Schema (checks 1-6): dialogue trees, choice conditions and effects (DialogueRules), shops (ShopRules), the base service lines (ServiceCatalog) and manifest service_lines / start_gold, quest reward gold and chest gold, character gone_flags, the gone-NPC quest lint, the review rules (text tokens, give-on-cycle lint, one gone_flags message), and reference scans / renames through the new json fields. Runtime (checks 7-20): DialogueRunner visibility, all-or-nothing choices, services and giver / turn-in enforcement, turn-in confirm / cancel, presence, node effects, ShopRules prices / stock / loot sales / selling, party gold from quests, chests and NewGame, dialogue fights, save v4. Commit 3 (DialoguePanelChecks): the shop return rule and the dialogue / shop debug panels in the region map scene. Fixtures: a content fork (test-only QUEST and CONSUMABLE items), sandbox quest / service / map files and save folder, and hand-built worlds in an EMPTY sandbox world root; the shipped data is only read (ids picked by rule)."
---

# Properties
* `TAG`: String = "Dialogue" (const)
* `W`: String = "dg_base" (const)
* `QUEST_ITEM`: String = "qa_dlg_quest_token" (const)
* `GONE`: Dictionary = {"key": "dg.mara_gone", "value": true} (const)
* `out`: Array[String] = []
* `R`: String = "dg_run" (const)
* `KID`: String = "dg_kid" (const)
* `START_GOLD`: int = 50 (const)
* `CHEST_GOLD`: int = 25 (const)
* `GIFT`: int = 5 (const)
* `FIXED`: int = 7 (const)
* `LOOT_PRICE`: int = 150 (const)
* `I15`: String = "qa_dlg_value15" (const)
* `I25`: String = "qa_dlg_value25" (const)
* `I0`: String = "qa_dlg_value0" (const)
* `MARA_GONE`: Dictionary = {"key": "dr.mara_gone", "value": true} (const)
* `HEST_GONE`: Dictionary = {"key": "dr.hest_gone", "value": true} (const)

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

### `static _has(errors: Array[String], needle: String) -> bool`
```logic
if not TestHelpers.any_contains(errors, needle):
	print("      [%s] expected '%s' in %s" % [TAG, needle, str(errors)])
	return false
return true
```

### `static _node(t: Dictionary, id: String) -> Dictionary`
```logic
for n: Variant in DictUtil.array_of(t["nodes"]):
	var node: Dictionary = DictUtil.dict_of(n)
	if str(node.get("id", "")) == id:
		return node
return {}
```

### `static _choices(t: Dictionary, id: String) -> Array`
```logic
return DictUtil.array_of(_node(t, id).get("choices", []))
```

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING DIALOGUE TEST (DLG1-L)                  ===")
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
		ok = _test_recruit(fx) and ok
		ok = _test_shops(fx) and ok
		ok = _test_gold_gone_services(fx) and ok
		ok = _test_lint() and ok
		ok = _test_review_rules() and ok
		ok = _test_ref_scans() and ok
		ok = _test_persistence() and ok
		MapCatalog.use_path(SANDBOX + "/maps.json")
		SaveGame.set_base_dir(SANDBOX + "/saves")
		var rx: Dictionary = _runtime_fixtures()
		if _build_runtime(rx):
			ok = _test_visibility() and ok
			ok = _test_all_or_nothing() and ok
			ok = _test_services() and ok
			ok = _test_enforcement() and ok
			ok = _test_turn_in() and ok
			ok = _test_gone() and ok
			ok = _test_node_effects() and ok
			ok = _test_prices() and ok
			ok = _test_stock() and ok
			ok = _test_loot_sale() and ok
			ok = _test_selling() and ok
			ok = _test_gold() and ok
			ok = _test_fight() and ok
			ok = _test_save_v4() and ok
			ok = DialoguePanelChecks.run(tree_parent) and ok   # commit 3: shop return rule, debug panels
		else:
			ok = false
		SaveGame.reset_base_dir()
		MapCatalog.use_path(MapCatalog.DEFAULT_PATH)
	else:
		ok = false
TestHelpers.restore_services()
QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
TestHelpers.cleanup(SANDBOX + "/catalog")
TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox
return ok
```

### `static _test_shipped() -> bool`
```logic
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
var expected_buys: Array[String] = []
for t: String in ItemFamilies.ITEM_TYPES:
	if t != "QUEST":
		expected_buys.append(t)
checks["shop defaults"] = JsonUtil.strings(s["buys"]) == expected_buys and ShopRules.buy_types() == expected_buys and is_equal_approx(DictUtil.to_float(s["buy_mult"]), 1.0) and is_equal_approx(DictUtil.to_float(s["sell_mult"]), 0.5) and str(s["name"]) == "New Shop"
checks["chest gold default"] = DictUtil.to_int(DictUtil.dict_of(MapRules.MARKER_KINDS["chest"]).get("gold", -1)) == 0
checks["gold reward key"] = QuestRules.REWARD_KEYS.has("gold")
return _report("Shipped data validates with the new kinds and fields; defaults", checks)
```

### `static _content_fixtures() -> Dictionary`
```logic
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
```

### `static _tree(fx: Dictionary) -> Dictionary`
```logic
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
```

### `static _shop(fx: Dictionary) -> Dictionary`
```logic
var items: Array[String] = JsonUtil.strings(fx["items"])
return {"id": "shop_a", "name": "Shop A", "keeper": "mara",
	"stock": [{"item": items[0], "count": -1, "price": -1, "requires_flags": []}, {"item": items[1], "count": 2, "price": 40, "requires_flags": [{"key": "dg.open"}]}],
	"loot": [{"loot": "l_item", "price": 150}], "buy_mult": 1.15, "sell_mult": 0.85, "buys": ["CONSUMABLE", "WEAPON"], "requires_flags": []}
```

### `static _build_world(fx: Dictionary) -> bool`
```logic
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
```

### `static _test_trees(fx: Dictionary) -> bool`
```logic
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
checks["node effect take"] = _one(_errs("dialogues", t), "Node effects may only be set_flag, add_flag or give.")
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
```

### `static _test_conditions(fx: Dictionary) -> bool`
```logic
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
```

### `static _test_effects(fx: Dictionary) -> bool`
```logic
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
	"turn_in second": [[flag, turn], "turn_in must come first; only set_flag, add_flag or give may follow."],
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
	checks[str(label)] = _one(DialogueRules.effects_errors(W, pair[0], "Choice", false), str(pair[1]))
var nine: Array = []
for i: int in range(DialogueRules.MAX_EFFECTS + 1):
	nine.append({"effect": "set_flag", "key": "dg.f%d" % i})
checks["9 effects"] = _one(DialogueRules.effects_errors(W, nine, "Choice", false), "at most %d effects" % DialogueRules.MAX_EFFECTS)
checks["turn_in then set_flag and give ok"] = DialogueRules.effects_errors(W, [turn, flag, {"effect": "give", "gold": 3}], "Choice", false).is_empty()
checks["flag then shop ok"] = DialogueRules.effects_errors(W, [flag, shop], "Choice", false).is_empty()
checks["node level refuses start_quest"] = _one(DialogueRules.effects_errors(W, [{"effect": "start_quest", "quest": "q_main"}], "Node 'x'", true), "Node effects may only be set_flag, add_flag or give.")
return _report("Check 3: effects (unknown, order rules, give/take bounds, refs: errors)", checks)
```

### `static _unit_of_side(side: String) -> String`
```logic
for row: Dictionary in ContentCatalog.entries("units"):
	if str(row.get("side", "enemy")) == side:
		return str(row["id"])
return ""
```

### `static _test_recruit(fx: Dictionary) -> bool`
```logic
var checks: Dictionary = {}
var party_unit: String = _unit_of_side("party")
var enemy_unit: String = _unit_of_side("enemy")
var item_qid: String = ""
var errors: Array[String] = []
errors.append_array(WorldCatalog.upsert(W, "items", {"id": "rc_blade", "name": "Recruit blade", "base_item": JsonUtil.strings(fx["items"])[0]}))
if WorldCatalog.get_own(W, "items", "rc_blade").is_empty():
	errors.clear()   # the item fixture is optional; the named-item case falls back to tuning
else:
	item_qid = "rc_blade"
errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "rc_hero", "name": "Rc Hero", "role": "party", "base_unit": party_unit, "passives": [str(fx["passive"])]}))
errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "rc_npc", "name": "Rc Npc", "role": "npc", "base_unit": party_unit}))
errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "rc_foe_unit", "name": "Rc Foe Unit", "role": "party", "base_unit": enemy_unit}))
errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "rc_tuned", "name": "Rc Tuned", "role": "party", "base_unit": party_unit, "tuning": {"power": 1}}))
checks["fixtures stored"] = errors.is_empty() and not party_unit.is_empty() and not enemy_unit.is_empty()
if not errors.is_empty():
	print("      [%s] recruit fixtures: %s" % [TAG, str(errors)])
var rc: Dictionary = {"effect": "recruit", "character": "rc_hero"}
var flag: Dictionary = {"effect": "set_flag", "key": "dg.rc"}
checks["known effect"] = DialogueRules.EFFECTS.has("recruit") and str(DialogueRules.EFFECT_REF_KEYS.get("character", "")) == "characters"
checks["valid recruit (passives allowed)"] = DialogueRules.effects_errors(W, [rc, flag], "Choice", false).is_empty()
checks["party_character_errors clean"] = WorldCatalog.party_character_errors(W, "rc_hero").is_empty()
var bad: Dictionary = {
	"unknown character": [[{"effect": "recruit", "character": "rc_nobody"}], "Recruit: unknown character 'rc_nobody'."],
	"no character": [[{"effect": "recruit"}], "choose a character"],
	"extra key": [[{"effect": "recruit", "character": "rc_hero", "who": 1}], "unknown key 'who'"],
	"two recruits": [[rc, {"effect": "recruit", "character": "rc_hero"}], "at most one recruit per choice."],
	"not role party": [[{"effect": "recruit", "character": "rc_npc"}], "Party hero 'rc_npc' needs role party"],
	"enemy base unit": [[{"effect": "recruit", "character": "rc_foe_unit"}], "Party hero 'rc_foe_unit' must play as a party unit"],
	"tuning": [[{"effect": "recruit", "character": "rc_tuned"}], "Party hero 'rc_tuned' may not carry named items, disciplines or tuning yet (RECRUIT1)."],
}
for label: Variant in bad:
	var pair: Array = DictUtil.array_of(bad[label])
	checks[str(label)] = _one(DialogueRules.effects_errors(W, pair[0], "Choice", false), str(pair[1]))
if not item_qid.is_empty():
	var armed_errs: Array[String] = WorldCatalog.upsert(W, "characters", {"id": "rc_armed", "name": "Rc Armed", "role": "party", "base_unit": party_unit, "named_items": [item_qid]})
	checks["named item"] = armed_errs.is_empty() and WorldCatalog.party_character_errors(W, "rc_armed") == ["Party hero 'rc_armed' may not carry named items, disciplines or tuning yet (RECRUIT1)."]
	var _gone_armed: String = WorldCatalog.remove(W, "characters", "rc_armed")
checks["node level refuses recruit"] = _one(DialogueRules.effects_errors(W, [rc], "Node 'x'", true), "Node effects may only be set_flag, add_flag or give.")
# A stored tree: the recruit ref is found by reference scans and follows a rename.
var tree: Dictionary = {"id": "d_rc", "name": "Recruit talk", "speaker": "mara", "start": "hello", "services": false, "nodes": [
	{"id": "hello", "text": "Join?", "choices": [{"text": "Join me", "to": "", "effects": [rc, flag]}, {"text": "Bye", "to": ""}]}]}
var stored: Array[String] = WorldCatalog.upsert(W, "dialogues", tree)
checks["tree stored"] = stored.is_empty()
if not stored.is_empty():
	print("      [%s] recruit tree: %s" % [TAG, str(stored)])
checks["recruit ref scanned"] = _keys_from(WorldCatalog.references_to(W, "characters", "rc_hero"), "dialogues", "d_rc") == ["nodes"]
var hero: Dictionary = WorldCatalog.get_own(W, "characters", "rc_hero")
hero["id"] = "rc_hero2"
var renamed: bool = WorldCatalog.upsert(W, "characters", hero, "rc_hero").is_empty()
var after: Dictionary = WorldCatalog.get_own(W, "dialogues", "d_rc")
checks["recruit ref renamed"] = renamed and str(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(_choices(after, "hello")[0])["effects"])[0])["character"]) == "rc_hero2"
var _gone_tree: String = WorldCatalog.remove(W, "dialogues", "d_rc")
for id: String in ["rc_hero2", "rc_npc", "rc_foe_unit", "rc_tuned"]:
	var _gone: String = WorldCatalog.remove(W, "characters", id)
if not item_qid.is_empty():
	var _gone_item: String = WorldCatalog.remove(W, "items", item_qid)
checks["fixture world still validates"] = WorldCatalog.validate_world(W).is_empty()
return _report("Check 3b: recruit effect (RECRUIT1-L 1.4: keys, choices only, one per choice, ref, party hero rules, scans)", checks)
```

### `static _test_shops(fx: Dictionary) -> bool`
```logic
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
```

### `static _test_gold_gone_services(fx: Dictionary) -> bool`
```logic
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
```

### `static _set_quest(variants: Array, turn_in: String = "") -> void`
```logic
var q: Dictionary = WorldCatalog.get_own(W, "quests", "q_main")
q["variants"] = variants
q["turn_in"] = turn_in
WorldCatalog.upsert(W, "quests", q)
```

### `static _set_gone(id: String, flags: Array) -> void`
```logic
var c: Dictionary = WorldCatalog.get_own(W, "characters", id)
c["gone_flags"] = flags
WorldCatalog.upsert(W, "characters", c)
```

### `static _test_lint() -> bool`
```logic
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
```

### `static _test_ref_scans() -> bool`
```logic
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
```

### `static _test_persistence() -> bool`
```logic
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
```

### `static _lint_tree(nodes: Array, services: bool) -> Dictionary`
```logic
return {"id": "d_lint", "name": "Lint", "start": "a", "services": services, "nodes": nodes}
```

### `static _test_review_rules() -> bool`
```logic
var checks: Dictionary = {}
var mara: Dictionary = WorldCatalog.get_own(W, "characters", "mara")
var not_list: Dictionary = mara.merged({"gone_flags": {"key": "dg.x"}}, true)
var v_errs: Array[String] = _errs("characters", not_list)
var b_errs: Array[String] = WorldCatalog.blocking_errors(W, "characters", not_list, "mara")
checks["non-list gone_flags: one problem"] = _one(v_errs, "Gone flags must be a list") and _one(b_errs, "Gone flags must be a list") and v_errs == b_errs
# Authored texts: {npc} and {gold} only.
var t: Dictionary = {"id": "d_tok", "name": "Tokens", "start": "a", "services": false, "nodes": [{"id": "a", "text": "{npc} has {gold}.", "choices": [{"text": "Hi {npc}", "to": "b"}]}, {"id": "b", "text": "Bye.", "end": true}]}
checks["npc and gold tokens ok"] = _errs("dialogues", t).is_empty()
_node(t, "a")["text"] = "About {quest}."
checks["quest token in a node"] = _one(_errs("dialogues", t), "Unknown token 'quest'")
_node(t, "a")["text"] = "Fine."
DictUtil.dict_of(_choices(t, "a")[0])["text"] = "Visit {shop}"
checks["shop token in a choice"] = _one(_errs("dialogues", t), "Unknown token 'shop'")
# give on a cycle: a -> b -> a (next) warns; a choice guarded by unless + the node's own set_flag does not.
var gift: Dictionary = {"effect": "give", "gold": 1}
var loop: Dictionary = _lint_tree([{"id": "a", "text": "A.", "choices": [{"text": "Go", "to": "b"}, {"text": "Bye", "to": ""}]}, {"id": "b", "text": "B.", "next": "a", "effects": [gift]}], false)
var warned: Array[String] = DialogueRules.tree_warnings(loop)
checks["give on a cycle warns"] = warned.size() == 1 and warned[0].contains("Node 'b' gives items or gold and sits on a dialogue cycle")
var guarded: Dictionary = loop.duplicate(true)
DictUtil.dict_of(_choices(guarded, "a")[0])["unless"] = [{"key": "dg.got"}]
DictUtil.array_of(_node(guarded, "b")["effects"]).append({"effect": "set_flag", "key": "dg.got"})
checks["unless + set_flag guard is quiet"] = DialogueRules.tree_warnings(guarded).is_empty()
var other_flag: Dictionary = guarded.duplicate(true)
DictUtil.dict_of(_choices(other_flag, "a")[0])["unless"] = [{"key": "dg.other"}]
checks["guard on another flag still warns"] = DialogueRules.tree_warnings(other_flag).size() == 1
var no_cycle: Dictionary = _lint_tree([{"id": "a", "text": "A.", "choices": [{"text": "Go", "to": "b"}]}, {"id": "b", "text": "B.", "end": true, "effects": [gift]}], false)
checks["give off a cycle is quiet"] = DialogueRules.tree_warnings(no_cycle).is_empty()
var start_gift: Dictionary = _lint_tree([{"id": "a", "text": "A.", "effects": [gift], "choices": [{"text": "Bye", "to": ""}]}], true)
checks["give on the start node with services warns"] = DialogueRules.tree_warnings(start_gift).size() == 1
start_gift["services"] = false
checks["same start node without services is quiet"] = DialogueRules.tree_warnings(start_gift).is_empty()
var shop_back: Dictionary = _lint_tree([{"id": "a", "text": "A.", "choices": [{"text": "Go", "to": "b"}]}, {"id": "b", "text": "B.", "effects": [gift], "choices": [{"text": "Shop", "to": "", "effects": [{"effect": "open_shop", "shop": "shop_b"}]}]}], false)
checks["open_shop back to a give node warns"] = DialogueRules.tree_warnings(shop_back).size() == 1
return _report("Review rules (one gone_flags message, {npc} / {gold} only in authored texts, give-on-cycle lint)", checks)
```

### `static _q(local: String) -> String`
```logic
return WorldCatalog.qid(R, local)
```

### `static _runtime_fixtures() -> Dictionary`
```logic
var rows: Array[Array] = []
for e: Dictionary in ContentCatalog.entries("units"):
	if str(e.get("side", "")) != "party" or ContentFactory.class_of(e) == null:
		continue
	var id: String = str(e["id"])
	var c: Combatant = PartyBuilder.build(UnitState.at_level(id, DictUtil.to_int(e.get("level", 1), 1), id))
	if c != null and c.stats != null:
		rows.append([id, c.stats.get_effective_stat("KNOW")])
var lo: Array = []
var hi: Array = []
for a: Array in rows:
	for b: Array in rows:
		if DictUtil.to_int(a[1]) < DictUtil.to_int(b[1]) and DictUtil.to_int(b[1]) <= DialogueRules.STAT_MAX and lo.is_empty():
			lo = a
			hi = b
var weapon: String = ""
for row: Dictionary in ContentCatalog.entries("items"):
	if str(row.get("item_type", "")) == "WEAPON" and weapon.is_empty():
		weapon = str(row["id"])
var added: Array[String] = []
for pair: Array in [[I15, 15], [I25, 25], [I0, 0]]:
	added.append_array(TestHelpers.add_item_of_type(str(pair[0]), "CONSUMABLE", DictUtil.to_int(pair[1]), {"heal": 10}))
if not added.is_empty():
	print("      [%s] runtime items: %s" % [TAG, str(added)])
return {"lo": str(lo[0]) if not lo.is_empty() else "", "hi": str(hi[0]) if not hi.is_empty() else "", "know_hi": DictUtil.to_int(hi[1]) if not hi.is_empty() else 0, "weapon": weapon, "items_ok": added.is_empty()}
```

### `static _map() -> Dictionary`
```logic
return {"id": "m_dg", "name": "Dialogue yard", "rows": ["########", "#......#", "#......#", "#......#", "#......#", "#......#", "#......#", "########"],
	"anchors": [{"id": "spawn", "x": 1, "y": 1}, {"id": "a_npc", "x": 3, "y": 1}, {"id": "a_hest", "x": 3, "y": 3}, {"id": "a_chest", "x": 1, "y": 3}]}
```

### `static _mara_tree() -> Dictionary`
```logic
var know: int = DictUtil.to_int(_rx["know_hi"])
return {"id": "d_mara", "name": "Mara", "start": "hello", "nodes": [
	{"id": "hello", "text": "{npc} nods; you hold {gold} gold.", "choices": [
		{"text": "Ask", "to": "lore", "requires": [{"kind": "stat", "stat": "KNOW", "min": know}], "show_locked": true},
		{"text": "Secret", "to": "hello", "requires": [{"key": "dr.secret"}]},
		{"text": "Gift once", "to": "hello", "unless": [{"key": "dr.gift"}], "effects": [{"effect": "set_flag", "key": "dr.gift"}, {"effect": "give", "gold": GIFT}]},
		{"text": "Trade", "to": "", "effects": [{"effect": "give", "gold": 5}, {"effect": "take", "items": {I15: 2}}]},
		{"text": "Double take", "to": "hello", "effects": [{"effect": "take", "items": {I15: 1}}, {"effect": "take", "items": {I15: 1}}]},
		{"text": "Start Tev's", "to": "hello", "effects": [{"effect": "start_quest", "quest": "q_tev"}]},
	]},
	{"id": "lore", "text": "Old tales.", "next": "hello", "effects": [{"effect": "give", "gold": 1}]},
]}
```

### `static _hest_tree() -> Dictionary`
```logic
return {"id": "d_hest", "name": "Hest", "start": "counter", "nodes": [
	{"id": "counter", "text": "Buying?", "choices": [
		{"text": "Hand it over", "to": "counter", "effects": [{"effect": "turn_in", "quest": "q_hest"}, {"effect": "set_flag", "key": "dr.thanked"}]},
		{"text": "Fight", "effects": [{"effect": "set_flag", "key": "dr.fought"}, {"effect": "fight", "encounter": "enc_a"}]},
		{"text": "Bye", "to": ""},
	]},
]}
```

### `static _build_runtime(rx: Dictionary) -> bool`
```logic
_rx = rx
var checks: Dictionary = {"two units with different KNOW": not str(rx["lo"]).is_empty() and DictUtil.to_int(rx["know_hi"]) >= DialogueRules.STAT_MIN, "a weapon": not str(rx["weapon"]).is_empty(), "test items": DictUtil.to_bool(rx["items_ok"])}
if not TestHelpers.failed(checks).is_empty():
	return _report("Runtime fixtures", checks)
var lo: String = str(rx["lo"])
var errors: Array[String] = []
errors.append_array(MapCatalog.upsert(_map()))
errors.append_array(QuestCatalog.upsert({"id": "t_gold", "name": "Gold", "objectives": [{"kind": "flag"}], "rewards": {"gold": QuestRules.SOFT_MAX_GOLD}}))
errors.append_array(QuestCatalog.upsert({"id": "t_win", "name": "Win", "objectives": [{"kind": "win_encounter"}], "rewards": {"xp": 1}}))
errors.append_array(WorldCatalog.create_world(R, "Dialogue runtime"))
errors.append_array(WorldCatalog.update_manifest(R, {"start_gold": START_GOLD}))
errors.append_array(WorldCatalog.create_world(KID, "Dialogue kid", [R]))
for pair: Array in [["lo", "party", lo], ["hi", "party", str(rx["hi"])], ["foe", "enemy", lo]]:
	errors.append_array(WorldCatalog.upsert(R, "characters", {"id": pair[0], "name": str(pair[0]).capitalize(), "role": pair[1], "base_unit": pair[2]}))
errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "mara", "name": "Mara", "base_unit": lo, "dialogue": "d_mara", "gone_flags": [MARA_GONE]}))
errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "tev", "name": "Tev", "base_unit": lo}))
errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "hest", "name": "Hest", "base_unit": lo, "dialogue": "d_hest", "gone_flags": [HEST_GONE]}))
errors.append_array(WorldCatalog.upsert(R, "encounters", {"id": "enc_a", "name": "Enc A", "party": ["lo"], "enemies": ["foe"]}))
errors.append_array(WorldCatalog.upsert(R, "locations", {"id": "yard", "name": "Yard", "map": "m_dg", "markers": [
	{"id": "npc_mara", "kind": "npc", "anchor": "a_npc", "character": "mara"},
	{"id": "npc_hest", "kind": "npc", "anchor": "a_hest", "character": "hest"},
	{"id": "box", "kind": "chest", "anchor": "a_chest", "gold": CHEST_GOLD},
]}))
errors.append_array(WorldCatalog.upsert(R, "loot", {"id": "l_ring", "name": "Ring", "reward_kind": "item", "ref": I25, "count": 1}))
errors.append_array(WorldCatalog.upsert(R, "loot", {"id": "l_gem", "name": "Gem", "reward_kind": "item", "ref": I15, "count": 1}))
errors.append_array(WorldCatalog.upsert(R, "loot", {"id": "l_locked", "name": "Locked", "reward_kind": "item", "ref": I25, "count": 1, "requires_flags": [{"key": "dr.vip"}]}))
errors.append_array(WorldCatalog.upsert(R, "quests", {"id": "q_offer", "name": "Offer", "description": "Raise the signal for us.", "template": "t_gold", "giver": "mara",
	"steps": [{"text": "Signal", "flag": "dr.signal"}], "loot_pool": ["l_ring"], "loot_picks": 1,
	"variants": [{"id": "v_tev", "when_flags": [{"key": "dr.swap"}], "set": {"giver": "tev"}}, {"id": "v_gone", "when_flags": [MARA_GONE], "set": {"available": false}}]}))
errors.append_array(WorldCatalog.upsert(R, "quests", {"id": "q_tev", "name": "Tev's", "template": "t_gold", "giver": "tev", "steps": [{"text": "Tev", "flag": "dr.tev"}]}))
errors.append_array(WorldCatalog.upsert(R, "quests", {"id": "q_hest", "name": "Hest's", "template": "t_gold", "giver": "hest", "turn_in": "hest", "steps": [{"text": "Hest", "flag": "dr.hest"}], "loot_pool": ["l_gem"], "loot_picks": 1}))
errors.append_array(WorldCatalog.upsert(R, "quests", {"id": "q_journal", "name": "Journal", "template": "t_gold", "steps": [{"text": "Note", "flag": "dr.j"}]}))
errors.append_array(WorldCatalog.upsert(R, "quests", {"id": "q_win", "name": "Win", "template": "t_win", "steps": [{"text": "Win", "encounter": "enc_a"}]}))
errors.append_array(WorldCatalog.upsert(R, "shops", {"id": "s_hest", "name": "Hest's counter", "keeper": "hest", "buys": ["CONSUMABLE"],
	"stock": [{"item": I15, "count": 2, "price": -1, "requires_flags": []}, {"item": I25, "count": -1, "price": -1, "requires_flags": []}, {"item": I0, "count": -1, "price": -1, "requires_flags": []}, {"item": str(rx["weapon"]), "count": -1, "price": FIXED, "requires_flags": []}],
	"loot": [{"loot": "l_ring", "price": LOOT_PRICE}, {"loot": "l_locked", "price": LOOT_PRICE}]}))
errors.append_array(WorldCatalog.upsert(R, "shops", {"id": "s_zero", "name": "Tev's stall", "keeper": "tev", "sell_mult": 0.0}))
errors.append_array(WorldCatalog.upsert(R, "dialogues", _mara_tree()))
errors.append_array(WorldCatalog.upsert(R, "dialogues", _hest_tree()))
var problems: Dictionary = WorldCatalog.validate_world(R)
checks["stored"] = errors.is_empty()
checks["runtime world validates"] = problems.is_empty() and WorldCatalog.validate_world(KID).is_empty()
if not errors.is_empty() or not problems.is_empty():
	print("      [%s] runtime fixture problems: %s %s" % [TAG, str(errors), str(problems)])
return _report("Runtime fixture world (dialogues, shops, quests, map with npc and chest markers)", checks)
```

### `static _gs() -> GameState`
```logic
var gs: GameState = GameState.of(NewGame.create(R, [str(_rx["lo"]), str(_rx["hi"])] as Array[String], 7)["state"])
gs.party.active_ids = [str(_rx["lo"])] as Array[String]
return gs
```

### `static _find(v: Dictionary, needle: String) -> int`
```logic
var choices: Array = DictUtil.array_of(v.get("choices", []))
for i: int in range(choices.size()):
	if str(DictUtil.dict_of(choices[i]).get("text", "")).contains(needle):
		return i
return -1
```

### `static _line(id: String, quest_ref: String = "") -> String`
```logic
var values: Dictionary = {}
if not quest_ref.is_empty():
	values["quest"] = str(WorldCatalog.resolve(R, "quests", quest_ref).get("name", ""))
return DialogueRules.fill(DialogueRules.service_line(R, id), values)
```

### `static _same(gs: GameState, before: Dictionary) -> bool`
```logic
return JsonUtil.deep_equal(gs.to_dict(), before)
```

### `static _choice_at(v: Dictionary, i: int) -> Dictionary`
```logic
var choices: Array = DictUtil.array_of(v.get("choices", []))
return DictUtil.dict_of(choices[i]) if i >= 0 and i < choices.size() else {}
```

### `static _test_visibility() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var r: DialogueRunner = DialogueRunner.open(gs, "mara")
checks["opened"] = r.errors.is_empty() and r.node_id == "hello" and not r.closed
var v: Dictionary = r.view()
var ask: int = _find(v, "Ask")
var row: Dictionary = _choice_at(v, ask)
checks["failing show_locked choice is locked"] = ask >= 0 and DictUtil.to_bool(row.get("locked", false))
checks["stat prefix generated"] = str(row.get("text", "")).begins_with("[KNOW %d] " % DictUtil.to_int(_rx["know_hi"]))
checks["failing hidden choice is hidden"] = _find(v, "Secret") == -1
checks["npc and gold tokens filled"] = str(v["text"]) == "Mara nods; you hold %d gold." % gs.party.gold and str(v["speaker"]) == _q("mara")
var before: Dictionary = gs.to_dict()
checks["locked choice changes nothing"] = not DictUtil.to_bool(r.choose(ask)["ok"]) and _same(gs, before) and r.node_id == "hello"
checks["unknown index refused"] = not DictUtil.to_bool(r.choose(99)["ok"]) and _same(gs, before)
gs.party.active_ids = [str(_rx["hi"])] as Array[String]
v = r.view()
checks["best ACTIVE member decides"] = _find(v, "Ask") >= 0 and not DictUtil.to_bool(_choice_at(v, _find(v, "Ask")).get("locked", true))
var _f: String = gs.set_flag("dr.secret", true)
checks["condition met shows the choice"] = _find(r.view(), "Secret") >= 0
var all: Array = DictUtil.array_of(r.view()["choices"])
checks["services follow the authored choices"] = str(DictUtil.dict_of(all.back()).get("text", "")) == _line("farewell")
return _report("Check 7: visibility (hidden vs show_locked, locked changes nothing, [STAT min] prefix, best active member)", checks)
```

### `static _test_all_or_nothing() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.add_to_stash(I15, 1)
var r: DialogueRunner = DialogueRunner.open(gs, "mara")
var before: Dictionary = gs.to_dict()
var trade: Dictionary = r.choose(_find(r.view(), "Trade"))
checks["give + take with too few changes nothing"] = not DictUtil.to_bool(trade["ok"]) and TestHelpers.any_contains(JsonUtil.strings(trade["errors"]), "Not enough") and _same(gs, before) and not r.closed
checks["two takes are summed"] = not DictUtil.to_bool(r.choose(_find(r.view(), "Double take"))["ok"]) and _same(gs, before)
gs.party.add_to_stash(I15, 1)
checks["both takes run with enough"] = DictUtil.to_bool(r.choose(_find(r.view(), "Double take"))["ok"]) and not gs.party.stash.has(I15)
return _report("Check 8: all-or-nothing choices (failing take keeps the give back; takes summed)", checks)
```

### `static _test_services() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var offer: String = _line("offer", "q_offer")
checks["offered by the giver"] = _find(DialogueRunner.open(gs, "mara").view(), offer) >= 0
var tev: DialogueRunner = DialogueRunner.open(gs, "tev")
var tv: Dictionary = tev.view()
checks["not offered by another NPC"] = _find(tv, offer) == -1
var all: Array = DictUtil.array_of(tv["choices"])
checks["no tree: greeting + farewell"] = tev.start == DialogueRunner.ROOT and str(tv["text"]) == DialogueRules.fill(DialogueRules.service_line(R, "greeting"), {"npc": "Tev", "gold": gs.party.gold}) and str(DictUtil.dict_of(all.back()).get("text", "")) == _line("farewell")
var _s: String = QuestRunner.set_flag(gs, "dr.swap", true)
tev = DialogueRunner.open(gs, "tev")
checks["a giver variant moves the offer"] = _find(tev.view(), offer) >= 0 and _find(DialogueRunner.open(gs, "mara").view(), offer) == -1
var went: Dictionary = tev.choose(_find(tev.view(), offer))
var ov: Dictionary = tev.view()
checks["offer node shows the description"] = DictUtil.to_bool(went["ok"]) and str(ov["text"]) == "Raise the signal for us." and _find(ov, _line("accept")) == 0 and _find(ov, _line("decline")) == 1
checks["decline returns to start"] = DictUtil.to_bool(tev.choose(1)["ok"]) and tev.node_id == tev.start and QuestRunner.status(gs, _q("q_offer")) == "available"
var _o: Dictionary = tev.choose(_find(tev.view(), offer))
checks["accept starts it"] = DictUtil.to_bool(tev.choose(0)["ok"]) and QuestRunner.status(gs, _q("q_offer")) == "active" and tev.node_id == tev.start
var _r: String = QuestRunner.set_flag(gs, "dr.signal", true)
var done: String = _line("turn_in", "q_offer")
checks["ready shows at the effective turn-in only"] = QuestRunner.status(gs, _q("q_offer")) == "ready" and _find(DialogueRunner.open(gs, "tev").view(), done) >= 0 and _find(DialogueRunner.open(gs, "mara").view(), done) == -1
gs.clear_flag("dr.swap")
var _c: Array[String] = QuestRunner.refresh(gs)
checks["turn-in follows the variant back"] = _find(DialogueRunner.open(gs, "mara").view(), done) >= 0 and _find(DialogueRunner.open(gs, "tev").view(), done) == -1
checks["shop service at the keeper"] = _find(DialogueRunner.open(gs, "hest").view(), _line("shop")) >= 0 and _find(DialogueRunner.open(gs, "mara").view(), _line("shop")) == -1
var hest: DialogueRunner = DialogueRunner.open(gs, "hest")
var shop_res: Dictionary = hest.choose(_find(hest.view(), _line("shop")))
checks["shop service opens the shop"] = str(shop_res.get("shop", "")) == _q("s_hest") and not DictUtil.to_bool(hest.choose(0)["ok"]) and DictUtil.to_bool(hest.close_shop()["ok"]) and hest.node_id == hest.start and hest.shop_qid.is_empty()
return _report("Check 9: services (offers at the resolved giver, variants move them, accept / decline, turn-in at the effective turn-in, shop)", checks)
```

### `static _test_enforcement() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var r: DialogueRunner = DialogueRunner.open(gs, "mara")
var before: Dictionary = gs.to_dict()
var res: Dictionary = r.choose(_find(r.view(), "Start Tev's"))
checks["authored start_quest of another giver refused"] = not DictUtil.to_bool(res["ok"]) and JsonUtil.strings(res["errors"]) == ["Only Tev gives this quest."] and _same(gs, before)
var _j: Array[String] = QuestRunner.start(gs, _q("q_journal"))
var _jf: String = QuestRunner.set_flag(gs, "dr.j", true)
checks["journal turn-in without giver / turn-in works"] = DictUtil.to_bool(DialogueRules.complete_from_journal(gs, _q("q_journal"))["ok"]) and QuestRunner.status(gs, _q("q_journal")) == "completed"
var _o: Array[String] = QuestRunner.start(gs, _q("q_offer"))
var _of: String = QuestRunner.set_flag(gs, "dr.signal", true)
var refused: Dictionary = DialogueRules.complete_from_journal(gs, _q("q_offer"))
checks["journal turn-in refused with a turn-in NPC"] = not DictUtil.to_bool(refused["ok"]) and JsonUtil.strings(refused["errors"]) == ["This quest is turned in to Mara."] and QuestRunner.status(gs, _q("q_offer")) == "ready"
return _report("Check 10: enforcement (start_quest only at the giver; journal turn-in only without NPCs)", checks)
```

### `static _test_turn_in() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var _s: Array[String] = QuestRunner.start(gs, _q("q_hest"))
var _f: String = QuestRunner.set_flag(gs, "dr.hest", true)
var r: DialogueRunner = DialogueRunner.open(gs, "hest")
var hand: int = _find(r.view(), "Hand it over")
var before: Dictionary = gs.to_dict()
var pend: Dictionary = r.choose(hand)
checks["pending lists the offers"] = str(pend.get("pending", "")) == "turn_in" and JsonUtil.strings(pend.get("offers", [])) == QuestRunner.offers(gs, _q("q_hest")) and JsonUtil.strings(pend.get("offers", [])).has(_q("l_gem")) and DictUtil.to_int(pend.get("picks_allowed", 0)) == 1 and _same(gs, before)
checks["busy while pending"] = not DictUtil.to_bool(r.choose(hand)["ok"]) and DictUtil.to_bool(r.view()["pending"])
checks["bad pick changes nothing"] = not DictUtil.to_bool(r.confirm_turn_in("", [_q("nope")] as Array[String])["ok"]) and _same(gs, before) and DictUtil.to_bool(r.view()["pending"])
checks["cancel keeps everything"] = DictUtil.to_bool(r.cancel()["ok"]) and _same(gs, before) and not DictUtil.to_bool(r.view()["pending"]) and r.node_id == "counter"
var _p: Dictionary = r.choose(hand)
var gold0: int = gs.party.gold
var reward: int = DictUtil.to_int(DictUtil.dict_of(QuestCatalog.get_entry("t_gold")["rewards"]).get("gold", 0))
var done: Dictionary = r.confirm_turn_in("", [_q("l_gem")] as Array[String])
checks["valid pick completes"] = DictUtil.to_bool(done["ok"]) and QuestRunner.status(gs, _q("q_hest")) == "completed" and gs.loot_claimed.has(_q("l_gem"))
checks["trailing set_flag applied"] = gs.get_flag("dr.thanked", false) == true and r.node_id == "counter"
checks["quest gold reaches the party once"] = reward > 0 and gs.party.gold == gold0 + reward and DictUtil.to_int(DictUtil.dict_of(done.get("granted", {})).get("gold", 0)) == reward
checks["no second grant"] = not DictUtil.to_bool(QuestRunner.complete(gs, _q("q_hest"))["ok"]) and gs.party.gold == gold0 + reward
return _report("Check 11: turn-in (pending offers, bad pick / cancel change nothing, confirm completes then trailing effects, gold once)", checks)
```

### `static _marker(id: String) -> Dictionary`
```logic
for m: Dictionary in ExploreRunner.markers_of(WorldCatalog.resolve(R, "locations", "yard")):
	if str(m["id"]) == id:
		return m
return {}
```

### `static _test_gone() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var loc: String = _q("yard")
var hest_offer: String = _line("offer", "q_hest")
checks["present before"] = DialogueRunner.open(gs, "hest").errors.is_empty() and ExploreRunner.presence(gs, loc, _marker("npc_hest")) == "present" and ShopRules.is_open(gs, "s_hest") and _find(DialogueRunner.open(gs, "hest").view(), hest_offer) >= 0
var _g: String = QuestRunner.set_flag(gs, str(HEST_GONE["key"]), true)
var r: DialogueRunner = DialogueRunner.open(gs, "hest")
checks["gone: can't be opened"] = r.closed and r.errors == (["Hest is gone."] as Array[String])
checks["gone: npc marker absent"] = ExploreRunner.presence(gs, loc, _marker("npc_hest")) == "absent" and ExploreRunner.presence(gs, loc, _marker("npc_mara")) == "present"
checks["gone: keeper's shop closed"] = not ShopRules.is_open(gs, "s_hest") and ShopRules.offers(gs, "s_hest").is_empty() and not DictUtil.to_bool(ShopRules.buy(gs, "s_hest", "item", I25)["ok"])
checks["gone: services never generated elsewhere"] = _find(DialogueRunner.open(gs, "mara").view(), hest_offer) == -1 and _find(DialogueRunner.open(gs, "tev").view(), hest_offer) == -1
gs.clear_flag(str(HEST_GONE["key"]))
checks["cleared: restored"] = DialogueRunner.open(gs, "hest").errors.is_empty() and ExploreRunner.presence(gs, loc, _marker("npc_hest")) == "present" and ShopRules.is_open(gs, "s_hest")
return _report("Check 12: gone (no conversation, npc marker absent, shop closed, services hidden; clearing restores)", checks)
```

### `static _test_node_effects() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.active_ids = [str(_rx["hi"])] as Array[String]
var r: DialogueRunner = DialogueRunner.open(gs, "mara")
var gold0: int = gs.party.gold
var first: Dictionary = r.choose(_find(r.view(), "Ask"))
checks["node effects run when shown"] = DictUtil.to_bool(first["ok"]) and r.node_id == "lore" and gs.party.gold == gold0 + 1 and DictUtil.to_bool(r.view()["next"])
checks["advance follows next"] = DictUtil.to_bool(r.advance()["ok"]) and r.node_id == "hello"
var _again: Dictionary = r.choose(_find(r.view(), "Ask"))
checks["and again each time"] = gs.party.gold == gold0 + 2
var _back: Dictionary = r.advance()
var gold1: int = gs.party.gold
var gift: int = _find(r.view(), "Gift once")
checks["repeat-once choice shows once"] = gift >= 0 and DictUtil.to_bool(r.choose(gift)["ok"]) and gs.party.gold == gold1 + GIFT and _find(r.view(), "Gift once") == -1
checks["a choice node needs a choice"] = not DictUtil.to_bool(r.advance()["ok"])
var bye: Dictionary = r.choose(_find(r.view(), _line("farewell")))
checks["farewell ends"] = DictUtil.to_bool(bye.get("end", false)) and r.closed and DictUtil.to_bool(r.view()["end"]) and DictUtil.array_of(r.view()["choices"]).is_empty() and not DictUtil.to_bool(r.choose(0)["ok"])
return _report("Check 13: node effects run each time shown; unless + set_flag shows once", checks)
```

### `static _test_prices() -> bool`
```logic
var checks: Dictionary = {}
for mults: Array in [[1.0, 0.5], [1.15, 0.85]]:
	var shop: Dictionary = {"buy_mult": mults[0], "sell_mult": mults[1]}
	var bm: int = roundi(DictUtil.to_float(mults[0]) * 100.0)
	var sm: int = roundi(DictUtil.to_float(mults[1]) * 100.0)
	for value: int in [15, 25]:
		var label: String = "%d at %s/%s" % [value, str(mults[0]), str(mults[1])]
		@warning_ignore("integer_division")
		var buy_oracle: int = (value * bm + 99) / 100   # integer ceil, no floats (DLG1-F)
		@warning_ignore("integer_division")
		var sell_oracle: int = (value * sm) / 100        # integer floor
		checks["buy ceil " + label] = ShopRules.buy_price(shop, value) == buy_oracle
		checks["sell floor " + label] = ShopRules.sell_price(shop, value) == sell_oracle
	checks["fixed price ignores %s" % str(mults[0])] = ShopRules.buy_price(shop, 15, FIXED) == FIXED
	checks["minimum buy 1 at %s" % str(mults[0])] = ShopRules.buy_price(shop, 0) == 1
var gs: GameState = _gs()
var entry: Dictionary = WorldCatalog.resolve(R, "shops", "s_hest")
var prices: Dictionary = {}
for o: Dictionary in ShopRules.offers(gs, "s_hest"):
	prices[str(o["id"])] = DictUtil.to_int(o["price"])
checks["offers use the rule"] = DictUtil.to_int(prices.get(I15, -1)) == ShopRules.buy_price(entry, 15) and DictUtil.to_int(prices.get(str(_rx["weapon"]), -1)) == FIXED and DictUtil.to_int(prices.get(I0, -1)) == 1 and DictUtil.to_int(prices.get(_q("l_ring"), -1)) == LOOT_PRICE
return _report("Check 14: prices (ceil buy, floor sell, fixed price, minimum 1)", checks)
```

### `static _left(gs: GameState, id: String) -> int`
```logic
for o: Dictionary in ShopRules.offers(gs, "s_hest"):
	if str(o["id"]) == id:
		return DictUtil.to_int(o["left"])
return -99
```

### `static _test_stock() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.gold = 1000
var price: int = ShopRules.buy_price(WorldCatalog.resolve(R, "shops", "s_hest"), 15)
var one: Dictionary = ShopRules.buy(gs, "s_hest", "item", I15)
var two: Dictionary = ShopRules.buy(gs, "s_hest", "item", I15)
var before: Dictionary = gs.to_dict()
var three: Dictionary = ShopRules.buy(gs, "s_hest", "item", I15)
checks["count-2 entry sells twice"] = DictUtil.to_bool(one["ok"]) and DictUtil.to_bool(two["ok"]) and gs.party.gold == 1000 - 2 * price and DictUtil.to_int(gs.party.stash.get(I15, 0)) == 2
checks["then sold out"] = not DictUtil.to_bool(three["ok"]) and JsonUtil.strings(three["errors"]) == ["Sold out."] and _same(gs, before) and _left(gs, I15) == 0
checks["gs.shops counts it"] = DictUtil.to_int(DictUtil.dict_of(gs.shops.get(_q("s_hest"), {})).get(I15, 0)) == 2
var back: GameState = GameState.from_dict(DictUtil.dict_of(JSON.parse_string(JSON.stringify(gs.to_dict()))))
checks["gs.shops round-trips"] = JsonUtil.deep_equal(back.shops, gs.shops) and back.validate().is_empty()
checks["unlimited never runs out"] = DictUtil.to_bool(ShopRules.buy(gs, "s_hest", "item", I25, 3)["ok"]) and DictUtil.to_bool(ShopRules.buy(gs, "s_hest", "item", I25, 3)["ok"]) and _left(gs, I25) == ShopRules.UNLIMITED and not DictUtil.dict_of(gs.shops.get(_q("s_hest"), {})).has(I25)
gs.party.gold = 0
before = gs.to_dict()
var broke: Dictionary = ShopRules.buy(gs, "s_hest", "item", I25)
checks["not enough gold changes nothing"] = JsonUtil.strings(broke["errors"]) == ["Not enough gold."] and _same(gs, before)
checks["not for sale"] = not DictUtil.to_bool(ShopRules.buy(gs, "s_hest", "item", QUEST_ITEM)["ok"])
return _report("Check 15: stock (limited sells out, gs.shops round-trips, unlimited, not enough gold)", checks)
```

### `static _test_loot_sale() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.gold = 1000
var ring: String = _q("l_ring")
checks["quest offers the loot first"] = QuestRunner.offers(gs, _q("q_offer")).has(ring) and _left(gs, ring) == 1
checks["failing loot requires_flags not listed"] = _left(gs, _q("l_locked")) == -99
checks["loot count 2 refused"] = not DictUtil.to_bool(ShopRules.buy(gs, "s_hest", "loot", ring, 2)["ok"])
var held: int = DictUtil.to_int(gs.party.stash.get(I25, 0))
var bought: Dictionary = ShopRules.buy(gs, "s_hest", "loot", ring)
checks["buying claims it"] = DictUtil.to_bool(bought["ok"]) and gs.loot_claimed.has(ring) and gs.party.gold == 1000 - LOOT_PRICE and DictUtil.to_int(gs.party.stash.get(I25, 0)) == held + 1
checks["the quest no longer offers it"] = not QuestRunner.offers(gs, _q("q_offer")).has(ring)
checks["the shop no longer lists it"] = _left(gs, ring) == -99 and not DictUtil.to_bool(ShopRules.buy(gs, "s_hest", "loot", ring)["ok"])
var _v: String = gs.set_flag("dr.vip", true)
checks["listed once its flag holds"] = _left(gs, _q("l_locked")) == 1
return _report("Check 16: loot sale (claims the loot, leaves the quest pool and the shop; requires_flags respected)", checks)
```

### `static _test_selling() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
for id: String in [QUEST_ITEM, str(_rx["weapon"]), I0]:
	gs.party.add_to_stash(id, 1)
gs.party.add_to_stash(I15, 2)
var listed: Array[String] = []
for row: Dictionary in ShopRules.sell_list(gs, "s_hest"):
	listed.append(str(row["item"]))
checks["only bought types with a price >= 1"] = listed == ([I15] as Array[String])
checks["sell_mult 0 lists nothing"] = ShopRules.sell_list(gs, "s_zero").is_empty()
var gold0: int = gs.party.gold
var sold: Dictionary = ShopRules.sell(gs, "s_hest", I15)
checks["gold rises by the sell price"] = DictUtil.to_bool(sold["ok"]) and gs.party.gold == gold0 + ShopRules.sell_price(WorldCatalog.resolve(R, "shops", "s_hest"), 15) and DictUtil.to_int(gs.party.stash.get(I15, 0)) == 1
var before: Dictionary = gs.to_dict()
checks["QUEST items refused"] = not DictUtil.to_bool(ShopRules.sell(gs, "s_hest", QUEST_ITEM)["ok"]) and _same(gs, before)
checks["types outside buys refused"] = not DictUtil.to_bool(ShopRules.sell(gs, "s_hest", str(_rx["weapon"]))["ok"]) and _same(gs, before)
checks["more than held refused"] = not DictUtil.to_bool(ShopRules.sell(gs, "s_hest", I15, 2)["ok"]) and _same(gs, before)
checks["sold items never restock"] = _left(gs, I15) == 2
return _report("Check 17: selling (QUEST and other types not listed, sell_mult 0, gold rises)", checks)
```

### `static _test_gold() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
checks["NewGame uses start_gold"] = gs.party.gold == START_GOLD and DialogueRules.start_gold(R) == START_GOLD
var kid: GameState = GameState.of(NewGame.create(KID, [str(_rx["lo"])] as Array[String], 7)["state"])
checks["NewGame inherits start_gold"] = kid != null and kid.party.gold == START_GOLD
var _m: Array[String] = WorldCatalog.update_manifest(KID, {"start_gold": 0})
var kid0: GameState = GameState.of(NewGame.create(KID, [str(_rx["lo"])] as Array[String], 7)["state"])
checks["a child's own start_gold wins"] = kid0 != null and kid0.party.gold == 0
var _m2: Array[String] = WorldCatalog.update_manifest(KID, {"start_gold": -1})
var entered: Dictionary = ExploreRunner.enter(gs, "yard")
var chest_at: Vector2i = MapRules.anchor_pos(MapCatalog.get_entry("m_dg"), "a_chest")
gs.position = chest_at + Vector2i(0, -1)
var gold0: int = gs.party.gold
var opened: Dictionary = ExploreRunner.interact(gs, Vector2i(0, 1))
checks["chest gold on first open"] = DictUtil.to_bool(entered["ok"]) and str(opened.get("kind", "")) == "chest" and DictUtil.to_int(opened.get("gold", 0)) == CHEST_GOLD and gs.party.gold == gold0 + CHEST_GOLD
var again: Dictionary = ExploreRunner.interact(gs, Vector2i(0, 1))
checks["not on the second"] = DictUtil.to_bool(again.get("empty", false)) and gs.party.gold == gold0 + CHEST_GOLD
checks["RewardGrant gold echoed"] = DictUtil.to_int(DictUtil.dict_of(RewardGrant.apply(gs, {"gold": 3}, "")["granted"]).get("gold", 0)) == 3 and gs.party.gold == gold0 + CHEST_GOLD + 3
gs.party.gold = PartyState.GOLD_MAX - 1
checks["gold capped at GOLD_MAX"] = DictUtil.to_bool(RewardGrant.apply(gs, {"gold": 5}, "")["ok"]) and gs.party.gold == PartyState.GOLD_MAX and gs.validate().is_empty()
var take_errs: Array[String] = RewardGrant.take(gs, {I25: 1}, 0)
checks["take validates first"] = take_errs.size() == 1 and take_errs[0].begins_with("Not enough") and gs.party.gold == PartyState.GOLD_MAX
return _report("Check 18: gold (quest once in check 11, chest first open only, NewGame inherited start_gold, cap)", checks)
```

### `static _test_fight() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
var _e: Dictionary = ExploreRunner.enter(gs, "yard")
gs.position = Vector2i(2, 2)
var _w: Array[String] = QuestRunner.start(gs, _q("q_win"))
var r: DialogueRunner = DialogueRunner.open(gs, "hest")
var res: Dictionary = r.choose(_find(r.view(), "Fight"))
var fight: Dictionary = DictUtil.dict_of(res.get("fight", {}))
checks["returns the fight and closes"] = DictUtil.to_bool(res["ok"]) and JsonUtil.deep_equal(fight, {"source": "dialogue", "encounter": _q("enc_a"), "character": _q("hest")}) and r.closed
checks["effects before the fight applied"] = gs.get_flag("dr.fought", false) == true
var expected: GameState = GameState.from_dict(gs.to_dict())
var _wk: Dictionary = ExploreRunner.wake(expected)
var lost: Dictionary = DialogueRunner.after_fight(gs, fight, "lost")
checks["lost wakes by the EXP1 rule"] = DictUtil.to_bool(lost["ok"]) and gs.equals(expected) and gs.position == MapRules.anchor_pos(MapCatalog.get_entry("m_dg"), MapRules.SPAWN)
var won: Dictionary = DialogueRunner.after_fight(gs, fight, "won")
checks["won advances a waiting win_encounter quest"] = DictUtil.to_bool(won["ok"]) and JsonUtil.strings(won["advanced"]).has(_q("q_win")) and QuestRunner.status(gs, _q("q_win")) == "ready"
checks["bad outcome refused"] = not DictUtil.to_bool(DialogueRunner.after_fight(gs, fight, "draw")["ok"])
return _report("Check 19: fight effect (returns the fight, closes; lost wakes, won advances quests)", checks)
```

### `static _write_raw(slot: int, text: String) -> void`
```logic
DirAccess.make_dir_recursive_absolute(SaveGame.base_dir())
var f := FileAccess.open(SaveGame.path_for(slot), FileAccess.WRITE)
f.store_string(text)
f.close()
```

### `static _test_save_v4() -> bool`
```logic
var checks: Dictionary = {}
var gs: GameState = _gs()
gs.party.gold = 123
gs.shops = {_q("s_hest"): {I15: 1}}
var v3: Dictionary = gs.to_dict()
v3.erase("shops")
DictUtil.dict_of(v3["party"]).erase("gold")
_write_raw(1, JSON.stringify({"format": SaveGame.FORMAT, "version": 3, "saved_at": 0, "game_state": v3}))
var read: Dictionary = SaveGame.read(1)
var st: GameState = GameState.of(read["state"])
checks["v3 migrates to gold 0 and shops {}"] = DictUtil.to_bool(read["ok"]) and st != null and st.party.gold == 0 and st.shops.is_empty() and JsonUtil.strings(read["warnings"]).is_empty()
var written: bool = SaveGame.write(gs, 2).is_empty()
var back: GameState = GameState.of(SaveGame.read(2)["state"])
checks["v4 round trip"] = written and back != null and back.equals(gs) and DictUtil.to_int(DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(SaveGame.path_for(2))))["version"]) == SaveGame.VERSION and SaveGame.VERSION >= 4   # 5 since TRAV1-L
var negative: GameState = GameState.from_dict(gs.to_dict())
negative.party.gold = -1
var zero: GameState = GameState.from_dict(gs.to_dict())
zero.shops = {_q("s_hest"): {I15: 0}}
var bad_qid: GameState = GameState.from_dict(gs.to_dict())
bad_qid.shops = {"s_hest": {I15: 1}}
var raw: Dictionary = gs.to_dict()
DictUtil.dict_of(raw["party"])["gold"] = 1.5
var not_whole: GameState = GameState.from_dict(raw)
var raw_shops: Dictionary = gs.to_dict()
raw_shops["shops"] = [1]
var not_object: GameState = GameState.from_dict(raw_shops)
for pair: Array in [["negative gold", negative], ["sold count 0", zero], ["unqualified shop id", bad_qid], ["gold not whole", not_whole], ["shops not an object", not_object]]:
	var g: GameState = GameState.of(pair[1])
	checks["%s fails validate and write" % str(pair[0])] = not g.validate().is_empty() and not SaveGame.write(g, 3).is_empty()
checks["nothing written for refusals"] = not SaveGame.exists(3)
return _report("Check 20: save v4 (v3 migrates; bad gold / shops fail validate and write)", checks)
```

