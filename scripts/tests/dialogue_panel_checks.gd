class_name DialoguePanelChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - DLG1-L commit 3 checks, run by DialogueTestSuite on its runtime fixture world (dg_run):
# the shop return rule (user decision 2026-10-17: closing a shop back to its node does not re-run the node's effects)
# and the debug panels in the region map scene (talk, locked choice, shop buy and close, turn-in step, dialogue fight).
# DLG1-F: buy count cap, selling at the gold cap, the turn-in pre-check on confirm, gold-cap notes from dialogue give.
# POI-FIX: real key and mouse events through the main-menu-hosted region map (hint, bump note, E on chest / npc /
# nothing, click on a marker next to the party, click on a far one).
# ==============================================================================

const DEBUG_SCENE: String = "res://scenes/overworld/region_map_debug.tscn"
const MENU_SCENE: String = "res://scenes/main_menu.tscn"
const R: String = DialogueTestSuite.R

static func run(tree_parent: Node) -> bool:
	var ok: bool = _test_shop_return()
	ok = _test_followups() and ok
	ok = _test_panels(tree_parent) and ok
	ok = _test_map_input(tree_parent) and ok
	return ok

static func _q(local: String) -> String:
	return WorldCatalog.qid(R, local)

static func _report(label: String, checks: Dictionary) -> bool:
	return DialogueTestSuite._report(label, checks)

# Kit: a start node that gives 1 gold, a choice opening the shop and staying ("" to), one opening it then moving to a
# node that gives 2 gold, and the shop service.
static func _build_kit() -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "kit", "name": "Kit", "base_unit": str(DialogueTestSuite._rx["lo"]), "dialogue": "d_kit"}))
	errors.append_array(WorldCatalog.upsert(R, "shops", {"id": "s_kit", "name": "Kit's cart", "keeper": "kit", "stock": [{"item": DialogueTestSuite.I25, "count": -1, "price": -1, "requires_flags": []}]}))
	errors.append_array(WorldCatalog.upsert(R, "dialogues", {"id": "d_kit", "name": "Kit", "start": "hi", "nodes": [
		{"id": "hi", "text": "Coin for you.", "effects": [{"effect": "give", "gold": 1}], "choices": [
			{"text": "Browse", "to": "", "effects": [{"effect": "open_shop", "shop": "s_kit"}]},
			{"text": "Browse and go", "to": "after", "effects": [{"effect": "open_shop", "shop": "s_kit"}]},
		]},
		{"id": "after", "text": "Two more.", "effects": [{"effect": "give", "gold": 2}], "end": true},
	]}))
	return errors.is_empty() and WorldCatalog.validate_world(R).is_empty()

static func _test_shop_return() -> bool:
	var checks: Dictionary = {"kit fixture": _build_kit()}
	var gs: GameState = DialogueTestSuite._gs()
	var gold0: int = gs.party.gold
	var r: DialogueRunner = DialogueRunner.open(gs, "kit")
	checks["opening runs the node effects"] = r.errors.is_empty() and gs.party.gold == gold0 + 1
	var opened: Dictionary = r.choose(DialogueTestSuite._find(r.view(), "Browse"))
	var back: Dictionary = r.close_shop()
	checks["closing back to the node does not re-run its effects"] = str(opened.get("shop", "")) == _q("s_kit") and DictUtil.to_bool(back["ok"]) and r.node_id == "hi" and gs.party.gold == gold0 + 1
	var service: Dictionary = r.choose(DialogueTestSuite._find(r.view(), DialogueTestSuite._line("shop")))
	var _s: Dictionary = r.close_shop()
	checks["the shop service returns the same way"] = str(service.get("shop", "")) == _q("s_kit") and r.node_id == "hi" and gs.party.gold == gold0 + 1 and r.shop_qid.is_empty()
	var _o: Dictionary = r.choose(DialogueTestSuite._find(r.view(), "Browse and go"))
	var moved: Dictionary = r.close_shop()
	checks["a different to node is shown with its effects"] = str(moved.get("node", "")) == "after" and gs.party.gold == gold0 + 3
	return _report("Shop return (user 2026-10-17): back to the opening node without its effects; another to node runs them", checks)

static func _test_followups() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = DialogueTestSuite._gs()
	gs.party.gold = 1000
	var before: Dictionary = gs.to_dict()
	var big: Dictionary = ShopRules.buy(gs, "s_hest", "item", DialogueTestSuite.I25, ShopRules.BUY_MAX + 1)
	var huge: Dictionary = ShopRules.buy(gs, "s_hest", "item", DialogueTestSuite.I25, 1 << 50)
	checks["buy count above BUY_MAX refused, nothing changes"] = not DictUtil.to_bool(big["ok"]) and not DictUtil.to_bool(huge["ok"]) and str(huge["errors"]).contains("at most") and JsonUtil.deep_equal(gs.to_dict(), before)
	gs.party.add_to_stash(DialogueTestSuite.I25, 1)
	gs.party.gold = PartyState.GOLD_MAX - 1
	var sold: Dictionary = ShopRules.sell(gs, "s_hest", DialogueTestSuite.I25, 1)
	checks["selling at the gold cap reports the added amount"] = DictUtil.to_bool(sold["ok"]) and gs.party.gold == PartyState.GOLD_MAX and DictUtil.to_int(sold["gold"]) == 1 and DictUtil.to_int(sold["price"]) > 1 and str(sold["notes"]).contains("capped")
	var r: DialogueRunner = DialogueRunner.open(gs, "kit")
	var _o: Dictionary = r.choose(DialogueTestSuite._find(r.view(), "Browse and go"))
	var moved: Dictionary = r.close_shop()
	checks["a dialogue give at the cap surfaces the note"] = str(moved.get("node", "")) == "after" and str(moved.get("notes", [])).contains("capped") and gs.party.gold == PartyState.GOLD_MAX
	var gs2: GameState = DialogueTestSuite._gs()
	var _st: Array[String] = QuestRunner.start(gs2, _q("q_hest"))
	var _f: String = QuestRunner.set_flag(gs2, "dr.hest", true)
	var hest: DialogueRunner = DialogueRunner.open(gs2, "hest")
	var pend: Dictionary = hest.choose(DialogueTestSuite._find(hest.view(), "Hand it over"))
	var _done: Dictionary = QuestRunner.complete(gs2, _q("q_hest"))
	var late: Dictionary = hest.confirm_turn_in()
	checks["confirm re-runs the pre-check"] = str(pend.get("pending", "")) == "turn_in" and not DictUtil.to_bool(late["ok"]) and " ".join(PackedStringArray(DictUtil.array_of(late["errors"]))).contains("be turned in (it is completed)") and not gs2.has_flag("dr.thanked")
	return _report("DLG1-F: buy count cap, sell and give at the gold cap report, confirm_turn_in pre-check", checks)

static func _test_panels(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	var packed: PackedScene = load(DEBUG_SCENE) as PackedScene
	checks["scene loads"] = packed != null
	if packed == null or tree_parent == null:
		return _report("Dialogue and shop panels (headless without a tree: load only)", checks)
	var view: RegionMapView = packed.instantiate() as RegionMapView
	tree_parent.add_child(view)
	checks["new game"] = view.start_game(R).is_empty() and view.gs != null
	if view.gs == null:
		tree_parent.remove_child(view)
		view.free()
		return _report("Dialogue and shop panels", checks)
	var gs: GameState = view.gs
	CurrentGame.start(gs)
	var dp: DialoguePanel = view.dialogue_panel
	var sp: ShopPanel = view.shop_panel
	# locked choice greyed: Mara's KNOW-gated Ask with only the low-KNOW member active
	gs.party.active_ids = [str(DialogueTestSuite._rx["lo"])] as Array[String]
	view.talk("mara")
	var ask: int = DialogueTestSuite._find(dp.runner.view(), "Ask") if dp.runner != null else -1
	var ask_btn: Button = dp._choices.get_child(ask) as Button if ask >= 0 and ask < dp._choices.get_child_count() else null
	checks["talk opens the dialogue panel and blocks the map"] = dp.is_open() and view.busy()
	checks["CurrentGame is busy during the dialogue"] = CurrentGame.busy()
	checks["a locked choice is a disabled button"] = ask_btn != null and ask_btn.disabled
	var _bye: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), DialogueTestSuite._line("farewell")))
	checks["farewell closes the panel"] = not dp.is_open() and not view.busy()
	checks["CurrentGame busy clears when the dialogue closes"] = not CurrentGame.busy()
	# shop: open from Hest, buy one, close back to the dialogue
	view.talk("hest")
	var _shop: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), DialogueTestSuite._line("shop")))
	checks["the shop service swaps in the shop panel"] = sp.is_open() and not dp.visible and sp.shop_qid == _q("s_hest") and view.busy() and CurrentGame.busy()
	var gold0: int = gs.party.gold
	var bought: Dictionary = sp.buy_one("item", DialogueTestSuite.I25)
	checks["buy goes through ShopRules"] = DictUtil.to_bool(bought["ok"]) and gs.party.gold < gold0 and DictUtil.to_int(gs.party.stash.get(DialogueTestSuite.I25, 0)) >= 1
	sp.close()
	checks["close returns to the dialogue"] = not sp.is_open() and dp.is_open() and dp.runner.node_id == dp.runner.start
	# turn-in step: recipient and loot pick
	var _st: Array[String] = QuestRunner.start(gs, _q("q_hest"))
	var _f: String = QuestRunner.set_flag(gs, "dr.hest", true)
	dp.refresh()
	var _hand: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), "Hand it over"))
	checks["turn-in shows the recipient and loot step"] = not dp.pending.is_empty() and dp._turn_in.get_child_count() > 0 and not dp.recipient_id.is_empty()
	dp.toggle_pick(_q("l_gem"), true)
	var done: Dictionary = dp.confirm_turn_in()
	checks["confirm completes the quest with the pick"] = DictUtil.to_bool(done.get("ok", false)) and QuestRunner.status(gs, _q("q_hest")) == "completed" and gs.loot_claimed.has(_q("l_gem")) and gs.has_flag("dr.thanked") and dp.pending.is_empty()
	# dialogue fight settles through the outcome buttons
	var _fight: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), "Fight"))
	checks["a dialogue fight opens the outcome panel"] = str(view.pending_fight.get("source", "")) == "dialogue" and not dp.is_open() and view.busy()
	view.settle("won")
	checks["won settles through DialogueRunner.after_fight"] = view.pending_fight.is_empty() and not view.busy() and not CurrentGame.busy()
	view.talk("mara")
	checks["a view freed mid-talk clears the busy lock"] = CurrentGame.busy()
	tree_parent.remove_child(view)
	checks["exit_tree clears the busy lock"] = not CurrentGame.busy()
	CurrentGame.end()
	view.free()
	return _report("Dialogue and shop panels (talk, locked choice, shop buy and close, turn-in step, dialogue fight)", checks)

# A key press and release pushed through the viewport (_input, GUI, then _unhandled_input), not a direct handler call.
static func _key(vp: Viewport, code: Key) -> void:
	for down: bool in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		vp.push_input(ev, true)

# A left click on `tile` of the map canvas, pushed through the viewport (the GUI picks the control under it).
static func _click(vp: Viewport, view: RegionMapView, tile: Vector2i) -> void:
	var canvas: Control = view._canvas
	var at: Vector2 = canvas.get_global_rect().position + view.tile_centre(tile)   # MAPVIEW1: camera offset applied
	for down: bool in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = down
		ev.position = at
		ev.global_position = at
		vp.push_input(ev, true)

static func _last(view: RegionMapView) -> String:
	return "" if view.log_lines.is_empty() else view.log_lines[-1]

static func _farewell(dp: DialoguePanel) -> void:
	if dp.runner != null:
		var _bye: Dictionary = dp.pick(DialogueTestSuite._find(dp.runner.view(), DialogueTestSuite._line("farewell")))

static func _test_map_input(tree_parent: Node) -> bool:
	var label: String = "POI-FIX: map input through the main menu host (hint, bump note, E opens chest / npc / nothing, click next to / far)"
	var checks: Dictionary = {}
	if tree_parent == null:
		checks["scene loads"] = load(MENU_SCENE) is PackedScene
		return _report(label + " (headless without a tree: load only)", checks)
	var layer := CanvasLayer.new()   # on top of anything earlier suites left in the arena, so clicks reach the map
	layer.layer = 128
	tree_parent.add_child(layer)
	var host := Control.new()
	host.size = SettingsUiTestSuite.VIEW
	layer.add_child(host)
	var mm: MainMenuView = (load(MENU_SCENE) as PackedScene).instantiate() as MainMenuView
	host.add_child(mm)
	var gs: GameState = DialogueTestSuite._gs()
	var entered: Dictionary = ExploreRunner.enter(gs, "yard")   # the fixture world has no start location
	CurrentGame.start(gs)
	mm.open_game(gs)
	var view: RegionMapView = mm.game_view as RegionMapView
	checks["the main menu hosts the region map in the yard"] = DictUtil.to_bool(entered["ok"]) and view != null and view.gs == gs and gs.location_id == _q("yard")
	if view == null:
		CurrentGame.end()
		tree_parent.remove_child(layer)
		layer.free()
		return _report(label, checks)
	SettingsUiTestSuite.layout(mm)
	var vp: Viewport = mm.get_viewport()
	var dp: DialoguePanel = view.dialogue_panel
	checks["a hint line names the keys"] = view.hint != null and view.hint.is_visible_in_tree() and view.hint.text == RegionMapView.HINT
	var t: Dictionary = ExploreRunner.template_of(ExploreRunner.location(gs))
	var chest: Vector2i = ExploreRunner.marker_pos(t, DialogueTestSuite._marker("box"), 0)
	var mara: Vector2i = ExploreRunner.marker_pos(t, DialogueTestSuite._marker("npc_mara"), 0)
	var hest: Vector2i = ExploreRunner.marker_pos(t, DialogueTestSuite._marker("npc_hest"), 0)
	# chest: walk one step, bump it, E opens it, E again finds it empty
	gs.position = chest + Vector2i(0, -2)
	view._refresh()
	_key(vp, KEY_DOWN)
	checks["an arrow key steps"] = gs.position == chest + Vector2i(0, -1)
	_key(vp, KEY_DOWN)
	checks["bumping the chest: no move, faces it, notes press E"] = gs.position == chest + Vector2i(0, -1) and view.facing == Vector2i(0, 1) and _last(view) == str(RegionMapView.BUMP_NOTES["chest"])
	var gold0: int = gs.party.gold
	_key(vp, KEY_E)
	checks["E opens the chest"] = gs.party.gold == gold0 + DialogueTestSuite.CHEST_GOLD and _last(view).begins_with("Opened")
	_key(vp, KEY_E)
	checks["E again: the chest is empty"] = _last(view) == "The chest is empty." and gs.party.gold == gold0 + DialogueTestSuite.CHEST_GOLD
	# nothing: face away from the chest
	_key(vp, KEY_UP)
	_key(vp, KEY_UP)
	_key(vp, KEY_E)
	checks["E facing no marker notes it"] = view._marker_kind_at(gs.position + view.facing).is_empty() and _last(view) == RegionMapView.NOTHING_NOTE
	# npc: bump, then E opens the conversation
	gs.position = mara + Vector2i(0, 1)
	view._refresh()
	_key(vp, KEY_UP)
	checks["bumping an npc notes press E"] = gs.position == mara + Vector2i(0, 1) and _last(view) == str(RegionMapView.BUMP_NOTES["npc"])
	_key(vp, KEY_E)
	checks["E opens the npc conversation"] = dp.is_open() and dp.runner != null and dp.runner.character_name == "Mara" and view.busy()
	_farewell(dp)
	checks["farewell frees the map"] = not dp.is_open() and not view.busy()
	# click: a marker next to the party faces it and talks; a far one notes walk closer
	gs.position = hest + Vector2i(-1, 0)
	view.facing = Vector2i(0, 1)
	view._refresh()
	_click(vp, view, hest)
	checks["a click on an npc next to the party faces it and talks"] = view.facing == Vector2i(1, 0) and dp.is_open() and dp.runner != null and dp.runner.character_name == "Hest"
	_farewell(dp)
	var far: Vector2i = mara if not ExploreRunner.DIRS.has(mara - gs.position) else chest
	var pos0: Vector2i = gs.position
	_click(vp, view, far)
	checks["a click on a marker further away notes walk closer"] = not ExploreRunner.DIRS.has(far - pos0) and _last(view) == RegionMapView.FAR_NOTE and not dp.is_open() and gs.position == pos0
	CurrentGame.end()
	tree_parent.remove_child(layer)
	layer.free()
	return _report(label, checks)
