class_name CombatResolver
extends RefCounted

const TOTAL_BLOCKS: int = 10

class ScheduledAction:
	extends RefCounted
	var combatant: Combatant
	var command: BattleCommand
	var start_block: int
	var end_block: int
	var is_cancelled: bool = false

	func _init(p_combatant: Combatant, p_command: BattleCommand, p_start: int, p_end: int) -> void:
		combatant = p_combatant
		command = p_command
		start_block = p_start
		end_block = p_end

signal block_started(block: int)
signal block_completed(block: int)
signal log_emitted(message: String)

var scheduled_actions: Array[ScheduledAction] = []
var registered_combatants: Array[Combatant] = []
var current_round: int = 1
var grid: CombatGrid = null

func register_combatant(combatant: Combatant) -> void:
	if not registered_combatants.has(combatant):
		registered_combatants.append(combatant)

func schedule_action(combatant: Combatant, command: BattleCommand, start_block: int) -> bool:
	register_combatant(combatant)
	var end_block: int = start_block + command.block_cost - 1
	if start_block < 1 or end_block > TOTAL_BLOCKS:
		push_error("Action '%s' out of timeline bounds (Blocks %d-%d, max %d)" % [command.command_name, start_block, end_block, TOTAL_BLOCKS])
		return false

	# Check for overlap on this combatant's timeline
	for existing in scheduled_actions:
		if existing.combatant == combatant and not existing.is_cancelled:
			if not (end_block < existing.start_block or start_block > existing.end_block):
				push_error("Action '%s' overlaps with existing action '%s' on [%s]'s timeline" % [
					command.command_name, existing.command.command_name, combatant.name
				])
				return false

	scheduled_actions.append(ScheduledAction.new(combatant, command, start_block, end_block))
	return true

# Backward-compatible queuing helper: schedules action to next available block
func queue_action(command: BattleCommand) -> void:
	var combatant: Combatant = command.source
	register_combatant(combatant)
	var next_start: int = 1
	for existing in scheduled_actions:
		if existing.combatant == combatant and not existing.is_cancelled:
			next_start = maxi(next_start, existing.end_block + 1)
	if next_start + command.block_cost - 1 <= TOTAL_BLOCKS:
		schedule_action(combatant, command, next_start)
	else:
		push_error("Timeline full for [%s]: cannot queue '%s'" % [combatant.name, command.command_name])

static func _get_target_display_name(target_val: Variant) -> String:
	if target_val is Combatant:
		@warning_ignore("unsafe_cast")
		return (target_val as Combatant).name
	elif target_val != null:
		return str(target_val)
	return "None"

func get_actions_at_block(block: int) -> Array[ScheduledAction]:
	var result: Array[ScheduledAction] = []
	for a in scheduled_actions:
		if not a.is_cancelled and a.start_block <= block and a.end_block >= block:
			result.append(a)
	return result

func execute_block(block: int) -> void:
	block_started.emit(block)
	_resolve_block(block)
	block_completed.emit(block)

func finish_round() -> void:
	print("\n--- ROUND %d CONCLUDED (ROUND ROLLOVER) ---" % current_round)
	for c: Combatant in registered_combatants:
		if c.is_active():
			c.tick_round_buffs()
			var target_name: String = _get_target_display_name(c.active_target)
			var pos_str: String = str(grid.get_position(c)) if grid != null else "N/A"
			var msg: String = "  [%s] Status: %d/%d HP | Pos: %s | Stance: %s | Target: %s" % [
				c.name, c.hp, c.max_hp,
				pos_str,
				CombatTypes.stance_to_string(c.current_stance),
				target_name
			]
			print(msg)
			log_emitted.emit(msg)
		elif c.is_withdrawn:
			var msg: String = "  [%s] Status: WITHDRAWN" % c.name
			print(msg)
			log_emitted.emit(msg)

	scheduled_actions.clear()
	current_round += 1

func resolve_round() -> void:
	print("\n==========================================")
	print("=== RESOLVING 10-BLOCK ROUND %d ===" % current_round)
	print("==========================================")

	for block: int in range(1, TOTAL_BLOCKS + 1):
		execute_block(block)

	finish_round()

func resolve_turn() -> void:
	resolve_round()

func is_combat_resolved(allies: Array[Combatant], enemies: Array[Combatant]) -> bool:
	var active_allies: int = 0
	for a: Combatant in allies:
		if a.is_active():
			active_allies += 1
	var active_enemies: int = 0
	for e: Combatant in enemies:
		if e.is_active():
			active_enemies += 1
	return active_allies == 0 or active_enemies == 0

func _resolve_block(block: int) -> void:
	# 1. Reset transient block-level defense states
	for c: Combatant in registered_combatants:
		c.reset_block_transient_state()

	# 2. Check and activate continuous defensive postures active during this block
	for action in scheduled_actions:
		if not action.is_cancelled and action.combatant.is_active():
			if block >= action.start_block and block <= action.end_block:
				if action.command is DefendCommand:
					action.combatant.defense_mitigation_ratio = (action.command as DefendCommand).mitigation_ratio

	# 3. Find actions that finish/resolve on this block
	var resolving: Array[ScheduledAction] = []
	for action in scheduled_actions:
		if not action.is_cancelled and action.end_block == block:
			resolving.append(action)

	if resolving.is_empty():
		return

	print("\n[--- Block %d ---]" % block)

	# 4. Sort descending by initiative: (priority * 1000) + speed
	resolving.sort_custom(func(a: ScheduledAction, b: ScheduledAction) -> bool:
		return a.command.get_initiative() > b.command.get_initiative()
	)

	# 5. Execute commands in order
	for action in resolving:
		if action.is_cancelled:
			continue
		if not action.combatant.is_active():
			action.is_cancelled = true
			continue

		# Execute the command
		action.command.execute()

		# Update stance upon resolution
		if action.combatant.is_active() and action.command.resulting_stance != CombatTypes.StanceType.NEUTRAL:
			action.combatant.set_stance(action.command.resulting_stance)

		# Check for knockouts or withdrawals and cancel future actions
		for c: Combatant in registered_combatants:
			if not c.is_active():
				_cancel_actions_for(c)

func _cancel_actions_for(combatant: Combatant) -> void:
	for action in scheduled_actions:
		if action.combatant == combatant and not action.is_cancelled:
			action.is_cancelled = true