class_name QuestTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Quest Test Suite (S8, spec M4-L)
# Slice 1 (schema): quest template validation and soft caps (QuestCatalog), world quest fields
# (template binding, flags, auto_start, requires cycles, turn_in), links, variants, the world loot
# kind and loot pools, reward budget warnings, reference scans and renames through the new json fields.
# Fixtures: QuestCatalog on a sandbox file and two hand-built worlds in a sandbox world root; the
# shipped catalog is only read (ids picked by rule), shipped worlds and templates are never written.
# ==============================================================================

const TAG: String = "Quest"
# Holds the quest file and an EMPTY world root (worlds/): the fixture worlds are built from scratch, not forked.
static var SANDBOX: String = TestHelpers.sandbox("quest")
const PARENT: String = "qa_base"
const CHILD: String = "qa_child"

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

# The `key` of every reference match held by entry `id`.
static func _keys_from(matches: Array[Dictionary], id: String) -> Array[String]:
	var out: Array[String] = []
	for m: Dictionary in matches:
		if str(m["id"]) == id:
			out.append(str(m["key"]))
	return out

static func _has(errors: Array[String], needle: String) -> bool:
	for e: String in errors:
		if e.contains(needle):
			return true
	return false

# ------------------------------------------------------------------------------

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING QUEST TEST (M4-L schema)                ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	var ok: bool = true
	ok = _test_shipped() and ok
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	QuestCatalog.use_path(SANDBOX + "/quests.json")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	var sandboxed: bool = QuestCatalog.file_path().begins_with("user://") and WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty()
	ok = _report("Sandbox (empty quest file and world root under user://)", {"sandboxed": sandboxed}) and ok
	if sandboxed:
		var fx: Dictionary = _content_fixtures()
		ok = _report("Content fixtures picked by rule", {"enough items": DictUtil.array_of(fx["items"]).size() >= 9, "two passives": DictUtil.array_of(fx["passives"]).size() >= 2, "two skills": DictUtil.array_of(fx["skills"]).size() >= 2, "a basic family": not str(fx["basic"]).is_empty()}) and ok
		ok = _test_templates(fx) and ok
		ok = _build_worlds(fx) and ok
		ok = _test_world_quests() and ok
		ok = _test_links() and ok
		ok = _test_variants() and ok
		ok = _test_loot(fx) and ok
		ok = _test_ref_scans() and ok
	QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox (quest file included)
	return ok

static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["shipped templates load"] = QuestCatalog.load_errors().is_empty() and not QuestCatalog.save_blocked()
	checks["shipped templates validate"] = QuestCatalog.validate_all().is_empty()
	checks["shipped worlds validate with the new fields"] = WorldCatalog.validate_all().is_empty()
	checks["loot is a world kind"] = WorldSchema.KINDS.has("loot") and not WorldSchema.fields("loot").is_empty()
	var d: Dictionary = WorldSchema.defaults("quests")
	checks["quest defaults empty"] = str(d["template"]).is_empty() and DictUtil.array_of(d["variants"]).is_empty() and DictUtil.dict_of(d["links"]).is_empty() and DictUtil.array_of(d["loot_pool"]).is_empty() and not DictUtil.to_bool(d["auto_start"])
	return _report("Shipped data validates (templates, worlds with the new quest fields; loot kind exists)", checks)

# Read-only picks from the shipped catalog: item ids, passive ids, non-basic family heads, one basic head.
static func _content_fixtures() -> Dictionary:
	var items: Array[String] = ContentCatalog.ids("items")
	var passives: Array[String] = ContentCatalog.ids("passives")
	var skills: Array[String] = []
	var basic: String = ""
	for ranked: Dictionary in ContentCatalog.skill_entries():
		var head: String = str(ranked.get("id", ""))
		if ContentCatalog.skill_entry(head).is_empty() or DictUtil.to_bool(ranked.get("system", false)):
			continue
		if SkillCatalog.is_basic_family(head):
			if basic.is_empty():
				basic = head
		elif skills.size() < 2:
			skills.append(head)
	return {"items": items, "passives": passives, "skills": skills, "basic": basic}

static func _template(id: String, kinds: Array[String], rewards: Dictionary = {}) -> Dictionary:
	var objectives: Array = []
	for k: String in kinds:
		objectives.append({"kind": k})
	return {"id": id, "name": "Template %s" % id, "objectives": objectives, "rewards": rewards}

# ------------------------------------------------------------------------------
# Check 1: templates
# ------------------------------------------------------------------------------

static func _test_templates(fx: Dictionary) -> bool:
	var items: Array[String] = JsonUtil.strings(fx["items"])
	var passives: Array[String] = JsonUtil.strings(fx["passives"])
	var skills: Array[String] = JsonUtil.strings(fx["skills"])
	var valid: Dictionary = _template("t_valid", ["reach_location", "win_encounter"], {"xp": QuestRules.SOFT_MAX_XP, "items": {items[0]: 2}, "passives": [passives[0]], "skills": [skills[0]]})
	var checks: Dictionary = {}
	checks["valid has no errors"] = QuestCatalog.validate_entry(valid).is_empty()
	checks["valid has no warnings"] = QuestCatalog.warnings_for(valid).is_empty()
	var nine: Array[String] = []
	for i: int in range(QuestRules.MAX_OBJECTIVES + 1):
		nine.append("flag")
	var many_items: Dictionary = {}
	for i: int in range(mini(items.size(), QuestRules.MAX_ITEM_KEYS + 1)):
		many_items[items[i]] = 1
	var bad: Dictionary = {
		"empty objectives": _template("t_bad", []),
		"9 objectives": _template("t_bad", nine),
		"unknown kind": _template("t_bad", ["have_item"]),
		"extra objective key": {"id": "t_bad", "name": "Bad", "objectives": [{"kind": "flag", "flag": "x"}]},
		"unknown reward key": _template("t_bad", ["flag"], {"gold": 5}),
		"xp above max": _template("t_bad", ["flag"], {"xp": QuestRules.XP_MAX + 1}),
		"xp not whole": _template("t_bad", ["flag"], {"xp": 1.5}),
		"item count 0": _template("t_bad", ["flag"], {"items": {items[0]: 0}}),
		"item count 100": _template("t_bad", ["flag"], {"items": {items[0]: QuestRules.ITEM_COUNT_MAX + 1}}),
		"9 item keys": _template("t_bad", ["flag"], {"items": many_items}),
		"unknown item": _template("t_bad", ["flag"], {"items": {"no_such_item_qa": 1}}),
		"unknown passive": _template("t_bad", ["flag"], {"passives": ["no_such_passive_qa"]}),
		"unknown skill": _template("t_bad", ["flag"], {"skills": ["no_such_skill_qa"]}),
		"basic skill": _template("t_bad", ["flag"], {"skills": [str(fx["basic"])]}),
		"duplicate passive": _template("t_bad", ["flag"], {"passives": [passives[0], passives[0]]}),
		"empty name": {"id": "t_bad", "name": " ", "objectives": [{"kind": "flag"}]},
		"bad id": _template("Bad Id", ["flag"]),
	}
	for label: Variant in bad:
		checks[str(label)] = not QuestCatalog.validate_entry(DictUtil.dict_of(bad[label])).is_empty()
	# Soft caps warn but never block.
	var warned: Dictionary = {
		"2 passives": {"passives": [passives[0], passives[1]]},
		"2 skills": {"skills": [skills[0], skills[1]]},
		"xp above soft cap": {"xp": QuestRules.SOFT_MAX_XP + 1},
		"items above soft cap": {"items": {items[0]: QuestRules.SOFT_MAX_ITEMS + 1}},
	}
	var n: int = 0
	for label: Variant in warned:
		var t: Dictionary = _template("t_warn_%d" % n, ["flag"], DictUtil.dict_of(warned[label]))
		n += 1
		checks["warns: " + str(label)] = QuestCatalog.warnings_for(t).size() == 1 and QuestCatalog.upsert(t).is_empty()
	checks["warned templates stored"] = QuestCatalog.ids().size() == warned.size()
	checks["warnings_all lists them"] = QuestCatalog.warnings_all().size() == warned.size()
	checks["validate_all stays {}"] = QuestCatalog.validate_all().is_empty()
	# Upsert / remove / save / reload, and a duplicate id is refused.
	checks["upsert valid"] = QuestCatalog.upsert(valid).is_empty()
	checks["duplicate id refused"] = not QuestCatalog.upsert(valid, "t_warn_0").is_empty()
	checks["upsert bad refused"] = not QuestCatalog.upsert(DictUtil.dict_of(bad["unknown kind"])).is_empty() and not QuestCatalog.has("t_bad")
	checks["remove"] = QuestCatalog.remove("t_warn_0").is_empty() and not QuestCatalog.has("t_warn_0") and not QuestCatalog.remove("t_warn_0").is_empty()
	var before: Array[Dictionary] = QuestCatalog.entries()
	checks["save"] = QuestCatalog.save() == OK and not QuestCatalog.is_dirty()
	QuestCatalog.reload_from_disk()
	checks["reload equal"] = JsonUtil.deep_equal(before, QuestCatalog.entries()) and QuestCatalog.validate_all().is_empty()
	# A file that loads with errors refuses to save.
	var broken: String = SANDBOX + "/broken_quests.json"
	var f := FileAccess.open(broken, FileAccess.WRITE)
	f.store_string("{\"kind\": \"quest_templates\", \"entries\": [3]}")
	f.close()
	QuestCatalog.use_path(broken)
	checks["broken file blocks save"] = QuestCatalog.save_blocked() and not QuestCatalog.load_errors().is_empty() and QuestCatalog.save() != OK and not QuestCatalog.validate_all().is_empty()
	QuestCatalog.use_path(SANDBOX + "/quests.json")
	# Fixture templates for the world checks.
	checks["fixture templates"] = (QuestCatalog.upsert(_template("t_two", ["reach_location", "win_encounter"], {"xp": 10})).is_empty()
		and QuestCatalog.upsert(_template("t_flag", ["flag"], {"xp": 5})).is_empty()
		and QuestCatalog.upsert(_template("t_passive", ["flag"], {"passives": [passives[0]]})).is_empty()
		and QuestCatalog.upsert(_template("t_plain", ["flag"])).is_empty())
	return _report("Templates: every bad case errors, soft caps only warn, upsert/remove/save/reload, broken file blocks save", checks)

# ------------------------------------------------------------------------------
# World fixtures
# ------------------------------------------------------------------------------

static func _build_worlds(fx: Dictionary) -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.create_world(PARENT, "QA Base"))
	errors.append_array(WorldCatalog.create_world(CHILD, "QA Child", [PARENT]))
	for id: String in ["loc_a", "loc_b"]:
		errors.append_array(WorldCatalog.upsert(PARENT, "locations", {"id": id, "name": id}))
	errors.append_array(WorldCatalog.upsert(PARENT, "encounters", {"id": "enc_a", "name": "Enc A", "location": "loc_b"}))
	for id: String in ["npc_a", "npc_b"]:
		errors.append_array(WorldCatalog.upsert(PARENT, "characters", {"id": id, "name": id}))
	errors.append_array(WorldCatalog.upsert(PARENT, "factions", {"id": "fac_a", "name": "Fac A"}))
	errors.append_array(WorldCatalog.upsert(PARENT, "loot", {"id": "loot_item", "name": "Loot Item", "reward_kind": "item", "ref": JsonUtil.strings(fx["items"])[0], "count": 2}))
	errors.append_array(WorldCatalog.upsert(PARENT, "loot", {"id": "loot_passive", "name": "Loot Passive", "reward_kind": "passive", "ref": JsonUtil.strings(fx["passives"])[1]}))
	errors.append_array(WorldCatalog.upsert(PARENT, "loot", {"id": "loot_skill", "name": "Loot Skill", "reward_kind": "skill", "ref": JsonUtil.strings(fx["skills"])[0]}))
	errors.append_array(WorldCatalog.upsert(PARENT, "quests", _quest("q_base")))
	return _report("Fixture worlds (parent + child) built through upsert", {"no errors": errors.is_empty(), "lineage": WorldCatalog.lineage(CHILD) == [PARENT, CHILD]})

# A valid templated quest of PARENT on t_two; `over` replaces fields.
static func _quest(id: String, over: Dictionary = {}) -> Dictionary:
	var q: Dictionary = {
		"id": id, "name": "Quest %s" % id, "giver": "npc_a", "template": "t_two",
		"steps": [{"text": "Go to A", "location": "loc_a"}, {"text": "Win", "encounter": "enc_a"}],
	}
	q.merge(over, true)
	return q

static func _errs(entry: Dictionary, world: String = PARENT, kind: String = "quests") -> Array[String]:
	return WorldCatalog.validate_entry(world, kind, entry, str(entry.get("id", "")))

# ------------------------------------------------------------------------------
# Check 2: world quests
# ------------------------------------------------------------------------------

static func _test_world_quests() -> bool:
	var checks: Dictionary = {}
	checks["valid quest"] = _errs(_quest("q_x")).is_empty()
	checks["stored quest valid"] = WorldCatalog.validate_world(PARENT).keys().filter(func(k: Variant) -> bool: return str(k).contains("quests") or str(k).contains("loot")).is_empty()
	checks["full flag quest valid"] = _errs(_quest("q_x", {"template": "t_flag", "steps": [{"text": "Flag", "flag": "qa.door", "value": 2}], "requires_flags": [{"key": "qa.a"}, {"key": "qa.b", "value": "x"}], "complete_flags": [{"key": "qa.c", "value": false}], "auto_start": true, "turn_in": "npc_b"})).is_empty()
	checks["step count mismatch"] = _has(_errs(_quest("q_x", {"steps": [{"text": "Only one", "location": "loc_a"}]})), "steps")
	checks["win_encounter without encounter"] = _has(_errs(_quest("q_x", {"steps": [{"text": "Go", "location": "loc_a"}, {"text": "Win", "location": "loc_b"}]})), "needs \"encounter\"")
	checks["reach_location without location"] = not _errs(_quest("q_x", {"steps": [{"text": "Go"}, {"text": "Win", "encounter": "enc_a"}]})).is_empty()
	checks["unknown bound location"] = not _errs(_quest("q_x", {"steps": [{"text": "Go", "location": "nowhere"}, {"text": "Win", "encounter": "enc_a"}]})).is_empty()
	checks["unused step key ignored"] = _errs(_quest("q_x", {"steps": [{"text": "Go", "location": "loc_a", "encounter": "not_used_here"}, {"text": "Win", "encounter": "enc_a"}]})).is_empty()
	checks["flag key Bad Key (step)"] = not _errs(_quest("q_x", {"template": "t_flag", "steps": [{"text": "Flag", "flag": "Bad Key"}]})).is_empty()
	checks["flag key Bad Key (requires_flags)"] = not _errs(_quest("q_x", {"requires_flags": [{"key": "Bad Key"}]})).is_empty()
	checks["flag value float"] = not _errs(_quest("q_x", {"complete_flags": [{"key": "qa.a", "value": 1.5}]})).is_empty()
	checks["condition extra key"] = not _errs(_quest("q_x", {"requires_flags": [{"key": "qa.a", "when": 1}]})).is_empty()
	checks["requires_flags not a list (blocking)"] = not WorldCatalog.blocking_errors(PARENT, "quests", _quest("q_x", {"requires_flags": {"key": "qa.a"}})).is_empty()
	checks["auto_start without template"] = not _errs({"id": "q_x", "name": "Lore", "auto_start": true}).is_empty()
	checks["unknown template"] = not _errs(_quest("q_x", {"template": "no_such_template"})).is_empty()
	checks["turn_in unknown character"] = not _errs(_quest("q_x", {"turn_in": "nobody"})).is_empty()
	checks["lore-only old shape valid"] = _errs({"id": "q_lore", "name": "Lore", "giver": "npc_a", "steps": [{"text": "Visit", "location": "loc_a"}, {"text": "Fight", "location": "loc_b", "encounter": "enc_a"}, {"text": "Talk"}], "reward": "Thanks"}).is_empty()
	# Requires cycle: q_c1 -> q_c2 -> q_c1 (and a chain without a cycle is fine).
	WorldCatalog.upsert(PARENT, "quests", _quest("q_c1", {"requires": ["q_c2"]}))
	WorldCatalog.upsert(PARENT, "quests", _quest("q_c2", {"requires": ["q_c1"]}))
	checks["requires cycle"] = _has(_errs(WorldCatalog.get_own(PARENT, "quests", "q_c1")), "circular") and _has(_errs(WorldCatalog.get_own(PARENT, "quests", "q_c2")), "circular")
	checks["requires chain ok"] = _errs(_quest("q_x", {"requires": ["q_base"]})).is_empty()
	checks["self require"] = _has(_errs(_quest("q_base", {"requires": ["q_base"]})), "circular")
	WorldCatalog.remove(PARENT, "quests", "q_c1")
	WorldCatalog.remove(PARENT, "quests", "q_c2")
	return _report("World quests: binding per objective, flags, auto_start, template, turn_in, requires cycles; lore-only stays valid", checks)

# ------------------------------------------------------------------------------
# Check 3: links
# ------------------------------------------------------------------------------

static func _test_links() -> bool:
	var checks: Dictionary = {}
	var full: Dictionary = {"follows": ["q_base"], "related_quests": ["q_base"], "related_characters": ["npc_b"], "related_locations": ["loc_b"], "related_factions": ["fac_a"]}
	checks["valid links"] = _errs(_quest("q_x", {"links": full})).is_empty()
	checks["unknown key"] = not _errs(_quest("q_x", {"links": {"sequel": ["q_base"]}})).is_empty()
	checks["unresolved ref"] = not _errs(_quest("q_x", {"links": {"related_locations": ["nowhere"]}})).is_empty()
	checks["follows itself"] = _has(_errs(_quest("q_base", {"links": {"follows": ["q_base"]}})), "itself")
	checks["leads_to itself"] = _has(_errs(_quest("q_base", {"links": {"leads_to": ["q_base"]}})), "itself")
	checks["lore-only quest may carry links"] = _errs({"id": "q_x", "name": "Lore", "links": full}).is_empty()
	# One-sided leads_to: a warning only, both quests stay valid.
	var stored: bool = WorldCatalog.upsert(PARENT, "quests", _quest("q_next", {"links": {}})).is_empty() and WorldCatalog.upsert(PARENT, "quests", _quest("q_base", {"links": {"leads_to": ["q_next"]}})).is_empty()
	var one_sided: Array[String] = QuestRules.link_warnings(PARENT)
	checks["one-sided warns"] = stored and one_sided.size() == 1 and _errs(WorldCatalog.get_own(PARENT, "quests", "q_base")).is_empty()
	WorldCatalog.upsert(PARENT, "quests", _quest("q_next", {"links": {"follows": ["q_base"]}}))
	checks["two-sided does not warn"] = QuestRules.link_warnings(PARENT).is_empty()
	checks["child sees the warning set"] = QuestRules.link_warnings(CHILD).is_empty()
	return _report("Links: unknown key, unresolved ref, self follows/leads_to error; one-sided chain warns only", checks)

# ------------------------------------------------------------------------------
# Check 4: variants
# ------------------------------------------------------------------------------

static func _variant(id: String, set_d: Dictionary, when: Array = [{"key": "qa.dead"}]) -> Dictionary:
	return {"id": id, "when_flags": when, "set": set_d}

static func _test_variants() -> bool:
	var checks: Dictionary = {}
	var good: Array = [
		_variant("v_giver", {"giver": "npc_b", "turn_in": "", "description": "Changed", "reward": "Other"}),
		{"id": "v_unless", "unless_flags": [{"key": "qa.alive", "value": 1}], "set": {"available": false}},
		_variant("v_steps", {"steps": {"0": {"location": "loc_b", "text": "Go to B"}, "1": {"encounter": "enc_a"}}}),
		_variant("v_rewards", {"rewards_from": "t_flag", "links": {"related_factions": ["fac_a"]}}),
		_variant("v_loot", {"loot_pool": ["loot_item"], "loot_picks": 0}),
	]
	checks["valid variant set"] = _errs(_quest("q_x", {"variants": good})).is_empty()
	var bad: Dictionary = {
		"unconditional": [{"id": "v", "set": {"giver": "npc_b"}}],
		"duplicate id": [_variant("v", {"giver": "npc_b"}), _variant("v", {"reward": "x"})],
		"set template": [_variant("v", {"template": "t_flag"})],
		"set requires": [_variant("v", {"requires": []})],
		"set complete_flags": [_variant("v", {"complete_flags": []})],
		"set objectives": [_variant("v", {"objectives": []})],
		"step index out of range": [_variant("v", {"steps": {"2": {"text": "x"}}})],
		"step index not a number": [_variant("v", {"steps": {"first": {"text": "x"}}})],
		"encounter on a reach_location step": [_variant("v", {"steps": {"0": {"encounter": "enc_a"}}})],
		"emptied required binding": [_variant("v", {"steps": {"1": {"encounter": ""}}})],
		"emptied text": [_variant("v", {"steps": {"0": {"text": ""}}})],
		"unknown rewards_from": [_variant("v", {"rewards_from": "no_such_template"})],
		"unknown giver": [_variant("v", {"giver": "nobody"})],
		"available not bool": [_variant("v", {"available": "no"})],
		"empty set": [_variant("v", {})],
		"unknown variant key": [{"id": "v", "when_flags": [{"key": "qa.a"}], "set": {"reward": "x"}, "priority": 1}],
		"bad condition": [_variant("v", {"reward": "x"}, [{"key": "Bad Key"}])],
	}
	for label: Variant in bad:
		checks[str(label)] = not _errs(_quest("q_x", {"variants": bad[label]})).is_empty()
	# The message names the rule that failed (not just any error).
	var why: Dictionary = {"unconditional": "no condition", "duplicate id": "used twice", "set template": "can't be overridden", "set requires": "can't be overridden",
		"step index out of range": "out of range", "encounter on a reach_location step": "can't set 'encounter'", "emptied required binding": "can't be emptied", "unknown rewards_from": "unknown quest template"}
	for label: Variant in why:
		checks["message: " + str(label)] = _has(_errs(_quest("q_x", {"variants": bad[label]})), str(why[label]))
	var nine: Array = []
	for i: int in range(QuestRules.MAX_VARIANTS + 1):
		nine.append(_variant("v%d" % i, {"reward": "r%d" % i}))
	checks["9 variants"] = not _errs(_quest("q_x", {"variants": nine})).is_empty()
	checks["8 variants ok"] = _errs(_quest("q_x", {"variants": nine.slice(0, QuestRules.MAX_VARIANTS)})).is_empty()
	checks["variants without template"] = not _errs({"id": "q_x", "name": "Lore", "variants": [_variant("v", {"reward": "x"})]}).is_empty()
	return _report("Variants: whitelist, conditions, ids, per-step partials, rewards_from, count; a valid set has no errors", checks)

# ------------------------------------------------------------------------------
# Check 16: loot kind and pools
# ------------------------------------------------------------------------------

static func _test_loot(fx: Dictionary) -> bool:
	var checks: Dictionary = {}
	var items: Array[String] = JsonUtil.strings(fx["items"])
	var passives: Array[String] = JsonUtil.strings(fx["passives"])
	var skills: Array[String] = JsonUtil.strings(fx["skills"])
	checks["stored loot valid"] = _errs(WorldCatalog.get_own(PARENT, "loot", "loot_item"), PARENT, "loot").is_empty() and _errs(WorldCatalog.get_own(PARENT, "loot", "loot_passive"), PARENT, "loot").is_empty() and _errs(WorldCatalog.get_own(PARENT, "loot", "loot_skill"), PARENT, "loot").is_empty()
	checks["defaults"] = str(WorldSchema.defaults("loot")["rarity"]) == "rare" and str(WorldSchema.defaults("loot")["reward_kind"]) == "item" and DictUtil.to_int(WorldSchema.defaults("loot")["count"]) == 1
	var bad: Dictionary = {
		"unknown item ref": {"reward_kind": "item", "ref": "no_such_item_qa"},
		"unknown passive ref": {"reward_kind": "passive", "ref": "no_such_passive_qa"},
		"unknown skill ref": {"reward_kind": "skill", "ref": "no_such_skill_qa"},
		"basic skill": {"reward_kind": "skill", "ref": str(fx["basic"])},
		"item id as passive": {"reward_kind": "passive", "ref": items[0]},
		"count 2 on a passive": {"reward_kind": "passive", "ref": passives[0], "count": 2},
		"count 2 on a skill": {"reward_kind": "skill", "ref": skills[0], "count": 2},
		"count 0 on an item": {"reward_kind": "item", "ref": items[0], "count": 0},
		"count 100 on an item": {"reward_kind": "item", "ref": items[0], "count": QuestRules.ITEM_COUNT_MAX + 1},
		"bad rarity": {"reward_kind": "item", "ref": items[0], "rarity": "legendary"},
		"bad reward kind": {"reward_kind": "gold", "ref": items[0]},
		"empty ref": {"reward_kind": "item", "ref": ""},
		"bad requires_flags": {"reward_kind": "item", "ref": items[0], "requires_flags": [{"key": "Bad Key"}]},
	}
	for label: Variant in bad:
		var e: Dictionary = {"id": "l_x", "name": "L"}
		e.merge(DictUtil.dict_of(bad[label]), true)
		checks[str(label)] = not _errs(e, PARENT, "loot").is_empty()
	checks["good loot with flags"] = _errs({"id": "l_x", "name": "L", "reward_kind": "item", "ref": items[0], "count": 3, "rarity": "unique", "requires_flags": [{"key": "qa.found"}]}, PARENT, "loot").is_empty()
	# Inheritance and override of a parent loot entry from the child.
	var child_sees: bool = WorldCatalog.has_entry(CHILD, "loot", "loot_item") and str(WorldCatalog.resolve(CHILD, "loot", "loot_item").get("_qid", "")) == "%s:loot_item" % PARENT
	var ov_id: String = WorldCatalog.override_entry(CHILD, "loot", "%s:loot_item" % PARENT)
	var ov_ok: bool = not ov_id.is_empty() and str(WorldCatalog.resolve(CHILD, "loot", "loot_item").get("_world", "")) == CHILD and str(WorldCatalog.resolve(PARENT, "loot", "loot_item").get("_world", "")) == PARENT
	checks["child inherits"] = child_sees
	checks["child overrides"] = ov_ok and _errs(WorldCatalog.get_own(CHILD, "loot", ov_id), CHILD, "loot").is_empty()
	checks["child pool resolves the override"] = _errs(_quest("q_cx", {"loot_pool": ["loot_item"]}), CHILD).is_empty()
	# Pools on quests.
	checks["valid pool"] = _errs(_quest("q_x", {"loot_pool": ["loot_item", "loot_passive"], "loot_picks": 2})).is_empty()
	checks["picks 0 ok"] = _errs(_quest("q_x", {"loot_pool": ["loot_item"], "loot_picks": 0})).is_empty()
	checks["empty pool ignores picks"] = _errs(_quest("q_x", {"loot_pool": [], "loot_picks": QuestRules.MAX_PICKS})).is_empty()
	checks["unknown pool ref"] = not _errs(_quest("q_x", {"loot_pool": ["no_such_loot"]})).is_empty()
	checks["duplicate pool ref"] = not _errs(_quest("q_x", {"loot_pool": ["loot_item", "loot_item"]})).is_empty()
	checks["duplicate via qualified ref"] = not _errs(_quest("q_x", {"loot_pool": ["loot_item", "%s:loot_item" % PARENT]})).is_empty()
	var nine: Array = []
	for i: int in range(QuestRules.MAX_POOL + 1):
		nine.append("loot_item")
	checks["9 refs"] = _has(_errs(_quest("q_x", {"loot_pool": nine})), "at most")
	checks["picks 4"] = not _errs(_quest("q_x", {"loot_pool": ["loot_item", "loot_passive", "loot_skill"], "loot_picks": QuestRules.MAX_PICKS + 1})).is_empty()
	checks["picks above pool size"] = not _errs(_quest("q_x", {"loot_pool": ["loot_item"], "loot_picks": 2})).is_empty()
	checks["picks stored unclamped"] = DictUtil.to_int(WorldSchema.normalize("quests", {"loot_picks": 7})["loot_picks"]) == 7
	checks["pool without template"] = not _errs({"id": "q_x", "name": "Lore", "loot_pool": ["loot_item"]}).is_empty()
	# Variant loot_pool / loot_picks validate like the base field (picks are not cross-checked with the pool).
	checks["variant pool valid"] = _errs(_quest("q_x", {"variants": [_variant("v", {"loot_pool": ["loot_skill"], "loot_picks": QuestRules.MAX_PICKS})]})).is_empty()
	checks["variant pool unknown"] = not _errs(_quest("q_x", {"variants": [_variant("v", {"loot_pool": ["no_such_loot"]})]})).is_empty()
	checks["variant pool duplicate"] = not _errs(_quest("q_x", {"variants": [_variant("v", {"loot_pool": ["loot_item", "loot_item"]})]})).is_empty()
	checks["variant picks 4"] = not _errs(_quest("q_x", {"variants": [_variant("v", {"loot_picks": QuestRules.MAX_PICKS + 1})]})).is_empty()
	# Reward budget warnings (generic + one pooled special per kind; picks above the soft cap).
	var budget: Array[bool] = []
	budget.append(WorldCatalog.upsert(PARENT, "quests", _quest("q_budget", {"template": "t_passive", "steps": [{"text": "F", "flag": "qa.f"}], "loot_pool": ["loot_passive"]})).is_empty())
	checks["1 passive + pooled passive warns"] = budget[0] and _has(QuestRules.reward_warnings(PARENT), "%s:q_budget" % PARENT)
	WorldCatalog.upsert(PARENT, "quests", _quest("q_budget", {"template": "t_plain", "steps": [{"text": "F", "flag": "qa.f"}], "loot_pool": ["loot_passive"]}))
	checks["0 passives + pooled passive does not warn"] = not _has(QuestRules.reward_warnings(PARENT), "q_budget")
	WorldCatalog.upsert(PARENT, "quests", _quest("q_budget", {"template": "t_plain", "steps": [{"text": "F", "flag": "qa.f"}], "loot_pool": ["loot_passive", "loot_item"], "loot_picks": 2}))
	checks["loot_picks 2 warns"] = _has(QuestRules.reward_warnings(PARENT), "loot picks")
	WorldCatalog.upsert(PARENT, "quests", _quest("q_budget", {"template": "t_plain", "steps": [{"text": "F", "flag": "qa.f"}], "loot_pool": ["loot_item"], "variants": [_variant("v", {"rewards_from": "t_passive", "loot_pool": ["loot_passive"]})]}))
	checks["variant worst case warns"] = _has(QuestRules.reward_warnings(PARENT), "passives")
	WorldCatalog.remove(PARENT, "quests", "q_budget")
	return _report("Loot: refs per reward kind, counts, rarity, inheritance/override, pools, picks, variant pools, reward budget warnings", checks)

# ------------------------------------------------------------------------------
# Reference scans and renames through links, loot_pool and variant set
# ------------------------------------------------------------------------------

static func _test_ref_scans() -> bool:
	var checks: Dictionary = {}
	var q: Dictionary = _quest("q_refs", {
		"links": {"related_locations": ["loc_b"], "related_characters": ["npc_b"]},
		"loot_pool": ["loot_skill"],
		"variants": [_variant("v", {"giver": "npc_b", "steps": {"0": {"location": "loc_b"}}, "links": {"related_locations": ["loc_b"]}, "loot_pool": ["loot_skill"]})],
	})
	checks["stored"] = WorldCatalog.upsert(PARENT, "quests", q).is_empty()
	var loc_refs: Array[String] = _keys_from(WorldCatalog.references_to(PARENT, "locations", "loc_b"), "q_refs")
	checks["location refs: links + variant step + variant links"] = loc_refs.count("links") == 1 and loc_refs.count("variants") == 2
	var npc_refs: Array[String] = _keys_from(WorldCatalog.references_to(PARENT, "characters", "npc_b"), "q_refs")
	checks["character refs: links + variant giver"] = npc_refs.count("links") == 1 and npc_refs.count("variants") == 1
	var loot_refs: Array[String] = _keys_from(WorldCatalog.references_to(PARENT, "loot", "loot_skill"), "q_refs")
	checks["loot refs: pool + variant pool"] = loot_refs.count("loot_pool") == 1 and loot_refs.count("variants") == 1
	checks["pooled loot can't be removed"] = not WorldCatalog.remove(PARENT, "loot", "loot_skill").is_empty()
	# Renames rewrite every path.
	var loc: Dictionary = WorldCatalog.get_own(PARENT, "locations", "loc_b")
	loc["id"] = "loc_b2"
	var loot: Dictionary = WorldCatalog.get_own(PARENT, "loot", "loot_skill")
	loot["id"] = "loot_skill2"
	checks["renames stored"] = WorldCatalog.upsert(PARENT, "locations", loc, "loc_b").is_empty() and WorldCatalog.upsert(PARENT, "loot", loot, "loot_skill").is_empty()
	var after: Dictionary = WorldCatalog.get_own(PARENT, "quests", "q_refs")
	var v_set: Dictionary = DictUtil.dict_of(DictUtil.dict_of(DictUtil.array_of(after["variants"])[0])["set"])
	checks["links rewritten"] = JsonUtil.strings(DictUtil.dict_of(after["links"])["related_locations"]) == ["loc_b2"]
	checks["variant step rewritten"] = str(DictUtil.dict_of(DictUtil.dict_of(v_set["steps"])["0"])["location"]) == "loc_b2"
	checks["variant links rewritten"] = JsonUtil.strings(DictUtil.dict_of(v_set["links"])["related_locations"]) == ["loc_b2"]
	checks["pool rewritten"] = JsonUtil.strings(after["loot_pool"]) == ["loot_skill2"] and JsonUtil.strings(v_set["loot_pool"]) == ["loot_skill2"]
	checks["still valid"] = _errs(after).is_empty()
	# Save and reload keep the new fields (byte-stable second save).
	checks["save"] = WorldCatalog.save_all() == OK
	var path: String = WorldCatalog.path_for(PARENT, "quests")
	var first: String = FileAccess.get_file_as_string(path)
	WorldCatalog.reload_from_disk()
	checks["reload equal"] = JsonUtil.deep_equal(WorldSchema.normalize("quests", WorldCatalog.get_own(PARENT, "quests", "q_refs")), WorldSchema.normalize("quests", after))
	checks["loot file written"] = FileAccess.file_exists(WorldCatalog.path_for(PARENT, "loot")) and WorldCatalog.own_ids(PARENT, "loot").has("loot_skill2")
	WorldCatalog.upsert(PARENT, "quests", WorldCatalog.get_own(PARENT, "quests", "q_refs"))
	checks["second save byte-stable"] = WorldCatalog.save_all() == OK and FileAccess.get_file_as_string(path) == first
	return _report("Reference scans and renames follow links, loot_pool and variant set; save/reload keep the fields", checks)
