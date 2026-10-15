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
var smart_skills: bool = true   # AI1: use disengage, area, cleanse, taunt/haste, debuff and riposte skills (false = the pre-AI1 behaviour, for A/B runs)
var area_min_foes: int = 2      # AI1: an area attack is chosen only if it hits at least this many foes (clamped to 1..6 on use)

const AREA_MIN_FOES_MIN: int = 1
const AREA_MIN_FOES_MAX: int = 6

# area_min_foes as the AI uses it.
func area_threshold() -> int:
	return clampi(area_min_foes, AREA_MIN_FOES_MIN, AREA_MIN_FOES_MAX)

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
	copy.smart_skills = smart_skills
	copy.area_min_foes = area_min_foes
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
