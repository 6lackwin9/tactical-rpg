class_name AiProfile
extends RefCounted

# ==============================================================================
# Tactical RPG - AI Profile (how a unit plans its rounds)
# Role picks the behaviour pattern; the other fields tune it. The same profile
# drives enemies every round and the player's "AI preset" sequences (see
# PlanPresets). `skill_cap` is the highest skill level it may use (a stand-in
# for progression). Damage comes from the unit itself (DamageCalculator).
# ==============================================================================

enum Role {
	AGGRESSOR,   # closes distance and trades blows
	SKIRMISHER,  # hunts the weakest foe, falls back when hurt
	SNIPER,      # holds maximum range, backs away from melee
	SUPPORT,     # opens with Focus, then fights like a sniper
}

enum TargetPref { NEAREST, WEAKEST }

var role: Role = Role.AGGRESSOR
var skill_cap: int = 1
var flee_ratio: float = 0.25    # withdraw at or below this HP fraction (0 = never)
var defend_below: float = 0.4   # guard once when at or below this HP fraction
var target_pref: TargetPref = TargetPref.NEAREST
var prefer_level: int = 0       # skill level to favour (0 = highest affordable)
var reckless: bool = false      # never flees or guards, always charges the nearest foe (berserker behaviour)
var heal_below: float = 0.7     # SUPPORT role: heal the most wounded ally at or below this HP fraction

static func create(p_role: Role, p_flee_ratio: float = 0.25, p_skill_cap: int = 1) -> AiProfile:
	var profile := AiProfile.new()
	profile.role = p_role
	profile.flee_ratio = p_flee_ratio
	profile.skill_cap = p_skill_cap
	profile.target_pref = TargetPref.NEAREST if p_role == Role.AGGRESSOR else TargetPref.WEAKEST
	return profile

func duplicate_profile() -> AiProfile:
	var copy := AiProfile.new()
	copy.role = role
	copy.skill_cap = skill_cap
	copy.flee_ratio = flee_ratio
	copy.defend_below = defend_below
	copy.target_pref = target_pref
	copy.prefer_level = prefer_level
	copy.reckless = reckless
	copy.heal_below = heal_below
	return copy

static func role_name(p_role: Role) -> String:
	match p_role:
		Role.AGGRESSOR:
			return "Aggressor"
		Role.SKIRMISHER:
			return "Skirmisher"
		Role.SNIPER:
			return "Sniper"
	return "Support"
