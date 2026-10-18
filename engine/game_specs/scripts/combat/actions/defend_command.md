---
class: DefendCommand
extends: BattleCommand
---

# Properties
* `mitigation_ratio`: float
* `grant_status`: String = ""   # buff made active the moment the defend starts (StatusEffects.defend_buffs(), e.g. riposte)
* `status_rounds`: int = 1      # rounds it lasts, counting the current one
* `guard_broken`: bool = false

# Methods

### `_init(p_source: Combatant, p_mitigation_ratio: float = 0.5, p_block_cost: int = 2, p_priority: int = 1) -> void`
```logic
super(p_source, p_priority, p_block_cost)
mitigation_ratio = p_mitigation_ratio
command_name = "Defend"
```

### `on_start() -> String`
```logic
if grant_status.is_empty() or not source.is_active():
	return ""
if StatusEffects.defend_buffs().has(grant_status):
	StatusEffects.grant_now(source, grant_status, status_rounds, status_magnitude, source.name)
	return "[Riposte] %s readies a riposte (%d round%s, ends on a stance change)" % [source.name, status_rounds, "" if status_rounds == 1 else "s"]
# A support-grant buff (taunt) is queued like every support buff: it starts next round.
StatusEffects.queue(source, grant_status, status_rounds, status_magnitude, source.name)
return "[%s] %s will be %s next round (%d round%s)" % [grant_status.capitalize(), source.name, StatusEffects.display_name(grant_status).to_lower(), status_rounds, "" if status_rounds == 1 else "s"]
```

### `execute() -> void`
```logic
if not source.is_alive():
	failed = true
	return
if guard_broken:
	execution_log = "[Defend] %s's guard was broken (no mitigation)" % source.name
	return
source.defense_mitigation_ratio = mitigation_ratio
var mit_pct: int = int(mitigation_ratio * 100.0)
print("[%s] assumes a Defensive Stance and raises guard (mitigates %d%% incoming damage)!" % [
	source.name,
	mit_pct
])
execution_log = "[Defend] %s raised guard (mitigates %d%% damage, Defensive Stance)" % [
	source.name, mit_pct
]
```

