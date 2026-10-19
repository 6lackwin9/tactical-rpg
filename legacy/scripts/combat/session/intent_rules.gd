class_name IntentRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Intent Rules (enemy intent tells; INTENT1, decision combat "DR1")
# Pure read-out of the enemies' scheduled actions for the planning UI. Enemy plans are
# drafted before the party plans (CombatEncounter.schedule_enemy_ai) and never re-planned,
# so a tell always names the plan that resolves. Display only: nothing here changes
# planning, AI drafts, resolution, recordings or the sim.
#
# Read through CombatSession.enemy_intents() (planning only). One record per active
# enemy on the grid:
#   unit      Combatant         the enemy
#   kind      String            kind_of(first scheduled action); with none: IDLE ("Holding") for an enemy that was
#                               on the grid when the round's enemies were scheduled, ARRIVING for one that was not
#                               (a wave unit placed later)
#   target    Combatant or null attack / support: the command target; move: the Engage's
#                               approach target (WalkCommand.approach_target); else null
#   cell      Vector2i          area: the aimed tile; move or flee without a target: the goal; else NO_CELL
#   blocks    Vector2i          (start_block, end_block) of the first action; (0, 0) when arriving or idle
#   skill     String            display name of the first action ("" when arriving or idle)
#   revealed  bool              a foresight holder is targeted by any of the enemy's actions this round, or the
#                               enemy is enraged (PACE1 Q2 A: its whole plan is shown to everyone)
#   enraged   bool              PACE1: the enemy is enraged this round (CombatEncounter.is_enraged)
#   readers   Array[Combatant]  the foresight holders it targets (empty unless revealed); their
#                               Combatant.foresight_source names the passive (hover card "Read by ...")
#   plan      Array[Dictionary] revealed (foresight or enraged): every scheduled action in block order as
#                               {kind, target, cell, blocks, skill}; else []
# ==============================================================================

const NO_CELL: Vector2i = Vector2i(-1, -1)
const ARRIVING: String = "arriving"
# Every kind a record can carry (UI chips map these; "act" = any other command).
const IDLE: String = "idle"
const KINDS: Array[String] = ["attack", "area", "support", "guard", "focus", "rest", "retreat", "move", "flee", "switch", "stunned", "act", ARRIVING, IDLE]

# The tell kind of one command.
static func kind_of(cmd: BattleCommand) -> String:
	if cmd is AreaAttackCommand:
		return "area"
	if cmd is AttackCommand:
		return "attack"
	if cmd is SupportCommand:
		return "support"
	if cmd is DefendCommand:
		return "guard"
	if cmd is FocusCommand:
		return "focus"
	if cmd is RestCommand:
		return "rest"
	if cmd is DisengageCommand:
		return "retreat"
	if cmd is WalkCommand:
		var walk: WalkCommand = cmd
		return "flee" if walk.grid != null and walk.goal != NO_CELL and walk.grid.is_exit_tile(walk.goal) else "move"
	if cmd is TargetCommand:
		return "switch"
	if cmd is StatusBlockCommand:
		return "stunned"
	return "act"

# The unit a command aims at: attack and support targets, the approach target of an Engage move; else null.
static func target_of(cmd: BattleCommand) -> Combatant:
	if cmd is AttackCommand:
		var attack: AttackCommand = cmd
		return attack.target
	if cmd is SupportCommand:
		var support: SupportCommand = cmd
		return support.target
	if cmd is WalkCommand:
		var walk: WalkCommand = cmd
		return walk.approach_target
	return null

# The tile a command aims at: an area's aimed tile (its first cell when the aim is unknown), the goal of a move
# without a target; else NO_CELL.
static func cell_of(cmd: BattleCommand) -> Vector2i:
	if cmd is AreaAttackCommand:
		var area: AreaAttackCommand = cmd
		if area.aim_cell != NO_CELL:
			return area.aim_cell
		return area.cells[0] if not area.cells.is_empty() else NO_CELL
	if cmd is WalkCommand:
		var walk: WalkCommand = cmd
		return walk.goal if walk.approach_target == null else NO_CELL
	return NO_CELL

# One plan line: {kind, target, cell, blocks, skill}.
static func action_record(a: CombatResolver.ScheduledAction) -> Dictionary:
	return {
		"kind": kind_of(a.command),
		"target": target_of(a.command),
		"cell": cell_of(a.command),
		"blocks": Vector2i(a.start_block, a.end_block),
		"skill": a.command.command_name,
	}

# `unit`'s live (not cancelled) scheduled actions in block order.
static func actions_of(unit: Combatant, scheduled: Array[CombatResolver.ScheduledAction]) -> Array[CombatResolver.ScheduledAction]:
	var out: Array[CombatResolver.ScheduledAction] = []
	for a: CombatResolver.ScheduledAction in scheduled:
		if a.combatant == unit and not a.is_cancelled:
			out.append(a)
	out.sort_custom(func(x: CombatResolver.ScheduledAction, y: CombatResolver.ScheduledAction) -> bool: return x.start_block < y.start_block)
	return out

# True when the action aims at `holder`: a single-target attack or support (an enemy's support on a hero is a debuff)
# with `holder` as target, or an area whose cells include `holder_cell` (the holder's round-start tile).
static func targets_unit(a: CombatResolver.ScheduledAction, holder: Combatant, holder_cell: Vector2i) -> bool:
	if a.command is AreaAttackCommand:
		var area: AreaAttackCommand = a.command
		return holder_cell != NO_CELL and area.cells.has(holder_cell)
	if a.command is AttackCommand or a.command is SupportCommand:
		return target_of(a.command) == holder
	return false

# The active foresight holders of `party` on the grid that any of `actions` targets (rule 3; empty = not revealed).
static func readers_of(actions: Array[CombatResolver.ScheduledAction], party: Array[Combatant], grid: CombatGrid) -> Array[Combatant]:
	var out: Array[Combatant] = []
	for holder: Combatant in party:
		if not holder.has_foresight() or not holder.is_active() or grid == null or not grid.has_combatant(holder):
			continue
		var cell: Vector2i = grid.get_position(holder)
		for a: CombatResolver.ScheduledAction in actions:
			if targets_unit(a, holder, cell):
				out.append(holder)
				break
	return out

# The record of one enemy (see the header for the keys).
static func intent_of(unit: Combatant, party: Array[Combatant], scheduled: Array[CombatResolver.ScheduledAction], grid: CombatGrid, on_grid_at_schedule: Array[Combatant] = [], enraged: bool = false) -> Dictionary:
	var actions: Array[CombatResolver.ScheduledAction] = actions_of(unit, scheduled)
	var plan: Array[Dictionary] = []
	var readers: Array[Combatant] = []
	if actions.is_empty():
		var empty_kind: String = IDLE if on_grid_at_schedule.has(unit) else ARRIVING
		return {"unit": unit, "kind": empty_kind, "target": null, "cell": NO_CELL, "blocks": Vector2i.ZERO, "skill": "", "revealed": enraged, "readers": readers, "plan": plan, "enraged": enraged}
	var record: Dictionary = action_record(actions[0])
	readers = readers_of(actions, party, grid)
	if not readers.is_empty() or enraged:
		for a: CombatResolver.ScheduledAction in actions:
			plan.append(action_record(a))
	record["unit"] = unit
	record["revealed"] = not readers.is_empty() or enraged
	record["readers"] = readers
	record["plan"] = plan
	record["enraged"] = enraged
	return record

# One record per active enemy on the grid, in `enemies` order.
# `enraged_units`: the enemies enraged this round (PACE1; CombatEncounter.is_enraged).
static func intents(enemies: Array[Combatant], party: Array[Combatant], scheduled: Array[CombatResolver.ScheduledAction], grid: CombatGrid, on_grid_at_schedule: Array[Combatant] = [], enraged_units: Array[Combatant] = []) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for e: Combatant in enemies:
		if e.is_active() and grid != null and grid.has_combatant(e):
			out.append(intent_of(e, party, scheduled, grid, on_grid_at_schedule, enraged_units.has(e)))
	return out
