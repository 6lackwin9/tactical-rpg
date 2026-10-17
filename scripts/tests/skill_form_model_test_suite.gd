class_name SkillFormModelTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Form Model Test Suite (FORGE1 / SKILL1-L commit 2)
# The headless brain of the skill tab: visible fields per kind equal the SkillKinds sections, dynamic limits, kind-switch
# resets, the Save-blocking reasons, the SkillFilter predicates, the ranks view (resolved values, highlighted modifiers)
# and the live description preview. Fixtures are built here; expectations come from rules.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("skill_form_model_test")
const TAG: String = "SkillForm"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SKILL FORM MODEL TEST (SKILL1-L)        ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_visible_fields() and ok
	ok = _test_dynamic_limits() and ok
	ok = _test_kind_switch() and ok
	ok = _test_save_state() and ok
	ok = _test_ranks_view() and ok
	ok = _test_description_editor() and ok
	ok = _test_filter() and ok
	ok = _test_key_fields_and_requires() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

static func _test_visible_fields() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var all_ok: bool = true
	var gate_ok: bool = true
	for kind: String in SkillKinds.authorable():
		var entry: Dictionary = SkillFormModel.new_entry("t_form_%s" % kind.to_lower())
		entry = SkillFormModel.switch_kind(entry, kind)
		var shown: Array[String] = SkillFormModel.visible_keys(entry)
		for key: String in shown:
			# every shown field must belong to the kind (or be a core field every kind has)
			all_ok = all_ok and SkillKinds.has_field(kind, key)
		var want_gate: bool = SkillKinds.sections_of(kind).has("gate")
		gate_ok = gate_ok and shown.has("weapon_types") == want_gate
		for key: String in SkillKinds.sectioned_fields():
			if SkillKinds.has_field(kind, key) and ContentSchema.is_visible(ContentSchema.field("skills", key), entry) and not ["family", "tier", "template", "unlock_offset", "system", "target_mode"].has(key):
				all_ok = all_ok and shown.has(key)
	checks["shown fields equal the kind's sections"] = all_ok
	checks["the weapon gate is shown exactly for kinds with the gate section"] = gate_ok
	var layout: Array[Dictionary] = SkillFormModel.layout(SkillFormModel.new_entry("t_form_layout"))
	checks["the core sections come first, identity before classification"] = layout.size() >= 2 and str(layout[0]["id"]) == "identity" and str(layout[1]["id"]) == "classification"
	return TestHelpers.report(TAG, "Visible fields per kind", checks)

static func _test_dynamic_limits() -> bool:
	var checks: Dictionary = {}
	var entry: Dictionary = TestHelpers.attack("t_form_lim", {"reach_min": 1, "reach_max": 3})
	checks["reach_min is capped by reach_max"] = DictUtil.to_int(SkillFormModel.state(entry, "reach_min")["max"]) == 3
	var dash: Dictionary = TestHelpers.attack("t_form_dash", {"area": "BURST"})
	checks["no gap-close with an area attack"] = DictUtil.to_int(SkillFormModel.state(dash, "dash_tiles")["max"]) == 0
	var off: Dictionary = TestHelpers.attack("t_form_off", {"weapon_hand": "off"})
	checks["no gap-close with an off-hand strike"] = DictUtil.to_int(SkillFormModel.state(off, "dash_tiles")["max"]) == 0
	var dashing: Dictionary = TestHelpers.attack("t_form_dash2", {"dash_tiles": 2})
	var hand: Dictionary = DictUtil.dict_of(SkillFormModel.state(dashing, "weapon_hand")["disabled_options"])
	checks["off and both are disabled while the skill gap-closes"] = hand.has("off") and hand.has("both")
	var plain: Dictionary = DictUtil.dict_of(SkillFormModel.state(entry, "weapon_hand")["disabled_options"])
	checks["both is disabled exactly while BOTH_ENABLED is false"] = plain.has("both") == (not SkillDef.BOTH_ENABLED)
	var gated: Dictionary = TestHelpers.attack("t_form_gate", {"weapon_types": ["sword"]})
	checks["a listed weapon type forces a proficiency minimum of at least 1"] = DictUtil.to_int(SkillFormModel.state(gated, "min_proficiency")["min"]) >= 1
	var line: Dictionary = TestHelpers.attack("t_form_line", {"area": "LINE"})
	checks["LINE caps reach_max at 1"] = DictUtil.to_int(SkillFormModel.state(line, "reach_max")["max"]) == 1
	var support: Dictionary = SkillFormModel.switch_kind(SkillFormModel.new_entry("t_form_sup"), "SUPPORT")
	var no_heal: Dictionary = support.duplicate(true)
	no_heal["heal_mult"] = 0.0
	checks["heal_stat is empty-only without a heal multiplier"] = SkillFormModel.state(no_heal, "heal_stat")["options"] == [""]
	var taunt: Dictionary = SkillFormModel.switch_kind(SkillFormModel.new_entry("t_form_taunt"), "DEFEND")
	taunt["grant_status"] = StatusEffects.TAUNT
	var modes: Dictionary = DictUtil.dict_of(SkillFormModel.state(taunt, "target_mode")["disabled_options"])
	checks["taunt is cast on the taunter (no ally modes)"] = modes.has("ALLY") or not SkillKinds.has_field("DEFEND", "target_mode")
	return TestHelpers.report(TAG, "Dynamic limits and disabled options", checks)

static func _test_kind_switch() -> bool:
	var checks: Dictionary = {}
	var entry: Dictionary = TestHelpers.attack("t_form_sw", {"power_mult": 2.0, "status_effect": "stagger", "status_potency": 30, "ranks": [{"level": 3, "power_mult": 2.5, "status_potency": 40}]})
	var support: Dictionary = SkillFormModel.switch_kind(entry, "SUPPORT")
	var defaults: Dictionary = ContentSchema.defaults("skills")
	checks["fields the new kind lacks go back to their default"] = is_equal_approx(DictUtil.to_float(support["power_mult"]), DictUtil.to_float(defaults["power_mult"])) and str(support["status_effect"]) == str(defaults["status_effect"])
	var mods: Array[Dictionary] = SkillRanks.modifiers_of(support)
	checks["modifiers for fields the new kind lacks are dropped, others kept"] = mods.size() == 1 and not mods[0].has("power_mult") and not mods[0].has("status_potency") and mods[0].has("level")
	var seeded: Dictionary = SkillFormModel.switch_kind(SkillFormModel.new_entry("t_form_k"), "DISENGAGE")
	checks["a kind default fills a field still at its default"] = DictUtil.to_int(seeded["jump_tiles"]) >= 1
	var granting: Dictionary = SkillFormModel.switch_kind(SkillFormModel.new_entry("t_form_g"), "DEFEND")
	var offered: Array[String] = SkillFormModel.grant_options("DEFEND")
	granting["grant_status"] = offered[offered.size() - 1]
	var to_focus: Dictionary = SkillFormModel.switch_kind(granting, "FOCUS")
	checks["an invalid grant is cleared"] = str(to_focus["grant_status"]).is_empty()
	var every_grant_ok: bool = true
	for status: String in SkillFormModel.grant_options("DEFEND"):
		var probe: Dictionary = SkillFormModel.switch_kind(SkillFormModel.new_entry("t_form_gp"), "DEFEND")
		probe["grant_status"] = status
		probe["mitigation"] = 0.0
		var problems: Array[String] = TestHelpers.messages(ContentCatalog.validate_skill_entry(probe))
		every_grant_ok = every_grant_ok and not TestHelpers.any_contains(problems, "can't grant") and not TestHelpers.any_contains(problems, "cannot grant")
	checks["no offered DEFEND grant is rejected as a wrong buff"] = every_grant_ok
	return TestHelpers.report(TAG, "Kind switching", checks)

static func _test_save_state() -> bool:
	var checks: Dictionary = {}
	var good: Dictionary = TestHelpers.attack("t_form_ok")
	var state: Dictionary = SkillFormModel.save_state(good)
	checks["a valid draft is not blocked"] = not DictUtil.to_bool(state["blocked"])
	var bad: Dictionary = TestHelpers.attack("t_form_bad", {"description": "{poer}"})
	var blocked: Dictionary = SkillFormModel.save_state(bad)
	checks["a bad token blocks Save with a reason"] = DictUtil.to_bool(blocked["blocked"]) and not Array(blocked["reasons"]).is_empty()
	checks["the problem is attributed to the description field"] = not SkillFormModel.problems_for(blocked, "description").is_empty()
	var long_name: Dictionary = TestHelpers.attack("t_form_long", {"name": "N".repeat(31), "ranks": [{"level": 3}]})
	var long_state: Dictionary = SkillFormModel.save_state(long_name)
	checks["the longest derived rank name counts against the cap"] = DictUtil.to_bool(long_state["blocked"]) and not SkillFormModel.problems_for(long_state, "name").is_empty()
	var rank_bad: Dictionary = TestHelpers.attack("t_form_rk", {"ranks": [{"heal_base": 5}]})
	var rank_state: Dictionary = SkillFormModel.save_state(rank_bad)
	checks["a modifier for a field the kind lacks blocks Save at that rank"] = DictUtil.to_bool(rank_state["blocked"]) and not SkillFormModel.problems_for(rank_state, "heal_base", 2).is_empty()
	var digits: Dictionary = TestHelpers.attack("t_form_dig", {"description": "Hits for 150 now."})
	var digit_state: Dictionary = SkillFormModel.save_state(digits)
	checks["literal digits are a warning and do not block"] = not DictUtil.to_bool(digit_state["blocked"]) and not Array(digit_state["warnings"]).is_empty()
	return TestHelpers.report(TAG, "Save state and reasons", checks)

static func _test_ranks_view() -> bool:
	var checks: Dictionary = {}
	var entry: Dictionary = TestHelpers.attack("t_form_rv", {"power_mult": 1.0, "ranks": [{"level": 3, "power_mult": 1.4}, {"level": 4, "power_mult": 1.8}]})
	var view: Dictionary = SkillFormModel.ranks_view(entry)
	var rows: Array = view["rows"]
	checks["one row per rank"] = rows.size() == 3
	var r2: Dictionary = DictUtil.dict_of(rows[1])
	checks["rank 2 shows the resolved values and highlights its modifiers"] = is_equal_approx(DictUtil.to_float(DictUtil.dict_of(r2["values"])["power_mult"]), 1.4) and Array(r2["modified"]).has("power_mult") and Array(r2["modified"]).has("level")
	var r1: Dictionary = DictUtil.dict_of(rows[0])
	checks["rank 1 has no highlighted modifiers"] = Array(r1["modified"]).is_empty()
	checks["rank names are derived"] = str(r2["name"]) == SkillRanks.rank_name(str(entry["name"]), 2)
	checks["a full family can't add; any family above one rank can remove"] = not DictUtil.to_bool(view["can_add"]) and DictUtil.to_bool(view["can_remove"])
	var edited: Dictionary = SkillFormModel.set_modifier(entry, 2, "power_mult", 1.2)
	var edited_rows: Array = SkillFormModel.ranks_view(edited)["rows"]
	var e3: Dictionary = DictUtil.dict_of(DictUtil.dict_of(edited_rows[2])["values"])
	checks["editing rank 2 leaves rank 3 untouched (no cascade)"] = is_equal_approx(DictUtil.to_float(e3["power_mult"]), 1.8)
	var base_value: Variant = SkillRanks.effective(entry, 1)["power_mult"]
	var dropped: Dictionary = SkillFormModel.set_modifier(entry, 2, "power_mult", base_value)
	checks["a value equal to the base drops the modifier"] = not SkillRanks.modifiers_of(dropped)[0].has("power_mult")
	var reset: Dictionary = SkillFormModel.reset_modifier(entry, 3, "level")
	checks["reset removes one cell"] = not SkillRanks.modifiers_of(reset)[1].has("level") and SkillRanks.modifiers_of(reset)[1].has("power_mult")
	var copied: Dictionary = SkillFormModel.copy_previous(SkillFormModel.reset_modifier(entry, 3, "power_mult"), 3)
	checks["copy previous copies the values, no link remains"] = is_equal_approx(DictUtil.to_float(SkillRanks.modifiers_of(copied)[1].get("power_mult", 0.0)), 1.4)
	var one: Dictionary = TestHelpers.attack("t_form_one")
	var added: Dictionary = SkillFormModel.add_rank(one)
	checks["add rank appends an empty modifier set"] = SkillRanks.rank_count(added) == 2
	var capped: Dictionary = added
	for i: int in range(StatRules.MAX_CLASS_TIER + 2):
		capped = SkillFormModel.add_rank(capped)
	checks["add rank stops at the class cap"] = SkillRanks.rank_count(capped) == StatRules.MAX_CLASS_TIER
	checks["remove top rank returns to a single rank without a ranks key"] = not SkillFormModel.remove_top_rank(added).has("ranks")
	var basic: Dictionary = TestHelpers.attack("t_form_basic", {"basic": true})
	checks["a basic skill shows 'single rank' and can't add"] = not DictUtil.to_bool(SkillFormModel.ranks_view(basic)["can_add"]) and str(SkillFormModel.ranks_editable(basic)["reason"]) == "single rank"
	return TestHelpers.report(TAG, "Ranks view and edits", checks)

static func _test_description_editor() -> bool:
	var checks: Dictionary = {}
	var entry: Dictionary = TestHelpers.attack("t_form_desc", {"power_mult": 1.0, "description": "{name}: {power}", "ranks": [{"level": 3, "power_mult": 1.5}]})
	var preview: Array[Dictionary] = SkillFormModel.description_preview(entry)
	checks["one preview line per rank"] = preview.size() == 2
	checks["the preview is the engine-rendered text"] = str(preview[1]["text"]) == str(SkillRanks.expand(entry)[1]["description"]) and int(preview[1]["length"]) == str(preview[1]["text"]).length()
	checks["the rank differences show in the preview"] = str(preview[0]["text"]) != str(preview[1]["text"])
	var palette: Array[String] = SkillFormModel.token_palette(entry)
	checks["the palette lists exactly the valid tokens for the kind"] = palette == SkillText.tokens_for("ATTACK") and not palette.has("heal")
	var support: Array[String] = SkillFormModel.token_palette(SkillFormModel.switch_kind(entry, "SUPPORT"))
	checks["the palette follows the kind"] = support.has("heal") and not support.has("power")
	var inserted: Dictionary = SkillFormModel.insert_token("Hits for .", 9, "power")
	checks["a token inserts at the caret and moves it"] = str(inserted["text"]) == "Hits for {power}." and int(inserted["caret"]) == 9 + "{power}".length()
	var clamped: Dictionary = SkillFormModel.insert_token("ab", 99, "name")
	checks["a caret past the end is clamped"] = str(clamped["text"]) == "ab{name}"
	return TestHelpers.report(TAG, "Description editor", checks)

static func _test_filter() -> bool:
	var checks: Dictionary = {}
	var attack: Dictionary = TestHelpers.attack("t_form_fa", {"name": "Zed Slash", "ranks": [{"level": 3}]})
	var heal: Dictionary = SkillFormModel.switch_kind(TestHelpers.attack("t_form_fh", {"name": "Alpha Mend", "grade": "advanced"}), "SUPPORT")
	var list: Array[Dictionary] = [attack, heal]
	var filters: Dictionary = SkillFilter.default_filters()
	checks["default filters pass both"] = SkillFilter.apply(list, filters).size() == 2
	filters["text"] = "ZED"
	checks["text search is case-insensitive over id, name and description"] = SkillFilter.apply(list, filters).size() == 1
	filters = SkillFilter.default_filters()
	filters["kinds"] = ["SUPPORT"]
	checks["kind chip"] = SkillFilter.apply(list, filters).size() == 1 and str(SkillFilter.apply(list, filters)[0]["id"]) == "t_form_fh"
	filters = SkillFilter.default_filters()
	filters["rank_count"] = 2
	checks["rank count chip"] = SkillFilter.apply(list, filters).size() == 1 and str(SkillFilter.apply(list, filters)[0]["id"]) == "t_form_fa"
	filters = SkillFilter.default_filters()
	filters["grade"] = "advanced"
	checks["grade chip"] = SkillFilter.apply(list, filters).size() == 1 and str(SkillFilter.apply(list, filters)[0]["id"]) == "t_form_fh"
	filters = SkillFilter.default_filters()
	var sorted_names: Array[String] = []
	for e: Dictionary in SkillFilter.apply(list, filters, "name"):
		sorted_names.append(str(e["name"]))
	checks["name sort"] = sorted_names == ["Alpha Mend", "Zed Slash"]
	var by_ranks: Array[Dictionary] = SkillFilter.apply(list, filters, "ranks")
	checks["rank-count sort puts the larger family first"] = str(by_ranks[0]["id"]) == "t_form_fa"
	var engine: Dictionary = TestHelpers.attack("t_form_eng", {"system": true})
	checks["engine rows are hidden unless show_system"] = SkillFilter.apply([engine], filters).is_empty()
	filters["show_system"] = true
	checks["...and shown when asked"] = SkillFilter.apply([engine], filters).size() == 1
	var shipped: Array[Dictionary] = ContentCatalog.skill_entries()
	var unused: int = 0
	var used: int = 0
	for e: Dictionary in shipped:
		if SkillFilter.used_as(e, "unused"):
			unused += 1
		else:
			used += 1
	checks["usage 'unused' agrees with references_to over the catalog"] = unused + used == shipped.size()
	return TestHelpers.report(TAG, "SkillFilter predicates", checks)

static func _test_key_fields_and_requires() -> bool:
	var checks: Dictionary = {}
	var system_head: String = ""
	var advanced_head: String = ""
	for row: Dictionary in ContentCatalog.entries("skills"):
		if not str(row.get("family", "")).is_empty():
			continue
		if system_head.is_empty() and ContentCatalog.is_system_origin(row) and not DictUtil.to_bool(row.get("system", false)) and not DictUtil.to_bool(row.get("basic", false)):
			system_head = str(row["id"])
	if not system_head.is_empty():
		# TOOLS-RO: the viewer has no locks; the data rule (ContentCatalog.key_field_errors) still refuses a key-field change.
		var stored: Dictionary = ContentCatalog.skill_entry(system_head)
		var other_grade: Dictionary = stored.duplicate(true)
		other_grade["grade"] = "advanced" if str(stored.get("grade", "novice")) == "novice" else "novice"
		var key_problem: Callable = func(p: Dictionary) -> bool: return str(p["field"]) == "key"
		checks["a key-field change of a stored system skill is a validation problem"] = ContentCatalog.validate_skill_entry(other_grade, system_head).any(key_problem)
		ContentCatalog.begin_system_edit()
		checks["...but not inside the system-edit scope"] = not ContentCatalog.validate_skill_entry(other_grade, system_head).any(key_problem)
		ContentCatalog.end_system_edit()
		checks["the field state carries no lock"] = not SkillFormModel.state(stored, "kind", system_head).has("locked")
	else:
		checks["a shipped system skill exists to test the key-field rule"] = false
	var fresh: Dictionary = SkillFormModel.new_entry("t_form_new")
	checks["a new draft is a user asset"] = str(fresh["origin"]) == ContentSchema.ORIGIN_USER
	checks["requires is disabled while the grade is novice"] = not DictUtil.to_bool(SkillFormModel.state(fresh, "requires")["enabled"])
	fresh["grade"] = "advanced"
	checks["requires is enabled for a higher grade"] = DictUtil.to_bool(SkillFormModel.state(fresh, "requires")["enabled"])
	var choices: Array[String] = SkillFormModel.requires_choices(fresh, "")
	var choices_ok: bool = not choices.is_empty()
	for id: String in choices:
		var row: Dictionary = ContentCatalog.get_entry("skills", id)
		choices_ok = choices_ok and ContentCatalog.grade_rank(str(row.get("grade", "novice"))) < ContentCatalog.grade_rank("advanced") and not DictUtil.to_bool(row.get("basic", false)) and not DictUtil.to_bool(row.get("system", false))
		if str(row.get("grade", "novice")) == "advanced":
			advanced_head = id
	checks["the requires picker offers only lower-grade, non-basic, non-engine heads"] = choices_ok and advanced_head.is_empty()
	var grade_state: Dictionary = DictUtil.dict_of(SkillFormModel.state(fresh, "grade")["disabled_options"])
	checks["the expert grade is listed disabled (reserved)"] = grade_state.has("expert")
	var basic_state: Dictionary = SkillFormModel.state(fresh, "basic")
	checks["the basic box is disabled with a reason"] = not DictUtil.to_bool(basic_state["enabled"]) and not str(basic_state["reason"]).is_empty()
	return TestHelpers.report(TAG, "Key-field rule and requires picker", checks)
