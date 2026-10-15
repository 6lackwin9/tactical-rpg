extends SceneTree
# C1-A1 seed (idempotent): caster skill gating, priest proficiency, wpn_tome retype.
# Run: tools/fc.sh run data/seeds/c1a1.gd

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var errs: Array[String] = []
	for sid: String in ["arcane_bolt", "hex"]:
		var s: Dictionary = ContentCatalog.get_entry("skills", sid).duplicate(true)
		s["weapon_types"] = ["staff", "wand"]
		errs.append_array(ContentCatalog.upsert("skills", s))
	var pr: Dictionary = ContentCatalog.get_entry("classes", "priest").duplicate(true)
	var prof: Dictionary = pr["proficiency"]
	prof.erase("shield")
	prof["tome"] = {"level": 3, "scaling": 1.5}
	prof["orb"] = {"level": 2, "scaling": 1.25}
	errs.append_array(ContentCatalog.upsert("classes", pr))
	var t: Dictionary = ContentCatalog.get_entry("items", "wpn_tome").duplicate(true)
	t["slot"] = "OFF_HAND"
	t["weapon_type"] = "tome"
	t["off_hand_category"] = "tome"
	t["base_power"] = 0
	t["base_range"] = 1
	t["scaling_stat"] = "INT"
	t["required_stats"] = {"KNOW": 11}
	t["stat_bonuses"] = {"KNOW": 1}
	t["starter_stash"] = true
	t["description"] = "A bound book of rites held in the off hand. Strengthens healing and the grip of curses."
	errs.append_array(ContentCatalog.upsert("items", t))
	print("upsert errors: ", errs)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
