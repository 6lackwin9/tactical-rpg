class_name StatusEffects
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Effects (round-long conditions that shape the next plan)
# Statuses never interrupt the current round: anything inflicted this round
# becomes active at the start of the NEXT round, for that one round (or for the
# `rounds` the skill grants: Combatant.status_rounds_left counts them down).
#   STAGGER  blocks 1 and 4 can't be used (shown as Staggered blocks)
#   STUN     blocks 1-2 can't be used (the unit acts from block 3)
#   ROOT     the unit can't move this round
#   BLEED    (debuff) loses BLEED_PCT of max HP at the end of the round; ignores guard and armor
#   HASTE    (buff) +HASTE_TILES tiles per move block while active
#   TAUNT    (buff on the taunter) enemy AI units that can reach it must target it
#   RIPOSTE  (buff, DEFEND skill) counters adjacent melee attackers (RiposteRules). Unlike the
#            queued statuses it is active IMMEDIATELY (from the block the skill starts, see
#            grant_now) and is removed by any stance change (Combatant.set_stance)
# Debuffs come from attack skills (rolled against resistance) and AP debt
# (overspending: 3+ debt staggers, 5+ stuns; no resist roll, it's self-inflicted).
# Buffs come from SUPPORT skills (SupportCommand) and skip resistance.
# ==============================================================================

const STAGGER: String = "stagger"
const STUN: String = "stun"
const ROOT: String = "root"
const BLEED: String = "bleed"
const HASTE: String = "haste"
const TAUNT: String = "taunt"
const RIPOSTE: String = "riposte"

const BLEED_PCT: float = 0.06    # of max HP per round while bleeding (minimum 1)
const HASTE_TILES: int = 1       # extra tiles per move block while hasted

const DEBT_STAGGER: int = 3
const DEBT_STUN: int = 5

# Statuses an attack skill can inflict on a foe (rolled against resistance).
static func debuffs() -> Array[String]:
	return [STAGGER, STUN, ROOT, BLEED]

# Every buff status (no resistance roll).
static func buffs() -> Array[String]:
	return [HASTE, TAUNT, RIPOSTE]

# Buffs a SUPPORT skill can grant (queued for next round).
static func support_buffs() -> Array[String]:
	return [HASTE, TAUNT]

# Buffs a DEFEND skill can grant (active immediately).
static func defend_buffs() -> Array[String]:
	return [RIPOSTE]

# Valid keys of a unit's / passive's status_resist.
static func resistable() -> Array[String]:
	return debuffs()

# Damage a bleeding unit takes at the end of the round (a flat share of max HP).
static func bleed_damage(c: Combatant) -> int:
	return maxi(1, roundi(float(c.max_hp) * BLEED_PCT))

static func display_name(status: String) -> String:
	match status:
		STAGGER:
			return "Staggered"
		STUN:
			return "Stunned"
		ROOT:
			return "Rooted"
		BLEED:
			return "Bleeding"
		HASTE:
			return "Hasted"
		TAUNT:
			return "Taunting"
		RIPOSTE:
			return "Riposte"
	return status.capitalize()

# Blocks the status takes away from a round.
static func blocked_blocks(status: String) -> Array[int]:
	match status:
		STAGGER:
			return [1, 4]
		STUN:
			return [1, 2]
	var none: Array[int] = []
	return none

# Chance (0-100) that `potency` lands on `target`: potency minus the target's
# debuff resistance and any passive resistance to this status (100+ = immune).
static func apply_chance(target: Combatant, status: String, potency: int) -> int:
	var resist: float = target.debuff_resistance() + float(DictUtil.int_of(target.status_resist, status))
	return clampi(roundi(float(potency) - resist), 0, 100)

# Rolls a skill-inflicted status. On success it is queued for the target's next round.
static func try_inflict(target: Combatant, status: String, potency: int, rng: RandomNumberGenerator, rounds: int = 1) -> bool:
	if target == null or not target.is_alive():
		return false
	var chance: int = apply_chance(target, status, potency)
	var roll: int = rng.randi_range(1, 100) if rng != null else 100
	if roll > chance:
		return false
	queue(target, status, rounds)
	return true

# Queues a status for the target's next round, lasting `rounds` rounds (the longer wins).
static func queue(target: Combatant, status: String, rounds: int = 1) -> void:
	target.pending_statuses[status] = true
	target.pending_status_rounds[status] = maxi(DictUtil.int_of(target.pending_status_rounds, status), maxi(1, rounds))

# Makes a buff active right now (self-buffs from DEFEND skills), lasting `rounds` rounds
# counting the current one (the longer wins).
static func grant_now(target: Combatant, status: String, rounds: int = 1) -> void:
	target.statuses[status] = true
	target.status_rounds_left[status] = maxi(DictUtil.int_of(target.status_rounds_left, status), maxi(1, rounds))

# Next-round penalty for overspending: "" or the status it causes.
static func debt_status(debt: int) -> String:
	if debt >= DEBT_STUN:
		return STUN
	if debt >= DEBT_STAGGER:
		return STAGGER
	return ""

# Round start: last round's statuses expire (multi-round ones count down), queued ones
# and AP-debt penalties take effect, then AP recovers (debt eats into the recovery).
# Returns the statuses now active, for the log.
static func roll_over(c: Combatant, ap_regen: int) -> Array[String]:
	var carried: Dictionary = {}   # multi-round statuses with rounds left after the one that ended
	for status: Variant in c.status_rounds_left:
		var left: int = DictUtil.int_of(c.status_rounds_left, status) - 1
		if left > 0:
			carried[str(status)] = left
	c.statuses.clear()
	c.status_rounds_left.clear()
	for status: Variant in carried:
		c.statuses[str(status)] = true
		c.status_rounds_left[str(status)] = carried[status]
	for status: Variant in c.pending_statuses:
		c.statuses[str(status)] = true
		c.status_rounds_left[str(status)] = maxi(DictUtil.int_of(c.status_rounds_left, status), DictUtil.int_of(c.pending_status_rounds, status, 1))
	c.pending_statuses.clear()
	c.pending_status_rounds.clear()
	c.riposted.clear()
	var debt_penalty: String = debt_status(maxi(0, -c.ap))
	if not debt_penalty.is_empty():
		c.statuses[debt_penalty] = true
	c.ap = c.max_ap if ap_regen < 0 else mini(c.max_ap, c.ap + ap_regen + c.ap_regen_bonus)
	var active: Array[String] = []
	for status: Variant in c.statuses:
		active.append(str(status))
	return active
