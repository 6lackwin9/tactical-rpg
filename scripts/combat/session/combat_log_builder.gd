class_name CombatLogBuilder
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Log Builder
# Turns resolved scheduled actions into ActionLogEntry cells and formats raw
# resolver messages into BBCode lines for the text feed.
# ==============================================================================

static func from_action(act: CombatResolver.ScheduledAction, round_num: int, block_num: int, allies: Array[Combatant], grid: CombatGrid) -> ActionLogEntry:
	var entry := ActionLogEntry.new(round_num, block_num)
	entry.actor_name = act.combatant.name
	entry.actor_is_ally = allies.has(act.combatant)

	if act.command is AttackCommand:
		var attack: AttackCommand = act.command as AttackCommand
		entry.category = "ATTACK"
		var target_c: Combatant = attack.target
		var atk_range: int = attack.source.attack_range
		if atk_range > 1:
			entry.skill_subtype = "Ranged Projectile"
			entry.skill_name = "Quick Snipe" if entry.actor_is_ally else "Longbow Volley"
		else:
			entry.skill_subtype = "Melee Physical"
			entry.skill_name = "Heavy Slash" if entry.actor_is_ally else "Power Strike"
		entry.value_num = attack.power
		entry.outcome_tag = "HIT"
		if target_c != null:
			entry.target_name = target_c.name
			entry.target_is_ally = allies.has(target_c)
			if not target_c.is_alive():
				entry.outcome_tag = "DEFEATED"
			elif target_c.defense_mitigation_ratio > 0.0:
				entry.outcome_tag = "MITIGATED"
			entry.status_text = "Target HP: %d/%d" % [target_c.hp, target_c.max_hp]
		else:
			entry.target_name = "Target"
		entry.detail_text = "Power: %d | Range: %d" % [attack.power, atk_range]

	elif act.command is MoveCommand:
		var move: MoveCommand = act.command as MoveCommand
		entry.category = "MOVE"
		entry.skill_name = "Sprint" if move.steps > 1 else "Tactical Move"
		entry.skill_subtype = "Sprint Relocation" if move.steps > 1 else "Standard Step"
		entry.value_num = move.steps
		entry.coord_to = grid.get_position(act.combatant) if grid != null else Vector2i(-1, -1)
		if act.combatant.is_withdrawn:
			entry.outcome_tag = "WITHDRAWN"
			entry.status_text = "Safely exited battle grid"
		else:
			entry.outcome_tag = "MOVED"
			entry.status_text = "Position: %s" % str(entry.coord_to)
		entry.detail_text = "Stepped %d tiles (%s)" % [move.steps, str(move.direction)]

	elif act.command is DefendCommand:
		var defend: DefendCommand = act.command as DefendCommand
		entry.category = "DEFEND"
		entry.skill_name = "Defensive Stance"
		entry.skill_subtype = "Shield Guard"
		entry.value_num = int(defend.mitigation_ratio * 100.0)
		entry.outcome_tag = "%d%% MITIGATION" % entry.value_num
		entry.detail_text = "Damage mitigation posture active"
		entry.status_text = "Duration: %d Blocks" % defend.block_cost

	elif act.command is TargetCommand:
		var target_cmd: TargetCommand = act.command as TargetCommand
		entry.category = "TARGET"
		entry.skill_name = "Target Lock"
		entry.skill_subtype = "Engagement Focus"
		var target_value: Variant = target_cmd.target_value
		if target_value is Combatant:
			var locked: Combatant = target_value
			entry.target_name = locked.name
			entry.target_is_ally = allies.has(locked)
		elif target_value != null:
			entry.target_name = str(target_value)
		else:
			entry.target_name = "None"
		entry.outcome_tag = "LOCKED"
		entry.detail_text = "Assigned tactical target focus"
		entry.status_text = "Target Locked"

	elif act.command is FocusCommand:
		var focus: FocusCommand = act.command as FocusCommand
		entry.category = "BUFF"
		entry.skill_name = "Focus Skill"
		entry.skill_subtype = "Attack Multiplier"
		entry.value_num = int((focus.boost_multiplier - 1.0) * 100.0)
		entry.outcome_tag = "+%d%% ATK" % entry.value_num
		entry.detail_text = "+%d%% Attack Power for %d Round(s)" % [entry.value_num, focus.duration_rounds]
		entry.status_text = "Empowered"

	elif act.command is RestCommand:
		var rest: RestCommand = act.command as RestCommand
		entry.category = "BUFF"
		entry.skill_name = "Rest"
		entry.skill_subtype = "Recover AP"
		entry.value_num = rest.recover_ap
		entry.outcome_tag = "+%d AP" % rest.recover_ap
		entry.detail_text = "Catches their breath and returns to Neutral"
		entry.status_text = "Recovering"

	elif act.command is HoldStanceCommand:
		entry.category = "DEFEND"
		entry.skill_name = "Hold Stance"
		entry.skill_subtype = "Posture Maintenance"
		entry.outcome_tag = "MAINTAINED"
		entry.detail_text = "Current stance maintained"
		entry.status_text = "Active"

	else:
		entry.category = "EVENT"
		entry.skill_name = act.command.command_name
		entry.skill_subtype = "Standard Action"
		entry.outcome_tag = "EXECUTED"
		entry.detail_text = act.command.command_name
	return entry

# Colors and iconifies raw CombatResolver log messages for the text feed.
static func format_resolver_message(msg: String) -> String:
	var formatted: String = "[color=#e2e8f0]" + msg + "[/color]"
	if msg.begins_with("[Attack]"):
		if "DEFEATED" in msg:
			formatted = "[color=#f87171]⚔ [/color][b][color=#ef4444]" + msg.substr(9) + "[/color][/b]"
		else:
			formatted = "[color=#f87171]⚔ " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Attack Out of Range]") or msg.begins_with("[Attack Miss]"):
		var tag_len: int = 22 if msg.begins_with("[Attack Out of Range]") else 14
		formatted = "[color=#fb7185]❌ [i]" + msg.substr(tag_len) + "[/i][/color]"
	elif msg.begins_with("[Move]") or msg.begins_with("[Sprint]"):
		var tag_len_move: int = 9 if msg.begins_with("[Sprint]") else 7
		formatted = "[color=#38bdf8]➜ [b]" + msg.left(tag_len_move - 1).strip_edges() + "[/b]: " + msg.substr(tag_len_move) + "[/color]"
	elif msg.begins_with("[Move Blocked]") or msg.begins_with("[Sprint Blocked]"):
		formatted = "[color=#f59e0b]⚠ " + msg + "[/color]"
	elif msg.begins_with("[Withdrawal]"):
		formatted = "[b][color=#fbbf24]⚑ " + msg.substr(13) + "[/color][/b]"
	elif msg.begins_with("[Skill:"):
		formatted = "[color=#c084fc]✦ [b]" + msg + "[/b][/color]"
	elif msg.begins_with("[Defend]"):
		formatted = "[color=#60a5fa]🛡 [b]" + msg + "[/b][/color]"
	elif msg.begins_with("[Target]"):
		formatted = "[color=#34d399]🎯 " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Action Started]"):
		formatted = "[color=#94a3b8]⏳ " + msg.substr(17) + "[/color]"
	elif msg.begins_with("[Exhausted]"):
		formatted = "[color=#f59e0b]⚡ [b]" + msg.substr(12) + "[/b][/color]"
	elif msg.begins_with("[Rest]"):
		formatted = "[color=#4ade80]💤 " + msg.substr(7) + "[/color]"
	elif msg.begins_with("[Stance]"):
		formatted = "[color=#c084fc]◈ " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Approach]"):
		formatted = "[color=#38bdf8]➜ " + msg.substr(11) + "[/color]"
	elif msg.begins_with("[Cancelled]"):
		formatted = "[color=#ef4444]⛔ " + msg.substr(12) + "[/color]"
	elif msg.strip_edges().begins_with("[") and "Status:" in msg:
		formatted = "[color=#64748b]" + msg + "[/color]"
	return "  " + formatted
