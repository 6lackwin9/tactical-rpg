extends SceneTree
# M4-A asset seed (idempotent): quest templates, Ashlands loot, linked quests.
# Run: tools/fc.sh run data/seeds/m4a.gd

var failed: bool = false

func _c(key: String) -> Dictionary:
	return {"key": key, "value": true}

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _init() -> void:
	QuestCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	var obj_r: Dictionary = {"kind": "reach_location"}
	var obj_w: Dictionary = {"kind": "win_encounter"}
	var templates: Array[Dictionary] = [
		{"id": "reach_and_clear", "name": "Reach and clear", "description": "Go to a place, then win the fight there.", "objectives": [obj_r, obj_w], "rewards": {"xp": 60, "items": {"con_heal_potion": 2}}, "tags": []},
		{"id": "two_stop_delivery", "name": "Two-stop delivery", "description": "Pick something up at one place and bring it to another.", "objectives": [obj_r, obj_r], "rewards": {"xp": 40, "items": {"con_focus_elixir": 1}}, "tags": []},
		{"id": "clear_site", "name": "Clear a site", "description": "Win one fight.", "objectives": [obj_w], "rewards": {"xp": 80, "items": {"con_heal_potion": 1}}, "tags": []},
		{"id": "signal_and_return", "name": "Signal and return", "description": "Wait for a signal, then return to a place.", "objectives": [{"kind": "flag"}, obj_r], "rewards": {"xp": 50, "items": {"con_focus_elixir": 1}}, "tags": []},
	]
	for t: Dictionary in templates:
		_chk("template " + str(t["id"]), QuestCatalog.upsert(t))
	var loot: Array[Dictionary] = [
		{"id": "wardens_drill", "name": "Wardens' Drill", "description": "Mara's drill against fear, taught to those who held the line.", "reward_kind": "passive", "ref": "steady_nerves", "count": 1, "rarity": "rare", "requires_flags": [], "tags": []},
		{"id": "rallying_cry", "name": "Rallying Cry", "description": "The Wardens' call that steadies a wavering line.", "reward_kind": "skill", "ref": "rally", "count": 1, "rarity": "rare", "requires_flags": [], "tags": []},
		{"id": "keep_ring", "name": "Keep Ring", "description": "A heavy ring from the keep's strongbox.", "reward_kind": "item", "ref": "acc_bear_ring", "count": 1, "rarity": "rare", "requires_flags": [_c("ashlands.courtyard_held")], "tags": []},
		{"id": "keep_stores", "name": "Keep Stores", "description": "A satchel of the keep's healing draughts.", "reward_kind": "item", "ref": "con_heal_potion", "count": 3, "rarity": "common", "requires_flags": [], "tags": []},
	]
	for l: Dictionary in loot:
		_chk("loot " + str(l["id"]), WorldCatalog.upsert("ashlands", "loot", l))

	var hold: Dictionary = WorldCatalog.get_own("ashlands", "quests", "hold_the_courtyard")
	hold.merge({
		"template": "reach_and_clear", "turn_in": "", "requires_flags": [],
		"complete_flags": [_c("ashlands.courtyard_held")], "auto_start": true,
		"loot_pool": ["keep_stores", "wardens_drill", "keep_ring"], "loot_picks": 1,
		"links": {"leads_to": ["cinder_gate_watch", "warden_oath"], "related_characters": ["mara_voss", "tev_quill"], "related_locations": ["warden_keep", "ruined_courtyard"], "related_factions": ["ember_wardens", "ashen_horde"]},
		"variants": [],
	}, true)
	var gate: Dictionary = {
		"id": "cinder_gate_watch", "name": "Watch at the Cinder Gate",
		"description": "Mara suspects the warband left scouts in the pass. Find them before they find the keep.",
		"giver": "mara_voss", "faction": "ember_wardens", "region": "cinder_pass",
		"requires": ["hold_the_courtyard"], "template": "clear_site",
		"steps": [{"text": "Break the ambush at the Cinder Gate", "encounter": "pass_ambush"}],
		"requires_flags": [], "complete_flags": [_c("ashlands.pass_cleared")], "auto_start": false,
		"reward": "The Wardens' thanks, and a pick from the keep.", "tags": [], "turn_in": "",
		"loot_pool": ["wardens_drill", "keep_ring", "rallying_cry"], "loot_picks": 1,
		"links": {"follows": ["hold_the_courtyard"], "leads_to": ["warden_oath"], "related_characters": ["mara_voss"], "related_factions": ["ashen_horde"]},
		"variants": [{"id": "mara_fallen", "when_flags": [_c("ashlands.mara_fallen")], "unless_flags": [], "set": {
			"giver": "tev_quill", "turn_in": "tev_quill",
			"description": "Mara is gone. Tev means to finish what she started and clear the pass himself.",
			"reward": "Tev's thanks and what the keep can spare.", "rewards_from": "reach_and_clear",
			"loot_pool": ["keep_stores", "rallying_cry"]}}],
	}
	var oath: Dictionary = {
		"id": "warden_oath", "name": "The Warden's Oath",
		"description": "Once the pass is safe, Tev will speak for the party before the Wardens.",
		"giver": "tev_quill", "faction": "ember_wardens", "region": "ashen_marches",
		"requires": ["hold_the_courtyard"], "template": "signal_and_return",
		"steps": [
			{"text": "Wait for word that the Cinder Gate is clear", "flag": "ashlands.pass_cleared", "value": true},
			{"text": "Return to Warden Keep and swear the oath", "location": "warden_keep"}],
		"requires_flags": [], "complete_flags": [_c("ashlands.oath_sworn")], "auto_start": false,
		"reward": "A Warden's gift, and the oath.", "tags": [], "turn_in": "",
		"loot_pool": ["rallying_cry", "wardens_drill"], "loot_picks": 1,
		"links": {"follows": ["hold_the_courtyard", "cinder_gate_watch"], "related_characters": ["tev_quill", "mara_voss"], "related_locations": ["warden_keep"], "related_factions": ["ember_wardens"]},
		"variants": [
			{"id": "mara_fallen", "when_flags": [_c("ashlands.mara_fallen")], "unless_flags": [], "set": {
				"description": "With Mara gone, Tev asks the party to swear the oath over her empty seat.",
				"steps": {"1": {"text": "Swear the oath in the Ruined Courtyard where Mara fell", "location": "ruined_courtyard"}}}},
			{"id": "tev_fallen", "when_flags": [_c("ashlands.tev_fallen")], "unless_flags": [], "set": {"available": false}},
		],
	}
	for q: Dictionary in [hold, gate, oath]:
		_chk("quest " + str(q["id"]), WorldCatalog.upsert("ashlands", "quests", q))

	var iron: Dictionary = WorldCatalog.get_own("iron_reaches", "quests", "iron_for_embers")
	iron.merge({
		"template": "two_stop_delivery", "turn_in": "", "requires_flags": [],
		"complete_flags": [_c("iron_reaches.shipment_delivered")], "auto_start": false,
		"loot_pool": [], "loot_picks": 1,
		"links": {"related_characters": ["orsa_tallow"], "related_locations": ["slag_gate", "ashlands:warden_keep"], "related_factions": ["forge_guild", "ashlands:ember_wardens"]},
		"variants": [],
	}, true)
	_chk("quest iron_for_embers", WorldCatalog.upsert("iron_reaches", "quests", iron))

	if failed:
		print("STOPPED: not saving")
		quit(1)
		return
	print("quest validate_all: ", QuestCatalog.validate_all())
	print("quest warnings_all: ", QuestCatalog.warnings_all())
	print("quest ids: ", QuestCatalog.ids())
	print("world validate_all: ", WorldCatalog.validate_all())
	print("save quests: ", QuestCatalog.save())
	print("save ashlands: ", WorldCatalog.save_world("ashlands"))
	print("save iron_reaches: ", WorldCatalog.save_world("iron_reaches"))
	quit()
