---
class: CombatTypes
extends: RefCounted
description: "Three stances in a rock-paper-scissors loop, plus Neutral (no advantage and no exposure; the home stance for moving, retargeting and resting). Offensive beats Tactical (punishes preparation) Tactical  beats Defensive (bypasses a static guard) Defensive beats Offensive (absorbs and counters)"
---

# Properties
* `STANCE_SWITCH_COST`: int = 1   # AP paid when a skill changes the unit's stance (Move/Rest/keepers never pay) (const)
* `STANCE_ADVANTAGE`: float = 1.25 (const)
* `STANCE_DISADVANTAGE`: float = 0.75 (const)

# Methods

### `static stance_beats(attacker: StanceType, defender: StanceType) -> bool`
```logic
return (attacker == StanceType.OFFENSIVE and defender == StanceType.TACTICAL) \
	or (attacker == StanceType.TACTICAL and defender == StanceType.DEFENSIVE) \
	or (attacker == StanceType.DEFENSIVE and defender == StanceType.OFFENSIVE)
```

### `static get_matchup_multiplier(attacker_stance: StanceType, defender_stance: StanceType) -> float`
```logic
if stance_beats(attacker_stance, defender_stance):
	return STANCE_ADVANTAGE
if stance_beats(defender_stance, attacker_stance):
	return STANCE_DISADVANTAGE
return 1.0
```

### `static stance_to_string(stance: StanceType) -> String`
```logic
match stance:
	StanceType.NEUTRAL: return "Neutral"
	StanceType.OFFENSIVE: return "Offensive"
	StanceType.DEFENSIVE: return "Defensive"
	StanceType.TACTICAL: return "Tactical"
	_: return "Unknown"
```

