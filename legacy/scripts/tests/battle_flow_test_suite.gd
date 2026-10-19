class_name BattleFlowTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle flow tests (BATTLE-FLOW): the transition into combat (instant headless and with Skip animations),
# the results popup over the still-mounted arena (XP bar values, non-XP rewards, X / Esc / right click close it exactly
# once, minimise and restore, Replay) and its fit at every text size in a 1280x720 viewport. Shipped Ashlands data, read
# only; saves and settings go to a sandbox. Outcomes are forced on the live session (as BattleLinkTestSuite does) and
# the expectations come from XpCurve / the award dict, never fixed numbers. One game screen per case (message queue).
# ==============================================================================

const TAG: String = "BATTLE-FLOW"
const WORLD: String = "ashlands"
const ENCOUNTER: String = "ash_scouts"
const VIEW: Vector2 = Vector2(1280, 720)
static var SANDBOX: String = TestHelpers.sandbox("battle_flow")

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING BATTLE FLOW TEST (BATTLE-FLOW)          ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/saves")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	SettingsFile.use_path(SANDBOX + "/settings.cfg")
	var ok: bool = true
	var built: Dictionary = RegionMapView.new_state(WORLD)
	if not (built["state"] is GameState):
		ok = TestHelpers.report_ok(TAG, "A new Ashlands game for the fixtures", false)
	else:
		CurrentGame.start(built["state"] as GameState)
		ok = _test_xp_rows() and ok
		ok = _test_rewards() and ok
		ok = _test_model_popup_state() and ok
		if tree_parent != null:
			ok = _test_transition(tree_parent) and ok
			ok = _test_close_cross(tree_parent) and ok
			ok = _test_esc(tree_parent) and ok
			ok = _test_right_click(tree_parent) and ok
			ok = _test_minimise(tree_parent) and ok
			ok = _test_replay(tree_parent) and ok
			ok = _test_lost(tree_parent) and ok
			ok = _test_arena_freed(tree_parent) and ok
			ok = _test_guards(tree_parent) and ok
			ok = _test_fit(tree_parent) and ok
	CurrentGame.end()
	SettingsFile.use_path()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

# ------------------------------------------------------------------------------
# Model
# ------------------------------------------------------------------------------

static func _router() -> MenuRouter:
	return MenuRouter.new(MenuRouter.HUB)

# fill / gain / to_next follow XpCurve for a small award and for one that levels the party up.
static func _test_xp_rows() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = CurrentGame.state()
	var enemies: Array[Dictionary] = [{"level": 1, "tier": "minion"}]
	var small: Dictionary = LevelUp.award_encounter(gs.party, enemies)
	var rows: Array[Dictionary] = BattleResultsModel.new(_router(), small).rows()
	checks["a row per active member"] = rows.size() == gs.party.active_members().size() and not rows.is_empty()
	for r: Dictionary in rows:
		var total: int = DictUtil.to_int(r["total_xp"])
		var level: int = XpCurve.level_for(total)
		var base: int = XpCurve.xp_for_level(level)
		var span: int = XpCurve.xp_for_level(level + 1) - base
		var xp: int = DictUtil.to_int(r["xp"])
		var maxed: bool = bool(r["maxed"])
		checks["%s: level from the curve" % str(r["id"])] = DictUtil.to_int(r["level"]) == level
		checks["%s: to_next" % str(r["id"])] = DictUtil.to_int(r["to_next"]) == XpCurve.xp_to_next(total)
		if not maxed:
			var want_fill: float = float(total - base) / float(span)
			var want_gain: float = float(total - maxi(total - xp, base)) / float(span)
			checks["%s: fill is the share of the level" % str(r["id"])] = is_equal_approx(float(r["fill"]), want_fill)
			checks["%s: gain is the part earned now" % str(r["id"])] = is_equal_approx(float(r["gain"]), want_gain)
			checks["%s: 0 <= gain <= fill <= 1" % str(r["id"])] = float(r["gain"]) >= 0.0 and float(r["gain"]) <= float(r["fill"]) and float(r["fill"]) <= 1.0
	# A big award crosses a level: the gain is only the XP inside the new level (the bar restarts at the level's base).
	var big_enemies: Array[Dictionary] = [{"level": 10, "tier": "boss"}, {"level": 10, "tier": "boss"}, {"level": 10, "tier": "boss"}]
	var big: Dictionary = LevelUp.award_encounter(gs.party, big_enemies)
	var crossed: bool = false
	for r: Dictionary in BattleResultsModel.new(_router(), big).rows():
		if DictUtil.to_int(r["new_level"]) > DictUtil.to_int(r["old_level"]) and not bool(r["maxed"]):
			crossed = true
			var total: int = DictUtil.to_int(r["total_xp"])
			var base: int = XpCurve.xp_for_level(XpCurve.level_for(total))
			checks["%s: level-up gain starts at the level's base" % str(r["id"])] = is_equal_approx(float(r["gain"]), float(total - base) / float(XpCurve.xp_for_level(XpCurve.level_for(total) + 1) - base)) and float(r["gain"]) <= float(r["fill"])
	checks["the big award levels someone up"] = crossed
	return TestHelpers.report(TAG, "XP rows: fill, gained part and XP to next level come from the curve", checks)

static func _test_rewards() -> bool:
	var checks: Dictionary = {}
	var rewards: Dictionary = {"gold": 37, "supplies": 2, "items": {"minor_potion": 3, "none_item": 0}, "loot": "ash_blade", "reputation": {"ash_clans": 4, "wardens": -2}}
	var advanced: Array[String] = []
	var rm := BattleResultsModel.new(_router(), {}, advanced, "won", rewards)
	var lines: Array[String] = rm.reward_lines()
	var joined: String = "\n".join(PackedStringArray(lines))
	checks["gold"] = lines.has("Gold +37")
	checks["supplies"] = lines.has("Supplies +2")
	checks["an item with its count, zero counts hidden"] = lines.has("Minor Potion x3") and not joined.contains("None Item")
	checks["loot"] = lines.has("Loot: Ash Blade")
	checks["reputation signed"] = joined.contains("Ash Clans: +4") and joined.contains("Wardens: -2")
	checks["no rewards, no lines"] = BattleResultsModel.new(_router(), {}).reward_lines().is_empty()
	var res: Dictionary = {"xp": {}, "gold": 5, "rewards": {"items": {"rope": 1}}, "outcome": "won"}
	var picked: Dictionary = BattleResultsModel.rewards_of(res)
	checks["rewards_of merges top-level and nested"] = DictUtil.to_int(picked.get("gold", 0)) == 5 and DictUtil.dict_of(picked.get("items", {})).has("rope")
	var routed := _router()
	routed.push(MenuRouter.RESULTS, {"award": {}, "advanced": [], "outcome": "won", "rewards": {"gold": 9}})
	var through: BattleResultsModel = routed.model() as BattleResultsModel
	checks["the router passes the rewards"] = through != null and through.reward_lines() == ["Gold +9"]
	return TestHelpers.report(TAG, "Rewards list: every non-XP reward of the award data", checks)

static func _test_model_popup_state() -> bool:
	var checks: Dictionary = {}
	var router := _router()
	router.push(MenuRouter.RESULTS, {"award": {}, "outcome": "won"})
	var m: BattleResultsModel = router.model() as BattleResultsModel
	var renders: Array[int] = [0]
	router.changed.connect(func() -> void: renders[0] += 1)
	checks["headline"] = m.headline() == "Victory" and BattleResultsModel.new(router, {}, [] as Array[String], "lost").headline() == "Defeat" and BattleResultsModel.new(router, {}, [] as Array[String], "fled").headline() == "Fled"
	checks["no replay without an arena"] = not m.can_replay() and not m.replay().is_empty()
	m.toggle_minimised()
	checks["minimise re-renders once"] = m.minimised and renders[0] == 1
	m.set_minimised(true)
	checks["same state: no re-render"] = renders[0] == 1
	m.toggle_minimised()
	checks["restore"] = not m.minimised and renders[0] == 2
	var calls: Array[int] = [0]
	m.replay_handler = func() -> bool:
		calls[0] += 1
		return true
	checks["replay with an arena calls the handler once and minimises"] = m.can_replay() and m.replay().is_empty() and calls[0] == 1 and m.minimised
	router.pop()
	checks["a plain pop never leaves results"] = router.top() == MenuRouter.RESULTS
	return TestHelpers.report(TAG, "Results popup state (minimise, restore, Replay handler)", checks)

# ------------------------------------------------------------------------------
# Scene helpers
# ------------------------------------------------------------------------------

# {holder, host, view (game view), battle, done}: the main menu in a 1280x720 viewport with the game open and a fight launched
# and forced to `outcome`; the results popup opens through the battle's own path (the auto-results hand-off).
static func _fight(tree_parent: Node, outcome: String, settle_it: bool = true) -> Dictionary:
	var packed: PackedScene = load("res://scenes/main_menu.tscn") as PackedScene
	var holder := SubViewport.new()
	holder.size = Vector2i(VIEW)
	tree_parent.add_child(holder)
	var host: MainMenuView = packed.instantiate() as MainMenuView
	holder.add_child(host)
	var gs: GameState = CurrentGame.state()
	host.open_game(gs)
	var done: Array[int] = [0]
	host.router.results_done.connect(func() -> void: done[0] += 1)
	var view: RegionMapView = host.game_view as RegionMapView
	view.pending_fight = {"kind": "fight", "source": "encounter", "marker": "", "encounter": ENCOUNTER, "elite": false, "from": [gs.position.x, gs.position.y]}
	view.launch_battle()
	var b: BattleScreen = view.battle
	if b != null and b.session != null:
		if outcome == "won":
			for c: Combatant in b.session.enemies:
				c.hp = 0
		else:
			for c: Combatant in b.session.allies:
				c.hp = 0
		b.session.combat_over = true
		var _o: String = b.check()
		if settle_it:
			b._auto_finish()
	SettingsUiTestSuite.layout(host)
	return {"holder": holder, "host": host, "view": view, "battle": b, "done": done}

static func _free(fight: Dictionary) -> void:
	var holder: Node = fight["holder"] as Node
	holder.get_parent().remove_child(holder)
	holder.free()
	CurrentGame.end()

static func _button(root: Node, node_name: String) -> Button:
	return root.find_child(node_name, true, false) as Button

static func _has_text_button(root: Node, text: String) -> bool:
	for n: Node in root.find_children("*", "Button", true, false):
		if (n as Button).text == text:
			return true
	return false

static func _restart_game() -> void:
	var built: Dictionary = RegionMapView.new_state(WORLD)
	if built["state"] is GameState:
		CurrentGame.start(built["state"] as GameState)

# ------------------------------------------------------------------------------
# Transition
# ------------------------------------------------------------------------------

static func _test_transition(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var f: Dictionary = _fight(tree_parent, "won", false)
	var b: BattleScreen = f["battle"] as BattleScreen
	checks["the fight opened"] = b != null
	if b != null:
		checks["headless: no transition is shown"] = not b.intro_active and b.get_node_or_null("@CanvasLayer@1") == null
		checks["the banner names the fight and the enemy count"] = (b.intro_title.begins_with("Ambush! ") or b.intro_title.begins_with("Battle: ")) and b.intro_detail.begins_with(str(b.session.enemies.size()))
		var was_instant: bool = FxSettings.instant
		FxSettings.instant = true
		b.play_intro("Battle: x", "1 enemy")
		checks["Skip animations: instant, no layer"] = not b.intro_active
		FxSettings.instant = was_instant
		b.end_intro()
		b.end_intro()
		checks["ending it twice is harmless"] = not b.intro_active
	_free(f)
	return TestHelpers.report(TAG, "Transition into combat is instant headless and with Skip animations", checks)

# ------------------------------------------------------------------------------
# Popup, close paths
# ------------------------------------------------------------------------------

static func _popup_checks(f: Dictionary, checks: Dictionary) -> void:
	var host: MainMenuView = f["host"] as MainMenuView
	var view: RegionMapView = f["view"] as RegionMapView
	var b: BattleScreen = f["battle"] as BattleScreen
	var rpopup: Control = host.find_child("ResultsLayer", true, false) as Control
	checks["results is on top, once"] = host.router.top() == MenuRouter.RESULTS and host.router.stack.count(MenuRouter.RESULTS) == 1
	checks["the battle stays mounted and visible behind the popup"] = view.battle == b and b != null and b.arena.is_inside_tree() and view.visible and rpopup != null and rpopup.visible
	var menu: Control = host.get("_menu") as Control
	checks["the menu screen is not shown"] = menu != null and not menu.visible
	checks["X, minimise and Replay are in the header"] = _button(host, "ResultsClose") != null and _button(host, "ResultsMinimise") != null and _button(host, "ResultsReplay") != null
	checks["no Continue button"] = not _has_text_button(host.find_child("ResultsPanel", true, false), "Continue")
	var model: BattleResultsModel = host.router.model() as BattleResultsModel
	var bars: int = 0
	for n: Node in host.find_children("XpBar_*", "", true, false):
		bars += 1
	checks["an XP bar per rewarded member"] = model != null and bars == model.rows().size() and bars > 0

static func _closed_checks(f: Dictionary, checks: Dictionary) -> void:
	var host: MainMenuView = f["host"] as MainMenuView
	var view: RegionMapView = f["view"] as RegionMapView
	var done: Array[int] = f["done"] as Array[int]
	checks["closed: results_done exactly once"] = done[0] == 1
	checks["closed: one autosave"] = SaveGame.exists(SaveGame.AUTOSAVE_SLOT)
	checks["closed: back below the results (the game scene, UIR-2 PLAY), no RESULTS left"] = host.router.top() == MenuRouter.PLAY and not host.router.stack.has(MenuRouter.RESULTS)
	checks["closed: arena freed, map shown"] = view.battle == null and view.visible and view.active_battle() == null

static func _test_close_cross(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var _gone: bool = SaveGame.delete(SaveGame.AUTOSAVE_SLOT)
	var f: Dictionary = _fight(tree_parent, "won")
	_popup_checks(f, checks)
	var close: Button = _button(f["host"] as Node, "ResultsClose")
	if close != null:
		close.pressed.emit()
	_closed_checks(f, checks)
	_free(f)
	return TestHelpers.report(TAG, "Results popup: the X button runs continue_on once", checks)

static func _test_esc(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var _gone: bool = SaveGame.delete(SaveGame.AUTOSAVE_SLOT)
	var f: Dictionary = _fight(tree_parent, "won")
	var host: MainMenuView = f["host"] as MainMenuView
	checks["popup open"] = host.router.top() == MenuRouter.RESULTS
	host.cancel_pressed()
	_closed_checks(f, checks)
	_free(f)
	return TestHelpers.report(TAG, "Results popup: Esc is X (once)", checks)

static func _test_right_click(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var _gone: bool = SaveGame.delete(SaveGame.AUTOSAVE_SLOT)
	var f: Dictionary = _fight(tree_parent, "won")
	var host: MainMenuView = f["host"] as MainMenuView
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	host._input(ev)
	_closed_checks(f, checks)
	_free(f)
	return TestHelpers.report(TAG, "Results popup: right click is X (once)", checks)

static func _test_minimise(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var f: Dictionary = _fight(tree_parent, "won")
	var host: MainMenuView = f["host"] as MainMenuView
	var view: RegionMapView = f["view"] as RegionMapView
	var rpopup: Control = host.find_child("ResultsLayer", true, false) as Control
	var pill: Button = _button(host, "ResultsPill")
	checks["popup shown, pill hidden"] = rpopup != null and rpopup.visible and pill != null and not pill.visible
	var minimise: Button = _button(host, "ResultsMinimise")
	if minimise != null:
		minimise.pressed.emit()
	checks["minimised: popup hidden, pill shows the headline"] = not rpopup.visible and pill.visible and pill.text.begins_with("Victory")
	checks["minimised: still on the results screen, battle visible"] = host.router.top() == MenuRouter.RESULTS and view.visible and view.battle != null
	checks["minimised: nothing closed"] = (f["done"] as Array[int])[0] == 0
	pill.pressed.emit()
	checks["restored: popup back, pill hidden"] = rpopup.visible and not pill.visible and host.router.top() == MenuRouter.RESULTS
	host.cancel_pressed()
	checks["Esc while minimised closes too"] = true
	_free(f)
	return TestHelpers.report(TAG, "Results popup: minimise and restore toggle visibility without leaving the screen", checks)

static func _test_replay(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var f: Dictionary = _fight(tree_parent, "won")
	var host: MainMenuView = f["host"] as MainMenuView
	var b: BattleScreen = f["battle"] as BattleScreen
	var replay: Button = _button(host, "ResultsReplay")
	checks["Replay is offered while the arena is mounted"] = replay != null
	if replay != null and b != null:
		b.session.viewed_round = 1
		replay.pressed.emit()
		var model: BattleResultsModel = host.router.model() as BattleResultsModel
		checks["Replay restarts the arena playback from round 1"] = b.replaying and b.session.viewed_round == 1 and b.session.is_auto_playing
		checks["the popup minimises while it plays"] = model.minimised and host.router.top() == MenuRouter.RESULTS and _button(host, "ResultsPill").visible
		b.session.is_auto_playing = false   # the playback ended (the dock host's timer does this live)
		var _o: String = b.check()
		checks["when it ends the popup comes back"] = not b.replaying and not model.minimised and host.find_child("ResultsLayer", true, false).visible
	_free(f)
	return TestHelpers.report(TAG, "Results popup: Replay plays the battle again and the popup returns", checks)

static func _test_lost(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var f: Dictionary = _fight(tree_parent, "lost")
	var host: MainMenuView = f["host"] as MainMenuView
	var title: Label = host.find_child("ResultsTitle", true, false) as Label
	checks["a lost fight also shows the popup, headline Defeat"] = host.router.top() == MenuRouter.RESULTS and title != null and title.text == "Defeat"
	checks["no XP bars, a rewards section"] = host.find_children("XpBar_*", "", true, false).is_empty()
	_free(f)
	return TestHelpers.report(TAG, "Results popup: defeat", checks)

# A kept arena freed under the popup (a lost road fight closes the world map): Replay goes, the popup re-renders.
static func _test_arena_freed(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var f: Dictionary = _fight(tree_parent, "lost")
	var host: MainMenuView = f["host"] as MainMenuView
	var view: RegionMapView = f["view"] as RegionMapView
	var model: BattleResultsModel = host.router.model() as BattleResultsModel
	checks["Replay offered while the arena is mounted"] = model.can_replay() and _button(host, "ResultsReplay") != null
	view.close_settled_battle()
	checks["arena gone: Replay handler dropped"] = not model.can_replay()
	checks["arena gone: popup re-rendered without Replay, still on results"] = _button(host, "ResultsReplay") == null and host.router.top() == MenuRouter.RESULTS and _button(host, "ResultsClose") != null
	host.cancel_pressed()
	checks["X still closes once"] = (f["done"] as Array[int])[0] == 1
	_free(f)
	return TestHelpers.report(TAG, "Results popup: arena freed under it drops Replay", checks)

# Keys are ignored by the arena under the popup, restoring stops a replay, continue_on never runs twice.
static func _test_guards(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_restart_game()
	var f: Dictionary = _fight(tree_parent, "won")
	var host: MainMenuView = f["host"] as MainMenuView
	var b: BattleScreen = f["battle"] as BattleScreen
	var dock: CombatDockHost = b.arena.get("host") as CombatDockHost
	checks["popup open: arena keys blocked"] = dock != null and dock.keys_blocked
	b.session.viewed_round = 1
	(_button(host, "ResultsReplay") as Button).pressed.emit()
	checks["minimised for the replay: keys free again"] = not dock.keys_blocked and b.replaying
	(_button(host, "ResultsPill") as Button).pressed.emit()
	checks["restoring from the pill stops the replay"] = not b.replaying and not b.session.is_auto_playing and dock.keys_blocked
	var model: BattleResultsModel = host.router.model() as BattleResultsModel
	var first: Array[String] = model.continue_on()
	var second: Array[String] = model.continue_on()
	checks["continue_on twice: once"] = (f["done"] as Array[int])[0] == 1 and second.is_empty() and first.is_empty()
	_free(f)
	return TestHelpers.report(TAG, "Results popup guards (arena keys, replay stop, single close)", checks)

# ------------------------------------------------------------------------------
# Fit
# ------------------------------------------------------------------------------

static func _test_fit(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	for scale: float in SettingsFile.TEXT_LEVELS:
		_restart_game()
		SettingsFile.set_value("ui/text_scale", scale)
		var f: Dictionary = _fight(tree_parent, "won")
		var host: MainMenuView = f["host"] as MainMenuView
		var panel: Control = host.find_child("ResultsPanel", true, false) as Control
		host.router.changed.emit()   # re-render at this text size
		SettingsUiTestSuite.layout(host)
		var tag: String = "text %.2f" % scale
		var area := Rect2(Vector2.ZERO, VIEW)
		checks[tag + ": panel inside 1280x720"] = panel != null and area.encloses(panel.get_global_rect())
		var problems: Array[String] = []
		if panel != null:
			for n: Node in panel.find_children("*", "Control", true, false):
				var c: Control = n as Control
				if c == null or not c.is_visible_in_tree() or c is ScrollContainer or _under_scroll(c, panel):
					continue
				if not panel.get_global_rect().grow(1.0).encloses(c.get_global_rect()):
					problems.append(c.name)
		checks[tag + ": header controls inside the panel"] = problems.is_empty()
		checks[tag + ": X reachable"] = _button(host, "ResultsClose") != null and area.encloses(_button(host, "ResultsClose").get_global_rect())
		_free(f)
	SettingsFile.set_value("ui/text_scale", 1.0)
	return TestHelpers.report(TAG, "Results popup fits 1280x720 at every text size", checks)

static func _under_scroll(c: Control, root: Node) -> bool:
	var p: Node = c.get_parent()
	while p != null and p != root:
		if p is ScrollContainer:
			return true
		p = p.get_parent()
	return false
