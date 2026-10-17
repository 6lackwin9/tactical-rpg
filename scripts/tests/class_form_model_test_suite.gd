class_name ClassFormModelTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Class Form Model Test Suite (CLASS1-L, FORGE1_CLASSES 2-5)
# The headless class tab: five sections from the schema groups, control ranges equal to the rule constants, a new class
# that validates, problems from errors (never from warnings), the key-field rule on system classes (no viewer locks),
# the path ordering helper and standard / custom levels, the default-loadout preview (acceptance 6), the family picker, the gear pickers
# grouped by item family with the dual-wield guard and the accessory flag, the stat / budget / coverage panels and the
# unit AI group visibility, and the read-only Forge viewer class tab. Read-only on the shipped catalog (fixtures by rule).
# ==============================================================================

const TAG: String = "ClassForm"

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING CLASS FORM MODEL TEST (CLASS1-L)        ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_layout_and_ranges() and ok
	ok = _test_new_and_save_state() and ok
	ok = _test_key_fields() and ok
	ok = _test_path_helpers() and ok
	ok = _test_default_loadout() and ok
	ok = _test_pickers() and ok
	ok = _test_panels() and ok
	ok = _test_unit_ai() and ok
	ok = _test_forge_ui(tree_parent) and ok
	return ok

static func _shipped(id: String = "") -> Dictionary:
	return ContentCatalog.get_entry("classes", id if not id.is_empty() else ContentCatalog.ids("classes")[0])

# ------------------------------------------------------------------------------

static func _test_layout_and_ranges() -> bool:
	var checks: Dictionary = {}
	var grouped: bool = true
	for f: Dictionary in ContentSchema.fields("classes"):
		grouped = grouped and ClassUnitFields.CLASS_GROUPS.has(str(f.get("group", "")))
	checks["every class field has a group in the five sections"] = grouped
	var ids: Array[String] = []
	var all_keys: Array[String] = []
	for block: Dictionary in ClassForgeModel.layout("classes"):
		ids.append(str(block["id"]))
		for k: Variant in DictUtil.array_of(block["fields"]):
			all_keys.append(str(k))
	checks["layout = the five sections in order"] = ids == ClassUnitFields.CLASS_GROUPS
	checks["layout lists every class field once"] = all_keys.size() == ContentSchema.fields("classes").size()
	checks["ai_role sits in identity; no other AI control on the class"] = ClassForgeModel.group_of("classes", "ai_role") == "identity" and ["target_pref", "flee_ratio", "defend_below", "reckless", "heal_below"].all(func(k: String) -> bool: return ContentSchema.field("classes", k).is_empty())
	var r: Dictionary = ClassForgeModel.ranges()
	var base_r: Dictionary = r["base_stats"]
	var growth_r: Dictionary = r["growth"]
	var lvl_r: Dictionary = r["proficiency_level"]
	var sc_r: Dictionary = r["proficiency_scaling"]
	var path_r: Dictionary = r["path_level"]
	checks["base stat range = StatRules floor / ceiling / budget"] = DictUtil.to_int(base_r["min"]) == StatRules.STAT_FLOOR and DictUtil.to_int(base_r["max"]) == StatRules.STAT_CEILING and DictUtil.to_int(base_r["total"]) == StatRules.CLASS_BUDGET
	checks["growth range = 0..GROWTH_STAT_MAX, total GROWTH_TOTAL"] = DictUtil.to_int(growth_r["max"]) == StatRules.GROWTH_STAT_MAX and DictUtil.to_int(growth_r["total"]) == StatRules.GROWTH_TOTAL
	checks["proficiency level 1..MAX_LEVEL, scaling MIN..MAX, default DEFAULT_SCALING"] = DictUtil.to_int(lvl_r["max"]) == Proficiency.MAX_LEVEL and is_equal_approx(DictUtil.to_float(sc_r["min"]), Proficiency.MIN_SCALING) and is_equal_approx(DictUtil.to_float(sc_r["max"]), Proficiency.MAX_SCALING) and is_equal_approx(DictUtil.to_float(sc_r["default"]), Proficiency.DEFAULT_SCALING)
	checks["path levels 1..MAX_LEVEL with the standard list"] = DictUtil.to_int(path_r["max"]) == StatRules.MAX_LEVEL and DictUtil.array_of(path_r["standard"]) == Array(ClassArchetype.STANDARD_PATH_LEVELS)
	var schema_base: Dictionary = ContentSchema.field("classes", "base_stats")
	checks["schema base_stats range = the same constants"] = DictUtil.to_int(schema_base["min"]) == StatRules.STAT_FLOOR and DictUtil.to_int(schema_base["max"]) == StatRules.STAT_CEILING
	checks["pouch limit = the inventory battle pouch"] = ClassForgeModel.pouch_limit() == Inventory.new().max_pouch_slots
	return TestHelpers.report(TAG, "Five sections from the schema; ranges are the rule constants", checks)

static func _test_new_and_save_state() -> bool:
	var checks: Dictionary = {}
	var fresh_id: String = ContentCatalog.unique_id("classes", "t_cf_new")
	var fresh: Dictionary = ClassForgeModel.new_entry(fresh_id)
	checks["a new class validates"] = ClassForgeModel.draft_problems(fresh).is_empty()
	checks["...is a user class with one weapon row and an empty path"] = str(fresh["origin"]) == ContentSchema.ORIGIN_USER and DictUtil.dict_of(fresh["proficiency"]).size() == 1 and DictUtil.dict_of(fresh["skill_path"]).is_empty()
	checks["...Remaining is 0 for base stats and growth"] = ClassForgeModel.remaining(fresh, "base_stats") == 0 and ClassForgeModel.remaining(fresh, "growth") == 0
	var state: Dictionary = ClassForgeModel.save_state(fresh)
	checks["...Save is allowed"] = not DictUtil.to_bool(state["blocked"])
	var off: Dictionary = fresh.duplicate(true)
	var stats: Dictionary = DictUtil.dict_of(off["base_stats"]).duplicate()
	stats[StatRules.STATS[0]] = DictUtil.to_int(stats[StatRules.STATS[0]]) + 1
	off["base_stats"] = stats
	var off_state: Dictionary = ClassForgeModel.save_state(off)
	checks["an off-budget draft blocks Save with a reason on base_stats"] = DictUtil.to_bool(off_state["blocked"]) and not DictUtil.array_of(off_state["reasons"]).is_empty() and ClassForgeModel.draft_problems(off).any(func(p: Dictionary) -> bool: return str(p["field"]) == "base_stats")
	checks["...Remaining shows -1"] = ClassForgeModel.remaining(off, "base_stats") == -1
	var long_path: Dictionary = {}
	for step: Dictionary in _all_plain_heads():
		long_path[str(step["id"])] = 1
	var warned: Dictionary = fresh.duplicate(true)
	warned["skill_path"] = long_path
	var w_state: Dictionary = ClassForgeModel.save_state(warned)
	checks["a draft with warnings only is not blocked"] = long_path.size() > StatRules.MAX_CLASS_SLOTS and not DictUtil.to_bool(w_state["blocked"]) and not DictUtil.array_of(w_state["warnings"]).is_empty()
	var bad_path: Dictionary = fresh.duplicate(true)
	bad_path["skill_path"] = {SkillCatalog.basic_ids()[0]: 1}
	checks["a basic in the path blocks Save on skill_path"] = ClassForgeModel.draft_problems(bad_path).any(func(p: Dictionary) -> bool: return str(p["field"]) == "skill_path")
	return TestHelpers.report(TAG, "New class validates; errors block Save, warnings never do", checks)

# Plain novice ATTACK heads that need no gear and nothing else (picked by rule), as family_choices rows.
static func _all_plain_heads() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for row: Dictionary in ClassForgeModel.family_choices({}):
		if str(row["grade"]) == "novice" and DictUtil.array_of(row["requires"]).is_empty():
			out.append(row)
	return out

# TOOLS-RO: the viewer has no locks (no field state carries one); the data rule (ContentCatalog.key_field_errors) still
# makes a key-field change of a system class a problem, outside the system-edit scope only.
static func _test_key_fields() -> bool:
	var checks: Dictionary = {}
	var system: Dictionary = _shipped()
	var id: String = str(system["id"])
	var no_lock: bool = true
	for f: Dictionary in ContentSchema.fields("classes"):
		no_lock = no_lock and not ClassForgeModel.state(system, str(f["key"]), id).has("locked")
	checks["no class field state carries a lock"] = no_lock
	var changed: Dictionary = system.duplicate(true)
	changed["ai_role"] = "SKIRMISHER" if str(changed["ai_role"]) != "SKIRMISHER" else "AGGRESSOR"
	var role_problem: Callable = func(p: Dictionary) -> bool: return str(p["field"]) == "ai_role"
	checks["a key-field change of a system class is a problem"] = ClassForgeModel.draft_problems(changed, id).any(role_problem)
	ContentCatalog.begin_system_edit()
	checks["...but not inside the system-edit scope"] = not ClassForgeModel.draft_problems(changed, id).any(role_problem)
	ContentCatalog.end_system_edit()
	return TestHelpers.report(TAG, "System classes: key-field rule (no viewer locks)", checks)

static func _test_path_helpers() -> bool:
	var checks: Dictionary = {}
	var steps: Array[Dictionary] = ClassForgeModel.ordered_path({"b": 3, "a": 1, "c": 3})
	checks["ordered by level, then entry position"] = _skills(steps) == ["a", "b", "c"]
	checks["swapping two same-level entries swaps their order"] = _skills(ClassForgeModel.ordered_path({"c": 3, "a": 1, "b": 3})) == ["a", "c", "b"]
	checks["changing a level re-sorts"] = _skills(ClassForgeModel.ordered_path({"b": 3, "a": 1, "c": 1})) == ["a", "c", "b"]
	var e: Dictionary = {"skill_path": {"b": 3, "a": 1}}
	checks["set_path_level edits in place (key position unchanged)"] = DictUtil.dict_of(ClassForgeModel.set_path_level(e, "b", 7)["skill_path"]).keys() == ["b", "a"]
	var heads: Array[Dictionary] = _all_plain_heads()
	checks["two plain heads exist"] = heads.size() >= 2
	if heads.size() >= 2:
		var h0: String = str(heads[0]["id"])
		var h1: String = str(heads[1]["id"])
		var added: Dictionary = ClassForgeModel.add_path_family({"skill_path": {"b": 3}}, h0)
		checks["a new family goes last"] = DictUtil.dict_of(added["skill_path"]).keys() == ["b", h0]
		var removed: Dictionary = ClassForgeModel.remove_path_family({"skill_path": {"x": 1, h0: 3, "y": 5}}, h0)
		checks["removal keeps the others' order"] = DictUtil.dict_of(removed["skill_path"]).keys() == ["x", "y"]
		var replaced: Dictionary = ClassForgeModel.replace_path_family({"skill_path": {"x": 1, h0: 3, "y": 5}}, h0, h1)
		checks["replacing a row's family keeps position and level"] = DictUtil.dict_of(replaced["skill_path"]).keys() == ["x", h1, "y"] and DictUtil.to_int(DictUtil.dict_of(replaced["skill_path"])[h1]) == 3
		var choices: Array[Dictionary] = ClassForgeModel.family_choices({"skill_path": {h0: 1}})
		checks["the picker flags a used family"] = choices.any(func(c: Dictionary) -> bool: return str(c["id"]) == h0 and DictUtil.to_bool(c["used"]))
		checks["the picker never lists basics, engine skills or higher tiers"] = choices.all(func(c: Dictionary) -> bool:
			var row: Dictionary = ContentCatalog.get_entry("skills", str(c["id"]))
			return not DictUtil.to_bool(row.get("basic", false)) and not DictUtil.to_bool(row.get("system", false)) and DictUtil.to_int(row.get("tier", 1), 1) == 1)
		checks["the counter names the entries and the active slots"] = ClassForgeModel.path_counter({"skill_path": {h0: 1, h1: 3}}) == "2 families, first %d active by default" % StatRules.MAX_CLASS_SLOTS
	var standard: Array[int] = ClassArchetype.STANDARD_PATH_LEVELS.duplicate()
	standard.append(ClassForgeModel.CUSTOM_LEVEL)
	checks["a novice family offers every standard level plus Custom"] = heads.is_empty() or ClassForgeModel.level_options(str(heads[0]["id"])) == standard
	var advanced: String = ""
	for row: Dictionary in ClassForgeModel.family_choices({}):
		if str(row["grade"]) != "novice":
			advanced = str(row["id"])
			break
	checks["an advanced family offers only levels >= ADVANCED_MIN_LEVEL"] = not advanced.is_empty() and ClassForgeModel.level_options(advanced).all(func(l: int) -> bool: return l == ClassForgeModel.CUSTOM_LEVEL or l >= StatRules.ADVANCED_MIN_LEVEL)
	var custom: int = -1
	for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
		if not ClassArchetype.STANDARD_PATH_LEVELS.has(lvl):
			custom = lvl
			break
	checks["a level outside the standard list is tagged custom"] = custom > 0 and ClassForgeModel.is_custom_level(custom) and not ClassForgeModel.is_custom_level(ClassArchetype.STANDARD_PATH_LEVELS[0])
	var rows: Array[Dictionary] = ClassForgeModel.path_rows(_shipped())
	checks["path rows follow entry order with a ladder per row"] = rows.size() == DictUtil.dict_of(_shipped()["skill_path"]).size() and rows.all(func(r: Dictionary) -> bool: return str(r["ladder"]).begins_with("T1 Lv %d" % DictUtil.to_int(r["level"])))
	return TestHelpers.report(TAG, "Path table: ordering, standard / custom levels, picker, ladder", checks)

static func _skills(steps: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for s: Dictionary in steps:
		out.append(str(s["skill"]))
	return out

# Acceptance 6: preview == active_class_families of a built Lv MAX unit with an empty loadout == first 5 of path_steps.
static func _test_default_loadout() -> bool:
	var checks: Dictionary = {}
	var all_ok: bool = true
	var detail: Array[String] = []
	for e: Dictionary in ContentCatalog.entries("classes"):
		var preview: Array[String] = ClassForgeModel.default_loadout(e, StatRules.MAX_LEVEL)
		var unit: Combatant = ContentFactory.unit({"id": "t_cf_u", "name": "U", "class_id": str(e["id"]), "level": StatRules.MAX_LEVEL}, "", 12)
		var first_five: Array[String] = []
		for step: Dictionary in ContentFactory.class_archetype(e).path_steps():
			if first_five.size() < StatRules.MAX_CLASS_SLOTS:
				first_five.append(SkillCatalog.family_of(str(step["skill"])))
		var same: bool = preview == unit.active_class_families and preview == first_five
		if not same:
			detail.append(str(e["id"]))
		all_ok = all_ok and same
	checks["every shipped class: preview = built unit = first five by (level, position)%s" % ("" if detail.is_empty() else " (" + ", ".join(detail) + ")")] = all_ok
	var heads: Array[Dictionary] = _all_plain_heads()
	if heads.size() >= StatRules.MAX_CLASS_SLOTS + 1:
		var path: Dictionary = {}
		for i: int in range(StatRules.MAX_CLASS_SLOTS + 1):
			path[str(heads[i]["id"])] = 3
		var swapped: Dictionary = {}
		var keys: Array = path.keys()
		swapped[keys[1]] = 3
		swapped[keys[0]] = 3
		for i: int in range(2, keys.size()):
			swapped[keys[i]] = 3
		var a: Array[String] = ClassForgeModel.default_loadout({"skill_path": path})
		var b: Array[String] = ClassForgeModel.default_loadout({"skill_path": swapped})
		checks["reordering two same-level entries swaps them in the preview"] = a.size() == StatRules.MAX_CLASS_SLOTS and a[0] == b[1] and a[1] == b[0]
		var later: Dictionary = path.duplicate()
		later[keys[0]] = 7
		checks["raising a level re-sorts (the sixth entry becomes active)"] = not ClassForgeModel.default_loadout({"skill_path": later}).has(str(keys[0])) and ClassForgeModel.default_loadout({"skill_path": later}).has(str(keys[keys.size() - 1]))
	return TestHelpers.report(TAG, "Default-loadout preview (acceptance 6)", checks)

static func _test_pickers() -> bool:
	var checks: Dictionary = {}
	var e: Dictionary = _shipped()
	var main_choices: Array[Dictionary] = ClassForgeModel.slot_choices(e, "main_hand")
	checks["main hand choices exist and all fit the slot"] = not main_choices.is_empty() and main_choices.all(func(c: Dictionary) -> bool: return ContentFactory.item_fits_slot(ContentCatalog.get_entry("items", str(c["id"])), "MAIN_HAND"))
	var groups_seen: Array[String] = []
	var contiguous: bool = true
	var last: String = ""
	for c: Dictionary in main_choices:
		var g: String = str(c["group"])
		if g != last:
			contiguous = contiguous and not groups_seen.has(g)
			groups_seen.append(g)
			last = g
	checks["choices are grouped by item family (each group once, contiguous)"] = contiguous and groups_seen.size() > 1
	var prof: Proficiency = Proficiency.from_dict(DictUtil.dict_of(e["proficiency"]))
	checks["untrained items are disabled with 'No training'"] = main_choices.all(func(c: Dictionary) -> bool:
		var gear: String = str(ContentCatalog.get_entry("items", str(c["id"])).get("weapon_type", ""))
		var trained: bool = gear.is_empty() or prof.can_use(gear)
		return DictUtil.to_bool(c["enabled"]) == trained and (trained or str(c["reason"]).contains("No training")))
	var saved_dw: bool = Equipment.allow_plain_dual_wield
	Equipment.allow_plain_dual_wield = false
	var strict_off: Array[Dictionary] = ClassForgeModel.slot_choices(e, "off_hand")
	checks["dual-wield guard off: no main-hand weapon is offered for the off hand"] = strict_off.all(func(c: Dictionary) -> bool: return ContentFactory.item(ContentCatalog.get_entry("items", str(c["id"]))) is Equipment and (ContentFactory.item(ContentCatalog.get_entry("items", str(c["id"]))) as Equipment).hand() != Equipment.HAND_MAIN_1H)
	Equipment.allow_plain_dual_wield = true
	var loose_off: Array[Dictionary] = ClassForgeModel.slot_choices(e, "off_hand")
	checks["dual-wield guard on: plain 1H weapons join the off-hand list"] = loose_off.size() > strict_off.size()
	Equipment.allow_plain_dual_wield = saved_dw
	var two_handed: String = ""
	for item: Dictionary in ContentCatalog.entries("items"):
		if DictUtil.to_bool(item.get("two_handed", false)):
			two_handed = str(item["id"])
			break
	var with_2h: Dictionary = e.duplicate(true)
	with_2h["main_hand"] = two_handed
	checks["a two-handed main hand disables the off hand with the reason"] = not two_handed.is_empty() and not DictUtil.to_bool(ClassForgeModel.state(with_2h, "off_hand", "")["enabled"]) and str(ClassForgeModel.state(with_2h, "off_hand", "")["reason"]).contains("two-handed")
	var ring: String = ""
	for item: Dictionary in ContentCatalog.entries("items"):
		if str(item.get("item_type", "")) == "ACCESSORY":
			ring = str(item["id"])
			break
	var acc: Dictionary = e.duplicate(true)
	acc["accessory_1"] = ring
	var greyed: Dictionary = DictUtil.dict_of(ClassForgeModel.state(acc, "accessory_2", "")["disabled_options"])
	checks["the accessory in slot 1 is greyed in slot 2 while the flag is off"] = not ring.is_empty() and greyed.has(ring)
	var saved_acc: bool = StatRules.allow_duplicate_accessory
	StatRules.allow_duplicate_accessory = true
	checks["...and offered with the flag on"] = not DictUtil.dict_of(ClassForgeModel.state(acc, "accessory_2", "")["disabled_options"]).has(ring)
	StatRules.allow_duplicate_accessory = saved_acc
	var unit_choices: Array[Dictionary] = ClassForgeModel.slot_choices({"class_id": str(e["id"])}, "main_hand", "units")
	checks["unit pickers also offer 'none' first"] = not unit_choices.is_empty() and str(unit_choices[0]["id"]) == ContentSchema.NO_ITEM
	return TestHelpers.report(TAG, "Gear pickers: family groups, training, dual-wield guard, 2H, accessory flag", checks)

static func _test_panels() -> bool:
	var checks: Dictionary = {}
	var e: Dictionary = _shipped()
	var stats: Dictionary = ClassForgeModel.stat_panel(e)
	checks["stat panel: base total = budget for a shipped class"] = DictUtil.to_int(stats["base_total"]) == StatRules.CLASS_BUDGET and DictUtil.to_int(stats["growth_total"]) == StatRules.GROWTH_TOTAL
	checks["stat panel: six rows, three vitals rows (1, 5, MAX_LEVEL)"] = DictUtil.array_of(stats["rows"]).size() == StatRules.STATS.size() and DictUtil.array_of(stats["vitals"]).size() == 3
	var row0: Dictionary = DictUtil.dict_of(DictUtil.array_of(stats["rows"])[0])
	checks["stat panel: level 10 = base + growth x 9"] = DictUtil.to_int(row0["lv10"]) == DictUtil.to_int(row0["base"]) + DictUtil.to_int(row0["growth"]) * (StatRules.MAX_LEVEL - 1)
	var budget: Dictionary = ClassForgeModel.budget_panel(e)
	checks["budget panel total = GearBudget.loadout_value, band from TARGET_BUDGET"] = is_equal_approx(DictUtil.to_float(budget["total"]), GearBudget.loadout_value(e)) and is_equal_approx(DictUtil.to_float(budget["low"]), GearBudget.band_low()) and is_equal_approx(DictUtil.to_float(budget["unarmed_value"]), GearBudget.UNARMED_VALUE)
	var fist: Dictionary = ClassForgeModel.new_entry("t_cf_fist")
	fist["proficiency"] = {"sword": {"level": 1, "scaling": 1.5}, SkillDef.UNARMED_PROFICIENCY: {"level": 2, "scaling": 1.5}}
	var cov: Array[Dictionary] = ClassForgeModel.coverage(fist)
	checks["coverage: an empty main hand counts as using fist"] = not cov.any(func(r: Dictionary) -> bool: return str(r["text"]).contains("'%s'" % SkillDef.UNARMED_PROFICIENCY))
	checks["coverage: a trained type no item uses is an info row"] = cov.any(func(r: Dictionary) -> bool: return str(r["severity"]) == "info" and str(r["text"]).contains("'sword'"))
	checks["coverage: no armor trained is an info row"] = cov.any(func(r: Dictionary) -> bool: return str(r["severity"]) == "info" and str(r["text"]).contains("No armor"))
	var last_weapon: Dictionary = ClassForgeModel.can_remove_proficiency({"proficiency": {"sword": {"level": 1}}}, "sword")
	checks["the last weapon type cannot be removed"] = not DictUtil.to_bool(last_weapon["allowed"]) and not str(last_weapon["reason"]).is_empty()
	var main_type: String = str(ContentCatalog.get_entry("items", str(e.get("main_hand", ""))).get("weapon_type", ""))
	checks["a type the default gear uses cannot be removed"] = main_type.is_empty() or not DictUtil.to_bool(ClassForgeModel.can_remove_proficiency(e, main_type)["allowed"])
	checks["the add-type dropdown offers only unused types"] = ClassForgeModel.proficiency_type_choices(e).all(func(g: String) -> bool: return not DictUtil.dict_of(e["proficiency"]).has(g))
	return TestHelpers.report(TAG, "Stat, gear budget and proficiency coverage panels", checks)

static func _test_unit_ai() -> bool:
	var checks: Dictionary = {}
	var support_class: String = ""
	var other_class: String = ""
	for c: Dictionary in ContentCatalog.entries("classes"):
		if str(c["ai_role"]) == "SUPPORT" and support_class.is_empty():
			support_class = str(c["id"])
		elif str(c["ai_role"]) != "SUPPORT" and other_class.is_empty():
			other_class = str(c["id"])
	checks["heal_below shows for an effective SUPPORT role (class role)"] = support_class.is_empty() or ClassForgeModel.unit_field_visible({"class_id": support_class}, "heal_below")
	checks["...and for an own SUPPORT role"] = ClassForgeModel.unit_field_visible({"class_id": other_class, "ai_role": "SUPPORT"}, "heal_below")
	checks["...hidden otherwise"] = not ClassForgeModel.unit_field_visible({"class_id": other_class}, "heal_below")
	checks["reckless hides flee and guard thresholds"] = not ClassForgeModel.unit_field_visible({"reckless": true}, "flee_ratio") and not ClassForgeModel.unit_field_visible({"reckless": true}, "defend_below") and ClassForgeModel.unit_field_visible({}, "flee_ratio")
	var ai_keys: Array[String] = []
	for block: Dictionary in ClassForgeModel.layout("units", {"class_id": other_class, "ai_role": "SUPPORT"}):
		if str(block["id"]) == ClassUnitFields.UNIT_AI_GROUP:
			for k: Variant in DictUtil.array_of(block["fields"]):
				ai_keys.append(str(k))
	checks["the unit layout has an AI group with role, targets, thresholds, reckless, heal_below"] = ["ai_role", "target_pref", "flee_ratio", "defend_below", "reckless", "heal_below"].all(func(k: String) -> bool: return ai_keys.has(k))
	return TestHelpers.report(TAG, "Unit AI group visibility", checks)

# ------------------------------------------------------------------------------
# The class tab and the unit AI group in the Content Forge viewer (TOOLS-RO: read-only, on the shipped catalog).
# ------------------------------------------------------------------------------

static func _test_forge_ui(tree_parent: Node) -> bool:
	if tree_parent == null:
		return TestHelpers.report_ok(TAG, "Forge class tab viewer (needs a scene tree)", true, "skipped")
	var checks: Dictionary = {}
	var host: Node = (load("res://scenes/content_forge.tscn") as PackedScene).instantiate()
	tree_parent.add_child(host)
	var forge: ContentForge = host.get("forge")
	forge.show_kind("classes")
	var id: String = ContentCatalog.ids("classes")[0]
	forge.select(id)
	var system: Dictionary = ContentCatalog.get_entry("classes", id)
	checks["a class opens with the derived panels"] = forge.selected_id == id and forge.class_panels_text().contains("Gear budget") and forge.class_panels_text().contains("Proficiency coverage")
	var stats_box: Control = forge.form.field_control("base_stats")
	var spin: SpinBox = stats_box.find_child("Stat_" + StatRules.STATS[0], true, false) as SpinBox if stats_box != null else null
	checks["base stats: read-only spinners, ranges from StatRules"] = spin != null and not spin.editable and is_equal_approx(spin.min_value, StatRules.STAT_FLOOR) and is_equal_approx(spin.max_value, StatRules.STAT_CEILING)
	var name_edit: LineEdit = forge.form.field_control("name") as LineEdit
	checks["the name is shown read-only"] = name_edit != null and not name_edit.editable and name_edit.text == str(system["name"])
	var path_box: Control = forge.form.field_control("skill_path")
	var rows: Node = path_box.find_child("Rows", true, false) if path_box != null else null
	checks["the path table shows one row per entry"] = rows != null and rows.get_child_count() == DictUtil.dict_of(system["skill_path"]).size()
	var prof_box: Control = forge.form.field_control("proficiency")
	checks["the proficiency rows show one row per trained type"] = prof_box != null and prof_box.get_child_count() == DictUtil.dict_of(system["proficiency"]).size()
	var main_opt: OptionButton = forge.form.field_control("main_hand") as OptionButton
	var separators: int = 0
	if main_opt != null:
		for i: int in range(main_opt.item_count):
			if main_opt.is_item_separator(i):
				separators += 1
	checks["the main hand picker is grouped by item family and disabled"] = separators > 1 and main_opt.disabled
	var live: Array[String] = TestHelpers.live_controls(forge.form)
	checks["no control of the class form can edit or act (%s)" % ", ".join(live)] = live.is_empty()
	checks["no lock icon in the form"] = not forge.form.find_children("*", "Label", true, false).any(func(n: Node) -> bool: return (n as Label).text.contains("🔒"))
	# Units: the AI group follows ClassForgeModel.layout (heal_below only for SUPPORT, no thresholds while reckless).
	forge.show_kind("units")
	var picks: Array[String] = []
	for want: Callable in [
		func(u: Dictionary) -> bool: return ClassForgeModel.effective_role(u) != "SUPPORT" and not DictUtil.to_bool(u.get("reckless", false)),
		func(u: Dictionary) -> bool: return ClassForgeModel.effective_role(u) == "SUPPORT",
		func(u: Dictionary) -> bool: return DictUtil.to_bool(u.get("reckless", false))]:
		for u: Dictionary in ContentCatalog.entries("units"):
			if str(u.get("stat_mode", "derived")) == "derived" and want.call(u):
				picks.append(str(u["id"]))
				break
	var layout_ok: bool = not picks.is_empty()
	for unit_id: String in picks:
		forge.select(unit_id)
		var shown: Array[String] = []
		for block: Dictionary in ClassForgeModel.layout("units", forge.form.draft):
			for key: Variant in DictUtil.array_of(block["fields"]):
				shown.append(str(key))
		for key: String in ["heal_below", "flee_ratio", "defend_below"]:
			layout_ok = layout_ok and (forge.form.field_control(key) != null) == shown.has(key)
	checks["the unit AI group shows exactly the layout's fields (%d units by rule)" % picks.size()] = layout_ok
	host.queue_free()
	return TestHelpers.report(TAG, "Forge viewer: class tab and unit AI group (scene)", checks)
