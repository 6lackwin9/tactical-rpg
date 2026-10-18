---
class: IntentRules
extends: RefCounted
description: "Tactical RPG - Intent Rules (enemy intent tells; INTENT1, decision combat \"DR1\") Pure read-out of the enemies' scheduled actions for the planning UI. Enemy plans are drafted before the party plans (CombatEncounter.schedule_enemy_ai) and never re-planned, so a tell always names the plan that resolves. Display only: nothing here changes planning, AI drafts, resolution, recordings or the sim. Read through CombatSession.enemy_intents() (planning only). One record per active enemy on the grid: unit      Combatant         the enemy kind      String            kind_of(first scheduled action); with none: IDLE (\"Holding\") for an enemy that was on the grid when the round's enemies were scheduled, ARRIVING for one that was not (a wave unit placed later) target    Combatant or null attack / support: the command target; move: the Engage's approach target (WalkCommand.approach_target); else null cell      Vector2i          area: the aimed tile; move or flee without a target: the goal; else NO_CELL blocks    Vector2i          (start_block, end_block) of the first action; (0, 0) when arriving or idle skill     String            display name of the first action (\"\" when arriving or idle) revealed  bool              a foresight holder is targeted by any of the enemy's actions this round (reveals) readers   Array[Combatant]  the foresight holders it targets (empty unless revealed); their Combatant.foresight_source names the passive (hover card \"Read by ...\") plan      Array[Dictionary] revealed: every scheduled action in block order as {kind, target, cell, blocks, skill}; else []"
---

# Properties
* `NO_CELL`: Vector2i = Vector2i(-1, -1) (const)
* `ARRIVING`: String = "arriving" (const)
* `IDLE`: String = "idle" (const)
* `KINDS`: Array[String] = ["attack", "area", "support", "guard", "focus", "rest", "retreat", "move", "flee", "switch", "stunned", "act", ARRIVING, IDLE] (const)
* `out`: Array[CombatResolver.ScheduledAction] = []
* `out`: Array[Combatant] = []
* `cell`: Vector2i = grid.get_position(holder)
* `out`: Array[Dictionary] = []

# Methods

### `static kind_of(cmd: BattleCommand) -> String`
```logic
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
```

### `static target_of(cmd: BattleCommand) -> Combatant`
```logic
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
```

### `static cell_of(cmd: BattleCommand) -> Vector2i`
```logic
if cmd is AreaAttackCommand:
	var area: AreaAttackCommand = cmd
	if area.aim_cell != NO_CELL:
		return area.aim_cell
	return area.cells[0] if not area.cells.is_empty() else NO_CELL
if cmd is WalkCommand:
	var walk: WalkCommand = cmd
	return walk.goal if walk.approach_target == null else NO_CELL
return NO_CELL
```

### `static action_record(a: CombatResolver.ScheduledAction) -> Dictionary`
```logic
return {
	"kind": kind_of(a.command),
	"target": target_of(a.command),
	"cell": cell_of(a.command),
	"blocks": Vector2i(a.start_block, a.end_block),
	"skill": a.command.command_name,
}
```

### `static targets_unit(a: CombatResolver.ScheduledAction, holder: Combatant, holder_cell: Vector2i) -> bool`
```logic
if a.command is AreaAttackCommand:
	var area: AreaAttackCommand = a.command
	return holder_cell != NO_CELL and area.cells.has(holder_cell)
if a.command is AttackCommand or a.command is SupportCommand:
	return target_of(a.command) == holder
return false
```

### `static intent_of(unit: Combatant, party: Array[Combatant], scheduled: Array[CombatResolver.ScheduledAction], grid: CombatGrid, on_grid_at_schedule: Array[Combatant] = []) -> Dictionary`
```logic
var actions: Array[CombatResolver.ScheduledAction] = actions_of(unit, scheduled)
var plan: Array[Dictionary] = []
var readers: Array[Combatant] = []
if actions.is_empty():
	var empty_kind: String = IDLE if on_grid_at_schedule.has(unit) else ARRIVING
	return {"unit": unit, "kind": empty_kind, "target": null, "cell": NO_CELL, "blocks": Vector2i.ZERO, "skill": "", "revealed": false, "readers": readers, "plan": plan}
var record: Dictionary = action_record(actions[0])
readers = readers_of(actions, party, grid)
if not readers.is_empty():
	for a: CombatResolver.ScheduledAction in actions:
		plan.append(action_record(a))
record["unit"] = unit
record["revealed"] = not readers.is_empty()
record["readers"] = readers
record["plan"] = plan
return record
```

