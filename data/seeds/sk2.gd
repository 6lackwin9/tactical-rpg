extends SceneTree
# SK2-A asset seed (idempotent): advanced families, riposte tier offsets, warrior riposte at level 3, unlock passives.
# Authoring only; every change goes through ContentCatalog.upsert.
# Run: tools/fc.sh run data/seeds/sk2.gd

const ADVANCED_FAMILIES: Array[String] = [
	"shield_wall", "deadeye", "greater_mend", "renewal", "judgement", "charge_in",
	"cyclone_kick", "lacerate", "assassinate", "glacial_burst", "inferno",
]

const WARRIOR_PATH_ORDER: Array[Array] = [
	[1, "taunt_shout"], [3, "shield_bash"], [3, "riposte"], [4, "intimidate"], [5, "sunder"], [5, "shield_wall"],
]

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var errs: Array[String] = []

	# 1. advanced flag on every tier of the eleven families (atomic per family: the flag must match across all tiers).
	for fid: String in ADVANCED_FAMILIES:
		var rows: Array = []
		for i: int in 3:
			var sid: String = fid if i == 0 else "%s_%d" % [fid, i + 1]
			var s: Dictionary = ContentCatalog.get_entry("skills", sid).duplicate(true)
			if s.is_empty():
				errs.append("MISSING skill %s" % sid)
				continue
			s["advanced"] = true
			rows.append(s)
		errs.append_array(ContentCatalog.upsert_many("skills", rows))

	# 2. riposte tiers 2 and 3 open with their tier 1 (offset 0).
	for sid: String in ["riposte_2", "riposte_3"]:
		var s: Dictionary = ContentCatalog.get_entry("skills", sid).duplicate(true)
		if s.is_empty():
			errs.append("MISSING skill %s" % sid)
			continue
		s["unlock_offset"] = 0
		errs.append_array(ContentCatalog.upsert("skills", s))

	# 3. warrior path: riposte moves from level 5 to 3; basics keep their slots, non-basic rows take the new order.
	var w: Dictionary = ContentCatalog.get_entry("classes", "warrior").duplicate(true)
	var old_path: Array = w["skill_path"]
	var new_path: Array = []
	var slot_rows: Array[Dictionary] = []
	for pr: Array in WARRIOR_PATH_ORDER:
		slot_rows.append({"level": pr[0], "skill": pr[1]})
	var next_slot: int = 0
	for row: Variant in old_path:
		var rd: Dictionary = row
		var sk: Dictionary = ContentCatalog.get_entry("skills", String(rd["skill"]))
		if bool(sk.get("basic", false)):
			new_path.append(rd)
		else:
			if next_slot >= slot_rows.size():
				errs.append("WARRIOR path has an unexpected non-basic row %s" % rd["skill"])
				break
			new_path.append(slot_rows[next_slot])
			next_slot += 1
	if next_slot != slot_rows.size():
		errs.append("WARRIOR path non-basic count mismatch (expected %d)" % slot_rows.size())
	else:
		w["skill_path"] = new_path
		errs.append_array(ContentCatalog.upsert("classes", w))

	# 4. unlock passive templates (no source grants them yet).
	var passives: Array[Dictionary] = [
		{
			"id": "field_medic", "name": "Field Medic",
			"description": "Opens healing families one level earlier.",
			"effects": [{"type": "unlock", "group": "heal", "levels": 1}],
		},
		{
			"id": "arcane_study", "name": "Arcane Study",
			"description": "Opens spell families one level earlier.",
			"effects": [{"type": "unlock", "group": "spell", "levels": 1}],
		},
		{
			"id": "riposte_drill", "name": "Riposte Drill",
			"description": "Opens Riposte two levels earlier (level 1 for a warrior).",
			"effects": [{"type": "unlock", "family": "riposte", "levels": 2}],
		},
	]
	for p: Dictionary in passives:
		errs.append_array(ContentCatalog.upsert("passives", p))

	print("upsert errors: ", errs)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
