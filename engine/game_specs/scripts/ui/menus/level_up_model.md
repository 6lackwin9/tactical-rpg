---
class: LevelUpModel
extends: RefCounted
description: "Tactical RPG - Level-up model (UI2-L rule 3): a stat draft over the member's unspent points with a before / after preview; confirm() goes through LevelUp.confirm (errors verbatim) and is permanent. Deferring keeps the points."
---

# Properties
* `NO_POINTS`: String = "No points left." (const)
* `member_id`: String = ""
* `NO_MEMBER`: String = "No such party member." (const)
* `SPEND_QUESTION`: String = "Spend these points? This is permanent." (const)
* `draft`: Dictionary = {}   # stat -> drafted points (> 0 only)
* `last_errors`: Array[String] = []   # errors of the last asked confirm (the host shows them)
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `m`: UnitState = _member()
* `none`: Array[String] = [NO_MEMBER]
* `errors`: Array[String] = LevelUp.confirm(m, draft.duplicate())
* `out`: Array[Dictionary] = []
* `m`: UnitState = _member()
* `then_state`: UnitState = m.duplicate_state()
* `now_state`: UnitState = m.duplicate_state()
* `now`: Combatant = PartyBuilder.build(now_state)
* `then`: Combatant = PartyBuilder.build(then_state)
* `was`: bool = then.owned_families().has(fam) and then.tier_unlocked(sk)

# Methods

### `_init(p_member: String) -> void`
```logic
member_id = p_member
```

### `_member() -> UnitState`
```logic
return PartyPreview.member(member_id)
```

### `drafted() -> int`
```logic
return StatRules.total_of(draft)
```

### `points_left() -> int`
```logic
var m: UnitState = _member()
return m.unspent() - drafted() if m != null else 0
```

### `has_draft() -> bool`
```logic
return drafted() > 0
```

### `auto() -> void`
```logic
var m: UnitState = _member()
var arch: ClassArchetype = m.archetype() if m != null else null
if arch == null or points_left() <= 0:
	return
var extra: Dictionary = StatRules.auto_spend(arch.growth_per_level, points_left())
for key: Variant in extra:
	if DictUtil.to_int(extra[key]) > 0:
		draft[str(key)] = DictUtil.to_int(draft.get(str(key), 0)) + DictUtil.to_int(extra[key])
```

### `_confirm_now(router: MenuRouter) -> void`
```logic
last_errors = confirm()
router.changed.emit()
```

### `cancel() -> void`
```logic
draft = {}
```

### `drafted_state() -> UnitState`
```logic
var m: UnitState = _member()
if m == null:
	return null
var work: UnitState = m.duplicate_state()
var _e: Array[String] = LevelUp.confirm(work, draft.duplicate())
return work
```

### `before() -> Dictionary`
```logic
var m: UnitState = _member()
return PartyPreview.derived(m) if m != null else {}
```

### `after() -> Dictionary`
```logic
var work: UnitState = drafted_state()
return PartyPreview.derived(work) if work != null else {}
```

