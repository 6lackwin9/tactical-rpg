class_name WorldSchema
extends RefCounted

# ==============================================================================
# Tactical RPG - World Schema (the lore layer on top of the content catalog)
# A world is a folder of lore entries: regions, locations, factions, characters,
# named items, disciplines, quests, lore entries, encounters, loot (the world
# loot table quests offer as special rewards), dialogue trees, shops and routes (the world map). Entries never
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

const KINDS: Array[String] = ["regions", "locations", "factions", "characters", "items", "disciplines", "quests", "lore", "encounters", "loot", "dialogues", "shops", "routes"]

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
		"dialogues":
			return "💬"
		"shops":
			return "🛒"
		"routes":
			return "🛤"
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
		"dialogues":
			list = _dialogue_fields()
		"shops":
			list = _shop_fields()
		"routes":
			list = _route_fields()
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
		_f("start_location", "Start location", "ref", "", {"ref": "locations", "optional": true, "help": "Where a new game enters this world. Empty = the nearest parent world's."}),
		# Starting party (UI1-L, S13): fixed per world; WorldCatalog.manifest_errors checks it, start_party() resolves it.
		_f("start_party", "Start party", "refs", [], {"ref": "base:units", "ref_filter": {"side": "party"}, "help": "Party-side units a new game starts with (at most 4). Empty = the nearest parent world's."}),
		# Dialogue and gold (DLG1-L, S11); ranges are checked by WorldCatalog.manifest_errors, not clamped here.
		_f("start_gold", "Start gold", "int", -1, {"help": "Party gold of a new game (-1 = the nearest parent world's, else 0)."}),
		# Travel supplies (TRAV2-L, S10); the range is checked by WorldCatalog.manifest_errors.
		_f("start_supplies", "Start supplies", "int", -1, {"help": "Party supplies of a new game (-1 = the nearest parent world's, else 10); 0..99."}),
		_f("service_lines", "Service lines", "json", {}, {"help": "{\"greeting\": \"{npc} nods.\"}: overrides of the base service lines (data/dialogue/services.json), merged along the lineage (nearest wins)."}),
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
		# Region map binding (EXP1-L, S9); MapRules.location_errors validates them.
		_f("map", "Map template", "string", "", {"help": "MapCatalog template id (data/maps). Empty = a menu location with no walkable map."}),
		_f("zone_bindings", "Zone bindings", "json", {}, {"help": "{\"zone_id\": {\"encounters\": [[\"enc\", 3]], \"elites\": [], \"elite_pct\": 10, \"steps_min\": 12, \"steps_max\": 20}} for DANGER zones of the map."}),
		_f("markers", "Markers", "json", [], {"help": "[{\"id\", \"kind\": exit|encounter|roamer|npc|chest|lore|point, \"anchor\", ...}] placed on the map's anchors (a roamer walks a path instead)."}),
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
		# Dialogue and presence (DLG1-L, S11).
		_f("dialogue", "Dialogue", "ref", "", {"ref": "dialogues", "optional": true, "help": "The tree shown when the party talks to this character. Empty = service choices only."}),
		_f("gone_flags", "Gone when", "json", [], {"help": "[{\"key\": \"flag.key\", \"value\": true}]; ANY holding = the character is gone (no dialogue, no npc markers, shops closed)."}),
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
		# Reputation (REP1-L, S11): applied once on completion; not a variant field.
		_f("reputation", "Reputation", "json", {}, {"help": "{\"faction\": 10}: reputation deltas (-50..50, not 0, at most 4 factions) applied when the quest completes (needs a template)."}),
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

# Dialogue trees (DLG1-L, S11); DialogueRules.tree_errors validates the nodes.
static func _dialogue_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Dialogue"),
		_f("speaker", "Speaker", "ref", "", {"ref": "characters", "optional": true, "help": "Default speaker of every node. Empty = the character talked to."}),
		_f("start", "Start node", "string", "", {"help": "Id of the first node shown."}),
		_f("nodes", "Nodes", "json", [], {"help": "[{\"id\", \"speaker\", \"text\", \"choices\": [{\"text\", \"to\", \"requires\", \"unless\", \"effects\", \"show_locked\"}], \"effects\", \"next\", \"end\"}]; a node has exactly one of choices, next or end."}),
		_f("services", "Service choices", "bool", true, {"help": "Append the generated quest offer / turn-in / shop / farewell choices to the start node."}),
	]

# Shops (DLG1-L, S11); ShopRules.shop_errors validates them. Multipliers are checked there, not clamped here.
static func _shop_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Shop"),
		_f("keeper", "Keeper", "ref", "", {"ref": "characters", "help": "The character who runs the shop (gone keeper = closed shop)."}),
		_f("stock", "Stock", "json", [], {"help": "[{\"item\": \"base item id\", \"count\": -1, \"price\": -1, \"requires_flags\": []}]; count -1 = unlimited, price -1 = derived from the item value."}),
		_f("loot", "Loot for sale", "json", [], {"help": "[{\"loot\": \"world loot ref\", \"price\": 150}]; item loot only, sold once (claimed)."}),
		_f("buy_mult", "Buy multiplier", "float", 1.0, {"help": "0.5..3.0: derived buy price = ceil(value x mult)."}),
		_f("supply_price", "Supply price", "int", 0, {"help": "Gold per travel supply (fixed, unlimited, ignores multipliers and reputation); 0 = sells no supplies; 1..999."}),
		_f("sell_mult", "Sell multiplier", "float", 0.5, {"help": "0.0..1.0: sell price = floor(value x mult); 0 = buys nothing."}),
		_f("buys", "Buys types", "json", ShopRules.buy_types(), {"help": "Item types the shop buys (every item type but QUEST by default; never QUEST)."}),
		_f("requires_flags", "Open when", "json", [], {"help": "[{\"key\", \"value\"}]; the shop is closed while any fails."}),
	]

# World map routes (TRAV1-L, S10); TravelRules.route_errors validates them. Ranges are checked there, not clamped here.
static func _route_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Route"),
		_f("from", "From", "ref", "", {"ref": "locations", "help": "Where the route starts (a world map node)."}),
		_f("to", "To", "ref", "", {"ref": "locations", "help": "Where it leads; may be world:id into a connected world."}),
		_f("two_way", "Two way", "bool", true, {"help": "Off = travel only from -> to."}),
		_f("time", "Travel time", "int", 3, {"help": "1..20 world clock units per leg."}),
		_f("zone", "Fight zone", "string", "", {"help": "Combat zone of road fights (empty = each encounter's own)."}),
		_f("encounter_pct", "Encounter chance %", "int", 20, {"help": "0..100 per leg on the first traversal."}),
		_f("repeat_pct", "Repeat chance %", "int", -1, {"help": "-1 = half the encounter chance (rounded down); else 0..100 per leg once traversed."}),
		_f("encounters", "Encounters", "json", [], {"help": "[[\"encounter\", weight 1..100]...], at most 8."}),
		_f("elites", "Elites", "json", [], {"help": "Same shape; the elite chance picks from these."}),
		_f("elite_pct", "Elite chance %", "int", 5, {"help": "0..100."}),
		_f("scripted", "Scripted fights", "json", [], {"help": "[{\"id\", \"encounter\", \"requires_flags\", \"unless_flags\", \"sets_flags\"}], at most 4; one-shot, before the roll."}),
		_f("requires_flags", "Open when", "json", [], {"help": "[{\"key\", \"value\"}]; any failing = the route is closed."}),
		_f("unless_flags", "Closed when", "json", [], {"help": "[{\"key\", \"value\"}]; any holding = the route is closed."}),
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
		_f("exits", "Exit tiles", "json", [[11, 4]]),
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
		out.append_array(AssetRegistry.keys("unit"))
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
