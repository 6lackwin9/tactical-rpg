extends SceneTree
# S12S-A asset seed (idempotent): five status families (slow, expose, poison, fortify, regen) on class paths at level 6.
# Authoring only; every change goes through ContentCatalog.upsert_skill / upsert. Run: tools/fc.sh run data/seeds/s12s.gd
# Numbers are PLACEHOLDER starting values for the balance pass (docs/specs/S12S_A.md).

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

func _skill(entry: Dictionary) -> void:
	entry["origin"] = ContentSchema.ORIGIN_USER
	errs.append_array(ContentCatalog.upsert_skill(entry))

func _skills() -> void:
	_skill({
		"id": "crippling_shot", "name": "Crippling Shot", "kind": "ATTACK", "level": 2, "blocks": 2,
		"description": "{name}: {power} power, {potency} to Slow for {rounds}. Needs {weapons} training {proficiency}.",
		"weapon_types": ["bow", "crossbow"], "min_proficiency": 1,
		"power_mult": 0.7, "status_effect": "slow", "status_potency": 75, "status_rounds": 2,
		"ranks": [
			{"level": 3, "power_mult": 0.9, "status_potency": 80},
			{"level": 4, "power_mult": 1.05, "status_potency": 85, "status_rounds": 3},
		],
	})
	_skill({
		"id": "rend_armor", "name": "Rend Armor", "kind": "ATTACK", "level": 2, "blocks": 2,
		"description": "{name}: {power} power, {potency} to Expose (-{magnitude}) for {rounds}. Needs {weapons} training {proficiency}.",
		"weapon_types": ["sword", "axe", "mace", "hammer"], "min_proficiency": 1,
		"power_mult": 0.8, "status_effect": "expose", "status_potency": 70, "status_rounds": 2, "status_magnitude": 6,
		"ranks": [
			{"level": 3, "power_mult": 1.0, "status_potency": 75, "status_magnitude": 8},
			{"level": 4, "power_mult": 1.2, "status_potency": 80, "status_magnitude": 10, "status_rounds": 3},
		],
	})
	_skill({
		"id": "envenom", "name": "Envenom", "kind": "ATTACK", "level": 2, "blocks": 1,
		"description": "{name}: {power} power, {potency} to Poison ({magnitude} per round, stacks) for {rounds}. Needs {weapons} training {proficiency}.",
		"weapon_types": ["dagger"], "min_proficiency": 1,
		"power_mult": 0.6, "status_effect": "poison", "status_potency": 80, "status_rounds": 3, "status_magnitude": 4,
		"ranks": [
			{"level": 3, "power_mult": 0.75, "status_potency": 85, "status_magnitude": 5},
			{"level": 4, "power_mult": 0.9, "status_potency": 90, "status_magnitude": 6},
		],
	})
	_skill({
		"id": "aegis", "name": "Aegis", "kind": "SUPPORT", "level": 2, "blocks": 1,
		"description": "{name}: grants Fortify (+{magnitude}) for {rounds} starting next round.",
		"target_mode": "ALLY_OR_SELF", "heal_base": 0, "heal_mult": 0.0,
		"grant_status": "fortify", "status_rounds": 2, "status_magnitude": 6,
		"ranks": [
			{"level": 3, "status_magnitude": 8},
			{"level": 4, "status_magnitude": 10, "status_rounds": 3},
		],
	})
	_skill({
		"id": "rejuvenate", "name": "Rejuvenate", "kind": "SUPPORT", "level": 2, "blocks": 2,
		"description": "{name}: grants Regen ({magnitude} per round) for {rounds} starting next round.",
		"target_mode": "ALLY", "heal_base": 0, "heal_mult": 0.0,
		"grant_status": "regen", "status_rounds": 2, "status_magnitude": 4,
		"ranks": [
			{"level": 3, "status_magnitude": 5, "status_rounds": 3},
			{"level": 4, "status_magnitude": 6, "status_rounds": 3},
		],
	})

func _paths() -> void:
	_append_path("ranger", ["crippling_shot"])
	_append_path("warrior", ["rend_armor"])
	_append_path("rogue", ["envenom"])
	_append_path("priest", ["aegis", "rejuvenate"])

func _append_path(class_id: String, skills: Array[String]) -> void:
	var e: Dictionary = ContentCatalog.get_entry("classes", class_id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING classes %s" % class_id)
		return
	var path: Dictionary = DictUtil.dict_of(e.get("skill_path", {}))
	for s: String in skills:
		path[s] = 6
	e["skill_path"] = path
	errs.append_array(ContentCatalog.upsert("classes", e))
