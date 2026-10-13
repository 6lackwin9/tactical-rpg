class_name SkillDef
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Definition (one tier of a command family)
# Every skill has a level 1-5 and its base AP cost IS its level: a level 2
# Attack, Defend or Move always costs 2 AP unless an effect shifts it
# (Combatant.ap_cost_modifier). Higher levels are stronger or faster, and they
# unlock through skill progression. Each kind belongs to a stance pool; see
# CombatTypes.can_flow() for which stances may follow each other.
# ==============================================================================

enum Kind { ATTACK, DEFEND, FOCUS, MOVE, NEUTRAL }

const MIN_LEVEL: int = 1
const MAX_LEVEL: int = 5

var id: String = ""
var display_name: String = ""
var kind: Kind = Kind.ATTACK
var level: int = 1
var blocks: int = 1            # timeline length
var steps: int = 1             # MOVE: tiles covered in a straight line
var power_mult: float = 1.0    # ATTACK: multiplier on the unit's strike power
var mitigation: float = 0.5    # DEFEND: damage reduction
var boost: float = 1.5         # FOCUS: attack multiplier
var boost_rounds: int = 2      # FOCUS: duration
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
func ap_cost(actor: Combatant = null) -> int:
	var shift: int = actor.ap_cost_modifier if actor != null else 0
	return clampi(level + shift, MIN_LEVEL, MAX_LEVEL)

func stance() -> CombatTypes.StanceType:
	match kind:
		Kind.ATTACK:
			return CombatTypes.StanceType.OFFENSIVE
		Kind.DEFEND:
			return CombatTypes.StanceType.DEFENSIVE
		Kind.FOCUS:
			return CombatTypes.StanceType.TACTICAL
		Kind.MOVE:
			return CombatTypes.StanceType.MOBILE
	return CombatTypes.StanceType.NEUTRAL

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
	return "🔄"

func label() -> String:
	return "%s %s" % [icon(), display_name]

func build_command(actor: Combatant, grid: CombatGrid, direction: Vector2i, target: Combatant, base_power: int) -> BattleCommand:
	var command: BattleCommand
	match kind:
		Kind.ATTACK:
			command = AttackCommand.new(actor, target, maxi(1, roundi(float(base_power) * power_mult)), blocks, 0, grid)
		Kind.DEFEND:
			command = DefendCommand.new(actor, mitigation, blocks, 1)
		Kind.FOCUS:
			command = FocusCommand.new(actor, boost, boost_rounds, blocks, 0)
		Kind.MOVE:
			# A move with a target is the live approach of an Engage step (re-paths when it runs).
			if target != null:
				command = ApproachCommand.new(actor, grid, target, steps, 1)
			else:
				command = MoveCommand.new(actor, grid, direction, steps, 1)
		_:
			command = EnterNeutralCommand.new(actor, 1)
	command.command_name = display_name
	return command
