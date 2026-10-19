class_name CombatResolver
extends RefCounted

const DEFAULT_BLOCKS: int = 6

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
signal action_note(action: ScheduledAction, message: String)   # something happened as a command started (e.g. a chase)
# BREPORT1 1.3: one combo of a resolved command (kind exploit, break or flow; a flow's target is the actor, status "").
# Emitted in _apply_combo for what it applies or logs; changes nothing.
signal combo_resolved(kind: String, actor: Combatant, target: Combatant, status: String, skill: String, killed: bool)

var scheduled_actions: Array[ScheduledAction] = []
var registered_combatants: Array[Combatant] = []
var current_round: int = 1
var grid: CombatGrid = null
var total_blocks: int = DEFAULT_BLOCKS   # blocks in a round (per encounter)
var rng: RandomNumberGenerator = RandomNumberGenerator.new()   # seeded per battle: crit & status rolls stay replayable

const DEFAULT_SEED: int = 1337

func _init() -> void:
	rng.seed = DEFAULT_SEED   # a fresh RNG is randomly seeded; battles (CombatSession) set the encounter's seed

func register_combatant(combatant: Combatant) -> void:
	if not registered_combatants.has(combatant):
		registered_combatants.append(combatant)

func schedule_action(combatant: Combatant, command: BattleCommand, start_block: int) -> bool:
	register_combatant(combatant)
	var end_block: int = start_block + command.block_cost - 1
	if start_block < 1 or end_block > total_blocks:
		push_error("Action '%s' out of timeline bounds (Blocks %d-%d, max %d)" % [command.command_name, start_block, end_block, total_blocks])
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

static func _get_target_display_name(target_val: Variant) -> String:
	if target_val is Combatant:
		@warning_ignore("unsafe_cast")
		return (target_val as Combatant).name
	elif target_val != null:
		return str(target_val)
	return "None"

func execute_block(block: int) -> void:
	block_started.emit(block)
	_resolve_block(block)
	block_completed.emit(block)

# Filled by apply_round_end_effects (S12S-L): Combatant -> HP regained from regen this round end.
var last_round_end_heals: Dictionary = {}
# Filled by apply_round_end_effects: one row per step that changed HP, in resolution order:
# {"unit": Combatant, "status": String, "amount": int (HP actually lost, or regained for regen), "source": String (applier name)}.
var last_round_end_steps: Array[Dictionary] = []

# End-of-round effects, applied once after the last block, per unit in registered order (S12S-L rule 3):
#   1. bleed  loses StatusEffects.bleed_damage (6% max HP, min 1)
#   2. poison loses the stacked magnitude
#   3. burn   loses magnitude % of max HP (min 1)
#   4. regen  regains magnitude % of max HP (min 1, capped at missing HP; Combatant.heal like SupportRules)
#   then every active unit on a round-end hazard tile rolls its status (HazardRules, S12T-L)
# Damage steps ignore guard and armor. Each step is skipped once the unit is no longer active, so a unit
# defeated by steps 1-3 does not regenerate. Returns Combatant -> damage dealt (bleed + poison + burn).
func apply_round_end_effects() -> Dictionary:
	var dealt: Dictionary = {}
	last_round_end_heals = {}
	last_round_end_steps = []
	for c: Combatant in registered_combatants:
		if c.is_active() and c.has_status(StatusEffects.BLEED):
			var hp_before: int = c.hp
			var amount: int = c.take_damage(StatusEffects.bleed_damage(c), true)
			_record_round_end(c, StatusEffects.BLEED, amount, hp_before - c.hp, dealt)
			_emit_round_end("[Bleed] %s bleeds for %d damage (HP %d/%d)%s" % [c.name, amount, c.hp, c.max_hp, _defeat_suffix(c)])
		if c.is_active() and c.has_status(StatusEffects.POISON) and StatusEffects.poison_damage(c) > 0:
			var hp_before: int = c.hp
			var amount: int = c.take_damage(StatusEffects.poison_damage(c), true)
			_record_round_end(c, StatusEffects.POISON, amount, hp_before - c.hp, dealt)
			_emit_round_end("[Poison] %s takes %d poison damage (HP %d/%d)%s" % [c.name, amount, c.hp, c.max_hp, _defeat_suffix(c)])
		if c.is_active() and c.has_status(StatusEffects.BURN):
			var hp_before: int = c.hp
			var amount: int = c.take_damage(StatusEffects.burn_damage(c), true)
			_record_round_end(c, StatusEffects.BURN, amount, hp_before - c.hp, dealt)
			_emit_round_end("[Burn] %s burns for %d damage (HP %d/%d)%s" % [c.name, amount, c.hp, c.max_hp, _defeat_suffix(c)])
		if c.is_active() and c.has_status(StatusEffects.REGEN):
			var before: int = c.hp
			c.heal(mini(StatusEffects.regen_amount(c), c.max_hp - c.hp))
			var healed: int = c.hp - before
			if healed > 0:
				last_round_end_heals[c] = healed
				last_round_end_steps.append({"unit": c, "status": StatusEffects.REGEN, "amount": healed, "source": str(c.status_sources.get(StatusEffects.REGEN, ""))})
				_emit_round_end("[Regen] %s regenerates %d HP (HP %d/%d)" % [c.name, healed, c.hp, c.max_hp])
	# 5. Terrain hazards (S12T-L): units standing on a round-end hazard tile roll its status (starts next round).
	for line: String in HazardRules.at_round_end(grid, registered_combatants, rng):
		_emit_round_end(line)
	return dealt

# `amount` is the step's damage (as logged and returned, like bleed always was); the step row records `lost`,
# the HP actually lost (no overkill), so per-source sums match the HP the units lost.
func _record_round_end(c: Combatant, status: String, amount: int, lost: int, dealt: Dictionary) -> void:
	dealt[c] = DictUtil.int_of(dealt, c) + amount
	last_round_end_steps.append({"unit": c, "status": status, "amount": lost, "source": str(c.status_sources.get(status, ""))})

static func _defeat_suffix(c: Combatant) -> String:
	return "" if c.is_alive() else " -> %s DEFEATED!" % c.name

func _emit_round_end(msg: String) -> void:
	print(msg)
	log_emitted.emit(msg)

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
	print("=== RESOLVING %d-BLOCK ROUND %d ===" % [total_blocks, current_round])
	print("==========================================")

	for block: int in range(1, total_blocks + 1):
		execute_block(block)

	apply_round_end_effects()
	finish_round()

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

func prepare_block(block: int) -> Array[ScheduledAction]:
	# 1. Reset transient block-level defense states
	for c: Combatant in registered_combatants:
		c.reset_block_transient_state()

	# 1b. Planner commands starting now pay AP and enter their stance
	_begin_actions(block)

	# 2. Check and activate continuous defensive postures active during this block
	for action in scheduled_actions:
		if not action.is_cancelled and action.combatant.is_active():
			if block >= action.start_block and block <= action.end_block:
				if action.command is DefendCommand:
					var defend: DefendCommand = action.command as DefendCommand
					if not defend.guard_broken:   # a stance break ended this guard for its remaining blocks (S12C-L)
						action.combatant.defense_mitigation_ratio = defend.mitigation_ratio

	# 2b. Notify when multi-block actions begin executing on this block
	for action in scheduled_actions:
		if not action.is_cancelled and action.combatant.is_active():
			if action.start_block == block and action.end_block > action.start_block:
				var channel_msg: String = "[Action Started] %s begins %s (resolves on Block %d)" % [
					action.combatant.name, action.command.command_name, action.end_block
				]
				print(channel_msg)
				log_emitted.emit(channel_msg)

	# 3. Find actions that finish/resolve on this block
	var resolving: Array[ScheduledAction] = []
	for action in scheduled_actions:
		if not action.is_cancelled and action.end_block == block:
			resolving.append(action)

	if resolving.is_empty():
		return []

	print("\n[--- Block %d ---]" % block)

	# 4. Sort descending by initiative: (priority * 1000) + speed
	resolving.sort_custom(func(a: ScheduledAction, b: ScheduledAction) -> bool:
		return a.command.get_initiative() > b.command.get_initiative()
	)
	return resolving

# Commands pay AP (skill + automatic stance switch) and enter their stance when
# they START (the stance stays until another command changes it). Overspending is allowed: the unit goes into AP
# debt and pays for it next round (reduced recovery; 3+ debt staggers, 5+ stuns).
func _begin_actions(block: int) -> void:
	for action: ScheduledAction in scheduled_actions:
		if action.is_cancelled or action.start_block != block:
			continue
		var c: Combatant = action.combatant
		if not c.is_active():
			continue
		var cmd: BattleCommand = action.command
		var total: int = cmd.ap_cost + cmd.stance_switch_cost
		var was_in_debt: bool = c.ap < 0
		c.ap -= total
		if c.ap < 0 and total > 0:
			var penalty: String = StatusEffects.debt_status(-c.ap)
			var msg: String = "[AP Debt] %s overspends on %s: AP %d%s." % [c.name, cmd.command_name, c.ap, (" -> %s next round" % StatusEffects.display_name(penalty)) if not penalty.is_empty() else (" (less AP next round)" if not was_in_debt else "")]
			log_emitted.emit(msg)
		if cmd.stance_on_start >= 0:
			var new_stance: CombatTypes.StanceType = cmd.stance_on_start as CombatTypes.StanceType
			if new_stance != c.current_stance:
				c.set_stance(new_stance)
				if cmd.stance_switch_cost > 0:
					log_emitted.emit("[Stance] %s shifts to %s stance (+%d AP)" % [c.name, CombatTypes.stance_to_string(new_stance), cmd.stance_switch_cost])
		cmd.rng = rng   # a dash entering a hazard tile rolls its status when the attack starts (S12T-L)
		var start_msg: String = cmd.on_start()
		if not start_msg.is_empty():
			log_emitted.emit(start_msg)
			action_note.emit(action, start_msg)
func execute_single_action(action: ScheduledAction) -> void:
	if action.is_cancelled:
		return
	if not action.combatant.is_active():
		action.is_cancelled = true
		var cancel_msg: String = "[Cancelled] %s's %s cancelled (unit inactive)." % [
			action.combatant.name, action.command.command_name
		]
		print(cancel_msg)
		log_emitted.emit(cancel_msg)
		return

	# Execute the command
	action.command.rng = rng
	action.command.execute()
	if not action.command.execution_log.is_empty():
		log_emitted.emit(action.command.execution_log)
	for reaction: Dictionary in action.command.reactions:
		log_emitted.emit(str(reaction["log"]))
	_apply_combo(action)

	# Check for knockouts or withdrawals and cancel future actions
	for c: Combatant in registered_combatants:
		if not c.is_active():
			_cancel_actions_for(c)

# Combo results of a command that just resolved (S12C-L): stance breaks end the broken units' running guards (their
# DefendCommands stop re-applying mitigation), then a flow move puts the unit in its end stance for free (not when the
# command failed or the unit is no longer active). set_stance side effects stay (leaving Defensive ends riposte).
func _apply_combo(action: ScheduledAction) -> void:
	var cmd: BattleCommand = action.command
	for broken: Combatant in cmd.broken_targets:
		for other: ScheduledAction in scheduled_actions:
			var running: bool = other.start_block <= action.end_block and other.end_block >= action.end_block
			if other.combatant == broken and running and not other.is_cancelled and other.command is DefendCommand:
				(other.command as DefendCommand).guard_broken = true
	for note: String in cmd.combo_notes:
		print(note)
		log_emitted.emit(note)
	var c: Combatant = action.combatant
	for row: Dictionary in cmd.exploited:
		var raw: Variant = row["target"]
		if raw is Combatant:
			var victim: Combatant = raw
			combo_resolved.emit("exploit", c, victim, str(row["status"]), cmd.command_name, not victim.is_alive())
	for broken: Combatant in cmd.broken_targets:
		combo_resolved.emit("break", c, broken, "", cmd.command_name, not broken.is_alive())
	if cmd.stance_on_end >= 0 and not cmd.failed and c.is_active():
		var end_stance: CombatTypes.StanceType = cmd.stance_on_end as CombatTypes.StanceType
		combo_resolved.emit("flow", c, c, "", cmd.command_name, not c.is_alive())
		c.set_stance(end_stance)
		var msg: String = "[Flow] %s flows into %s stance" % [c.name, CombatTypes.stance_to_string(end_stance)]
		print(msg)
		log_emitted.emit(msg)

func _resolve_block(block: int) -> void:
	var resolving := prepare_block(block)
	for action in resolving:
		execute_single_action(action)

func _cancel_actions_for(combatant: Combatant) -> void:
	for action in scheduled_actions:
		if action.combatant == combatant and not action.is_cancelled:
			action.is_cancelled = true
