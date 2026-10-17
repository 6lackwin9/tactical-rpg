class_name DialogueRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Dialogue Rules (S11, spec DLG1-L)
# Constants and pure validation for world dialogue trees (WorldCatalog kind `dialogues`), dialogue choice
# conditions and effects, base service lines (ServiceCatalog, manifest `service_lines`) and the gone-NPC quest
# lint. No state, no RNG. The runtime (DialogueRunner, presence, services) builds on these in DLG1-L commit 2.
# A tree is {id, name, speaker, start, nodes, services}; a node has exactly one of non-empty `choices`, a `next`
# node id or `end: true`. Node effects are set_flag / give only; choice effects are all-or-nothing at runtime.
# Markers, quests, loot, routes and `gone_flags` keep the M4 flag-only condition grammar (QuestRules).
# ==============================================================================

const MAX_NODES: int = 64
const MAX_CHOICES: int = 6
const MAX_EFFECTS: int = 8
const MAX_TEXT: int = 1000

const NODE_KEYS: Array[String] = ["id", "speaker", "text", "choices", "effects", "next", "end"]
const CHOICE_KEYS: Array[String] = ["text", "to", "requires", "unless", "effects", "show_locked"]

# Condition kind -> its keys besides `kind`, with defaults (a missing required key reads as its default and fails).
const CONDITION_KINDS: Dictionary = {
	"flag": {"key": "", "value": true},
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
	"start_quest": {"quest": ""},
	"turn_in": {"quest": ""},
	"give": {"items": {}, "gold": 0},
	"take": {"items": {}, "gold": 0},
	"open_shop": {"shop": ""},
	"fight": {"encounter": ""},
}
const NODE_EFFECTS: Array[String] = ["set_flag", "give"]
const EFFECT_GOLD_MAX: int = 99999

# Json ref keys (WorldCatalog reference scans and renames): condition / effect key -> the world kind it names.
const CONDITION_REF_KEYS: Dictionary = {"quest": "quests"}
const EFFECT_REF_KEYS: Dictionary = {"quest": "quests", "shop": "shops", "encounter": "encounters"}

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

# Soft problems of one tree: nodes no `to` / `next` reaches from the start node.
static func tree_warnings(e: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var nodes: Array = DictUtil.array_of(JsonUtil.normalize(e.get("nodes", [])))
	var by_id: Dictionary = _nodes_by_id(nodes)
	var start: String = str(e.get("start", ""))
	if not by_id.has(start):
		return out   # an error already
	var reached: Dictionary = {start: true}
	var queue: Array[String] = [start]
	while not queue.is_empty():
		var node: Dictionary = DictUtil.dict_of(by_id.get(queue.pop_front(), {}))
		var targets: Array[String] = []
		if node.get("next", "") is String:
			targets.append(str(node.get("next", "")))
		for c: Variant in DictUtil.array_of(node.get("choices", [])):
			var to: Variant = DictUtil.dict_of(c).get("to", "")
			if to is String:
				targets.append(str(to))
		for t: String in targets:
			if by_id.has(t) and not reached.has(t):
				reached[t] = true
				queue.append(t)
	for n: Variant in nodes:
		var nid: String = str(DictUtil.dict_of(n).get("id", ""))
		if by_id.has(nid) and not reached.has(nid):
			out.append("Node '%s' is not reachable from the start node." % nid)
	return out

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

static func _text_errors(raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is String) or str(raw).strip_edges().is_empty():
		errors.append("%s needs text." % label)
	elif str(raw).length() > MAX_TEXT:
		errors.append("%s: text is longer than %d characters." % [label, MAX_TEXT])
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
			errors.append("%s: Node effects may only be set_flag or give." % el)
		var allowed: Dictionary = EFFECTS[effect]
		for key: Variant in fx:
			if str(key) != "effect" and not allowed.has(str(key)):
				errors.append("%s: unknown key '%s'." % [el, str(key)])
		errors.append_array(_effect_errors(world_id, effect, fx, el))
	if not node_level:
		errors.append_array(_order_errors(names, label))
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
static func _order_errors(names: Array[String], label: String) -> Array[String]:
	var errors: Array[String] = []
	var turn_at: int = names.find("turn_in")
	if turn_at >= 0:
		var bad: bool = turn_at != 0
		for k: int in range(turn_at + 1, names.size()):
			if not names[k].is_empty() and not NODE_EFFECTS.has(names[k]):
				bad = true
		if bad:
			errors.append("%s: turn_in must come first; only set_flag or give may follow." % label)
	for last_only: String in ["open_shop", "fight"]:
		for k: int in range(names.size()):
			if names[k] == last_only and k != names.size() - 1:
				errors.append("%s: %s must be the last effect." % [label, last_only])
	if names.has("fight") and (names.has("turn_in") or names.has("open_shop")):
		errors.append("%s: fight can't share a choice with turn_in or open_shop." % label)
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

# Same flag key and same value (value defaults to true; strict type equality as QuestRules.condition_met).
static func _same_condition(a: Dictionary, b: Dictionary) -> bool:
	var va: Variant = JsonUtil.normalize(a.get("value", true))
	var vb: Variant = JsonUtil.normalize(b.get("value", true))
	return str(a.get("key", "")) == str(b.get("key", "")) and typeof(va) == typeof(vb) and va == vb

static func _condition_text(cond: Dictionary) -> String:
	return "%s = %s" % [str(cond.get("key", "")), str(JsonUtil.normalize(cond.get("value", true)))]
