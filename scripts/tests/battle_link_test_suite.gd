class_name BattleLinkTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle link tests (BATTLE1-L): a map or dialogue fight in the region map scene opens the real combat
# arena with the saved party (BattleScreen over BattleBridge.encounter_for), the session's outcome (won / lost / fled)
# settles through ExploreRunner / DialogueRunner.after_fight, the arena closes and fight_settled carries the outcome.
# Shipped Ashlands data, read only; saves go to a sandbox. Outcomes are forced on the live session (HP, combat_over,
# round limit) so the test does not depend on battle balance.
# ==============================================================================

const SCENE: String = "res://scenes/overworld/region_map_debug.tscn"
const WORLD: String = "ashlands"
const ENCOUNTER: String = "ash_scouts"
static var SANDBOX: String = TestHelpers.sandbox("battle_link")

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING BATTLE LINK TEST (BATTLE1-L)            ===")
	print("=======================================================")
	var packed: PackedScene = load(SCENE) as PackedScene
	if packed == null or tree_parent == null:
		return TestHelpers.report_ok("BATTLE1", "Region map scene loads (headless without a tree: load only)", packed != null)
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var view: RegionMapView = packed.instantiate() as RegionMapView
	tree_parent.add_child(view)
	var ok: bool = view.start_game(WORLD).is_empty() and view.gs != null
	ok = TestHelpers.report_ok("BATTLE1", "New Ashlands game in the region map scene", ok) and ok
	if view.gs != null:
		ok = _test_won(view) and ok
		ok = _test_lost(view) and ok
		ok = _test_fled(view) and ok
		ok = _test_dialogue(view) and ok
		ok = _test_refused(view) and ok
		ok = _test_results(view.gs) and ok
	tree_parent.remove_child(view)
	view.free()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

static func _fight(view: RegionMapView, source: String = "encounter", encounter: String = ENCOUNTER) -> void:
	view.pending_fight = {"kind": "fight", "source": source, "encounter": encounter, "marker": "", "elite": false,
		"from": [view.gs.position.x, view.gs.position.y]}

static func _xp(gs: GameState) -> int:
	var total: int = 0
	for m: UnitState in gs.party.active_members():
		total += m.total_xp
	return total

static func _test_won(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var settled: Array[Dictionary] = []
	var on_settled: Callable = func(res: Dictionary) -> void: settled.append(res)
	view.fight_settled.connect(on_settled)
	_fight(view)
	view.launch_battle()
	var b: BattleScreen = view.battle
	checks["the fight opens the arena"] = b != null and b.session != null and b.arena != null and b.arena.is_inside_tree()
	if b == null or b.session == null:
		view.fight_settled.disconnect(on_settled)
		return TestHelpers.report("BATTLE1", "Won battle", checks)
	checks["the arena fields the active party"] = b.session.allies.size() == view.gs.party.active_members().size() and not b.session.enemies.is_empty()
	checks["the map is hidden and busy while the battle runs"] = view.busy() and not view._canvas.visible and not view._fight_panel.visible
	checks["no outcome and no banner at the start"] = b.check().is_empty() and not b._banner.visible
	b.finish()
	checks["Continue is ignored before an outcome"] = view.battle == b and not view.pending_fight.is_empty()
	for c: Combatant in b.session.enemies:
		c.hp = 0
	b.session.combat_over = true
	checks["every enemy down is won, with the banner"] = b.check() == "won" and b._banner.visible
	var xp0: int = _xp(view.gs)
	b.finish()
	checks["Continue settles the fight and closes the arena"] = view.pending_fight.is_empty() and view.battle == null and not view.busy() and view._canvas.visible
	checks["won awards XP to the party"] = _xp(view.gs) > xp0
	checks["fight_settled carries the outcome and XP"] = settled.size() == 1 and str(settled[0].get("outcome", "")) == "won" and not DictUtil.dict_of(settled[0].get("xp", {})).is_empty()
	view.fight_settled.disconnect(on_settled)
	return TestHelpers.report("BATTLE1", "Won battle (arena with the party, banner, XP, fight_settled)", checks)

static func _test_lost(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	_fight(view)
	view.launch_battle()
	var b: BattleScreen = view.battle
	checks["the fight opens the arena"] = b != null and b.session != null
	if b == null or b.session == null:
		return TestHelpers.report("BATTLE1", "Lost battle", checks)
	for c: Combatant in b.session.allies:
		c.hp = 0
	b.session.combat_over = true
	checks["every party member down is lost"] = b.check() == "lost"
	var xp0: int = _xp(view.gs)
	b.finish()
	checks["lost settles (wake) without XP"] = view.pending_fight.is_empty() and view.battle == null and _xp(view.gs) == xp0 and not view.gs.location_id.is_empty()
	return TestHelpers.report("BATTLE1", "Lost battle (wake at a safe place, no XP)", checks)

static func _test_fled(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var back_to: Vector2i = view.gs.position
	_fight(view)
	view.gs.position = back_to + Vector2i(1, 0)   # where the fight was triggered
	view.launch_battle()
	var b: BattleScreen = view.battle
	checks["the fight opens the arena"] = b != null and b.session != null
	if b == null or b.session == null:
		return TestHelpers.report("BATTLE1", "Fled battle", checks)
	b.session.phase = CombatSession.Phase.REPLAY
	b.session.current_round = b.session.encounter.max_rounds
	checks["the round limit with both sides standing is fled"] = b.check() == "fled"
	b.finish()
	checks["fled steps the party back"] = view.pending_fight.is_empty() and view.battle == null and view.gs.position == back_to
	return TestHelpers.report("BATTLE1", "Fled battle (round limit, back to the tile before the fight)", checks)

static func _test_dialogue(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	_fight(view, "dialogue", "pass_ambush")
	view.pending_fight["character"] = "test"
	view.launch_battle()
	var b: BattleScreen = view.battle
	checks["a dialogue fight opens the arena"] = b != null and b.session != null
	if b == null or b.session == null:
		return TestHelpers.report("BATTLE1", "Dialogue fight", checks)
	for c: Combatant in b.session.enemies:
		c.hp = 0
	b.session.combat_over = true
	var xp0: int = _xp(view.gs)
	b.finish()
	checks["won settles through DialogueRunner.after_fight"] = view.pending_fight.is_empty() and view.battle == null and _xp(view.gs) > xp0
	return TestHelpers.report("BATTLE1", "Dialogue fight (same arena, settles with XP)", checks)

# The results screen gets the outcome through MenuRouter (BattleResultsModel headline and note).
static func _test_results(gs: GameState) -> bool:
	var checks: Dictionary = {}
	CurrentGame.start(gs)
	var router := MenuRouter.new(MenuRouter.HUB)
	router.push(MenuRouter.RESULTS, {"award": {}, "advanced": [], "outcome": "lost"})
	var lost: BattleResultsModel = router.model() as BattleResultsModel
	checks["the router passes the outcome"] = lost != null and lost.outcome == "lost"
	checks["lost: Defeat headline and a wake note"] = lost != null and lost.headline().contains("Defeat") and not lost.outcome_note().is_empty()
	var won := BattleResultsModel.new(router, {})
	checks["won by default, with no note"] = won.outcome == "won" and won.headline().contains("Victory") and won.outcome_note().is_empty()
	CurrentGame.end()
	return TestHelpers.report("BATTLE1", "Results screen outcome (router args, headline, note)", checks)

static func _test_refused(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	_fight(view, "encounter", "no_such_encounter")
	view.launch_battle()
	checks["an unknown encounter opens no arena"] = view.battle == null and not view.launch_errors.is_empty()
	checks["the fight panel keeps Start battle and the debug outcomes"] = view._fight_panel.visible and view._dev_box.visible
	view.settle("fled")
	checks["a debug outcome still settles it"] = view.pending_fight.is_empty() and view.launch_errors.is_empty() and not view._fight_panel.visible
	return TestHelpers.report("BATTLE1", "Refused battle (reasons, Start battle, debug outcomes)", checks)
