extends SceneTree
# CAMP1-A2 asset seed (idempotent): Ashlands campaign "The Long Dry" story data (docs/specs/CAMP1.md sections 3-6).
# Needs data/seeds/camp1_unit.gd first (goblin_warlord). Run: tools/fc.sh run data/seeds/camp1_story.gd

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _up(world: String, kind: String, e: Dictionary) -> void:
	_chk(kind + " " + str(e["id"]), WorldCatalog.upsert(world, kind, e))

func _f(key: String) -> Array:
	return [{"key": key, "value": true}]

func _own(world: String, kind: String, id: String) -> Dictionary:
	var e: Dictionary = WorldCatalog.get_own(world, kind, id)
	if e.is_empty():
		failed = true
		print("ERROR missing ", world, ":", kind, ":", id)
	return e

func _without(arr: Array, key: String, val: String) -> Array:
	var out: Array = []
	for x: Variant in arr:
		if not (x is Dictionary and str((x as Dictionary).get(key, "")) == val):
			out.append(x)
	return out

func _marker(id: String, kind: String, anchor: String, extra: Dictionary) -> Dictionary:
	var m: Dictionary = {"id": id, "kind": kind, "anchor": anchor, "requires_flags": [], "unless_flags": [], "sets_flags": []}
	m.merge(extra, true)
	if anchor == "":
		m.erase("anchor")
	return m

func _quest(id: String, nm: String, desc: String, giver: String, turn_in: String, region: String, requires: Array, req_flags: Array, template: String, steps: Array, auto: bool, done: String, variants: Array, pool: Array, rep: Dictionary, reward: String, links: Dictionary) -> Dictionary:
	return {"id": id, "name": nm, "description": desc, "giver": giver, "faction": "ember_wardens", "region": region,
		"requires": requires, "steps": steps, "reward": reward, "template": template, "turn_in": turn_in,
		"requires_flags": req_flags, "complete_flags": _f(done), "auto_start": auto, "links": links, "variants": variants,
		"loot_pool": pool, "loot_picks": 1, "reputation": rep, "tags": []}

func _enc(id: String, nm: String, desc: String, intro: String, enemies: Array, wave: Array, label: String) -> Dictionary:
	return {"id": id, "name": nm, "description": desc, "intro": intro, "location": "ashmaw_camp", "zone": "desert_outpost",
		"arena": "CLOSED", "exits": [[11, 4]], "max_rounds": 8, "total_blocks": 6, "ap_regen": 4, "unit_ap": 12, "rng_seed": 1337,
		"party": ["mara_voss", "tev_quill"], "enemies": enemies, "wave_units": wave, "wave_round": 2, "wave_block": 1,
		"wave_label": label, "tags": []}

func _enemy(id: String, nm: String, title: String, base: String, bg: String, dlg: String, gone: Array) -> Dictionary:
	return {"id": id, "name": nm, "title": title, "role": "enemy", "base_unit": base, "art": "", "background": bg,
		"personality": "", "faction": "ashen_horde", "home": "", "disciplines": [], "passives": [], "named_items": [],
		"tuning": {}, "dialogue": dlg, "gone_flags": gone, "tags": []}

func _quests() -> void:
	_up("ashlands", "quests", _quest("steel_for_the_gate", "Steel for the Gate",
		"The Cinder Gate will not hold another Long Dry on timber. Mara sends you up the south road to Slag Gate for Orsa Tallow's steel.",
		"mara_voss", "hest_the_quartermaster", "ashen_marches", ["warden_oath"], [], "signal_and_return",
		[{"text": "Load Orsa's steel at Slag Gate", "flag": "ashlands.steel_loaded", "value": true}, {"text": "Bring the steel to Warden Keep", "location": "warden_keep"}],
		true, "ashlands.steel_delivered",
		[{"id": "mara_fallen", "when_flags": _f("ashlands.mara_fallen"), "unless_flags": [], "set": {"giver": "tev_quill"}}],
		["keep_stores", "keep_ring"], {"ember_wardens": 10}, "Hest's thanks and a pick from the stores.",
		{"follows": ["warden_oath"], "leads_to": ["keep_one_fire_lit"], "related_characters": ["mara_voss", "hest_the_quartermaster"]}))
	_up("ashlands", "quests", _quest("keep_one_fire_lit", "Keep One Fire Lit",
		"Grakka Ashmaw has gathered the warbands past the Cinder Gate. Break him and the Horde scatters.",
		"mara_voss", "", "cinder_pass", ["steel_for_the_gate"], _f("ashlands.steel_spent"), "reach_and_clear",
		[{"text": "Find the Ashmaw Camp", "location": "ashmaw_camp"}, {"text": "Break Grakka Ashmaw", "encounter": "ashmaw_stand"}],
		false, "ashlands.horde_broken",
		[{"id": "mara_fallen", "when_flags": _f("ashlands.mara_fallen"), "unless_flags": [], "set": {"giver": "tev_quill", "turn_in": "tev_quill"}},
		{"id": "steel_to_gate", "when_flags": _f("ashlands.steel_to_gate"), "unless_flags": [], "set": {"steps": {"1": {"text": "Break Grakka; the barred gate cut off his flankers", "encounter": "ashmaw_stand_barred"}}}}],
		["chieftains_torc", "keep_ring", "rallying_cry", "wardens_drill"], {"ember_wardens": 15, "ashen_horde": -20},
		"The Wardens' gratitude and the chieftain's torc.",
		{"follows": ["steel_for_the_gate"], "related_characters": ["mara_voss", "grakka_ashmaw"], "related_locations": ["ashmaw_camp"]}))
	var h: Dictionary = _own("ashlands", "quests", "hold_the_courtyard")
	h["description"] = "The Long Dry has come early, and the Horde with it. Mara needs the Courtyard held while the keep's gates are shored up."
	_up("ashlands", "quests", h)
	var o: Dictionary = _own("ashlands", "quests", "warden_oath")
	var links: Dictionary = o["links"]
	var lt: Array = links.get("leads_to", [])
	if not lt.has("steel_for_the_gate"):
		lt.append("steel_for_the_gate")
	links["leads_to"] = lt
	_up("ashlands", "quests", o)

func _choice(text: String, to: String, extra: Dictionary) -> Dictionary:
	var c: Dictionary = {"text": text, "to": to}
	c.merge(extra, true)
	return c

func _add_choices(world: String, did: String, node: String, add: Array) -> void:
	var d: Dictionary = _own(world, "dialogues", did)
	for n: Variant in d["nodes"]:
		var nd: Dictionary = n as Dictionary
		if str(nd["id"]) == node:
			var cs: Array = nd.get("choices", [])
			for a: Variant in add:
				var keep: Array = []
				for c: Variant in cs:
					if str((c as Dictionary)["to"]) != str((a as Dictionary)["to"]):
						keep.append(c)
				cs = keep
				cs.append(a)
			nd["choices"] = cs
	_up(world, "dialogues", d)

func _dialogues() -> void:
	var qs: Dictionary = {"kind": "quest", "quest": "steel_for_the_gate", "status": "completed"}
	var kq: Dictionary = {"kind": "quest", "quest": "keep_one_fire_lit"}
	var mara: Dictionary = _own("ashlands", "dialogues", "mara_talk")
	var nodes: Array = mara["nodes"]
	for id: String in ["steel_choice", "gate_barred", "party_armed", "end_gate", "end_arms"]:
		nodes = _without(nodes, "id", id)
	nodes.append({"id": "steel_choice", "text": "Mara weighs a bar of slag-steel. \"Enough for one thing done well. The gate, or your hands?\"", "choices": [
		_choice("Bar the Cinder Gate.", "gate_barred", {"effects": [
			{"effect": "set_flag", "key": "ashlands.steel_spent", "value": true}, {"effect": "set_flag", "key": "ashlands.steel_to_gate", "value": true},
			{"effect": "add_flag", "key": "rep.ashlands.ember_wardens", "delta": 10}]}),
		_choice("Arm the party.", "party_armed", {"effects": [
			{"effect": "set_flag", "key": "ashlands.steel_spent", "value": true}, {"effect": "set_flag", "key": "ashlands.steel_to_arms", "value": true},
			{"effect": "give", "items": {"arm_plate": 1}}]}),
		_choice("Let me think.", "hello", {})]})
	nodes.append({"id": "gate_barred", "text": "\"Then the gorge is shut behind you. Whatever waits in the wastes waits alone.\"", "next": "hello"})
	nodes.append({"id": "party_armed", "text": "\"Then go out sharp. Hest has the rest on his counter.\"", "next": "hello"})
	nodes.append({"id": "end_gate", "text": "\"One fire lit, and a door that holds. The Long Dry is over. This year.\"", "next": "hello"})
	nodes.append({"id": "end_arms", "text": "\"The wastes will remember that steel. Rest; you have earned the hearth.\"", "next": "hello"})
	mara["nodes"] = nodes
	_up("ashlands", "dialogues", mara)
	var done: Dictionary = {"kind": "quest", "quest": "keep_one_fire_lit", "status": "completed"}
	_add_choices("ashlands", "mara_talk", "hello", [
		_choice("The steel is here. Where does it go?", "steel_choice", {"requires": [qs], "unless": _f("ashlands.steel_spent")}),
		_choice("It's over. The Horde is scattered.", "end_gate", {"requires": [done, {"key": "ashlands.steel_to_gate", "value": true}],
			"unless": _f("ashlands.campaign_done"), "effects": [{"effect": "set_flag", "key": "ashlands.campaign_done", "value": true}]}),
		_choice("It's over. The Horde is scattered.", "end_arms", {"requires": [done, {"key": "ashlands.steel_to_arms", "value": true}],
			"unless": _f("ashlands.campaign_done"), "effects": [{"effect": "set_flag", "key": "ashlands.campaign_done", "value": true}]})])
	_add_choices("ashlands", "tev_talk", "hello", [
		_choice("Where do we get steel?", "steel", {"requires": _f("ashlands.oath_sworn"), "unless": _f("ashlands.steel_delivered")})])
	var tev: Dictionary = _own("ashlands", "dialogues", "tev_talk")
	var tn: Array = _without(tev["nodes"], "id", "steel")
	tn.append({"id": "steel", "text": "\"Slag Gate, up the south road. Orsa sells to us at cost. Take the signpost to the world map.\"", "next": "hello"})
	tev["nodes"] = tn
	_up("ashlands", "dialogues", tev)
	_up("ashlands", "dialogues", {"id": "grakka_parley", "name": "Grakka at the fire", "speaker": "grakka_ashmaw", "start": "hello", "nodes": [
		{"id": "hello", "text": "A goblin in a crown of rusted nails grins over the fire. \"Come to beg for your little flame?\"", "choices": [
			_choice("Leave the pass and live.", "scorn", {}),
			_choice("Then we fight.", "", {"requires": [{"kind": "quest", "quest": "keep_one_fire_lit", "status": "active"}], "unless": _f("ashlands.steel_to_gate"), "effects": [{"effect": "fight", "encounter": "ashmaw_stand"}]}),
			_choice("Then we fight. Your flankers are not coming.", "", {"requires": [{"kind": "quest", "quest": "keep_one_fire_lit", "status": "active"}, {"key": "ashlands.steel_to_gate", "value": true}], "effects": [{"effect": "fight", "encounter": "ashmaw_stand_barred"}]}),
			_choice("Leave.", "", {})]},
		{"id": "scorn", "text": "\"The Long Dry is mine. Your fire goes out tonight.\"", "next": "hello"}], "services": false, "tags": []})
	_up("iron_reaches", "dialogues", {"id": "orsa_talk", "name": "Orsa at the forge gate", "speaker": "orsa_tallow", "start": "hello", "nodes": [
		{"id": "hello", "text": "Orsa wipes soot from her hands. \"Wardens. Thought you'd come.\"", "choices": [
			_choice("Why sell to the Wardens at cost?", "why", {}),
			_choice("Mara sent us for the steel.", "loaded", {"requires": _f("ashlands.oath_sworn"), "unless": _f("ashlands.steel_loaded"), "effects": [
				{"effect": "set_flag", "key": "ashlands.steel_loaded", "value": true}, {"effect": "add_flag", "key": "rep.iron_reaches.forge_guild", "delta": 5}]}),
			_choice("Farewell.", "", {})]},
		{"id": "why", "text": "\"A Warden captain pulled me out of a slag fire. I pay the hearth, not the order.\"", "next": "hello"},
		{"id": "loaded", "text": "\"Forty bars on one wagon. The Horde smells steel on a road, so ride hard.\"", "next": "hello"}], "services": false, "tags": []})

func _world() -> void:
	_up("ashlands", "characters", _enemy("grakka_ashmaw", "Grakka Ashmaw", "Chieftain of the Horde", "goblin_warlord",
		"Wears a crown of rusted nails, one for every Warden post he burned.", "grakka_parley",
		[{"key": "quest.ashlands.keep_one_fire_lit.ready", "value": true}, {"key": "ashlands.horde_broken", "value": true}]))
	var orsa: Dictionary = _own("iron_reaches", "characters", "orsa_tallow")
	orsa["dialogue"] = "orsa_talk"
	_up("iron_reaches", "characters", orsa)
	_up("ashlands", "encounters", _enc("ashmaw_stand", "Stand at the Ashmaw Camp", "Grakka and his guard make a stand; the flankers pour out of the gorge at Round 2.",
		"Grakka rises from the fire, nail-crown glinting.", ["grakka_ashmaw", "ash_shaman", "ash_archer"], ["ash_berserker", "ash_brawler", "ash_skirmisher"], "The flankers pour out of the gorge!"))
	_up("ashlands", "encounters", _enc("ashmaw_stand_barred", "Stand at the Ashmaw Camp (barred gate)", "Grakka and his guard stand alone; the barred gate cut off the flankers.",
		"Grakka looks past you to the gorge. Nobody comes.", ["grakka_ashmaw", "ash_shaman", "ash_archer"], [], "Reinforcements arrive!"))
	_up("ashlands", "locations", {"id": "ashmaw_camp", "name": "The Ashmaw Camp", "description": "A ring of hide tents in the wastes, smoke rising from a dozen fires.",
		"kind": "camp", "region": "cinder_pass", "exits": [{"to": "cinder_gate", "label": "back to the Cinder Gate"}],
		"encounters": ["ashmaw_stand", "ashmaw_stand_barred", "ash_raiders", "ash_scouts", "ash_patrol"], "map": "gorge_ruins",
		"zone_bindings": {"ruins": {"encounters": [["ash_raiders", 3], ["ash_scouts", 1]], "elites": [["ash_patrol", 1]], "elite_pct": 15, "steps_min": 10, "steps_max": 16}},
		"markers": [
			_marker("to_world", "exit", "west", {"to": "world_map"}),
			_marker("grakka", "npc", "site_b", {"character": "grakka_ashmaw"}),
			_marker("war_chest", "chest", "ruin_chest", {"items": {"con_heal_potion": 2}, "gold": 40}),
			_marker("patrol", "roamer", "", {"encounter": "ash_patrol", "path": "patrol", "radius": 1, "once": true})], "tags": []})
	var sg: Dictionary = _own("iron_reaches", "locations", "slag_gate")
	sg["map"] = "walled_town"
	sg["markers"] = [
		_marker("orsa", "npc", "npc_a", {"character": "orsa_tallow"}),
		_marker("signpost", "exit", "road", {"to": "world_map"}),
		_marker("yard_crate", "chest", "chest_a", {"items": {"con_heal_potion": 1}, "gold": 20})]
	_up("iron_reaches", "locations", sg)
	var cg: Dictionary = _own("ashlands", "locations", "cinder_gate")
	cg["markers"] = _without(cg["markers"], "id", "banner")
	cg["markers"].append(_marker("banner", "lore", "ruin_lore", {"requires_flags": _f("quest.ashlands.cinder_gate_watch.ready"), "lore": "ashmaw_banner"}))
	_up("ashlands", "locations", cg)
	_up("ashlands", "routes", {"id": "wastes_track", "name": "The wastes track", "from": "cinder_gate", "to": "ashmaw_camp", "two_way": true, "time": 3,
		"zone": "desert_outpost", "encounter_pct": 30, "repeat_pct": -1, "encounters": [["ash_raiders", 2], ["ash_scouts", 1]],
		"elites": [["ash_patrol", 1]], "elite_pct": 10, "scripted": [], "requires_flags": _f("ashlands.steel_spent"), "unless_flags": [], "tags": []})
	var sr: Dictionary = _own("iron_reaches", "routes", "south_road")
	sr["scripted"] = _without(sr["scripted"], "id", "wagon_raid")
	sr["scripted"].append({"id": "wagon_raid", "encounter": "ashlands:ash_raiders", "requires_flags": _f("ashlands.steel_loaded"), "unless_flags": _f("ashlands.steel_delivered"), "sets_flags": []})
	_up("iron_reaches", "routes", sr)
	_up("ashlands", "lore", {"id": "ashmaw_banner", "name": "The Ashmaw Banner", "category": "rumor",
		"text": "A hide banner nailed to the arch, one skull per warband. Somebody is uniting the Horde.", "tags": []})
	_up("ashlands", "loot", {"id": "chieftains_torc", "name": "Chieftain's Torc", "description": "", "reward_kind": "item", "ref": "acc_swift_amulet",
		"count": 1, "rarity": "unique", "requires_flags": [], "tags": []})
	var shop: Dictionary = _own("ashlands", "shops", "hest_stores")
	var stock: Array = []
	for r: Variant in shop["stock"]:
		if not (r is Dictionary and (r as Dictionary).get("requires_flags", []) == _f("ashlands.steel_to_arms")):
			stock.append(r)
	for row: Array in [["wpn_broadsword", 2], ["wpn_claymore", 1], ["shd_kite", 1], ["arm_plate", 1]]:
		stock.append({"item": row[0], "count": row[1], "price": -1, "requires_flags": _f("ashlands.steel_to_arms")})
	shop["stock"] = stock
	_up("ashlands", "shops", shop)

func _init() -> void:
	WorldCatalog.reload_from_disk()
	_world()
	_quests()
	_dialogues()
	if not failed:
		print("save ashlands ", WorldCatalog.save_world("ashlands"))
		print("save iron_reaches ", WorldCatalog.save_world("iron_reaches"))
	print("validate_all ", WorldCatalog.validate_all())
	for w: String in ["ashlands", "iron_reaches"]:
		print(w, " reward ", QuestRules.reward_warnings(w), " link ", QuestRules.link_warnings(w), " dlg ", DialogueRules.quest_warnings(w), " travel ", TravelRules.warnings(w))
	print("FAILED" if failed else "OK")
	quit(1 if failed else 0)
