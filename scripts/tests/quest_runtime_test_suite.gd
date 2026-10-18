class_name QuestRuntimeTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Quest Runtime Test Suite (S8, spec M4-L slice 2; checks 5-15 and 17)
# QuestRunner statuses and flow, flag objectives, RewardGrant, complete_flags, auto_start, variant resolution,
# rebinding, available:false (blocked / unavailable), rewards_from, inheritance keys, save v2 and the 1 -> 2
# migration, and world loot offers / picks / claims. Fixtures: templates in a sandbox QuestCatalog file and two
# hand-built worlds in an empty sandbox world root; the shipped catalog is only read (ids picked by rule).
# Expected numbers come from the fixture templates, never from shipped content.
# ==============================================================================

const TAG: String = "QuestRun"
static var SANDBOX: String = TestHelpers.sandbox("quest_runtime")
const RUN: String = "qa_run"
const KID: String = "qa_run_kid"
const HOME: String = "qa_x_home"   # QUESTX: connects FAR
const FAR: String = "qa_x_far"
const XKID: String = "qa_x_kid"   # extends HOME
const ISO: String = "qa_x_iso"   # neither extended nor connected

static var _fx: Dictionary = {}

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING QUEST RUNTIME TEST (M4-L runtime)       ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	QuestCatalog.use_path(SANDBOX + "/quests.json")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var ok: bool = _build()
	if ok:
		ok = _test_status() and ok
		ok = _test_flow() and ok
		ok = _test_flag_objectives() and ok
		ok = _test_rewards() and ok
		ok = _test_complete_flags_and_auto() and ok
		ok = _test_variants_resolve() and ok
		ok = _test_rebinding() and ok
		ok = _test_available_false() and ok
		ok = _test_ready_flag() and ok
		ok = _test_rewards_from() and ok
		ok = _test_inheritance() and ok
		ok = _test_save() and ok
		ok = _test_loot() and ok
		ok = _test_cross_world() and ok
	SaveGame.reset_base_dir()
	SaveMigrations.reset()
	QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	TestHelpers.cleanup_worlds(SANDBOX)
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

static func _party_unit() -> String:
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and str(e.get("stat_mode", "derived")) == "derived":
			return str(e["id"])
	return ""

static func _fresh(world: String = RUN) -> GameState:
	var uid: String = str(_fx["unit"])
	var res: Dictionary = NewGame.create(world, [uid, uid] as Array[String])
	return GameState.of(res["state"])

static func _step_flag(text: String, key: String) -> Dictionary:
	return {"text": text, "flag": key}

# A one-flag-objective quest on rt_one; `extra` replaces fields.
static func _one(id: String, flag: String, extra: Dictionary = {}) -> Dictionary:
	var q: Dictionary = {"id": id, "name": id, "template": "rt_one", "giver": "c1", "steps": [_step_flag("Flag", flag)]}
	q.merge(extra, true)
	return q

static func _two_steps(loc: String, enc: String) -> Array:
	return [{"text": "Reach %s" % loc, "location": loc}, {"text": "Win %s" % enc, "encounter": enc}]

static func _build() -> bool:
	var errors: Array[String] = []
	var items: Array[String] = ContentCatalog.ids("items")
	var passives: Array[String] = ContentCatalog.ids("passives")
	var uid: String = _party_unit()
	_fx = {"unit": uid, "items": items, "passives": passives}
	errors.append_array(WorldCatalog.create_world(RUN, "QA Run"))
	errors.append_array(WorldCatalog.create_world(KID, "QA Run Kid", [RUN]))
	# Skill rewards: non-basic, non-system family heads the party unit does not own at its level.
	var probe: GameState = _fresh()
	var owned: Array[String] = probe.party.members[0].owned_families() if probe != null else [] as Array[String]
	var skills: Array[String] = []
	for ranked: Dictionary in ContentCatalog.skill_entries():
		var head: String = str(ranked.get("id", ""))
		if skills.size() < 2 and not DictUtil.to_bool(ranked.get("system", false)) and not SkillCatalog.is_basic_family(head) and not owned.has(head) and not ContentCatalog.skill_entry(head).is_empty():
			skills.append(head)
	_fx["skills"] = skills
	var fixtures_ok: bool = probe != null and items.size() >= 2 and passives.size() >= 2 and skills.size() >= 2
	if not fixtures_ok:
		return _report("Fixtures picked by rule (party unit, 2 items, 2 passives, 2 unowned skills)", {"enough": false})
	_fx["t_two_rewards"] = {"xp": 40, "items": {items[0]: 2}, "passives": [passives[0]], "skills": [skills[0]]}
	_fx["t_alt_rewards"] = {"xp": 77, "items": {items[1]: 1}}
	_fx["t_one_rewards"] = {"xp": 3}
	for t: Dictionary in [
		{"id": "rt_two", "name": "Reach and win", "objectives": [{"kind": "reach_location"}, {"kind": "win_encounter"}], "rewards": _fx["t_two_rewards"]},
		{"id": "rt_alt", "name": "Alt rewards", "objectives": [{"kind": "reach_location"}, {"kind": "win_encounter"}], "rewards": _fx["t_alt_rewards"]},
		{"id": "rt_flags", "name": "Two flags", "objectives": [{"kind": "flag"}, {"kind": "flag"}]},
		{"id": "rt_one", "name": "One flag", "objectives": [{"kind": "flag"}], "rewards": _fx["t_one_rewards"]},
	]:
		errors.append_array(QuestCatalog.upsert(t))
	for id: String in ["l1", "l2", "l3"]:
		errors.append_array(WorldCatalog.upsert(RUN, "locations", {"id": id, "name": id}))
	for id: String in ["e1", "e2"]:
		errors.append_array(WorldCatalog.upsert(RUN, "encounters", {"id": id, "name": id}))
	for id: String in ["c1", "c2", "c3"]:
		errors.append_array(WorldCatalog.upsert(RUN, "characters", {"id": id, "name": id}))
	for l: Dictionary in [
		{"id": "lt_item", "name": "Item", "reward_kind": "item", "ref": items[1], "count": 3, "rarity": "common"},
		{"id": "lt_passive", "name": "Passive", "reward_kind": "passive", "ref": passives[1]},
		{"id": "lt_flagged", "name": "Flagged", "reward_kind": "item", "ref": items[0], "requires_flags": [{"key": "rt.found"}]},
		{"id": "lt_skill", "name": "Skill", "reward_kind": "skill", "ref": skills[1]},
	]:
		errors.append_array(WorldCatalog.upsert(RUN, "loot", l))
	var quests: Array[Dictionary] = [
		{"id": "q_flow", "name": "Flow", "template": "rt_two", "giver": "c1", "steps": _two_steps("l1", "e1"),
			"complete_flags": [{"key": "rt.flow_done"}, {"key": "rt.order", "value": 1}, {"key": "rt.order", "value": 2}]},
		_one("q_gate", "rt.gate_go", {"requires_flags": [{"key": "rt.flow_done"}]}),
		_one("q_req", "rt.req_go", {"requires": ["q_flow"]}),
		_one("q_lock", "rt.lock_go", {"requires_flags": [{"key": "rt.key"}]}),
		{"id": "q_lore", "name": "Lore", "steps": [{"text": "Just a story"}]},
		{"id": "q_flags", "name": "Flags", "template": "rt_flags", "steps": [_step_flag("A", "rt.a"), _step_flag("B", "rt.b")]},
		_one("q_strict", "rt.strict"),
		_one("q_auto", "rt.never", {"auto_start": true}),
		_one("q_auto_locked", "rt.never", {"auto_start": true, "requires_flags": [{"key": "rt.nope"}]}),
		{"id": "q_var", "name": "Var", "template": "rt_two", "giver": "c1", "description": "base", "reward": "base", "steps": _two_steps("l1", "e1"), "variants": [
			{"id": "va", "when_flags": [{"key": "rt.va"}], "set": {"giver": "c2"}},
			{"id": "vb", "when_flags": [{"key": "rt.vb"}], "set": {"giver": "c3", "reward": "B"}},
			{"id": "vc", "when_flags": [{"key": "rt.va"}], "unless_flags": [{"key": "rt.mute"}], "set": {"description": "C"}},
		]},
		{"id": "q_rebind", "name": "Rebind", "template": "rt_two", "steps": _two_steps("l1", "e1"), "variants": [
			{"id": "moved", "when_flags": [{"key": "rt.moved"}], "set": {"steps": {"0": {"location": "l2"}, "1": {"encounter": "e2"}}}},
		]},
		{"id": "q_block", "name": "Block", "template": "rt_two", "steps": _two_steps("l1", "e1"), "variants": [
			{"id": "gone", "when_flags": [{"key": "rt.gone"}], "set": {"available": false}},
		]},
		{"id": "q_rf", "name": "Rewards from", "template": "rt_two", "steps": _two_steps("l3", "e2"), "variants": [
			{"id": "alt", "when_flags": [{"key": "rt.alt"}], "set": {"rewards_from": "rt_alt"}},
		]},
		_one("q_loot1", "rt.l1", {"loot_pool": ["lt_item", "lt_passive", "lt_flagged"], "loot_picks": 2}),
		_one("q_loot2", "rt.l2", {"loot_pool": ["lt_item", "lt_passive"], "loot_picks": 1, "variants": [
			{"id": "swap", "when_flags": [{"key": "rt.swap"}], "set": {"loot_pool": ["lt_skill"]}},
		]}),
		_one("q_noloot", "rt.nl"),
	]
	for q: Dictionary in quests:
		errors.append_array(WorldCatalog.upsert(RUN, "quests", q))
	errors.append_array(QuestCatalog.upsert({"id": "rt_cross", "name": "Cross", "objectives": [{"kind": "reach_location"}, {"kind": "reach_location"}]}))
	for w: String in [HOME, FAR, ISO]:
		errors.append_array(WorldCatalog.create_world(w, w))
	errors.append_array(WorldCatalog.create_world(XKID, XKID, [HOME]))
	errors.append_array(WorldCatalog.update_manifest(HOME, {"connects": [FAR]}))
	for loc: Array in [[HOME, "h1"], [FAR, "f1"], [ISO, "i1"]]:
		errors.append_array(WorldCatalog.upsert(str(loc[0]), "locations", {"id": str(loc[1]), "name": str(loc[1])}))
	errors.append_array(WorldCatalog.upsert(HOME, "quests", {"id": "x_cross", "name": "Cross", "template": "rt_cross", "steps": [{"text": "Home", "location": "h1"}, {"text": "Far", "location": "%s:f1" % FAR}]}))
	errors.append_array(WorldCatalog.upsert(FAR, "quests", {"id": "x_cross", "name": "Far twin", "template": "rt_cross", "steps": [{"text": "A", "location": "f1"}, {"text": "B", "location": "f1"}]}))
	var ov: String = WorldCatalog.override_entry(KID, "quests", "%s:q_lock" % RUN)
	var problems: Array[String] = []
	for key: Variant in WorldCatalog.validate_world(RUN):
		if str(key).contains("/quests:") or str(key).contains("/loot:"):
			problems.append(str(key))
	return _report("Fixtures (templates, worlds, quests, loot) built and valid", {"no upsert errors": errors.is_empty(), "quests and loot valid": problems.is_empty(), "child override": ov == "q_lock"})

static func _q(id: String) -> String:
	return "%s:%s" % [RUN, id]

static func _ready_flag_quest(gs: GameState, id: String, flag: String) -> bool:
	return QuestRunner.start(gs, id).is_empty() and QuestRunner.set_flag(gs, flag, true).is_empty() and QuestRunner.status(gs, id) == "ready"

static func _ready_two(gs: GameState, id: String, loc: String, enc: String) -> bool:
	if not QuestRunner.start(gs, id).is_empty():
		return false
	QuestRunner.on_location_reached(gs, loc)
	QuestRunner.on_encounter_won(gs, enc)
	return QuestRunner.status(gs, id) == "ready"

static func _xp(gs: GameState) -> Array[int]:
	var out: Array[int] = []
	for m: UnitState in gs.party.members:
		out.append(m.total_xp)
	return out

static func _snapshot(gs: GameState) -> String:
	return JSON.stringify(gs.to_dict())

# ------------------------------------------------------------------------------
# Check 5: status
# ------------------------------------------------------------------------------

static func _test_status() -> bool:
	var gs: GameState = _fresh()
	var checks: Dictionary = {}
	checks["no requires, no flags = available"] = QuestRunner.status(gs, "q_flow") == "available"
	checks["unmet requires_flags = locked"] = QuestRunner.status(gs, "q_lock") == "locked"
	checks["unmet requires = locked"] = QuestRunner.status(gs, "q_req") == "locked"
	checks["set_flag flips to available"] = QuestRunner.set_flag(gs, "rt.key", true).is_empty() and QuestRunner.status(gs, "q_lock") == "available"
	checks["lore-only = invalid"] = QuestRunner.status(gs, "q_lore") == "invalid" and QuestRunner.resolved(gs, "q_lore").is_empty()
	checks["unknown ref = invalid"] = QuestRunner.status(gs, "no_such_quest") == "invalid"
	checks["locked can't start"] = not QuestRunner.start(gs, "q_req").is_empty() and not gs.quests.has(_q("q_req"))
	checks["quests_with available"] = QuestRunner.quests_with(gs, "available").has(_q("q_flow")) and not QuestRunner.quests_with(gs, "available").has(_q("q_req"))
	checks["bad flag key refused"] = not QuestRunner.set_flag(gs, "Bad Key", true).is_empty()
	return _report("Status: available, locked by flags / requires, flips after set_flag, lore-only and unknown are invalid", checks)

# ------------------------------------------------------------------------------
# Check 6: flow [reach_location, win_encounter]
# ------------------------------------------------------------------------------

static func _test_flow() -> bool:
	var gs: GameState = _fresh()
	var checks: Dictionary = {}
	checks["start"] = QuestRunner.start(gs, "q_flow").is_empty() and QuestRunner.status(gs, "q_flow") == "active"
	checks["stored shape"] = JsonUtil.deep_equal(gs.quests[_q("q_flow")], {"state": "active", "objective": 0})
	checks["second start refused"] = not QuestRunner.start(gs, "q_flow").is_empty()
	var cur0: Dictionary = QuestRunner.current_objective(gs, "q_flow")
	checks["objective 0 is the location"] = str(cur0.get("kind", "")) == "reach_location" and str(cur0.get("target", "")) == _q("l1") and DictUtil.to_int(cur0.get("index", -1)) == 0
	checks["encounter first does nothing"] = QuestRunner.on_encounter_won(gs, "e1").is_empty() and DictUtil.to_int(DictUtil.dict_of(gs.quests[_q("q_flow")])["objective"]) == 0
	checks["other location does nothing"] = not QuestRunner.on_location_reached(gs, "l2").has(_q("q_flow"))
	checks["location advances"] = QuestRunner.on_location_reached(gs, "l1").has(_q("q_flow"))
	checks["no retroactive location credit"] = not QuestRunner.on_location_reached(gs, "l1").has(_q("q_flow")) and DictUtil.to_int(DictUtil.dict_of(gs.quests[_q("q_flow")])["objective"]) == 1
	var cur1: Dictionary = QuestRunner.current_objective(gs, "q_flow")
	checks["objective 1 is the encounter"] = str(cur1.get("kind", "")) == "win_encounter" and str(cur1.get("target", "")) == _q("e1")
	checks["qualified ref works"] = QuestRunner.on_encounter_won(gs, _q("e1")).has(_q("q_flow"))
	checks["ready"] = QuestRunner.status(gs, "q_flow") == "ready" and QuestRunner.current_objective(gs, "q_flow").is_empty()
	var first: Dictionary = QuestRunner.complete(gs, "q_flow")
	checks["complete ok"] = DictUtil.to_bool(first["ok"]) and QuestRunner.status(gs, "q_flow") == "completed"
	var xp_after: Array[int] = _xp(gs)
	var second: Dictionary = QuestRunner.complete(gs, "q_flow")
	checks["second complete fails"] = not DictUtil.to_bool(second["ok"]) and _joined(second["errors"]).contains("quest is not ready") and _xp(gs) == xp_after
	checks["completed can't be abandoned"] = not QuestRunner.abandon(gs, "q_flow").is_empty()
	return _report("Flow: encounter first does nothing, location then encounter -> ready, complete grants once", checks)

static func _joined(v: Variant) -> String:
	return "; ".join(PackedStringArray(JsonUtil.strings(v)))

static func _without_saved_at(text: String) -> String:
	var keep: PackedStringArray = []
	for line: String in text.split("\n"):
		if not line.contains("\"saved_at\""):
			keep.append(line)
	return "\n".join(keep)

# ------------------------------------------------------------------------------
# Check 7: flag objectives
# ------------------------------------------------------------------------------

static func _test_flag_objectives() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	gs.set_flag("rt.a", true)
	gs.set_flag("rt.b", true)
	checks["met flags before start: start chains both"] = QuestRunner.start(gs, "q_flags").is_empty() and QuestRunner.status(gs, "q_flags") == "ready"
	var gs2: GameState = _fresh()
	QuestRunner.start(gs2, "q_flags")
	QuestRunner.set_flag(gs2, "rt.b", true)
	checks["later flag first: still at 0"] = QuestRunner.status(gs2, "q_flags") == "active" and DictUtil.to_int(QuestRunner.current_objective(gs2, "q_flags")["index"]) == 0
	gs2.set_flag("rt.a", true)
	var changed: Array[String] = QuestRunner.refresh(gs2)
	checks["two met objectives chain in one refresh"] = changed.has(_q("q_flags")) and QuestRunner.status(gs2, "q_flags") == "ready"
	var gs3: GameState = _fresh()
	QuestRunner.start(gs3, "q_strict")
	QuestRunner.set_flag(gs3, "rt.strict", 1)
	checks["flag 1 does not satisfy true"] = QuestRunner.status(gs3, "q_strict") == "active"
	QuestRunner.set_flag(gs3, "rt.strict", true)
	checks["true satisfies"] = QuestRunner.status(gs3, "q_strict") == "ready"
	checks["condition_met strict"] = not QuestRules.condition_met(gs3, {"key": "rt.missing", "value": true}) and QuestRules.condition_met(gs3, {"key": "rt.missing", "value": false})
	return _report("Flag objectives: met before start advance on start, chain in one refresh, 1 is not true", checks)

# ------------------------------------------------------------------------------
# Check 8: rewards
# ------------------------------------------------------------------------------

static func _test_rewards() -> bool:
	var checks: Dictionary = {}
	var rewards: Dictionary = _fx["t_two_rewards"]
	var item: String = str(DictUtil.dict_of(rewards["items"]).keys()[0])
	var item_n: int = DictUtil.to_int(DictUtil.dict_of(rewards["items"])[item])
	var passive: String = JsonUtil.strings(rewards["passives"])[0]
	var skill: String = JsonUtil.strings(rewards["skills"])[0]
	# Empty recipient = first active member.
	var gs: GameState = _fresh()
	checks["fixture ready"] = _ready_two(gs, "q_flow", "l1", "e1")
	var xp0: Array[int] = _xp(gs)
	var stash0: int = DictUtil.to_int(gs.party.stash.get(item, 0))
	var res: Dictionary = QuestRunner.complete(gs, "q_flow")
	var m0: UnitState = gs.party.members[0]
	var m1: UnitState = gs.party.members[1]
	var xp_ok: bool = true
	for i: int in range(gs.party.members.size()):
		xp_ok = xp_ok and gs.party.members[i].total_xp == xp0[i] + DictUtil.to_int(rewards["xp"])
	checks["xp to every active member"] = DictUtil.to_bool(res["ok"]) and xp_ok
	checks["stash rises"] = DictUtil.to_int(gs.party.stash.get(item, 0)) == stash0 + item_n
	checks["passive on the first active member only"] = m0.passives.has(passive) and not m1.passives.has(passive)
	checks["skill on the first active member only"] = m0.learned_skills.has(skill) and not m1.learned_skills.has(skill)
	var granted: Dictionary = DictUtil.dict_of(res["granted"])
	checks["granted echo"] = DictUtil.to_int(granted["xp"]) == DictUtil.to_int(rewards["xp"]) and str(granted["recipient"]) == m0.id and JsonUtil.strings(granted["passives"]) == [passive]
	checks["state still valid (saveable)"] = gs.validate().is_empty()
	# Named recipient.
	var gs2: GameState = _fresh()
	_ready_two(gs2, "q_flow", "l1", "e1")
	var named: Dictionary = QuestRunner.complete(gs2, "q_flow", gs2.party.members[1].id)
	checks["named recipient"] = DictUtil.to_bool(named["ok"]) and gs2.party.members[1].passives.has(passive) and not gs2.party.members[0].passives.has(passive)
	# Unknown recipient: error, nothing changed.
	var gs3: GameState = _fresh()
	_ready_two(gs3, "q_flow", "l1", "e1")
	var before: String = _snapshot(gs3)
	var bad: Dictionary = QuestRunner.complete(gs3, "q_flow", "nobody_here")
	checks["unknown recipient refused, nothing changed"] = not DictUtil.to_bool(bad["ok"]) and _snapshot(gs3) == before and QuestRunner.status(gs3, "q_flow") == "ready"
	# Already owned = note, no duplicate.
	var gs4: GameState = _fresh()
	_ready_two(gs4, "q_flow", "l1", "e1")
	gs4.party.members[0].passives.append(passive)
	gs4.party.members[0].learned_skills.append(skill)
	var owned: Dictionary = QuestRunner.complete(gs4, "q_flow")
	checks["already owned = notes"] = DictUtil.to_bool(owned["ok"]) and DictUtil.array_of(owned["notes"]).size() == 2 and gs4.party.members[0].passives.count(passive) == 1 and gs4.party.members[0].learned_skills.count(skill) == 1
	# RewardGrant on its own validates first.
	var gs5: GameState = _fresh()
	var snap5: String = _snapshot(gs5)
	checks["RewardGrant refuses bad rewards unchanged"] = not DictUtil.to_bool(RewardGrant.apply(gs5, {"xp": 5, "items": {"no_such_item_qa": 1}})["ok"]) and _snapshot(gs5) == snap5
	return _report("Rewards: xp to every active member, stash, passive/skill to one recipient, unknown recipient changes nothing, owned = note", checks)

# ------------------------------------------------------------------------------
# Check 9: complete_flags, abandon, auto_start
# ------------------------------------------------------------------------------

static func _test_complete_flags_and_auto() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	_ready_two(gs, "q_flow", "l1", "e1")
	checks["gate locked before"] = QuestRunner.status(gs, "q_gate") == "locked" and QuestRunner.status(gs, "q_req") == "locked"
	QuestRunner.complete(gs, "q_flow")
	checks["flags set in order (later wins)"] = gs.get_flag("rt.flow_done") == true and gs.get_flag("rt.order") is int and gs.get_flag("rt.order") == 2
	checks["complete_flags make another quest available in the same call"] = QuestRunner.status(gs, "q_gate") == "available"
	checks["requires completed makes available"] = QuestRunner.status(gs, "q_req") == "available"
	var gs2: GameState = _fresh()
	QuestRunner.start(gs2, "q_flow")
	QuestRunner.on_location_reached(gs2, "l1")
	checks["abandon"] = QuestRunner.abandon(gs2, "q_flow").is_empty() and not gs2.quests.has(_q("q_flow")) and QuestRunner.status(gs2, "q_flow") == "available"
	checks["restart at objective 0"] = QuestRunner.start(gs2, "q_flow").is_empty() and DictUtil.to_int(QuestRunner.current_objective(gs2, "q_flow")["index"]) == 0
	checks["abandon not started refused"] = not QuestRunner.abandon(gs2, "q_strict").is_empty()
	var gs3: GameState = _fresh()
	checks["auto_start not before refresh"] = QuestRunner.status(gs3, "q_auto") == "available"
	var changed: Array[String] = QuestRunner.refresh(gs3)
	checks["auto_start available is active after refresh"] = changed.has(_q("q_auto")) and QuestRunner.status(gs3, "q_auto") == "active"
	checks["auto_start locked stays locked"] = QuestRunner.status(gs3, "q_auto_locked") == "locked"
	QuestRunner.set_flag(gs3, "rt.nope", true)
	checks["auto_start unlocked starts on set_flag"] = QuestRunner.status(gs3, "q_auto_locked") == "active"
	checks["refresh idempotent"] = QuestRunner.refresh(gs3).is_empty()
	return _report("complete_flags in order unlock quests in the same call; abandon restarts at 0; auto_start on refresh", checks)

# ------------------------------------------------------------------------------
# Check 10: variants resolve
# ------------------------------------------------------------------------------

static func _test_variants_resolve() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	var base: Dictionary = QuestRunner.resolved(gs, "q_var")
	checks["no flags = base"] = str(base["giver"]) == "c1" and str(base["description"]) == "base" and JsonUtil.strings(base["applied_variants"]).is_empty() and DictUtil.to_bool(base["available"])
	checks["turn_in falls back to giver"] = str(base["turn_in"]) == "c1"
	checks["template attached"] = DictUtil.array_of(base["objectives"]).size() == 2 and JsonUtil.deep_equal(base["rewards"], _fx["t_two_rewards"])
	gs.set_flag("rt.va", true)
	var one: Dictionary = QuestRunner.resolved(gs, "q_var")
	checks["one variant changes giver"] = str(one["giver"]) == "c2" and str(one["turn_in"]) == "c2"
	checks["unless not held: applies"] = str(one["description"]) == "C"
	checks["applied order"] = JsonUtil.strings(one["applied_variants"]) == ["va", "vc"]
	gs.set_flag("rt.vb", true)
	var two: Dictionary = QuestRunner.resolved(gs, "q_var")
	checks["same field: later wins"] = str(two["giver"]) == "c3"
	checks["different fields: both apply"] = str(two["reward"]) == "B" and str(two["description"]) == "C"
	checks["applied all in order"] = JsonUtil.strings(two["applied_variants"]) == ["va", "vb", "vc"]
	gs.set_flag("rt.mute", true)
	var muted: Dictionary = QuestRunner.resolved(gs, "q_var")
	checks["unless suppresses"] = str(muted["description"]) == "base" and JsonUtil.strings(muted["applied_variants"]) == ["va", "vb"]
	checks["deterministic"] = JsonUtil.deep_equal(QuestRunner.resolved(gs, "q_var"), muted)
	checks["entry untouched"] = str(WorldCatalog.get_own(RUN, "quests", "q_var")["giver"]) == "c1"
	return _report("Variants: base without flags, ordered merge (later wins per field), unless suppresses, applied ids in order, deterministic", checks)

# ------------------------------------------------------------------------------
# Check 11: rebinding
# ------------------------------------------------------------------------------

static func _test_rebinding() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	QuestRunner.start(gs, "q_rebind")
	QuestRunner.on_location_reached(gs, "l1")
	QuestRunner.set_flag(gs, "rt.moved", true)
	checks["completed objective not undone"] = DictUtil.to_int(QuestRunner.current_objective(gs, "q_rebind")["index"]) == 1
	checks["current target rebound"] = str(QuestRunner.current_objective(gs, "q_rebind")["target"]) == _q("e2")
	checks["old target ignored"] = not QuestRunner.on_encounter_won(gs, "e1").has(_q("q_rebind"))
	checks["new target advances"] = QuestRunner.on_encounter_won(gs, "e2").has(_q("q_rebind")) and QuestRunner.status(gs, "q_rebind") == "ready"
	var gs2: GameState = _fresh()
	QuestRunner.start(gs2, "q_rebind")
	QuestRunner.set_flag(gs2, "rt.moved", true)
	checks["rebound first objective: old location ignored"] = not QuestRunner.on_location_reached(gs2, "l1").has(_q("q_rebind"))
	checks["rebound first objective: new location advances"] = QuestRunner.on_location_reached(gs2, "l2").has(_q("q_rebind"))
	return _report("Rebinding: the current objective follows the flag, the old target is ignored, completed objectives stay", checks)

# ------------------------------------------------------------------------------
# Check 12: available false
# ------------------------------------------------------------------------------

static func _test_available_false() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	gs.set_flag("rt.gone", true)
	checks["not started -> unavailable"] = QuestRunner.status(gs, "q_block") == "unavailable"
	checks["unavailable can't start"] = not QuestRunner.start(gs, "q_block").is_empty() and not gs.quests.has(_q("q_block"))
	gs.clear_flag("rt.gone")
	QuestRunner.start(gs, "q_block")
	QuestRunner.on_location_reached(gs, "l1")
	var blob: String = JSON.stringify(gs.quests)
	QuestRunner.set_flag(gs, "rt.gone", true)
	checks["active -> blocked"] = QuestRunner.status(gs, "q_block") == "blocked"
	checks["events ignored while blocked"] = not QuestRunner.on_encounter_won(gs, "e1").has(_q("q_block")) and QuestRunner.current_objective(gs, "q_block").is_empty()
	checks["blob unchanged while blocked"] = JSON.stringify(gs.quests) == blob
	QuestRunner.set_flag(gs, "rt.gone", false)
	checks["cleared -> active, same index"] = QuestRunner.status(gs, "q_block") == "active" and DictUtil.to_int(QuestRunner.current_objective(gs, "q_block")["index"]) == 1
	checks["blob byte-equal after the flips"] = JSON.stringify(gs.quests) == blob
	QuestRunner.on_encounter_won(gs, "e1")
	QuestRunner.set_flag(gs, "rt.gone", true)
	var xp: Array[int] = _xp(gs)
	checks["blocked ready: complete refuses"] = QuestRunner.status(gs, "q_block") == "blocked" and not DictUtil.to_bool(QuestRunner.complete(gs, "q_block")["ok"]) and _xp(gs) == xp
	checks["blocked: abandon works"] = QuestRunner.abandon(gs, "q_block").is_empty() and not gs.quests.has(_q("q_block"))
	checks["quests_with blocked/unavailable"] = QuestRunner.quests_with(gs, "unavailable").has(_q("q_block"))
	return _report("available false: unavailable (no start), blocked (events ignored, complete refuses, abandon works), blob stable", checks)

# ------------------------------------------------------------------------------
# Check 12b (QUEST2): ready flag quest.<qid dotted>.ready
# ------------------------------------------------------------------------------

static func _test_ready_flag() -> bool:
	var checks: Dictionary = {}
	var key: String = QuestRunner.ready_flag(_q("q_flow"))
	checks["key format: quest. + dotted qid + .ready"] = key == "quest.%s.ready" % MapRules.dotted(_q("q_flow")) and key == "quest.%s.q_flow.ready" % RUN and GameState.flag_key_ok(key)
	var gs: GameState = _fresh()
	QuestRunner.start(gs, "q_flow")
	checks["not set while active"] = not gs.has_flag(key)
	QuestRunner.on_location_reached(gs, "l1")
	checks["not set after a partial objective"] = not gs.has_flag(key)
	QuestRunner.on_encounter_won(gs, "e1")
	checks["set on ready"] = QuestRunner.status(gs, "q_flow") == "ready" and gs.get_flag(key) == true
	checks["kept after complete"] = DictUtil.to_bool(QuestRunner.complete(gs, "q_flow")["ok"]) and gs.get_flag(key) == true
	var gs2: GameState = _fresh()
	checks["flag objective ready sets it"] = _ready_flag_quest(gs2, "q_strict", "rt.strict") and gs2.get_flag(QuestRunner.ready_flag(_q("q_strict"))) == true
	checks["abandon from ready clears it"] = QuestRunner.abandon(gs2, "q_strict").is_empty() and not gs2.has_flag(QuestRunner.ready_flag(_q("q_strict")))
	checks["restart does not set it back"] = QuestRunner.start(gs2, "q_flow").is_empty() and not gs2.has_flag(QuestRunner.ready_flag(_q("q_flow")))
	var gs3: GameState = _fresh()
	var bkey: String = QuestRunner.ready_flag(_q("q_block"))
	_ready_two(gs3, "q_block", "l1", "e1")
	QuestRunner.set_flag(gs3, "rt.gone", true)
	checks["blocked (variant) keeps it"] = QuestRunner.status(gs3, "q_block") == "blocked" and gs3.get_flag(bkey) == true
	QuestRunner.set_flag(gs3, "rt.gone", false)
	checks["unblocked keeps it"] = QuestRunner.status(gs3, "q_block") == "ready" and gs3.get_flag(bkey) == true
	QuestRunner.set_flag(gs3, "rt.gone", true)
	checks["abandon from blocked clears it"] = QuestRunner.abandon(gs3, "q_block").is_empty() and not gs3.has_flag(bkey)
	var gs4: GameState = _fresh()
	QuestRunner.start(gs4, "q_block")
	QuestRunner.on_location_reached(gs4, "l1")
	QuestRunner.set_flag(gs4, "rt.gone", true)
	QuestRunner.set_flag(gs4, "rt.gone", false)
	checks["blocked/unblocked while active: not set"] = QuestRunner.status(gs4, "q_block") == "active" and not gs4.has_flag(bkey)
	return _report("Ready flag: set on ready, kept through complete and blocked flips, cleared by abandon, key = quest.<dotted qid>.ready", checks)

# ------------------------------------------------------------------------------
# Check 13: rewards_from
# ------------------------------------------------------------------------------

static func _test_rewards_from() -> bool:
	var checks: Dictionary = {}
	var alt: Dictionary = _fx["t_alt_rewards"]
	var item: String = str(DictUtil.dict_of(alt["items"]).keys()[0])
	var gs: GameState = _fresh()
	gs.set_flag("rt.alt", true)
	checks["ready"] = _ready_two(gs, "q_rf", "l3", "e2")
	var xp0: Array[int] = _xp(gs)
	var stash0: int = DictUtil.to_int(gs.party.stash.get(item, 0))
	var res: Dictionary = QuestRunner.complete(gs, "q_rf")
	var granted: Dictionary = DictUtil.dict_of(res["granted"])
	checks["other template's xp"] = DictUtil.to_bool(res["ok"]) and gs.party.members[0].total_xp == xp0[0] + DictUtil.to_int(alt["xp"])
	checks["other template's items"] = DictUtil.to_int(gs.party.stash.get(item, 0)) == stash0 + DictUtil.to_int(DictUtil.dict_of(alt["items"])[item])
	checks["base specials not granted"] = JsonUtil.strings(granted["passives"]).is_empty() and JsonUtil.strings(granted["skills"]).is_empty()
	return _report("rewards_from: complete under the variant grants the other template's rewards", checks)

# ------------------------------------------------------------------------------
# Check 14: inheritance keys
# ------------------------------------------------------------------------------

static func _test_inheritance() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh(KID)
	checks["kid state"] = gs != null and gs.world_id == KID
	checks["inherited quest keyed by the parent qid"] = QuestRunner.start(gs, "q_flow").is_empty() and gs.quests.has(_q("q_flow"))
	gs.set_flag("rt.key", true)
	checks["override keyed by the child qid"] = QuestRunner.start(gs, "q_lock").is_empty() and gs.quests.has("%s:q_lock" % KID) and not gs.quests.has(_q("q_lock"))
	checks["parent qid resolves to the override"] = QuestRunner.status(gs, _q("q_lock")) == "active"
	checks["inherited events"] = QuestRunner.on_location_reached(gs, "l1").has(_q("q_flow"))
	return _report("Inheritance: inherited quest keeps the parent qid, a child override gets the child qid", checks)

# ------------------------------------------------------------------------------
# QUESTX: quests keep tracking in a connected world
# ------------------------------------------------------------------------------

static func _test_cross_world() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh(HOME)
	var cq: String = "%s:x_cross" % HOME
	checks["accepted at home"] = gs != null and QuestRunner.start(gs, "x_cross").is_empty() and gs.quests.has(cq)
	QuestRunner.on_location_reached(gs, "h1")
	checks["first step done at home"] = DictUtil.to_int(DictUtil.dict_of(gs.quests.get(cq, {})).get("objective", -1)) == 1
	gs.world_id = FAR   # the party crosses into the connected world
	checks["still active"] = QuestRunner.status(gs, cq) == "active" and QuestRunner.quests_with(gs, "active").has(cq)
	checks["objective target is world-qualified"] = str(QuestRunner.current_objective(gs, cq).get("target", "")) == "%s:f1" % FAR
	checks["journal data (group list + name) still shows it"] = QuestRunner.quests_with(gs, "active").has(cq) and str(QuestRunner.resolved(gs, cq).get("name", "")) == "Cross"
	checks["same-named far quest is a different quest"] = QuestRunner.status(gs, "%s:x_cross" % FAR) == "available" and cq != "%s:x_cross" % FAR
	var advanced: Array[String] = QuestRunner.on_location_reached(gs, "f1")
	checks["step counted in the connected world"] = advanced.has(cq) and not advanced.has("%s:x_cross" % FAR)
	checks["quest ready"] = QuestRunner.status(gs, cq) == "ready" and QuestRunner.quests_with(gs, "ready").has(cq)
	checks["completes from the connected world"] = bool(QuestRunner.complete(gs, cq)["ok"]) and QuestRunner.status(gs, cq) == "completed"
	# Reverse direction: accepted in FAR (which lists no connections), tracked while the party is in HOME (which connects to FAR).
	var back: GameState = _fresh(FAR)
	var fq: String = "%s:x_cross" % FAR
	checks["far quest accepted"] = QuestRunner.start(back, "x_cross").is_empty()
	back.world_id = HOME
	checks["far quest visible from home (connection followed both ways)"] = QuestRunner.status(back, fq) == "active"
	# Inherited quest + child override of its target location: the event fires in the child and the quest progresses.
	var kid: GameState = _fresh(XKID)
	checks["override made"] = kid != null and WorldCatalog.override_entry(XKID, "locations", "%s:h1" % HOME) == "h1"
	checks["inherited quest accepted"] = QuestRunner.start(kid, "%s:x_cross" % HOME).is_empty()
	QuestRunner.on_location_reached(kid, "h1")
	checks["child-overridden location advances the quest"] = DictUtil.to_int(DictUtil.dict_of(kid.quests.get(cq, {})).get("objective", -1)) == 1
	# Negative: a world that is neither extended nor connected stays invisible.
	var lone: GameState = _fresh(ISO)
	checks["iso state"] = lone != null and QuestRunner.status(lone, "%s:x_cross" % FAR) == "invalid"
	var stray: GameState = _fresh(HOME)
	stray.quests[fq] = {"state": "active", "objective": 0}
	stray.world_id = ISO
	checks["unlinked world: invisible"] = QuestRunner.status(stray, fq) == "invalid" and QuestRunner.quests_with(stray, "active").is_empty()
	checks["unlinked world: events don't progress it"] = QuestRunner.on_location_reached(stray, "%s:f1" % FAR).is_empty() and DictUtil.to_int(DictUtil.dict_of(stray.quests[fq]).get("objective", -1)) == 0
	return _report("QUESTX: active quests of connected worlds keep tracking; unlinked worlds stay invisible", checks)

# ------------------------------------------------------------------------------
# Check 15: save
# ------------------------------------------------------------------------------

static func _test_save() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	QuestRunner.start(gs, "q_strict")
	gs.set_flag("rt.a", true)
	gs.set_flag("rt.b", true)
	QuestRunner.start(gs, "q_flags")
	_ready_two(gs, "q_flow", "l1", "e1")
	QuestRunner.complete(gs, "q_flow")
	gs.loot_claimed = [_q("lt_skill"), _q("lt_item")] as Array[String]
	var states: Array[String] = []
	for key: Variant in gs.quests:
		states.append(str(DictUtil.dict_of(gs.quests[key])["state"]))
	checks["every stored state present"] = states.has("active") and states.has("ready") and states.has("completed")
	checks["write"] = SaveGame.write(gs, 1).is_empty()
	var back: Dictionary = SaveGame.read(1)
	var state: GameState = GameState.of(back["state"])
	checks["round trip equal"] = DictUtil.to_bool(back["ok"]) and state != null and state.equals(gs)
	checks["statuses round trip"] = state != null and QuestRunner.status(state, "q_strict") == "active" and QuestRunner.status(state, "q_flags") == "ready" and QuestRunner.status(state, "q_flow") == "completed"
	var env: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(SaveGame.path_for(1))))
	checks["current version on disk (>= 2)"] = DictUtil.to_int(env["version"]) == SaveGame.VERSION and SaveGame.VERSION >= 2
	checks["loot_claimed saved sorted"] = JsonUtil.strings(DictUtil.dict_of(env["game_state"])["loot_claimed"]) == [_q("lt_item"), _q("lt_skill")]
	# Bad blobs fail validate and write.
	var done: GameState = GameState.from_dict(gs.to_dict())
	done.quests[_q("q_x")] = {"state": "done", "objective": 0}
	var extra: GameState = GameState.from_dict(gs.to_dict())
	extra.quests[_q("q_x")] = {"state": "active", "objective": 0, "note": "x"}
	var bad_key: GameState = GameState.from_dict(gs.to_dict())
	bad_key.quests["no_colon"] = {"state": "active", "objective": 0}
	checks["state done fails"] = not done.validate().is_empty() and not SaveGame.write(done, 2).is_empty()
	checks["extra key fails"] = not extra.validate().is_empty() and not SaveGame.write(extra, 2).is_empty()
	checks["unqualified key fails"] = not bad_key.validate().is_empty()
	var dup: GameState = GameState.from_dict(gs.to_dict())
	dup.loot_claimed.append(_q("lt_item"))
	var malformed: GameState = GameState.from_dict(gs.to_dict())
	malformed.loot_claimed.append("not qualified")
	checks["duplicate loot_claimed fails"] = not dup.validate().is_empty()
	checks["malformed loot_claimed fails"] = not malformed.validate().is_empty()
	checks["nothing written for refusals"] = not SaveGame.exists(2)
	# v1 envelope: one good and one bad entry; JSON floats normalised before validation.
	var v1_state: Dictionary = gs.to_dict()
	v1_state.erase("loot_claimed")
	v1_state["quests"] = {_q("q_strict"): {"state": "active", "objective": 0}, _q("q_bad"): {"state": "done", "objective": 0}}
	var v1_text: String = JSON.stringify({"format": SaveGame.FORMAT, "version": 1, "saved_at": 0, "game_state": v1_state})
	var f := FileAccess.open(SaveGame.path_for(3), FileAccess.WRITE)
	f.store_string(v1_text)
	f.close()
	var migrated: Dictionary = SaveGame.read(3)
	var m_state: GameState = GameState.of(migrated["state"])
	var warnings: Array[String] = JsonUtil.strings(migrated["warnings"])
	checks["v1 migrates"] = DictUtil.to_bool(migrated["ok"]) and m_state != null
	checks["good entry kept (float objective normalised)"] = m_state != null and m_state.quests.has(_q("q_strict")) and DictUtil.dict_of(m_state.quests[_q("q_strict")])["objective"] is int
	checks["bad entry dropped with one warning"] = m_state != null and not m_state.quests.has(_q("q_bad")) and warnings.count("dropped invalid quest entry '%s'" % _q("q_bad")) == 1 and warnings.size() == 1
	checks["v1 gains loot_claimed []"] = m_state != null and m_state.loot_claimed.is_empty()
	var no_quests: Dictionary = SaveMigrations.migrate({"version": 1, "game_state": {"world_id": RUN}})
	var nq_state: Dictionary = DictUtil.dict_of(DictUtil.dict_of(no_quests["data"])["game_state"])
	checks["v1 without quests -> quests {} and loot_claimed []"] = DictUtil.to_bool(no_quests["ok"]) and DictUtil.dict_of(nq_state["quests"]).is_empty() and DictUtil.array_of(nq_state["loot_claimed"]).is_empty()
	SaveMigrations.register(1, func(s: Dictionary) -> Dictionary: return s)
	SaveMigrations.reset()
	var after_reset: Dictionary = DictUtil.dict_of(DictUtil.dict_of(SaveMigrations.migrate({"version": 1, "game_state": {}})["data"])["game_state"])
	checks["reset keeps the built-in step"] = after_reset.has("loot_claimed")
	# check() warns and changes nothing.
	var odd: GameState = GameState.from_dict(gs.to_dict())
	odd.quests[_q("q_unknown")] = {"state": "active", "objective": 0}
	odd.quests[_q("q_strict")] = {"state": "active", "objective": QuestRules.MAX_OBJECTIVES}
	odd.loot_claimed.append(_q("lt_unknown"))
	var snap: String = _snapshot(odd)
	var found: Array[String] = QuestRunner.check(odd)
	checks["check warns (unknown qid, index, loot)"] = found.size() == 3 and _snapshot(odd) == snap
	checks["check clean state"] = QuestRunner.check(gs).is_empty()
	# Re-saving a loaded save is byte-stable (saved_at aside).
	var loaded: GameState = GameState.of(SaveGame.read(1)["state"])
	checks["re-save byte-stable"] = loaded != null and SaveGame.write(loaded, 0).is_empty() and _without_saved_at(FileAccess.get_file_as_string(SaveGame.path_for(0))) == _without_saved_at(FileAccess.get_file_as_string(SaveGame.path_for(1)))
	# Chain: built-in 1 -> 2, then a custom 2 -> 3.
	SaveMigrations.register(2, func(s: Dictionary) -> Dictionary:
		var o: Dictionary = s.duplicate(true)
		o["location_id"] = "v3"
		return o
	)
	var chained: Dictionary = SaveMigrations.migrate({"version": 1, "game_state": {"world_id": RUN, "quests": {_q("q_strict"): {"state": "active", "objective": 0.0}, "bad": 1}}}, 3)
	var c_state: Dictionary = DictUtil.dict_of(DictUtil.dict_of(chained["data"])["game_state"])
	checks["chain built-in then custom"] = DictUtil.to_bool(chained["ok"]) and str(c_state["location_id"]) == "v3" and c_state.has("loot_claimed") and DictUtil.dict_of(c_state["quests"]).keys() == [_q("q_strict")] and JsonUtil.strings(chained["warnings"]) == ["dropped invalid quest entry 'bad'"]
	SaveMigrations.reset()
	# A v1 quests blob that is not an object warns.
	var not_obj: Dictionary = SaveMigrations.migrate({"version": 1, "game_state": {"world_id": RUN, "quests": [1, 2]}})
	checks["v1 quests not an object warns"] = DictUtil.to_bool(not_obj["ok"]) and _joined(not_obj["warnings"]).contains("quests blob")
	# from_dict never coerces damaged blobs silently: validate (and so SaveGame.read / write) refuses them.
	var good_dict: Dictionary = gs.to_dict()
	var damaged: Dictionary = {"loot_claimed string": "x", "loot_claimed number element": [5], "quests list": null}
	for label: Variant in damaged:
		var d: Dictionary = good_dict.duplicate(true)
		if str(label) == "quests list":
			d["quests"] = [_q("q_strict")]
		else:
			d["loot_claimed"] = damaged[label]
		checks["from_dict refuses " + str(label)] = not GameState.from_dict(d).validate().is_empty()
	var bad_env: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(SaveGame.path_for(1))))
	DictUtil.dict_of(bad_env["game_state"])["loot_claimed"] = "everything"
	var f6 := FileAccess.open(SaveGame.path_for(6), FileAccess.WRITE)
	f6.store_string(JSON.stringify(bad_env))
	f6.close()
	checks["damaged loot_claimed: load refused"] = not DictUtil.to_bool(SaveGame.read(6)["ok"])
	return _report("Save v2: statuses and loot_claimed round trip, bad blobs refused, v1 migration keeps good entries with warnings, check() warns only", checks)

# ------------------------------------------------------------------------------
# Check 17: loot runtime
# ------------------------------------------------------------------------------

static func _test_loot() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _fresh()
	checks["fixtures ready"] = _ready_flag_quest(gs, "q_loot1", "rt.l1") and _ready_flag_quest(gs, "q_loot2", "rt.l2")
	var e_item: String = _q("lt_item")
	var e_passive: String = _q("lt_passive")
	checks["both quests offer the shared entries"] = QuestRunner.offers(gs, "q_loot1") == [e_item, e_passive] and QuestRunner.offers(gs, "q_loot2") == [e_item, e_passive]
	checks["flag-gated entry not offered"] = not QuestRunner.offers(gs, "q_loot1").has(_q("lt_flagged"))
	checks["picks_allowed"] = QuestRunner.picks_allowed(gs, "q_loot1") == 2 and QuestRunner.picks_allowed(gs, "q_loot2") == 1
	checks["deterministic offers"] = QuestRunner.offers(gs, "q_loot1") == QuestRunner.offers(gs, "q_loot1")
	var snap: String = _snapshot(gs)
	var not_offered: Dictionary = QuestRunner.complete(gs, "q_loot1", "", [_q("lt_flagged")] as Array[String])
	var dup: Dictionary = QuestRunner.complete(gs, "q_loot1", "", [e_item, e_item] as Array[String])
	var too_many: Dictionary = QuestRunner.complete(gs, "q_loot2", "", [e_item, e_passive] as Array[String])
	checks["pick not offered = error"] = not DictUtil.to_bool(not_offered["ok"])
	checks["duplicate pick = error"] = not DictUtil.to_bool(dup["ok"])
	checks["more than allowed = error"] = not DictUtil.to_bool(too_many["ok"])
	checks["errors change nothing"] = _snapshot(gs) == snap
	var passive_ref: String = str(WorldCatalog.resolve(RUN, "loot", "lt_passive")["ref"])
	var xp0: Array[int] = _xp(gs)
	var res: Dictionary = QuestRunner.complete(gs, "q_loot1", "", [e_passive] as Array[String])
	checks["pick grants generic + entry"] = DictUtil.to_bool(res["ok"]) and gs.party.members[0].total_xp == xp0[0] + DictUtil.to_int(DictUtil.dict_of(_fx["t_one_rewards"])["xp"]) and gs.party.members[0].passives.has(passive_ref)
	checks["claimed"] = gs.loot_claimed == [e_passive] and JsonUtil.strings(DictUtil.dict_of(res["granted"])["loot"]) == [e_passive]
	checks["claimed entry leaves the other pool, others stay"] = QuestRunner.offers(gs, "q_loot2") == [e_item]
	QuestRunner.set_flag(gs, "rt.found", true)
	checks["flag-gated entry appears once the flag is set"] = QuestRunner.offers(gs, "q_loot1").has(_q("lt_flagged"))
	QuestRunner.set_flag(gs, "rt.swap", true)
	checks["variant replaces the pool"] = QuestRunner.offers(gs, "q_loot2") == [_q("lt_skill")]
	var claimed_before: int = gs.loot_claimed.size()
	var xp1: Array[int] = _xp(gs)
	var zero: Dictionary = QuestRunner.complete(gs, "q_loot2", "", [] as Array[String])
	checks["0 picks: generic only, nothing claimed"] = DictUtil.to_bool(zero["ok"]) and gs.loot_claimed.size() == claimed_before and gs.party.members[0].total_xp == xp1[0] + DictUtil.to_int(DictUtil.dict_of(_fx["t_one_rewards"])["xp"])
	checks["unclaimed entries stay offered"] = QuestRunner.offers(gs, "q_loot2") == [_q("lt_skill")]
	var gs2: GameState = _fresh()
	_ready_flag_quest(gs2, "q_noloot", "rt.nl")
	var xp2: Array[int] = _xp(gs2)
	checks["empty pool: no offers, 0 picks"] = QuestRunner.offers(gs2, "q_noloot").is_empty() and QuestRunner.picks_allowed(gs2, "q_noloot") == 0
	checks["empty pool: generic rewards only"] = DictUtil.to_bool(QuestRunner.complete(gs2, "q_noloot")["ok"]) and gs2.party.members[0].total_xp == xp2[0] + DictUtil.to_int(DictUtil.dict_of(_fx["t_one_rewards"])["xp"]) and gs2.loot_claimed.is_empty()
	checks["state saves"] = gs.validate().is_empty() and SaveGame.write(gs, 0).is_empty()
	return _report("Loot: shared offers, claim removes from every pool, 0 picks = generic only, bad picks change nothing, flags gate and swap pools", checks)
