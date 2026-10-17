class_name ContentTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Content Catalog & Forge Test Suite
# The shipped catalog (res://data/catalog) loads, validates, passes every check
# the old Forge editor ran on Apply / Save (TOOLS-RO) and matches the engine;
# entries round-trip through ContentFactory; validation catches bad content;
# content authored through the API saves, reloads and plays a full battle in
# the combat engine; the read-only Content Forge viewer browses it and playtests.
# Writes go to a sandbox copy (a TestHelpers sandbox) so the shipped catalog is
# never touched.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("content_test")

const TAG: String = "Content"

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING CONTENT CATALOG & FORGE TEST           ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_shipped_catalog() and ok
	ok = _test_shipped_save_checks() and ok
	ok = _test_engine_parity() and ok
	ok = _test_round_trip() and ok
	ok = _test_validation() and ok

	# Everything below writes: work on a sandbox copy of the catalog.
	var forked: bool = TestHelpers.fork(SANDBOX)
	ok = TestHelpers.report_ok(TAG, "Sandbox fork of the shipped catalog", forked and ContentCatalog.ids("skills").size() > 0) and ok
	if forked:
		ok = _test_authoring() and ok
		ok = _test_authored_units_in_planning() and ok
		ok = _test_full_integration() and ok
		ok = _test_forge_ui(tree_parent) and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

# ------------------------------------------------------------------------------

static func _test_shipped_catalog() -> bool:
	var counts: Array[String] = []
	var all_present: bool = true
	for kind: String in ContentSchema.KINDS:
		var n: int = ContentCatalog.entries(kind).size()
		counts.append("%s %d" % [kind, n])
		# item_families stay empty until ITEM1-A migrates items.json (legacy flat items, ContentCatalog.items_inherit()).
		# terrain stays empty until S12T-A writes terrain.json (Terrain code defaults apply; warnings_all lists the rows).
		all_present = all_present and (n > 0 or kind == "item_families" or (kind == "terrain" and not ContentCatalog.terrain_file_exists()))
	var problems: Dictionary = ContentCatalog.validate_all()
	return TestHelpers.report_ok(TAG, "Shipped catalog loads every kind and every entry validates", all_present and problems.is_empty() and ContentCatalog.load_errors().is_empty(), ", ".join(counts) if problems.is_empty() else str(problems))

# TOOLS-RO: every check the Content Forge ran before Apply / Save, over all shipped data with no tool open. validate_all()
# covers validate_entry per row (schema, references, icon keys, skill / item / class / unit rules, key fields stay put),
# the required engine skills and the skill fold; this adds the per-entry checks Apply ran on top: the ranked skill family
# (rank count, derived rank names, rank modifiers, template tokens: validate_skill_entry), the class draft problems (key
# fields included), the item and family table problems (ItemFormModel.draft_problems), and the save() refusals.
static func _test_shipped_save_checks() -> bool:
	var checks: Dictionary = {}
	var bad: Array[String] = []
	for ranked: Dictionary in ContentCatalog.skill_entries():
		for p: Dictionary in ContentCatalog.validate_skill_entry(ranked, str(ranked["id"])):
			bad.append("skills:%s: %s" % [str(ranked["id"]), str(p["message"])])
	var skill_count: int = ContentCatalog.skill_entries().size()
	checks["every ranked skill family passes the skill Apply check (%d families) %s" % [skill_count, str(bad)]] = bad.is_empty() and skill_count > 0
	bad = []
	for cls: Dictionary in ContentCatalog.entries("classes"):
		for p: Dictionary in ClassForgeModel.draft_problems(cls, str(cls["id"])):
			bad.append("classes:%s: %s" % [str(cls["id"]), str(p["message"])])
	checks["every class passes the class Apply check %s" % str(bad)] = bad.is_empty()
	bad = []
	if ContentCatalog.items_inherit():
		for k: String in ["items", "item_families"]:
			for e: Dictionary in ContentCatalog.entries(k):
				for p: Dictionary in ItemFormModel.draft_problems(k, e, str(e["id"])):
					bad.append("%s:%s: %s" % [k, str(e["id"]), str(p["message"])])
	checks["every item and family passes the item Apply check %s" % str(bad)] = bad.is_empty()
	var icons_ok: bool = true
	for e: Dictionary in ContentCatalog.entries("items"):
		var icon: String = str(e.get("icon_id", ""))
		icons_ok = icons_ok and (icon.is_empty() or AssetRegistry.has_key("item", icon))
	checks["every item icon_id is a known item icon key"] = icons_ok
	var blocked: Array[String] = []
	for k: String in ContentSchema.KINDS:
		if ContentCatalog.save_blocked(k):
			blocked.append(k)
	var fold_errors: Array = SkillRanks.collapse(ContentCatalog.entries("skills"))["errors"]
	checks["no kind is save-blocked and the skills fold for save %s" % str(blocked)] = blocked.is_empty() and fold_errors.is_empty()
	return TestHelpers.report(TAG, "Shipped data passes every former Forge Apply / Save check", checks)

# The engine's skills, items, classes and encounters are the catalog's.
static func _test_engine_parity() -> bool:
	var volley: SkillDef = SkillCatalog.find("volley")
	var skills_ok: bool = (volley != null and volley.area == SkillDef.Area.PLUS and volley.reach_min == 2 and volley.reach_max == 3
		and SkillCatalog.find("rest").system and not SkillCatalog.find("strike").system
		and SkillCatalog.lowest(SkillDef.Kind.ATTACK).id == "strike" and SkillCatalog.all().size() == ContentCatalog.entries("skills").size())
	var claymore: Equipment = ItemCatalog.create_iron_claymore()
	var stash := Inventory.new()
	ItemCatalog.populate_party_starter_stash(stash)
	var items_ok: bool = (claymore.base_power == 24 and claymore.is_two_handed and claymore.threshold_tiers.size() == 1
		and DictUtil.int_of(claymore.required_stats, "STR") == 14 and ItemCatalog.create_healing_draught().max_stack == 5
		and stash.items.size() == 21)
	var classes_ok: bool = ClassArchetype.create_warrior().get_base_stat("STR") == DictUtil.to_int(DictUtil.dict_of(ContentCatalog.get_entry("classes", "warrior")["base_stats"])["STR"]) and ClassArchetype.create("brawler").make_proficiency().level_of("fist") == 3
	var enc := CatalogEncounter.new("goblin_skirmish")
	enc.build()
	var units_ok: bool = (enc.unit("warrior").tier == "hero" and enc.unit("goblin_archer").attack_range >= 3 and enc.unit("warrior").max_ap == 12
		and enc.unit("goblin_archer").hp_scale == 0.4 and enc.unit("warrior").max_hp > enc.unit("goblin_brawler").max_hp * 2 and enc.profile_of(enc.unit("goblin_brawler")).flee_ratio == 0.25 and enc.total_blocks == 6
		and enc.zone_id == "ruined_courtyard" and enc.class_of(enc.unit("goblin_shaman")) == str(ContentCatalog.get_entry("units", "goblin_shaman").get("art", "")) and enc.unit("warrior").catalog_id == "warrior")
	return TestHelpers.report_ok(TAG, "Engine content comes from the catalog (skills, items, stash, classes, Goblin Skirmish)", skills_ok and items_ok and classes_ok and units_ok,
		"skills %s, items %s, classes %s, units %s" % [skills_ok, items_ok, classes_ok, units_ok])

# Every entry rebuilds into the same entry after a trip through the engine objects.
static func _test_round_trip() -> bool:
	var bad: Array[String] = []
	for e: Dictionary in ContentCatalog.entries("skills"):
		if JSON.stringify(ContentFactory.skill_to_entry(ContentFactory.skill(e))) != JSON.stringify(ContentSchema.normalize("skills", e)):
			bad.append(str(e["id"]))
	for e: Dictionary in ContentCatalog.entries("items"):
		var back: Dictionary = ContentFactory.item_to_entry(ContentFactory.item(e), DictUtil.to_bool(e["starter_stash"]))
		# The engine item carries the resolved numbers only: the family template keys (FORGE1_ITEMS 2.1) are catalog-side.
		var have: Dictionary = ContentSchema.to_storage("items", back)
		var want: Dictionary = ContentSchema.to_storage("items", e)
		for key: String in ["family", "hand", "modifiers", "tiers"]:
			have.erase(key)
			want.erase(key)
		# The catalog keeps the description template; the engine item carries the rendered text (ItemText tokens).
		want["description"] = ItemText.render(str(want.get("description", "")), e)
		if JSON.stringify(have) != JSON.stringify(want):
			bad.append(str(e["id"]))
	for e: Dictionary in ContentCatalog.entries("classes"):
		if JSON.stringify(ContentFactory.class_to_entry(ContentFactory.class_archetype(e))) != JSON.stringify(ContentSchema.normalize("classes", e)):
			bad.append(str(e["id"]))
	return TestHelpers.report_ok(TAG, "Skills, items and classes round-trip through engine objects unchanged", bad.is_empty(), ", ".join(bad))

static func _has_error(errors: Array[String], fragment: String) -> bool:
	for err: String in errors:
		if err.contains(fragment):
			return true
	return false

static func _test_validation() -> bool:
	var checks: Dictionary = {}
	checks["bad id"] = _has_error(ContentCatalog.validate_entry("skills", {"id": "Bad Id", "name": "X"}), "snake_case")
	checks["duplicate id"] = _has_error(ContentCatalog.validate_entry("skills", {"id": "strike", "name": "Strike 2"}), "already used")
	checks["engine-only kind"] = _has_error(ContentCatalog.validate_entry("skills", {"id": "warp", "name": "Warp", "kind": "MOVE"}), "engine-only")
	checks["reach band"] = _has_error(ContentCatalog.validate_entry("skills", {"id": "lob", "name": "Lob", "reach_min": 3, "reach_max": 2}), "Reach min")
	if ContentCatalog.items_inherit():
		# Families: a shield's slot follows its hand, and a shield family has only the off hand.
		var shield_family: String = ""
		for f: Dictionary in ContentCatalog.entries("item_families"):
			if ItemFamilies.item_type_of(f) == "SHIELD":
				shield_family = str(f["id"])
		checks["shield slot"] = _has_error(ContentCatalog.validate_entry("items", {"id": "odd_shield", "name": "Odd", "family": shield_family, "hand": ItemFamilies.HAND_MAIN_1H, "modifiers": [], "tiers": []}), "not allowed by family")
	else:
		checks["shield slot"] = _has_error(ContentCatalog.validate_entry("items", {"id": "odd_shield", "name": "Odd", "item_type": "SHIELD", "slot": "MAIN_HAND"}), "OFF_HAND")
	checks["unknown ref"] = _has_error(ContentCatalog.validate_entry("units", {"id": "ghost", "name": "Ghost", "main_hand": "nope"}), "unknown item")
	checks["wrong slot"] = _has_error(ContentCatalog.validate_entry("units", {"id": "ghost", "name": "Ghost", "main_hand": "shd_kite"}), "can't be")
	checks["2H + shield"] = _has_error(ContentCatalog.validate_entry("units", {"id": "ghost", "name": "Ghost", "main_hand": "wpn_claymore", "off_hand": "shd_kite"}), "two-handed")
	checks["derived needs class"] = _has_error(ContentCatalog.validate_entry("units", {"id": "ghost", "name": "Ghost", "stat_mode": "derived"}), "need a class")
	checks["class budget"] = _has_error(ContentCatalog.validate_entry("classes", {"id": "fat", "name": "Fat", "base_stats": {"STR": 18, "DEX": 18, "INT": 18, "CONS": 18, "PERC": 18, "KNOW": 18}}), "must total")
	checks["class growth"] = _has_error(ContentCatalog.validate_entry("classes", {"id": "fast", "name": "Fast", "growth": {"STR": 3, "DEX": 3, "INT": 3}}), "gains exactly")
	checks["class proficiency gear"] = _has_error(ContentCatalog.validate_entry("classes", {"id": "odd", "name": "Odd", "proficiency": {"banjo": {"level": 1}}}), "unknown gear type")
	checks["class path: AP level no longer gates a step"] = not _has_error(ContentCatalog.validate_entry("classes", {"id": "odd", "name": "Odd", "skill_path": {"hamstring": 1, "riposte": 1}}), "too advanced")
	checks["class path: a higher tier row is refused"] = _has_error(ContentCatalog.validate_entry("classes", {"id": "odd", "name": "Odd", "skill_path": {"hamstring": 1, "riposte_2": 3}}), "higher tier")
	# One-release converter (CLASS1, off since CLASS1-M): when switched on, a repeated family collapses to one key at its lowest level.
	var conv_saved: bool = ClassUnitFields.list_path_converter
	ClassUnitFields.list_path_converter = true
	var listed: Dictionary = ContentSchema.normalize("classes", {"id": "odd", "name": "Odd", "skill_path": [{"level": 1, "skill": "hamstring"}, {"level": 3, "skill": "hamstring"}]})
	ClassUnitFields.list_path_converter = conv_saved
	checks["class path: the old list shape converts to one key per family"] = DictUtil.dict_of(listed["skill_path"]) == {"hamstring": 1}
	checks["skills: tier 1 can't carry an unlock offset"] = _has_error(ContentCatalog.validate_entry("skills", {"id": "odd_t1", "name": "Odd", "kind": "ATTACK", "unlock_offset": 2}), "tier 1")
	checks["skills: basic skills can't be advanced"] = _has_error(ContentCatalog.validate_entry("skills", {"id": "odd_basic", "name": "Odd", "kind": "ATTACK", "basic": true, "grade": "advanced"}), "advanced")
	checks["passives: unlock effect validates"] = Passives.validate_effects([{"type": "unlock", "family": "riposte", "levels": 2}, {"type": "unlock", "group": "heal", "levels": 1}], func(id: String) -> bool: return ContentCatalog.has("skills", id)).is_empty() and not Passives.validate_effects([{"type": "unlock", "family": "riposte", "levels": 3}], func(_id: String) -> bool: return true).is_empty() and not Passives.validate_effects([{"type": "unlock", "family": "riposte", "group": "heal", "levels": 1}], func(_id: String) -> bool: return true).is_empty() and not Passives.validate_effects([{"type": "unlock", "family": "nope", "levels": 1}], func(id: String) -> bool: return ContentCatalog.has("skills", id)).is_empty() and not Passives.validate_effects([{"type": "unlock", "group": "melee", "levels": 1}], func(_id: String) -> bool: return true).is_empty()
	checks["passives: unlock is not allowed on items"] = not Passives.validate_effects([{"type": "unlock", "family": "riposte", "levels": 1}], func(_id: String) -> bool: return true, true, true).is_empty()
	checks["class gear training"] = _has_error(ContentCatalog.validate_entry("classes", {"id": "odd", "name": "Odd", "proficiency": {"sword": {"level": 1}}, "main_hand": "wpn_shortbow"}), "no training")
	checks["spend pool"] = _has_error(ContentCatalog.validate_entry("units", {"id": "ghost", "name": "Ghost", "class_id": "warrior", "level": 1, "spend": {"STR": 9}}), "spends exactly 4")
	checks["passive effects"] = _has_error(ContentCatalog.validate_entry("passives", {"id": "bad_pass", "name": "Bad", "effects": [{"type": "stat", "stat": "LUCK", "value": 2}]}), "unknown stat")
	checks["empty encounter"] = _has_error(ContentCatalog.validate_entry("encounters", {"id": "void", "name": "Void"}), "at least one enemy")
	checks["side filter"] = _has_error(ContentCatalog.validate_entry("encounters", {"id": "mix", "name": "Mix", "party": ["goblin_archer"], "enemies": ["warrior"]}), "can't be used here")
	checks["system delete refused (origin system)"] = ContentCatalog.remove("items", "wpn_shortbow").contains("system asset")
	ContentCatalog.begin_system_edit(true)
	checks["in-use delete refused"] = ContentCatalog.remove("items", "wpn_shortbow").contains("still used")
	ContentCatalog.end_system_edit()
	checks["system delete refused"] = ContentCatalog.remove("skills", "move").contains("system")
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	return TestHelpers.report_ok(TAG, "Validation catches bad ids, kinds, reach, slots, refs, rosters; refuses unsafe deletes", failed.is_empty(), ", ".join(failed))

# ------------------------------------------------------------------------------
# Authoring (sandbox)
# ------------------------------------------------------------------------------

static func _test_authoring() -> bool:
	var errors: Array[String] = []
	errors.append_array(ContentCatalog.upsert("skills", {"id": "test_cleave", "name": "Cleaving Blow", "kind": "ATTACK", "level": 2, "blocks": 2,
		"power_mult": 1.3, "status_effect": "stagger", "status_potency": 30, "description": "Forge test skill."}))
	errors.append_array(ContentCatalog.upsert("items", {"id": "test_glaive", "name": "Test Glaive", "item_type": "WEAPON", "slot": "MAIN_HAND",
		"weapon_type": "sword", "base_power": 30, "two_handed": true, "scaling_stat": "STR", "required_stats": {"STR": 10}, "rarity": "RARE", "weight": 5.0}))
	errors.append_array(ContentCatalog.upsert("classes", {"id": "test_paladin", "name": "Test Paladin", "art": "warrior",
		"base_stats": {"STR": 14, "DEX": 8, "INT": 8, "CONS": 15, "PERC": 9, "KNOW": 8}, "growth": {"STR": 2, "CONS": 2, "PERC": 1},
		"proficiency": {"sword": {"level": 2, "scaling": 1.5}, "plate": {"level": 2, "scaling": 1.5}}, "skill_path": {"hamstring": 1}}))
	errors.append_array(ContentCatalog.upsert("units", {"id": "test_knight", "name": "Test Knight", "side": "party", "art": "warrior",
		"class_id": "test_paladin", "level": 3, "stat_mode": "derived", "skill_cap": 2, "extra_skills": ["test_cleave"], "ai_role": "AGGRESSOR",
		"flee_ratio": 0.0, "main_hand": "test_glaive", "body": "arm_plate"}))
	errors.append_array(ContentCatalog.upsert("units", {"id": "test_raider", "name": "Test Raider", "side": "enemy", "art": "brawler",
		"stat_mode": "fixed", "hp": 30, "speed": 12, "attack_range": 1, "power": 8, "ai_role": "AGGRESSOR", "status_resist": {"stagger": 100}}))
	errors.append_array(ContentCatalog.upsert("encounters", {"id": "test_trial", "name": "Test Trial", "zone": "stone_arena",
		"party": ["test_knight"], "enemies": ["test_raider", "test_raider"], "max_rounds": 8, "total_blocks": 6, "unit_ap": 12, "ap_regen": 4}))
	var saved: bool = ContentCatalog.save_all() == OK and FileAccess.file_exists(ContentCatalog.path_for("units"))
	ContentCatalog.reload_from_disk()
	var reloaded: bool = (ContentCatalog.has("skills", "test_cleave") and ContentCatalog.has("encounters", "test_trial")
		and SkillCatalog.find("test_cleave") != null and is_equal_approx(SkillCatalog.find("test_cleave").power_mult, 1.3)
		and ContentCatalog.validate_all().is_empty())
	# Renaming an id rewrites references to it.
	var raider: Dictionary = ContentCatalog.get_entry("units", "test_raider")
	raider["id"] = "test_marauder"
	raider["name"] = "Test Marauder"
	errors.append_array(ContentCatalog.upsert("units", raider, "test_raider"))
	var roster: Array = ContentCatalog.get_entry("encounters", "test_trial")["enemies"]
	var renamed: bool = roster.size() == 2 and str(roster[0]) == "test_marauder" and not ContentCatalog.has("units", "test_raider")
	# Templates: duplicating keeps every field under a fresh id.
	var copy_id: String = ContentCatalog.duplicate_entry("units", "goblin_archer")
	var copy: Dictionary = ContentCatalog.get_entry("units", copy_id)
	var templated: bool = copy_id == "goblin_archer_copy" and str(copy["class_id"]) == "ranger" and str(copy["tier"]) == "minion"
	ContentCatalog.remove("units", copy_id)
	ContentCatalog.save_all()
	return TestHelpers.report_ok(TAG, "Author skill/item/class/units/encounter, save to disk, reload, rename refs, duplicate as template",
		errors.is_empty() and saved and reloaded and renamed and templated, "errors %s, saved %s, reloaded %s, renamed %s, template %s" % [str(errors), saved, reloaded, renamed, templated])

# Authored units bring their class, gear, skill list and resistances into planning.
static func _test_authored_units_in_planning() -> bool:
	var enc := CatalogEncounter.new("test_trial")
	var session := CombatSession.new(enc)
	session.start()
	var knight: Combatant = enc.unit("test_knight")
	var raider_a: Combatant = enc.unit("test_marauder", 0)
	var raider_b: Combatant = enc.unit("test_marauder", 1)
	var built_ok: bool = (knight != null and raider_a != null and raider_b != null and raider_a.name == "Test Marauder" and raider_b.name == "Test Marauder 2"
		and knight.inventory.get_main_weapon() != null and knight.inventory.get_main_weapon().id == "test_glaive"
		and knight.stats.get_base_stat("CONS") > 16 and knight.max_hp == knight.hp and knight.max_ap == 12
		and DictUtil.int_of(raider_a.status_resist, "stagger") == 100 and enc.grid.has_combatant(raider_b))
	var attack_names: Array[String] = []
	for opt: PlanOption in session.plan.target_options(knight, raider_a):
		attack_names.append(opt.step.skill.id)
	var self_names: Array[String] = []
	for opt: PlanOption in session.plan.target_options(knight, knight):
		self_names.append(opt.step.skill.id)
	var skills_ok: bool = attack_names.has("test_cleave") and attack_names.has("strike") and not attack_names.has("quick_stab") and self_names.has("rest") and self_names.has("guard") and not self_names.has("focus")
	session.dispose()
	return TestHelpers.report_ok(TAG, "Authored units: class-derived vitals, gear, numbered duplicates, whitelisted skills, resistances", built_ok and skills_ok,
		"built %s, attacks %s, self %s" % [built_ok, str(attack_names), str(self_names)])

# A catalog encounter plays to a finish through the full engine (AI on both sides).
static func _test_full_integration() -> bool:
	var enc := CatalogEncounter.new("test_trial")
	var session := CombatSession.new(enc)
	session.start()
	var rounds: int = 0
	for round_num: int in range(enc.max_rounds):
		session.plan_apply_preset_all("balanced")
		if not session.confirm_plan():
			break
		_resolve(session)
		rounds += 1
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var used_cleave: bool = false
	var foes_hurt: bool = false
	for e: ActionLogEntry in session.log_entries:
		if e.skill_name == "Cleaving Blow":
			used_cleave = true
	for foe: Combatant in enc.enemies:
		if foe.hp < foe.max_hp or not foe.is_active():
			foes_hurt = true
	var finished: bool = session.combat_over or rounds == enc.max_rounds
	var ok: bool = rounds > 0 and finished and used_cleave and foes_hurt
	session.dispose()
	# The shipped Veterans' Drill (class-derived dojo heroes) also plays out.
	var drill := CombatSession.new(CatalogEncounter.new("veterans_drill"))
	drill.start()
	drill.plan_apply_preset_all("balanced")
	var drill_ok: bool = drill.confirm_plan()
	_resolve(drill)
	drill_ok = drill_ok and drill.phase == CombatSession.Phase.REPLAY and drill.enemies.size() == 3
	drill.dispose()
	return TestHelpers.report_ok(TAG, "Full integration: authored encounter plays to the end in the combat engine with the new skill", ok and drill_ok,
		"rounds %d, over %s, cleave %s, foes hurt %s, drill %s" % [rounds, session.combat_over, used_cleave, foes_hurt, drill_ok])

static func _resolve(session: CombatSession) -> void:
	for i: int in range(600):
		if session.phase == CombatSession.Phase.REPLAY:
			return
		session.step_next()

# ------------------------------------------------------------------------------
# Content Forge viewer (TOOLS-RO: read-only; runs on the sandbox fork with the authored test content)
# ------------------------------------------------------------------------------

static func _test_forge_ui(tree_parent: Node) -> bool:
	if tree_parent == null:
		return TestHelpers.report_ok(TAG, "Content Forge viewer (needs a scene tree)", true, "skipped")
	var host: Node = (load("res://scenes/content_forge.tscn") as PackedScene).instantiate()
	tree_parent.add_child(host)
	var forge: ContentForge = host.get("forge")
	var checks: Dictionary = {}
	checks["opens on skills"] = forge.kind == "skills" and not forge.selected_id.is_empty()
	var before: String = JSON.stringify(ContentCatalog.skill_entries())
	var dirty_before: bool = ContentCatalog.is_dirty()

	# Skill tab: the form shows the ranked entry; the ranks panel has no edit actions.
	var ranked_head: String = ""
	for ranked: Dictionary in ContentCatalog.skill_entries():
		if SkillRanks.rank_count(ranked) > 1 and ranked_head.is_empty():
			ranked_head = str(ranked["id"])
	forge.select(ranked_head)
	checks["skills: the form shows the ranked entry"] = not ranked_head.is_empty() and SkillRanks.rank_count(forge.form.draft) == SkillRanks.rank_count(ContentCatalog.skill_entry(ranked_head))
	var ranks_panel: Node = forge.form.find_child("RanksPanel", true, false)
	checks["skills: the ranks panel shows one resolved panel per rank"] = ranks_panel != null and ranks_panel.find_children("Resolved_*", "", true, false).size() == SkillRanks.rank_count(forge.form.draft)
	var live: Array[String] = TestHelpers.live_controls(forge.form)
	checks["skills: no control of the form can edit or act (%s)" % ", ".join(live)] = live.is_empty()
	var inputs: Array[Node] = []
	for cls: String in ["LineEdit", "TextEdit", "SpinBox", "OptionButton", "CheckBox"]:
		inputs.append_array(forge.form.find_children("*", cls, true, false))
	checks["skills: the form is a display, no input widgets (%d)" % inputs.size()] = inputs.is_empty()
	var name_field: Label = forge.form.field_control("name") as Label
	checks["skills: field values show as labels"] = name_field != null and name_field.text == str(forge.form.draft["name"])

	# Skill list (FORGE-VIEW): names only, engine rows always listed, basic grade grouping, stance filter, no problem / rank UI.
	var texts: Array[String] = forge.list_texts()
	var listed: Array[String] = forge.list_ids()
	var names_only: bool = texts.size() == listed.size() and not listed.is_empty()
	for i: int in range(mini(texts.size(), listed.size())):
		names_only = names_only and texts[i] == str(SkillRanks.effective(ContentCatalog.skill_entry(listed[i]), 1)["name"])
	checks["skills: list rows show only the skill name"] = names_only
	var engine_heads: Array[String] = []
	var basic_heads: Array[String] = []
	for ranked: Dictionary in ContentCatalog.skill_entries():
		var base: Dictionary = SkillRanks.effective(ranked, 1)
		if DictUtil.to_bool(base.get("system", false)):
			engine_heads.append(str(ranked["id"]))
		if DictUtil.to_bool(base.get("system", false)) or DictUtil.to_bool(base.get("basic", false)):
			basic_heads.append(str(ranked["id"]))
	checks["skills: engine rows are always listed"] = not engine_heads.is_empty() and engine_heads.all(func(h: String) -> bool: return listed.has(h)) and listed.size() == ContentCatalog.skill_entries().size()
	forge.set_skill_filter("grade", SkillFilter.BASIC_GRADE)
	var basic_listed: Array[String] = forge.list_ids()
	basic_listed.sort()
	basic_heads.sort()
	checks["skills: the basic grade lists the basic and engine skills"] = basic_listed == basic_heads
	forge.set_skill_filter("grade", "any")
	var bar: Node = forge.find_child("SkillBar", true, false)
	var gone: Array[String] = []
	for old_name: String in ["Check_has_problems", "Check_show_system", "RankFilter", "Tri_basic"]:
		if bar != null and bar.find_child(old_name, true, false) != null:
			gone.append(old_name)
	var stance_opt: OptionButton = bar.find_child("StanceFilter", true, false) as OptionButton if bar != null else null
	var sort_opt: OptionButton = bar.find_child("SortBy", true, false) as OptionButton if bar != null else null
	checks["skills: no problems, engine-rows, rank or basic controls (%s)" % ", ".join(gone)] = bar != null and gone.is_empty()
	checks["skills: the stance filter offers any stance + each catalog stance"] = stance_opt != null and stance_opt.item_count == 1 + SkillFilter.stance_options(ContentCatalog.skill_entries()).size()
	checks["skills: the sorts are name, kind and AP"] = sort_opt != null and sort_opt.item_count == SkillFilter.SORTS.size()
	checks["skills: no problems box"] = not forge.has_method("problems_text")

	# Skill preview (FORGE-VIEW): the in-game elements per rank, cycled by the rank bar.
	forge.select(ranked_head)
	var show: SkillShowcase = forge.showcase()
	var ranked_entry: Dictionary = ContentCatalog.skill_entry(ranked_head)
	var rank_buttons: Array[Node] = show.find_children("Rank_*", "Button", true, false)
	checks["preview: one rank button per rank"] = rank_buttons.size() == SkillRanks.rank_count(ranked_entry)
	var rank_ok: bool = true
	for r: int in range(1, SkillRanks.rank_count(ranked_entry) + 1):
		forge.set_preview_rank(r)
		var rank_name: String = str(SkillRanks.resolve(ranked_entry, r)["name"])
		var shown: Array[String] = show.shown_texts()
		var menu: Node = show.find_child("ActionMenu", true, false)
		var in_menu: bool = menu != null and menu.find_children("*", "Button", true, false).any(func(b: Node) -> bool: return (b as Button).text.contains(rank_name))
		rank_ok = rank_ok and show.rank == r and in_menu and shown.has(rank_name) and show.find_child("LogDock", true, false) != null and show.find_child("LoadoutLine", true, false) != null
	checks["preview: each rank shows its name in the action menu, log and loadout elements"] = rank_ok
	show.cycle(1)
	checks["preview: cycling wraps from the top rank to rank 1"] = show.rank == 1
	var sims_ok: bool = true
	for ranked: Dictionary in ContentCatalog.skill_entries():
		for r: int in range(1, SkillRanks.rank_count(ranked) + 1):
			var sim: Dictionary = SkillShowcase.simulate(SkillRanks.resolve(ranked, r))
			var option: PlanOption = sim["option"]
			sims_ok = sims_ok and (option.valid or not option.reason.is_empty()) and (not option.valid or sim["log"] is ActionLogEntry)
	checks["preview: every catalog rank plans (or says why not) and a valid plan resolves to a log cell"] = sims_ok

	# No edit or maintenance action anywhere in the Forge (only browsing, filters, Reload, Validate All and Playtest).
	var actions: Array[String] = []
	for node: Node in forge.find_children("*", "Button", true, false):
		var text: String = (node as Button).text
		for word: String in ["Save", "Apply", "Revert", "New", "Duplicate", "Delete", "Promote", "Demote"]:
			if text.contains(word):
				actions.append(text)
	checks["no Save, Apply, Revert, New, Duplicate, Delete or Promote button (%s)" % ", ".join(actions)] = actions.is_empty()
	checks["no Forge method writes (save_all, apply, new_entry, duplicate_selected, delete_selected)"] = not ["save_all", "apply", "new_entry", "duplicate_selected", "delete_selected", "toggle_origin", "revert"].any(func(m: String) -> bool: return forge.has_method(m))

	# Items: conditional sections follow the family's item type (weapon vs consumable, picked by rule).
	forge.show_kind("items")
	var weapon_id: String = ""
	var consumable_id: String = ""
	for item: Dictionary in ContentCatalog.entries("items"):
		var type: String = str(item.get("item_type", ""))
		if type == "WEAPON" and weapon_id.is_empty():
			weapon_id = str(item["id"])
		elif type == "CONSUMABLE" and consumable_id.is_empty():
			consumable_id = str(item["id"])
	forge.select(weapon_id)
	checks["weapon fields shown"] = forge.form.panel_control("base") != null and forge.form.field_control("modifiers") != null and forge.form.field_control("heal") == null
	forge.select(consumable_id)
	checks["consumable fields shown"] = forge.form.field_control("heal") != null and forge.form.field_control("modifiers") == null and forge.form.panel_control("base") == null
	checks["the item list has one row per item"] = forge.list_ids().size() == ContentCatalog.entries("items").size()

	# Units: playtest an authored unit in a sandbox fight in the real arena.
	forge.show_kind("units")
	forge.select("test_knight")
	forge.playtest()
	var playing: bool = DictUtil.to_bool(host.call("is_playtesting"))
	var arena: Node = host.get("arena")
	var arena_session: CombatSession = arena.get("session") if arena != null else null
	checks["playtest runs the arena"] = playing and arena_session != null and arena_session.allies.size() == 1 and arena_session.allies[0].name == "Test Knight"
	host.call("close_playtest")
	checks["back to forge"] = not DictUtil.to_bool(host.call("is_playtesting"))

	# Encounters list, preview, validate all; browsing never changes the catalog.
	forge.show_kind("encounters")
	forge.select("test_trial")
	checks["encounter preview deploys"] = forge.selected_id == "test_trial" and ForgePreview.describe("encounters", ContentCatalog.get_entry("encounters", "test_trial")).contains("Test Marauder 2")
	checks["Validate All reports ContentCatalog.validate_all() on the status line"] = JSON.stringify(forge.validate_all()) == JSON.stringify(ContentCatalog.validate_all()) and not forge.status_text().is_empty()
	checks["browsing never dirties or changes the catalog"] = ContentCatalog.is_dirty() == dirty_before and DictUtil.to_bool(host.call("has_unsaved_changes")) == dirty_before and JSON.stringify(ContentCatalog.skill_entries()) == before
	host.queue_free()
	return TestHelpers.report(TAG, "Content Forge viewer: browse, ranked skills, skill list and preview, conditional item sections, no edit actions, playtest, validate", checks)
