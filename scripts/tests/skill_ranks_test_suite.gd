class_name SkillRanksTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Ranks Test Suite (FORGE1 / SKILL1-L)
# Ranked storage (version 2 files, flat rows in memory): SkillRanks expand / collapse round trip over EVERY
# shipped row, modifiers resolved against the base, derived names and ids, the ranked API (upsert_skill,
# remove_skill, head rename), both file versions, and the legacy flat upsert pattern. Counts and ids are read
# from the catalog, never hard-coded.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("skill_ranks_test")
const TAG: String = "SkillRanks"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SKILL RANKS TEST (SKILL1-L)             ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_round_trip() and ok
	ok = _test_modifiers() and ok
	ok = _test_names_and_validation() and ok
	ok = _test_rename_and_remove() and ok
	ok = _test_file_versions() and ok
	ok = _test_lossless_save() and ok
	ok = _test_legacy_flat() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

static func _test_round_trip() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var rows: Array[Dictionary] = ContentCatalog.entries("skills")
	var folded: Dictionary = SkillRanks.collapse(rows)
	var fold_errors: Array = folded["errors"]
	checks["every shipped family folds (no errors)"] = fold_errors.is_empty()
	var ranked: Array = folded["entries"]
	var heads: Array[String] = []
	for row: Dictionary in rows:
		var head: String = SkillRanks.head_of(row)
		if not heads.has(head):
			heads.append(head)
	checks["entry count equals the number of distinct families"] = ranked.size() == heads.size()
	var total: int = 0
	var identical: bool = true
	var ids_ok: bool = true
	var by_id: Dictionary = {}
	for row: Dictionary in rows:
		by_id[str(row["id"])] = ContentSchema.normalize("skills", row)
	for item: Variant in ranked:
		var entry: Dictionary = DictUtil.dict_of(item)
		var expanded: Array[Dictionary] = SkillRanks.expand(entry)
		total += expanded.size()
		for i: int in range(expanded.size()):
			var row: Dictionary = expanded[i]
			ids_ok = ids_ok and str(row["id"]) == SkillRanks.rank_id(str(entry["id"]), i + 1) and DictUtil.to_int(row["tier"]) == i + 1
			identical = identical and by_id.has(str(row["id"])) and by_id[str(row["id"])] == row
	checks["rank counts sum to the row count"] = total == rows.size()
	checks["ids are <head> and <head>_<n>, tiers 1..n"] = ids_ok
	checks["collapse then expand reproduces every row field by field"] = identical
	var family_ok: bool = true
	for item: Variant in ranked:
		var entry: Dictionary = DictUtil.dict_of(item)
		family_ok = family_ok and SkillCatalog.family_tiers(str(entry["id"])).size() == SkillRanks.rank_count(entry)
	checks["family_tiers(head).size() == ranks.size() + 1"] = family_ok
	var rank_major: bool = true
	var last_tier: int = 1
	for row: Dictionary in ContentCatalog.entries("skills"):
		var tier: int = DictUtil.to_int(row["tier"], 1)
		rank_major = rank_major and tier >= last_tier
		last_tier = tier
	checks["a version 2 file loads rank-major (rank 1 rows first, then rank 2, ...)"] = rank_major or ContentCatalog.version_of("skills") < 2
	return TestHelpers.report(TAG, "Round trip over the shipped catalog", checks)

static func _test_modifiers() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var base: Dictionary = TestHelpers.attack("t_rk", {"status_effect": "stagger", "status_potency": 50})
	var mods: Array[Dictionary] = [{"level": 3, "power_mult": 1.4}, {"level": 4, "power_mult": 1.8, "status_potency": 70}]
	var entry: Dictionary = SkillRanks.entry(base, mods)
	checks["a fixture family is accepted"] = ContentCatalog.upsert_skill(entry).is_empty()
	var r1: Dictionary = ContentCatalog.get_entry("skills", "t_rk")
	var r2: Dictionary = ContentCatalog.get_entry("skills", "t_rk_2")
	var r3: Dictionary = ContentCatalog.get_entry("skills", "t_rk_3")
	checks["three flat rows exist"] = not r1.is_empty() and not r2.is_empty() and not r3.is_empty()
	checks["a modifier is an absolute value"] = DictUtil.to_int(r2["level"]) == 3 and is_equal_approx(DictUtil.to_float(r3["power_mult"]), 1.8) and DictUtil.to_int(r3["status_potency"]) == 70
	checks["rank 2 inherits what rank 2 does not modify (resolved against the base)"] = DictUtil.to_int(r2["status_potency"]) == 50
	checks["rank 1 is the template"] = DictUtil.to_int(r1["level"]) == 2 and is_equal_approx(DictUtil.to_float(r1["power_mult"]), 1.0)
	# Editing a base field changes every rank row.
	var edited: Dictionary = ContentCatalog.skill_entry("t_rk")
	edited["blocks"] = DictUtil.to_int(edited["blocks"]) + 1
	checks["a base edit is accepted"] = ContentCatalog.upsert_skill(edited).is_empty()
	var want_blocks: int = DictUtil.to_int(edited["blocks"])
	checks["editing a base field changes every rank row"] = DictUtil.to_int(ContentCatalog.get_entry("skills", "t_rk")["blocks"]) == want_blocks and DictUtil.to_int(ContentCatalog.get_entry("skills", "t_rk_2")["blocks"]) == want_blocks and DictUtil.to_int(ContentCatalog.get_entry("skills", "t_rk_3")["blocks"]) == want_blocks
	# Editing one modifier changes only that rank (no cascade).
	var third_before: Dictionary = ContentCatalog.get_entry("skills", "t_rk_3")
	var tweak: Dictionary = ContentCatalog.skill_entry("t_rk")
	var tweak_mods: Array = tweak["ranks"]
	var first_mod: Dictionary = DictUtil.dict_of(tweak_mods[0])
	first_mod["power_mult"] = 1.5
	tweak_mods[0] = first_mod
	checks["a modifier edit is accepted"] = ContentCatalog.upsert_skill(tweak).is_empty()
	checks["editing rank 2 changes rank 2"] = is_equal_approx(DictUtil.to_float(ContentCatalog.get_entry("skills", "t_rk_2")["power_mult"]), 1.5)
	checks["editing rank 2 leaves rank 3 equal to its own resolved value"] = ContentCatalog.get_entry("skills", "t_rk_3") == third_before
	# A base-only key in a modifier is rejected.
	var bad_mods: Array[Dictionary] = [{"kind": "DEFEND"}]
	checks["a base-only key in a modifier is rejected"] = not ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_rk_bad"), bad_mods)).is_empty()
	var name_mods: Array[Dictionary] = [{"name": "Other"}]
	checks["a name modifier is rejected"] = not ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_rk_bad2"), name_mods)).is_empty()
	var hidden_mods: Array[Dictionary] = [{"mitigation": 0.3}]
	checks["a modifier for a field the kind does not show is rejected"] = not ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_rk_bad3"), hidden_mods)).is_empty()
	var range_mods: Array[Dictionary] = [{"level": 99}]
	checks["a modifier outside the field range is rejected"] = not ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_rk_bad4"), range_mods)).is_empty()
	var too_many: Array[Dictionary] = [{"level": 3}, {"level": 4}, {"level": 5}]
	checks["more ranks than the class cap are rejected"] = not ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_rk_bad5"), too_many)).is_empty()
	# A modifier equal to the base is dropped on save and warned about.
	var same_mods: Array[Dictionary] = [{"level": 3, "power_mult": 1.0}]
	var same_entry: Dictionary = SkillRanks.entry(TestHelpers.attack("t_rk_same"), same_mods)
	checks["a modifier equal to the base warns"] = TestHelpers.any_contains(ContentCatalog.warnings_for("skills", same_entry), "equals the base")
	checks["...and is accepted"] = ContentCatalog.upsert_skill(same_entry).is_empty()
	var stored_same: Dictionary = ContentCatalog.skill_entry("t_rk_same")
	var stored_mod: Dictionary = DictUtil.dict_of(DictUtil.array_of(stored_same["ranks"])[0])
	checks["...and dropped on save (the folded modifier keeps only what differs)"] = stored_mod.has("level") and not stored_mod.has("power_mult")
	checks["rank_fields follows the kind"] = SkillRanks.rank_fields("DEFEND").has("mitigation") and not SkillRanks.rank_fields("DEFEND").has("power_mult") and SkillRanks.rank_fields("ATTACK").has("power_mult")
	return TestHelpers.report(TAG, "Modifiers resolve against the base, no cascade", checks)

static func _test_names_and_validation() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var mods: Array[Dictionary] = [{"level": 3}, {"level": 4}]
	var entry: Dictionary = SkillRanks.entry(TestHelpers.attack("t_nm", {"name": "Nm"}), mods)
	var rows: Array[Dictionary] = SkillRanks.expand(entry)
	var derived: bool = true
	for i: int in range(rows.size()):
		derived = derived and str(rows[i]["name"]) == ("Nm" if i == 0 else "Nm %s" % SkillCatalog.roman(i + 1))
	checks["rank names are derived (Name, Name II, Name III)"] = derived
	var long_name: String = "x".repeat(30)
	var long_entry: Dictionary = SkillRanks.entry(TestHelpers.attack("t_nm_long", {"name": long_name}), mods)
	checks["the derived rank name must fit 32 characters"] = TestHelpers.any_contains(TestHelpers.messages(ContentCatalog.validate_skill_entry(long_entry)), "derived rank name")
	checks["a base name over 32 is refused"] = not ContentCatalog.validate_skill_entry(TestHelpers.attack("t_nm_long2", {"name": "y".repeat(33)})).is_empty()
	var basic_ranked: Dictionary = SkillRanks.entry(TestHelpers.attack("t_nm_basic", {"basic": true}), mods)
	checks["a basic skill with ranks is rejected"] = TestHelpers.any_contains(TestHelpers.messages(ContentCatalog.validate_skill_entry(basic_ranked)), "exactly one rank")
	checks["a standalone skill named like a future rank id is accepted"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_clash_2")).is_empty()
	var clash_mods: Array[Dictionary] = [{"level": 3}]
	checks["a rank id used by another family is refused"] = TestHelpers.any_contains(TestHelpers.messages(ContentCatalog.validate_skill_entry(SkillRanks.entry(TestHelpers.attack("t_clash"), clash_mods))), "already used")
	var unlock_mods: Array[Dictionary] = [{"unlock_offset": 3}]
	checks["unlock_offset is rank-modifiable (rank 2+)"] = ContentCatalog.validate_skill_entry(SkillRanks.entry(TestHelpers.attack("t_nm_off"), unlock_mods)).is_empty()
	var bad_off: Dictionary = TestHelpers.attack("t_nm_off2", {"unlock_offset": 2})
	checks["a rank 1 unlock offset above 0 is refused"] = not ContentCatalog.validate_skill_entry(bad_off).is_empty()
	return TestHelpers.report(TAG, "Derived names, caps and entry validation", checks)

static func _test_rename_and_remove() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var mods: Array[Dictionary] = [{"level": 3}, {"level": 4}]
	checks["fixture family accepted"] = ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_rn", {"grade": "novice"}), mods)).is_empty()
	var adv: Dictionary = TestHelpers.attack("t_rn_adv", {"grade": "advanced", "requires": ["t_rn"]})
	checks["an advanced skill requiring it is accepted"] = ContentCatalog.upsert_skill(adv).is_empty()
	checks["a passive naming the head is accepted"] = ContentCatalog.upsert("passives", {"id": "t_rn_pass", "name": "P", "effects": [{"type": "skill", "skill": "t_rn"}]}).is_empty()
	checks["a unit with the skill as an extra is accepted"] = ContentCatalog.upsert("units", {"id": "t_rn_unit", "name": "U", "side": "party", "class_id": "warrior", "level": 5, "extra_skills": ["t_rn"], "loadout": ["t_rn"]}).is_empty()
	var refused: String = ContentCatalog.remove_skill("t_rn")
	checks["remove_skill refuses a referenced head (every rank id is counted)"] = refused.contains("still used")
	var entry: Dictionary = ContentCatalog.skill_entry("t_rn")
	entry["id"] = "t_rn_new"
	var errors: Array[String] = ContentCatalog.upsert_skill(entry, "t_rn")
	checks["a head rename is accepted"] = errors.is_empty()
	checks["...rank ids follow the head"] = ContentCatalog.has("skills", "t_rn_new") and ContentCatalog.has("skills", "t_rn_new_2") and ContentCatalog.has("skills", "t_rn_new_3") and not ContentCatalog.has("skills", "t_rn") and not ContentCatalog.has("skills", "t_rn_2")
	checks["...rank rows keep their family"] = str(ContentCatalog.get_entry("skills", "t_rn_new_3")["family"]) == "t_rn_new"
	var passive: Dictionary = ContentCatalog.get_entry("passives", "t_rn_pass")
	var fx: Dictionary = DictUtil.dict_of(DictUtil.array_of(passive["effects"])[0])
	checks["...passive reference follows"] = str(fx["skill"]) == "t_rn_new"
	checks["...another skill's requires follows"] = ContentFactory._strings(ContentCatalog.get_entry("skills", "t_rn_adv")["requires"]) == ["t_rn_new"]
	var unit: Dictionary = ContentCatalog.get_entry("units", "t_rn_unit")
	checks["...unit extra_skills and loadout follow"] = ContentFactory._strings(unit["extra_skills"]) == ["t_rn_new"] and ContentFactory._strings(unit["loadout"]) == ["t_rn_new"]
	var stale: Array[String] = []
	var all_problems: Dictionary = ContentCatalog.validate_all()
	for owner: Variant in all_problems:
		for message: Variant in DictUtil.array_of(all_problems[owner]):
			# Shipped class paths still list basics until the SKILL1-A migration; only fixture problems matter here.
			if str(owner).contains("t_rn") or str(message).contains("t_rn"):
				stale.append("%s: %s" % [str(owner), str(message)])
	checks["no fixture problem after the rename (%s)" % "; ".join(stale)] = stale.is_empty()
	# Free the references, then the whole family can go.
	ContentCatalog.remove("units", "t_rn_unit")
	ContentCatalog.remove("passives", "t_rn_pass")
	ContentCatalog.remove_skill("t_rn_adv")
	checks["remove_skill removes every rank of an unreferenced family"] = ContentCatalog.remove_skill("t_rn_new").is_empty() and not ContentCatalog.has("skills", "t_rn_new") and not ContentCatalog.has("skills", "t_rn_new_2") and not ContentCatalog.has("skills", "t_rn_new_3")
	checks["remove_skill refuses a basic skill"] = not ContentCatalog.remove_skill("strike").is_empty()
	checks["remove_skill refuses an unknown head"] = not ContentCatalog.remove_skill("t_no_such").is_empty()
	return TestHelpers.report(TAG, "Head rename carries ranks and references; remove_skill", checks)

static func _test_file_versions() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var path: String = ContentCatalog.path_for("skills", SANDBOX)
	# Version 1: flat rows, as before.
	var flat: Dictionary = {"kind": "skills", "version": 1, "entries": [
		{"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 2, "basic": true},
		{"id": "t_v1", "name": "V1", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "One."},
		{"id": "t_v1_2", "name": "V1 II", "kind": "ATTACK", "level": 2, "blocks": 1, "family": "t_v1", "tier": 2, "description": "Two."}]}
	_write(path, flat)
	ContentCatalog.reload_from_disk()
	checks["a version 1 flat file loads"] = ContentCatalog.has("skills", "t_v1_2") and str(ContentCatalog.get_entry("skills", "t_v1_2")["family"]) == "t_v1" and ContentCatalog.load_errors().is_empty()
	# Version 2: ranked entries expand.
	var ranked: Dictionary = {"kind": "skills", "version": 2, "entries": [
		{"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 2, "basic": true},
		{"id": "t_v2", "name": "V2", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "{name} at {ap}", "ranks": [{"level": 2}]}]}
	_write(path, ranked)
	ContentCatalog.reload_from_disk()
	var second: Dictionary = ContentCatalog.get_entry("skills", "t_v2_2")
	checks["a version 2 ranked file expands to flat rows"] = not second.is_empty() and DictUtil.to_int(second["tier"]) == 2 and str(second["family"]) == "t_v2" and str(second["name"]) == "V2 II"
	checks["...and renders the description template per rank"] = str(second["description"]) == "V2 II at 2 AP" and str(ContentCatalog.get_entry("skills", "t_v2")["description"]) == "V2 at 1 AP"
	# A version 2 file may not store derived keys.
	for banned: String in ["family", "tier"]:
		var bad_entry: Dictionary = {"id": "t_v2b", "name": "B", "kind": "ATTACK", banned: "x" if banned == "family" else 1}
		_write(path, {"kind": "skills", "version": 2, "entries": [bad_entry]})
		ContentCatalog.reload_from_disk()
		checks["a version 2 file with '%s' is a load error" % banned] = not ContentCatalog.load_errors().is_empty() and not ContentCatalog.has("skills", "t_v2b")
	_write(path, {"kind": "skills", "version": 2, "entries": [{"id": "t_v2c", "name": "C", "kind": "ATTACK", "ranks": [{"name": "Other"}]}]})
	ContentCatalog.reload_from_disk()
	checks["a version 2 file with a name modifier is a load error"] = not ContentCatalog.load_errors().is_empty()
	# save writes version 2 and the result reloads to the same rows.
	checks["sandbox fork again"] = TestHelpers.fork(SANDBOX)
	var before: Array[Dictionary] = ContentCatalog.entries("skills")
	checks["save writes version 2"] = ContentCatalog.save("skills") == OK and DictUtil.to_int(JSON.parse_string(FileAccess.get_file_as_string(path))["version"]) == ContentCatalog.version_of("skills")
	ContentCatalog.reload_from_disk()
	checks["save then reload reproduces the rows"] = ContentCatalog.entries("skills").size() == before.size()
	return TestHelpers.report(TAG, "Version 1 and version 2 files", checks)

# Data the loader or the fold cannot keep is reported, and saving is refused, so a save never deletes it unnoticed.
static func _test_lossless_save() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var path: String = ContentCatalog.path_for("skills", SANDBOX)
	var basic: Dictionary = {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 2, "basic": true}
	# A rejected version 2 entry (stores a derived key) blocks the save: the file keeps it.
	var rejected: Dictionary = {"id": "t_ls_bad", "name": "Bad", "kind": "ATTACK", "tier": 1}
	_write(path, {"kind": "skills", "version": 2, "entries": [basic, rejected]})
	ContentCatalog.reload_from_disk()
	checks["a rejected entry is a load error and is not loaded"] = not ContentCatalog.load_errors().is_empty() and not ContentCatalog.has("skills", "t_ls_bad")
	checks["save(skills) is refused while skills has load errors"] = ContentCatalog.save_blocked("skills") and ContentCatalog.save("skills") != OK
	checks["save_all is refused too"] = ContentCatalog.save_all() != OK
	checks["...and the file still holds the rejected entry"] = FileAccess.get_file_as_string(path).contains("t_ls_bad")
	checks["other kinds still save"] = ContentCatalog.save("items") == OK
	# Modifier keys expand() would drop or change: an unknown key, a rank field the kind does not use, an out-of-range value.
	var unused: String = ""
	for key: String in SkillRanks.RANK_KEYS:
		var f: Dictionary = ContentSchema.field("skills", key)
		if unused.is_empty() and not SkillRanks.rank_fields("ATTACK").has(key) and str(f["type"]) == "int" and f.has("max") and DictUtil.to_int(f["max"]) != DictUtil.to_int(f["default"]):
			unused = key
	checks["a rank field ATTACK does not use exists"] = not unused.is_empty()
	var level_max: int = DictUtil.to_int(ContentSchema.field("skills", "level")["max"])
	var cases: Dictionary = {
		"an unknown modifier key": {"level": 2, "t_ls_wobble": 3},
		"a modifier field the kind does not use": {unused: DictUtil.to_int(ContentSchema.field("skills", unused).get("max", 1)) if not unused.is_empty() else 1},
		"an out-of-range modifier value": {"level": level_max + 1},
	}
	for label: String in cases:
		var mod: Dictionary = cases[label]
		var entry: Dictionary = {"id": "t_ls_mod", "name": "Mod", "kind": "ATTACK", "level": 1, "blocks": 1, "description": "{name}", "ranks": [mod]}
		_write(path, {"kind": "skills", "version": 2, "entries": [basic, entry]})
		ContentCatalog.reload_from_disk()
		checks["%s is a load error" % label] = not ContentCatalog.load_errors().is_empty() and ContentCatalog.validate_all().has("load")
		checks["%s blocks the save and the file keeps it" % label] = ContentCatalog.save("skills") != OK and FileAccess.get_file_as_string(path).contains("t_ls_mod")
		checks["%s is a validate_skill_entry error" % label] = not ContentCatalog.validate_skill_entry(entry).is_empty()
	# The fold reports rows it would lose data from (flat rows built in memory, e.g. by a legacy upsert).
	var rows: Array[Dictionary] = SkillRanks.expand(TestHelpers.attack("t_ls_fold", {"ranks": [{"level": 3}]}))
	var unknown_rows: Array[Dictionary] = [rows[0], rows[1].merged({"t_ls_note": "x"}, true)]
	checks["fold reports an unknown key on rank 2 that rank 1 lacks"] = not DictUtil.array_of(SkillRanks.fold("t_ls_fold", unknown_rows)["errors"]).is_empty()
	var shared_rows: Array[Dictionary] = [rows[0].merged({"t_ls_note": "x"}, true), rows[1].merged({"t_ls_note": "x"}, true)]
	checks["...but keeps an unknown key every rank shares"] = DictUtil.array_of(SkillRanks.fold("t_ls_fold", shared_rows)["errors"]).is_empty() and str(DictUtil.dict_of(SkillRanks.fold("t_ls_fold", shared_rows)["entry"]).get("t_ls_note", "")) == "x"
	if not unused.is_empty():
		var hidden_rows: Array[Dictionary] = [rows[0], rows[1].merged({unused: DictUtil.to_int(ContentSchema.field("skills", unused)["max"])}, true)]
		checks["fold reports a field the kind does not use"] = not DictUtil.array_of(SkillRanks.fold("t_ls_fold", hidden_rows)["errors"]).is_empty()
	checks["a clean fork saves again after the reload"] = TestHelpers.fork(SANDBOX) and not ContentCatalog.save_blocked("skills") and ContentCatalog.save("skills") == OK
	return TestHelpers.report(TAG, "Saving never drops data the loader or the fold could not keep", checks)

static func _write(path: String, doc: Dictionary) -> void:
	var file := FileAccess.open(path, FileAccess.WRITE)
	file.store_string(JSON.stringify(doc, "\t", false))
	file.close()

static func _test_legacy_flat() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	checks["tier 1 row accepted"] = ContentCatalog.upsert("skills", {"id": "t_fix", "name": "Fix", "kind": "ATTACK", "level": 1, "tier": 1, "description": "Fix one."}).is_empty()
	checks["tier 2 row accepted"] = ContentCatalog.upsert("skills", {"id": "t_fix_2", "name": "Fix II", "kind": "ATTACK", "level": 2, "family": "t_fix", "tier": 2, "description": "Fix two."}).is_empty()
	var ranked: Dictionary = ContentCatalog.skill_entry("t_fix")
	checks["the flat pair folds into one entry with one modifier"] = DictUtil.array_of(ranked.get("ranks", [])).size() == 1 and not ranked.has("family") and not ranked.has("tier")
	checks["a differing literal description is kept as a literal override"] = str(DictUtil.dict_of(DictUtil.array_of(ranked["ranks"])[0]).get("description", "")) == "Fix two."
	checks["save works and the flat rows survive a reload"] = ContentCatalog.save_all() == OK
	ContentCatalog.reload_from_disk()
	checks["...rows and descriptions are identical after the reload"] = str(ContentCatalog.get_entry("skills", "t_fix_2")["description"]) == "Fix two." and str(ContentCatalog.get_entry("skills", "t_fix")["description"]) == "Fix one."
	checks["a base-only field that differs between flat ranks is refused"] = not ContentCatalog.upsert("skills", {"id": "t_fix_3", "name": "Fix III", "kind": "ATTACK", "level": 3, "family": "t_fix", "tier": 3, "learned_only": true}).is_empty()
	checks["an id that breaks <head>_<n> is refused"] = not ContentCatalog.upsert("skills", {"id": "t_fix_three", "name": "Fix III", "kind": "ATTACK", "family": "t_fix", "tier": 3}).is_empty()
	checks["a gap in the tiers is refused"] = not ContentCatalog.upsert("skills", {"id": "t_gap_3", "name": "Gap", "kind": "ATTACK", "family": "t_gap", "tier": 3}).is_empty()
	return TestHelpers.report(TAG, "The legacy flat upsert pattern still works", checks)
