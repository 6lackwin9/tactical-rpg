class_name WorldSchema
extends RefCounted

# ==============================================================================
# Tactical RPG - World Schema (the lore layer on top of the content catalog)
# A world is a folder of lore entries: regions, locations, factions, characters,
# named items, disciplines, quests, lore entries, encounters and loot (the world
# loot table quests offer as special rewards). Entries never
# carry gameplay of their own: they REFERENCE base assets from the Content Forge
# catalog (`base:units`, `base:items`...) and add lore (names, backgrounds, art,
# factions), plus optional explicit tuning deltas.
#
# Field format is the ContentSchema one (type, default, min/max, options, when,
# help...). `ref` names what a ref/refs field points at:
#   "regions" ...      another entry kind of the same world (or an ancestor world)
#   "base:units" ...   a content-catalog kind
#   "worlds"           another world
# Entries in other worlds are written "world:id"; unqualified refs resolve in the
# entry's own world first, then its ancestors (nearest wins).
# Every kind has `override` (the qualified id of an ancestor entry it replaces) and `tags`.
# ==============================================================================

const KINDS: Array[String] = ["regions", "locations", "factions", "characters", "items", "disciplines", "quests", "lore", "encounters", "loot"]

# Deltas a character may apply to its base unit (flat unless _pct).
const TUNING_KEYS: Array[String] = ["hp_pct", "power", "armor", "speed", "ap"]
# Deltas a named item may apply to its base item.
const ITEM_TUNING_KEYS: Array[String] = ["power", "mitigation", "burden"]

static func kind_label(kind: String) -> String:
	match kind:
		"world":
			return "Overview"
		"npcs":
			return "NPCs"
	return kind.capitalize()

static func kind_icon(kind: String) -> String:
	match kind:
		"world":
			return "🌍"
		"regions":
			return "🗺"
		"locations":
			return "📍"
		"factions":
			return "⚑"
		"characters":
			return "🧑"
		"items":
			return "🗡"
		"disciplines":
			return "✦"
		"quests":
			return "📜"
		"lore":
			return "📖"
		"encounters":
			return "⚔"
		"loot":
			return "🎁"
	return "•"

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	var f: Dictionary = {"key": key, "label": label, "type": type, "default": default}
	f.merge(extra, true)
	return f

static func fields(kind: String) -> Array[Dictionary]:
	var list: Array[Dictionary] = []
	match kind:
		"world":
			list = _world_fields()
		"regions":
			list = _region_fields()
		"locations":
			list = _location_fields()
		"factions":
			list = _faction_fields()
		"characters":
			list = _character_fields()
		"items":
			list = _item_fields()
		"disciplines":
			list = _discipline_fields()
		"quests":
			list = _quest_fields()
		"lore":
			list = _lore_fields()
		"encounters":
			list = _encounter_fields()
		"loot":
			list = _loot_fields()
		_:
			return list
	if kind != "world":
		list.append(_f("override", "Overrides", "string", "", {"help": "Qualified id (world:id) of an ancestor world's entry this one replaces. Everything that referred to it now refers to this entry."}))
		list.append(_f("tags", "Tags", "strings", [], {"help": "Free-form tags for searching and grouping."}))
	return list

static func field(kind: String, key: String) -> Dictionary:
	for f: Dictionary in fields(kind):
		if str(f.get("key", "")) == key:
			return f
	return {}

static func _world_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", "", {"help": "Folder name and namespace of every entry (world:id)."}),
		_f("name", "Name", "string", "New World"),
		_f("tagline", "Tagline", "string", ""),
		_f("description", "Description", "text", ""),
		_f("extends", "Extends", "refs", [], {"ref": "worlds", "help": "Parent worlds: this world inherits all their entries (later parents override earlier ones). Order matters."}),
		_f("connects", "Connects to", "refs", [], {"ref": "worlds", "help": "Worlds this one links to without inheriting (locations may exit into them)."}),
	]

static func _region_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Region"),
		_f("description", "Description", "text", ""),
		_f("climate", "Climate / terrain", "string", ""),
		_f("parent", "Part of", "ref", "", {"ref": "regions", "optional": true}),
		_f("faction", "Controlled by", "ref", "", {"ref": "factions", "optional": true}),
	]

static func _location_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Location"),
		_f("description", "Description", "text", ""),
		_f("kind", "Kind", "enum", "wild", {"options": ["town", "ruin", "wild", "fortress", "gate", "camp", "dungeon"]}),
		_f("region", "Region", "ref", "", {"ref": "regions", "optional": true}),
		_f("exits", "Exits", "json", [], {"help": "[{\"to\": \"location\", \"label\": \"north road\"}]; `to` may be world:id to cross into a connected world."}),
		_f("encounters", "Encounters here", "refs", [], {"ref": "encounters"}),
	]

static func _faction_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Faction"),
		_f("motto", "Motto", "string", ""),
		_f("description", "Description", "text", ""),
		_f("home", "Home region", "ref", "", {"ref": "regions", "optional": true}),
		_f("relations", "Relations", "json", {}, {"help": "{\"faction_id\": \"ally|rival|enemy|neutral\"}"}),
		_f("grants", "Member passives", "refs", [], {"ref": "base:passives", "help": "Every member of the faction gets these base passives."}),
	]

static func _character_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Character"),
		_f("title", "Title", "string", "", {"help": "e.g. Captain of the Ashen Guard."}),
		_f("role", "Role", "enum", "npc", {"options": ["party", "npc", "enemy"]}),
		_f("base_unit", "Plays as (base unit)", "ref", "", {"ref": "base:units", "help": "The Content Forge unit this character behaves as: vitals, class, loadout, skills and AI come from it."}),
		_f("art", "Art", "enum", "", {"options_from": "art_inherit", "help": "Token / portrait template (empty = the base unit's)."}),
		_f("background", "Background", "text", ""),
		_f("personality", "Personality", "string", ""),
		_f("faction", "Faction", "ref", "", {"ref": "factions", "optional": true}),
		_f("home", "Home", "ref", "", {"ref": "locations", "optional": true}),
		_f("disciplines", "Disciplines", "refs", [], {"ref": "disciplines", "help": "Trained disciplines: each grants its passives."}),
		_f("passives", "Passives", "refs", [], {"ref": "base:passives", "help": "Traits from the character's background (also granted: its faction's, its disciplines' and its named items')."}),
		_f("named_items", "Named items", "refs", [], {"ref": "items", "help": "Equipped on top of the base loadout, each in its own slot."}),
		_f("tuning", "Tuning deltas", "json", {}, {"help": "Explicit gameplay deltas vs the base unit: {\"hp_pct\": 10, \"power\": 3, \"armor\": 2, \"speed\": 1, \"ap\": 2}. Empty = identical to the base."}),
	]

static func _item_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Named Item"),
		_f("base_item", "Based on (base item)", "ref", "", {"ref": "base:items", "help": "The Content Forge item this is a named version of."}),
		_f("description", "Description", "text", ""),
		_f("history", "History", "text", ""),
		_f("owner", "Original owner", "ref", "", {"ref": "characters", "optional": true}),
		_f("tuning", "Tuning deltas", "json", {}, {"help": "{\"power\": 2, \"mitigation\": 1, \"burden\": -1}. Empty = identical to the base item."}),
		_f("stat_bonuses", "Extra stat bonuses", "stats", {}, {"keys": ContentSchema.STAT_KEYS}),
		_f("grants", "Wearer passives", "refs", [], {"ref": "base:passives", "help": "Base passives granted while the item is worn by a character."}),
	]

static func _discipline_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Discipline"),
		_f("description", "Description", "text", ""),
		_f("passives", "Grants passives", "refs", [], {"ref": "base:passives", "help": "The base passives this discipline teaches (gear training, stat edges, resistances...)."}),
		_f("teacher", "Taught by", "ref", "", {"ref": "characters", "optional": true}),
		_f("faction", "Faction", "ref", "", {"ref": "factions", "optional": true}),
	]

static func _quest_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Quest"),
		_f("description", "Summary", "text", ""),
		_f("giver", "Given by", "ref", "", {"ref": "characters", "optional": true}),
		_f("faction", "Faction", "ref", "", {"ref": "factions", "optional": true}),
		_f("region", "Region", "ref", "", {"ref": "regions", "optional": true}),
		_f("requires", "Requires quests", "refs", [], {"ref": "quests"}),
		_f("steps", "Steps", "json", [], {"help": "[{\"text\": \"Reach the gate\", \"location\": \"id\", \"encounter\": \"id\"}]"}),
		_f("reward", "Reward", "text", ""),
		# Runtime binding (M4-L). Ranges are checked by WorldCatalog validation, not clamped here, so bad values report.
		_f("template", "Template", "string", "", {"help": "QuestCatalog template id (objectives + generic rewards). Empty = lore-only quest; with a template, steps bind one per objective."}),
		_f("turn_in", "Turn in to", "ref", "", {"ref": "characters", "optional": true, "help": "Empty = the giver."}),
		_f("requires_flags", "Requires flags", "json", [], {"help": "[{\"key\": \"flag.key\", \"value\": true}]; all must hold to start."}),
		_f("complete_flags", "Sets flags on completion", "json", [], {"help": "[{\"key\": \"flag.key\", \"value\": true}], set in order."}),
		_f("auto_start", "Auto start", "bool", false, {"help": "Starts by itself once available (needs a template)."}),
		_f("links", "Links", "json", {}, {"help": "Metadata only: {\"follows\": [], \"leads_to\": [], \"related_quests\": [], \"related_characters\": [], \"related_locations\": [], \"related_factions\": []}."}),
		_f("variants", "Variants", "json", [], {"help": "[{\"id\", \"when_flags\": [...], \"unless_flags\": [...], \"set\": {giver, turn_in, description, reward, rewards_from, steps, links, available, loot_pool, loot_picks}}], applied in order."}),
		_f("loot_pool", "Loot pool", "json", [], {"help": "World loot refs offered as a pick on completion (on top of the generic rewards)."}),
		_f("loot_picks", "Loot picks", "int", 1, {"help": "How many pool entries the player may take (0..3)."}),
	]

# World loot table: special rewards quests offer from their loot_pool; each is claimed once per save.
static func _loot_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Loot"),
		_f("description", "Description", "text", ""),
		_f("reward_kind", "Reward kind", "enum", "item", {"options": ["item", "passive", "skill"]}),
		_f("ref", "Grants (base id)", "string", "", {"help": "Base item id, passive id or skill family head (not a basic skill)."}),
		_f("count", "Count", "int", 1, {"help": "Items: 1..99; passives and skills: 1."}),
		_f("rarity", "Rarity", "enum", "rare", {"options": ["common", "rare", "unique"]}),
		_f("requires_flags", "Requires flags", "json", [], {"help": "[{\"key\", \"value\"}]; not offered while any fails."}),
	]

static func _lore_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Title", "string", "New Lore Entry"),
		_f("category", "Category", "enum", "history", {"options": ["history", "myth", "place", "person", "rumor", "item"]}),
		_f("text", "Text", "text", ""),
	]

static func _encounter_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Encounter"),
		_f("description", "Description", "text", ""),
		_f("intro", "Intro (shown when the battle starts)", "text", ""),
		_f("location", "Location", "ref", "", {"ref": "locations", "optional": true}),
		_f("zone", "Zone", "enum", "stone_arena", {"options_from": "zones"}),
		_f("arena", "Arena", "enum", "CLOSED", {"options": ["CLOSED", "OPEN"]}),
		_f("exits", "Exit tiles", "json", [[8, 4]]),
		_f("max_rounds", "Max rounds", "int", 6, {"min": 1, "max": 30}),
		_f("total_blocks", "Blocks / round", "int", 6, {"min": 3, "max": 10}),
		_f("ap_regen", "AP regen / round", "int", 4, {"min": -1, "max": 20}),
		_f("unit_ap", "Unit AP", "int", 12, {"min": 0, "max": 40}),
		_f("rng_seed", "RNG seed", "int", 1337, {"min": 0, "max": 2147483647}),
		_f("party", "Party", "refs", [], {"ref": "characters", "ref_filter": {"role": "party"}}),
		_f("enemies", "Enemies", "refs", [], {"ref": "characters", "ref_filter": {"role": "enemy"}, "dupes": true}),
		_f("wave_units", "Reinforcements", "refs", [], {"ref": "characters", "ref_filter": {"role": "enemy"}, "dupes": true}),
		_f("wave_round", "Wave round", "int", 1, {"min": 1, "max": 30}),
		_f("wave_block", "Wave block", "int", 4, {"min": 1, "max": 10}),
		_f("wave_label", "Wave label", "string", "Reinforcements arrive!"),
	]

# ------------------------------------------------------------------------------
# Options, defaults, normalising (same rules as ContentSchema)
# ------------------------------------------------------------------------------

static func options_of(f: Dictionary) -> Array[String]:
	if str(f.get("options_from", "")) == "art_inherit":
		var out: Array[String] = [""]
		out.append_array(UnitArt.CLASS_KEYS)
		return out
	return ContentSchema.options_of(f)

static func defaults(kind: String) -> Dictionary:
	var d: Dictionary = {}
	for f: Dictionary in fields(kind):
		d[str(f["key"])] = ContentSchema._copy(f["default"])
	return d

static func normalize(kind: String, entry: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	for f: Dictionary in fields(kind):
		var key: String = str(f["key"])
		out[key] = ContentSchema.coerce(f, entry.get(key, f["default"]))
	for key: Variant in entry:
		if not out.has(key) and not str(key).begins_with("_"):
			out[key] = ContentSchema._copy(entry[key])
	return out

# The entry as written to disk: fields hidden by "when" and empty `override` are left out.
static func to_storage(kind: String, entry: Dictionary) -> Dictionary:
	var full: Dictionary = normalize(kind, entry)
	var out: Dictionary = {}
	for f: Dictionary in fields(kind):
		var key: String = str(f["key"])
		if key == "override" and str(full[key]).is_empty():
			continue
		out[key] = full[key]
	for key: Variant in full:
		if not out.has(key) and field(kind, str(key)).is_empty():
			out[key] = full[key]
	return out

# Ref fields of a kind: [{"key", "ref", "multi", "filter"}].
static func ref_fields(kind: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for f: Dictionary in fields(kind):
		var ref_kind: String = str(f.get("ref", ""))
		if not ref_kind.is_empty():
			out.append({"key": str(f["key"]), "ref": ref_kind, "multi": str(f["type"]) == "refs", "field": f})
	return out
