extends SceneTree
# BAL2-A asset seed (idempotent): growth, warrior gear (D1), L1 paths, party loadouts, goblin shaman, brawler unarmed.
# Authoring only; every change goes through ContentCatalog.upsert. Run: tools/fc.sh run data/seeds/bal2.gd

const APPLY_A2: bool = true   # warrior kite shield + D1 item numbers (user gear budget answer 2026-10-17)
const APPLY_A6: bool = true   # brawler bare hands (needs BAL2-L commit 5)

var errs: Array[String] = []

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	_a1_growth()
	if APPLY_A2:
		_a2_warrior()
	_a3_paths()
	_a4_loadouts()
	_a5_shaman()
	if APPLY_A6:
		_a6_brawler()
	ContentCatalog.end_system_edit()
	for e: String in errs:
		print("ERROR: ", e)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()

func _edit(kind: String, id: String, changes: Dictionary) -> void:
	var e: Dictionary = ContentCatalog.get_entry(kind, id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING %s %s" % [kind, id])
		return
	for k: String in changes.keys():
		e[k] = changes[k]
	errs.append_array(ContentCatalog.upsert(kind, e))

func _path(rows: Array) -> Array:
	var out: Array = []
	for r: Array in rows:
		out.append({"level": r[0], "skill": r[1]})
	return out

func _a1_growth() -> void:
	_edit("classes", "warrior", {"growth": {"CONS": 2, "STR": 2, "PERC": 1}})
	_edit("classes", "ranger", {"growth": {"DEX": 2, "PERC": 1, "STR": 1, "CONS": 1}})
	_edit("classes", "rogue", {"growth": {"DEX": 2, "PERC": 1, "STR": 1, "CONS": 1}})
	_edit("classes", "mage", {"growth": {"INT": 2, "CONS": 1, "KNOW": 1, "PERC": 1}})

func _a2_warrior() -> void:
	_edit("classes", "warrior", {
		"off_hand": "shd_kite",
		"description": "Tank: huge constitution, kite shield and plate; a wall that holds the line and hits lightly.",
	})
	_edit("items", "shd_kite", {"mitigation": 6})
	_edit("items", "arm_plate", {"mitigation": 8})

func _a3_paths() -> void:
	_edit("classes", "ranger", {"skill_path": _path([
		[1, "aimed_shot"], [3, "pinning_shot"], [3, "leap_back"], [5, "barbed_arrow"],
		[5, "volley"], [5, "deadeye"], [5, "focus"]])})
	_edit("classes", "brawler", {"skill_path": _path([
		[1, "body_blow"], [3, "lunge"], [3, "sweep_kick"], [4, "hamstring"],
		[5, "haymaker"], [5, "charge_in"], [5, "cyclone_kick"]])})

func _a4_loadouts() -> void:
	_edit("units", "warrior", {"loadout": ["taunt_shout", "shield_bash", "riposte", "intimidate", "shield_wall"]})
	_edit("units", "ranger", {"loadout": ["aimed_shot", "pinning_shot", "leap_back", "barbed_arrow", "deadeye"]})
	_edit("units", "rogue", {"loadout": ["quick_stab", "pounce", "lacerate", "backflip", "hamstring"]})
	_edit("units", "mage", {"loadout": ["arcane_bolt", "arc_lance", "firebomb", "glacial_burst", "focus"]})

func _a5_shaman() -> void:
	_edit("units", "goblin_shaman", {
		"main_hand": "wpn_cleric_mace",
		"off_hand": "none",
		"body": "arm_robe",
		"pouch": ["none"],
		"description": "Smites and mends its allies.",
	})

func _a6_brawler() -> void:
	_edit("classes", "brawler", {
		"main_hand": "",
		"description": "Bruiser: bare fists and a thick hide; leaps into a crowd and fights several foes at once.",
	})
