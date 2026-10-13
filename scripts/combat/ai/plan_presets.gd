class_name PlanPresets
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Presets (one-click AI-drafted sequences)
# Each preset tweaks the unit's AiProfile with a different focus, then TacticalAI
# drafts a valid sequence from it. The player can edit the draft before
# confirming. Reduces tedium without taking control away.
# ==============================================================================

const IDS: Array[String] = ["balanced", "aggressive", "focus_weakest", "defensive", "ranged_kite", "economy", "withdraw"]

static func label(id: String) -> String:
	match id:
		"balanced":
			return "⚖ Balanced"
		"aggressive":
			return "🔥 All-out Attack"
		"focus_weakest":
			return "🎯 Focus Weakest"
		"defensive":
			return "🛡 Guard & Counter"
		"ranged_kite":
			return "🏹 Kite at Range"
		"economy":
			return "💰 Economy (low-tier skills)"
		"withdraw":
			return "🏃 Fall Back"
	return id

static func description(id: String) -> String:
	match id:
		"balanced":
			return "The unit's own role: fights, guards when hurt, flees when nearly down."
		"aggressive":
			return "Nearest enemy, strongest affordable skills, no guarding or fleeing."
		"focus_weakest":
			return "Goes for the enemy that is closest to falling."
		"defensive":
			return "Guards first, then trades blows."
		"ranged_kite":
			return "Holds maximum range and backs away from melee."
		"economy":
			return "Uses level 1 skills to stretch AP over more actions."
		"withdraw":
			return "Heads for the exit and leaves the battle."
	return ""

# The unit's profile adjusted for `id`.
static func profile_for(id: String, base: AiProfile) -> AiProfile:
	var p: AiProfile = base.duplicate_profile()
	match id:
		"aggressive":
			p.role = AiProfile.Role.AGGRESSOR
			p.target_pref = AiProfile.TargetPref.NEAREST
			p.defend_below = 0.0
			p.flee_ratio = 0.0
		"focus_weakest":
			p.role = AiProfile.Role.AGGRESSOR
			p.target_pref = AiProfile.TargetPref.WEAKEST
			p.defend_below = 0.25
		"defensive":
			p.defend_below = 1.0
		"ranged_kite":
			p.role = AiProfile.Role.SNIPER
			p.target_pref = AiProfile.TargetPref.WEAKEST
		"economy":
			p.prefer_level = 1
		"withdraw":
			p.flee_ratio = 1.0
	return p
