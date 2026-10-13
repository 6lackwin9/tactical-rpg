class_name SkillDef
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Definition (one tier of a command family)
# Leveled skills (level 1-5) cost AP equal to their level: a level 2 Attack or
# Defend always costs 2 AP unless an effect shifts it (Combatant.ap_cost_modifier).
# Move is the exception: 1 AP per tile covered (1-2 blocks, once per round). Rest
# is free and recovers AP. Switch Target is a free 1-block step inserted
# automatically when an attack changes the current target lock.
#
# Stances (rock-paper-scissors: Offensive > Tactical > Defensive > Offensive):
#   Attack -> Offensive, Defend -> Defensive, Focus -> Tactical.
#   Move keeps whatever stance the unit is in; Rest returns it to Neutral.
# Using a skill from another stance switches stance automatically for +1 AP
# (no block cost). Rest switches to Neutral for free.
# ==============================================================================

enum Kind { ATTACK, DEFEND, FOCUS, MOVE, REST, TARGET, STATUS }

# Area attacks target a tile: PLUS = tile + 4 neighbours, BURST = 3x3, LINE = 3 tiles
# straight out from the caster (aim at an adjacent tile).
enum Area { NONE, PLUS, BURST, LINE }

const MIN_LEVEL: int = 1
const MAX_LEVEL: int = 5

var id: String = ""
var display_name: String = ""
var kind: Kind = Kind.ATTACK
var level: int = 1
var blocks: int = 1            # timeline length
var power_mult: float = 1.0    # ATTACK: multiplier on the unit's strike power
var mitigation: float = 0.5    # DEFEND: damage reduction
var boost: float = 1.5         # FOCUS: attack multiplier
var boost_rounds: int = 2      # FOCUS: duration
var recover_ap: int = 2        # REST: AP regained
var area: Area = Area.NONE     # ATTACK: area shape (tile-targeted)
var offhand: bool = false      # ATTACK: melee off-hand jab (range 1) for ranged units
var stance_override: int = -1  # ATTACK in another stance pool (e.g. Shield Bash is Defensive)
var status_effect: String = "" # status rolled on hit (StatusEffects), active next round
var status_potency: int = 0
var reach_min: int = 0          # skill-specific reach band (0 = the unit's default band)
var reach_max: int = 0
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
# Move: the cost PER TILE (1 + shift). Rest: always free.
func ap_cost(actor: Combatant = null) -> int:
	if kind == Kind.REST or kind == Kind.TARGET or kind == Kind.STATUS:
		return 0
	var shift: int = actor.ap_cost_modifier if actor != null else 0
	var base: int = 1 if kind == Kind.MOVE else level
	return clampi(base + shift, MIN_LEVEL, MAX_LEVEL)

# The stance this skill puts the unit in (Move keeps the current one; see stance_after).
func stance() -> CombatTypes.StanceType:
	if stance_override >= 0:
		return stance_override as CombatTypes.StanceType
	match kind:
		Kind.ATTACK:
			return CombatTypes.StanceType.OFFENSIVE
		Kind.DEFEND:
			return CombatTypes.StanceType.DEFENSIVE
		Kind.FOCUS:
			return CombatTypes.StanceType.TACTICAL
	return CombatTypes.StanceType.NEUTRAL

func stance_after(current: CombatTypes.StanceType) -> CombatTypes.StanceType:
	return current if (kind == Kind.MOVE or kind == Kind.TARGET or kind == Kind.STATUS) else stance()

# Automatic stance-switch penalty (AP): changing stance costs 1; Move and Rest never do.
func switch_cost(current: CombatTypes.StanceType) -> int:
	if kind == Kind.MOVE or kind == Kind.REST or kind == Kind.TARGET or kind == Kind.STATUS:
		return 0
	return 0 if current == stance() else 1

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
		Kind.TARGET:
			return "🎯"
		Kind.STATUS:
			return "✖"
	return "💤"

# Distance band (min, max) at which `c` can use this attack: skill-specific if set;
# off-hand jabs and line areas 1; ranged units 2..3 (or their longer range); melee 1..range.
func reach_for(c: Combatant) -> Vector2i:
	if reach_max > 0:
		return Vector2i(reach_min, reach_max)
	if offhand or area == Area.LINE:
		return Vector2i(1, 1)
	if c.attack_range > 1:
		return Vector2i(2, maxi(3, c.attack_range))
	return Vector2i(1, c.attack_range)

func is_area() -> bool:
	return area != Area.NONE

# Tiles an area attack aimed at `cell` hits, cast from `from`.
func area_cells(from: Vector2i, cell: Vector2i) -> Array[Vector2i]:
	var cells: Array[Vector2i] = []
	match area:
		Area.PLUS:
			cells = [cell, cell + Vector2i(1, 0), cell + Vector2i(-1, 0), cell + Vector2i(0, 1), cell + Vector2i(0, -1)]
		Area.BURST:
			for dx: int in range(-1, 2):
				for dy: int in range(-1, 2):
					cells.append(cell + Vector2i(dx, dy))
		Area.LINE:
			var dir: Vector2i = cell - from
			for i: int in range(1, 4):
				cells.append(from + dir * i)
		_:
			cells = [cell]
	return cells

func label() -> String:
	return "%s %s" % [icon(), display_name]
