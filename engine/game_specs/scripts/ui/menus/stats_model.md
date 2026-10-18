---
class: StatsModel
extends: RefCounted
description: "Tactical RPG - Stats model (UI2-L): one member's six stats split into base, growth and spend, the derived block (PartyPreview.derived) and the XP bar. Read only."
---

# Properties
* `member_id`: String = ""

# Methods

### `_init(p_member: String) -> void`
```logic
member_id = p_member
```

### `stats() -> Dictionary`
```logic
var out: Dictionary = {}
var m: UnitState = PartyPreview.member(member_id)
var arch: ClassArchetype = m.archetype() if m != null else null
if arch == null:
	return out
var cs := CharacterStats.new(arch, m.level(), m.spend)
for s: String in StatRules.STATS:
	var base: int = arch.get_base_stat(s)
	var spend: int = DictUtil.to_int(m.spend.get(s, 0))
	var total: int = cs.get_base_stat(s)
	out[s] = {"base": base, "growth": total - base - spend, "spend": spend, "total": total}
return out
```

### `derived() -> Dictionary`
```logic
var m: UnitState = PartyPreview.member(member_id)
return PartyPreview.derived(m) if m != null else {}
```

### `xp() -> Dictionary`
```logic
var m: UnitState = PartyPreview.member(member_id)
if m == null:
	return {}
return {"total": m.total_xp, "level": m.level(), "to_next": XpCurve.xp_to_next(m.total_xp)}
```

