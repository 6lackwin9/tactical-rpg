---
class: HealRules
extends: RefCounted
description: "Tactical RPG - Heal rules (HEAL1-L, spec docs/specs/HEAL1.md section 3, option C): HP between battles. Each party member stores hp_lost (UnitState); current HP = clampi(max_hp - hp_lost, 1, max_hp) with max_hp derived from stats and gear. After a won battle every member who fought rallies RALLY_PCT of max HP (a downed hero gets up at that much); fled = at least 1 HP, no rally; lost = the defeat rule (on_defeat). A rest (rest place on entry, make camp for CAMP_COST supplies on the world map) fully restores every roster member. Pure and static; the sim shares hp_after."
---

# Properties
* `RALLY_PCT`: int = 25 (const)
* `CAMP_COST`: int = 3 (const)
* `MODE_REST_PLACE`: String = "rest_place" (const)
* `MODE_FORCED_REST`: String = "forced_rest" (const)
* `MODE_BACK_TO_TOWN`: String = "back_to_town" (const)
* `REFUSE_NOT_AT_NODE`: String = "Make camp on the world map." (const)
* `REFUSE_RESTED`: String = "Everyone is rested." (const)
* `REFUSE_SUPPLIES`: String = "Making camp needs 3 supplies." (const)

# Methods

### `static rally(max_hp: int) -> int`
```logic
return ceili(max_hp * RALLY_PCT / 100.0)
```

### `static hp_after(hp: int, max_hp: int, outcome: String) -> int`
```logic
match outcome:
	"won":
		if hp <= 0:
			return mini(max_hp, rally(max_hp))
		return mini(max_hp, hp + rally(max_hp))
	"lost":
		return max_hp
return clampi(hp, 1, maxi(max_hp, 1))
```

### `static start_hp(max_hp: int, hp_lost: int) -> int`
```logic
return clampi(max_hp - maxi(hp_lost, 0), 1, maxi(max_hp, 1))
```

### `static max_hp_of(state: UnitState) -> int`
```logic
if state == null:
	return 0
var c: Combatant = PartyBuilder.build(state.duplicate_state())
return c.max_hp if c != null else 0
```

### `static current_hp(state: UnitState) -> Dictionary`
```logic
var max_hp: int = max_hp_of(state)
var lost: int = state.hp_lost if state != null else 0
return {"hp": start_hp(max_hp, lost) if max_hp > 0 else 0, "max": max_hp}
```

### `static hurt(party: PartyState) -> bool`
```logic
for m: UnitState in party.members:
	if m.hp_lost > 0:
		return true
return false
```

### `static rest(party: PartyState) -> int`
```logic
var restored: int = 0
for m: UnitState in party.members:
	if m.hp_lost <= 0:
		continue
	var max_hp: int = max_hp_of(m)
	if max_hp > 0:
		restored += max_hp - start_hp(max_hp, m.hp_lost)
	m.hp_lost = 0
return restored
```

### `static is_rest_place(gs: GameState, location_ref: String) -> bool`
```logic
if location_ref.is_empty():
	return false
return DictUtil.to_bool(ExploreRunner.resolve_location(gs, location_ref).get("rest", false))
```

### `static can_camp(gs: GameState) -> String`
```logic
if TravelRunner.current_node(gs).is_empty():
	return REFUSE_NOT_AT_NODE
if not hurt(gs.party):
	return REFUSE_RESTED
if gs.party.supplies < CAMP_COST:
	return REFUSE_SUPPLIES
return ""
```

### `static make_camp(gs: GameState) -> Dictionary`
```logic
var errors: Array[String] = []
var events: Array[Dictionary] = []
var refusal: String = can_camp(gs)
if not refusal.is_empty():
	errors.append(refusal)
	return {"ok": false, "errors": errors, "spent": 0, "restored": 0, "events": events}
var spent: int = gs.party.spend_supplies(CAMP_COST)
var restored: int = rest(gs.party)
events.append({"kind": "camped", "location": gs.location_id, "spent": spent, "restored": restored})
events.append({"kind": "autosave"})
return {"ok": true, "errors": errors, "spent": spent, "restored": restored, "events": events}
```

### `static last_rest(gs: GameState) -> String`
```logic
var ref: String = str(gs.explore.get("last_rest", ""))
var hit: Dictionary = ExploreRunner.resolve_location(gs, ref) if not ref.is_empty() else {}
return str(hit.get("_qid", "")) if not hit.is_empty() else ""
```

### `static on_defeat(gs: GameState, checkpoint: String) -> Dictionary`
```logic
var plan: Dictionary = defeat_plan(gs, checkpoint)
settle_defeat(gs, plan)
return plan
```

### `static defeat_plan(gs: GameState, checkpoint: String) -> Dictionary`
```logic
var at: String = checkpoint if not checkpoint.is_empty() else MapRules.start_location(gs.world_id)
if is_rest_place(gs, at):
	return {"mode": MODE_REST_PLACE, "spent": 0, "to": checkpoint}
if gs.party.supplies >= CAMP_COST:
	return {"mode": MODE_FORCED_REST, "spent": CAMP_COST, "to": checkpoint}
return {"mode": MODE_BACK_TO_TOWN, "spent": 0, "to": last_rest(gs)}
```

### `static settle_defeat(gs: GameState, plan: Dictionary) -> void`
```logic
var _paid: int = gs.party.spend_supplies(DictUtil.to_int(plan.get("spent", 0)))
for m: UnitState in gs.party.members:
	m.hp_lost = 0
```

