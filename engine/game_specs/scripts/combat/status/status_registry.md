---
class: StatusRegistry
extends: RefCounted
description: "Tactical RPG - Status Registry (the ONE table of every valid buff / debuff) StatusEffects derives debuffs() / buffs() / resistable() / support_buffs() / defend_buffs() / display_name() from ROWS and applies the stacking rule through stack(). Row fields: id             status key (a StatusEffects constant) kind           \"buff\" | \"debuff\" resistable     debuffs only: a unit / passive may carry status_resist for it, and attack skills roll it against resistance. Self-inflicted debuffs (AP debt) skip the roll. default_rounds rounds it lasts when the source does not say (skills carry their own status_rounds) stacking       what a second application does (see stack()): refresh_max_duration  rounds = longer of the two, potency = stronger of the two (today's behaviour) extend_duration       rounds add up (cap = max total rounds), potency = stronger stack_potency         rounds = longer, potency adds up (cap = max total potency) none                  the first application stays until it expires cap            0 = no cap. extend_duration: max rounds; every other rule: max potency display        player-facing name grant          buffs only: \"support\" (queued by a SUPPORT skill) or \"defend\" (active now, DEFEND skill); \"\" for debuffs magnitude_min  0..100: lowest magnitude a skill may give the status (S12S-L) magnitude_max  >= magnitude_min; 0 (with min 0) = the status carries no magnitude magnitude_unit \"pct\" (weaken: % of outgoing power), \"flat\" (expose / fortify armor points, poison HP per round) or \"pct_max_hp\" (regen / burn: % of max HP per round) A magnitude travels in the `potency` channel of stack(), capped by `cap` (StatusEffects.has_magnitude reads magnitude_max). HOW TO ADD A STATUS (e.g. weaken, slice D): 1. add `const WEAKEN: String = \"weaken\"` to StatusEffects and one row to ROWS below (all fields); 2. effects of the status live where they act (blocked_blocks, damage hook, ...), never in the registry; 3. the derived lists, schema validation (status_effect / grant_status / cleanse), passive `resist` and the Forge pick it up automatically; extend StatusEffects' header comment; 4. the registry test requires every StatusEffects id constant to have a row, so a forgotten row fails."
---

# Properties
* `KIND_BUFF`: String = "buff" (const)
* `KIND_DEBUFF`: String = "debuff" (const)
* `REFRESH_MAX_DURATION`: String = "refresh_max_duration" (const)
* `EXTEND_DURATION`: String = "extend_duration" (const)
* `STACK_POTENCY`: String = "stack_potency" (const)
* `NONE`: String = "none" (const)
* `RULES`: Array[String] = [REFRESH_MAX_DURATION, EXTEND_DURATION, STACK_POTENCY, NONE] (const)
* `UNIT_PCT`: String = "pct" (const)
* `UNIT_FLAT`: String = "flat" (const)
* `UNIT_PCT_MAX_HP`: String = "pct_max_hp" (const)
* `UNITS`: Array[String] = [UNIT_PCT, UNIT_FLAT, UNIT_PCT_MAX_HP] (const)
* `ROWS`: Array[Dictionary] = [ (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `seen`: Array[String] = []
* `id`: String = str(r.get("id", ""))
* `grant`: String = str(r.get("grant", ""))
* `mag_min`: int = DictUtil.to_int(r.get("magnitude_min", 0))
* `mag_max`: int = DictUtil.to_int(r.get("magnitude_max", 0))

# Methods

### `static row(status: String) -> Dictionary`
```logic
for r: Dictionary in ROWS:
	if str(r["id"]) == status:
		return r
return {}
```

### `static has(status: String) -> bool`
```logic
return not row(status).is_empty()
```

### `static default_rounds(status: String) -> int`
```logic
return DictUtil.to_int(row(status).get("default_rounds", 1), 1)
```

### `static stacking_of(status: String) -> String`
```logic
return str(row(status).get("stacking", REFRESH_MAX_DURATION))
```

### `static cap_of(status: String) -> int`
```logic
return DictUtil.to_int(row(status).get("cap", 0))
```

### `static magnitude_min(status: String) -> int`
```logic
return DictUtil.to_int(row(status).get("magnitude_min", 0))
```

### `static magnitude_max(status: String) -> int`
```logic
return DictUtil.to_int(row(status).get("magnitude_max", 0))
```

### `static magnitude_unit(status: String) -> String`
```logic
return str(row(status).get("magnitude_unit", UNIT_PCT))
```

### `static stack(existing_rounds: int, existing_potency: int, new_rounds: int, new_potency: int, status: String) -> Dictionary`
```logic
return stack_with(stacking_of(status), cap_of(status), existing_rounds, existing_potency, new_rounds, new_potency)
```

### `static stack_with(rule: String, cap: int, existing_rounds: int, existing_potency: int, new_rounds: int, new_potency: int) -> Dictionary`
```logic
var add_rounds: int = maxi(1, new_rounds)
var rounds: int = add_rounds
var potency: int = new_potency
if existing_rounds > 0:
	match rule:
		EXTEND_DURATION:
			rounds = existing_rounds + add_rounds
			potency = maxi(existing_potency, new_potency)
		STACK_POTENCY:
			rounds = maxi(existing_rounds, add_rounds)
			potency = existing_potency + new_potency
		NONE:
			rounds = existing_rounds
			potency = existing_potency
		_:
			rounds = maxi(existing_rounds, add_rounds)
			potency = maxi(existing_potency, new_potency)
if cap > 0:
	if rule == EXTEND_DURATION:
		rounds = mini(rounds, cap)
	else:
		potency = mini(potency, cap)
return {"rounds": rounds, "potency": potency}
```

