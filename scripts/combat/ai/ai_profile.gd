class_name AiProfile
extends RefCounted

# ==============================================================================
# Tactical RPG - AI Profile (how a unit plans its rounds)
# Role picks the behaviour pattern; power is the strike strength used for the
# unit's Attack commands (the player's allies use it too for their plans).
# ==============================================================================

enum Role {
	AGGRESSOR,   # closes distance and trades blows
	SKIRMISHER,  # hunts the weakest foe, falls back when hurt
	SNIPER,      # holds maximum range, backs away from melee
	SUPPORT,     # opens with Focus, then fights like a sniper
}

var role: Role = Role.AGGRESSOR
var power: int = 20
var flee_ratio: float = 0.25    # flee to an exit at or below this HP fraction (0 = never)
var defend_below: float = 0.4   # guard once when at or below this HP fraction

static func create(p_role: Role, p_power: int, p_flee_ratio: float = 0.25) -> AiProfile:
	var profile := AiProfile.new()
	profile.role = p_role
	profile.power = p_power
	profile.flee_ratio = p_flee_ratio
	return profile

static func role_name(p_role: Role) -> String:
	match p_role:
		Role.AGGRESSOR:
			return "Aggressor"
		Role.SKIRMISHER:
			return "Skirmisher"
		Role.SNIPER:
			return "Sniper"
	return "Support"
