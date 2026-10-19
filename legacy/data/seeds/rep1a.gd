extends SceneTree
# REP1-A asset seed (idempotent): quest reputation, Hest's rep-gated choice, ashlands:ember_patrol.
# Run: tools/fc.sh run data/seeds/rep1a.gd

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _rep(world: String, id: String, rep: Dictionary) -> void:
	var q: Dictionary = WorldCatalog.get_own(world, "quests", id)
	if q.is_empty():
		failed = true
		print("ERROR missing quest ", world, ":", id)
		return
	q["reputation"] = rep
	_chk("quest " + id, WorldCatalog.upsert(world, "quests", q))

func _init() -> void:
	WorldCatalog.reload_from_disk()
	_rep("ashlands", "hold_the_courtyard", {"ember_wardens": 10})
	_rep("ashlands", "cinder_gate_watch", {"ember_wardens": 10, "ashen_horde": -10})
	_rep("ashlands", "warden_oath", {"ember_wardens": 15})
	_rep("iron_reaches", "iron_for_embers", {"forge_guild": 10})

	if not failed:
		var patrol: Dictionary = {
			"id": "ember_patrol", "name": "Sweep the Ruins",
			"description": "Hest wants the ruins swept of raiders before the next supply run.",
			"giver": "hest_the_quartermaster", "turn_in": "", "faction": "ember_wardens", "region": "cinder_pass",
			"requires": [], "template": "clear_site",
			"steps": [{"text": "Break a raiding party in the ruins", "encounter": "ash_raiders"}],
			"requires_flags": [{"key": "rep.ashlands.ember_wardens", "op": ">=", "value": 30}],
			"complete_flags": [{"key": "ashlands.ruins_swept", "value": true}],
			"auto_start": false,
			"reward": "Hest's gratitude and a pick from the stores.",
			"links": {"related_characters": ["hest_the_quartermaster"], "related_locations": ["ruined_courtyard", "cinder_gate"], "related_factions": ["ember_wardens", "ashen_horde"]},
			"variants": [], "loot_pool": ["keep_stores"], "loot_picks": 1,
			"reputation": {"ember_wardens": 5}, "tags": []}
		_chk("quest ember_patrol", WorldCatalog.upsert("ashlands", "quests", patrol))

	if not failed:
		var t: Dictionary = WorldCatalog.get_own("ashlands", "dialogues", "hest_talk")
		if t.is_empty():
			failed = true
			print("ERROR missing hest_talk")
		else:
			var choice: Dictionary = {
				"text": "Any spare stores for friends of the Wardens?", "to": "spare",
				"requires": [{"key": "rep.ashlands.ember_wardens", "op": ">=", "value": 10}],
				"unless": [{"key": "ashlands.hest_spare_given", "value": true}],
				"show_locked": true,
				"effects": [{"effect": "set_flag", "key": "ashlands.hest_spare_given", "value": true}, {"effect": "give", "items": {"con_heal_potion": 2}}]}
			var nodes: Array = t.get("nodes", [])
			var kept: Array = []
			for n: Variant in nodes:
				var nd: Dictionary = n as Dictionary
				if str(nd.get("id", "")) == "spare":
					continue
				if str(nd.get("id", "")) == "hello":
					var cs: Array = []
					for c: Variant in nd.get("choices", []):
						if str((c as Dictionary).get("to", "")) != "spare":
							cs.append(c)
					cs.append(choice)
					nd["choices"] = cs
				kept.append(nd)
			kept.append({"id": "spare", "text": "\"For you? Two, and don't tell the others.\"", "next": "hello"})
			t["nodes"] = kept
			_chk("dialogue hest_talk", WorldCatalog.upsert("ashlands", "dialogues", t))

	if not failed:
		print("save ashlands ", WorldCatalog.save_world("ashlands"))
		print("save iron_reaches ", WorldCatalog.save_world("iron_reaches"))
	print("validate_all ", WorldCatalog.validate_all())
	for w: String in ["ashlands", "iron_reaches"]:
		print(w, " reward ", QuestRules.reward_warnings(w), " link ", QuestRules.link_warnings(w), " dlg ", DialogueRules.quest_warnings(w))
	print("FAILED" if failed else "OK")
	quit(1 if failed else 0)
