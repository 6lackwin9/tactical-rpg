class_name ItemFamiliesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Item families suite (FORGE1_ITEMS 1, 2.5, 8.1; ITEM1-L commit 1)
# Runs on a sandbox fork whose items use families (TestHelpers.fork_migrated: the ITEM1 seed migrates the fork while the
# shipped data is still flat). Family record validation, the gear-token invariant, the engine rule lists against the
# code constants, remove / retype / rename refusals, origin locks, storage versions, and strict hand typing (8.1).
# Expectations come from the rules and the catalog, never from fixed counts.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("item_families")
const TAG: String = "ItemFamilies"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING ITEM FAMILIES TEST (ITEM1-L)            ===")
	print("=======================================================")
	var report: Dictionary = TestHelpers.fork_migrated(SANDBOX)
	var ok: bool = TestHelpers.report_ok(TAG, "Sandbox fork uses item families (migration seed on the fork when the shipped data is flat)",
		ContentCatalog.items_inherit() and (report.is_empty() or DictUtil.to_bool(report.get("ok", false))), str(report.get("families_created", "shipped families")))
	if not ContentCatalog.items_inherit():
		TestHelpers.cleanup(SANDBOX)
		return false
	ok = _test_catalog_invariants() and ok
	ok = _test_engine_rules_match_code() and ok
	ok = _test_record_validation() and ok
	ok = _test_refusals_and_origin() and ok
	ok = _test_strict_hands() and ok
	ok = _test_storage() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _ids_with_rule(rule: String) -> Array[String]:
	var out: Array[String] = []
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.has_rule(f, rule):
			out.append(str(f["id"]))
	out.sort()
	return out

static func _only(list: Array[String], value: String) -> bool:
	return list.size() == 1 and list[0] == value

# The first family of `item_type` (chosen by rule).
static func _family_of_type(item_type: String) -> Dictionary:
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.item_type_of(f) == item_type:
			return f
	return {}

static func _sorted(list: Array[String]) -> Array[String]:
	var out: Array[String] = list.duplicate()
	out.sort()
	return out

# A main-category weapon family that has items (chosen by rule).
static func _main_family_with_items() -> Dictionary:
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.item_type_of(f) == "WEAPON" and ItemFamilies.category(f) == "main" and not ContentCatalog.items_of_family(str(f["id"])).is_empty():
			return f
	return {}

# A family without items (chosen by rule).
static func _family_without_items() -> Dictionary:
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ContentCatalog.items_of_family(str(f["id"])).is_empty():
			return f
	return {}

# A minimal valid user ACCESSORY family (no gear token needed).
static func _accessory_family(id: String) -> Dictionary:
	return {"id": id, "name": "Rings", "item_type": "ACCESSORY", "slots": ["ACCESSORY_1", "ACCESSORY_2"],
		"base": {"none": {"power": 0, "range": 1, "block_cost": 1, "burden": 0, "mitigation": 0}}, "bands": {},
		"adjustable": ["burden"], "modifier_kinds": ["stat_bonus", "requirement"], "max_modifiers": 4, "max_tiers": 0}

# ------------------------------------------------------------------------------
# 1. Invariants of the migrated catalog
# ------------------------------------------------------------------------------

static func _test_catalog_invariants() -> bool:
	var checks: Dictionary = {}
	var families: Array[Dictionary] = ContentCatalog.entries("item_families")
	var ids: Array[String] = ContentCatalog.ids("item_families")
	var tokens_ok: bool = true
	for token: String in Proficiency.gear_types():
		tokens_ok = tokens_ok and ids.has(token)
	checks["every gear token has its family"] = tokens_ok and ItemValidator.family_set_errors(families).is_empty()
	var gear_named: bool = true
	var all_system: bool = true
	for f: Dictionary in families:
		if ItemFamilies.is_gear_type(ItemFamilies.item_type_of(f)):
			gear_named = gear_named and Proficiency.gear_types().has(str(f["id"]))
		all_system = all_system and ContentCatalog.is_system_origin(f)
	checks["every WEAPON / SHIELD / ARMOR family is named after a gear token"] = gear_named
	checks["shipped (migrated) families are origin system"] = all_system
	var items_ok: bool = true
	var category_ok: bool = true
	for row: Dictionary in ContentCatalog.entries("items"):
		var fam: Dictionary = ContentCatalog.family_of(row)
		items_ok = items_ok and not fam.is_empty() and str(row["item_type"]) == ItemFamilies.item_type_of(fam)
		category_ok = category_ok and str(row["off_hand_category"]) == ItemFamilies.off_hand_category(fam)
	checks["every item names a known family and takes its item type"] = items_ok
	checks["the off-hand category is derived from the family (shield, orb, tome; else empty)"] = category_ok
	checks["validate_all is empty"] = ContentCatalog.validate_all().is_empty()
	var one_category: bool = true
	for f: Dictionary in families:
		if ItemFamilies.HAND_TYPES.has(ItemFamilies.item_type_of(f)):
			one_category = one_category and not ItemFamilies.category(f).is_empty()
	checks["every handed family has exactly one hand category"] = one_category
	return TestHelpers.report(TAG, "Catalog invariants (%d families)" % families.size(), checks)

# ------------------------------------------------------------------------------
# 2. Engine rules equal the code constants (commit 1 keeps the constants as the engine's source)
# ------------------------------------------------------------------------------

static func _test_engine_rules_match_code() -> bool:
	var checks: Dictionary = {}
	checks["focus = Equipment.FOCUS_TYPES"] = _ids_with_rule(ItemFamilies.RULE_FOCUS) == _sorted(Equipment.FOCUS_TYPES)
	var ranged: Array[String] = []
	for t: String in Equipment.ALWAYS_RANGED_TYPES:
		if Proficiency.gear_types().has(t):
			ranged.append(t)
	checks["always_ranged = Equipment.ALWAYS_RANGED_TYPES (gear tokens)"] = _ids_with_rule(ItemFamilies.RULE_ALWAYS_RANGED) == _sorted(ranged)
	checks["spell_source = SkillDef.SPELL_SOURCE_TYPES"] = _ids_with_rule(ItemFamilies.RULE_SPELL_SOURCE) == _sorted(SkillDef.SPELL_SOURCE_TYPES)
	var shields: Array[String] = []
	for f: Dictionary in ContentCatalog.entries("item_families"):
		if ItemFamilies.item_type_of(f) == "SHIELD":
			shields.append(str(f["id"]))
	checks["shield = the SHIELD families"] = _ids_with_rule(ItemFamilies.RULE_SHIELD) == _sorted(shields)
	var focus_off: bool = true
	for id: String in _ids_with_rule(ItemFamilies.RULE_FOCUS):
		focus_off = focus_off and ItemFamilies.category(ContentCatalog.get_entry("item_families", id)) == "off"
	checks["focus families are off-hand only"] = focus_off
	return TestHelpers.report(TAG, "Engine rule lists equal the code constants", checks)

# ------------------------------------------------------------------------------
# 3. Family record validation (test-only fixtures)
# ------------------------------------------------------------------------------

static func _family_errors(entry: Dictionary) -> Array[String]:
	return ContentCatalog.validate_entry("item_families", entry)

static func _test_record_validation() -> bool:
	var checks: Dictionary = {}
	var main: Dictionary = _main_family_with_items()
	var token: String = str(main.get("id", ""))
	var mixed: Dictionary = main.duplicate(true)
	mixed["hands"] = [ItemFamilies.HAND_MAIN_1H, ItemFamilies.HAND_OFF]
	checks["mixing main and off hands is an error"] = TestHelpers.any_contains(_family_errors(mixed), "One hand category")
	var shield: Dictionary = _family_of_type("SHIELD")
	var shield_main: Dictionary = shield.duplicate(true)
	shield_main["hands"] = [ItemFamilies.HAND_MAIN_1H]
	checks["a SHIELD family is off-hand only"] = not shield.is_empty() and TestHelpers.any_contains(_family_errors(shield_main), "off-hand only")
	var focus_main: Dictionary = main.duplicate(true)
	focus_main["engine_rules"] = [ItemFamilies.RULE_FOCUS]
	checks["focus on a main-hand family is an error"] = TestHelpers.any_contains(_family_errors(focus_main), "focus needs")
	var unknown_rule: Dictionary = main.duplicate(true)
	unknown_rule["engine_rules"] = ["t_no_such_rule"]
	checks["engine rules are a closed list"] = TestHelpers.any_contains(_family_errors(unknown_rule), "closed list")
	var know: Dictionary = main.duplicate(true)
	know["scaling_stats"] = ["KNOW"]
	checks["weapons never scale from KNOW"] = TestHelpers.any_contains(_family_errors(know), "scale from")
	var stranger: Dictionary = main.duplicate(true)
	stranger["id"] = "t_fam_blade"
	checks["a WEAPON family must be a gear token"] = TestHelpers.any_contains(_family_errors(stranger), "gear token")
	var bad_base: Dictionary = main.duplicate(true)
	var base: Dictionary = DictUtil.dict_of(bad_base["base"])
	var first_hand: String = ItemFamilies.hands_of(main)[0]
	DictUtil.dict_of(base[first_hand])["power"] = 999
	checks["a base number outside the schema range is an error"] = TestHelpers.any_contains(_family_errors(bad_base), "outside 0-200")
	var missing_base: Dictionary = main.duplicate(true)
	missing_base["base"] = {}
	checks["every hand needs its base row"] = TestHelpers.any_contains(_family_errors(missing_base), "missing")
	var bad_band: Dictionary = main.duplicate(true)
	bad_band["bands"] = {first_hand: {"power": {"min": 9, "max": 3}}}
	checks["a band with min above max is an error"] = TestHelpers.any_contains(_family_errors(bad_band), "above max")
	var kinds: Dictionary = _accessory_family("t_fam_kinds")
	kinds["modifier_kinds"] = ["t_no_kind"]
	checks["modifier kinds are ItemModifiers.KINDS"] = TestHelpers.any_contains(_family_errors(kinds), "Unknown modifier kind")
	var adjust_power: Dictionary = _accessory_family("t_fam_adj")
	adjust_power["adjustable"] = ["power"]
	checks["adjustable numbers follow the item type (no power on accessories)"] = TestHelpers.any_contains(_family_errors(adjust_power), "not a number")
	checks["a valid user accessory family validates"] = _family_errors(_accessory_family("t_fam_ok")).is_empty()
	checks["the fixtures picked a main family by rule"] = not token.is_empty()
	return TestHelpers.report(TAG, "Family record validation", checks)

# ------------------------------------------------------------------------------
# 4. Remove / retype / rename refusals, origin locks
# ------------------------------------------------------------------------------

static func _test_refusals_and_origin() -> bool:
	var checks: Dictionary = {}
	var main: Dictionary = _main_family_with_items()
	var id: String = str(main["id"])
	var users: Array[Dictionary] = ContentCatalog.items_of_family(id)
	checks["a family with items is never removed"] = not ContentCatalog.remove("item_families", id).is_empty()
	var renamed: Dictionary = main.duplicate(true)
	renamed["name"] = "Renamed " + str(main["name"])
	checks["a free field (name) of a system family can change"] = ContentCatalog.upsert("item_families", renamed).is_empty()
	var key_edit: Dictionary = main.duplicate(true)
	key_edit["hands"] = [ItemFamilies.HAND_OFF]
	checks["a key field (hands) of a system family is locked"] = TestHelpers.any_contains(ContentCatalog.upsert("item_families", key_edit), "key field")
	ContentCatalog.begin_system_edit()
	var breaking: Array[String] = ContentCatalog.upsert("item_families", key_edit)
	ContentCatalog.end_system_edit()
	var named: bool = TestHelpers.any_contains(breaking, "breaks") and TestHelpers.any_contains(breaking, str(users[0]["id"]))
	checks["changing hands under the items is refused with the failing ids (even in the seed scope)"] = named
	var retype: Dictionary = main.duplicate(true)
	retype["item_type"] = "ARMOR"
	ContentCatalog.begin_system_edit()
	var retype_errors: Array[String] = ContentCatalog.upsert("item_families", retype)
	ContentCatalog.end_system_edit()
	checks["changing item_type under the items is refused"] = not retype_errors.is_empty()
	ContentCatalog.begin_system_edit()
	var moved: Dictionary = main.duplicate(true)
	moved["id"] = id + "_moved"
	var rename_errors: Array[String] = ContentCatalog.upsert("item_families", moved, id)
	ContentCatalog.end_system_edit()
	checks["the id of a family with items is read-only"] = TestHelpers.any_contains(rename_errors, "read-only")
	checks["the family is unchanged after the refusals"] = ItemFamilies.hands_of(ContentCatalog.get_entry("item_families", id)) == ItemFamilies.hands_of(main) and ContentCatalog.has("item_families", id)
	var empty: Dictionary = _family_without_items()
	checks["a system family without items is still not deletable outside SAFE DELETE"] = not empty.is_empty() and ContentCatalog.remove("item_families", str(empty["id"])).contains("system asset")
	var user_errors: Array[String] = ContentCatalog.upsert("item_families", _accessory_family("t_fam_user"))
	checks["a new family is a user asset"] = user_errors.is_empty() and str(ContentCatalog.get_entry("item_families", "t_fam_user")["origin"]) == ContentSchema.ORIGIN_USER
	checks["a user family without items is deletable"] = ContentCatalog.remove("item_families", "t_fam_user").is_empty()
	return TestHelpers.report(TAG, "Remove, retype, rename refusals and origin locks", checks)

# ------------------------------------------------------------------------------
# 5. Strict hand typing (8.1)
# ------------------------------------------------------------------------------

static func _test_strict_hands() -> bool:
	var checks: Dictionary = {}
	var main: Dictionary = _main_family_with_items()
	var saved: bool = Equipment.allow_plain_dual_wield
	var main_1h: Array[String] = ItemFamilies.slots_for(main, ItemFamilies.HAND_MAIN_1H, false)
	var dual: Array[String] = ItemFamilies.slots_for(main, ItemFamilies.HAND_MAIN_1H, true)
	checks["main_1h fits MAIN_HAND only; the legacy flag adds OFF_HAND"] = _only(main_1h, "MAIN_HAND") and dual.has("OFF_HAND") and dual.has("MAIN_HAND")
	checks["main_2h fits MAIN_HAND only, flag or not"] = _only(ItemFamilies.slots_for(main, ItemFamilies.HAND_MAIN_2H, true), "MAIN_HAND")
	checks["off fits OFF_HAND only"] = _only(ItemFamilies.slots_for(main, ItemFamilies.HAND_OFF, true), "OFF_HAND")
	# Every resolved weapon of a main family is a main-hand weapon with no off-hand category (the retype included).
	var retyped_ok: bool = true
	var main_1h_items: Array[Equipment] = []
	var off_items: Array[Equipment] = []
	for row: Dictionary in ContentCatalog.entries("items"):
		var fam: Dictionary = ContentCatalog.family_of(row)
		if ItemFamilies.category(fam) == "main":
			retyped_ok = retyped_ok and ItemFamilies.MAIN_HANDS.has(str(row["hand"])) and str(row["off_hand_category"]).is_empty()
		var built: Item = ContentFactory.item(row)
		if built is Equipment:
			var eq: Equipment = built
			if eq.hand() == Equipment.HAND_MAIN_1H:
				main_1h_items.append(eq)
			elif eq.hand() == Equipment.HAND_OFF:
				off_items.append(eq)
	checks["items of main families are main-hand weapons without an off-hand category"] = retyped_ok
	var filter := StashFilter.new()
	filter.slot_filter = "OFF_HAND"
	Equipment.allow_plain_dual_wield = false
	var strict_ok: bool = not main_1h_items.is_empty()
	for eq: Equipment in main_1h_items:
		strict_ok = strict_ok and not filter.matches_slot(eq)
	for eq: Equipment in off_items:
		strict_ok = strict_ok and filter.matches_slot(eq)
	Equipment.allow_plain_dual_wield = true
	var legacy_ok: bool = true
	for eq: Equipment in main_1h_items:
		legacy_ok = legacy_ok and filter.matches_slot(eq)
	Equipment.allow_plain_dual_wield = saved
	checks["the stash OFF_HAND filter hides plain main_1h weapons by default"] = strict_ok
	checks["with the legacy flag they show again"] = legacy_ok
	# Acceptance scan: no shipped class or unit holds a main_1h item in its off hand (flag false).
	var hits: Array[String] = []
	for kind: String in ["classes", "units"]:
		for e: Dictionary in ContentCatalog.entries(kind):
			var off_id: String = str(e.get("off_hand", ""))
			var item: Dictionary = ContentCatalog.get_entry("items", off_id)
			if not item.is_empty() and str(item.get("hand", "")) == ItemFamilies.HAND_MAIN_1H:
				hits.append("%s:%s holds %s" % [kind, str(e["id"]), off_id])
	checks["no class or unit holds a main_1h item in the off hand"] = hits.is_empty()
	var off_entry: Dictionary = {}
	if not main_1h_items.is_empty():
		off_entry = ContentCatalog.get_entry("items", main_1h_items[0].id)
	Equipment.allow_plain_dual_wield = false
	var strict_fit: bool = not off_entry.is_empty() and not ContentFactory.item_fits_slot(off_entry, "OFF_HAND")
	var worn_fit: bool = not off_entry.is_empty() and ContentFactory.item_fits_slot(off_entry, "OFF_HAND", true)
	Equipment.allow_plain_dual_wield = saved
	checks["catalog loadout validation gates a plain main_1h off-hand item"] = strict_fit
	checks["worn (saved) gear is kept: the keep_worn check accepts it"] = worn_fit
	if not hits.is_empty():
		print("    [%s] off-hand scan hits (report, do not fix silently): %s" % [TAG, ", ".join(hits)])
	return TestHelpers.report(TAG, "Strict hand typing, flag both ways (%d main_1h, %d off items)" % [main_1h_items.size(), off_items.size()], checks)

# ------------------------------------------------------------------------------
# 6. Storage: items.json version 2 (family + modifiers), item_families.json version 1
# ------------------------------------------------------------------------------

static func _test_storage() -> bool:
	var checks: Dictionary = {}
	var before: Dictionary = {}
	for row: Dictionary in ContentCatalog.entries("items"):
		before[str(row["id"])] = row
	checks["save_all works"] = ContentCatalog.save_all() == OK
	var items_doc: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(ContentCatalog.path_for("items", SANDBOX))))
	var fam_doc: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(ContentCatalog.path_for("item_families", SANDBOX))))
	checks["items.json is version 2"] = DictUtil.to_int(items_doc.get("version", 0)) == ContentCatalog.version_of("items") and ContentCatalog.version_of("items") == 2
	checks["item_families.json is version 1"] = DictUtil.to_int(fam_doc.get("version", 0)) == 1
	var no_derived: bool = true
	for raw: Variant in DictUtil.array_of(items_doc.get("entries", [])):
		var stored: Dictionary = DictUtil.dict_of(raw)
		for key: String in ItemModifiers.DERIVED_KEYS:
			no_derived = no_derived and not stored.has(key)
		no_derived = no_derived and stored.has("family") and not stored.has("origin")
	checks["stored items carry family + modifiers, no derived keys, no default origin"] = no_derived
	ContentCatalog.reload_from_disk()
	var same: bool = ContentCatalog.load_errors().is_empty()
	for row: Dictionary in ContentCatalog.entries("items"):
		same = same and before.has(str(row["id"])) and ItemModifiers.engine_equal(row, DictUtil.dict_of(before[str(row["id"])]))
	checks["reloading the version 2 file gives the same resolved rows"] = same and ContentCatalog.entries("items").size() == before.size()
	checks["validate_all stays empty after the reload"] = ContentCatalog.validate_all().is_empty()
	return TestHelpers.report(TAG, "Storage versions and reload", checks)
