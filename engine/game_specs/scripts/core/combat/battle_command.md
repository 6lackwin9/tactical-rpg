---
class: BattleCommand
extends: RefCounted
description: "Tactical RPG - Battle Command (one scheduled action on a unit's timeline) Built from a PlanEntry. When it STARTS the resolver charges its AP (skill + automatic stance switch; debt allowed), enters stance_on_start and calls on_start(); its effect (execute) lands at the end of its last block."
---

# Properties
* `source`: Combatant
* `priority`: int = 0  # High-priority actions (Switch Target, Defend) resolve first within a block
* `block_cost`: int = 1  # Timeline duration in blocks
* `command_name`: String = "Command"
* `execution_log`: String = ""
* `reactions`: Array[Dictionary] = []   # counters this command provoked (RiposteRules.counter results), logged after it
* `ap_cost`: int = 0                 # skill AP (excludes the stance switch)
* `stance_switch_cost`: int = 0     # extra AP for changing stance on start
* `stance_on_start`: int = -1       # CombatTypes.StanceType to enter at start, -1 = keep
* `est_damage`: int = 0              # planner's expected hit (attacks), for previews
* `status_effect`: String = ""        # status this command tries to inflict (StatusEffects)
* `status_potency`: int = 0
* `status_magnitude`: int = 0         # carried by the inflicted instance (unit from the status registry)
* `rng`: RandomNumberGenerator = null   # battle RNG, set by the resolver before execute()
* `stance_on_end`: int = -1          # CombatTypes.StanceType after execute(), -1 = none
* `failed`: bool = false             # execute() could not act (inactive unit, no target, out of reach): no flow
* `disrupt`: int = SkillDef.Disrupt.NONE
* `exploit`: Array[Dictionary] = []  # [{"status", "power_mult"}]
* `exploit_consume`: bool = false
* `broken_targets`: Array[Combatant] = []   # filled by execute(): units whose stance this command broke
* `combo_notes`: Array[String] = []         # break log lines, emitted after the reactions
* `exploited`: Array[Dictionary] = []       # filled by execute(): [{"target": Combatant, "status"}] per exploited hit (BREPORT1 1.3)

# Methods

### `_init(p_source: Combatant, p_priority: int = 0, p_block_cost: int = 1) -> void`
```logic
source = p_source
priority = p_priority
block_cost = p_block_cost
```

### `get_initiative() -> int`
```logic
return (priority * 1000) + source.speed
```

### `can_execute() -> bool`
```logic
if not source.is_alive():
	return false
return true
```

### `on_start() -> String`
```logic
return ""
```

### `execute() -> void`
```logic
pass
```

### `exploit_against(target: Combatant) -> Dictionary`
```logic
return SkillDef.exploit_match(exploit, target)
```

### `after_hit(target: Combatant, taken: int, matched: Dictionary) -> void`
```logic
if disrupt == SkillDef.Disrupt.STANCE_BREAK and taken > 0 and target.is_active() \
		and (target.current_stance == CombatTypes.StanceType.OFFENSIVE or target.current_stance == CombatTypes.StanceType.DEFENSIVE):
	target.set_stance(CombatTypes.StanceType.TACTICAL)
	target.defense_mitigation_ratio = 0.0
	broken_targets.append(target)
	combo_notes.append("[Break] %s breaks %s's stance (now Tactical)" % [source.name, target.name])
var status: String = str(matched.get("status", ""))
if exploit_consume and not status.is_empty():
	StatusEffects.remove_active(target, status)
```

### `note_exploit(target: Combatant, matched: Dictionary) -> void`
```logic
var status: String = str(matched.get("status", ""))
if not status.is_empty():
	exploited.append({"target": target, "status": status})
```

### `static exploit_note(matched: Dictionary) -> String`
```logic
var status: String = str(matched.get("status", ""))
return "" if status.is_empty() else " (exploits %s)" % StatusEffects.display_name(status)
```

