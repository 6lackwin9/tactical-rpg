---
class: StatusEffects
extends: RefCounted
description: "Tactical RPG - Status Effects (round-long conditions that shape the next plan) Statuses never interrupt the current round: anything inflicted this round becomes active at the start of the NEXT round, for that one round (or for the `rounds` the skill grants: Combatant.status_rounds_left counts them down). STAGGER  blocks 1 and 4 can't be used (shown as Staggered blocks) STUN     blocks 1-2 can't be used (the unit acts from block 3) ROOT     the unit can't move this round BLEED    (debuff) loses BLEED_PCT of max HP at the end of the round; ignores guard and armor WEAKEN   (debuff) the unit's outgoing skill power (SkillDef.power_for, riposte counter) is cut by the instance's magnitude percent (Combatant.weaken_factor); magnitude capped at 25 by the registry HASTE    (buff) +HASTE_TILES tiles per move block while active TAUNT    (buff on the taunter) enemy AI units that can reach it must target it RIPOSTE  (buff, DEFEND skill) counters adjacent melee attackers (RiposteRules). Unlike the queued statuses it is active IMMEDIATELY (from the block the skill starts, see grant_now) and is removed by any stance change (Combatant.set_stance) SLOW     (debuff) -SLOW_TILES tiles per move block (floor 1; cancels haste), Combatant.move_per_block EXPOSE   (debuff) -magnitude armor rating (floor 0), DamageCalculator.armor_rating FORTIFY  (buff, support/DEFEND grant) +magnitude armor rating POISON   (debuff, stack_potency: magnitudes add up to the cap) loses the magnitude in HP at round end BURN     (debuff) loses magnitude % of max HP at round end (minimum 1) REGEN    (buff) regains magnitude % of max HP at round end (minimum 1, capped at missing HP) Round end order per unit (CombatResolver.apply_round_end_effects): bleed -> poison -> burn -> regen. Magnitudes: a status carries one when its registry row has magnitude_max > 0 (has_magnitude). Sources: Combatant.status_sources / pending_status_sources hold the unit name of the last applier. Debuffs come from attack skills (rolled against resistance) and AP debt (overspending: 3+ debt staggers, 5+ stuns; no resist roll, it's self-inflicted). Buffs come from SUPPORT skills (SupportCommand) and skip resistance. The list of statuses, their kind, resistability, default rounds, stacking rule, cap and display name lives in StatusRegistry (the single table; the lists below derive from it, and its header explains how to add a status)."
---

# Properties
* `STAGGER`: String = "stagger" (const)
* `STUN`: String = "stun" (const)
* `ROOT`: String = "root" (const)
* `BLEED`: String = "bleed" (const)
* `HASTE`: String = "haste" (const)
* `TAUNT`: String = "taunt" (const)
* `RIPOSTE`: String = "riposte" (const)
* `WEAKEN`: String = "weaken" (const)
* `SLOW`: String = "slow" (const)
* `EXPOSE`: String = "expose" (const)
* `FORTIFY`: String = "fortify" (const)
* `REGEN`: String = "regen" (const)
* `POISON`: String = "poison" (const)
* `BURN`: String = "burn" (const)
* `BLEED_PCT`: float = 0.06    # of max HP per round while bleeding (minimum 1) (const)
* `HASTE_TILES`: int = 1       # extra tiles per move block while hasted (const)
* `SLOW_TILES`: int = 1        # fewer tiles per move block while slowed (structural mirror of HASTE_TILES) (const)
* `DEBT_STAGGER`: int = 3 (const)
* `DEBT_STUN`: int = 5 (const)
* `none`: Array[int] = []
* `carried`: Dictionary = {}   # multi-round statuses with rounds left after the one that ended
* `left`: int = DictUtil.int_of(c.status_rounds_left, status) - 1
* `old_magnitudes`: Dictionary = c.status_magnitudes.duplicate()
* `old_sources`: Dictionary = c.status_sources.duplicate()
* `queued_rounds`: int = DictUtil.int_of(c.pending_status_rounds, status, 1)
* `merged`: Dictionary = stack(DictUtil.int_of(c.status_rounds_left, status), DictUtil.int_of(c.status_magnitudes, status), queued_rounds, DictUtil.int_of(c.pending_status_magnitudes, status), str(status))
* `debt_penalty`: String = debt_status(maxi(0, -c.ap))
* `active`: Array[String] = []

# Methods

### `static stack(existing_rounds: int, existing_potency: int, new_rounds: int, new_potency: int, status: String) -> Dictionary`
```logic
return StatusRegistry.stack(existing_rounds, existing_potency, new_rounds, new_potency, status)
```

### `static bleed_damage(c: Combatant) -> int`
```logic
return maxi(1, roundi(float(c.max_hp) * BLEED_PCT))
```

### `static poison_damage(c: Combatant) -> int`
```logic
return c.active_magnitude(POISON)
```

### `static burn_damage(c: Combatant) -> int`
```logic
return maxi(1, roundi(float(c.max_hp) * float(c.active_magnitude(BURN)) / 100.0))
```

### `static regen_amount(c: Combatant) -> int`
```logic
return maxi(1, roundi(float(c.max_hp) * float(c.active_magnitude(REGEN)) / 100.0))
```

### `static display_name(status: String) -> String`
```logic
var shown: String = str(StatusRegistry.row(status).get("display", ""))
return shown if not shown.is_empty() else status.capitalize()
```

### `static label_with_magnitude(status: String, magnitude: int) -> String`
```logic
var shown: String = SkillText.format_magnitude(status, magnitude)
return display_name(status) if shown.is_empty() else "%s %s" % [display_name(status), shown]
```

### `static chip_text(c: Combatant, status: String) -> String`
```logic
return label_with_magnitude(status, DictUtil.int_of(c.status_magnitudes, status))
```

### `static apply_chance(target: Combatant, status: String, potency: int) -> int`
```logic
var resist: float = target.debuff_resistance() + float(target.resist_for(status))
return clampi(roundi(float(potency) - resist), 0, 100)
```

### `static try_inflict(target: Combatant, status: String, potency: int, rng: RandomNumberGenerator, rounds: int = 1, magnitude: int = 0, source: String = "") -> bool`
```logic
if target == null or not target.is_alive():
	return false
var chance: int = apply_chance(target, status, potency)
var roll: int = rng.randi_range(1, 100) if rng != null else 100
if roll > chance:
	return false
queue(target, status, rounds, magnitude, source)
return true
```

### `static has_magnitude(status: String) -> bool`
```logic
return StatusRegistry.magnitude_max(status) > 0
```

### `static magnitude_or_min(status: String, magnitude: int) -> int`
```logic
if magnitude <= 0 and has_magnitude(status):
	return StatusRegistry.magnitude_min(status)
return magnitude
```

### `static queue(target: Combatant, status: String, rounds: int = 1, magnitude: int = 0, source: String = "") -> void`
```logic
magnitude = magnitude_or_min(status, magnitude)
target.pending_statuses[status] = true
var existing_rounds: int = DictUtil.int_of(target.pending_status_rounds, status)
var combined: Dictionary = stack(existing_rounds, DictUtil.int_of(target.pending_status_magnitudes, status), rounds, magnitude, status)
target.pending_status_rounds[status] = DictUtil.to_int(combined["rounds"])
if has_magnitude(status):
	target.pending_status_magnitudes[status] = DictUtil.to_int(combined["potency"])
if not source.is_empty():
	target.pending_status_sources[status] = source
```

### `static grant_now(target: Combatant, status: String, rounds: int = 1, magnitude: int = 0, source: String = "") -> void`
```logic
magnitude = magnitude_or_min(status, magnitude)
var combined: Dictionary = stack(DictUtil.int_of(target.status_rounds_left, status), DictUtil.int_of(target.status_magnitudes, status), rounds, magnitude, status)
target.statuses[status] = true
target.status_rounds_left[status] = DictUtil.to_int(combined["rounds"])
if has_magnitude(status):
	target.status_magnitudes[status] = DictUtil.to_int(combined["potency"])
if not source.is_empty():
	target.status_sources[status] = source
```

### `static has_any(target: Combatant, status: String) -> bool`
```logic
return target.statuses.has(status) or target.pending_statuses.has(status)
```

### `static remove(target: Combatant, status: String) -> bool`
```logic
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
```

### `static remove_active(target: Combatant, status: String) -> bool`
```logic
var found: bool = target.statuses.has(status)
target.statuses.erase(status)
target.status_rounds_left.erase(status)
target.status_magnitudes.erase(status)
target.status_sources.erase(status)
return found
```

### `static debt_status(debt: int) -> String`
```logic
if debt >= DEBT_STUN:
	return STUN
if debt >= DEBT_STAGGER:
	return STAGGER
return ""
```

