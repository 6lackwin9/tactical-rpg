class_name PlanEntry
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Entry (one planned command on a unit's 10-block timeline)
# A lightweight description of a command (kind, start block, direction, target)
# shared by the player's planning UI and the enemy AI. build_command() turns it
# into the real BattleCommand the resolver executes.
# ==============================================================================

enum Kind { MOVE, SPRINT, ATTACK, DEFEND, FOCUS, HOLD, NEUTRAL }

var kind: Kind = Kind.MOVE
var start_block: int = 1
var direction: Vector2i = Vector2i.ZERO
var target: Combatant = null
var power: int = 20

static func create(p_kind: Kind, p_start: int, p_direction: Vector2i = Vector2i.ZERO, p_target: Combatant = null, p_power: int = 20) -> PlanEntry:
	var entry := PlanEntry.new()
	entry.kind = p_kind
	entry.start_block = p_start
	entry.direction = p_direction
	entry.target = p_target
	entry.power = p_power
	return entry

static func blocks_of(p_kind: Kind) -> int:
	match p_kind:
		Kind.ATTACK, Kind.DEFEND, Kind.FOCUS:
			return 2
	return 1

# Action-point cost: acting hard in a round is limited by the AP budget.
static func ap_of(p_kind: Kind) -> int:
	match p_kind:
		Kind.MOVE:
			return 1
		Kind.DEFEND, Kind.FOCUS:
			return 2
		Kind.SPRINT, Kind.ATTACK:
			return 3
	return 0

static func kind_name(p_kind: Kind) -> String:
	match p_kind:
		Kind.MOVE:
			return "Move"
		Kind.SPRINT:
			return "Sprint"
		Kind.ATTACK:
			return "Attack"
		Kind.DEFEND:
			return "Defend"
		Kind.FOCUS:
			return "Focus"
		Kind.HOLD:
			return "Hold Stance"
	return "Enter Neutral"

static func dir_arrow(dir: Vector2i) -> String:
	if dir == Vector2i(1, 0):
		return "→"
	if dir == Vector2i(-1, 0):
		return "←"
	if dir == Vector2i(0, -1):
		return "↑"
	if dir == Vector2i(0, 1):
		return "↓"
	return "·"

static func needs_direction(p_kind: Kind) -> bool:
	return p_kind == Kind.MOVE or p_kind == Kind.SPRINT

static func needs_target(p_kind: Kind) -> bool:
	return p_kind == Kind.ATTACK

func blocks() -> int:
	return blocks_of(kind)

func end_block() -> int:
	return start_block + blocks_of(kind) - 1

func ap_cost() -> int:
	return ap_of(kind)

func build_command(actor: Combatant, grid: CombatGrid) -> BattleCommand:
	match kind:
		Kind.MOVE:
			return MoveCommand.new(actor, grid, direction, 1, 1)
		Kind.SPRINT:
			return MoveCommand.new(actor, grid, direction, 2, 1)
		Kind.ATTACK:
			return AttackCommand.new(actor, target, power, 2, 0, grid)
		Kind.DEFEND:
			return DefendCommand.new(actor, 0.5, 2, 1)
		Kind.FOCUS:
			return FocusCommand.new(actor, 1.5, 2, 2, 0)
		Kind.HOLD:
			return HoldStanceCommand.new(actor, 0)
	return EnterNeutralCommand.new(actor, 1)
