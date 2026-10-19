class_name BattleScreen
extends Node

# ==============================================================================
# Tactical RPG - Battle screen (BATTLE1-L): plays a map, dialogue or road fight in the real combat arena.
# Thin: builds the saved party's GameEncounter through BattleBridge.encounter_for, mounts scenes/combat_arena.tscn
# with it, re-reads BattleBridge.outcome on every session change and shows an end banner over the arena while the
# battle has an outcome (rewinding a round hides it again). Continue emits `finished(outcome)`; the owner settles the
# fight (ExploreRunner / DialogueRunner.after_fight) and calls close(). Combat files stay untouched.
# BATTLE-FLOW: play_intro() is a short skippable fade-in with a banner ("Ambush!" / "Battle:" + name, enemy count), instant
# under FxSettings.instant (Skip animations) and headless. With `auto_results` (set by the map views under the main menu
# host) an outcome settles itself after a beat instead of waiting for Continue, and the host shows the results popup over
# the still-mounted arena. replay() plays the finished battle again from round 1 (replay_finished when it ends);
# set_shown() hides or shows the arena layers while a menu screen covers the game.
# BREPORT1-L: a BattleMetrics follows the session from the mount (round checkpoints undo rewound rounds); report() builds
# the results popup's battle report once the outcome is known, then freezes the collector (Replay cannot change it).
# ==============================================================================

signal finished(outcome: String)
signal replay_finished()

const ARENA_SCENE: String = "res://scenes/combat_arena.tscn"
const BANNER_LAYER: int = 20
const HEADLINE_FONT: int = 20
const INTRO_LAYER: int = 25
const INTRO_FADE_IN: float = 0.25   # seconds at speed 1 (FxSettings.scaled); the whole intro is about 1.3 s
const INTRO_HOLD: float = 0.7
const INTRO_FADE_OUT: float = 0.4
const AUTO_RESULTS_DELAY: float = 1.0   # the last blow stays visible this long before the results popup
const HEADLINES: Dictionary = {
	"won": "Victory",
	"lost": "Defeat: the party wakes at the last safe place, fully healed",
	"fled": "The party withdrew",
}

var fight: Dictionary = {}
var arena: Node = null
var session: CombatSession = null
var outcome: String = ""
var auto_results: bool = false     # an outcome settles by itself (the results popup replaces the Continue banner)
var replaying: bool = false
var intro_active: bool = false
var intro_title: String = ""
var intro_detail: String = ""

var _banner_layer: CanvasLayer = null
var _banner: PanelContainer = null
var _banner_label: Label = null
var _intro_layer: CanvasLayer = null
var _intro_tween: Tween = null
var _intro_root: Control = null
var _arming: bool = false          # replay() is stepping the session; check() must not react
var _settled: bool = false         # finished was emitted by the auto path
var _auto_pending: bool = false
var _metrics: BattleMetrics = null
var _report: Dictionary = {}       # built by the first report() call with an outcome

# {ok, errors, screen: BattleScreen or null}: the battle for `p_fight` ({encounter, zone?, ...} from a fight event) added
# under `parent` and started.
static func open(parent: Node, gs: GameState, p_fight: Dictionary) -> Dictionary:
	var errors: Array[String] = []
	var built: Dictionary = BattleBridge.encounter_for(gs, str(p_fight.get("encounter", "")), str(p_fight.get("zone", "")))   # road fights carry the route zone (TRAV1)
	if not DictUtil.to_bool(built["ok"]):
		errors.assign(DictUtil.array_of(built["errors"]))
		return {"ok": false, "errors": errors, "screen": null}
	var enc: CombatEncounter = built["encounter"] as CombatEncounter
	var packed: PackedScene = load(ARENA_SCENE) as PackedScene
	if enc == null or packed == null:
		errors.append("The combat arena could not be loaded.")
		return {"ok": false, "errors": errors, "screen": null}
	var screen := BattleScreen.new()
	screen.fight = p_fight
	parent.add_child(screen)
	screen._mount(packed.instantiate(), enc, gs)
	if screen.session == null:
		parent.remove_child(screen)
		screen.queue_free()
		errors.append("The combat arena did not start a battle.")
		return {"ok": false, "errors": errors, "screen": null}
	MusicDirector.request(screen, MusicDirector.combat_context(enc), MusicDirector.COMBAT, gs.world_id, gs.location_id)   # MUSIC1
	var info: Dictionary = WorldCatalog.resolve(gs.world_id, "encounters", str(p_fight.get("encounter", "")))
	var label: String = str(info.get("name", str(p_fight.get("encounter", "")).replace("_", " ").capitalize()))
	var ambush: bool = str(p_fight.get("marker", "")).is_empty() and str(p_fight.get("source", "")) != "dialogue"
	screen.play_intro(("Ambush! " if ambush else "Battle: ") + label, "%d %s" % [screen.session.enemies.size(), "enemy" if screen.session.enemies.size() == 1 else "enemies"])
	return {"ok": true, "errors": errors, "screen": screen}

func _mount(node: Node, enc: CombatEncounter, store: GameState = null) -> void:
	arena = node
	arena.set("autoplay_store", store)   # the save that remembers the tactics and the Autoplay switch
	arena.set("encounter", enc)   # read by the arena's _ready, before it builds the session
	add_child(arena)
	session = arena.get("session") as CombatSession
	_build_banner()
	if session != null:
		_metrics = BattleMetrics.new()
		_metrics.quiet = true   # live self-check failures are kept in fail_lines, not printed
		_metrics.attach(session)
		session.changed.connect(check)
	check()

func _build_banner() -> void:
	_banner_layer = CanvasLayer.new()
	_banner_layer.layer = BANNER_LAYER
	add_child(_banner_layer)
	_banner = PanelContainer.new()
	_banner.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_TITLE, 6, 24, 18, 2))
	_banner.set_anchors_preset(Control.PRESET_CENTER_TOP)
	_banner.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_banner.offset_top = 64
	_banner.visible = false
	_banner_layer.add_child(_banner)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	_banner.add_child(box)
	_banner_label = UiTheme.label("", HEADLINE_FONT, UiTheme.COL_TITLE)
	box.add_child(_banner_label)
	var hint: Label = UiTheme.label("Review the replay, or continue to the results.", UiTheme.FONT_LG, UiTheme.COL_MUTED)
	box.add_child(hint)
	box.add_child(UiTheme.button("Continue", "Leave the battle and apply its outcome (XP, quests, defeat or retreat)", finish))

# Re-reads the outcome (won / lost / fled, "" while the battle goes on) and shows or hides the end banner.
func check() -> String:
	outcome = BattleBridge.outcome(session) if session != null else ""
	if _banner != null:
		_banner.visible = not outcome.is_empty() and not auto_results
		_banner_label.text = str(HEADLINES.get(outcome, ""))
	if session != null and not _arming:
		_follow_replay()
		if auto_results and not outcome.is_empty() and not replaying and not _settled and not _auto_pending:
			_schedule_auto_finish()
	return outcome

func _schedule_auto_finish() -> void:
	_auto_pending = true
	if FxSettings.instant or not is_inside_tree():
		_auto_finish.call_deferred()
	else:
		get_tree().create_timer(FxSettings.scaled(AUTO_RESULTS_DELAY)).timeout.connect(_auto_finish)

func _auto_finish() -> void:
	_auto_pending = false
	if _settled or session == null or replaying or check().is_empty():
		return
	_settled = true
	_write_back()
	finished.emit(outcome)

# ------------------------------------------------------------------------------
# Replay (BATTLE-FLOW): the arena's own round playback, round 1 to the last, then replay_finished
# ------------------------------------------------------------------------------

# The battle report (BattleReport.build): {} while the battle has no outcome or no session; the first call with an
# outcome closes and freezes the collector and caches the dict, later calls return the same report.
func report() -> Dictionary:
	if not _report.is_empty():
		return _report
	if session == null or _metrics == null:
		return {}
	var result: String = BattleBridge.outcome(session)
	if result.is_empty():
		return {}
	_metrics.finish()
	_report = BattleReport.build(_metrics, session, result)
	return _report

# Starts the playback from round 1. False when there is no session.
func replay() -> bool:
	if session == null:
		return false
	replaying = true
	_arming = true
	session.select_round(1)
	if not session.is_auto_playing:
		session.toggle_auto_play()
	_arming = false
	return true

# Ends a replay in progress (the playback stops where it is).
func stop_replay() -> void:
	if not replaying:
		return
	replaying = false
	if session != null and session.is_auto_playing:
		_arming = true
		session.toggle_auto_play()
		_arming = false

# While the results popup covers the arena its keyboard shortcuts are ignored.
func set_keys_blocked(blocked: bool) -> void:
	if arena == null:
		return
	var found: Variant = arena.get("host")
	if found is CombatDockHost:
		found.keys_blocked = blocked

# After a round finished playing, the next recorded round starts; after the last one the replay is over.
func _follow_replay() -> void:
	if not replaying or session.is_auto_playing:
		return
	_arming = true
	if session.viewed_round < session.max_unlocked_round:
		session.select_round(session.viewed_round + 1)
		session.toggle_auto_play()
		_arming = false
	else:
		_arming = false
		replaying = false
		replay_finished.emit()

# Shows or hides the arena and this screen's layers (a menu screen covers the game while the battle stays mounted).
func set_shown(shown: bool) -> void:
	var arena_item: CanvasItem = arena as CanvasItem
	if arena_item != null:
		arena_item.visible = shown
	if arena != null:
		for n: Node in arena.get_children():
			var layer: CanvasLayer = n as CanvasLayer
			if layer != null:
				layer.visible = shown
	if _banner_layer != null:
		_banner_layer.visible = shown
	if _intro_layer != null:
		_intro_layer.visible = shown

# ------------------------------------------------------------------------------
# Transition into the battle (BATTLE-FLOW)
# ------------------------------------------------------------------------------

# A fade-in from black with the banner text; any click, Enter or Esc skips it. Instant (no layer, no tween) under
# FxSettings.instant and headless, so tests never wait.
func play_intro(title: String, detail: String) -> void:
	intro_title = title
	intro_detail = detail
	if FxSettings.instant or DisplayServer.get_name() == "headless" or not is_inside_tree():
		return
	intro_active = true
	_intro_layer = CanvasLayer.new()
	_intro_layer.layer = INTRO_LAYER
	add_child(_intro_layer)
	var root := Control.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.mouse_filter = Control.MOUSE_FILTER_STOP
	_intro_layer.add_child(root)
	_intro_root = root
	var shade := ColorRect.new()
	shade.color = Color.BLACK
	shade.set_anchors_preset(Control.PRESET_FULL_RECT)
	shade.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(shade)
	var center := CenterContainer.new()
	center.set_anchors_preset(Control.PRESET_FULL_RECT)
	center.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(center)
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 10)
	box.modulate.a = 0.0
	center.add_child(box)
	var head: Label = UiTheme.label(title, 44, UiTheme.COL_TITLE)
	head.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(head)
	var sub: Label = UiTheme.label(detail, 22, UiTheme.COL_TEXT)
	sub.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(sub)
	var hint: Label = UiTheme.label("Click to skip", 14, UiTheme.COL_MUTED)
	hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	box.add_child(hint)
	_intro_tween = create_tween()
	var _a: PropertyTweener = _intro_tween.tween_property(box, "modulate:a", 1.0, FxSettings.scaled(INTRO_FADE_IN))
	var _b: IntervalTweener = _intro_tween.tween_interval(FxSettings.scaled(INTRO_HOLD))
	var _c: PropertyTweener = _intro_tween.tween_property(_intro_root, "modulate:a", 0.0, FxSettings.scaled(INTRO_FADE_OUT))
	var _d: CallbackTweener = _intro_tween.tween_callback(end_intro)

# Ends the transition at once (also the tween's last step).
func end_intro() -> void:
	if _intro_tween != null and _intro_tween.is_valid():
		_intro_tween.kill()
	_intro_tween = null
	if _intro_layer != null:
		remove_child(_intro_layer)
		_intro_layer.queue_free()
		_intro_layer = null
		_intro_root = null
	intro_active = false

func _input(event: InputEvent) -> void:
	if not intro_active:
		return
	var mouse: InputEventMouseButton = event as InputEventMouseButton
	var skip: bool = mouse != null and mouse.pressed
	if event is InputEventKey and event.is_pressed() and (event.is_action("ui_accept") or event.is_action("ui_cancel")):
		skip = true
	if skip:
		end_intro()
		get_viewport().set_input_as_handled()

# Continue: emits `finished` with the current outcome (ignored while the battle has none).
func finish() -> void:
	if check().is_empty():
		return
	_settled = true
	_write_back()
	finished.emit(outcome)

# ITEMUSE1 rule 7: the heroes' unused pouch items go back to their saved pouches (every outcome; guests never).
# HEAL1 rule 3.4: then the heroes' HP (won / fled; a loss is healed by ExploreRunner.wake).
func _write_back() -> void:
	if session != null and session.encounter is GameEncounter:
		var game_enc: GameEncounter = session.encounter as GameEncounter
		game_enc.write_back_pouches()
		game_enc.write_back_hp(outcome)

# Frees the arena (its _exit_tree disposes the session) and this screen.
func close() -> void:
	if session != null and session.changed.is_connected(check):
		session.changed.disconnect(check)
	if _metrics != null:
		_metrics.freeze()
	session = null
	var parent: Node = get_parent()
	if parent != null:
		parent.remove_child(self)
	queue_free()
