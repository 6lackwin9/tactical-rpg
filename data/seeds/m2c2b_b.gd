extends SceneTree
# M2C2B phase B seed (idempotent): C1 caster families (hex, arcane_bolt, arc_lance, firebomb), C1-A2 orb/tome items and
# the mage/priest default off-hand items, class paths.
# Run: tools/fc.sh run data/seeds/m2c2b_b.gd
const Lib = preload("res://data/seeds/m2c2b_lib.gd")

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var errs: Array[String] = []
	errs.append_array(Lib.attack_family({"id": "hex", "name": "Hex", "ap": [2, 3, 4], "blocks": 2, "power": [0.6, 0.75, 0.9], "rmin": 2, "rmax": 3, "status": "stagger", "pot": [90, 95, 95], "wt": ["mace", "staff", "wand"], "minp": 2}))
	errs.append_array(Lib.attack_family({"id": "arcane_bolt", "name": "Arcane Bolt", "ap": [1, 2, 3], "blocks": 2, "power": [1.1, 1.4, 1.65], "wt": ["staff", "wand"], "minp": 1, "text": "A bolt of raw force."}))
	errs.append_array(Lib.attack_family({"id": "arc_lance", "name": "Arc Lance", "ap": [2, 3, 4], "blocks": 2, "power": [0.9, 1.1, 1.35], "area": "LINE", "wt": ["staff", "wand"], "minp": 1, "text": "Aim at an adjacent tile; hits the 3 tiles in a line."}))
	errs.append_array(Lib.attack_family({"id": "firebomb", "name": "Firebomb", "ap": [3, 4, 5], "blocks": 3, "power": [0.6, 0.75, 0.9], "area": "BURST", "rmin": 2, "rmax": 2, "wt": ["staff", "wand"], "minp": 1, "text": "Hits a 3x3 area around a tile 2 away."}))
	# C1-A2 items
	var tome: Dictionary = ContentCatalog.get_entry("items", "wpn_tome").duplicate(true)
	tome["weight"] = 1.2
	tome["value"] = 55
	tome["effects"] = [{"type": "skill_potency", "group": "heal", "value": 0.10}, {"type": "status_potency", "value": 5}]
	errs.append_array(ContentCatalog.upsert("items", tome))
	var orb: Dictionary = tome.duplicate(true)
	orb["id"] = "wpn_orb"
	orb["name"] = "Focus Orb"
	orb["description"] = "A glass sphere held in the off hand. Sharpens spells and lets them reach one tile further."
	orb["weight"] = 0.8
	orb["value"] = 60
	orb["starter_stash"] = false
	orb["weapon_type"] = "orb"
	orb["off_hand_category"] = "orb"
	orb["required_stats"] = {"INT": 12}
	orb["stat_bonuses"] = {}
	orb["effects"] = [{"type": "skill_potency", "group": "spell", "value": 0.10}, {"type": "spell_reach", "value": 1}]
	errs.append_array(ContentCatalog.upsert("items", orb))
	for pair: Array in [["mage", "wpn_orb"], ["priest", "wpn_tome"]]:
		var c: Dictionary = ContentCatalog.get_entry("classes", pair[0]).duplicate(true)
		c["off_hand"] = pair[1]
		errs.append_array(ContentCatalog.upsert("classes", c))
	errs.append_array(Lib.apply_paths())
	Lib.finish(errs, self)
