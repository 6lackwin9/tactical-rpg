---
class: DialogueRules
extends: RefCounted
description: "Tactical RPG - Dialogue Rules (S11, spec DLG1-L) Constants and pure validation for world dialogue trees (WorldCatalog kind `dialogues`), dialogue choice conditions and effects, base service lines (ServiceCatalog, manifest `service_lines`) and the gone-NPC quest lint, plus the pure runtime reads (presence `is_gone`, conditions, best stat, start gold, service lines, the journal turn-in rule) that DialogueRunner, ExploreRunner and ShopRules use. No state, no RNG. A tree is {id, name, speaker, start, nodes, services}; a node has exactly one of non-empty `choices`, a `next` node id or `end: true`. Node effects are set_flag / add_flag / give only; choice effects are all-or-nothing at runtime. Markers, quests, loot, routes and `gone_flags` keep the M4 flag-only condition grammar (QuestRules), with the REP1 operators (`op`); dialogue flag conditions delegate to it. `add_flag` (REP1-L) adds a delta to an int flag."
---

# Properties
* `MAX_NODES`: int = 64 (const)
* `MAX_CHOICES`: int = 6 (const)
* `MAX_EFFECTS`: int = 8 (const)
* `MAX_TEXT`: int = 1000 (const)
* `NODE_KEYS`: Array[String] = ["id", "speaker", "text", "choices", "effects", "next", "end"] (const)
* `CHOICE_KEYS`: Array[String] = ["text", "to", "requires", "unless", "effects", "show_locked"] (const)
* `CONDITION_KINDS`: Dictionary = { (const)
* `QUEST_STATUSES`: Array[String] = ["locked", "available", "unavailable", "active", "ready", "blocked", "completed"] (const)
* `GOLD_CONDITION_MAX`: int = 9999999 (const)
* `STAT_MIN`: int = 1 (const)
* `STAT_MAX`: int = 30 (const)
* `EFFECTS`: Dictionary = { (const)
* `NODE_EFFECTS`: Array[String] = ["set_flag", "add_flag", "give"] (const)
* `EFFECT_GOLD_MAX`: int = 99999 (const)
* `ADD_DELTA_MAX`: int = 100   # |delta| of add_flag (REP1-L 1.3) (const)
* `CONDITION_REF_KEYS`: Dictionary = {"quest": "quests"} (const)
* `EFFECT_REF_KEYS`: Dictionary = {"quest": "quests", "shop": "shops", "encounter": "encounters", "character": "characters"} (const)
* `SERVICE_IDS`: Array[String] = ["greeting", "offer", "accept", "decline", "turn_in", "shop", "farewell"] (const)
* `DEFAULT_LINES`: Dictionary = { (const)
* `TOKENS`: Array[String] = ["npc", "quest", "shop", "gold"] (const)
* `TEXT_TOKENS`: Array[String] = ["npc", "gold"]   # authored node and choice texts (DLG1 build choices) (const)
* `errors`: Array[String] = []
* `raw_nodes`: Variant = e.get("nodes", [])
* `nodes`: Array = DictUtil.array_of(JsonUtil.normalize(raw_nodes))
* `by_id`: Dictionary = _nodes_by_id(nodes)
* `seen`: Dictionary = {}
* `nid`: String = str(DictUtil.dict_of(nodes[i]).get("id", ""))
* `start`: String = str(e.get("start", ""))
* `out`: Array[String] = []
* `nodes`: Array = DictUtil.array_of(JsonUtil.normalize(e.get("nodes", [])))
* `by_id`: Dictionary = _nodes_by_id(nodes)
* `start`: String = str(e.get("start", ""))
* `edges`: Dictionary = _edges(by_id, start, DictUtil.to_bool(e.get("services", true)))
* `reached`: Dictionary = _reach(edges, start)
* `nid`: String = str(DictUtil.dict_of(n).get("id", ""))
* `node`: Dictionary = by_id[nid]
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `node`: Dictionary = raw
* `nid`: String = str(node.get("id", ""))
* `label`: String = "Node '%s'" % nid
* `speaker`: Variant = node.get("speaker", "")
* `choices_raw`: Variant = node.get("choices", [])
* `next_raw`: Variant = node.get("next", "")
* `end_raw`: Variant = node.get("end", false)
* `choices`: Array = DictUtil.array_of(choices_raw)
* `has_next`: bool = next_raw is String and not str(next_raw).is_empty()
* `has_end`: bool = end_raw is bool and DictUtil.to_bool(end_raw)
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `c`: Dictionary = raw
* `to`: Variant = c.get("to", "")
* `errors`: Array[String] = []
* `list`: Array = DictUtil.array_of(JsonUtil.normalize(raw))
* `cond`: Dictionary = list[i]
* `kind`: String = str(cond.get("kind", "flag"))
* `cl`: String = "%s %d (%s)" % [label, i + 1, kind]
* `allowed`: Dictionary = CONDITION_KINDS[kind]
* `errors`: Array[String] = []
* `key`: Variant = cond.get("key", "")
* `status`: Variant = cond.get("status", "")
* `item`: Variant = cond.get("item", "")
* `count`: Variant = cond.get("count", 1)
* `gmin`: Variant = cond.get("min", null)
* `stat`: Variant = cond.get("stat", "")
* `smin`: Variant = cond.get("min", null)
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `list`: Array = DictUtil.array_of(JsonUtil.normalize(raw))
* `names`: Array[String] = []
* `fx`: Dictionary = list[k]
* `effect`: String = str(fx.get("effect", ""))
* `el`: String = "%s effect %d (%s)" % [label, k + 1, effect]
* `allowed`: Dictionary = EFFECTS[effect]
* `errors`: Array[String] = []
* `key`: Variant = fx.get("key", "")
* `why`: String = RepRules.value_error(str(key), fx.get("value", true))
* `ref_errs`: Array[String] = _ref_value_errors(world_id, fx.get("quest", ""), "quests", el + " quest")
* `errors`: Array[String] = []
* `id`: String = str(raw)
* `errors`: Array[String] = []
* `key`: Variant = fx.get("key", "")
* `delta`: Variant = fx.get("delta", null)
* `b`: Variant = fx.get(bound, null)
* `lo`: Variant = fx.get("min", null)
* `hi`: Variant = fx.get("max", null)
* `errors`: Array[String] = []
* `items_raw`: Variant = fx.get("items", {})
* `items`: Dictionary = DictUtil.dict_of(items_raw)
* `count`: Variant = items[id]
* `gold`: Variant = fx.get("gold", 0)
* `errors`: Array[String] = []
* `share`: bool = names.has("fight") and (names.has("turn_in") or names.has("open_shop"))
* `movers`: Array[String] = ["open_shop", "fight"]
* `turn_at`: int = names.find("turn_in")
* `bad`: bool = turn_at != 0
* `out`: Array[String] = []
* `re`: Variant
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `lines`: Dictionary = raw
* `out`: Array[String] = []
* `variants`: Array = DictUtil.array_of(JsonUtil.normalize(q.get("variants", [])))
* `giver`: String = str(q.get("giver", ""))
* `turn_in`: String = str(q.get("turn_in", ""))
* `roles`: Array[Array] = [["giver", giver, "giver"]]
* `ref`: String = str(role[1])
* `who`: Dictionary = WorldCatalog.resolve(world_id, "characters", ref)
* `cond`: Dictionary = DictUtil.dict_of(c)

# Methods

### `static _edges(by_id: Dictionary, start: String, services: bool) -> Dictionary`
```logic
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
```

### `static _reach(edges: Dictionary, from: String) -> Dictionary`
```logic
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
```

### `static _gives(node: Dictionary) -> bool`
```logic
return _has_effect(node.get("effects", []), "give")
```

### `static _has_effect(raw: Variant, effect: String) -> bool`
```logic
for fx: Variant in DictUtil.array_of(raw):
	if str(DictUtil.dict_of(fx).get("effect", "")) == effect:
		return true
return false
```

### `static _guarded(edges: Dictionary, by_id: Dictionary, nid: String) -> bool`
```logic
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
```

### `static _nodes_by_id(nodes: Array) -> Dictionary`
```logic
var out: Dictionary = {}
for n: Variant in nodes:
	if not (n is Dictionary):
		continue
	var node: Dictionary = n
	var nid: Variant = node.get("id", "")
	if nid is String and JsonUtil.id_ok(str(nid)) and not out.has(nid):
		out[str(nid)] = node
return out
```

### `static _rerouted(variants: Array, cond: Dictionary, field: String) -> bool`
```logic
for raw: Variant in variants:
	var v: Dictionary = DictUtil.dict_of(raw)
	var set_d: Dictionary = DictUtil.dict_of(v.get("set", {}))
	if not (set_d.has(field) or (set_d.get("available", true) is bool and not DictUtil.to_bool(set_d.get("available", true)))):
		continue
	for w: Variant in DictUtil.array_of(v.get("when_flags", [])):
		if _same_condition(DictUtil.dict_of(w), cond):
			return true
return false
```

### `static _same_condition(a: Dictionary, b: Dictionary) -> bool`
```logic
var va: Variant = JsonUtil.normalize(a.get("value", true))
var vb: Variant = JsonUtil.normalize(b.get("value", true))
return str(a.get("key", "")) == str(b.get("key", "")) and str(a.get("op", "==")) == str(b.get("op", "==")) and typeof(va) == typeof(vb) and va == vb
```

### `static _condition_text(cond: Dictionary) -> String`
```logic
var op: String = str(cond.get("op", "=="))
return "%s %s %s" % [str(cond.get("key", "")), "=" if op == "==" else op, str(JsonUtil.normalize(cond.get("value", true)))]
```

### `static is_gone(gs: GameState, character_ref: String) -> bool`
```logic
if character_ref.is_empty():
	return false
var c: Dictionary = WorldCatalog.resolve(gs.world_id, "characters", character_ref)
for cond: Variant in DictUtil.array_of(JsonUtil.normalize(c.get("gone_flags", []))):
	if QuestRules.condition_met(gs, DictUtil.dict_of(cond)):
		return true
return false
```

### `static is_member(gs: GameState, character_ref: String) -> bool`
```logic
var c: Dictionary = WorldCatalog.resolve(gs.world_id, "characters", character_ref) if not character_ref.is_empty() else {}
if c.is_empty():
	return false
for m: UnitState in gs.party.members:
	if m.id == str(c.get("id", "")) or m.lore_ref == str(c.get("_qid", "")):
		return true
return false
```

### `static recruits_member(gs: GameState, choice: Dictionary) -> bool`
```logic
for raw: Variant in DictUtil.array_of(JsonUtil.normalize(choice.get("effects", []))):
	var fx: Dictionary = DictUtil.dict_of(raw)
	if str(fx.get("effect", "")) == "recruit" and is_member(gs, str(fx.get("character", ""))):
		return true
return false
```

### `static character_qid(gs: GameState, ref: String) -> String`
```logic
return str(WorldCatalog.resolve(gs.world_id, "characters", ref).get("_qid", "")) if not ref.is_empty() else ""
```

### `static faction_qid(gs: GameState, character_ref: String) -> String`
```logic
if character_ref.is_empty():
	return ""
var c: Dictionary = WorldCatalog.resolve(gs.world_id, "characters", character_ref)
var fref: String = str(c.get("faction", ""))
if c.is_empty() or fref.is_empty():
	return ""
return str(WorldCatalog.resolve(str(c.get("_world", gs.world_id)), "factions", fref).get("_qid", ""))
```

### `static giver_qid(gs: GameState, resolved_quest: Dictionary) -> String`
```logic
return character_qid(gs, str(resolved_quest.get("giver", "")))
```

### `static turn_in_qid(gs: GameState, resolved_quest: Dictionary) -> String`
```logic
return character_qid(gs, str(resolved_quest.get("turn_in", "")))
```

### `static start_gold(world_id: String) -> int`
```logic
var chain: Array[String] = WorldCatalog.lineage(world_id)
chain.reverse()
for w: String in chain:
	var v: int = DictUtil.to_int(WorldCatalog.manifest(w).get("start_gold", -1), -1)
	if v >= 0:
		return v
return 0
```

### `static service_line(world_id: String, id: String) -> String`
```logic
var text: String = ServiceCatalog.text_of(id)
for w: String in WorldCatalog.lineage(world_id):
	var lines: Dictionary = DictUtil.dict_of(WorldCatalog.manifest(w).get("service_lines", {}))
	if lines.get(id, null) is String and not str(lines[id]).strip_edges().is_empty():
		text = str(lines[id])
return text
```

### `static fill(text: String, values: Dictionary) -> String`
```logic
var out: String = text
for token: Variant in values:
	out = out.replace("{%s}" % str(token), str(values[token]))
return out
```

### `static best_stat(gs: GameState, stat: String) -> int`
```logic
var best: int = 0
for m: UnitState in gs.party.active_members():
	var c: Combatant = PartyBuilder.build(m)
	if c != null and c.stats != null:
		best = maxi(best, c.stats.get_effective_stat(stat))
return best
```

### `static condition_holds(gs: GameState, raw: Dictionary) -> bool`
```logic
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
```

### `static choice_open(gs: GameState, choice: Dictionary) -> bool`
```logic
for c: Variant in DictUtil.array_of(choice.get("requires", [])):
	if not condition_holds(gs, DictUtil.dict_of(c)):
		return false
for c: Variant in DictUtil.array_of(choice.get("unless", [])):
	if condition_holds(gs, DictUtil.dict_of(c)):
		return false
return true
```

### `static stat_prefix(choice: Dictionary) -> String`
```logic
var out: String = ""
for c: Variant in DictUtil.array_of(JsonUtil.normalize(choice.get("requires", []))):
	var cond: Dictionary = DictUtil.dict_of(c)
	if str(cond.get("kind", "flag")) == "stat":
		out += "[%s %d] " % [str(cond.get("stat", "")), DictUtil.to_int(cond.get("min", 0))]
return out
```

### `static rep_prefix(gs: GameState, choice: Dictionary) -> String`
```logic
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
```

### `static complete_from_journal(gs: GameState, ref: String, recipient_id: String = "", picks: Array[String] = []) -> Dictionary`
```logic
var q: Dictionary = QuestRunner.resolved(gs, ref)
var who: String = turn_in_qid(gs, q) if not q.is_empty() else ""
if who.is_empty() and not q.is_empty():
	who = giver_qid(gs, q)
if not who.is_empty():
	var errors: Array[String] = ["This quest is turned in to %s." % str(WorldCatalog.resolve(gs.world_id, "characters", who).get("name", who))]
	return {"ok": false, "errors": errors, "granted": {}, "notes": [] as Array[String]}
return QuestRunner.complete(gs, ref, recipient_id, picks)
```

