---
class: PartyState
extends: RefCounted
description: "Tactical RPG - Party State (S7): the roster, who is active in a fight, the shared stash. The roster may be larger than PARTY_MAX; only PARTY_MAX members take the field and earn XP. RECRUIT1-L 1.3: `max_active` (not saved; GameState.create / from_dict set it from WorldCatalog.party_max) is the world's cap on active members, at most PARTY_MAX. S11 (DLG1-L, save v4): party gold 0..GOLD_MAX from quests, chests, selling and dialogue gifts. S10 (TRAV2-L, save v6): travel supplies 0..SUPPLIES_MAX; a party counter, not a stash item."
---

# Properties
* `PARTY_MAX`: int = 4 (const)
* `GOLD_MAX`: int = 9999999 (const)
* `SUPPLIES_MAX`: int = 99 (const)
* `SUPPLIES_START`: int = 10        # new-game fallback and the v5 -> v6 migration value (const)
* `members`: Array[UnitState] = []
* `active_ids`: Array[String] = []     # subset of member ids; empty = the first max_active members
* `max_active`: int = PARTY_MAX        # RECRUIT1-L: the world's party cap (never saved)
* `stash`: Dictionary = {}             # item id -> count (>= 1)
* `gold`: int = 0                      # a non-whole saved value loads as -1 so validate() refuses it
* `supplies`: int = 0                  # same load rule as gold
* `out`: Array[String] = []
* `out`: Array[UnitState] = []
* `errors`: Array[String] = []
* `seen`: Array[String] = []
* `listed`: Array[String] = []

# Methods

### `member(unit_id: String) -> UnitState`
```logic
for m: UnitState in members:
	if m.id == unit_id:
		return m
return null
```

### `add_member(m: UnitState) -> void`
```logic
members.append(m)
```

### `add_to_stash(item_id: String, count: int = 1) -> void`
```logic
stash[item_id] = DictUtil.to_int(stash.get(item_id, 0)) + count
```

### `remove_from_stash(item_id: String, count: int = 1) -> bool`
```logic
var held: int = DictUtil.to_int(stash.get(item_id, 0))
if count < 1 or held < count:
	return false
if held == count:
	stash.erase(item_id)
else:
	stash[item_id] = held - count
return true
```

### `add_gold(amount: int) -> int`
```logic
var before: int = gold
gold = clampi(gold + maxi(amount, 0), 0, GOLD_MAX)
return gold - before
```

### `add_supplies(n: int) -> int`
```logic
var before: int = supplies
supplies = clampi(supplies + maxi(n, 0), 0, SUPPLIES_MAX)
return maxi(supplies - before, 0)
```

### `static supplies_cost(have: int, n: int) -> int`
```logic
return mini(maxi(n, 0), maxi(have, 0))
```

### `spend_supplies(n: int) -> int`
```logic
var spent: int = supplies_cost(supplies, n)
supplies -= spent
return spent
```

### `to_dict() -> Dictionary`
```logic
var rows: Array = []
for m: UnitState in members:
	rows.append(m.to_dict())
return {"members": rows, "active_ids": active_ids.duplicate(), "stash": stash.duplicate(), "gold": gold,
	"supplies": supplies}
```

### `static from_dict(d: Dictionary) -> PartyState`
```logic
var p := PartyState.new()
for raw: Variant in DictUtil.array_of(d.get("members", [])):
	p.members.append(UnitState.from_dict(DictUtil.dict_of(raw)))
p.active_ids = JsonUtil.strings(d.get("active_ids", []))
p.stash = JsonUtil.ints(d.get("stash", {}))
var raw_gold: Variant = JsonUtil.normalize(d.get("gold", 0))
p.gold = DictUtil.to_int(raw_gold) if raw_gold is int else -1
var raw_supplies: Variant = JsonUtil.normalize(d.get("supplies", 0))
p.supplies = DictUtil.to_int(raw_supplies) if raw_supplies is int else -1
return p
```

### `apply_cap(cap: int) -> void`
```logic
max_active = clampi(cap, 1, PARTY_MAX)
if active_ids.size() > max_active:
	active_ids = active_ids.slice(0, max_active)
```

### `equals(other: PartyState) -> bool`
```logic
return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())
```

