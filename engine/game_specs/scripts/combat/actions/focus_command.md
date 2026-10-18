---
class: FocusCommand
extends: BattleCommand
---

# Properties
* `boost_multiplier`: float
* `duration_rounds`: int

# Methods

### `_init(p_source: Combatant, p_boost_multiplier: float = 1.5, p_duration_rounds: int = 2, p_block_cost: int = 2, p_priority: int = 0) -> void`
```logic
super(p_source, p_priority, p_block_cost)
boost_multiplier = p_boost_multiplier
duration_rounds = p_duration_rounds
command_name = "Focus"
```

### `execute() -> void`
```logic
if not source.is_alive():
	return
source.apply_attack_boost(boost_multiplier, duration_rounds)
var boost_pct: int = int((boost_multiplier - 1.0) * 100.0)
print("[%s] channels Focus in Tactical Stance! Attack power boosted by +%d%% for %d rounds." % [
	source.name,
	boost_pct,
	duration_rounds
])
execution_log = "[Skill: Focus] %s channeled Focus (+%d%% Attack Boost for %d rounds, Tactical Stance)" % [
	source.name, boost_pct, duration_rounds
]
```

