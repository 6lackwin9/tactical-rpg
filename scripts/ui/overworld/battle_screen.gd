class_name BattleScreen
extends Node

# ==============================================================================
# Tactical RPG - Battle screen (BATTLE1-L): plays a map, dialogue or road fight in the real combat arena.
# Thin: builds the saved party's GameEncounter through BattleBridge.encounter_for, mounts scenes/combat_arena.tscn
# with it, re-reads BattleBridge.outcome on every session change and shows an end banner over the arena while the
# battle has an outcome (rewinding a round hides it again). Continue emits `finished(outcome)`; the owner settles the
# fight (ExploreRunner / DialogueRunner.after_fight) and calls close(). Combat files stay untouched.
# ==============================================================================

signal finished(outcome: String)

const ARENA_SCENE: String = "res://scenes/combat_arena.tscn"
const BANNER_LAYER: int = 20
const HEADLINE_FONT: int = 20
const HEADLINES: Dictionary = {
	"won": "Victory",
	"lost": "Defeat: the party wakes at the last safe place, fully healed",
	"fled": "The party withdrew",
}

var fight: Dictionary = {}
var arena: Node = null
var session: CombatSession = null
var outcome: String = ""

var _banner_layer: CanvasLayer = null
var _banner: PanelContainer = null
var _banner_label: Label = null

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
	screen._mount(packed.instantiate(), enc)
	if screen.session == null:
		parent.remove_child(screen)
		screen.queue_free()
		errors.append("The combat arena did not start a battle.")
		return {"ok": false, "errors": errors, "screen": null}
	return {"ok": true, "errors": errors, "screen": screen}

func _mount(node: Node, enc: CombatEncounter) -> void:
	arena = node
	arena.set("encounter", enc)   # read by the arena's _ready, before it builds the session
	add_child(arena)
	session = arena.get("session") as CombatSession
	_build_banner()
	if session != null:
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
		_banner.visible = not outcome.is_empty()
		_banner_label.text = str(HEADLINES.get(outcome, ""))
	return outcome

# Continue: emits `finished` with the current outcome (ignored while the battle has none).
func finish() -> void:
	if check().is_empty():
		return
	finished.emit(outcome)

# Frees the arena (its _exit_tree disposes the session) and this screen.
func close() -> void:
	if session != null and session.changed.is_connected(check):
		session.changed.disconnect(check)
	session = null
	var parent: Node = get_parent()
	if parent != null:
		parent.remove_child(self)
	queue_free()
