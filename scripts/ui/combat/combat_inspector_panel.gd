class_name CombatInspectorPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Unit Inspector Panel
# Portrait, vitals, stance, target, buffs and the command palette (allies) or
# threat intel (enemies) for the selected unit. The palette arms commands in the
# session so they can be placed on the timeline during planning. The 🎒 GEAR
# button asks the host for the Party & Stash screen.
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
	inspector.command_armed.connect(func(kind: int, dir: Vector2i) -> void: session.arm(kind, dir))
	inspector.auto_plan_requested.connect(func() -> void: session.plan_auto(session.focused_hero))
	inspector.clear_plan_requested.connect(func() -> void: session.plan_clear(session.focused_hero))

func refresh() -> void:
	if session == null:
		return
	var hero: Combatant = session.focused_hero
	var ap_left: int = session.plan.ap_left(hero) if hero != null else 0
	inspector.update_planning(session.is_planning(), session.armed_kind, session.armed_dir, ap_left, CombatPlan.ROUND_AP)
	var c: Combatant = session.selected_combatant
	if c == null or not is_instance_valid(c):
		return
	inspector.update_inspector(c, session.is_ally(c), session.position_text(c), session.encounter.get_sprite_texture(c), session.display_snapshot(), _threat_intel(c))

# What the AI is known to do (role, strike power, flee threshold) for the selected hostile.
func _threat_intel(c: Combatant) -> Array[Dictionary]:
	var intel: Array[Dictionary] = []
	if session.is_ally(c):
		return intel
	var profile: AiProfile = session.encounter.profile_of(c)
	var role_desc: String = ""
	match profile.role:
		AiProfile.Role.AGGRESSOR:
			role_desc = "Closes in on the nearest foe and trades blows"
		AiProfile.Role.SKIRMISHER:
			role_desc = "Hunts the weakest foe; backs off when hurt"
		AiProfile.Role.SNIPER:
			role_desc = "Holds maximum range; retreats from melee"
		AiProfile.Role.SUPPORT:
			role_desc = "Opens with Focus, then fights from range"
	intel.append({"name": "🧠 %s" % AiProfile.role_name(profile.role), "desc": role_desc, "color": Color(0.9, 0.6, 0.3)})
	intel.append({"name": "⚔ Strike %d power" % profile.power, "desc": "Range %d tile(s)" % c.attack_range, "color": Color(0.9, 0.35, 0.35)})
	if profile.defend_below > 0.0:
		intel.append({"name": "🛡 Guards at ≤%d%% HP" % int(profile.defend_below * 100.0), "desc": "Raises a 50% guard once per round", "color": Color(0.35, 0.65, 0.95)})
	if profile.flee_ratio > 0.0:
		intel.append({"name": "🏃 Flees at ≤%d%% HP" % int(profile.flee_ratio * 100.0), "desc": "Runs for the exit tile", "color": Color(0.3, 0.85, 0.5)})
	return intel