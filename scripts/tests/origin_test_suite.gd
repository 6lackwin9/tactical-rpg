class_name OriginTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Origin Test Suite (FORGE1 section 12 / SKILL1-L)
# origin system / user: only user assets store it, key fields of system assets are locked, delete only for user assets,
# explicit promote / demote, the system-edit scope for seeds, and SAFE DELETE (scan, refuse, remove). Fixture ids that
# must be found by the scan are written as literals in this file on purpose; the "free" ones are built by concatenation.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("origin_test")
const TAG: String = "Origin"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING ORIGIN TEST (SKILL1-L)                  ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_default_and_storage() and ok
	ok = _test_delete_and_create() and ok
	ok = _test_key_lock() and ok
	ok = _test_promote_demote() and ok
	ok = _test_safe_delete() and ok
	ok = _test_scope_and_invariants() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

# A shipped class skill entry with ranks (not basic, not engine).
# A shipped ranked class family whose grade can flip freely: it requires nothing and no other skill requires it
# (chosen by rule, so content changes to grades / requires never break the origin checks).
static func _system_family() -> Dictionary:
	var required: Dictionary = {}
	for row: Dictionary in ContentCatalog.entries("skills"):
		for id: String in ContentFactory._strings(row.get("requires", [])):
			required[id] = true
	for entry: Dictionary in ContentCatalog.skill_entries():
		if DictUtil.to_bool(entry.get("basic", false)) or DictUtil.to_bool(entry.get("system", false)) or not entry.has("ranks"):
			continue
		if required.has(str(entry["id"])) or not ContentFactory._strings(entry.get("requires", [])).is_empty():
			continue
		return entry
	return {}

static func _test_default_and_storage() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var all_system: bool = true
	for kind: String in ContentSchema.KINDS:
		for e: Dictionary in ContentCatalog.entries(kind):
			all_system = all_system and str(e["origin"]) == ContentSchema.ORIGIN_SYSTEM
	checks["a shipped file without origin loads every row as system"] = all_system
	checks["a new skill is stored as a user asset"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_or_new")).is_empty() and str(ContentCatalog.get_entry("skills", "t_or_new")["origin"]) == ContentSchema.ORIGIN_USER
	checks["a new item is a user asset too"] = ContentCatalog.upsert("items", {"id": "t_or_item", "name": "Item", "item_type": "MATERIAL"}).is_empty() and str(ContentCatalog.get_entry("items", "t_or_item")["origin"]) == ContentSchema.ORIGIN_USER
	checks["save_all works"] = ContentCatalog.save_all() == OK
	for kind: String in ["skills", "items"]:
		var doc: Variant = JSON.parse_string(FileAccess.get_file_as_string(ContentCatalog.path_for(kind, SANDBOX)))
		var entries_raw: Array = DictUtil.array_of(DictUtil.dict_of(doc).get("entries", []))
		var only_user: bool = true
		var user_seen: int = 0
		for raw: Variant in entries_raw:
			var e: Dictionary = DictUtil.dict_of(raw)
			if e.has("origin"):
				only_user = only_user and str(e["origin"]) == ContentSchema.ORIGIN_USER
				user_seen += 1
		checks["only user assets carry origin in %s.json" % kind] = only_user and user_seen == 1
	ContentCatalog.reload_from_disk()
	checks["origin survives a save and reload"] = str(ContentCatalog.get_entry("skills", "t_or_new")["origin"]) == ContentSchema.ORIGIN_USER and str(ContentCatalog.get_entry("items", "t_or_item")["origin"]) == ContentSchema.ORIGIN_USER
	checks["an unknown origin value is an error"] = not ContentCatalog.validate_entry("skills", {"id": "t_or_bad", "name": "Bad", "kind": "ATTACK", "origin": "other"}).is_empty()
	return TestHelpers.report(TAG, "Default system, only user assets store it", checks)

static func _test_delete_and_create() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var system_entry: Dictionary = _system_family()
	checks["a shipped class skill family exists to test with"] = not system_entry.is_empty()
	var head: String = str(system_entry.get("id", ""))
	checks["remove_skill refuses a system asset with the reason"] = ContentCatalog.remove_skill(head).contains("system asset")
	checks["remove refuses a system row with the reason"] = ContentCatalog.remove("skills", head).contains("system asset")
	checks["remove refuses a system item"] = ContentCatalog.remove("items", ContentCatalog.ids("items")[0]).contains("system asset")
	checks["a user skill is created"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_or_del")).is_empty()
	checks["...and can be deleted"] = ContentCatalog.remove_skill("t_or_del").is_empty() and not ContentCatalog.has("skills", "t_or_del")
	var explicit_system: Dictionary = TestHelpers.attack("t_or_sys", {"origin": "system"})
	checks["origin system for a new id is refused outside the system-edit scope"] = not ContentCatalog.upsert_skill(explicit_system).is_empty() and not ContentCatalog.has("skills", "t_or_sys")
	ContentCatalog.begin_system_edit()
	checks["...and accepted inside it"] = ContentCatalog.upsert_skill(explicit_system).is_empty() and str(ContentCatalog.get_entry("skills", "t_or_sys")["origin"]) == ContentSchema.ORIGIN_SYSTEM
	ContentCatalog.end_system_edit()
	checks["legacy upsert of a new id also creates a user asset"] = ContentCatalog.upsert("skills", {"id": "t_or_legacy", "name": "L", "kind": "ATTACK"}).is_empty() and str(ContentCatalog.get_entry("skills", "t_or_legacy")["origin"]) == ContentSchema.ORIGIN_USER
	checks["...and it can be removed again"] = ContentCatalog.remove("skills", "t_or_legacy").is_empty()
	var dup_id: String = ContentCatalog.duplicate_entry("skills", head)
	checks["a clone of a system skill is a user family with every rank"] = not dup_id.is_empty() and str(ContentCatalog.get_entry("skills", dup_id)["origin"]) == ContentSchema.ORIGIN_USER and ContentCatalog.family_rows(dup_id).size() == SkillRanks.rank_count(system_entry)
	checks["...which can be deleted"] = ContentCatalog.remove_skill(dup_id).is_empty()
	return TestHelpers.report(TAG, "Delete only for user assets; new assets are user", checks)

static func _test_key_lock() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var system_entry: Dictionary = _system_family()
	var head: String = str(system_entry["id"])
	var edited_name: Dictionary = system_entry.duplicate(true)
	edited_name["name"] = str(system_entry["name"]) + "!"
	checks["a non-key field of a system skill can change"] = ContentCatalog.upsert_skill(edited_name).is_empty() and str(ContentCatalog.get_entry("skills", head)["name"]).ends_with("!")
	var number: Dictionary = ContentCatalog.skill_entry(head)
	number["blocks"] = mini(6, DictUtil.to_int(number["blocks"]) + 1)
	checks["a number of a system skill can change"] = ContentCatalog.upsert_skill(number).is_empty()
	var other_kind: String = "DEFEND" if str(system_entry["kind"]) != "DEFEND" else "FOCUS"
	var kind_change: Dictionary = ContentCatalog.skill_entry(head)
	kind_change["kind"] = other_kind
	var kind_errors: Array[String] = ContentCatalog.upsert_skill(kind_change)
	checks["changing the kind of a system skill returns the locked-field error naming it"] = TestHelpers.any_contains(kind_errors, "key field") and TestHelpers.any_contains(kind_errors, "Kind")
	checks["...and changed nothing"] = str(ContentCatalog.get_entry("skills", head)["kind"]) == str(system_entry["kind"])
	var grade_change: Dictionary = ContentCatalog.skill_entry(head)
	grade_change["grade"] = "expert" if str(grade_change.get("grade", "novice")) != "expert" else "novice"
	checks["grade is a key field"] = TestHelpers.any_contains(ContentCatalog.upsert_skill(grade_change), "key field")
	var rank_change: Dictionary = ContentCatalog.skill_entry(head)
	rank_change.erase("ranks")
	checks["the number of ranks of a system skill is locked"] = TestHelpers.any_contains(ContentCatalog.upsert_skill(rank_change), "ranks")
	var renamed: Dictionary = ContentCatalog.skill_entry(head)
	renamed["id"] = "t_or_renamed"
	checks["the id of a system skill is locked"] = not ContentCatalog.upsert_skill(renamed, head).is_empty() and not ContentCatalog.has("skills", "t_or_renamed")
	var legacy: Dictionary = ContentCatalog.get_entry("skills", head)
	legacy["basic"] = true
	checks["the legacy upsert enforces the same lock"] = not ContentCatalog.upsert("skills", legacy).is_empty()
	var user_entry: Dictionary = {"id": "t_or_user", "name": "User", "kind": "FOCUS", "level": 2, "blocks": 1}
	checks["a user skill's key fields stay editable"] = ContentCatalog.upsert_skill(user_entry).is_empty()
	user_entry["kind"] = "DEFEND"
	checks["...kind included"] = ContentCatalog.upsert_skill(user_entry).is_empty() and str(ContentCatalog.get_entry("skills", "t_or_user")["kind"]) == "DEFEND"
	return TestHelpers.report(TAG, "Key fields of system assets are locked", checks)

static func _test_promote_demote() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	checks["fixtures accepted"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_or_ref")).is_empty() and ContentCatalog.upsert_skill(TestHelpers.attack("t_or_" + "free")).is_empty()
	checks["promote flips user to system"] = ContentCatalog.promote("skills", "t_or_ref").is_empty() and str(ContentCatalog.get_entry("skills", "t_or_ref")["origin"]) == ContentSchema.ORIGIN_SYSTEM
	checks["...and persists"] = _stored_origin("skills", "t_or_ref") == ""
	ContentCatalog.reload_from_disk()
	checks["...across a reload"] = str(ContentCatalog.get_entry("skills", "t_or_ref")["origin"]) == ContentSchema.ORIGIN_SYSTEM
	checks["delete is refused once promoted"] = ContentCatalog.remove_skill("t_or_ref").contains("system asset")
	checks["promoting a system asset is an error"] = not ContentCatalog.promote("skills", "t_or_ref").is_empty() and not ContentCatalog.promote("skills", "strike").is_empty()
	var demote_ref: String = ContentCatalog.demote("skills", "t_or_ref")
	checks["demote is refused for an asset a test names, listing the reference"] = demote_ref.contains("origin_test_suite")
	checks["demote of a basic skill is refused"] = ContentCatalog.demote("skills", "strike").contains("basic")
	checks["demote of an engine skill is refused"] = not ContentCatalog.demote("skills", "move").is_empty()
	checks["demote of an unknown asset is an error"] = not ContentCatalog.demote("skills", "t_or_missing").is_empty()
	var free_id: String = "t_or_" + "free"
	checks["promote then demote an unreferenced asset"] = ContentCatalog.promote("skills", free_id).is_empty() and ContentCatalog.remove_skill(free_id).contains("system asset") and ContentCatalog.demote("skills", free_id).is_empty()
	checks["...delete works again"] = ContentCatalog.remove_skill(free_id).is_empty()
	return TestHelpers.report(TAG, "Promote and demote", checks)

# "" when the stored entry has no origin key (= system), else its stored value; "?" when the entry is missing from the file.
static func _stored_origin(kind: String, id: String) -> String:
	var doc: Variant = JSON.parse_string(FileAccess.get_file_as_string(ContentCatalog.path_for(kind, ContentCatalog.get_root())))
	for raw: Variant in DictUtil.array_of(DictUtil.dict_of(doc).get("entries", [])):
		var e: Dictionary = DictUtil.dict_of(raw)
		if str(e.get("id", "")) == id:
			return str(e.get("origin", ""))
	return "?"

static func _test_safe_delete() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	ContentCatalog.begin_system_edit()
	var referenced: String = "t_or_sys_ref"
	var free_id: String = "t_or_sys_" + "free"
	checks["system fixtures created in the scope"] = ContentCatalog.upsert_skill(TestHelpers.attack(referenced)).is_empty() and ContentCatalog.upsert_skill(TestHelpers.attack(free_id)).is_empty()
	ContentCatalog.end_system_edit()
	checks["the fixtures are system assets"] = str(ContentCatalog.get_entry("skills", free_id)["origin"]) == ContentSchema.ORIGIN_SYSTEM
	var scan: Array[String] = AssetScan.scan("skills", referenced)
	checks["the scan lists the test file that names the id"] = TestHelpers.any_contains(scan, "origin_test_suite.gd")
	var refused: Dictionary = AssetScan.safe_delete("skills", referenced, true)
	checks["SAFE DELETE refuses while a reference exists and lists it"] = not DictUtil.to_bool(refused["ok"]) and not DictUtil.to_bool(refused["removed"]) and not DictUtil.array_of(refused["refs"]).is_empty()
	checks["...and nothing changed"] = ContentCatalog.has("skills", referenced)
	var dry: Dictionary = AssetScan.safe_delete("skills", free_id, false)
	checks["a dry run on an unreferenced asset removes nothing"] = DictUtil.to_bool(dry["ok"]) and not DictUtil.to_bool(dry["removed"]) and ContentCatalog.has("skills", free_id)
	var applied: Dictionary = AssetScan.safe_delete("skills", free_id, true)
	checks["--apply removes an unreferenced system asset"] = DictUtil.to_bool(applied["removed"]) and not ContentCatalog.has("skills", free_id)
	checks["...inside the scope only (the scope is closed again)"] = not ContentCatalog.in_system_edit()
	checks["SAFE DELETE refuses a basic skill"] = not DictUtil.to_bool(AssetScan.safe_delete("skills", "strike", true)["ok"]) and ContentCatalog.has("skills", "strike")
	checks["SAFE DELETE refuses a required engine skill"] = not DictUtil.to_bool(AssetScan.safe_delete("skills", "move", true)["ok"]) and ContentCatalog.has("skills", "move")
	var used: String = ""
	for entry: Dictionary in ContentCatalog.entries("classes"):
		for step: Variant in DictUtil.array_of(entry["skill_path"]):
			used = str(DictUtil.dict_of(step).get("skill", ""))
			break
		if not used.is_empty():
			break
	checks["SAFE DELETE refuses a skill a class path uses"] = not used.is_empty() and not DictUtil.to_bool(AssetScan.safe_delete("skills", used, true)["ok"]) and ContentCatalog.has("skills", used)
	checks["tools/safe_delete.gd exists"] = FileAccess.file_exists("res://tools/safe_delete.gd")
	return TestHelpers.report(TAG, "SAFE DELETE scans, refuses and removes", checks)

static func _test_scope_and_invariants() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var system_entry: Dictionary = _system_family()
	var head: String = str(system_entry["id"])
	var tweak: Dictionary = ContentCatalog.skill_entry(head)
	tweak["grade"] = "advanced" if str(tweak.get("grade", "novice")) == "novice" else "novice"
	checks["outside the scope a key edit is refused"] = not ContentCatalog.upsert_skill(tweak).is_empty()
	ContentCatalog.begin_system_edit()
	checks["inside the scope a key edit is allowed"] = ContentCatalog.upsert_skill(tweak).is_empty() and str(ContentCatalog.get_entry("skills", head)["grade"]) == str(tweak["grade"])
	checks["the scope is on"] = ContentCatalog.in_system_edit()
	ContentCatalog.reload_from_disk()
	checks["the scope is off after reload_from_disk"] = not ContentCatalog.in_system_edit()
	ContentCatalog.begin_system_edit()
	checks["remove stays refused in the scope without allow_remove"] = ContentCatalog.remove_skill("t_or_none").begins_with("No") and not ContentCatalog.remove_skill(head).is_empty()
	ContentCatalog.end_system_edit()
	var ui_calls: bool = false
	for dir: String in ["res://scripts/ui/forge"]:
		for file: String in DirAccess.get_files_at(dir):
			if file.ends_with(".gd") and FileAccess.get_file_as_string("%s/%s" % [dir, file]).contains("begin_system_edit"):
				ui_calls = true
	checks["the Forge never opens the system-edit scope"] = not ui_calls
	var basics_ok: bool = true
	for row: Dictionary in ContentCatalog.entries("skills"):
		if DictUtil.to_bool(row.get("basic", false)) or DictUtil.to_bool(row.get("system", false)):
			basics_ok = basics_ok and str(row["origin"]) == ContentSchema.ORIGIN_SYSTEM
	for required: String in SkillKinds.REQUIRED_IDS:
		basics_ok = basics_ok and ContentCatalog.has("skills", required) and str(ContentCatalog.get_entry("skills", required)["origin"]) == ContentSchema.ORIGIN_SYSTEM
	checks["all basics, engine rows and REQUIRED_IDS are origin system"] = basics_ok
	var broken: Dictionary = ContentCatalog.get_entry("skills", "strike")
	broken["origin"] = ContentSchema.ORIGIN_USER
	ContentCatalog.begin_system_edit()
	ContentCatalog.upsert("skills", broken)
	ContentCatalog.end_system_edit()
	checks["validate_all flags a basic skill that is not system"] = ContentCatalog.validate_all().has("skills:required")
	return TestHelpers.report(TAG, "The system-edit scope and the origin invariants", checks)
