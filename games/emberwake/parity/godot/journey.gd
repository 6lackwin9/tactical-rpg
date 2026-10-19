extends SceneTree

# Aegis journey recorder (engine/docs/ENGINE_SPEC.md section 6): plays seeded random journeys through the shipped
# game-loop rules (exploration, dialogue, shops, quests, travel, rest, level-ups, battle outcomes, a save round trip)
# and records every action with its result, the game state after it and the read-only views the screens draw, for the
# Rust replay (engine/crates/aegis_world/tests/journey.rs). Battles are not played: an outcome is drawn and handed to
# the runner's after_fight, and "hurt" stands in for the HP a battle writes back. Nothing is saved; nothing in the
# project calls this file.
#   godot --headless --path <project> -s <abs path to this file> -- out=<abs dir> [seeds=1,2] [steps=300]

const DIRS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]
const STATS: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]

var out_dir: String = ""
var seeds: Array[int] = [1, 2]
var steps: int = 300
var rng := RandomNumberGenerator.new()

var gs: GameState = null
var runner: DialogueRunner = null
var talk_to: String = ""
var pending: Dictionary = {}       # the choose() result of a turn-in waiting for confirm
var fight: Dictionary = {}
var fight_runner: String = ""      # explore | travel | dialogue
var on_world_map: bool = false
var dialogue_actions: int = 0

func _initialize() -> void:
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("out="):
			out_dir = arg.substr(4)
		elif arg.begins_with("seeds="):
			seeds.clear()
			for s: String in arg.substr(6).split(","):
				seeds.append(s.to_int())
		elif arg.begins_with("steps="):
			steps = arg.substr(6).to_int()
	if out_dir.is_empty():
		print("§ journey: missing out=<dir>")
		quit(1)
		return
	DirAccess.make_dir_recursive_absolute(out_dir)
	var journeys: Array = []
	for w: String in NewGame.playable_worlds():
		for s: int in seeds:
			journeys.append(_journey(w, s))
	var path: String = out_dir.path_join("journeys.json")
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	f.store_string(JSON.stringify({"steps": steps, "journeys": journeys}, "", false))
	f.close()
	print("§ journey: wrote %s (%d journeys x %d steps)" % [path, journeys.size(), steps])
	quit()

func _journey(world: String, seed_value: int) -> Dictionary:
	rng.seed = seed_value * 7919 + world.hash()
	var made: Dictionary = NewGame.create_for_world(world, seed_value)
	gs = GameState.of(made["state"])
	runner = null
	talk_to = ""
	pending = {}
	fight = {}
	fight_runner = ""
	on_world_map = false
	dialogue_actions = 0
	var start: Dictionary = gs.to_dict()
	var records: Array = []
	var first: Dictionary = {"op": "begin"}
	records.append(_record(first, _run(first)))
	for _i: int in range(steps):
		var act: Dictionary = _choose()
		records.append(_record(act, _run(act)))
	return {"world": world, "seed": seed_value, "start": start, "records": records}

func _record(act: Dictionary, res: Dictionary) -> Dictionary:
	return {"action": act, "result": res, "state": gs.to_dict(), "views": _views()}

func _pick_index(size: int) -> int:
	return rng.randi_range(0, size - 1)

func _dir_value() -> Array:
	var d: Vector2i = DIRS[_pick_index(DIRS.size())]
	return [d.x, d.y]

# ------------------------------------------------------------------------------
# Choosing (Godot side only; the Rust replay runs the recorded actions)
# ------------------------------------------------------------------------------

func _choose() -> Dictionary:
	if not fight.is_empty():
		var roll: int = rng.randi_range(0, 99)
		var outcome: String = "won" if roll < 60 else ("fled" if roll < 80 else "lost")
		return {"op": "after_fight", "runner": fight_runner, "fight": fight.duplicate(true), "outcome": outcome}
	if not talk_to.is_empty():
		return {"op": "talk", "character": talk_to}
	if runner != null:
		return _choose_dialogue()
	if on_world_map:
		return _choose_map()
	return _choose_explore()

func _choose_dialogue() -> Dictionary:
	dialogue_actions += 1
	if runner.closed or dialogue_actions > 14:
		return {"op": "leave"}
	if not runner.shop_qid.is_empty():
		var r: int = rng.randi_range(0, 9)
		if r < 4:
			var offers: Array[Dictionary] = ShopRules.offers(gs, runner.shop_qid)
			if not offers.is_empty():
				var o: Dictionary = offers[_pick_index(offers.size())]
				return {"op": "buy", "kind": str(o["kind"]), "id": str(o["id"]), "count": rng.randi_range(1, 2)}
		elif r < 7:
			var sells: Array[Dictionary] = ShopRules.sell_list(gs, runner.shop_qid)
			if not sells.is_empty():
				var s: Dictionary = sells[_pick_index(sells.size())]
				return {"op": "sell", "item": str(s["item"]), "count": 1}
		return {"op": "close_shop"}
	if not pending.is_empty():
		if rng.randi_range(0, 9) == 0:
			return {"op": "cancel"}
		var offered: Array[String] = JsonUtil.strings(pending.get("offers", []))
		var allowed: int = DictUtil.to_int(pending.get("picks_allowed", 0))
		var picks: Array[String] = []
		for id: String in offered:
			if picks.size() < allowed and rng.randi_range(0, 1) == 0:
				picks.append(id)
		var recipient: String = ""
		if rng.randi_range(0, 2) == 0:
			recipient = gs.party.members[_pick_index(gs.party.members.size())].id
		return {"op": "confirm", "recipient": recipient, "picks": picks}
	var view: Dictionary = runner.view()
	var choices: Array = DictUtil.array_of(view.get("choices", []))
	if DictUtil.to_bool(view.get("end", false)) or (DictUtil.to_bool(view.get("next", false)) and (choices.is_empty() or rng.randi_range(0, 1) == 0)):
		return {"op": "advance"}
	if choices.is_empty():
		return {"op": "leave"}
	var open: Array[int] = []
	var quest_services: Array[int] = []
	for i: int in range(choices.size()):
		var c: Dictionary = DictUtil.dict_of(choices[i])
		if not DictUtil.to_bool(c.get("locked", false)):
			open.append(i)
			if ["turn_in", "offer"].has(str(c.get("service", ""))):
				quest_services.append(i)
	if open.is_empty() or rng.randi_range(0, 9) == 0:
		return {"op": "choose", "index": rng.randi_range(-1, choices.size())}
	# Quest services first (turn in, hear offers), and an offer node is mostly accepted: journeys reach quest turn-ins.
	if not quest_services.is_empty() and rng.randi_range(0, 9) < 8:
		return {"op": "choose", "index": quest_services[_pick_index(quest_services.size())]}
	if str(view.get("node", "")).begins_with(DialogueRunner.OFFER_PREFIX) and open.has(0) and rng.randi_range(0, 9) < 8:
		return {"op": "choose", "index": 0}
	return {"op": "choose", "index": open[_pick_index(open.size())]}

func _choose_map() -> Dictionary:
	var node: String = TravelRunner.current_node(gs)
	if node.is_empty():
		return {"op": "close_map"}
	var r: int = rng.randi_range(0, 9)
	if r < 6:
		var known: Array[String] = TravelRules.known(gs)
		known.erase(node)
		if not known.is_empty():
			return {"op": "travel", "to": known[_pick_index(known.size())]}
	if r < 8 and HealRules.can_camp(gs).is_empty():
		return {"op": "camp"}
	return {"op": "enter", "to": node}

func _choose_explore() -> Dictionary:
	var r: int = rng.randi_range(0, 99)
	if r < 3:
		var m: UnitState = gs.party.members[_pick_index(gs.party.members.size())]
		return {"op": "hurt", "member": m.id, "hp_lost": rng.randi_range(0, 40)}
	if r < 6:
		var m2: UnitState = gs.party.members[_pick_index(gs.party.members.size())]
		var draft: Dictionary = {}
		var points: int = maxi(m2.unspent(), 0) + rng.randi_range(0, 1)
		for _i: int in range(points):
			var stat: String = STATS[_pick_index(STATS.size())]
			draft[stat] = DictUtil.to_int(draft.get(stat, 0)) + 1
		return {"op": "level_up", "member": m2.id, "draft": draft}
	if r < 7:
		var active: Array[String] = QuestRunner.quests_with(gs, "active")
		if not active.is_empty():
			return {"op": "abandon", "quest": active[_pick_index(active.size())]}
	if r < 9:
		var ready: Array[String] = QuestRunner.quests_with(gs, "ready")
		if not ready.is_empty():
			var q: String = ready[_pick_index(ready.size())]
			var picks: Array[String] = QuestRunner.offers(gs, q).slice(0, QuestRunner.picks_allowed(gs, q))
			return {"op": "journal", "quest": q, "picks": picks}
	if r < 10:
		return {"op": "reload"}
	var loc: Dictionary = ExploreRunner.location(gs)
	if str(loc.get("map", "")).is_empty():
		var menu: Array[Dictionary] = ExploreRunner.menu(gs)
		if menu.is_empty():
			return {"op": "begin"}
		return {"op": "exit", "to": str(menu[_pick_index(menu.size())]["to"])}
	if rng.randi_range(0, 99) < 12:
		return {"op": "step", "dir": _dir_value()}
	return _toward_goal(loc)

# Goal-directed walking: a present marker is picked (weighted by kind), the party walks there by BFS over open,
# unblocked tiles, then faces and interacts (npc, chest, lore) or steps onto it (exit, encounter, point).
const GOAL_WEIGHTS: Dictionary = {"npc": 4, "exit": 2, "chest": 2, "lore": 1, "encounter": 3, "roamer": 2, "point": 1}
var goal: String = ""
var goal_loc: String = ""

func _toward_goal(loc: Dictionary) -> Dictionary:
	var t: Dictionary = ExploreRunner.template_of(loc)
	var loc_qid: String = str(loc["_qid"])
	var loc_steps: int = DictUtil.to_int(gs.explore.get("loc_steps", 0))
	var markers: Array[Dictionary] = ExploreRunner.markers_of(loc)
	var target: Dictionary = {}
	if goal_loc == loc_qid:
		for m: Dictionary in markers:
			if str(m["id"]) == goal and ExploreRunner.presence(gs, loc_qid, m) != "absent":
				target = m
	if target.is_empty():
		var pool: Array[Dictionary] = []
		var total: int = 0
		for m: Dictionary in markers:
			if ExploreRunner.presence(gs, loc_qid, m) != "absent":
				pool.append(m)
				total += DictUtil.to_int(GOAL_WEIGHTS.get(str(m["kind"]), 1))
		if pool.is_empty():
			return {"op": "step", "dir": _dir_value()}
		var r: int = rng.randi_range(0, total - 1)
		for m: Dictionary in pool:
			r -= DictUtil.to_int(GOAL_WEIGHTS.get(str(m["kind"]), 1))
			if r < 0:
				target = m
				break
		goal = str(target["id"])
		goal_loc = loc_qid
	var at: Vector2i = ExploreRunner.marker_pos(t, target, loc_steps)
	var solid: bool = ["npc", "chest", "lore"].has(str(target["kind"])) or ExploreRunner.presence(gs, loc_qid, target) == "locked"
	if solid:
		for d: Vector2i in DIRS:
			if gs.position + d == at:
				goal = ""
				return {"op": "interact", "dir": [d.x, d.y]}
	var first: Vector2i = _bfs_first(t, loc_qid, markers, loc_steps, at, solid)
	if first == Vector2i.ZERO:
		goal = ""
		return {"op": "step", "dir": _dir_value()}
	return {"op": "step", "dir": [first.x, first.y]}

# The first step of a shortest path to `at` (or to a tile next to it when `adjacent`); ZERO when unreachable.
func _bfs_first(t: Dictionary, loc_qid: String, markers: Array[Dictionary], loc_steps: int, at: Vector2i, adjacent: bool) -> Vector2i:
	var blocked: Dictionary = {}
	for m: Dictionary in markers:
		var state: String = ExploreRunner.presence(gs, loc_qid, m)
		if state == "locked" or (state == "present" and ["npc", "chest", "lore"].has(str(m["kind"]))):
			blocked[ExploreRunner.marker_pos(t, m, loc_steps)] = true
	var start: Vector2i = gs.position
	var first_of: Dictionary = {start: Vector2i.ZERO}
	var queue: Array[Vector2i] = [start]
	while not queue.is_empty():
		var cur: Vector2i = queue.pop_front()
		var reached: bool = (cur == at) if not adjacent else (absi(cur.x - at.x) + absi(cur.y - at.y) == 1)
		if reached and cur != start:
			var f: Variant = first_of[cur]
			return f if f is Vector2i else Vector2i.ZERO
		for d: Vector2i in DIRS:
			var nxt: Vector2i = cur + d
			if first_of.has(nxt) or not MapRules.is_open(t, nxt.x, nxt.y):
				continue
			if blocked.has(nxt) and (adjacent or nxt != at):
				continue
			var fd: Variant = first_of[cur]
			first_of[nxt] = d if cur == start else fd
			queue.append(nxt)
	return Vector2i.ZERO

# ------------------------------------------------------------------------------
# Running (mirrored by the Rust replay)
# ------------------------------------------------------------------------------

func _dir_of(act: Dictionary) -> Vector2i:
	var d: Array = DictUtil.array_of(act.get("dir", []))
	return Vector2i(DictUtil.to_int(d[0]), DictUtil.to_int(d[1])) if d.size() == 2 else Vector2i.ZERO

func _scan_events(res: Dictionary) -> void:
	for raw: Variant in DictUtil.array_of(res.get("events", [])):
		var e: Dictionary = DictUtil.dict_of(raw)
		match str(e.get("kind", "")):
			"fight":
				fight = e
				fight_runner = "explore"
			"exit":
				if str(e.get("to", "")) == MapRules.WORLD_MAP:
					on_world_map = true

func _run(act: Dictionary) -> Dictionary:
	var op: String = str(act["op"])
	var res: Dictionary = {}
	match op:
		"begin":
			res = ExploreRunner.begin(gs)
		"step":
			res = ExploreRunner.step(gs, _dir_of(act))
			_scan_events(res)
		"interact":
			res = ExploreRunner.interact(gs, _dir_of(act))
			if str(res.get("kind", "")) == "npc":
				talk_to = str(res.get("character", ""))
		"exit":
			res = ExploreRunner.take_exit(gs, str(act["to"]))
			_scan_events(res)
		"talk":
			talk_to = ""
			dialogue_actions = 0
			runner = DialogueRunner.open(gs, str(act["character"]))
			res = {"errors": runner.errors, "closed": runner.closed}
			if runner.closed:
				runner = null
		"choose":
			res = runner.choose(DictUtil.to_int(act["index"]))
			if res.has("fight"):
				fight = DictUtil.dict_of(res["fight"])
				fight_runner = "dialogue"
				runner = null
			elif res.has("pending"):
				pending = res
		"advance":
			res = runner.advance()
		"confirm":
			res = runner.confirm_turn_in(str(act["recipient"]), JsonUtil.strings(act["picks"]))
			if DictUtil.to_bool(res["ok"]):
				pending = {}
		"cancel":
			res = runner.cancel()
			pending = {}
		"buy":
			res = ShopRules.buy(gs, runner.shop_qid, str(act["kind"]), str(act["id"]), DictUtil.to_int(act["count"]))
		"sell":
			res = ShopRules.sell(gs, runner.shop_qid, str(act["item"]), DictUtil.to_int(act["count"]))
		"close_shop":
			res = runner.close_shop()
		"leave":
			runner = null
			pending = {}
		"after_fight":
			var f: Dictionary = DictUtil.dict_of(act["fight"])
			var outcome: String = str(act["outcome"])
			match str(act["runner"]):
				"explore":
					res = ExploreRunner.after_fight(gs, f, outcome)
				"travel":
					res = TravelRunner.after_fight(gs, f, outcome)
				_:
					res = DialogueRunner.after_fight(gs, f, outcome)
			fight = {}
		"travel":
			res = TravelRunner.travel(gs, str(act["to"]))
			if str(res.get("status", "")) == "fight":
				fight = DictUtil.dict_of(res["fight"])
				fight_runner = "travel"
		"camp":
			res = HealRules.make_camp(gs)
		"enter":
			res = ExploreRunner.enter(gs, str(act["to"]))
			on_world_map = false
		"close_map":
			on_world_map = false
		"hurt":
			var m: UnitState = gs.party.member(str(act["member"]))
			m.hp_lost = DictUtil.to_int(act["hp_lost"])
		"level_up":
			var m2: UnitState = gs.party.member(str(act["member"]))
			res = {"errors": LevelUp.confirm(m2, DictUtil.dict_of(act["draft"]))}
		"abandon":
			res = {"errors": QuestRunner.abandon(gs, str(act["quest"]))}
		"journal":
			res = DialogueRules.complete_from_journal(gs, str(act["quest"]), "", JsonUtil.strings(act["picks"]))
		"reload":
			var parsed: Variant = JSON.parse_string(JSON.stringify(gs.to_dict()))
			gs = GameState.from_dict(DictUtil.dict_of(parsed))
			res = ExploreRunner.resume(gs)
			res["valid"] = gs.validate()
	return res

# What the screens read after an action (all read-only).
func _views() -> Dictionary:
	var v: Dictionary = {}
	var quests: Dictionary = {}
	for e: Dictionary in WorldCatalog.merged_entries(gs.world_id, "quests"):
		var qid: String = str(e["_qid"])
		quests[qid] = QuestRunner.status(gs, qid)
	v["quests"] = quests
	var objectives: Array = []
	for qid: String in QuestRunner.quests_with(gs, "active"):
		objectives.append(QuestRunner.current_objective(gs, qid))
	v["objectives"] = objectives
	var offers: Dictionary = {}
	for qid: String in QuestRunner.quests_with(gs, "ready"):
		offers[qid] = {"offers": QuestRunner.offers(gs, qid), "picks": QuestRunner.picks_allowed(gs, qid)}
	v["turn_ins"] = offers
	var hp: Array = []
	for m: UnitState in gs.party.members:
		hp.append(HealRules.current_hp(m))
	v["hp"] = hp
	v["camp"] = HealRules.can_camp(gs)
	v["menu"] = ExploreRunner.menu(gs)
	var loc: Dictionary = ExploreRunner.location(gs)
	var markers: Array = []
	for raw: Variant in DictUtil.array_of(loc.get("markers", [])):
		markers.append(ExploreRunner.marker_view(gs, loc, DictUtil.dict_of(raw)))
	v["markers"] = markers
	if on_world_map:
		v["map"] = TravelRules.view(gs)
	if runner != null:
		v["dialogue"] = runner.view()
		if not runner.shop_qid.is_empty():
			v["shop_offers"] = ShopRules.offers(gs, runner.shop_qid)
			v["shop_sell"] = ShopRules.sell_list(gs, runner.shop_qid)
	return v
