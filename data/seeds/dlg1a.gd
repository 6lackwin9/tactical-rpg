extends SceneTree
# DLG1-A asset seed (idempotent): service lines, gold, Ashlands dialogues, keep shop.
# Run: tools/fc.sh run data/seeds/dlg1a.gd

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _flag(key: String) -> Array:
	return [{"key": key, "value": true}]

func _setf(key: String) -> Dictionary:
	return {"effect": "set_flag", "key": key, "value": true}

func _gold(n: int) -> Dictionary:
	return {"effect": "give", "gold": n}

func _node(id: String, text: String, extra: Dictionary) -> Dictionary:
	var n: Dictionary = {"id": id, "text": text}
	n.merge(extra, true)
	return n

func _tree(id: String, nm: String, speaker: String, nodes: Array) -> Dictionary:
	return {"id": id, "name": nm, "speaker": speaker, "start": "hello", "nodes": nodes, "services": true, "tags": []}

func _merge_char(id: String, extra: Dictionary) -> void:
	var c: Dictionary = WorldCatalog.get_own("ashlands", "characters", id)
	if c.is_empty():
		failed = true
		print("ERROR missing character ", id)
		return
	c.merge(extra, true)
	_chk("char " + id, WorldCatalog.upsert("ashlands", "characters", c))

func _chest_gold(loc_id: String, marker_id: String, gold: int) -> void:
	var loc: Dictionary = WorldCatalog.get_own("ashlands", "locations", loc_id)
	var found: bool = false
	var markers: Array = loc.get("markers", [])
	for m: Variant in markers:
		if m is Dictionary and str((m as Dictionary).get("id", "")) == marker_id:
			(m as Dictionary)["gold"] = gold
			found = true
	if not found:
		failed = true
		print("ERROR missing marker ", loc_id, "/", marker_id)
		return
	loc["markers"] = markers
	_chk("loc " + loc_id, WorldCatalog.upsert("ashlands", "locations", loc))

func _init() -> void:
	ServiceCatalog.reload_from_disk()
	QuestCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	MapCatalog.reload_from_disk()

	var lines: Array = [
		["greeting", "{npc} looks up."], ["offer", "About {quest}..."], ["accept", "We will do it."],
		["decline", "Not now."], ["turn_in", "We finished: {quest}."], ["shop", "Show me your wares."], ["farewell", "Farewell."]]
	for l: Array in lines:
		_chk("service " + str(l[0]), ServiceCatalog.upsert({"id": l[0], "text": l[1]}))
	print("service save ", ServiceCatalog.save())

	var tgold: Dictionary = {"reach_and_clear": 30, "two_stop_delivery": 25, "clear_site": 40, "signal_and_return": 20}
	for tid: String in tgold:
		var t: Dictionary = QuestCatalog.get_entry(tid)
		if t.is_empty():
			failed = true
			print("ERROR missing template ", tid)
			continue
		var r: Dictionary = (t.get("rewards", {}) as Dictionary).duplicate(true)
		r["gold"] = tgold[tid]
		t["rewards"] = r
		_chk("template " + tid, QuestCatalog.upsert(t))
	print("quest save ", QuestCatalog.save())

	_chk("manifest", WorldCatalog.update_manifest("ashlands", {"start_gold": 100}))  # ITEMPOW1-A raised it from 50

	var mara: Array = [
		_node("hello", "Mara looks up from a map pinned flat with knives. \"Make it quick.\"", {"choices": [
			{"text": "What is this place?", "to": "keep"},
			{"text": "These ash drifts are fresh. The Horde is close.", "to": "drifts",
				"requires": [{"kind": "stat", "stat": "KNOW", "min": 14}], "unless": _flag("ashlands.mara_drifts_told"), "show_locked": true},
			{"text": "The Courtyard is ours.", "to": "proud",
				"requires": [{"kind": "quest", "quest": "hold_the_courtyard", "status": "completed"}], "unless": _flag("ashlands.mara_courtyard_thanks")}]}),
		_node("keep", "\"Warden Keep. One fire, four walls and too few of us to hold either.\"", {"next": "hello"}),
		_node("drifts", "\"Sharp eyes. Take this; you will need it before I do.\"", {"next": "hello",
			"effects": [_setf("ashlands.mara_drifts_told"), {"effect": "give", "items": {"con_focus_elixir": 1}}]}),
		_node("proud", "\"It is. For now. Drink to it later.\"", {"next": "hello",
			"effects": [_setf("ashlands.mara_courtyard_thanks"), _gold(20)]})]
	var tev: Array = [
		_node("hello", "Tev is whittling arrow shafts. \"Smoke's wrong today.\"", {"choices": [
			{"text": "What do you see?", "to": "smoke"},
			{"text": "The pass is clear.", "to": "thanks",
				"requires": [{"kind": "quest", "quest": "cinder_gate_watch", "status": "completed"}], "unless": _flag("ashlands.tev_pass_thanks")}]}),
		_node("smoke", "\"Too much of it, too close, and upwind. Someone wants us looking the wrong way.\"", {"next": "hello"}),
		_node("thanks", "\"Then I owe you a drink and an arrow. Have the arrow money.\"", {"next": "hello",
			"effects": [_setf("ashlands.tev_pass_thanks"), _gold(15)]})]
	var hest: Array = [
		_node("hello", "Hest counts arrows without looking up. \"Buying or bothering?\"", {"choices": [
			{"text": "Trade a fire bomb back for coin?", "to": "traded",
				"requires": [{"kind": "items", "item": "con_fire_bomb", "count": 1}], "show_locked": true,
				"effects": [{"effect": "take", "items": {"con_fire_bomb": 1}}, _gold(20)]},
			{"text": "What do the stores need?", "to": "needs"}]}),
		_node("traded", "\"Fewer bombs in the hall is fine by me.\"", {"next": "hello"}),
		_node("needs", "\"Everything. Bring the potions back unopened and we will get along.\"", {"next": "hello"})]
	_chk("mara_talk", WorldCatalog.upsert("ashlands", "dialogues", _tree("mara_talk", "Mara at the map table", "mara_voss", mara)))
	_chk("tev_talk", WorldCatalog.upsert("ashlands", "dialogues", _tree("tev_talk", "Tev by the arrow bench", "tev_quill", tev)))
	_chk("hest_talk", WorldCatalog.upsert("ashlands", "dialogues", _tree("hest_talk", "Hest at the stores", "hest_the_quartermaster", hest)))

	var stock: Array = []
	for s: Array in [["con_heal_potion", -1, -1], ["con_focus_elixir", 5, -1], ["con_fire_bomb", 3, 40], ["shd_buckler", 1, -1], ["arm_leather", 1, -1]]:
		stock.append({"item": s[0], "count": s[1], "price": s[2], "requires_flags": []})
	_chk("hest_stores", WorldCatalog.upsert("ashlands", "shops", {
		"id": "hest_stores", "name": "Keep Stores Counter", "keeper": "hest_the_quartermaster", "stock": stock,
		"loot": [{"loot": "keep_ring", "price": 150}], "buy_mult": 1.0, "sell_mult": 0.5,
		"buys": ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL"], "requires_flags": [], "tags": []}))

	_merge_char("mara_voss", {"dialogue": "mara_talk", "gone_flags": _flag("ashlands.mara_fallen")})
	_merge_char("tev_quill", {"dialogue": "tev_talk", "gone_flags": _flag("ashlands.tev_fallen")})
	_merge_char("hest_the_quartermaster", {"dialogue": "hest_talk", "gone_flags": []})

	var q: Dictionary = WorldCatalog.get_own("ashlands", "quests", "hold_the_courtyard")
	if q.is_empty():
		failed = true
		print("ERROR missing quest hold_the_courtyard")
	else:
		q["variants"] = [{"id": "mara_fallen", "when_flags": _flag("ashlands.mara_fallen"), "unless_flags": [],
			"set": {"giver": "tev_quill", "turn_in": "tev_quill", "description": "Mara is gone. Tev holds the line in her place: clear the Courtyard."}}]
		_chk("quest hold_the_courtyard", WorldCatalog.upsert("ashlands", "quests", q))

	_chest_gold("warden_keep", "stores", 10)
	_chest_gold("ruined_courtyard", "ruin_cache", 25)
	_chest_gold("cinder_gate", "camp_cache", 15)

	print("save_world ", WorldCatalog.save_world("ashlands"))
	print("services ", ServiceCatalog.validate_all(), " warn ", ServiceCatalog.warnings_all())
	print("world ", WorldCatalog.validate_all())
	print("quests ", QuestCatalog.validate_all(), " warn ", QuestCatalog.warnings_all())
	print("maps ", MapCatalog.validate_all())
	print("quest_warnings ", DialogueRules.quest_warnings("ashlands"))
	print("dialogue_warnings ", DialogueRules.dialogue_warnings("ashlands"))
	print("FAILED" if failed else "OK")
	quit(1 if failed else 0)
