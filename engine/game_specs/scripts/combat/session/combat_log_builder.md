---
class: CombatLogBuilder
extends: RefCounted
description: "Tactical RPG - Combat Log Builder Turns resolved scheduled actions into ActionLogEntry cells and formats raw resolver messages into BBCode lines for the text feed."
---

# Methods

### `static from_action(act: CombatResolver.ScheduledAction, round_num: int, block_num: int, allies: Array[Combatant], grid: CombatGrid) -> ActionLogEntry`
```logic
var entry := ActionLogEntry.new(round_num, block_num)
entry.actor_name = act.combatant.name
entry.actor_is_ally = allies.has(act.combatant)

if act.command is AttackCommand:
	var attack: AttackCommand = act.command as AttackCommand
	entry.category = "ATTACK"
	var target_c: Combatant = attack.target
	var atk_range: int = attack.source.attack_range
	entry.skill_name = attack.command_name
	entry.skill_subtype = "Reach %d-%d" % [attack.reach_min(), attack.reach_max()]
	entry.value_num = attack.last_damage
	entry.damage = attack.last_damage
	entry.breakdown = attack.last_breakdown
	entry.outcome_tag = "HIT"
	if attack.last_damage == 0 and attack.execution_log.begins_with("[Attack Out of Range]"):
		entry.outcome_tag = "OUT OF RANGE"
	elif attack.last_damage == 0 and attack.execution_log.begins_with("[Attack Miss]"):
		entry.outcome_tag = "MISS"
	if target_c != null:
		entry.target_name = target_c.name
		entry.target_is_ally = allies.has(target_c)
		if not target_c.is_alive():
			entry.outcome_tag = "DEFEATED"
		elif attack.last_crit and attack.last_damage > 0:
			entry.outcome_tag = "CRIT"
		elif target_c.defense_mitigation_ratio > 0.0:
			entry.outcome_tag = "MITIGATED"
		entry.status_text = "Target HP: %d/%d" % [target_c.hp, target_c.max_hp]
	else:
		entry.target_name = "Target"
	entry.detail_text = "Dealt %d | Skill x%.2f | Range %d" % [attack.last_damage, attack.skill_mult, atk_range]

elif act.command is WalkCommand:
	var move: WalkCommand = act.command as WalkCommand
	entry.category = "MOVE"
	entry.skill_name = "Approach" if move.approach_target != null else "Move"
	entry.skill_subtype = "%d tile%s" % [move.moved_tiles, "" if move.moved_tiles == 1 else "s"]
	entry.value_num = move.moved_tiles
	entry.coord_to = grid.get_position(act.combatant) if grid != null else Vector2i(-1, -1)
	if act.combatant.is_withdrawn:
		entry.outcome_tag = "WITHDRAWN"
		entry.status_text = "Safely exited battle grid"
	elif move.cut_short:
		entry.outcome_tag = "CUT SHORT"
		entry.status_text = "Stopped at %s (blocked by %s)" % [str(entry.coord_to), move.blocked_by]
	else:
		entry.outcome_tag = "MOVED"
		entry.status_text = "Position: %s" % str(entry.coord_to)
	entry.detail_text = "Walked %d of %d planned tiles" % [move.moved_tiles, move.path.size()]

elif act.command is DisengageCommand:
	var jump: DisengageCommand = act.command as DisengageCommand
	entry.category = "MOVE"
	entry.skill_name = jump.command_name
	entry.skill_subtype = "Disengage"
	entry.value_num = jump.moved_tiles
	entry.coord_to = grid.get_position(act.combatant) if grid != null else Vector2i(-1, -1)
	entry.outcome_tag = "JUMPED" if jump.landed else "FAILED"
	entry.status_text = ("Position: %s" % str(entry.coord_to))
	entry.detail_text = jump.execution_log.trim_prefix("[Disengage] ")

elif act.command is DefendCommand:
	var defend: DefendCommand = act.command as DefendCommand
	entry.category = "DEFEND"
	entry.skill_name = defend.command_name
	entry.skill_subtype = "Guard -%d%%" % roundi(defend.mitigation_ratio * 100.0)
	entry.value_num = int(defend.mitigation_ratio * 100.0)
	entry.outcome_tag = "%d%% MITIGATION" % entry.value_num
	entry.detail_text = "Damage mitigation posture active"
	entry.status_text = "Duration: %d Blocks" % defend.block_cost

elif act.command is TargetCommand:
	var target_cmd: TargetCommand = act.command as TargetCommand
	entry.category = "TARGET"
	entry.skill_name = "Switch Target"
	entry.skill_subtype = "Target Lock"
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
	entry.skill_name = focus.command_name
	entry.skill_subtype = "Attack Multiplier"
	entry.value_num = int((focus.boost_multiplier - 1.0) * 100.0)
	entry.outcome_tag = "+%d%% ATK" % entry.value_num
	entry.detail_text = "+%d%% Attack Power for %d Round(s)" % [entry.value_num, focus.duration_rounds]
	entry.status_text = "Empowered"

elif act.command is AreaAttackCommand:
	var area: AreaAttackCommand = act.command as AreaAttackCommand
	entry.category = "ATTACK"
	entry.skill_name = area.command_name
	entry.skill_subtype = "Area Attack"
	entry.value_num = area.last_damage
	entry.damage = area.last_damage
	entry.breakdown = area.last_breakdown
	entry.outcome_tag = "%d HIT%s" % [area.last_hits, "" if area.last_hits == 1 else "S"] if area.last_hits > 0 else "MISS"
	entry.target_name = "%d tiles" % area.cells.size()
	entry.detail_text = area.execution_log.trim_prefix("[Attack] ")

elif act.command is ItemCommand:
	var item_cmd: ItemCommand = act.command as ItemCommand
	entry.category = "ATTACK" if item_cmd.last_damage > 0 else "BUFF"
	entry.skill_name = item_cmd.command_name
	entry.skill_subtype = "Item"
	entry.value_num = item_cmd.last_damage if item_cmd.last_damage > 0 else item_cmd.last_heal
	entry.damage = item_cmd.last_damage
	if item_cmd.target != null:
		entry.target_name = item_cmd.target.name
		entry.target_is_ally = allies.has(item_cmd.target)
		entry.status_text = "HP %d/%d" % [item_cmd.target.hp, item_cmd.target.max_hp]
	if not item_cmd.used:
		entry.outcome_tag = "NOT USED"
	elif item_cmd.last_damage > 0:
		entry.outcome_tag = "DEFEATED" if item_cmd.target != null and not item_cmd.target.is_alive() else "%d DMG" % item_cmd.last_damage
	elif item_cmd.last_boost > 0.0:
		entry.outcome_tag = "+%d%% ATK" % roundi((item_cmd.last_boost - 1.0) * 100.0)
	else:
		entry.outcome_tag = "+%d HP" % item_cmd.last_heal
	entry.detail_text = item_cmd.execution_log.trim_prefix("[Item] ")

elif act.command is SupportCommand:
	var support: SupportCommand = act.command as SupportCommand
	entry.category = "BUFF"
	entry.skill_name = support.command_name
	entry.skill_subtype = "Support"
	entry.value_num = support.last_heal
	if support.target != null:
		entry.target_name = support.target.name
		entry.target_is_ally = allies.has(support.target)
		entry.status_text = "HP %d/%d" % [support.target.hp, support.target.max_hp]
	if support.failed:
		entry.outcome_tag = "FAILED"
	elif support.skill != null and support.skill.heals():
		entry.outcome_tag = "+%d HP" % support.last_heal
	elif not support.last_status.is_empty():
		entry.outcome_tag = StatusEffects.display_name(support.last_status).to_upper()
	elif not support.last_cleansed.is_empty():
		entry.outcome_tag = "CLEANSED"
	else:
		entry.outcome_tag = "EXECUTED"
	if not support.last_status.is_empty():
		entry.detail_text = "%s next round" % StatusEffects.display_name(support.last_status)
	else:
		entry.detail_text = support.execution_log.trim_prefix("[Support] ")

elif act.command is StatusBlockCommand:
	var lost: StatusBlockCommand = act.command as StatusBlockCommand
	entry.category = "EVENT"
	entry.skill_name = StatusEffects.display_name(lost.status)
	entry.skill_subtype = "Block lost"
	entry.outcome_tag = "LOST"
	entry.detail_text = "%s can't act this block" % act.combatant.name

elif act.command is RestCommand:
	var rest: RestCommand = act.command as RestCommand
	entry.category = "BUFF"
	entry.skill_name = "Rest"
	entry.skill_subtype = "Recover AP"
	entry.value_num = rest.recover_ap
	entry.outcome_tag = "+%d AP" % rest.recover_ap
	entry.detail_text = "Catches their breath and settles into Tactical"
	entry.status_text = "Recovering"


else:
	entry.category = "EVENT"
	entry.skill_name = act.command.command_name
	entry.skill_subtype = "Standard Action"
	entry.outcome_tag = "EXECUTED"
	entry.detail_text = act.command.command_name
return entry
```

### `static riposte_entry(reaction: Dictionary, round_num: int, block_num: int, allies: Array[Combatant]) -> ActionLogEntry`
```logic
var entry := ActionLogEntry.new(round_num, block_num)
var defender: Combatant = reaction["defender"]
var attacker: Combatant = reaction["attacker"]
entry.actor_name = defender.name
entry.actor_is_ally = allies.has(defender)
entry.target_name = attacker.name
entry.target_is_ally = allies.has(attacker)
entry.category = "ATTACK"
entry.skill_name = "Riposte"
entry.skill_subtype = "Counter"
entry.value_num = DictUtil.to_int(reaction["damage"])
entry.damage = entry.value_num
entry.breakdown = str(reaction["breakdown"])
entry.outcome_tag = "DEFEATED" if DictUtil.to_bool(reaction["defeated"]) else ("CRIT" if DictUtil.to_bool(reaction["crit"]) else "COUNTER")
entry.status_text = "Target HP: %d/%d" % [attacker.hp, attacker.max_hp]
entry.detail_text = "Countered for %d" % entry.damage
return entry
```

### `static chase_entry(act: CombatResolver.ScheduledAction, round_num: int, block_num: int, allies: Array[Combatant], grid: CombatGrid, message: String) -> ActionLogEntry`
```logic
var entry := ActionLogEntry.new(round_num, block_num)
entry.actor_name = act.combatant.name
entry.actor_is_ally = allies.has(act.combatant)
entry.category = "MOVE"
entry.skill_name = "Chase"
entry.skill_subtype = "Closes the gap (1 AP)"
entry.outcome_tag = "CHASED"
entry.coord_to = grid.get_position(act.combatant) if grid != null else Vector2i(-1, -1)
var attack: AttackCommand = act.command as AttackCommand
if attack != null and attack.target != null:
	entry.target_name = attack.target.name
	entry.target_is_ally = allies.has(attack.target)
entry.status_text = "Position: %s" % str(entry.coord_to)
entry.detail_text = message.trim_prefix("[Chase] ")
return entry
```

### `static dash_entry(act: CombatResolver.ScheduledAction, round_num: int, block_num: int, allies: Array[Combatant], grid: CombatGrid, message: String) -> ActionLogEntry`
```logic
var entry: ActionLogEntry = chase_entry(act, round_num, block_num, allies, grid, message)
entry.skill_name = "Dash"
entry.skill_subtype = "Gap-close (no extra AP)"
entry.outcome_tag = "DASHED"
entry.detail_text = message.trim_prefix("[Dash] ")
return entry
```

### `static format_resolver_message(msg: String) -> String`
```logic
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
elif msg.begins_with("[Chase]"):
	formatted = "[color=#38bdf8]➜ [b]Chase:[/b] " + msg.substr(8) + "[/color]"
elif msg.begins_with("[Support]"):
	formatted = "[color=#4ade80]✚ " + msg.substr(10) + "[/color]"
elif msg.begins_with("[Riposte]"):
	formatted = "[color=#60a5fa]⚔ [b]" + msg + "[/b][/color]"
elif msg.begins_with("[Bleed]"):
	formatted = "[color=#f87171]🩸 " + msg.substr(8) + "[/color]"
elif msg.begins_with("[Poison]"):
	formatted = "[color=#a3e635]☠ " + msg.substr(9) + "[/color]"
elif msg.begins_with("[Burn]"):
	formatted = "[color=#fb923c]🔥 " + msg.substr(7) + "[/color]"
elif msg.begins_with("[Regen]"):
	formatted = "[color=#4ade80]✚ " + msg.substr(8) + "[/color]"
elif msg.begins_with("[Dash]") or msg.begins_with("[Dash Blocked]"):
	formatted = "[color=#38bdf8]➜ " + msg + "[/color]"
elif msg.begins_with("[Move Cut Short]"):
	formatted = "[color=#f59e0b]⚠ [b]Cut short:[/b] " + msg.substr(17) + "[/color]"
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
```

