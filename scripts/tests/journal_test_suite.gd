class_name JournalTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Journal Test Suite (UI3-L, S8)
# Acceptance checks 1-6 of docs/specs/UI3_L.md on JournalModel (check 7 = the existing quest and save suites).
# Fixture: a sandbox fork of the shipped worlds (WorldCatalog.fork_to) and a NewGame of the Ashlands as CurrentGame;
# quests advance through QuestRunner events only; expected values come from QuestRunner reads, not literals.
# ==============================================================================

const TAG: String = "Journal"
static var SANDBOX: String = TestHelpers.sandbox("journal")
const FIRST: String = "hold_the_courtyard"
const SECOND: String = "cinder_gate_watch"
const DEATH_FLAG: String = "ashlands.mara_fallen"

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING JOURNAL TEST (UI3-L)                    ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = _report("Sandbox world fork", {"forked": TestHelpers.fork_worlds(SANDBOX + "/worlds")})
	if ok:
		ok = _test_flow() and ok
		ok = _test_hidden() and ok
		ok = _test_variant() and ok
		ok = _test_offers() and ok
		ok = _test_read_only() and ok
		ok = _test_missing() and ok
	CurrentGame.end()
	TestHelpers.cleanup_worlds(SANDBOX)
	return ok

static func _fresh() -> GameState:
	var gs: GameState = GameState.of(NewGame.create_for_world("ashlands", 5)["state"])
	CurrentGame.start(gs)
	return gs

static func _qid(gs: GameState, ref: String) -> String:
	return str(WorldCatalog.resolve(gs.world_id, "quests", ref).get("_qid", ""))

# Meets the quest's objectives one by one through the runner's own events.
static func _meet(gs: GameState, qid: String) -> void:
	for _i: int in range(12):
		var obj: Dictionary = QuestRunner.current_objective(gs, qid)
		if obj.is_empty():
			return
		match str(obj["kind"]):
			"reach_location":
				var _a: Array[String] = QuestRunner.on_location_reached(gs, str(obj["target"]))
			"win_encounter":
				var _b: Array[String] = QuestRunner.on_encounter_won(gs, str(obj["target"]))
			"flag":
				var _c: String = QuestRunner.set_flag(gs, str(obj["target"]), obj["value"])

static func _group_of(jm: JournalModel, qid: String) -> String:
	for g: Dictionary in jm.groups():
		if JsonUtil.strings(g["quests"]).has(qid):
			return str(g["status"])
	return ""

# Check 1: nothing shown before a start; Active with objective 1, Ready, Completed with done == total.
static func _test_flow() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	var qid: String = _qid(gs, FIRST)
	var jm := JournalModel.new()
	checks["fresh: no groups"] = jm.groups().is_empty()
	checks["fresh: not shown"] = _group_of(jm, qid).is_empty() and QuestRunner.status(gs, qid) == "available"
	checks["started"] = QuestRunner.start(gs, qid).is_empty()
	jm.refresh()
	var first_text: String = str(DictUtil.dict_of(DictUtil.array_of(QuestRunner.resolved(gs, qid)["steps"])[0])["text"])
	checks["active with objective 1"] = _group_of(jm, qid) == "active" and str(jm.entry(qid)["objective"]) == first_text and DictUtil.to_int(jm.entry(qid)["objectives_done"]) == 0
	_meet(gs, qid)
	jm.refresh()
	checks["ready"] = _group_of(jm, qid) == "ready" and jm.groups()[0]["status"] == "ready" and str(jm.entry(qid)["objective"]).is_empty()
	var offers: Array[String] = QuestRunner.offers(gs, qid)
	var picks: Array[String] = offers.slice(0, QuestRunner.picks_allowed(gs, qid))
	checks["completed"] = DictUtil.to_bool(QuestRunner.complete(gs, qid, gs.party.members[0].id, picks)["ok"])
	jm.refresh()
	var e: Dictionary = jm.entry(qid)
	checks["completed group, all done"] = _group_of(jm, qid) == "completed" and DictUtil.to_int(e["objectives_done"]) == DictUtil.to_int(e["objectives_total"]) and DictUtil.to_int(e["objectives_total"]) > 0
	checks["no offers once completed"] = JsonUtil.strings(e["offers"]).is_empty()
	checks["rewards listed"] = not JsonUtil.strings(e["rewards"]).is_empty()
	return _report("Check 1: active, ready, completed", checks)

# Check 2: available and locked quests never appear; every shown quest has a shown status.
static func _test_hidden() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	var jm := JournalModel.new()
	var hidden_ok: bool = true
	var seen: Array[String] = []
	for e: Dictionary in WorldCatalog.merged_entries(gs.world_id, "quests"):
		var qid: String = str(e["_qid"])
		var st: String = QuestRunner.status(gs, qid)
		seen.append(st)
		hidden_ok = hidden_ok and (jm.shown().has(qid) == ["ready", "active", "blocked", "completed"].has(st))
	checks["status decides"] = hidden_ok
	checks["an available and a locked quest exist"] = seen.has("available") and seen.has("locked")
	checks["entry of a hidden quest is empty"] = jm.entry(_qid(gs, SECOND)).is_empty()
	return _report("Check 2: unstarted quests stay hidden", checks)

# Check 3: the death variant changes the giver or puts the quest on hold, as QuestRunner resolves it.
static func _test_variant() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	var qid: String = _qid(gs, FIRST)
	var _s: Array[String] = QuestRunner.start(gs, qid)
	var jm := JournalModel.new()
	var before: Dictionary = jm.entry(qid)
	checks["flag set"] = QuestRunner.set_flag(gs, DEATH_FLAG, true).is_empty()
	jm.refresh()
	var st: String = QuestRunner.status(gs, qid)
	var after: Dictionary = jm.entry(qid)
	var q: Dictionary = QuestRunner.resolved(gs, qid)
	checks["group follows status"] = _group_of(jm, qid) == st
	if st == "blocked":
		checks["on hold shows text only"] = str(after["description"]) == str(q.get("description", "")) and str(after["giver"]).is_empty()
	else:
		checks["giver follows the variant"] = str(after["giver"]) == JournalModel.character_name(gs.world_id, str(q["giver"])) and str(after["description"]) == str(q.get("description", ""))
		checks["something changed"] = str(after["giver"]) != str(before["giver"]) or str(after["description"]) != str(before["description"])
	return _report("Check 3: death variant re-resolves", checks)

# Check 4: offers mapped to names; loot claimed by another quest disappears.
static func _test_offers() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	var first: String = _qid(gs, FIRST)
	var second: String = _qid(gs, SECOND)
	var _s: Array[String] = QuestRunner.start(gs, first)
	_meet(gs, first)
	var shared: String = ""
	for lqid: String in QuestRunner.offers(gs, first):
		if QuestRunner.resolved(gs, second).has("loot_pool") and shared.is_empty():
			for ref: String in JsonUtil.strings(QuestRunner.resolved(gs, second)["loot_pool"]):
				if str(WorldCatalog.resolve(gs.world_id, "loot", ref).get("_qid", "")) == lqid:
					shared = lqid
	var jm := JournalModel.new()
	var names: Array[String] = []
	for lqid: String in QuestRunner.offers(gs, first):
		names.append(str(WorldCatalog.resolve(gs.world_id, "loot", lqid)["name"]))
	checks["offers by name"] = JsonUtil.strings(jm.entry(first)["offers"]) == names and DictUtil.to_int(jm.entry(first)["picks_allowed"]) == QuestRunner.picks_allowed(gs, first)
	checks["a shared loot entry"] = not shared.is_empty()
	var shared_name: String = str(WorldCatalog.resolve(gs.world_id, "loot", shared).get("name", ""))
	var picks: Array[String] = [shared]
	checks["claimed by the first"] = DictUtil.to_bool(QuestRunner.complete(gs, first, gs.party.members[0].id, picks)["ok"])
	var _t: Array[String] = QuestRunner.start(gs, second)
	jm.refresh()
	var second_offers: Array[String] = QuestRunner.offers(gs, second)
	var second_names: Array[String] = []
	for lqid: String in second_offers:
		second_names.append(str(WorldCatalog.resolve(gs.world_id, "loot", lqid).get("name", "")))
	checks["journal offers = QuestRunner.offers after the claim"] = JsonUtil.strings(jm.entry(second)["offers"]) == second_names and not second_offers.has(shared)
	checks["gone from the second"] = _group_of(jm, second) == "active" and not JsonUtil.strings(jm.entry(second)["offers"]).has(shared_name)
	return _report("Check 4: offers and claimed loot", checks)

# Check 5: the model never changes the state.
static func _test_read_only() -> bool:
	var gs: GameState = _fresh()
	var _s: Array[String] = QuestRunner.start(gs, _qid(gs, FIRST))
	CurrentGame.mark_saved(-1)   # the dirty check compares against this clean copy
	var before: Dictionary = gs.to_dict()
	var jm := JournalModel.new()
	for _i: int in range(3):
		jm.refresh()
		for g: Dictionary in jm.groups():
			for qid: String in JsonUtil.strings(g["quests"]):
				var _e: Dictionary = jm.entry(qid)
		jm.move(1)
	return _report("Check 5: read only", {"state unchanged": JsonUtil.deep_equal(gs.to_dict(), before), "not dirty": not CurrentGame.is_dirty()})

# Check 6: unresolved ids show in brackets; selection moves within the shown list; no game, empty journal.
static func _test_missing() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	checks["unknown character"] = JournalModel.character_name(gs.world_id, "ps_nobody") == "[ps_nobody]"
	checks["unknown item"] = JournalModel.reward_lines({"items": {"ps_no_item": 2}}) == ["[ps_no_item] x2"]
	checks["empty ref"] = JournalModel.character_name(gs.world_id, "").is_empty()
	var _s: Array[String] = QuestRunner.start(gs, _qid(gs, FIRST))
	var jm := JournalModel.new()
	checks["selection defaults"] = jm.selected == jm.shown()[0]
	jm.move(-5)
	checks["move clamps"] = jm.selected == jm.shown()[0]
	checks["unknown select refused"] = not jm.select("nope").is_empty()
	CurrentGame.end()
	jm.refresh()
	checks["no game"] = jm.groups().is_empty() and jm.selected.is_empty()
	return _report("Check 6: missing data and selection", checks)
