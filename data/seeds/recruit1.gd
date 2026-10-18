extends SceneTree
# RECRUIT1-A seed (idempotent): Bryn + five recruits, Toller's muster dialogue, priest/brawler party units,
# manifest start_party/party_max, early encounter sizing. Run: tools/fc.sh run data/seeds/art1_a.gd (manifest rows
# for the new unit art, once) then tools/fc.sh run data/seeds/recruit1.gd.
# Spec: docs/specs/RECRUIT1.md section 3. Art keys = character ids (ART1 convention, supersedes spec art "").

var failed: bool = false

const HEROES: Array = [
	["bryn_hale", "Bryn Hale", "Warden Recruit", "warrior", "", "A Warden's son from the burned post at Emberfall; carries his father's shield and his temper."],
	["sella_marsh", "Sella Marsh", "Archer of the North Wall", "ranger", "Sella Marsh, the archer on the north wall.", "Hunts ash-hares in the wastes and counts every arrow twice."],
	["kit_vane", "Kit Vane", "Oathbound Cutpurse", "rogue", "Kit Vane, the cutpurse who took the oath.", "Stole from the Wardens once, got caught, and stayed."],
	["oren_dusk", "Oren Dusk", "Hedge-Mage of the Forge", "mage", "Oren Dusk, the hedge-mage by the forge.", "Reads embers the way Tev reads smoke; nobody asks where he learned fire."],
	["alda_wren", "Alda Wren", "Hearth-Keeper", "priest", "Sister Alda Wren, the hearth-keeper.", "Tends the keep's one fire and every Warden who comes back from the pass."],
	["dagny_coal", "Dagny Coal", "Pit Fighter of Slag Gate", "brawler", "Dagny Coal, the pit fighter from Slag Gate.", "Fought for coin in the slag pits until the Horde burned them; fists are what she kept."],
]

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _char(id: String, nm: String, title: String, role: String, base: String, bg: String, dialogue: String) -> Dictionary:
	var e: Dictionary = WorldCatalog.get_own("ashlands", "characters", id)
	if e.is_empty():
		e = WorldCatalog.get_own("ashlands", "characters", "tev_quill").duplicate(true)
	e["id"] = id
	e["name"] = nm
	e["title"] = title
	e["role"] = role
	e["base_unit"] = base
	e["art"] = id
	e["background"] = bg
	e["personality"] = ""
	e["faction"] = "ember_wardens"
	e["home"] = "warden_keep"
	e["disciplines"] = []
	e["passives"] = []
	e["named_items"] = []
	e["tuning"] = {}
	e["dialogue"] = dialogue
	e["gone_flags"] = []
	return e

func _units() -> void:
	var w: Dictionary = ContentCatalog.get_entry("units", "warrior")
	var rows: Array = [
		["priest", "Priest", ["smite", "rally", "mend", "purify", "greater_mend"]],
		["brawler", "Brawler", ["body_blow", "lunge", "sweep_kick", "hamstring", "haymaker"]],
	]
	for r: Array in rows:
		var e: Dictionary = w.duplicate(true)
		e["id"] = r[0]
		e["name"] = r[1]
		e["class_id"] = r[0]
		e["description"] = "Skirmish party member: a level 5 %s." % r[1]
		e["level"] = 5
		e["tier"] = "hero"
		e["loadout"] = r[2]
		_chk("unit " + str(r[0]), ContentCatalog.upsert("units", e))

func _dialogue() -> Dictionary:
	var roll_choices: Array = []
	for h: Array in HEROES:
		if str(h[4]) == "":
			continue
		roll_choices.append({"text": h[4], "to": "joined", "effects": [
			{"effect": "recruit", "character": h[0]},
			{"effect": "add_flag", "key": "ashlands.recruits", "delta": 1}]})
	roll_choices.append({"text": "Not yet.", "to": "hello"})
	return {"id": "muster_talk", "name": "Toller at the duty roll", "speaker": "brask_toller", "start": "hello", "services": false, "tags": [],
		"nodes": [
			{"id": "hello", "text": "Toller runs a thumb down the duty roll. \"The keep can spare a blade or two, Warden. No more.\"", "choices": [
				{"text": "Who can ride with me?", "to": "roll", "requires": [{"key": "ashlands.muster_1", "value": true}, {"key": "ashlands.recruits", "op": "<", "value": 1}]},
				{"text": "The oath is sworn. Who else?", "to": "roll", "requires": [{"key": "ashlands.muster_2", "value": true}, {"key": "ashlands.recruits", "op": "<", "value": 2}]},
				{"text": "Farewell.", "to": ""}]},
			{"id": "roll", "text": "Toller nods at the yard. \"Pick one. The rest stay on the wall.\"", "choices": roll_choices},
			{"id": "joined", "text": "Toller strikes a line through a name. \"Bring them back.\"", "next": "hello"},
		]}

func _quests() -> void:
	for pair: Array in [["hold_the_courtyard", "ashlands.muster_1"], ["warden_oath", "ashlands.muster_2"]]:
		var q: Dictionary = WorldCatalog.get_own("ashlands", "quests", str(pair[0]))
		var cf: Array = q["complete_flags"]
		var has: bool = false
		for f: Variant in cf:
			if str((f as Dictionary).get("key", "")) == str(pair[1]):
				has = true
		if not has:
			cf.append({"key": pair[1], "value": true})
		_chk("quest " + str(pair[0]), WorldCatalog.upsert("ashlands", "quests", q))
	var g: Dictionary = WorldCatalog.get_own("ashlands", "quests", "cinder_gate_watch")
	var add: String = " Take someone from Toller's roll first; the pass is no place to walk alone."
	if not str(g["description"]).ends_with(add):
		g["description"] = str(g["description"]) + add
	_chk("quest cinder_gate_watch", WorldCatalog.upsert("ashlands", "quests", g))

func _marker_has(ms: Array, id: String) -> bool:
	for m: Variant in ms:
		if str((m as Dictionary)["id"]) == id:
			return true
	return false

func _locations() -> void:
	var k: Dictionary = WorldCatalog.get_own("ashlands", "locations", "warden_keep")
	var ms: Array = k["markers"]
	if not _marker_has(ms, "muster"):
		ms.append({"id": "muster", "kind": "npc", "anchor": "npc_d", "requires_flags": [], "unless_flags": [], "sets_flags": [], "character": "brask_toller"})
	_chk("loc keep", WorldCatalog.upsert("ashlands", "locations", k))
	var r: Dictionary = WorldCatalog.get_own("ashlands", "locations", "ruined_courtyard")
	r["zone_bindings"]["ruins"]["encounters"] = [["ash_scouts", 2], ["ash_straggler", 3]]
	r["zone_bindings"]["ruins"]["elites"] = []
	r["zone_bindings"]["ruins"]["elite_pct"] = 0
	var encs: Array = r["encounters"]
	if not encs.has("ash_straggler"):
		encs.append("ash_straggler")
	for m: Variant in r["markers"]:
		var md: Dictionary = m
		if str(md["id"]) == "patrol":
			var rf: Array = md["requires_flags"]
			if rf.is_empty():
				rf.append({"key": "ashlands.pass_cleared", "value": true})
	_chk("loc courtyard", WorldCatalog.upsert("ashlands", "locations", r))

func _encounters() -> void:
	var c: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "courtyard_skirmish")
	c["enemies"] = ["skarn_ironjaw", "vex_cutpurse"]
	c["description"] = "Bryn against two Horde raiders in the Ruined Courtyard."
	c["intro"] = "The Horde has come early this year. Two raiders drop from the Courtyard wall, blades out."
	_chk("enc courtyard_skirmish", WorldCatalog.upsert("ashlands", "encounters", c))
	var s: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "ash_scouts").duplicate(true)
	s["id"] = "ash_straggler"
	s["name"] = "A Horde Straggler"
	s["description"] = "A lone Horde brawler cut off from his warband."
	s["intro"] = "A straggler lurches out of the rubble."
	s["enemies"] = ["ash_brawler"]
	_chk("enc ash_straggler", WorldCatalog.upsert("ashlands", "encounters", s))
	var p: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "pass_ambush")
	p["wave_units"] = ["nok_flamebow", "gruk_smokecaller"]
	p["description"] = "A brawler and a skirmisher bait the party; at Round 1 Block 4 an archer and a smokecaller fall on them."
	_chk("enc pass_ambush", WorldCatalog.upsert("ashlands", "encounters", p))

func _init() -> void:
	ContentCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	_units()
	for h: Array in HEROES:
		_chk("char " + str(h[0]), WorldCatalog.upsert("ashlands", "characters", _char(h[0], h[1], h[2], "party", h[3], h[5], "")))
	_chk("char toller", WorldCatalog.upsert("ashlands", "characters", _char("brask_toller", "Brask Toller", "Muster Sergeant of the Wardens", "npc", "warrior", "Keeps the duty roll in his head and the names of the dead on his belt.", "muster_talk")))
	_chk("dialogue", WorldCatalog.upsert("ashlands", "dialogues", _dialogue()))
	_quests()
	_encounters()
	_locations()
	_chk("manifest", WorldCatalog.update_manifest("ashlands", {"start_party": ["bryn_hale"], "party_max": 3}))
	print("units validate_all: ", ContentCatalog.validate_all())
	print("units save: ", ContentCatalog.save_all())
	print("world validate_all: ", WorldCatalog.validate_all())
	print("world save: ", WorldCatalog.save_world("ashlands"))
	print("FAILED" if failed else "OK")
	quit()
