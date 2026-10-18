---
class: DialogueRunner
extends: RefCounted
description: "Tactical RPG - Dialogue Runner (S11, spec DLG1-L 2.2-2.4): one conversation with one world character. One instance per conversation, never saved. open() picks the character's tree (`dialogue`; \"\" = a generated root with the `greeting` line and the service choices only). Showing a node runs its effects (every time it is shown), except the return to the node a shop was opened from (world-campaign.md \"DLG1 shop return (user 2026-10-17)\"). Visible choices: authored ones whose requires hold and unless don't; failing ones with show_locked appear locked; the rest are hidden; then, on the start node with services on, the generated service choices in order: quest offers (resolved giver = this character, status available), turn-ins (effective turn-in = this character, status ready), open shops kept by this character, farewell. Service choices may push the start node past MAX_CHOICES (allowed: the cap is an authoring limit). choose(index) pre-checks every effect first (all-or-nothing); a turn_in waits for confirm_turn_in / cancel; open_shop waits for close_shop; fight closes the conversation and hands the fight back. Nodes with `next` (and end nodes) move on through advance(). RECRUIT1-L: recruit adds a world party character to the roster (refused when already a member or the roster is full); a choice recruiting a member is hidden. REP1-L: add_flag adds to an int flag (a non-int flag refuses the choice; on a node it is skipped); view() carries the character's faction and its reputation band, and choices gated on a rep bound get the \"[Faction band] \" prefix."
---

# Properties
* `ROOT`: String = "$root"          # the generated root node of a character without a tree (const)
* `OFFER_PREFIX`: String = "$offer:"   # generated offer nodes: "$offer:<quest qid>" (never a valid authored id) (const)
* `gs`: GameState
* `character_qid`: String = ""
* `character_name`: String = ""
* `errors`: Array[String] = []        # non-empty = not opened
* `closed`: bool = false
* `start`: String = ""
* `services`: bool = true
* `node_id`: String = ""
* `shop_qid`: String = ""             # the open shop ("" = none)
* `_tree_speaker`: String = ""        # qid; "" = the character talked to
* `_nodes`: Dictionary = {}           # authored node id -> node (normalised)
* `_pending`: Dictionary = {}         # turn-in waiting for confirm: {quest, rest, to}
* `_shop_to`: String = ""
* `notes`: Array[String] = []         # notes of the last action (e.g. a gold-cap note from `give`); every result carries them
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `sqid`: String = str(shop["_qid"])
* `out`: Array[Dictionary] = []
* `is_open`: bool = DialogueRules.choice_open(gs, c)
* `v`: Dictionary = c.duplicate(true)
* `out`: Array[String] = []
* `take_items`: Dictionary = {}
* `take_gold`: int = 0
* `fx`: Dictionary = DictUtil.dict_of(raw)
* `fkey`: String = str(fx.get("key", ""))
* `ref`: String = str(fx.get("quest", ""))
* `st`: String = QuestRunner.status(gs, ref)
* `giver`: String = DialogueRules.giver_qid(gs, QuestRunner.resolved(gs, ref))
* `ref`: String = str(fx.get("quest", ""))
* `st`: String = QuestRunner.status(gs, ref)
* `taker`: String = DialogueRules.turn_in_qid(gs, QuestRunner.resolved(gs, ref))
* `items`: Dictionary = JsonUtil.ints(fx.get("items", {}))
* `who`: String = str(fx.get("character", ""))
* `cap`: int = WorldCatalog.party_max(gs.world_id)

# Methods

### `static open(p_gs: GameState, character_ref: String) -> DialogueRunner`
```logic
var r := DialogueRunner.new()
r.gs = p_gs
var c: Dictionary = WorldCatalog.resolve(p_gs.world_id, "characters", character_ref)
if c.is_empty():
	return r._refuse("Unknown character '%s'." % character_ref)
r.character_qid = str(c["_qid"])
r.character_name = str(c.get("name", c.get("id", "")))
if DialogueRules.is_gone(p_gs, r.character_qid):
	return r._refuse("%s is gone." % r.character_name)
var tree_ref: String = str(c.get("dialogue", ""))
if tree_ref.is_empty():
	r.start = ROOT
	r._nodes[ROOT] = {"id": ROOT, "text": DialogueRules.service_line(p_gs.world_id, "greeting"), "choices": []}
else:
	var tree: Dictionary = WorldCatalog.resolve(p_gs.world_id, "dialogues", tree_ref)
	if tree.is_empty():
		return r._refuse("Unknown dialogue '%s'." % tree_ref)
	for n: Variant in DictUtil.array_of(JsonUtil.normalize(tree.get("nodes", []))):
		var node: Dictionary = DictUtil.dict_of(n)
		r._nodes[str(node.get("id", ""))] = node
	r.start = str(tree.get("start", ""))
	r.services = DictUtil.to_bool(tree.get("services", true))
	r._tree_speaker = DialogueRules.character_qid(p_gs, str(tree.get("speaker", "")))
	if not r._nodes.has(r.start):
		return r._refuse("Start node '%s' does not exist." % r.start)
r._show(r.start)
return r
```

### `_refuse(message: String) -> DialogueRunner`
```logic
errors.append(message)
closed = true
return self
```

### `_node(id: String) -> Dictionary`
```logic
if id.begins_with(OFFER_PREFIX):
	var qid: String = id.trim_prefix(OFFER_PREFIX)
	var q: Dictionary = QuestRunner.resolved(gs, qid)
	var text: String = str(q.get("description", ""))
	if text.strip_edges().is_empty():
		text = str(q.get("name", qid))
	return {"id": id, "text": text, "_quest": qid, "choices": [
		{"text": _line("accept"), "to": start, "effects": [{"effect": "start_quest", "quest": qid}], "_quest": qid},
		{"text": _line("decline"), "to": start, "_quest": qid},
	]}
return DictUtil.dict_of(_nodes.get(id, {}))
```

### `_line(id: String) -> String`
```logic
return DialogueRules.service_line(gs.world_id, id)
```

### `_show(id: String) -> void`
```logic
node_id = id
_apply(DictUtil.array_of(JsonUtil.normalize(_node(id).get("effects", []))))
```

### `_tokens(extra: Dictionary) -> Dictionary`
```logic
var values: Dictionary = {"npc": character_name, "gold": gs.party.gold}
var qid: String = str(extra.get("_quest", ""))
if not qid.is_empty():
	values["quest"] = str(QuestRunner.resolved(gs, qid).get("name", qid))
var sqid: String = str(extra.get("_shop", ""))
if not sqid.is_empty():
	values["shop"] = str(ShopRules.shop_of(gs, sqid).get("name", sqid))
return values
```

### `view() -> Dictionary`
```logic
var node: Dictionary = _node(node_id)
var speaker: String = DialogueRules.character_qid(gs, str(node.get("speaker", "")))
if speaker.is_empty():
	speaker = _tree_speaker if not _tree_speaker.is_empty() else character_qid
var fq: String = DialogueRules.faction_qid(gs, character_qid)
var choices: Array[Dictionary] = []
if not closed:
	for c: Dictionary in _visible():
		choices.append({"text": DialogueRules.rep_prefix(gs, c) + DialogueRules.stat_prefix(c) + DialogueRules.fill(str(c.get("text", "")), _tokens(c)), "locked": DictUtil.to_bool(c["_locked"])})
return {
	"speaker": speaker, "speaker_name": str(WorldCatalog.resolve(gs.world_id, "characters", speaker).get("name", "")),
	"node": node_id, "text": DialogueRules.fill(str(node.get("text", "")), _tokens(node)), "choices": choices,
	"end": closed or DictUtil.to_bool(node.get("end", false)), "next": not closed and not str(node.get("next", "")).is_empty(),
	"shop": shop_qid, "pending": not _pending.is_empty(),
	"faction": fq, "faction_name": str(WorldCatalog.resolve(gs.world_id, "factions", fq).get("name", "")) if not fq.is_empty() else "",
	"band": RepRules.band(RepRules.standing(gs, fq)) if not fq.is_empty() else "",
}
```

### `static _fail(message: String) -> Dictionary`
```logic
var errs: Array[String] = [message]
return {"ok": false, "errors": errs}
```

### `_busy() -> String`
```logic
if closed:
	return "The conversation is over."
if not _pending.is_empty():
	return "Confirm or cancel the turn-in first."
if not shop_qid.is_empty():
	return "Close the shop first."
return ""
```

### `choose(index: int) -> Dictionary`
```logic
notes = []
return _with_notes(_choose(index))
```

### `_with_notes(res: Dictionary) -> Dictionary`
```logic
var all: Array[String] = []
all.assign(DictUtil.array_of(res.get("notes", [])))
all.append_array(notes)
res["notes"] = all
return res
```

### `_choose(index: int) -> Dictionary`
```logic
var busy: String = _busy()
if not busy.is_empty():
	return _fail(busy)
var visible: Array[Dictionary] = _visible()
if index < 0 or index >= visible.size():
	return _fail("There is no choice %d." % index)
var c: Dictionary = visible[index]
if DictUtil.to_bool(c["_locked"]):
	return _fail("That choice is locked.")
var effects: Array = DictUtil.array_of(JsonUtil.normalize(c.get("effects", [])))
var problems: Array[String] = _precheck(effects)
if not problems.is_empty():
	return {"ok": false, "errors": problems}
var no_errors: Array[String] = []
if not effects.is_empty() and str(DictUtil.dict_of(effects[0]).get("effect", "")) == "turn_in":
	var qid: String = str(QuestRunner.resolved(gs, str(DictUtil.dict_of(effects[0]).get("quest", ""))).get("_qid", ""))
	_pending = {"quest": qid, "rest": effects.slice(1), "to": str(c.get("to", ""))}
	return {"ok": true, "errors": no_errors, "pending": "turn_in", "quest": qid, "offers": QuestRunner.offers(gs, qid), "picks_allowed": QuestRunner.picks_allowed(gs, qid)}
_apply(effects)
var last: Dictionary = DictUtil.dict_of(effects.back()) if not effects.is_empty() else {}
match str(last.get("effect", "")):
	"fight":
		closed = true
		var enc: String = str(WorldCatalog.resolve(gs.world_id, "encounters", str(last.get("encounter", ""))).get("_qid", ""))
		return {"ok": true, "errors": no_errors, "fight": {"source": "dialogue", "encounter": enc, "character": character_qid}}
	"open_shop":
		shop_qid = str(ShopRules.shop_of(gs, str(last.get("shop", ""))).get("_qid", ""))
		var to_after: String = str(c.get("to", ""))
		_shop_to = to_after if not to_after.is_empty() else node_id
		return {"ok": true, "errors": no_errors, "shop": shop_qid}
return _move(str(c.get("to", "")))
```

### `_move(to: String) -> Dictionary`
```logic
var no_errors: Array[String] = []
if to.is_empty():
	closed = true
	return {"ok": true, "errors": no_errors, "end": true}
_show(to)
return {"ok": true, "errors": no_errors, "node": node_id}
```

### `_name_of(qid: String) -> String`
```logic
return str(WorldCatalog.resolve(gs.world_id, "characters", qid).get("name", qid))
```

### `_apply(effects: Array) -> void`
```logic
for raw: Variant in effects:
	var fx: Dictionary = DictUtil.dict_of(raw)
	match str(fx.get("effect", "")):
		"set_flag":
			var _err: String = QuestRunner.set_flag(gs, str(fx.get("key", "")), fx.get("value", true))
		"add_flag":
			var _add_err: String = QuestRunner.add_flag(gs, str(fx.get("key", "")), DictUtil.to_int(fx.get("delta", 0)), fx.get("min", null), fx.get("max", null))
		"start_quest":
			var _started: Array[String] = QuestRunner.start(gs, str(fx.get("quest", "")))
		"give":
			var given: Dictionary = RewardGrant.apply(gs, {"items": DictUtil.dict_of(fx.get("items", {})), "gold": DictUtil.to_int(fx.get("gold", 0))}, "")
			for n: Variant in DictUtil.array_of(given.get("notes", [])):
				notes.append(str(n))   # e.g. the gold-cap note (decision "DLG1 open items" (1))
		"take":
			var _taken: Array[String] = RewardGrant.take(gs, DictUtil.dict_of(fx.get("items", {})), DictUtil.to_int(fx.get("gold", 0)))
		"recruit":
			_recruit(str(fx.get("character", "")))
```

### `_recruit(who: String) -> void`
```logic
var hero: UnitState = NewGame.hero_from_character(gs.world_id, who)
if hero == null:
	return
for m: UnitState in gs.party.active_members():
	hero.total_xp = maxi(hero.total_xp, m.total_xp)
gs.party.add_member(hero)
if not gs.party.active_ids.is_empty():
	gs.party.active_ids.append(hero.id)
notes.append("%s joins the party." % PartyBuilder.display_name(hero))
```

### `confirm_turn_in(recipient_id: String = "", picks: Array[String] = []) -> Dictionary`
```logic
notes = []
if _pending.is_empty():
	return _fail("No turn-in is waiting.")
var again: Array = [{"effect": "turn_in", "quest": str(_pending["quest"])}]
again.append_array(DictUtil.array_of(_pending["rest"]))
var problems: Array[String] = _precheck(again)
if not problems.is_empty():
	return {"ok": false, "errors": problems}
var res: Dictionary = QuestRunner.complete(gs, str(_pending["quest"]), recipient_id, picks)
if not DictUtil.to_bool(res["ok"]):
	var errs: Array[String] = []
	errs.assign(DictUtil.array_of(res["errors"]))
	return {"ok": false, "errors": errs}
_apply(DictUtil.array_of(_pending["rest"]))
var to: String = str(_pending["to"])
_pending = {}
var moved: Dictionary = _move(to)
moved["granted"] = res["granted"]
moved["notes"] = res["notes"]
return _with_notes(moved)
```

### `cancel() -> Dictionary`
```logic
_pending = {}
var no_errors: Array[String] = []
return {"ok": true, "errors": no_errors, "node": node_id}
```

### `close_shop() -> Dictionary`
```logic
notes = []
return _with_notes(_close_shop())
```

### `_close_shop() -> Dictionary`
```logic
if shop_qid.is_empty():
	return _fail("No shop is open.")
shop_qid = ""
if _shop_to == node_id:
	var no_errors: Array[String] = []
	return {"ok": true, "errors": no_errors, "node": node_id}
return _move(_shop_to)
```

### `advance() -> Dictionary`
```logic
notes = []
return _with_notes(_advance())
```

### `_advance() -> Dictionary`
```logic
var busy: String = _busy()
if not busy.is_empty():
	return _fail(busy)
var node: Dictionary = _node(node_id)
var next: String = str(node.get("next", ""))
if not next.is_empty():
	return _move(next)
if DictUtil.to_bool(node.get("end", false)):
	return _move("")
return _fail("Pick a choice.")
```

### `static after_fight(p_gs: GameState, fight: Dictionary, outcome: String) -> Dictionary`
```logic
var applied: Dictionary = BattleBridge.apply(p_gs, str(fight.get("encounter", "")), outcome)
var events: Array[Dictionary] = []
var errs: Array[String] = []
errs.assign(DictUtil.array_of(applied["errors"]))
if not DictUtil.to_bool(applied["ok"]):
	return {"ok": false, "errors": errs, "xp": {}, "advanced": [] as Array[String], "events": events}
if outcome == "lost":
	var woke: Dictionary = ExploreRunner.wake(p_gs)
	for e: Variant in DictUtil.array_of(woke.get("events", [])):
		events.append(DictUtil.dict_of(e))
return {"ok": true, "errors": errs, "xp": applied["xp"], "advanced": applied["advanced"], "events": events}
```

