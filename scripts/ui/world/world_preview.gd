class_name WorldPreview
extends RefCounted

# ==============================================================================
# Tactical RPG - World Preview (what a world entry means, lore and gameplay)
# BBCode summaries for the World Forge's side panel: the lore as written, what the
# entry plays as (its base asset), where it diverges, who uses it, and the world's
# inheritance. Pure functions over WorldCatalog / ContentCatalog.
# ==============================================================================

static func _line(label: String, value: String) -> String:
	return "[color=#8ba3b8]%s:[/color] %s\n" % [label, value]

static func _esc(text: String) -> String:
	return text.replace("[", "[lb]")

static func _name_of(world_id: String, kind: String, ref: String) -> String:
	if ref.is_empty():
		return "-"
	var hit: Dictionary = WorldCatalog.resolve(world_id, kind, ref)
	return str(hit.get("name", "")) if not hit.is_empty() else "%s (missing)" % ref

static func _names_of(world_id: String, kind: String, refs: Variant) -> String:
	var parts: Array[String] = []
	if refs is Array:
		var list: Array = refs
		for r: Variant in list:
			parts.append(_name_of(world_id, kind, str(r)))
	return ", ".join(parts) if not parts.is_empty() else "-"

static func texture_for(kind: String, entry: Dictionary) -> Texture2D:
	match kind:
		"characters":
			var base: Dictionary = WorldFactory.unit_entry(entry)
			if base.is_empty():
				return null
			var team: UnitArt.Team = UnitArt.Team.FRIENDLY
			match str(entry.get("role", "")):
				"enemy":
					team = UnitArt.Team.HOSTILE
				"npc":
					team = UnitArt.Team.NEUTRAL
			return UnitArt.portrait(ContentFactory.art_of(base), team)
		"items":
			var named: Item = WorldFactory.named_item(entry)
			return ItemArt.icon(named) if named != null else null
	return null

static func describe(world_id: String, kind: String, entry: Dictionary) -> String:
	var e: Dictionary = WorldSchema.normalize(kind, entry)
	var text: String = ""
	match kind:
		"world":
			text = describe_world(str(e.get("id", world_id)))
		"regions":
			text = "[b]%s[/b]  (%s)\n" % [str(e["name"]), str(e["climate"])]
			text += _line("Part of", _name_of(world_id, "regions", str(e["parent"])))
			text += _line("Controlled by", _name_of(world_id, "factions", str(e["faction"])))
			text += _line("Locations", _list_where(world_id, "locations", "region", e))
			text += "\n" + _esc(str(e["description"]))
		"locations":
			var exits: Array[String] = []
			if e["exits"] is Array:
				for x: Variant in DictUtil.array_of(e["exits"]):
					var ex: Dictionary = x if x is Dictionary else {}
					exits.append("%s → %s" % [str(ex.get("label", "exit")), _name_of(world_id, "locations", str(ex.get("to", "")))])
			text = "[b]%s[/b]  (%s)\n" % [str(e["name"]), str(e["kind"])]
			text += _line("Region", _name_of(world_id, "regions", str(e["region"])))
			text += _line("Exits", "; ".join(exits) if not exits.is_empty() else "-")
			text += _line("Encounters", _names_of(world_id, "encounters", e["encounters"]))
			text += "\n" + _esc(str(e["description"]))
		"factions":
			var rel: Array[String] = []
			if e["relations"] is Dictionary:
				var rel_dict: Dictionary = e["relations"]
				for other: Variant in rel_dict:
					rel.append("%s %s" % [str(rel_dict[other]), _name_of(world_id, "factions", str(other))])
			text = "[b]%s[/b]  [i]%s[/i]\n" % [str(e["name"]), _esc(str(e["motto"]))]
			text += _line("Home", _name_of(world_id, "regions", str(e["home"])))
			text += _line("Relations", ", ".join(rel) if not rel.is_empty() else "-")
			text += _line("Members", _list_where(world_id, "characters", "faction", e))
			text += "\n" + _esc(str(e["description"]))
		"characters":
			text = _character(world_id, e)
		"items":
			text = _item(world_id, e)
		"disciplines":
			text = "[b]%s[/b]\n" % str(e["name"])
			text += _line("Grants", _passive_names(e["passives"]))
			text += _line("Taught by", _name_of(world_id, "characters", str(e["teacher"])))
			text += _line("Faction", _name_of(world_id, "factions", str(e["faction"])))
			text += _line("Trained by", _list_where(world_id, "characters", "disciplines", e))
			text += "\n" + _esc(str(e["description"]))
		"quests":
			var steps: Array[String] = []
			if e["steps"] is Array:
				var i: int = 1
				for s: Variant in DictUtil.array_of(e["steps"]):
					var step: Dictionary = s if s is Dictionary else {}
					steps.append("%d. %s" % [i, _esc(str(step.get("text", "")))])
					i += 1
			text = "[b]%s[/b]\n" % str(e["name"])
			text += _line("Given by", _name_of(world_id, "characters", str(e["giver"])))
			text += _line("Faction", _name_of(world_id, "factions", str(e["faction"])))
			text += _line("Requires", _names_of(world_id, "quests", e["requires"]))
			text += "\n" + _esc(str(e["description"])) + "\n\n" + "\n".join(steps) + "\n\n"
			text += _line("Reward", _esc(str(e["reward"])))
		"lore":
			text = "[b]%s[/b]  (%s)\n\n%s" % [str(e["name"]), str(e["category"]), _esc(str(e["text"]))]
		"encounters":
			text = _encounter(world_id, e)
	var ov: String = str(e.get("override", ""))
	if not ov.is_empty():
		text += "\n[color=#fbbf24]Overrides %s[/color]" % ov
	var id: String = str(e.get("id", ""))
	if kind != "world" and not id.is_empty():
		var labels: Array[String] = []
		for u: Dictionary in WorldCatalog.references_to(world_id, kind, id):
			labels.append("%s:%s" % [str(u["kind"]), str(u["id"])])
		text += "\n[color=#8ba3b8]Used by:[/color] %s" % (", ".join(labels) if not labels.is_empty() else "nothing yet")
	return text

# Names of the merged `kind` entries whose `key` references `entry`.
static func _list_where(world_id: String, kind: String, key: String, entry: Dictionary) -> String:
	var local: String = str(entry.get("id", ""))
	var target: String = WorldCatalog.qid(world_id, local)
	var found: Array[String] = []
	for other: Dictionary in WorldCatalog.merged_entries(world_id, kind):
		var value: Variant = other.get(key, null)
		var values: Array = value if value is Array else [value]
		for v: Variant in values:
			var hit: String = str(v)
			if not hit.is_empty() and (hit == local or hit == target):
				found.append(str(other.get("name", "")))
				break
	return ", ".join(found) if not found.is_empty() else "-"

static func _character(world_id: String, e: Dictionary) -> String:
	var base: Dictionary = ContentCatalog.get_entry("units", str(e["base_unit"]))
	var text: String = "[b]%s[/b]  %s\n" % [str(e["name"]), _esc(str(e["title"]))]
	text += _line("Role", str(e["role"]))
	text += _line("Faction", _name_of(world_id, "factions", str(e["faction"])))
	text += _line("Home", _name_of(world_id, "locations", str(e["home"])))
	if not str(e["personality"]).is_empty():
		text += _line("Personality", _esc(str(e["personality"])))
	text += "\n" + _esc(str(e["background"])) + "\n\n"
	text += "[color=#38bdf8][b]Plays as[/b][/color] %s [%s]\n" % [str(base.get("name", "(choose a base unit)")), str(e["base_unit"])]
	if base.is_empty():
		return text
	var c: Combatant = ContentFactory.unit(WorldFactory.unit_entry(e), "", 12)
	WorldFactory.apply_character(c, e, world_id)
	var plain: Combatant = ContentFactory.unit(base, "", 12)
	text += _line("Vitals", "HP %d, speed %d, range %d, AP %d" % [c.max_hp, c.speed, c.attack_range, c.max_ap])
	text += _line("Attack power / armor", "%d / %d" % [DamageCalculator.power_of(c, c.attack_range > 1), DamageCalculator.armor_rating(c)])
	var diff: String = WorldFactory.divergence(e, world_id)
	text += _line("Differs from base", diff if not diff.is_empty() else "[color=#4ade80]nothing: identical to the base unit[/color]")
	if not diff.is_empty():
		text += _line("Base unit", "HP %d, power %d, armor %d" % [plain.max_hp, DamageCalculator.power_of(plain, plain.attack_range > 1), DamageCalculator.armor_rating(plain)])
	return text

static func _item(world_id: String, e: Dictionary) -> String:
	var base: Dictionary = ContentCatalog.get_entry("items", str(e["base_item"]))
	var text: String = "[b]%s[/b]\n" % str(e["name"])
	text += _line("Plays as", "%s [%s]" % [str(base.get("name", "(choose a base item)")), str(e["base_item"])])
	text += _line("Original owner", _name_of(world_id, "characters", str(e["owner"])))
	var named: Item = WorldFactory.named_item(e)
	if named is Equipment:
		text += _line("Stats", UiTheme.equipment_summary(named as Equipment, CharacterStats.new()))
	var diff: Array[String] = []
	if e["tuning"] is Dictionary:
		var tuning: Dictionary = e["tuning"]
		for key: Variant in tuning:
			if DictUtil.to_int(tuning[key]) != 0:
				diff.append("%s %+d" % [str(key), DictUtil.to_int(tuning[key])])
	if e["stat_bonuses"] is Dictionary and not DictUtil.dict_of(e["stat_bonuses"]).is_empty():
		diff.append("stats " + str(e["stat_bonuses"]))
	text += _line("Differs from base", ", ".join(diff) if not diff.is_empty() else "[color=#4ade80]nothing: identical to the base item[/color]")
	text += "\n" + _esc(str(e["description"])) + "\n\n[i]" + _esc(str(e["history"])) + "[/i]\n"
	return text

static func _encounter(world_id: String, e: Dictionary) -> String:
	var text: String = "[b]%s[/b]\n" % str(e["name"])
	text += _line("Location", _name_of(world_id, "locations", str(e["location"])))
	text += _line("Rules", "%d rounds max, %d blocks / round, %d AP (+%d / round), zone %s" % [DictUtil.to_int(e["max_rounds"]), DictUtil.to_int(e["total_blocks"]), DictUtil.to_int(e["unit_ap"]), DictUtil.to_int(e["ap_regen"]), CombatZones.zone_name(str(e["zone"]))])
	text += _line("Party", _names_of(world_id, "characters", e["party"]))
	text += _line("Enemies", _names_of(world_id, "characters", e["enemies"]))
	if not DictUtil.array_of(e["wave_units"]).is_empty():
		text += _line("Wave (R%d B%d)" % [DictUtil.to_int(e["wave_round"]), DictUtil.to_int(e["wave_block"])], _names_of(world_id, "characters", e["wave_units"]))
	for phase_line: String in EncounterPhases.summary_lines(e.get("phases", []), func(ref: String) -> String: return _name_of(world_id, "characters", ref)):
		text += _esc(phase_line) + "\n"   # BOSS1 rule 9
	if not DictUtil.array_of(e.get("guests", [])).is_empty():
		text += _line("Guests", _names_of(world_id, "characters", e["guests"]))
	var name_of: Callable = func(ref: String) -> String: return _name_of(world_id, "characters", ref)
	text += _line("Objective", _esc(EncounterObjectives.text_of(e, name_of)))   # OBJ1 rule 12
	text += _line("Deployment", str(e.get("deployment", EncounterObjectives.STANDARD)))
	if not DictUtil.array_of(e["party"]).is_empty() and not DictUtil.array_of(e["enemies"]).is_empty():
		var enc := WorldEncounter.new(world_id, "")
		enc.apply_definition(e)
		enc.build()
		var placed: Array[String] = []
		for c: Combatant in enc.allies + enc.enemies:
			placed.append("%s %s" % [c.name, str(enc.grid.get_position(c)) if enc.grid.has_combatant(c) else ("[color=#f87171](off-field until its phase)[/color]" if enc.is_reserved(c) else "[color=#f87171](off-field until the wave)[/color]")])
		text += _line("Start tiles", ", ".join(placed))
	text += "\n" + _esc(str(e["description"]))
	if not str(e["intro"]).is_empty():
		text += "\n\n[i]" + _esc(str(e["intro"])) + "[/i]"
	return text

# The world as a whole: inheritance, connections and contents.
static func describe_world(world_id: String) -> String:
	var m: Dictionary = WorldCatalog.manifest(world_id)
	if m.is_empty():
		return "[i]New world (not created yet).[/i]"
	var text: String = "[b]%s[/b]  [i]%s[/i]\n" % [str(m["name"]), _esc(str(m["tagline"]))]
	var chain: Array[String] = WorldCatalog.lineage(world_id)
	text += _line("Inheritance", " → ".join(chain) if chain.size() > 1 else "none (a root world)")
	var connects: Array[String] = WorldCatalog._strings(m["connects"])
	text += _line("Connects to", ", ".join(connects) if not connects.is_empty() else "-")
	var below: Array[String] = WorldCatalog.descendants(world_id)
	text += _line("Extended by", ", ".join(below) if not below.is_empty() else "-")
	var counts: Array[String] = []
	for kind: String in WorldSchema.KINDS:
		var own: int = WorldCatalog.own_entries(world_id, kind).size()
		var all: int = WorldCatalog.merged_entries(world_id, kind).size()
		counts.append("%s %d%s" % [kind, own, (" (+%d inherited)" % (all - own)) if all != own else ""])
	text += _line("Entries", ", ".join(counts))
	text += "\n" + _esc(str(m["description"]))
	return text

static func _passive_names(ids: Variant) -> String:
	var names: PackedStringArray = PackedStringArray()
	for id: Variant in DictUtil.array_of(ids):
		names.append(str(ContentCatalog.get_entry("passives", str(id)).get("name", str(id))))
	return ", ".join(names) if not names.is_empty() else "-"
