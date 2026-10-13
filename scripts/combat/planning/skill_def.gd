class_name SkillDef
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Definition (one tier of a command family)
# Leveled skills (level 1-5) cost AP equal to their level: a level 2 Attack or
# Defend always costs 2 AP unless an effect shifts it (Combatant.ap_cost_modifier).
# Move is the exception: 1 AP per tile covered (1-2 blocks, once per round). Rest
# is free and recovers AP. Switch Target is a free 1-block step inserted
# automatically when an attack changes the current target lock.
#
# Stances (rock-paper-scissors: Offensive > Tactical > Defensive > Offensive):
#   Attack -> Offensive, Defend -> Defensive, Focus -> Tactical.
#   Move keeps whatever stance the unit is in; Rest returns it to Neutral.
# Using a skill from another stance switches stance automatically for +1 AP
# (no block cost). Rest switches to Neutral for free.
# ==============================================================================

enum Kind { ATTACK, DEFEND, FOCUS, MOVE, REST, TARGET }

const MIN_LEVEL: int = 1
const MAX_LEVEL: int = 5

var id: String = ""
var display_name: String = ""
var kind: Kind = Kind.ATTACK
var level: int = 1
var blocks: int = 1            # timeline length
var power_mult: float = 1.0    # ATTACK: multiplier on the unit's strike power
var mitigation: float = 0.5    # DEFEND: damage reduction
var boost: float = 1.5         # FOCUS: attack multiplier
var boost_rounds: int = 2      # FOCUS: duration
var recover_ap: int = 2        # REST: AP regained
var description: String = ""

static func create(p_id: String, p_name: String, p_kind: Kind, p_level: int, p_blocks: int, p_description: String) -> SkillDef:
	var skill := SkillDef.new()
	skill.id = p_id
	skill.display_name = p_name
	skill.kind = p_kind
	skill.level = clampi(p_level, MIN_LEVEL, MAX_LEVEL)
	skill.blocks = p_blocks
	skill.description = p_description
	return skill

# Base AP = level; effects on the unit shift it (never below 1 or above 5).
# Move: the cost PER TILE (1 + shift). Rest: always free.
func ap_cost(actor: Combatant = null) -> int:
	if kind == Kind.REST or kind == Kind.TARGET:
		return 0
	var shift: int = actor.ap_cost_modifier if actor != null else 0
	var base: int = 1 if kind == Kind.MOVE else level
	return clampi(base + shift, MIN_LEVEL, MAX_LEVEL)

# The stance this skill puts the unit in (Move keeps the current one; see stance_after).
func stance() -> CombatTypes.StanceType:
	match kind:
		Kind.ATTACK:
			return CombatTypes.StanceType.OFFENSIVE
		Kind.DEFEND:
			return CombatTypes.StanceType.DEFENSIVE
		Kind.FOCUS:
			return CombatTypes.StanceType.TACTICAL
	return CombatTypes.StanceType.NEUTRAL

func stance_after(current: CombatTypes.StanceType) -> CombatTypes.StanceType:
	return current if (kind == Kind.MOVE or kind == Kind.TARGET) else stance()

# Automatic stance-switch penalty (AP): changing stance costs 1; Move and Rest never do.
func switch_cost(current: CombatTypes.StanceType) -> int:
	if kind == Kind.MOVE or kind == Kind.REST or kind == Kind.TARGET:
		return 0
	return 0 if current == stance() else 1

func icon() -> String:
	match kind:
		Kind.ATTACK:
			return "⚔"
		Kind.DEFEND:
			return "🛡"
		Kind.FOCUS:
			return "✦"
		Kind.MOVE:
			return "➜"
		Kind.TARGET:
			return "🎯"
	return "💤"

func label() -> String:
	return "%s %s" % [icon(), display_name]
