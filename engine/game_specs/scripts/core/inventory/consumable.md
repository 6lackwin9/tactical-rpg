---
class: Consumable
extends: Item
description: "Tactical RPG - Consumable Item Model Usable in combat via ItemCommand (Utility Stance) or during exploration"
---

# Properties
* `consumable_type`: ConsumableType = ConsumableType.HEALTH_POTION
* `heal_amount`: int = 0
* `damage_amount`: int = 0
* `buff_stat`: String = ""
* `buff_percent`: float = 0.0
* `buff_duration_rounds`: int = 0
* `block_cost`: int = 1
* `is_combat_usable`: bool = true
* `is_exploration_usable`: bool = true
* `use_reach`: int = REACH_TYPE_DEFAULT   # ITEMUSE1 1.2: -1 = reach_for(consumable_type); 0 = self only
* `REACH_TYPE_DEFAULT`: int = -1 (const)
* `REACH_MAX`: int = 4 (const)
* `TYPE_REACH`: Dictionary = { (const)

# Methods

### `static reach_for(p_type: ConsumableType) -> int`
```logic
return DictUtil.to_int(TYPE_REACH.get(p_type, REACH_TYPE_DEFAULT))
```

### `static offered_in_battle(p_type: ConsumableType) -> bool`
```logic
return TYPE_REACH.has(p_type)
```

### `reach() -> int`
```logic
return use_reach if use_reach >= 0 else reach_for(consumable_type)
```

### `apply_to_combatant(target: Combatant) -> Dictionary`
```logic
var outcome: Dictionary = {
	"healed": 0,
	"damage": 0,
	"buff_applied": false,
	"description": ""
}

if target == null:
	return outcome

if heal_amount > 0 and target.is_alive():
	var before_hp: int = target.hp
	target.heal(heal_amount)
	outcome["healed"] = target.hp - before_hp
	outcome["description"] = "Restored %d HP" % outcome["healed"]

if damage_amount > 0 and target.is_alive():
	var dealt: int = target.take_damage(damage_amount)
	outcome["damage"] = dealt
	outcome["description"] = "Inflicted %d damage" % dealt

if buff_percent > 0.0 and target.is_alive():
	target.apply_attack_boost(1.0 + buff_percent, buff_duration_rounds)
	outcome["buff_applied"] = true
	outcome["description"] = "Applied +%d%% %s boost for %d rounds" % [int(buff_percent * 100), buff_stat, buff_duration_rounds]

return outcome
```

