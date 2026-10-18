class_name PlanStep
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Step (one input in a unit's round sequence)
# What the player (or AI) chooses; CombatPlan compiles each step into timeline
# entries at the lowest free blocks.
#   ENGAGE    attack skill + target; the moves needed to get in range are
#             inserted automatically (and chase the target when they run)
#   MOVE      manual reposition: a straight move of `tiles` in a direction
#   GOTO      move to a chosen tile (path-found, split into blocks by speed)
#   AREA      area attack aimed at a tile (no auto-move: the tile must be in reach)
#   WITHDRAW  auto-path to the exit tile and leave the battle
#   SELF      Defend / Focus / Rest, or a SUPPORT skill cast on the caster
#   ALLY      SUPPORT skill on a friendly unit; walks into the skill's reach first if needed
#   ITEM      use a battle-pouch consumable (`item_id`) on `target` (self, ally or foe by type; no auto-move;
#             ITEMUSE1). PlanCodec records it as {"mode": "ITEM", "item": id, "target": <unit ref>, ...}.
# ==============================================================================

enum Mode { ENGAGE, MOVE, GOTO, AREA, WITHDRAW, SELF, ALLY, ITEM }   # new modes go last (recorded ints)

var mode: Mode = Mode.SELF
var skill: SkillDef = null
var target: Combatant = null
var direction: Vector2i = Vector2i.ZERO
var tiles: int = 1
var cell: Vector2i = Vector2i(-1, -1)   # GOTO destination
var item_id: String = ""                # ITEM: the catalog consumable id used from the battle pouch

static func engage(p_skill: SkillDef, p_target: Combatant) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.ENGAGE
	step.skill = p_skill
	step.target = p_target
	return step

static func move(p_skill: SkillDef, p_direction: Vector2i, p_tiles: int = 1) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.MOVE
	step.skill = p_skill
	step.direction = p_direction
	step.tiles = p_tiles
	return step

static func go_to(p_skill: SkillDef, p_cell: Vector2i) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.GOTO
	step.skill = p_skill
	step.cell = p_cell
	return step

static func area_at(p_skill: SkillDef, p_cell: Vector2i) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.AREA
	step.skill = p_skill
	step.cell = p_cell
	return step

static func withdraw() -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.WITHDRAW
	return step

static func ally(p_skill: SkillDef, p_target: Combatant) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.ALLY
	step.skill = p_skill
	step.target = p_target
	return step

# ITEMUSE1 1.3: use pouch item `p_item_id` on `p_target` (the user itself for self items). Skill-less step.
static func use_item(p_item_id: String, p_target: Combatant) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.ITEM
	step.item_id = p_item_id
	step.target = p_target
	return step

static func self_skill(p_skill: SkillDef) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.SELF
	step.skill = p_skill
	return step

# Short chip text for the sequence preview.
func title() -> String:
	match mode:
		Mode.ENGAGE:
			return "%s → %s" % [skill.label(), target.name if target != null else "?"]
		Mode.MOVE:
			return "%s %s%d" % [skill.label(), PlanEntry.dir_arrow(direction), tiles]
		Mode.GOTO:
			return "%s to %s" % [skill.label(), str(cell)]
		Mode.AREA:
			return "%s at %s" % [skill.label(), str(cell)]
		Mode.ALLY:
			return "%s → %s" % [skill.label(), target.name if target != null else "?"]
		Mode.WITHDRAW:
			return "🏃 Withdraw"
		Mode.ITEM:
			var item_name: String = str(ContentCatalog.get_entry("items", item_id).get("name", item_id))
			return "Use %s → %s" % [item_name, target.name if target != null else "?"]
	return skill.label()

func duplicate_step() -> PlanStep:
	var copy := PlanStep.new()
	copy.mode = mode
	copy.skill = skill
	copy.target = target
	copy.direction = direction
	copy.tiles = tiles
	copy.cell = cell
	copy.item_id = item_id
	return copy
