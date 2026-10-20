extends SceneTree

# The content checks of the Godot build (PORT-CHECKS): what ContentCatalog and WorldCatalog report for the shipped
# data (validate_all, warnings_all) and for generated broken entries, so the engine's `aegis content check` can be
# compared message for message. Read-only: nothing is written to the catalogs.
#   godot --headless --path <project> -s <abs path> -- out=<abs dir>
# Writes checks.json: {"catalog": {"errors", "warnings"}, "worlds": {"errors", "warnings"},
#   "cases": [{"world" ("" = catalog), "kind", "current", "entry", "errors", "warnings"}]}.
# Cases: a few entries of every kind, each field set to a few wrong values (a number below and far above its range, an
# unknown or empty text, an unknown reference, an empty and a doubled list, a wrong shape), plus hand-made cases for
# the rules across entries (skill families, item families).

const PER_KIND: int = 3

var out_dir: String = ""

func _initialize() -> void:
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("out="):
			out_dir = arg.substr(4)
	var cases: Array = []
	for kind: String in ContentSchema.KINDS + ["item_families"]:
		for e: Dictionary in _samples(kind, ContentCatalog.entries(kind)):
			for mutated: Dictionary in _mutations(ContentSchema.fields(kind), e):
				cases.append(_catalog_case(kind, mutated, str(e.get("id", ""))))
	for e: Dictionary in _hand_made():
		cases.append(_catalog_case(str(e["kind"]), DictUtil.dict_of(e["entry"]), str(e.get("current", ""))))
	for w: String in WorldCatalog.world_ids():
		for kind: String in WorldSchema.KINDS:
			for e: Dictionary in _samples(kind, WorldCatalog.own_entries(w, kind)):
				for mutated: Dictionary in _mutations(WorldSchema.fields(kind), e):
					cases.append({"world": w, "kind": kind, "current": str(e.get("id", "")), "entry": mutated,
						"errors": WorldCatalog.validate_entry(w, kind, mutated, str(e.get("id", ""))), "warnings": [],
						"blocking": WorldCatalog.blocking_errors(w, kind, mutated, str(e.get("id", "")))})
	var doc: Dictionary = {
		"catalog": {"errors": ContentCatalog.validate_all(), "warnings": ContentCatalog.warnings_all()},
		"worlds": {"errors": WorldCatalog.validate_all(), "warnings": WorldCatalog.warnings_all()},
		"cases": cases,
		"art_keys": {"unit": AssetRegistry.keys("unit"), "item": AssetRegistry.keys("item")},
	}
	doc["lint"] = _world_lint()
	DirAccess.make_dir_recursive_absolute(out_dir)
	var f: FileAccess = FileAccess.open(out_dir.path_join("checks.json"), FileAccess.WRITE)
	f.store_string(JSON.stringify(doc, "", false))
	f.close()
	print("§ checks: %d cases -> %s" % [cases.size(), out_dir.path_join("checks.json")])
	quit()

func _catalog_case(kind: String, entry: Dictionary, current: String) -> Dictionary:
	return {"world": "", "kind": kind, "current": current, "entry": entry,
		"errors": ContentCatalog.validate_entry(kind, entry, current), "warnings": ContentCatalog.warnings_for(kind, entry)}

# A few entries of a kind: the first PER_KIND of each sub-kind (skill kind, item type), else the first PER_KIND.
func _samples(kind: String, entries: Array[Dictionary]) -> Array[Dictionary]:
	var key: String = {"skills": "kind", "items": "item_type", "item_families": "item_type"}.get(kind, "")
	var seen: Dictionary = {}
	var picked: Array[Dictionary] = []
	for e: Dictionary in entries:
		var group: String = str(e.get(key, "")) if not key.is_empty() else ""
		var n: int = DictUtil.to_int(seen.get(group, 0))
		if n < PER_KIND:
			seen[group] = n + 1
			picked.append(e)
	return picked

# Copies of `e` with one field set to a wrong value each.
func _mutations(fields: Array[Dictionary], e: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for f: Dictionary in fields:
		var key: String = str(f.get("key", ""))
		if key.is_empty():
			continue
		for v: Variant in _wrong_values(f, e.get(key, f.get("default", null))):
			var m: Dictionary = e.duplicate(true)
			m[key] = v
			out.append(m)
	return out

func _wrong_values(f: Dictionary, current: Variant) -> Array:
	match str(f.get("type", "")):
		"int":
			return [-1, 0, 9999]
		"float":
			return [-0.5, 0.0, 99.0]
		"bool":
			return [not DictUtil.to_bool(current)]
		"enum":
			return ["BOGUS", ""]
		"ref":
			return ["zz_missing", ""]
		"id":
			return ["", "Bad Id!", "zz_new_id"]
		"string", "text":
			return ["", "x".repeat(300)]
		"template":
			return ["", "{bogus} {power:x} {", "Deals 50 damage."]
		"refs", "strings", "multi_pick":
			var list: Array = current if current is Array else []
			var values: Array = [[], ["zz_missing"]]
			if not list.is_empty():
				values.append([list[0], list[0]])
			return values
		"stats":
			return [{}, {"ZZZ": 3}, {"STR": -5}]
	# json, tables and grids: an empty one and a stray row, in the shape the field holds (the validators expect it).
	if current is Array:
		return [[], [{"zz": 1}]]
	if current is Dictionary:
		return [{}, {"zz": 1}]
	return []

# The rules across entries no single-field change reaches.
func _hand_made() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var skills: Array[Dictionary] = ContentCatalog.entries("skills")
	for e: Dictionary in skills:
		if DictUtil.to_int(e.get("tier", 1), 1) == 2:
			var gap: Dictionary = e.duplicate(true)
			gap["tier"] = 3
			out.append({"kind": "skills", "current": str(e["id"]), "entry": gap})
			var wrong: Dictionary = e.duplicate(true)
			wrong["id"] = "%s_9" % str(e["family"])
			out.append({"kind": "skills", "current": str(e["id"]), "entry": wrong})
			var kind_mix: Dictionary = e.duplicate(true)
			kind_mix["kind"] = "FOCUS" if str(e["kind"]) != "FOCUS" else "ATTACK"
			out.append({"kind": "skills", "current": str(e["id"]), "entry": kind_mix})
			break
	for e: Dictionary in skills:
		if str(e.get("kind", "")) == "ATTACK" and DictUtil.to_int(e.get("tier", 1), 1) == 1:
			var combo: Dictionary = e.duplicate(true)
			combo["exploit"] = [{"status": "bleed", "power_mult": 3.0}, {"status": "bleed", "power_mult": 1.5}, {"status": "haste", "power_mult": 1.2}]
			combo["exploit_consume"] = true
			combo["disrupt"] = "STANCE_BREAK"
			combo["power_mult"] = 0.0
			out.append({"kind": "skills", "current": str(e["id"]), "entry": combo})
			var dash: Dictionary = e.duplicate(true)
			dash["dash_tiles"] = 2
			dash["area"] = "LINE"
			dash["weapon_hand"] = "off"
			dash["reach_max"] = 3
			out.append({"kind": "skills", "current": str(e["id"]), "entry": dash})
			break
	for e: Dictionary in skills:
		if str(e.get("kind", "")) == "SUPPORT":
			var sup: Dictionary = e.duplicate(true)
			sup["heal_base"] = 0
			sup["heal_mult"] = 0.0
			sup["grant_status"] = ""
			sup["cleanse_statuses"] = []
			out.append({"kind": "skills", "current": str(e["id"]), "entry": sup})
			var extra: Dictionary = e.duplicate(true)
			extra["extra_grants"] = [{"status": "taunt", "rounds": 5, "magnitude": 3}, {"status": "zz", "rounds": 1}]
			out.append({"kind": "skills", "current": str(e["id"]), "entry": extra})
			break
	for e: Dictionary in ContentCatalog.entries("items"):
		var mods: Dictionary = e.duplicate(true)
		mods["modifiers"] = [{"kind": "adjust", "field": "power", "delta": 0}, {"kind": "adjust", "field": "power", "delta": 2}, {"kind": "bogus"}, "x"]
		out.append({"kind": "items", "current": str(e["id"]), "entry": mods})
		var tiers: Dictionary = e.duplicate(true)
		tiers["tiers"] = [{"when": {"type": "stat", "stat": "STR", "value": 5}, "bonus": {"power": 1}}, {"when": {"type": "stat", "stat": "STR", "value": 3}, "bonus": {}}, {"when": {"type": "zz"}}]
		out.append({"kind": "items", "current": str(e["id"]), "entry": tiers})
		var ranked: Dictionary = e.duplicate(true)
		ranked["tiers"] = [{"label": "x".repeat(30), "desc": "y".repeat(130), "when": {"type": "stat_at_least", "stat": "STR", "value": 5, "zz": 1}, "bonus": {"power": 60, "mitigation": 2, "zz": 1}},
			{"when": {"type": "stat_at_least", "stat": "STR", "value": 3}, "bonus": {"stat_bonuses": {"STR": 9, "DEX": 1, "INT": 1, "ZZZ": 1}}},
			{"when": {"type": "proficiency_at_least", "gear": "zz", "level": 9}, "bonus": {"effects": [{"type": "zz"}, {"type": "status_potency", "value": 5}, {"type": "spell_reach", "value": 1}]}},
			{"when": {"type": "unit_level_at_least", "value": 20}, "bonus": {"power": 0}},
			{"when": {"type": "stat_at_least", "stat": "STR", "value": 5, "zz": 1}, "bonus": {"power": 1}}]
		out.append({"kind": "items", "current": str(e["id"]), "entry": ranked})
		var text: Dictionary = e.duplicate(true)
		text["description"] = "{power} {req:ZZ} {bon} {range:x} {tier9.value} {tier.value} {zzz} {mitigation} 12 %d" % DictUtil.to_int(e.get("base_power", 0))
		out.append({"kind": "items", "current": str(e["id"]), "entry": text})
		break
	for f: Dictionary in ContentCatalog.entries("item_families"):
		var fam: Dictionary = f.duplicate(true)
		fam["hands"] = ["off", "main_1h", "zz", "off"]
		fam["scaling_stats"] = ["KNOW"]
		fam["base"] = {"zz": {}, "main_1h": {"power": 900, "zz": 1}}
		fam["bands"] = {"zz": {}, "main_1h": {"power": {"min": 9, "max": 2}, "range": {"min": 0, "max": 9}, "bonus_value": {"min": -1, "max": 2}, "zz": {"min": 1, "max": 2}}}
		fam["engine_rules"] = ["focus", "zz", "shield", "spell_source"]
		fam["modifier_kinds"] = ["zz", "adjust"]
		fam["adjustable"] = ["zz", "power"]
		out.append({"kind": "item_families", "current": str(f["id"]), "entry": fam})
		var renamed: Dictionary = f.duplicate(true)
		renamed["id"] = "zz_family"
		out.append({"kind": "item_families", "current": str(f["id"]), "entry": renamed})
		break
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var foe: String = str(DictUtil.array_of(e.get("enemies", []))[0])
		for extra: Dictionary in _encounter_breaks(foe, DictUtil.to_int(e.get("max_rounds", 0))):
			var broken: Dictionary = e.duplicate(true)
			broken.merge(extra, true)
			out.append({"kind": "encounters", "current": str(e["id"]), "entry": broken})
		break
	return out

# Field sets that break the shared encounter validators (phases, objectives, morale, purse, exits), merged into a
# shipped encounter whose first enemy is `foe`.
func _encounter_breaks(foe: String, max_rounds: int) -> Array[Dictionary]:
	var good_phase: Dictionary = {"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": foe, "pct": 50, "or_round": 3},
		"effects": [{"type": "enrage", "unit": foe, "power": 20}, {"type": "line", "text": "It roars."}]}
	var bad_phase: Dictionary = {"id": "Bad Id", "name": "x".repeat(50), "zz": 1, "pause_autoplay": "no",
		"trigger": {"kind": "round", "round": max_rounds + 5, "or_round": 2, "zz": 1},
		"effects": [{"type": "line", "text": ""}, {"type": "line", "text": "two"}, {"type": "wave", "units": ["zz_missing", 3], "label": 5},
			{"type": "wave", "units": []}, {"type": "status", "unit": "nobody", "status": "bleed"}, {"type": "zz"}]}
	var more_effects: Dictionary = {"id": "more", "name": "More", "trigger": {"kind": "hp_below", "unit": "nobody", "pct": 0.5, "or_round": 1},
		"effects": [{"type": "status", "unit": foe, "status": "empower"}, {"type": "status", "unit": foe, "status": "haste", "rounds": 9, "magnitude": 2},
			{"type": "ai", "unit": foe}, {"type": "ai", "unit": foe, "role": "ZZ", "target_pref": "ZZ", "reckless": 1, "flee_ratio": 2.0},
			{"type": "enrage", "unit": "nobody", "power": 40}], "pause_autoplay": false}
	return [
		{"phases": [good_phase]},
		{"phases": [good_phase, bad_phase, more_effects, "x", {"id": "rage", "trigger": "x", "effects": "x"}]},
		{"phases": "x"},
		{"objective": "hold", "objective_tiles": [[0, 0], [99, 1], [1], [5, 4]], "objective_rounds": 0, "objective_text": "x".repeat(90)},
		{"objective": "hold", "objective_tiles": [], "arena": "OPEN", "objective_rounds": max_rounds + 1},
		{"objective": "slay", "objective_unit": "nobody", "guests": [foe, foe, "zz_a"], "protect": ["zz_b"]},
		{"objective": "escape", "exits": [], "arena": "CLOSED"},
		{"morale": true, "morale_break": 95, "morale_leader": "nobody", "morale_groups": [[0], [0], [], [99], [0.5]], "morale_text": "x".repeat(90)},
		{"morale": true, "morale_groups": "x"},
		{"morale": true, "morale_groups": [[1, 2], "x"]},
		{"purse": {"gold_min": -1, "gold_max": 600, "loot": "zz_missing", "once": "yes", "zz": 1}},
		{"purse": {"gold_min": 5.5, "gold_max": 2, "loot": 7}},
		{"purse": {"gold_min": 10, "gold_max": 150, "loot": "ashlands:wardens_drill"}},
		{"purse": {"gold_min": 0, "loot": "ashlands:keep_ring"}},
		{"purse": {"gold_min": 0}},
		{"purse": "x"},
		{"start_status": [], "exits": [[1, 2, 3]]},
		{"exits": [[20, 1], [-1, 0]]},
		{"exits": "x"},
		{"pace": "boss"},
		{"party": [], "enemies": [], "wave_units": [foe], "wave_block": 99, "wave_round": 99},
	]

# The rules across a world no single-field change reaches (dialogue trees, quest variants and links, map markers and
# bindings, shops, routes, start buffs) and the world lints: lint entries upserted into one world in memory (never
# saved), then validate_all / warnings_all of every world, then the worlds reloaded from disk.
# {"world", "entries": [{"kind", "entry", "refused"}], "errors", "warnings"}.
func _world_lint() -> Dictionary:
	var w: String = "ashlands" if WorldCatalog.world_ids().has("ashlands") else WorldCatalog.world_ids()[0]
	var loc: Dictionary = {}
	for e: Dictionary in WorldCatalog.own_entries(w, "locations"):
		if not str(e.get("map", "")).is_empty():
			loc = e
			break
	var map_id: String = str(loc.get("map", ""))
	var t: Dictionary = MapCatalog.get_entry(map_id)
	var anchors: Array[String] = []
	for a: String in MapRules.anchor_ids(t):
		if a != "spawn":
			anchors.append(a)
	var anchor := func(i: int) -> String: return anchors[mini(i, anchors.size() - 1)] if not anchors.is_empty() else ""
	var zones: Array[String] = MapRules.danger_zone_ids(t)
	var zone: String = zones[0] if not zones.is_empty() else "zz_zone"
	var item_id: String = str(ContentCatalog.entries("items")[0]["id"])
	var characters: Array[Dictionary] = WorldCatalog.own_entries(w, "characters")
	var who: String = str(characters[0]["id"])
	var other: String = str(characters[mini(1, characters.size() - 1)]["id"])
	var quests: Array[Dictionary] = WorldCatalog.own_entries(w, "quests")
	var quest: String = str(quests[0]["id"])
	var encounters: Array[Dictionary] = WorldCatalog.own_entries(w, "encounters")
	var enc: Dictionary = encounters[0].duplicate(true)
	var item_loot: String = ""
	var other_loot: String = ""
	for l: Dictionary in WorldCatalog.own_entries(w, "loot"):
		if str(l.get("reward_kind", "item")) == "item" and item_loot.is_empty():
			item_loot = str(l["id"])
		elif str(l.get("reward_kind", "item")) != "item" and other_loot.is_empty():
			other_loot = str(l["id"])
	var template: String = str(QuestCatalog.entries()[0]["id"])
	var long: String = "x".repeat(200)
	enc["id"] = "zz_enc"
	enc.erase("_world")
	enc.erase("_qid")
	enc["start_status"] = [{"status": "taunt"}, {"status": "haste", "rounds": 9, "magnitude": 4, "zz": 1, "requires_flags": [{"key": "Bad"}], "text": ""}, "x", {"status": "haste", "text": "ok"}]
	enc["purse"] = {"gold_min": 0, "gold_max": 300, "loot": "zz"}
	enc["pace"] = "boss"
	enc["phases"] = []
	enc["guests"] = [who, who]
	var entries: Array = [
		["dialogues", {"id": "zz_tree", "name": "Lint", "speaker": who, "start": "a", "services": true, "nodes": [
			{"id": "a", "text": "Hi {npc} {quest}", "choices": [
				{"text": "", "to": "zz", "zz": 1, "show_locked": 1,
					"requires": [{"kind": "zz"}, {"kind": "quest", "quest": "", "status": "zz"}, {"kind": "items", "item": "zz", "count": 0}, {"kind": "gold"}, {"kind": "stat", "stat": "ZZ", "min": 99}, {"kind": "flag", "key": "Bad", "op": ">", "value": "x"}, "x"],
					"effects": [{"effect": "open_shop", "shop": ""}, {"effect": "turn_in", "quest": "zz"}, {"effect": "fight", "encounter": "zz"}, {"effect": "recruit", "character": "zz"}, {"effect": "recruit", "character": who}, {"effect": "zz"}, "x"]},
				{"text": "give", "to": "b", "effects": [{"effect": "give", "items": {"zz": 0}, "gold": -1}, {"effect": "add_flag", "key": "rep.x.y", "delta": 0, "min": 5, "max": 1}, {"effect": "set_flag", "key": "rep.a.b", "value": 500}]},
				{"text": "Gift", "to": "g"},
				{"text": "Start", "effects": [{"effect": "start_quest", "quest": quest}, {"effect": "take", "items": {}, "gold": 0}, {"effect": "turn_in", "quest": quest}]}]},
			{"id": "b", "text": "x", "effects": [{"effect": "give", "gold": 5}, {"effect": "fight", "encounter": "zz"}], "next": "a"},
			{"id": "b", "text": "dup", "end": true},
			{"id": "g", "text": "Here, {gold}.", "effects": [{"effect": "give", "gold": 5}], "next": "a"},
			{"id": "Bad", "speaker": 5, "text": 5, "choices": "x", "next": 1, "end": "x"},
			{"id": "c", "text": "orphan", "end": true, "zz": 1},
			"x"]}],
		["quests", {"id": "zz_quest", "name": "Lint quest", "template": template, "giver": who, "turn_in": other,
			"steps": [{"text": "a"}, {"text": "", "location": "zz"}, {"zz": 1}],
			"requires": ["zz_quest_b"], "requires_flags": [{"key": "Bad"}], "complete_flags": [{"key": "rep.%s.zz" % w, "value": 900}],
			"reputation": {"zz": 5, "a": 0, "b": 1, "c": 1, "d": 1},
			"links": {"leads_to": [quest, "zz_quest", "zz_quest"], "zz": [], "follows": "x"},
			"loot_pool": [item_loot, item_loot, other_loot, "zz"], "loot_picks": 9,
			"variants": [{"id": "Bad", "zz": 1, "set": {"zz": 1, "giver": 5, "rewards_from": "zz", "available": "x", "steps": {"9": {}, "0": {"zz": 1, "text": ""}, "1": "x"}, "links": "x", "loot_pool": "x", "loot_picks": 9}},
				{"id": "v2", "when_flags": [{"key": "a"}], "set": {}}, "x",
				{"id": "v3", "when_flags": [{"key": "b"}], "set": {"loot_picks": 3, "rewards_from": template, "turn_in": "zz", "description": 5}}]}],
		["quests", {"id": "zz_quest_b", "name": "B", "requires": ["zz_quest"], "auto_start": true, "loot_pool": [item_loot], "variants": [{"id": "v", "when_flags": [{"key": "a"}], "set": {"available": false}}]}],
		["locations", {"id": "zz_loc", "name": "Lint place", "map": map_id, "map_pos": [2, 0.5],
			"exits": [{"to": str(loc.get("id", ""))}, {"to": ""}, {"to": "zz"}, {"to": "zz_world:zz"}],
			"zone_bindings": {"zz": {}, zone: {"encounters": [[str(encounters[0]["id"]), 0], [str(encounters[0]["id"]), 5], ["zz", 5], "x"], "elite_pct": 200, "steps_min": 50, "steps_max": 20, "zz": 1}, "x": "y"},
			"markers": [{"id": "m1", "kind": "exit", "anchor": anchor.call(0), "to": "zz"}, {"id": "m1", "kind": "zz"}, {"id": "Bad"}, "x",
				{"id": "m2", "kind": "roamer", "anchor": anchor.call(0), "encounter": "", "path": "zz", "radius": 9, "once": 1},
				{"id": "m3", "kind": "chest", "anchor": anchor.call(0), "items": {"zz": 0}, "loot": other_loot, "gold": -1},
				{"id": "m4", "kind": "chest", "anchor": "spawn"},
				{"id": "m5", "kind": "npc", "anchor": "zz", "character": "", "label": long, "note": "x"},
				{"id": "m6", "kind": "point", "anchor": anchor.call(1), "note": long, "zz": 1},
				{"id": "m7", "kind": "exit", "anchor": anchor.call(2), "to": "world_map", "to_anchor": "zz", "locked_text": 5},
				{"id": "m8", "kind": "encounter", "anchor": anchor.call(3), "encounter": "zz", "trigger": "zz"},
				{"id": "m9", "kind": "lore", "anchor": anchor.call(4), "lore": ""}]}],
		["shops", {"id": "zz_shop", "name": "Lint shop", "keeper": "", "stock": [{"item": "zz"}, {"item": item_id, "count": 0, "price": 0, "zz": 1, "requires_flags": "x"}, {"item": item_id}, "x"],
			"loot": [{"loot": other_loot, "price": 0}, {"loot": item_loot, "price": 10}, {"loot": item_loot, "price": 10}, {"zz": 1}, "x"],
			"buy_mult": 0.333, "sell_mult": 2.0, "supply_price": 5000, "buys": ["QUEST", "WEAPON", "WEAPON", 5], "requires_flags": [{"key": ""}]}],
		["routes", {"id": "zz_route", "name": "Lint road", "from": str(loc.get("id", "")), "to": str(loc.get("id", "")), "time": 0, "zone": "zz", "encounter_pct": 50, "repeat_pct": -5, "elite_pct": 200,
			"encounters": [], "elites": [], "scripted": [{"id": "s1", "encounter": ""}, {"id": "s1", "encounter": "zz", "zz": 1, "sets_flags": [{"key": "rep.q.r", "value": 999}]}, {"id": "Bad"}, "x", {"id": "s3"}]}],
		["characters", {"id": "zz_char", "name": "Lint person", "base_unit": "", "tuning": {}, "gone_flags": [{"key": "Bad"}]}],
		["factions", {"id": "zz_fac", "name": "Lint band", "relations": {"zz": "ally"}}],
		["regions", {"id": "zz_r1", "name": "R1", "parent": "zz_r2"}],
		["regions", {"id": "zz_r2", "name": "R2", "parent": "zz_r1"}],
		["items", {"id": "zz_item", "name": "Lint blade", "base_item": "", "tuning": {}}],
		["loot", {"id": "zz_loot", "name": "Lint loot", "reward_kind": "skill", "ref": "strike", "count": 2}],
		["loot", {"id": "zz_loot2", "name": "Lint loot 2", "reward_kind": "item", "ref": "zz", "count": 0, "requires_flags": [{"key": "x", "op": "~"}]}],
		["encounters", enc],
	]
	var out: Array = []
	for pair: Array in entries:
		var refused: Array[String] = WorldCatalog.upsert(w, str(pair[0]), DictUtil.dict_of(pair[1]))
		out.append({"kind": pair[0], "entry": pair[1], "refused": refused})
	var report: Dictionary = {"world": w, "entries": out, "errors": WorldCatalog.validate_all(), "warnings": WorldCatalog.warnings_all()}
	WorldCatalog.reload_from_disk()
	return report
