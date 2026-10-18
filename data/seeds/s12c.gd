extends SceneTree
# S12C-A asset seed (idempotent): three combo families (feint, crushing_blow, opportunist) on class paths at levels 6-7.
# Authoring only; every change goes through ContentCatalog.upsert_skill / upsert. Run: tools/fc.sh run data/seeds/s12c.gd
# Numbers are PLACEHOLDER starting values for the balance pass (docs/specs/S12C_A.md).

var errs: Array[String] = []

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	_skills()
	_paths()
	ContentCatalog.end_system_edit()
	for e: String in errs:
		print("ERROR: ", e)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()

# learned_only keeps them out of unrestricted hand-built units; class-derived units own them via the path.
func _skill(entry: Dictionary) -> void:
	entry["learned_only"] = true
	errs.append_array(ContentCatalog.upsert_skill(entry))

func _skills() -> void:
	_skill({
		"id": "feint", "name": "Feint", "kind": "ATTACK", "level": 2, "blocks": 1,
		"description": "{name}: {power} power, {flow}. Needs {weapons} training {proficiency}.",
		"weapon_types": ["dagger"], "min_proficiency": 1,
		"power_mult": 0.7, "ends_in_stance": "DEFENSIVE",
		"ranks": [{"level": 3, "power_mult": 0.9}, {"level": 4, "power_mult": 1.05}],
	})
	_skill({
		"id": "crushing_blow", "name": "Crushing Blow", "kind": "ATTACK", "level": 2, "blocks": 2,
		"description": "{name}: {power} power, {disrupt}. Needs {weapons} training {proficiency}.",
		"weapon_types": ["unarmed", "fist"], "min_proficiency": 1,
		"power_mult": 1.0, "disrupt": "STANCE_BREAK",
		"ranks": [{"level": 3, "power_mult": 1.25}, {"level": 4, "power_mult": 1.5}],
	})
	_skill({
		"id": "opportunist", "name": "Opportunist", "kind": "ATTACK", "level": 2, "blocks": 1,
		"description": "{name}: {power} power, {exploit}. Needs {weapons} training {proficiency}.",
		"weapon_types": ["dagger"], "min_proficiency": 1,
		"power_mult": 0.8,
		"exploit": [{"status": "expose", "power_mult": 1.3}, {"status": "slow", "power_mult": 1.2}],
		"exploit_consume": false,
		"ranks": [{"level": 3, "power_mult": 1.0}, {"level": 4, "power_mult": 1.2}],
	})

func _paths() -> void:
	_append_path("rogue", {"feint": 6, "opportunist": 7})
	_append_path("brawler", {"crushing_blow": 6})

func _append_path(class_id: String, steps: Dictionary) -> void:
	var e: Dictionary = ContentCatalog.get_entry("classes", class_id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING classes %s" % class_id)
		return
	var path: Dictionary = DictUtil.dict_of(e.get("skill_path", {}))
	for s: String in steps:
		path[s] = steps[s]
	e["skill_path"] = path
	errs.append_array(ContentCatalog.upsert("classes", e))
