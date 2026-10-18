class_name DialogueRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Dialogue Rules (S11, spec DLG1-L)
# Constants and pure validation for world dialogue trees (WorldCatalog kind `dialogues`), dialogue choice
# conditions and effects, base service lines (ServiceCatalog, manifest `service_lines`) and the gone-NPC quest
# lint, plus the pure runtime reads (presence `is_gone`, conditions, best stat, start gold, service lines, the journal
# turn-in rule) that DialogueRunner, ExploreRunner and ShopRules use. No state, no RNG.
# A tree is {id, name, speaker, start, nodes, services}; a node has exactly one of non-empty `choices`, a `next`
# node id or `end: true`. Node effects are set_flag / add_flag / give only; choice effects are all-or-nothing at runtime.
# Markers, quests, loot, routes and `gone_flags` keep the M4 flag-only condition grammar (QuestRules), with the REP1
# operators (`op`); dialogue flag conditions delegate to it. `add_flag` (REP1-L) adds a delta to an int flag.
# ==============================================================================

const MAX_NODES: int = 64
const MAX_CHOICES: int = 6
const MAX_EFFECTS: int = 8
const MAX_TEXT: int = 1000

const NODE_KEYS: Array[String] = ["id", "speaker", "text", "choices", "effects", "next", "end"]
const CHOICE_KEYS: Array[String] = ["text", "to", "requires", "unless", "effects", "show_locked"]

# Condition kind -> its keys besides `kind`, with defaults (a missing required key reads as its default and fails).
const CONDITION_KINDS: Dictionary = {
	"flag": {"key": "", "value": true, "op": "=="},
	"quest": {"quest": "", "status": ""},
	"items": {"item": "", "count": 1},
	"gold": {"min": null},
	"stat": {"stat": "", "min": null},
}
# QuestRunner.status values a quest condition may name.
const QUEST_STATUSES: Array[String] = ["locked", "available", "unavailable", "active", "ready", "blocked", "completed"]
const GOLD_CONDITION_MAX: int = 9999999
const STAT_MIN: int = 1
const STAT_MAX: int = 30

# Effect -> its keys besides `effect`, with defaults.
const EFFECTS: Dictionary = {
	"set_flag": {"key": "", "value": true},
	"add_flag": {"key": "", "delta": null, "min": null, "max": null},
	"start_quest": {"quest": ""},
	"turn_in": {"quest": ""},
	"give": {"items": {}, "gold": 0},
	"take": {"items": {}, "gold": 0},
	"open_shop": {"shop": ""},
	"fight": {"encounter": ""},
	"recruit": {"character": ""},   # RECRUIT1-L 1.4: choices only, at most one per choice
}
const NODE_EFFECTS: Array[String] = ["set_flag", "add_flag", "give"]
const EFFECT_GOLD_MAX: int = 99999
const ADD_DELTA_MAX: int = 100   # |delta| of add_flag (REP1-L 1.3)

# Json ref keys (WorldCatalog reference scans and renames): condition / effect key -> the world kind it names.
const CONDITION_REF_KEYS: Dictionary = {"quest": "quests"}
const EFFECT_REF_KEYS: Dictionary = {"quest": "quests", "shop": "shops", "encounter": "encounters", "character": "characters"}

# Base service lines (data/dialogue/services.json ids, in service order) and the fallback the runtime uses for a
# missing one. Lore-free; tokens are replaced when a node is shown.
const SERVICE_IDS: Array[String] = ["greeting", "offer", "accept", "decline", "turn_in", "shop", "farewell"]
const DEFAULT_LINES: Dictionary = {
	"greeting": "{npc} looks up.",
	"offer": "About {quest}...",
	"accept": "We will do it.",
	"decline": "Not now.",
	"turn_in": "We finished: {quest}.",
	"shop": "Show me your wares.",
	"farewell": "Farewell.",
}
const TOKENS: Array[String] = ["npc", "quest", "shop", "gold"]
const TEXT_TOKENS: Array[String] = ["npc", "gold"]   # authored node and choice texts (DLG1 build choices)

# ------------------------------------------------------------------------------
# Trees (called by WorldCatalog.validate_entry for kind `dialogues`)
# ------------------------------------------------------------------------------

static func tree_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var raw_nodes: Variant = e.get("nodes", [])
	if not (raw_nodes is Array):
		errors.append("Nodes must be a list of node objects.")
		return errors
	var nodes: Array = DictUtil.array_of(JsonUtil.normalize(raw_nodes))
	if nodes.is_empty() or nodes.size() > MAX_NODES:
		errors.append("A dialogue holds 1..%d nodes." % MAX_NODES)
	var by_id: Dictionary = _nodes_by_id(nodes)
	var seen: Dictionary = {}
	for i: int in range(nodes.size()):
		var nid: String = str(DictUtil.dict_of(nodes[i]).get("id", ""))
		if nodes[i] is Dictionary and JsonUtil.id_ok(nid):
			if seen.has(nid):
				errors.append("Node id '%s' is used twice." % nid)
			seen[nid] = true
		errors.append_array(_node_errors(world_id, i, nodes[i], by_id))
	var start: String = str(e.get("start", ""))
	if not by_id.has(start):
		errors.append("Start node '%s' does not exist." % start)
	elif DictUtil.to_bool(e.get("services", true)) and DictUtil.array_of(DictUtil.dict_of(by_id[start]).get("choices", [])).is_empty():
		errors.append("With services on, the start node needs choices.")
	return errors

# Soft problems of one tree: nodes no `to` / `next` reaches from the start node, and unguarded `give` nodes on a cycle.
static func tree_warnings(e: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var nodes: Array = DictUtil.array_of(JsonUtil.normalize(e.get("nodes", [])))
	var by_id: Dictionary = _nodes_by_id(nodes)
	var start: String = str(e.get("start", ""))
	if not by_id.has(start):
		return out   # an error already
	var edges: Dictionary = _edges(by_id, start, DictUtil.to_bool(e.get("services", true)))
	var reached: Dictionary = _reach(edges, start)
	reached[start] = true
	for n: Variant in nodes:
		var nid: String = str(DictUtil.dict_of(n).get("id", ""))
		if by_id.has(nid) and not reached.has(nid):
			out.append("Node '%s' is not reachable from the start node." % nid)
	for nid: Variant in by_id:
		var node: Dictionary = by_id[nid]
		if not _gives(node) or not _reach(edges, str(nid)).has(nid) or _guarded(edges, by_id, str(nid)):
			continue
		out.append("Node '%s' gives items or gold and sits on a dialogue cycle (it can be shown again and again); guard every way in with unless + set_flag." % str(nid))
	return out

# Node id -> [{to, choice ({} for a `next` edge or the generated service edge)}]. A choice with open_shop and `to` ""
# returns to its own node (close_shop); with services on the generated choices lead back to the start node.
static func _edges(by_id: Dictionary, start: String, services: bool) -> Dictionary:
	var out: Dictionary = {}
	for nid: Variant in by_id:
		var node: Dictionary = by_id[nid]
		var list: Array[Dictionary] = []
		var next: Variant = node.get("next", "")
		if next is String and by_id.has(str(next)):
			list.append({"to": str(next), "choice": {}})
		for c: Variant in DictUtil.array_of(node.get("choices", [])):
			var choice: Dictionary = DictUtil.dict_of(c)
			var to: String = str(choice.get("to", ""))
			if to.is_empty() and _has_effect(choice.get("effects", []), "open_shop"):
				to = str(nid)
			if by_id.has(to):
				list.append({"to": to, "choice": choice})
		if services and str(nid) == start:
			list.append({"to": start, "choice": {}})
		out[str(nid)] = list
	return out

# Every node reachable in one or more steps from `from` (`from` itself only through a cycle).
static func _reach(edges: Dictionary, from: String) -> Dictionary:
	var seen: Dictionary = {}
	var queue: Array[String] = [from]
	while not queue.is_empty():
		var at: String = queue.pop_front()
		for edge: Variant in DictUtil.array_of(edges.get(at, [])):
			var to: String = str(DictUtil.dict_of(edge).get("to", ""))
			if not seen.has(to):
				seen[to] = true
				queue.append(to)
	return seen

static func _gives(node: Dictionary) -> bool:
	return _has_effect(node.get("effects", []), "give")

static func _has_effect(raw: Variant, effect: String) -> bool:
	for fx: Variant in DictUtil.array_of(raw):
		if str(DictUtil.dict_of(fx).get("effect", "")) == effect:
			return true
	return false

# True when every way into `nid` is a choice whose `unless` holds a flag condition that the choice or the node itself
# sets (set_flag with the same key and value): the repeat-once pattern, so the gift is shown once.
static func _guarded(edges: Dictionary, by_id: Dictionary, nid: String) -> bool:
	var node: Dictionary = by_id[nid]
	for from: Variant in edges:
		for edge: Variant in DictUtil.array_of(edges[from]):
			var ed: Dictionary = DictUtil.dict_of(edge)
			if str(ed.get("to", "")) != nid:
				continue
			var choice: Dictionary = DictUtil.dict_of(ed.get("choice", {}))
			if choice.is_empty():
				return false
			var sets: Array = DictUtil.array_of(node.get("effects", [])) + DictUtil.array_of(choice.get("effects", []))
			var ok: bool = false
			for c: Variant in DictUtil.array_of(choice.get("unless", [])):
				var cond: Dictionary = DictUtil.dict_of(c)
				if str(cond.get("kind", "flag")) != "flag":
					continue
				for fx: Variant in sets:
					var f: Dictionary = DictUtil.dict_of(fx)
					if str(f.get("effect", "")) == "set_flag" and _same_condition(f, cond):
						ok = true
			if not ok:
				return false
	return true

# Tree warnings of every OWN dialogue of `world_id`, prefixed with the dialogue's qualified id.
static func dialogue_warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in WorldCatalog.own_entries(world_id, "dialogues"):
		for w: String in tree_warnings(e):
			out.append("%s: %s" % [str(e["_qid"]), w])
	return out

# Node id -> node for every node object with a valid id (the first one wins on duplicates).
static func _nodes_by_id(nodes: Array) -> Dictionary:
	var out: Dictionary = {}
	for n: Variant in nodes:
		if not (n is Dictionary):
			continue
		var node: Dictionary = n
		var nid: Variant = node.get("id", "")
		if nid is String and JsonUtil.id_ok(str(nid)) and not out.has(nid):
			out[str(nid)] = node
	return out

static func _node_errors(world_id: String, index: int, raw: Variant, by_id: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("Node %d must be an object." % (index + 1))
		return errors
	var node: Dictionary = raw
	var nid: String = str(node.get("id", ""))
	var label: String = "Node '%s'" % nid
	for key: Variant in node:
		if not NODE_KEYS.has(str(key)):
			errors.append("Node '%s': unknown key '%s'." % [nid, str(key)])
	if not (node.get("id", null) is String) or not JsonUtil.id_ok(nid):
		errors.append("Node %d: id '%s' must be lowercase snake_case (a-z, 0-9, _)." % [index + 1, nid])
		label = "Node %d" % (index + 1)
	var speaker: Variant = node.get("speaker", "")
	if not (speaker is String):
		errors.append("%s: speaker must be a character ref or \"\"." % label)
	elif not str(speaker).is_empty():
		errors.append_array(WorldCatalog.ref_errors(world_id, label + " speaker", "characters", str(speaker)))
	errors.append_array(_text_errors(node.get("text", ""), label))
	var choices_raw: Variant = node.get("choices", [])
	var next_raw: Variant = node.get("next", "")
	var end_raw: Variant = node.get("end", false)
	if not (choices_raw is Array):
		errors.append("%s: choices must be a list." % label)
	if not (next_raw is String):
		errors.append("%s: next must be a node id." % label)
	if not (end_raw is bool):
		errors.append("%s: end must be true or false." % label)
	var choices: Array = DictUtil.array_of(choices_raw)
	var has_next: bool = next_raw is String and not str(next_raw).is_empty()
	var has_end: bool = end_raw is bool and DictUtil.to_bool(end_raw)
	if int(not choices.is_empty()) + int(has_next) + int(has_end) != 1:
		errors.append("Node '%s' needs exactly one of choices, next or end." % nid)
	if has_next and not by_id.has(str(next_raw)):
		errors.append("%s: next '%s' is not a node of this dialogue." % [label, str(next_raw)])
	if choices.size() > MAX_CHOICES:
		errors.append("%s holds 1..%d choices (has %d)." % [label, MAX_CHOICES, choices.size()])
	for j: int in range(choices.size()):
		errors.append_array(_choice_errors(world_id, "%s choice %d" % [label, j + 1], choices[j], by_id))
	errors.append_array(effects_errors(world_id, node.get("effects", []), label, true))
	return errors

# Authored node / choice text: non-empty, at most MAX_TEXT, tokens TEXT_TOKENS only ({quest} and {shop} belong to the
# generated service lines).
static func _text_errors(raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is String) or str(raw).strip_edges().is_empty():
		errors.append("%s needs text." % label)
		return errors
	if str(raw).length() > MAX_TEXT:
		errors.append("%s: text is longer than %d characters." % [label, MAX_TEXT])
	for token: String in tokens_of(str(raw)):
		if not TEXT_TOKENS.has(token):
			errors.append("%s: Unknown token '%s' (authored texts allow {npc} and {gold})." % [label, token])
	return errors

static func _choice_errors(world_id: String, label: String, raw: Variant, by_id: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("%s must be an object {\"text\", \"to\", ...}." % label)
		return errors
	var c: Dictionary = raw
	for key: Variant in c:
		if not CHOICE_KEYS.has(str(key)):
			errors.append("%s: unknown key '%s'." % [label, str(key)])
	errors.append_array(_text_errors(c.get("text", ""), label))
	var to: Variant = c.get("to", "")
	if not (to is String):
		errors.append("%s: to must be a node id or \"\" (end)." % label)
	elif not str(to).is_empty() and not by_id.has(str(to)):
		errors.append("%s: to '%s' is not a node of this dialogue." % [label, str(to)])
	errors.append_array(conditions_errors(world_id, c.get("requires", []), label + " requires"))
	errors.append_array(conditions_errors(world_id, c.get("unless", []), label + " unless"))
	if not (c.get("show_locked", false) is bool):
		errors.append("%s: show_locked must be true or false." % label)
	errors.append_array(effects_errors(world_id, c.get("effects", []), label, false))
	return errors

# ------------------------------------------------------------------------------
# Conditions (dialogue choices only)
# ------------------------------------------------------------------------------

static func conditions_errors(world_id: String, raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("%s must be a list of conditions." % label)
		return errors
	var list: Array = DictUtil.array_of(JsonUtil.normalize(raw))
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("%s %d must be an object {\"kind\", ...}." % [label, i + 1])
			continue
		var cond: Dictionary = list[i]
		var kind: String = str(cond.get("kind", "flag"))
		if not (cond.get("kind", "flag") is String) or not CONDITION_KINDS.has(kind):
			errors.append("%s %d: unknown condition kind '%s' (use %s)." % [label, i + 1, kind, ", ".join(PackedStringArray(CONDITION_KINDS.keys()))])
			continue
		var cl: String = "%s %d (%s)" % [label, i + 1, kind]
		var allowed: Dictionary = CONDITION_KINDS[kind]
		for key: Variant in cond:
			if str(key) != "kind" and not allowed.has(str(key)):
				errors.append("%s: unknown key '%s'." % [cl, str(key)])
		errors.append_array(_condition_errors(world_id, kind, cond, cl))
	return errors

static func _condition_errors(world_id: String, kind: String, cond: Dictionary, cl: String) -> Array[String]:
	var errors: Array[String] = []
	match kind:
		"flag":
			var key: Variant = cond.get("key", "")
			if not (key is String) or not GameState.flag_key_ok(str(key)):
				errors.append("%s: flag key '%s' must be non-empty [a-z0-9_.]." % [cl, str(key)])
			if cond.has("value") and not QuestRules.flag_value_ok(cond["value"]):
				errors.append("%s: value must be a bool, a whole number or a String." % cl)
			errors.append_array(QuestRules.op_errors(cond, cl))
		"quest":
			errors.append_array(_ref_value_errors(world_id, cond.get("quest", ""), "quests", cl + " quest"))
			var status: Variant = cond.get("status", "")
			if not (status is String) or not QUEST_STATUSES.has(str(status)):
				errors.append("%s: status '%s' is not one of %s." % [cl, str(status), ", ".join(QUEST_STATUSES)])
		"items":
			var item: Variant = cond.get("item", "")
			if not (item is String) or ContentCatalog.get_entry("items", str(item)).is_empty():
				errors.append("%s: unknown base item '%s'." % [cl, str(item)])
			var count: Variant = cond.get("count", 1)
			if not (count is int) or DictUtil.to_int(count) < 1 or DictUtil.to_int(count) > QuestRules.ITEM_COUNT_MAX:
				errors.append("%s: count must be a whole number 1..%d." % [cl, QuestRules.ITEM_COUNT_MAX])
		"gold":
			var gmin: Variant = cond.get("min", null)
			if not (gmin is int) or DictUtil.to_int(gmin) < 0 or DictUtil.to_int(gmin) > GOLD_CONDITION_MAX:
				errors.append("%s: min must be a whole number 0..%d." % [cl, GOLD_CONDITION_MAX])
		"stat":
			var stat: Variant = cond.get("stat", "")
			if not (stat is String) or not ContentSchema.STAT_KEYS.has(str(stat)):
				errors.append("%s: stat '%s' is not one of %s." % [cl, str(stat), ", ".join(ContentSchema.STAT_KEYS)])
			var smin: Variant = cond.get("min", null)
			if not (smin is int) or DictUtil.to_int(smin) < STAT_MIN or DictUtil.to_int(smin) > STAT_MAX:
				errors.append("%s: min must be a whole number %d..%d." % [cl, STAT_MIN, STAT_MAX])
	return errors

# A required world ref held in a json value: non-empty String that resolves.
static func _ref_value_errors(world_id: String, raw: Variant, ref_kind: String, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is String) or str(raw).is_empty():
		errors.append("%s: choose a %s." % [label, ref_kind.trim_suffix("s")])
		return errors
	return WorldCatalog.ref_errors(world_id, label, ref_kind, str(raw))

# ------------------------------------------------------------------------------
# Effects (`node_level`: node effects, set_flag / give only; otherwise choice effects with the order rules)
# ------------------------------------------------------------------------------

static func effects_errors(world_id: String, raw: Variant, label: String, node_level: bool) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("%s: effects must be a list." % label)
		return errors
	var list: Array = DictUtil.array_of(JsonUtil.normalize(raw))
	if list.size() > MAX_EFFECTS:
		errors.append("%s holds at most %d effects (has %d)." % [label, MAX_EFFECTS, list.size()])
	var names: Array[String] = []
	for k: int in range(list.size()):
		if not (list[k] is Dictionary):
			errors.append("%s effect %d must be an object {\"effect\", ...}." % [label, k + 1])
			names.append("")
			continue
		var fx: Dictionary = list[k]
		var effect: String = str(fx.get("effect", ""))
		if not EFFECTS.has(effect):
			errors.append("%s effect %d: unknown effect '%s' (use %s)." % [label, k + 1, effect, ", ".join(PackedStringArray(EFFECTS.keys()))])
			names.append("")
			continue
		names.append(effect)
		var el: String = "%s effect %d (%s)" % [label, k + 1, effect]
		if node_level and not NODE_EFFECTS.has(effect):
			errors.append("%s: Node effects may only be set_flag, add_flag or give." % el)
		var allowed: Dictionary = EFFECTS[effect]
		for key: Variant in fx:
			if str(key) != "effect" and not allowed.has(str(key)):
				errors.append("%s: unknown key '%s'." % [el, str(key)])
		errors.append_array(_effect_errors(world_id, effect, fx, el))
	if not node_level:
		errors.append_array(_order_errors(names, label))
		if names.count("recruit") > 1:
			errors.append("%s: at most one recruit per choice." % label)
	return errors

static func _effect_errors(world_id: String, effect: String, fx: Dictionary, el: String) -> Array[String]:
	var errors: Array[String] = []
	match effect:
		"set_flag":
			var key: Variant = fx.get("key", "")
			if not (key is String) or not GameState.flag_key_ok(str(key)):
				errors.append("%s: flag key '%s' must be non-empty [a-z0-9_.]." % [el, str(key)])
			if fx.has("value") and not QuestRules.flag_value_ok(fx["value"]):
				errors.append("%s: value must be a bool, a whole number or a String." % el)
			var why: String = RepRules.value_error(str(key), fx.get("value", true))
			if not why.is_empty():
				errors.append("%s: %s" % [el, why])
		"add_flag":
			errors.append_array(_add_flag_errors(fx, el))
		"start_quest", "turn_in":
			var ref_errs: Array[String] = _ref_value_errors(world_id, fx.get("quest", ""), "quests", el + " quest")
			errors.append_array(ref_errs)
			if ref_errs.is_empty() and str(WorldCatalog.resolve(world_id, "quests", str(fx["quest"])).get("template", "")).is_empty():
				errors.append("%s: quest '%s' has no template (a lore-only quest never runs)." % [el, str(fx["quest"])])
		"give", "take":
			errors.append_array(_goods_errors(fx, el))
		"open_shop":
			errors.append_array(_ref_value_errors(world_id, fx.get("shop", ""), "shops", el + " shop"))
		"fight":
			errors.append_array(_ref_value_errors(world_id, fx.get("encounter", ""), "encounters", el + " encounter"))
		"recruit":
			errors.append_array(_recruit_errors(world_id, fx.get("character", ""), el))
	return errors

# recruit (RECRUIT1-L 1.4): a world character that resolves and may join the party (WorldCatalog.party_character_errors).
static func _recruit_errors(world_id: String, raw: Variant, el: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is String) or str(raw).is_empty():
		errors.append("%s: choose a character." % el)
		return errors
	var id: String = str(raw)
	if WorldCatalog.resolve(world_id, "characters", id).is_empty():
		errors.append("%s: Recruit: unknown character '%s'." % [el, id])
		return errors
	for why: String in WorldCatalog.party_character_errors(world_id, id):
		errors.append("%s: %s" % [el, why])
	return errors

# add_flag (REP1-L 1.3): a valid key, delta a whole number in -ADD_DELTA_MAX..ADD_DELTA_MAX and not 0, optional
# whole-number min / max with min <= max when both are given.
static func _add_flag_errors(fx: Dictionary, el: String) -> Array[String]:
	var errors: Array[String] = []
	var key: Variant = fx.get("key", "")
	if not (key is String) or not GameState.flag_key_ok(str(key)):
		errors.append("%s: flag key '%s' must be non-empty [a-z0-9_.]." % [el, str(key)])
	var delta: Variant = fx.get("delta", null)
	if not (delta is int) or DictUtil.to_int(delta) == 0 or absi(DictUtil.to_int(delta)) > ADD_DELTA_MAX:
		errors.append("%s: delta must be a whole number from -%d to %d, not 0." % [el, ADD_DELTA_MAX, ADD_DELTA_MAX])
	for bound: String in ["min", "max"]:
		var b: Variant = fx.get(bound, null)
		if b != null and not (b is int):
			errors.append("%s: %s must be a whole number." % [el, bound])
	var lo: Variant = fx.get("min", null)
	var hi: Variant = fx.get("max", null)
	if lo is int and hi is int and DictUtil.to_int(lo) > DictUtil.to_int(hi):
		errors.append("%s: min must not be above max." % el)
	return errors

# give / take: items {base id: 1..99} (at most MAX_ITEM_KEYS keys), gold 0..EFFECT_GOLD_MAX, not both empty.
static func _goods_errors(fx: Dictionary, el: String) -> Array[String]:
	var errors: Array[String] = []
	var items_raw: Variant = fx.get("items", {})
	var items: Dictionary = DictUtil.dict_of(items_raw)
	if not (items_raw is Dictionary):
		errors.append("%s: items must be an object {base item id: count}." % el)
	elif items.size() > QuestRules.MAX_ITEM_KEYS:
		errors.append("%s lists at most %d items." % [el, QuestRules.MAX_ITEM_KEYS])
	for id: Variant in items:
		if ContentCatalog.get_entry("items", str(id)).is_empty():
			errors.append("%s: unknown base item '%s'." % [el, str(id)])
		var count: Variant = items[id]
		if not (count is int) or DictUtil.to_int(count) < 1 or DictUtil.to_int(count) > QuestRules.ITEM_COUNT_MAX:
			errors.append("%s: count of '%s' must be a whole number 1..%d." % [el, str(id), QuestRules.ITEM_COUNT_MAX])
	var gold: Variant = fx.get("gold", 0)
	if not (gold is int) or DictUtil.to_int(gold) < 0 or DictUtil.to_int(gold) > EFFECT_GOLD_MAX:
		errors.append("%s: gold must be a whole number 0..%d." % [el, EFFECT_GOLD_MAX])
	if items.is_empty() and DictUtil.to_int(gold) <= 0:
		errors.append("%s needs items or gold." % el)
	return errors

# Choice order rules: turn_in first and followed only by set_flag / give; open_shop and fight last; fight never
# shares a choice with turn_in or open_shop. Unknown effects are "" (already reported).
# One message per problem: when fight shares the choice with turn_in or open_shop only that is reported (the order
# rules about those two effects would restate it).
static func _order_errors(names: Array[String], label: String) -> Array[String]:
	var errors: Array[String] = []
	var share: bool = names.has("fight") and (names.has("turn_in") or names.has("open_shop"))
	var movers: Array[String] = ["open_shop", "fight"]
	var turn_at: int = names.find("turn_in")
	if turn_at >= 0:
		var bad: bool = turn_at != 0
		for k: int in range(turn_at + 1, names.size()):
			if not names[k].is_empty() and not NODE_EFFECTS.has(names[k]) and not (share and movers.has(names[k])):
				bad = true
		if bad:
			errors.append("%s: turn_in must come first; only set_flag, add_flag or give may follow." % label)
	if share:
		errors.append("%s: fight can't share a choice with turn_in or open_shop." % label)
		return errors
	for last_only: String in movers:
		for k: int in range(names.size()):
			if names[k] == last_only and k != names.size() - 1:
				errors.append("%s: %s must be the last effect." % [label, last_only])
	return errors

# ------------------------------------------------------------------------------
# Service lines (ServiceCatalog entries and manifest `service_lines`)
# ------------------------------------------------------------------------------

# The `{token}` names used in `text`, in order.
static func tokens_of(text: String) -> Array[String]:
	var out: Array[String] = []
	var re := RegEx.new()
	re.compile("\\{([^{}]*)\\}")
	for m: RegExMatch in re.search_all(text):
		out.append(m.get_string(1))
	return out

# Problems of one service line: a known id, non-empty text, known tokens only.
static func line_errors(id: String, text: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not SERVICE_IDS.has(id):
		errors.append("%s: unknown service line '%s' (use %s)." % [label, id, ", ".join(SERVICE_IDS)])
	if not (text is String) or str(text).strip_edges().is_empty():
		errors.append("%s: service line '%s' needs text." % [label, id])
		return errors
	if str(text).length() > MAX_TEXT:
		errors.append("%s: service line '%s' is longer than %d characters." % [label, id, MAX_TEXT])
	for token: String in tokens_of(str(text)):
		if not TOKENS.has(token):
			errors.append("%s: Unknown token '%s'." % [label, token])
	return errors

# Manifest `service_lines`: {line id: text}.
static func service_lines_errors(raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("%s must be an object {line id: text}." % label)
		return errors
	var lines: Dictionary = raw
	for id: Variant in lines:
		errors.append_array(line_errors(str(id), lines[id], label))
	return errors

# ------------------------------------------------------------------------------
# Gone-NPC lint (warnings only)
# ------------------------------------------------------------------------------

# For each merged quest with a template: every `gone_flags` condition of its base giver G (or its effective turn-in T)
# needs a variant whose `when_flags` hold that same condition and whose `set` re-routes the field (`giver` for G, which
# also covers T when `turn_in` is ""; `turn_in` for T) or sets `available: false`.
static func quest_warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for q: Dictionary in WorldCatalog.merged_entries(world_id, "quests"):
		if str(q.get("template", "")).is_empty():
			continue
		var variants: Array = DictUtil.array_of(JsonUtil.normalize(q.get("variants", [])))
		var giver: String = str(q.get("giver", ""))
		var turn_in: String = str(q.get("turn_in", ""))
		var roles: Array[Array] = [["giver", giver, "giver"]]
		if not turn_in.is_empty():
			roles.append(["turn-in", turn_in, "turn_in"])
		for role: Array in roles:
			var ref: String = str(role[1])
			var who: Dictionary = WorldCatalog.resolve(world_id, "characters", ref)
			if ref.is_empty() or who.is_empty():
				continue
			for c: Variant in DictUtil.array_of(JsonUtil.normalize(who.get("gone_flags", []))):
				var cond: Dictionary = DictUtil.dict_of(c)
				if not _rerouted(variants, cond, str(role[2])):
					out.append("Quest '%s': %s '%s' can be gone (%s) and no variant re-routes it." % [str(q["_qid"]), str(role[0]), ref, _condition_text(cond)])
	return out

static func _rerouted(variants: Array, cond: Dictionary, field: String) -> bool:
	for raw: Variant in variants:
		var v: Dictionary = DictUtil.dict_of(raw)
		var set_d: Dictionary = DictUtil.dict_of(v.get("set", {}))
		if not (set_d.has(field) or (set_d.get("available", true) is bool and not DictUtil.to_bool(set_d.get("available", true)))):
			continue
		for w: Variant in DictUtil.array_of(v.get("when_flags", [])):
			if _same_condition(DictUtil.dict_of(w), cond):
				return true
	return false

# Same flag key, same op (default "=="; a set_flag effect has none) and same value (value defaults to true; strict
# type equality as QuestRules.condition_met).
static func _same_condition(a: Dictionary, b: Dictionary) -> bool:
	var va: Variant = JsonUtil.normalize(a.get("value", true))
	var vb: Variant = JsonUtil.normalize(b.get("value", true))
	return str(a.get("key", "")) == str(b.get("key", "")) and str(a.get("op", "==")) == str(b.get("op", "==")) and typeof(va) == typeof(vb) and va == vb

static func _condition_text(cond: Dictionary) -> String:
	var op: String = str(cond.get("op", "=="))
	return "%s %s %s" % [str(cond.get("key", "")), "=" if op == "==" else op, str(JsonUtil.normalize(cond.get("value", true)))]

# ------------------------------------------------------------------------------
# Runtime reads (pure over GameState + the catalogs; DialogueRunner, ExploreRunner presence, ShopRules use them)
# ------------------------------------------------------------------------------

# A character is gone when ANY of its gone_flags holds; re-read on every call (no saved state). Refs resolve from
# gs.world_id; an unknown character is never gone.
static func is_gone(gs: GameState, character_ref: String) -> bool:
	if character_ref.is_empty():
		return false
	var c: Dictionary = WorldCatalog.resolve(gs.world_id, "characters", character_ref)
	for cond: Variant in DictUtil.array_of(JsonUtil.normalize(c.get("gone_flags", []))):
		if QuestRules.condition_met(gs, DictUtil.dict_of(cond)):
			return true
	return false

# The resolved character's qualified id ("" when the ref is empty or unknown).
static func character_qid(gs: GameState, ref: String) -> String:
	return str(WorldCatalog.resolve(gs.world_id, "characters", ref).get("_qid", "")) if not ref.is_empty() else ""

# The resolved faction qid of a character ("" when the ref is empty or unknown, or it has no faction that resolves).
static func faction_qid(gs: GameState, character_ref: String) -> String:
	if character_ref.is_empty():
		return ""
	var c: Dictionary = WorldCatalog.resolve(gs.world_id, "characters", character_ref)
	var fref: String = str(c.get("faction", ""))
	if c.is_empty() or fref.is_empty():
		return ""
	return str(WorldCatalog.resolve(str(c.get("_world", gs.world_id)), "factions", fref).get("_qid", ""))

# The resolved quest's giver / effective turn-in (QuestRules.resolve already falls back to the giver) as qids.
static func giver_qid(gs: GameState, resolved_quest: Dictionary) -> String:
	return character_qid(gs, str(resolved_quest.get("giver", "")))

static func turn_in_qid(gs: GameState, resolved_quest: Dictionary) -> String:
	return character_qid(gs, str(resolved_quest.get("turn_in", "")))

# The party gold of a new game in `world_id`: the nearest manifest (the world, then up the lineage) with start_gold
# >= 0, else 0.
static func start_gold(world_id: String) -> int:
	var chain: Array[String] = WorldCatalog.lineage(world_id)
	chain.reverse()
	for w: String in chain:
		var v: int = DictUtil.to_int(WorldCatalog.manifest(w).get("start_gold", -1), -1)
		if v >= 0:
			return v
	return 0

# The text of service line `id` for `world_id`: the base line (ServiceCatalog, else DEFAULT_LINES) with the manifest
# `service_lines` merged along the lineage key by key (nearest wins).
static func service_line(world_id: String, id: String) -> String:
	var text: String = ServiceCatalog.text_of(id)
	for w: String in WorldCatalog.lineage(world_id):
		var lines: Dictionary = DictUtil.dict_of(WorldCatalog.manifest(w).get("service_lines", {}))
		if lines.get(id, null) is String and not str(lines[id]).strip_edges().is_empty():
			text = str(lines[id])
	return text

# `{token}` replaced by `values[token]`; unknown tokens stay as written.
static func fill(text: String, values: Dictionary) -> String:
	var out: String = text
	for token: Variant in values:
		out = out.replace("{%s}" % str(token), str(values[token]))
	return out

# The highest effective value of `stat` (PartyBuilder.build: gear and passives included) among the ACTIVE members.
static func best_stat(gs: GameState, stat: String) -> int:
	var best: int = 0
	for m: UnitState in gs.party.active_members():
		var c: Combatant = PartyBuilder.build(m)
		if c != null and c.stats != null:
			best = maxi(best, c.stats.get_effective_stat(stat))
	return best

# One dialogue condition (validated shape; kind defaults to flag).
static func condition_holds(gs: GameState, raw: Dictionary) -> bool:
	var cond: Dictionary = DictUtil.dict_of(JsonUtil.normalize(raw))
	match str(cond.get("kind", "flag")):
		"flag":
			return QuestRules.condition_met(gs, cond)
		"quest":
			return QuestRunner.status(gs, str(cond.get("quest", ""))) == str(cond.get("status", ""))
		"items":
			return DictUtil.to_int(gs.party.stash.get(str(cond.get("item", "")), 0)) >= DictUtil.to_int(cond.get("count", 1), 1)
		"gold":
			return gs.party.gold >= DictUtil.to_int(cond.get("min", 0))
		"stat":
			return best_stat(gs, str(cond.get("stat", ""))) >= DictUtil.to_int(cond.get("min", 0))
	return false

# True when every `requires` condition holds and no `unless` condition does.
static func choice_open(gs: GameState, choice: Dictionary) -> bool:
	for c: Variant in DictUtil.array_of(choice.get("requires", [])):
		if not condition_holds(gs, DictUtil.dict_of(c)):
			return false
	for c: Variant in DictUtil.array_of(choice.get("unless", [])):
		if condition_holds(gs, DictUtil.dict_of(c)):
			return false
	return true

# The generated "[STAT min] " prefix of a choice: one tag per stat condition in `requires`, in order.
static func stat_prefix(choice: Dictionary) -> String:
	var out: String = ""
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(choice.get("requires", []))):
		var cond: Dictionary = DictUtil.dict_of(c)
		if str(cond.get("kind", "flag")) == "stat":
			out += "[%s %d] " % [str(cond.get("stat", "")), DictUtil.to_int(cond.get("min", 0))]
	return out

# The generated "[<Faction name> <band>] " prefix of a choice (REP1-L 2.4): one tag per `requires` flag condition on a
# `rep.` key with an ordering op, the band being the band of the bound. A key naming no faction shows the key.
static func rep_prefix(gs: GameState, choice: Dictionary) -> String:
	var out: String = ""
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(choice.get("requires", []))):
		var cond: Dictionary = DictUtil.dict_of(c)
		var k: String = str(cond.get("key", ""))
		if str(cond.get("kind", "flag")) != "flag" or not RepRules.is_rep_key(k) or not QuestRules.ORDER_OPS.has(str(cond.get("op", "=="))):
			continue
		var faction: Dictionary = WorldCatalog.resolve(gs.world_id, "factions", RepRules.faction_qid_of(k))
		var label: String = str(faction.get("name", k)) if not faction.is_empty() else k
		out += "[%s %s] " % [label, RepRules.band(DictUtil.to_int(cond.get("value", 0)))]
	return out

# Completes a ready quest from the journal (no NPC): only when its giver and effective turn-in both resolve to no
# character; otherwise the turn-in goes through that NPC's dialogue. Same result shape as QuestRunner.complete.
static func complete_from_journal(gs: GameState, ref: String, recipient_id: String = "", picks: Array[String] = []) -> Dictionary:
	var q: Dictionary = QuestRunner.resolved(gs, ref)
	var who: String = turn_in_qid(gs, q) if not q.is_empty() else ""
	if who.is_empty() and not q.is_empty():
		who = giver_qid(gs, q)
	if not who.is_empty():
		var errors: Array[String] = ["This quest is turned in to %s." % str(WorldCatalog.resolve(gs.world_id, "characters", who).get("name", who))]
		return {"ok": false, "errors": errors, "granted": {}, "notes": [] as Array[String]}
	return QuestRunner.complete(gs, ref, recipient_id, picks)
