---
class: AiProfile
extends: RefCounted
description: "Tactical RPG - AI Profile (how a unit plans its rounds) Role picks the behaviour pattern; the other fields tune it. The same profile drives enemies every round and the player's \"AI preset\" sequences (see PlanPresets). `skill_cap` is an AP ceiling: the highest skill level (AP) it may use. Class-derived units get NO_CEILING (minion tier: StatRules.tier_skill_cap, BAL3 section 5); their skill unlocks are level-gated by Combatant.tier_unlocked (SK2). Damage comes from the unit itself (DamageCalculator)."
---

# Properties
* `NO_CEILING`: int = 5   # skill_cap meaning "no AP ceiling" (the highest skill level) (const)
* `role`: Role = Role.AGGRESSOR
* `skill_cap`: int = 1
* `flee_ratio`: float = 0.25    # withdraw at or below this HP fraction (0 = never)
* `defend_below`: float = 0.4   # guard once when at or below this HP fraction
* `target_pref`: TargetPref = TargetPref.NEAREST
* `prefer_level`: int = 0       # skill level to favour (0 = highest affordable)
* `reckless`: bool = false      # never flees or guards, always charges the nearest foe (berserker behaviour)
* `DEFAULT_HEAL_BELOW`: float = 0.7   # also the unit schema default of `heal_below` (CLASS1) (const)
* `heal_below`: float = DEFAULT_HEAL_BELOW   # SUPPORT role: heal the most wounded ally at or below this HP fraction
* `area_min_foes`: int = 2      # AI1: an area attack is chosen only if it hits at least this many foes (clamped to 1..6 on use)
* `use_items`: bool = false     # ITEMUSE1 1.4: may add a battle-pouch item step; only the sim (pouch=) sets it
* `AREA_MIN_FOES_MIN`: int = 1 (const)
* `AREA_MIN_FOES_MAX`: int = 6 (const)

# Methods

### `area_threshold() -> int`
```logic
return clampi(area_min_foes, AREA_MIN_FOES_MIN, AREA_MIN_FOES_MAX)
```

### `static create(p_role: Role, p_flee_ratio: float = 0.25, p_skill_cap: int = 1) -> AiProfile`
```logic
var profile := AiProfile.new()
profile.role = p_role
profile.flee_ratio = p_flee_ratio
profile.skill_cap = p_skill_cap
profile.target_pref = TargetPref.NEAREST if p_role == Role.AGGRESSOR else TargetPref.WEAKEST
return profile
```

### `duplicate_profile() -> AiProfile`
```logic
var copy := AiProfile.new()
copy.role = role
copy.skill_cap = skill_cap
copy.flee_ratio = flee_ratio
copy.defend_below = defend_below
copy.target_pref = target_pref
copy.prefer_level = prefer_level
copy.reckless = reckless
copy.heal_below = heal_below
copy.area_min_foes = area_min_foes
copy.use_items = use_items
return copy
```

### `static role_name(p_role: Role) -> String`
```logic
match p_role:
	Role.AGGRESSOR:
		return "Aggressor"
	Role.SKIRMISHER:
		return "Skirmisher"
	Role.SNIPER:
		return "Sniper"
return "Support"
```

