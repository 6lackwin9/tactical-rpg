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
#   Move keeps whatever stance the unit is in; Rest puts it in Tactical.
# Using a skill from another stance switches stance automatically for
# CombatTypes.STANCE_SWITCH_COST AP (no block cost). Rest enters Tactical for free.
# ==============================================================================

# SUPPORT: heals and/or grants a buff status to itself or a friendly unit (SupportRules,
# SupportCommand). It keeps the unit's stance like Move does (no switch cost).
#
# DISENGAGE (M2c-2a2): a jump of up to `jump_tiles` tiles AWAY from the nearest (or a chosen) enemy.
# It is the unit's move for the round (first in the sequence, refused when rooted), costs AP = the
# skill's level (not per tile), leaps over units but not walls, and keeps the stance like Move.
enum Kind { ATTACK, DEFEND, FOCUS, MOVE, REST, TARGET, STATUS, SUPPORT, DISENGAGE }

# Which hand's weapon an ATTACK uses: the main weapon, the off-hand weapon (or bare hands), or both (dual strike, engine pending).
enum WeaponHand { MAIN, OFF, BOTH }
const BOTH_ENABLED: bool = false   # `both` is rejected by validation while the engine cannot play it

# Skill hierarchy (FORGE1 11): a family of a higher grade can require families of a lower grade. `expert` is reserved.
enum Grade { NOVICE, ADVANCED, EXPERT }

# `weapon_types` token for "no main-hand weapon equipped" (bare hands). Its proficiency is the
# class's UNARMED_PROFICIENCY gear level (`fist`), so min_proficiency applies to it as well.
const UNARMED_TOKEN: String = "unarmed"
const UNARMED_PROFICIENCY: String = "fist"

# Every valid `weapon_types` entry: the gear types plus the unarmed token.
static func weapon_type_tokens() -> Array[String]:
	var out: Array[String] = Proficiency.gear_types()
	out.append(UNARMED_TOKEN)
	return out

# SUPPORT targeting: the caster only, a friendly unit other than the caster, or either.
enum TargetMode { SELF, ALLY, ALLY_OR_SELF }

# Area attacks target a tile: PLUS = tile + 4 neighbours, BURST = 3x3, LINE = 3 tiles
# straight out from the caster (aim at an adjacent tile).
# SWEEP (Brawler): an L of three tiles around the caster, (1,0) (1,1) (0,1) rotated into the quadrant
# of the aim tile, which is a DIAGONAL neighbour of the caster.
enum Area { NONE, PLUS, BURST, LINE, SWEEP }

const CLEANSE_ALL: String = "all"   # keyword in cleanse_statuses: every debuff

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
var weapon_hand: WeaponHand = WeaponHand.MAIN   # ATTACK: which hand's weapon the strike uses (FORGE1 4.1; see is_off_hand)
var stance_override: int = -1  # ATTACK in another stance pool (e.g. Shield Bash is Defensive)
var status_effect: String = "" # status rolled on hit (StatusEffects), active next round
var status_potency: int = 0
var status_magnitude: int = 0   # weaken only: percent the inflicted instance carries (not rolled; 1-25)
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
var cleanse_statuses: Array[String] = []   # SUPPORT: debuffs removed from the target, active and pending (CLEANSE_ALL = every debuff; empty = none)
var jump_tiles: int = 0         # DISENGAGE: how many tiles the jump covers at most
var learned_only: bool = false  # unrestricted (hand-built) units don't know it unless they learn it
var family: String = ""         # skill family (tiers of one skill); empty = its own id (see family_id())
var tier: int = 1               # position in the family (1..5); each tier is its own skill with its own AP, blocks and power
var basic: bool = false         # available to every class (counted outside the loadout slots)
var grade: Grade = Grade.NOVICE   # family-level: class-path steps of advanced+ need StatRules.ADVANCED_MIN_LEVEL, KNOW can open them earlier
var requires: Array[String] = []  # family ids that must ALL be owned to own / learn this skill (each of a lower grade)
var unlock_offset: int = -1     # levels after the family's unlock level this row opens (-1 = derive from the tier, see tier_offset)
var description: String = ""
var template: String = ""        # the description template this rank's text was rendered from (SkillText); "" = a literal description
var origin: String = "system"    # "system" (shipped) or "user" (ContentSchema.ORIGIN_*); informational for the engine
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

# Levels after the family's unlock level at which this row opens: the authored unlock_offset, else
# StatRules.TIER_UNLOCK_STEP per tier above 1 (tiers 1 / 2 / 3 -> 0 / 2 / 4).
func tier_offset() -> int:
	if unlock_offset >= 0:
		return unlock_offset
	return StatRules.TIER_UNLOCK_STEP * maxi(0, tier - 1)

# The family this skill belongs to (its own id when it stands alone).
func family_id() -> String:
	return id if family.is_empty() else family

# power_mult as `user` applies it: the skill's multiplier x the user's family potency (Combatant.potency_for)
# x the user's weaken factor (Combatant.weaken_factor, 1.0 when not weakened).
# This is the ONLY place potency and weaken enter attack damage (the value fed to DamageCalculator as skill_mult).
func power_for(user: Combatant) -> float:
	return power_mult if user == null else power_mult * user.potency_for_skill(self) * user.weaken_factor()

# False for a zero-power ATTACK (a debuff-only attack): it deals no damage and is never a hit for reactions.
func deals_damage() -> bool:
	return kind != Kind.ATTACK or power_mult > 0.0

# Base AP = level; effects on the unit shift it (never below 1 or above 5).
# Move: the cost PER TILE (1 + shift). Rest: always free.
func ap_cost(actor: Combatant = null) -> int:
	var ap_rule: String = str(SkillKinds.info(kind_name()).get("ap", "LEVEL"))
	if ap_rule == "FREE":
		return 0
	var shift: int = actor.ap_cost_modifier if actor != null else 0
	var base: int = 1 if ap_rule == "PER_TILE" else level
	return clampi(base + shift, MIN_LEVEL, MAX_LEVEL)

# The kind as the SkillKinds table key ("ATTACK").
func kind_name() -> String:
	return str(Kind.keys()[kind])

# The stance this skill puts the unit in (Move keeps the current one; see stance_after).
func stance() -> CombatTypes.StanceType:
	if stance_override >= 0:
		return stance_override as CombatTypes.StanceType
	match str(SkillKinds.info(kind_name()).get("stance", "NEUTRAL")):
		"OFFENSIVE":
			return CombatTypes.StanceType.OFFENSIVE
		"DEFENSIVE":
			return CombatTypes.StanceType.DEFENSIVE
		"TACTICAL":
			return CombatTypes.StanceType.TACTICAL
	return CombatTypes.StanceType.NEUTRAL

func keeps_stance() -> bool:
	var rule: String = str(SkillKinds.info(kind_name()).get("stance", ""))
	return rule == "KEEP" or (rule == "KEEP_UNLESS_SET" and stance_override < 0)

func stance_after(current: CombatTypes.StanceType) -> CombatTypes.StanceType:
	return current if keeps_stance() else stance()

# Automatic stance-switch penalty (AP): changing stance costs STANCE_SWITCH_COST; Move and Rest never pay (Rest enters Tactical for free).
func switch_cost(current: CombatTypes.StanceType) -> int:
	if kind == Kind.REST or keeps_stance():
		return 0
	return 0 if current == stance() else CombatTypes.STANCE_SWITCH_COST

# True when a SUPPORT skill heals.
func heals() -> bool:
	return kind == Kind.SUPPORT and (heal_base > 0 or heal_mult > 0.0)

# Main-hand gear types that open spell skills (the item family engine rule `spell_source`).
const SPELL_SOURCE_TYPES: Array[String] = ["staff", "wand"]

# Skill groups (DERIVED, nothing stored; see Passives.GROUPS): "spell" = an ATTACK skill gated on a staff or wand,
# "heal" = a SUPPORT skill that heals. Item / passive group bonuses apply to every skill in the group.
func in_group(group: String) -> bool:
	match group:
		"spell":
			if kind != Kind.ATTACK:
				return false
			for source: String in SPELL_SOURCE_TYPES:
				if weapon_types.has(source):
					return true
			return false
		"heal":
			return heals()
	return false

# True when a SUPPORT skill removes debuffs.
func cleanses() -> bool:
	return kind == Kind.SUPPORT and not cleanse_statuses.is_empty()

# The debuff ids this skill removes (the keyword expands to every debuff), in StatusEffects.debuffs() order.
func cleanse_list() -> Array[String]:
	var out: Array[String] = []
	if kind != Kind.SUPPORT:
		return out
	for status: String in StatusEffects.debuffs():
		if cleanse_statuses.has(CLEANSE_ALL) or cleanse_statuses.has(status):
			out.append(status)
	return out

# True when the skill must be aimed at a friendly unit other than the caster (or may be).
func targets_ally() -> bool:
	return kind == Kind.SUPPORT and target_mode != TargetMode.SELF

# Distance band at which a SUPPORT skill reaches its target (0 = the caster itself).
func support_reach(c: Combatant) -> Vector2i:
	if reach_max > 0:
		return Vector2i(reach_min, reach_max)
	return Vector2i(0, maxi(SUPPORT_REACH, c.attack_range))

func icon() -> String:
	return SkillKinds.icon_of(kind_name())

# Distance band (min, max) at which `c` can use this attack: skill-specific if set;
# off-hand jabs and line areas 1; ranged units 2..3 (or their longer range); melee 1..range.
func reach_for(c: Combatant) -> Vector2i:
	var extra: int = c.spell_reach_bonus() if in_group("spell") and area != Area.LINE and area != Area.SWEEP else 0   # spell_reach effects (not LINE / SWEEP)
	if reach_max > 0:
		return Vector2i(reach_min, reach_max + extra)
	if is_off_hand() or area == Area.LINE or area == Area.SWEEP:
		return Vector2i(1, 1)
	if c.attack_range > 1:
		return Vector2i(2, maxi(3, c.attack_range) + extra)
	return Vector2i(1, c.attack_range + extra)

func is_area() -> bool:
	return area != Area.NONE

# True when the strike swings the off-hand weapon (weapon_hand OFF, e.g. the off-hand jab).
func is_off_hand() -> bool:
	return weapon_hand == WeaponHand.OFF

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
		Area.SWEEP:
			var quadrant := Vector2i(signi(cell.x - from.x), signi(cell.y - from.y))
			cells = [from + Vector2i(quadrant.x, 0), from + quadrant, from + Vector2i(0, quadrant.y)]
		_:
			cells = [cell]
	return cells

func label() -> String:
	return "%s %s" % [icon(), display_name]
