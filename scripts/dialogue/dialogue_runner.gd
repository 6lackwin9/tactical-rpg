class_name DialogueRunner
extends RefCounted

# ==============================================================================
# Tactical RPG - Dialogue Runner (S11, spec DLG1-L 2.2-2.4): one conversation with one world character.
# One instance per conversation, never saved. open() picks the character's tree (`dialogue`; "" = a generated root
# with the `greeting` line and the service choices only). Showing a node runs its effects (every time it is shown),
# except the return to the node a shop was opened from (world-campaign.md "DLG1 shop return (user 2026-10-17)").
# Visible choices: authored ones whose requires hold and unless don't; failing ones with show_locked appear locked;
# the rest are hidden; then, on the start node with services on, the generated service choices in order: quest offers
# (resolved giver = this character, status available), turn-ins (effective turn-in = this character, status ready),
# open shops kept by this character, farewell. Service choices may push the start node past MAX_CHOICES (allowed: the
# cap is an authoring limit). choose(index) pre-checks every effect first (all-or-nothing); a turn_in waits for
# confirm_turn_in / cancel; open_shop waits for close_shop; fight closes the conversation and hands the fight back.
# Nodes with `next` (and end nodes) move on through advance().
# RECRUIT1-L: recruit adds a world party character to the roster (refused when already a member or the roster is
# full); a choice recruiting a member is hidden.
# REP1-L: add_flag adds to an int flag (a non-int flag refuses the choice; on a node it is skipped); view() carries the
# character's faction and its reputation band, and choices gated on a rep bound get the "[Faction band] " prefix.
# ==============================================================================

const ROOT: String = "$root"          # the generated root node of a character without a tree
const OFFER_PREFIX: String = "$offer:"   # generated offer nodes: "$offer:<quest qid>" (never a valid authored id)

var gs: GameState
var character_qid: String = ""
var character_name: String = ""
var errors: Array[String] = []        # non-empty = not opened
var closed: bool = false
var start: String = ""
var services: bool = true
var node_id: String = ""
var shop_qid: String = ""             # the open shop ("" = none)
var _tree_speaker: String = ""        # qid; "" = the character talked to
var _nodes: Dictionary = {}           # authored node id -> node (normalised)
var _pending: Dictionary = {}         # turn-in waiting for confirm: {quest, rest, to}
var _shop_to: String = ""
var notes: Array[String] = []         # notes of the last action (e.g. a gold-cap note from `give`); every result carries them

# ------------------------------------------------------------------------------
# Opening and showing
# ------------------------------------------------------------------------------

static func open(p_gs: GameState, character_ref: String) -> DialogueRunner:
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

func _refuse(message: String) -> DialogueRunner:
	errors.append(message)
	closed = true
	return self

# The current node: an authored node, the generated root or a generated offer node.
func _node(id: String) -> Dictionary:
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

func _line(id: String) -> String:
	return DialogueRules.service_line(gs.world_id, id)

# Shows node `id` and runs its effects (set_flag / give; validated content, so they can't fail half way).
func _show(id: String) -> void:
	node_id = id
	_apply(DictUtil.array_of(JsonUtil.normalize(_node(id).get("effects", []))))

# The node's own choices plus, on the start node with services on, the generated service choices.
func _choices_of(id: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(_node(id).get("choices", []))):
		out.append(DictUtil.dict_of(c))
	if id == start and services:
		out.append_array(_service_choices())
	return out

func _service_choices() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for qid: String in QuestRunner.quests_with(gs, "available"):
		if DialogueRules.giver_qid(gs, QuestRunner.resolved(gs, qid)) == character_qid:
			out.append({"text": _line("offer"), "to": OFFER_PREFIX + qid, "_quest": qid, "_service": "offer"})
	for qid: String in QuestRunner.quests_with(gs, "ready"):
		if DialogueRules.turn_in_qid(gs, QuestRunner.resolved(gs, qid)) == character_qid:
			out.append({"text": _line("turn_in"), "to": start, "effects": [{"effect": "turn_in", "quest": qid}], "_quest": qid, "_service": "turn_in"})
	for shop: Dictionary in WorldCatalog.merged_entries(gs.world_id, "shops"):
		var sqid: String = str(shop["_qid"])
		if DialogueRules.character_qid(gs, str(shop.get("keeper", ""))) == character_qid and ShopRules.is_open(gs, sqid):
			out.append({"text": _line("shop"), "to": "", "effects": [{"effect": "open_shop", "shop": sqid}], "_shop": sqid, "_service": "shop"})
	out.append({"text": _line("farewell"), "to": "", "_service": "farewell"})
	return out

# Choices as the player sees them: open ones, plus failing ones with show_locked (marked `_locked`).
func _visible() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for c: Dictionary in _choices_of(node_id):
		if DialogueRules.recruits_member(gs, c):
			continue   # RECRUIT1-L rule 6: hidden like a gone character's choice
		var is_open: bool = DialogueRules.choice_open(gs, c)
		if is_open or DictUtil.to_bool(c.get("show_locked", false)):
			var v: Dictionary = c.duplicate(true)
			v["_locked"] = not is_open
			out.append(v)
	return out

func _tokens(extra: Dictionary) -> Dictionary:
	var values: Dictionary = {"npc": character_name, "gold": gs.party.gold}
	var qid: String = str(extra.get("_quest", ""))
	if not qid.is_empty():
		values["quest"] = str(QuestRunner.resolved(gs, qid).get("name", qid))
	var sqid: String = str(extra.get("_shop", ""))
	if not sqid.is_empty():
		values["shop"] = str(ShopRules.shop_of(gs, sqid).get("name", sqid))
	return values

# {speaker (qid), speaker_name, node, text, choices: [{text, locked, reason (why a locked one is locked, UIR-6), service
# ("offer" / "turn_in" / "shop" / "farewell" for generated service choices, else "")}], end, next, shop, pending, faction (the talked-to
# character's faction qid, "" = none), faction_name, band (its reputation band, "" without a faction)}. `end`: the
# conversation is over or the node is an end node; `next`: the node moves on through advance().
func view() -> Dictionary:
	var node: Dictionary = _node(node_id)
	var speaker: String = DialogueRules.character_qid(gs, str(node.get("speaker", "")))
	if speaker.is_empty():
		speaker = _tree_speaker if not _tree_speaker.is_empty() else character_qid
	var fq: String = DialogueRules.faction_qid(gs, character_qid)
	var choices: Array[Dictionary] = []
	if not closed:
		for c: Dictionary in _visible():
			var locked: bool = DictUtil.to_bool(c["_locked"])
			choices.append({"text": DialogueRules.rep_prefix(gs, c) + DialogueRules.stat_prefix(c) + DialogueRules.fill(str(c.get("text", "")), _tokens(c)), "locked": locked,
				"reason": DialogueRules.lock_reason(gs, c) if locked else "", "service": str(c.get("_service", ""))})
	return {
		"speaker": speaker, "speaker_name": str(WorldCatalog.resolve(gs.world_id, "characters", speaker).get("name", "")),
		"node": node_id, "text": DialogueRules.fill(str(node.get("text", "")), _tokens(node)), "choices": choices,
		"end": closed or DictUtil.to_bool(node.get("end", false)), "next": not closed and not str(node.get("next", "")).is_empty(),
		"shop": shop_qid, "pending": not _pending.is_empty(),
		"faction": fq, "faction_name": str(WorldCatalog.resolve(gs.world_id, "factions", fq).get("name", "")) if not fq.is_empty() else "",
		"band": RepRules.band(RepRules.standing(gs, fq)) if not fq.is_empty() else "",
	}

# ------------------------------------------------------------------------------
# Choosing
# ------------------------------------------------------------------------------

static func _fail(message: String) -> Dictionary:
	var errs: Array[String] = [message]
	return {"ok": false, "errors": errs}

func _busy() -> String:
	if closed:
		return "The conversation is over."
	if not _pending.is_empty():
		return "Confirm or cancel the turn-in first."
	if not shop_qid.is_empty():
		return "Close the shop first."
	return ""

func choose(index: int) -> Dictionary:
	notes = []
	return _with_notes(_choose(index))

# Copies the action's notes into its result (`notes`, after any notes the result already holds).
func _with_notes(res: Dictionary) -> Dictionary:
	var all: Array[String] = []
	all.assign(DictUtil.array_of(res.get("notes", [])))
	all.append_array(notes)
	res["notes"] = all
	return res

func _choose(index: int) -> Dictionary:
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

func _move(to: String) -> Dictionary:
	var no_errors: Array[String] = []
	if to.is_empty():
		closed = true
		return {"ok": true, "errors": no_errors, "end": true}
	_show(to)
	return {"ok": true, "errors": no_errors, "node": node_id}

# Every effect checked against the current state (DLG1-L 2.4 step 2); [] = all of them can run.
func _precheck(effects: Array) -> Array[String]:
	var out: Array[String] = []
	var take_items: Dictionary = {}
	var take_gold: int = 0
	for raw: Variant in effects:
		var fx: Dictionary = DictUtil.dict_of(raw)
		match str(fx.get("effect", "")):
			"set_flag":
				if not GameState.flag_key_ok(str(fx.get("key", ""))) or not GameState.flag_value_ok(fx.get("value", true)):
					out.append("Flag '%s' can't be set." % str(fx.get("key", "")))
			"add_flag":
				var fkey: String = str(fx.get("key", ""))
				if not GameState.flag_key_ok(fkey) or not (fx.get("delta", null) is int):
					out.append("Flag '%s' can't be changed." % fkey)
				elif not (gs.get_flag(fkey, 0) is int):
					out.append("Flag '%s' is not a number." % fkey)
			"start_quest":
				var ref: String = str(fx.get("quest", ""))
				var st: String = QuestRunner.status(gs, ref)
				if st != "available":
					out.append("Quest '%s' can't start (it is %s)." % [ref, st])
				else:
					var giver: String = DialogueRules.giver_qid(gs, QuestRunner.resolved(gs, ref))
					if not giver.is_empty() and giver != character_qid:
						out.append("Only %s gives this quest." % _name_of(giver))
			"turn_in":
				var ref: String = str(fx.get("quest", ""))
				var st: String = QuestRunner.status(gs, ref)
				if st != "ready":
					out.append("Quest '%s' can't be turned in (it is %s)." % [ref, st])
				else:
					var taker: String = DialogueRules.turn_in_qid(gs, QuestRunner.resolved(gs, ref))
					if not taker.is_empty() and taker != character_qid:
						out.append("This quest is turned in to %s." % _name_of(taker))
			"give":
				for id: Variant in DictUtil.dict_of(fx.get("items", {})):
					if not ContentCatalog.has("items", str(id)):
						out.append("Unknown item '%s'." % str(id))
			"take":
				var items: Dictionary = JsonUtil.ints(fx.get("items", {}))
				for id: Variant in items:
					take_items[id] = DictUtil.to_int(take_items.get(id, 0)) + DictUtil.to_int(items[id])
				take_gold += DictUtil.to_int(fx.get("gold", 0))
			"open_shop":
				if not ShopRules.is_open(gs, str(fx.get("shop", ""))):
					out.append("The shop is closed.")
			"fight":
				if WorldCatalog.resolve(gs.world_id, "encounters", str(fx.get("encounter", ""))).is_empty():
					out.append("Unknown encounter '%s'." % str(fx.get("encounter", "")))
			"recruit":
				var who: String = str(fx.get("character", ""))
				var cap: int = WorldCatalog.party_max(gs.world_id)
				if NewGame.hero_from_character(gs.world_id, who) == null:
					out.append("'%s' can't join the party." % who)
				elif DialogueRules.is_member(gs, who):
					out.append("%s is already with you." % _name_of(who))
				elif gs.party.members.size() >= cap:
					out.append("The party is full (%d of %d)." % [gs.party.members.size(), cap])
	for id: Variant in take_items:
		if DictUtil.to_int(gs.party.stash.get(str(id), 0)) < DictUtil.to_int(take_items[id]):
			out.append("Not enough %s." % str(ContentCatalog.get_entry("items", str(id)).get("name", id)))
	if take_gold > gs.party.gold:
		out.append("Not enough gold.")
	return out

func _name_of(qid: String) -> String:
	return str(WorldCatalog.resolve(gs.world_id, "characters", qid).get("name", qid))

# Applies effects in order (pre-checked): set_flag, add_flag, start_quest, give, take, recruit. open_shop / fight move, they change nothing.
func _apply(effects: Array) -> void:
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

# RECRUIT1-L rule 5 (pre-checked): the hero joins with total_xp = max(its base level's threshold, the highest active
# total_xp), is appended to an explicit active_ids, and the event note "%s joins the party." is added.
func _recruit(who: String) -> void:
	var hero: UnitState = NewGame.hero_from_character(gs.world_id, who)
	if hero == null:
		return
	for m: UnitState in gs.party.active_members():
		hero.total_xp = maxi(hero.total_xp, m.total_xp)
	gs.party.add_member(hero)
	if not gs.party.active_ids.is_empty():
		gs.party.active_ids.append(hero.id)
	notes.append("%s joins the party." % PartyBuilder.display_name(hero))

# ------------------------------------------------------------------------------
# Turn-in, shop, moving on, fights
# ------------------------------------------------------------------------------

# Completes the pending quest (QuestRunner.complete, all-or-nothing), then the trailing effects, then moves on. The
# turn-in and the trailing effects are pre-checked again first: the state may have changed since choose().
func confirm_turn_in(recipient_id: String = "", picks: Array[String] = []) -> Dictionary:
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

# Drops the pending turn-in; the node stays and nothing changes.
func cancel() -> Dictionary:
	_pending = {}
	var no_errors: Array[String] = []
	return {"ok": true, "errors": no_errors, "node": node_id}

# Leaves the shop: shows the choice's `to` node ("" = the node the shop was opened from). Returning to the node the
# shop was opened from does not run its effects again (user decision 2026-10-17); another `to` node is shown as usual.
func close_shop() -> Dictionary:
	notes = []
	return _with_notes(_close_shop())

func _close_shop() -> Dictionary:
	if shop_qid.is_empty():
		return _fail("No shop is open.")
	shop_qid = ""
	if _shop_to == node_id:
		var no_errors: Array[String] = []
		return {"ok": true, "errors": no_errors, "node": node_id}
	return _move(_shop_to)

# Moves on from a `next` node (shows the next node) or an end node (closes the conversation).
func advance() -> Dictionary:
	notes = []
	return _with_notes(_advance())

func _advance() -> Dictionary:
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

# After a dialogue fight: BattleBridge.apply (XP + quest hook on won), plus ExploreRunner.wake on lost.
# Returns {ok, errors, xp, advanced, events, gold?, loot?} (gold / loot = a paid purse, PURSE1).
static func after_fight(p_gs: GameState, fight: Dictionary, outcome: String) -> Dictionary:
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
	var out: Dictionary = {"ok": true, "errors": errs, "xp": applied["xp"], "advanced": applied["advanced"], "events": events}
	out.merge(PurseRules.result_keys(DictUtil.dict_of(applied.get("purse", {}))))   # PURSE1 1.6: gold / loot lines
	return out
