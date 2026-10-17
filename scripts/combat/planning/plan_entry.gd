class_name PlanEntry
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Entry (one compiled command on a unit's 10-block timeline)
# Produced by CombatPlan from PlanSteps; build_command() turns it into the real
# planner-controlled BattleCommand: the resolver charges its AP (skill + any
# stance switch) when it starts, and the sequence stops if the unit can't pay.
# `step_index` points back to the step that produced it (an Engage step yields
# moves plus the attack).
# ==============================================================================

var skill: SkillDef = null
var start_block: int = 1
var direction: Vector2i = Vector2i.ZERO   # display hint for a move (first step)
var path: Array[Vector2i] = []            # MOVE: the unit steps taken
var tiles: int = 0                        # MOVE: tiles covered
var target: Combatant = null              # ATTACK target, or the chase target of an Engage move
var step_index: int = 0
var pos_before: Vector2i = Vector2i(-1, -1)  # where the unit stands before / after this command (preview ghosts)
var pos_after: Vector2i = Vector2i(-1, -1)
var goal: Vector2i = Vector2i(-1, -1)      # MOVE: the whole move's destination (a split move's halves share it)
var skill_ap: int = 0                     # AP for the skill itself
var blocks_override: int = 0               # MOVE: 1-2 blocks depending on the path length (0 = the skill's own)
var est_damage: int = 0                   # ATTACK: expected hit (DamageCalculator.expected); area: total on foes
var est_heal: int = 0                     # SUPPORT: expected HP restored (clamped to the target's missing HP)
var est_cleanse: Array[String] = []       # SUPPORT: debuffs the cast would remove from its target right now
var cells: Array[Vector2i] = []           # ATTACK (area): tiles hit
var hits_allies: int = 0                  # ATTACK (area): allies standing in the area (friendly fire)
var status_name: String = ""               # STATUS: the status that took this block
var chase: bool = false                    # ATTACK (single target): adjust one tile at the start if out of reach
var switch_ap: int = 0                    # automatic stance-switch penalty
var stance_before: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
var stance_after: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL

static func create(p_skill: SkillDef, p_start: int, p_step_index: int) -> PlanEntry:
	var entry := PlanEntry.new()
	entry.skill = p_skill
	entry.start_block = p_start
	entry.step_index = p_step_index
	return entry

func total_ap() -> int:
	return skill_ap + switch_ap

func blocks() -> int:
	return blocks_override if blocks_override > 0 else skill.blocks

func end_block() -> int:
	return start_block + blocks() - 1

func build_command(actor: Combatant, grid: CombatGrid) -> BattleCommand:
	var command: BattleCommand
	match skill.kind:
		SkillDef.Kind.ATTACK:
			if skill.is_area():
				var area := AreaAttackCommand.new(actor, grid, cells, skill.power_for(actor), skill.blocks)
				area.status_rounds = skill.status_rounds
				command = area
			else:
				var attack := AttackCommand.new(actor, target, skill.power_for(actor), skill.blocks, 0, grid)
				attack.use_offhand = skill.is_off_hand()
				attack.follow_tiles = 1 if (chase and skill.dash_tiles <= 0) else 0   # one-tile adjustment when the attack starts
				attack.dash_tiles = skill.dash_tiles
				attack.status_rounds = skill.status_rounds
				var band: Vector2i = skill.reach_for(actor)
				attack.min_range = band.x
				attack.max_range = band.y
				command = attack
			command.status_effect = skill.status_effect
			command.status_potency = skill.status_potency
			if not skill.status_effect.is_empty():
				command.status_potency += actor.status_potency_bonus()   # item / passive status_potency effects
			command.status_magnitude = skill.status_magnitude
		SkillDef.Kind.STATUS:
			command = StatusBlockCommand.new(actor, status_name)
		SkillDef.Kind.SUPPORT:
			var support := SupportCommand.new(actor, target, skill, grid, skill.blocks)
			support.est_heal = est_heal
			command = support
		SkillDef.Kind.DEFEND:
			var defend := DefendCommand.new(actor, skill.mitigation, skill.blocks, 1)
			defend.grant_status = skill.grant_status
			defend.status_rounds = skill.status_rounds
			defend.status_magnitude = skill.status_magnitude
			command = defend
		SkillDef.Kind.FOCUS:
			command = FocusCommand.new(actor, skill.boost, skill.boost_rounds, skill.blocks, 0)
		SkillDef.Kind.MOVE:
			# Engage approaches walk the planned path too (no live re-chasing); the attack
			# itself closes a one-tile gap when it starts.
			var walk := WalkCommand.new(actor, grid, path, blocks())
			walk.expected_start = pos_before
			walk.goal = goal
			walk.approach_target = target
			command = walk
		SkillDef.Kind.DISENGAGE:
			command = DisengageCommand.new(actor, grid, pos_after, skill.jump_tiles, skill.blocks)
		SkillDef.Kind.TARGET:
			command = TargetCommand.new(actor, target, 2)
		_:
			command = RestCommand.new(actor, skill.recover_ap, skill.blocks)
	command.command_name = skill.display_name
	command.ap_cost = skill_ap
	command.stance_switch_cost = switch_ap
	command.stance_on_start = -1 if skill.keeps_stance() else skill.stance()
	command.stance_on_end = skill.ends_in_stance   # flow move (S12C-L): -1 = none
	if skill.kind == SkillDef.Kind.ATTACK:
		command.disrupt = skill.disrupt
		command.exploit = skill.exploit.duplicate(true)
		command.exploit_consume = skill.exploit_consume
	command.est_damage = est_damage
	return command

# Planner note of a flow move: "flows into Defensive" ("" for other skills; S12C-L rule 1).
func flow_note() -> String:
	if skill == null or not skill.flows():
		return ""
	return "flows into %s" % CombatTypes.stance_to_string(skill.ends_in_stance as CombatTypes.StanceType)

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
