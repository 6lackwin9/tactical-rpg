---
class: PlanContext
extends: RefCounted
description: "Tactical RPG - Plan Context (who is on which side, and how each unit plans) The read-only world a CombatPlan / TacticalAI works against: the grid, both sides and each unit's AiProfile (unlocked skill level, role, thresholds)."
---

# Properties
* `grid`: CombatGrid = null
* `allies`: Array[Combatant] = []
* `enemies`: Array[Combatant] = []
* `profiles`: Dictionary = {}   # Combatant -> AiProfile
* `total_blocks`: int = 6       # blocks in a round
* `none`: Array[Combatant] = []
* `none`: Array[Combatant] = []

# Methods

### `static create(p_grid: CombatGrid, p_allies: Array[Combatant], p_enemies: Array[Combatant], p_profiles: Dictionary = {}, p_total_blocks: int = 6) -> PlanContext`
```logic
var ctx := PlanContext.new()
ctx.total_blocks = p_total_blocks
ctx.grid = p_grid
ctx.allies = p_allies
ctx.enemies = p_enemies
ctx.profiles = p_profiles
return ctx
```

### `profile_of(c: Combatant) -> AiProfile`
```logic
var raw: Variant = profiles.get(c, null)
if raw is AiProfile:
	var stored: AiProfile = raw
	return stored
var is_ranged: bool = c.attack_range > 1
return AiProfile.create(AiProfile.Role.SNIPER if is_ranged else AiProfile.Role.AGGRESSOR)
```

### `power_of(c: Combatant) -> int`
```logic
return DamageCalculator.power_of(c, c.attack_range > 1)
```

### `skill_cap_of(c: Combatant) -> int`
```logic
return profile_of(c).skill_cap
```

