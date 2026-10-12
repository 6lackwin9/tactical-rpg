class_name CombatInspectorPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Unit Inspector Panel
# Portrait, vitals, stance, target, buffs and the action palette (allies) or
# threat intel (enemies) for the selected unit. The 🎒 GEAR button asks the host
# for the Party & Stash screen.
# ==============================================================================

var inspector: UnitInspectorUI

func _init() -> void:
	super._init()
	panel_id = "inspector"
	panel_title = "Unit Inspector"
	panel_icon = "🧍"
	default_min_size = Vector2(340, 468)
	inspector = UnitInspectorUI.new()
	add_child(inspector)

func _on_bound() -> void:
	inspector.gear_modal_requested.connect(func(_c: Combatant) -> void: modal_requested.emit("gear"))

func refresh() -> void:
	if session == null:
		return
	var c: Combatant = session.selected_combatant
	if c == null or not is_instance_valid(c):
		return
	inspector.update_inspector(c, session.is_ally(c), session.position_text(c), session.encounter.get_sprite_texture(c), session.display_snapshot())
