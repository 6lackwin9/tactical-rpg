extends SceneTree
# CLASS1-M migration seed (CLASS1-L writes and tests it on a sandbox fork; the asset lane RUNS it on the shipped data).
# Idempotent. Converts every class `skill_path` from the old list [{level, skill}] to the dict {family: level} in list order
# (basics dropped; the one-release converter in ContentSchema.normalize already does this on load, the seed re-upserts every
# class and saves so the FILE carries the new shape). save_all also writes the new unit field `heal_below` (default 0.7) into
# units.json. Verifies FORGE1_CLASSES acceptance 7: the new dict has exactly the old non-basic skills with the same levels,
# class default loadouts at levels 1-10 are identical to the old list-order rule (computed from the stored list), validate_all
# is empty. Basics: not a before/after comparison (the loader already converts on load); it checks the rule that held before
# CLASS1 too (SKILL1 grant): every class-derived unit owns every basic at every level, path basics or not.
# Run: tools/fc.sh run data/seeds/class1_migrate.gd   (saves only when the report has no problems)

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var report: Dictionary = migrate()
	for key: Variant in report:
		print("%s: %s" % [str(key), str(report[key])])
	if DictUtil.to_bool(report["ok"]):
		print("save_all: ", ContentCatalog.save_all())
		ContentCatalog.reload_from_disk()
		print("file shape after save: ", "dict" if file_shape() == "dict" else file_shape())
		print("validate_all after reload: ", ContentCatalog.validate_all())
	else:
		print("NOT SAVED: the report lists problems")
	quit()

# "list", "dict", "mixed" or "none": the skill_path shape stored in classes.json under the CURRENT root.
static func file_shape() -> String:
	var lists: int = 0
	var dicts: int = 0
	for raw: Dictionary in raw_classes():
		var path: Variant = raw.get("skill_path", null)
		if path is Array:
			lists += 1
		elif path is Dictionary:
			dicts += 1
	if lists > 0 and dicts > 0:
		return "mixed"
	if lists > 0:
		return "list"
	return "dict" if dicts > 0 else "none"

# The class entries exactly as stored on disk (before the loader's converter).
static func raw_classes() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var text: String = FileAccess.get_file_as_string(ContentCatalog.path_for("classes"))
	var json := JSON.new()
	if text.is_empty() or json.parse(text) != OK:
		return out
	for raw: Variant in DictUtil.array_of(DictUtil.dict_of(json.data).get("entries", [])):
		if raw is Dictionary:
			out.append(DictUtil.dict_of(raw))
	return out

# The stored path as old-style steps [{level, skill}] in stored order (a dict file reads back in its key order).
static func stored_steps(raw: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var path: Variant = raw.get("skill_path", [])
	if path is Array:
		var list: Array = path
		for step: Variant in list:
			var s: Dictionary = DictUtil.dict_of(step)
			out.append({"level": DictUtil.to_int(s.get("level", 1), 1), "skill": str(s.get("skill", ""))})
	elif path is Dictionary:
		var d: Dictionary = path
		for key: Variant in d:
			out.append({"level": DictUtil.to_int(d[key], 1), "skill": str(key)})
	return out

# The pre-CLASS1 default loadout rule: the first MAX_CLASS_SLOTS non-basic families of the list (list order) reached by `level`.
static func old_default_loadout(steps: Array[Dictionary], level: int) -> Array[String]:
	var owned: Array[String] = []
	for step: Dictionary in steps:
		if DictUtil.to_int(step["level"]) <= level:
			owned.append(str(step["skill"]))
	return ContentFactory.families_of(owned).slice(0, StatRules.MAX_CLASS_SLOTS)

# "unit:level" of every class-derived unit entry that misses a basic skill at a level 1..MAX_LEVEL (the SKILL1 grant rule).
static func basics_missing() -> Array[String]:
	var out: Array[String] = []
	for u: Dictionary in ContentCatalog.entries("units"):
		if str(u.get("stat_mode", "derived")) != "derived" or ContentFactory.class_of(u) == null:
			continue
		for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
			var probe: Dictionary = u.duplicate(true)
			probe["level"] = lvl
			probe["spend"] = {}
			var c: Combatant = ContentFactory.unit(probe, "", 12)
			for basic_id: String in SkillCatalog.basic_ids():
				if not c.owns_skill(basic_id):
					out.append("%s:%d" % [str(u["id"]), lvl])
					break
	return out

# Migrates the CURRENT catalog root (the shipped catalog from _init, a sandbox fork in tests). Returns the report:
# classes, file_shape_before, basics_dropped, upsert_errors, path_diffs, order_diffs, loadout_diffs, basics_missing, validate_all, ok.
static func migrate() -> Dictionary:
	var report: Dictionary = {"classes": 0, "file_shape_before": file_shape(), "basics_dropped": [], "upsert_errors": [], "path_diffs": [],
		"order_diffs": [], "loadout_diffs": [], "basics_missing": []}
	var old_steps: Dictionary = {}   # class id -> stored steps
	for raw: Dictionary in raw_classes():
		old_steps[str(raw.get("id", ""))] = stored_steps(raw)
	var errors: Array = report["upsert_errors"]
	for entry: Dictionary in ContentCatalog.entries("classes"):
		report["classes"] = DictUtil.to_int(report["classes"]) + 1
		errors.append_array(ContentCatalog.upsert("classes", entry.duplicate(true)))
	for entry: Dictionary in ContentCatalog.entries("classes"):
		var id: String = str(entry["id"])
		var steps: Array[Dictionary] = []
		steps.assign(DictUtil.array_of(old_steps.get(id, [])))
		var path: Dictionary = DictUtil.dict_of(entry["skill_path"])
		var expected: Dictionary = {}
		var expected_order: Array[String] = []
		for step: Dictionary in steps:
			var skill_id: String = str(step["skill"])
			if DictUtil.to_bool(ContentCatalog.get_entry("skills", skill_id).get("basic", false)):
				var dropped: Array = report["basics_dropped"]
				dropped.append("%s:%s" % [id, skill_id])
				continue
			if not expected.has(skill_id):
				expected_order.append(skill_id)
				expected[skill_id] = DictUtil.to_int(step["level"])
			else:
				expected[skill_id] = mini(DictUtil.to_int(expected[skill_id]), DictUtil.to_int(step["level"]))
		if path != expected:
			var diffs: Array = report["path_diffs"]
			diffs.append("%s: %s != %s" % [id, str(path), str(expected)])
		var keys: Array[String] = []
		for key: Variant in path:
			keys.append(str(key))
		if keys != expected_order:
			var order_diffs: Array = report["order_diffs"]
			order_diffs.append("%s: %s != %s" % [id, str(keys), str(expected_order)])
		var arch: ClassArchetype = ContentFactory.class_archetype(entry)
		for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
			var now: Array[String] = ContentFactory._strings(ContentFactory.loadout_of({}, arch, lvl)["class"])
			var then: Array[String] = old_default_loadout(steps, lvl)
			if now != then:
				var loadout_diffs: Array = report["loadout_diffs"]
				loadout_diffs.append("%s Lv%d: %s != %s" % [id, lvl, str(now), str(then)])
	report["basics_missing"] = basics_missing()
	report["validate_all"] = ContentCatalog.validate_all()
	report["ok"] = errors.is_empty() and DictUtil.array_of(report["path_diffs"]).is_empty() and DictUtil.array_of(report["order_diffs"]).is_empty() \
		and DictUtil.array_of(report["loadout_diffs"]).is_empty() and DictUtil.array_of(report["basics_missing"]).is_empty() and DictUtil.dict_of(report["validate_all"]).is_empty()
	return report
