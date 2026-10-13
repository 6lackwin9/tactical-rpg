class_name StatusEffects
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Effects (round-long conditions that shape the next plan)
# Statuses never interrupt the current round: anything inflicted this round
# becomes active at the start of the NEXT round, for that one round.
#   STAGGER  blocks 1 and 4 can't be used (shown as Staggered blocks)
#   STUN     blocks 1-2 can't be used (the unit acts from block 3)
#   ROOT     the unit can't move this round
# Sources: skills (rolled against the target's resistance) and AP debt
# (overspending: 3+ debt staggers, 5+ stuns; no resist roll, it's self-inflicted).
# ==============================================================================

const STAGGER: String = "stagger"
const STUN: String = "stun"
const ROOT: String = "root"

const DEBT_STAGGER: int = 3
const DEBT_STUN: int = 5

static func display_name(status: String) -> String:
	match status:
		STAGGER:
			return "Staggered"
		STUN:
			return "Stunned"
		ROOT:
			return "Rooted"
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
static func try_inflict(target: Combatant, status: String, potency: int, rng: RandomNumberGenerator) -> bool:
	if target == null or not target.is_alive():
		return false
	var chance: int = apply_chance(target, status, potency)
	var roll: int = rng.randi_range(1, 100) if rng != null else 100
	if roll > chance:
		return false
	target.pending_statuses[status] = true
	return true

# Next-round penalty for overspending: "" or the status it causes.
static func debt_status(debt: int) -> String:
	if debt >= DEBT_STUN:
		return STUN
	if debt >= DEBT_STAGGER:
		return STAGGER
	return ""

# Round start: last round's statuses expire, queued ones and AP-debt penalties
# take effect, then AP recovers (debt eats into the recovery). Returns the
# statuses now active, for the log.
static func roll_over(c: Combatant, ap_regen: int) -> Array[String]:
	c.statuses.clear()
	for status: Variant in c.pending_statuses:
		c.statuses[str(status)] = true
	c.pending_statuses.clear()
	var debt_penalty: String = debt_status(maxi(0, -c.ap))
	if not debt_penalty.is_empty():
		c.statuses[debt_penalty] = true
	c.ap = c.max_ap if ap_regen < 0 else mini(c.max_ap, c.ap + ap_regen)
	var active: Array[String] = []
	for status: Variant in c.statuses:
		active.append(str(status))
	return active
