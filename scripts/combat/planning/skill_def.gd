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

# SUPPORT: heals and/or grants a buff status to itself or a friendly unit (SupportRules,
# SupportCommand). It keeps the unit's stance like Move does (no switch cost).
enum Kind { ATTACK, DEFEND, FOCUS, MOVE, REST, TARGET, STATUS, SUPPORT }

# SUPPORT targeting: the caster only, a friendly unit other than the caster, or either.
enum TargetMode { SELF, ALLY, ALLY_OR_SELF }

# Area attacks target a tile: PLUS = tile + 4 neighbours, BURST = 3x3, LINE = 3 tiles
# straight out from the caster (aim at an adjacent tile).
enum Area { NONE, PLUS, BURST, LINE }

const SUPPORT_REACH: int = 3   # default reach of a SUPPORT skill (or the caster's attack range if longer)
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
var weapon_types: Array[String] = []   # gear types that may use it (empty = any); class-derived units only
var min_proficiency: int = 0           # training level in the wielded weapon type needed (0 = none)
var status_rounds: int = 1      # rounds an inflicted / granted status lasts once active
var dash_tiles: int = 0         # ATTACK gap-close: move up to this many tiles to get adjacent, then strike (counts as the round's move)
var target_mode: TargetMode = TargetMode.ALLY_OR_SELF   # SUPPORT: who it may target
var heal_stat: String = ""      # SUPPORT: stat the heal scales with ("INT" / "KNOW"; "" = fixed heal only)
var heal_base: int = 0          # SUPPORT: flat heal
var heal_mult: float = 0.0      # SUPPORT: heal per point of heal_stat (see SupportRules.heal_amount)
var grant_status: String = ""   # SUPPORT: buff status given to the target (StatusEffects.buffs())
var learned_only: bool = false  # unrestricted (hand-built) units don't know it unless they learn it
var family: String = ""         # skill family (tiers of one skill); empty = its own id (see family_id())
var tier: int = 1               # position in the family (1..5); each tier is its own skill with its own AP, blocks and power
var basic: bool = false         # available to every class (counted outside the loadout slots)
var description: String = ""
var system: bool = false       # engine skill (Move, Rest, Switch Target, status blocks): always known, not deletable

static func create(p_id: String, p_name: String, p_kind: Kind, p_level: int, p_blocks: int, p_description: String) -> SkillDef:
	var skill := SkillDef.new()
	skill.id = p_id
	skill.display_name = p_name
	skill.kind = p_kind
	skill.level = clampi(p_level, MIN_LEVEL, MAX_LEVEL)
	skill.blocks = p_blocks
	skill.description = p_description
	return skill

# The family this skill belongs to (its own id when it stands alone).
func family_id() -> String:
	return id if family.is_empty() else family

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

func keeps_stance() -> bool:
	return kind == Kind.MOVE or kind == Kind.TARGET or kind == Kind.STATUS or (kind == Kind.SUPPORT and stance_override < 0)

func stance_after(current: CombatTypes.StanceType) -> CombatTypes.StanceType:
	return current if keeps_stance() else stance()

# Automatic stance-switch penalty (AP): changing stance costs 1; Move and Rest never do.
func switch_cost(current: CombatTypes.StanceType) -> int:
	if kind == Kind.REST or keeps_stance():
		return 0
	return 0 if current == stance() else 1

# True when a SUPPORT skill heals.
func heals() -> bool:
	return kind == Kind.SUPPORT and (heal_base > 0 or heal_mult > 0.0)

# True when the skill must be aimed at a friendly unit other than the caster (or may be).
func targets_ally() -> bool:
	return kind == Kind.SUPPORT and target_mode != TargetMode.SELF

# Distance band at which a SUPPORT skill reaches its target (0 = the caster itself).
func support_reach(c: Combatant) -> Vector2i:
	if reach_max > 0:
		return Vector2i(reach_min, reach_max)
	return Vector2i(0, maxi(SUPPORT_REACH, c.attack_range))

func icon() -> String:
	if kind == Kind.SUPPORT:
		return "+"
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
