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
#   WEAKEN   (debuff) the unit's outgoing skill power (SkillDef.power_for, riposte counter) is cut by the
#            instance's magnitude percent (Combatant.weaken_factor); magnitude capped at 25 by the registry
#   HASTE    (buff) +HASTE_TILES tiles per move block while active
#   TAUNT    (buff on the taunter) enemy AI units that can reach it must target it
#   RIPOSTE  (buff, DEFEND skill) counters adjacent melee attackers (RiposteRules). Unlike the
#            queued statuses it is active IMMEDIATELY (from the block the skill starts, see
#            grant_now) and is removed by any stance change (Combatant.set_stance)
#   SLOW, EXPOSE, FORTIFY, REGEN, POISON, BURN (S12S-L): registry rows with magnitude ranges; their effects
#            (tiles per block, armor rating, round-end tick) land in S12S-L commit 2
# Magnitudes: a status carries one when its registry row has magnitude_max > 0 (has_magnitude).
# Sources: Combatant.status_sources / pending_status_sources hold the unit name of the last applier.
# Debuffs come from attack skills (rolled against resistance) and AP debt
# (overspending: 3+ debt staggers, 5+ stuns; no resist roll, it's self-inflicted).
# Buffs come from SUPPORT skills (SupportCommand) and skip resistance.
# The list of statuses, their kind, resistability, default rounds, stacking rule, cap and display
# name lives in StatusRegistry (the single table; the lists below derive from it, and its header
# explains how to add a status).
# ==============================================================================

const STAGGER: String = "stagger"
const STUN: String = "stun"
const ROOT: String = "root"
const BLEED: String = "bleed"
const HASTE: String = "haste"
const TAUNT: String = "taunt"
const RIPOSTE: String = "riposte"
const WEAKEN: String = "weaken"
const SLOW: String = "slow"
const EXPOSE: String = "expose"
const FORTIFY: String = "fortify"
const REGEN: String = "regen"
const POISON: String = "poison"
const BURN: String = "burn"

const BLEED_PCT: float = 0.06    # of max HP per round while bleeding (minimum 1)
const HASTE_TILES: int = 1       # extra tiles per move block while hasted
const SLOW_TILES: int = 1        # fewer tiles per move block while slowed (structural mirror of HASTE_TILES)

const DEBT_STAGGER: int = 3
const DEBT_STUN: int = 5

# Statuses an attack skill can inflict on a foe (rolled against resistance).
static func debuffs() -> Array[String]:
	return StatusRegistry.ids_where(StatusRegistry.KIND_DEBUFF)

# Every buff status (no resistance roll).
static func buffs() -> Array[String]:
	return StatusRegistry.ids_where(StatusRegistry.KIND_BUFF)

# Buffs a SUPPORT skill can grant (queued for next round).
static func support_buffs() -> Array[String]:
	return StatusRegistry.ids_where(StatusRegistry.KIND_BUFF, false, "support")

# Buffs a DEFEND skill can grant (active immediately).
static func defend_buffs() -> Array[String]:
	return StatusRegistry.ids_where(StatusRegistry.KIND_BUFF, false, "defend")

# Valid keys of a unit's / passive's status_resist.
static func resistable() -> Array[String]:
	return StatusRegistry.ids_where(StatusRegistry.KIND_DEBUFF, true)

# Applies the status's stacking rule to an existing and a new application: {"rounds", "potency"}.
# See StatusRegistry.stack; every status is refresh_max_duration today.
static func stack(existing_rounds: int, existing_potency: int, new_rounds: int, new_potency: int, status: String) -> Dictionary:
	return StatusRegistry.stack(existing_rounds, existing_potency, new_rounds, new_potency, status)

# Damage a bleeding unit takes at the end of the round (a flat share of max HP).
static func bleed_damage(c: Combatant) -> int:
	return maxi(1, roundi(float(c.max_hp) * BLEED_PCT))

static func display_name(status: String) -> String:
	var shown: String = str(StatusRegistry.row(status).get("display", ""))
	return shown if not shown.is_empty() else status.capitalize()

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
	var resist: float = target.debuff_resistance() + float(target.resist_for(status))
	return clampi(roundi(float(potency) - resist), 0, 100)

# Rolls a skill-inflicted status. On success it is queued for the target's next round.
# `source` is the unit name of the applier (kept in Combatant.pending_status_sources).
static func try_inflict(target: Combatant, status: String, potency: int, rng: RandomNumberGenerator, rounds: int = 1, magnitude: int = 0, source: String = "") -> bool:
	if target == null or not target.is_alive():
		return false
	var chance: int = apply_chance(target, status, potency)
	var roll: int = rng.randi_range(1, 100) if rng != null else 100
	if roll > chance:
		return false
	queue(target, status, rounds, magnitude, source)
	return true

# True for statuses whose instance carries a magnitude next to its rounds (registry magnitude_max > 0).
static func has_magnitude(status: String) -> bool:
	return StatusRegistry.magnitude_max(status) > 0

# Queues a status for the target's next round, lasting `rounds` rounds (the longer wins).
# `magnitude` is stored only for statuses that carry one; a repeat follows the registry stack rule and cap.
# `source` (unit name; "" = unknown) replaces the queued instance's source when given (the newer applier wins).
static func queue(target: Combatant, status: String, rounds: int = 1, magnitude: int = 0, source: String = "") -> void:
	target.pending_statuses[status] = true
	var existing_rounds: int = DictUtil.int_of(target.pending_status_rounds, status)
	var combined: Dictionary = stack(existing_rounds, DictUtil.int_of(target.pending_status_magnitudes, status), rounds, magnitude, status)
	target.pending_status_rounds[status] = DictUtil.to_int(combined["rounds"])
	if has_magnitude(status):
		target.pending_status_magnitudes[status] = DictUtil.to_int(combined["potency"])
	if not source.is_empty():
		target.pending_status_sources[status] = source

# Makes a buff active right now (self-buffs from DEFEND skills), lasting `rounds` rounds
# counting the current one (the longer wins). `magnitude` and `source` as in queue().
static func grant_now(target: Combatant, status: String, rounds: int = 1, magnitude: int = 0, source: String = "") -> void:
	var combined: Dictionary = stack(DictUtil.int_of(target.status_rounds_left, status), DictUtil.int_of(target.status_magnitudes, status), rounds, magnitude, status)
	target.statuses[status] = true
	target.status_rounds_left[status] = DictUtil.to_int(combined["rounds"])
	if has_magnitude(status):
		target.status_magnitudes[status] = DictUtil.to_int(combined["potency"])
	if not source.is_empty():
		target.status_sources[status] = source

# True when `status` is active now or queued for next round.
static func has_any(target: Combatant, status: String) -> bool:
	return target.statuses.has(status) or target.pending_statuses.has(status)

# Removes `status` from the target, active and pending alike. Returns true if anything was removed.
static func remove(target: Combatant, status: String) -> bool:
	var found: bool = has_any(target, status)
	target.statuses.erase(status)
	target.status_rounds_left.erase(status)
	target.pending_statuses.erase(status)
	target.pending_status_rounds.erase(status)
	target.status_magnitudes.erase(status)
	target.pending_status_magnitudes.erase(status)
	target.status_sources.erase(status)
	target.pending_status_sources.erase(status)
	return found

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
	var old_magnitudes: Dictionary = c.status_magnitudes.duplicate()
	var old_sources: Dictionary = c.status_sources.duplicate()
	c.statuses.clear()
	c.status_rounds_left.clear()
	c.status_magnitudes.clear()
	c.status_sources.clear()
	for status: Variant in carried:
		c.statuses[str(status)] = true
		c.status_rounds_left[str(status)] = carried[status]
		if old_magnitudes.has(status):
			c.status_magnitudes[str(status)] = old_magnitudes[status]
		if old_sources.has(status):
			c.status_sources[str(status)] = old_sources[status]
	for status: Variant in c.pending_statuses:
		var queued_rounds: int = DictUtil.int_of(c.pending_status_rounds, status, 1)
		if has_magnitude(str(status)):
			var merged: Dictionary = stack(DictUtil.int_of(c.status_rounds_left, status), DictUtil.int_of(c.status_magnitudes, status), queued_rounds, DictUtil.int_of(c.pending_status_magnitudes, status), str(status))
			c.status_magnitudes[str(status)] = DictUtil.to_int(merged["potency"])
		c.statuses[str(status)] = true
		c.status_rounds_left[str(status)] = maxi(DictUtil.int_of(c.status_rounds_left, status), queued_rounds)
		if c.pending_status_sources.has(status):
			c.status_sources[str(status)] = c.pending_status_sources[status]   # the newer source wins
	c.pending_statuses.clear()
	c.pending_status_rounds.clear()
	c.pending_status_magnitudes.clear()
	c.pending_status_sources.clear()
	c.riposted.clear()
	var debt_penalty: String = debt_status(maxi(0, -c.ap))
	if not debt_penalty.is_empty():
		c.statuses[debt_penalty] = true
	c.ap = c.max_ap if ap_regen < 0 else mini(c.max_ap, c.ap + ap_regen + c.ap_regen_bonus)
	var active: Array[String] = []
	for status: Variant in c.statuses:
		active.append(str(status))
	return active
