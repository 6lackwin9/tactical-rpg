---
class: LoadoutModel
extends: RefCounted
description: "Tactical RPG - Loadout model (UI2-L rule 5): a draft of the member's active skill families, 5 class + 3 extra slots (StatRules), basics never slotted. An empty loadout means the default rule (the built unit's own pick); the first edit starts from those default slots. confirm() writes the draft and returns UnitState.validate messages verbatim."
---

# Properties
* `member_id`: String = ""
* `class_slots`: Array[String] = []
* `extra_slots`: Array[String] = []
* `using_default`: bool = true
* `_original`: Dictionary = {}
* `out`: Array[String] = []
* `c`: Combatant = _built(_member())
* `fam`: String = SkillCatalog.family_of(id)
* `errors`: Array[String] = []
* `list`: Array[String] = _slots(group)
* `errors`: Array[String] = []
* `list`: Array[String] = _slots(group)
* `m`: UnitState = _member()
* `work`: UnitState = m.duplicate_state()
* `errors`: Array[String] = []

# Methods

### `_init(p_member: String) -> void`
```logic
member_id = p_member
var m: UnitState = PartyPreview.member(member_id)
if m != null:
	_original = m.loadout.duplicate(true)
	cancel()
```

### `_member() -> UnitState`
```logic
return PartyPreview.member(member_id)
```

### `_built(state: UnitState) -> Combatant`
```logic
return PartyBuilder.build(state, false)
```

### `cancel() -> void`
```logic
var cls: Array[String] = JsonUtil.strings(_original.get("class", []))
var ext: Array[String] = JsonUtil.strings(_original.get("extra", []))
using_default = cls.is_empty() and ext.is_empty()
class_slots = cls
extra_slots = ext
if using_default:
	_fill_default()
```

### `_fill_default() -> void`
```logic
var work: UnitState = _member().duplicate_state()
work.loadout = {"class": [], "extra": []}
var c: Combatant = _built(work)
class_slots = c.active_class_families.duplicate() if c != null else ([] as Array[String])
extra_slots = c.active_extra_families.duplicate() if c != null else ([] as Array[String])
```

### `draft_loadout() -> Dictionary`
```logic
if using_default:
	return {"class": [] as Array[String], "extra": [] as Array[String]}
return {"class": class_slots.duplicate(), "extra": extra_slots.duplicate()}
```

### `has_draft() -> bool`
```logic
return not JsonUtil.deep_equal(draft_loadout(), {"class": JsonUtil.strings(_original.get("class", [])), "extra": JsonUtil.strings(_original.get("extra", []))})
```

### `families() -> Dictionary`
```logic
var out: Dictionary = {"class": [] as Array[Dictionary], "extra": [] as Array[Dictionary]}
var m: UnitState = _member()
var c: Combatant = _built(m) if m != null else null
if c == null:
	return out
var path: Array[String] = ContentFactory.families_of(m.archetype().skills_at(StatRules.MAX_LEVEL))
for fam: String in c.owned_families():
	if SkillCatalog.is_basic_family(fam):
		continue
	var t1: SkillDef = SkillCatalog.tier_one(fam)
	var tiers: Array[Dictionary] = []
	for sk: SkillDef in SkillCatalog.family_tiers(fam):
		tiers.append({"tier": sk.tier, "name": sk.display_name, "unlock_level": c.skill_unlock_level(sk), "castable": c.tier_unlocked(sk)})
	var row: Dictionary = {"family": fam, "name": SkillCatalog.family_name(fam), "grade": str(SkillDef.Grade.keys()[t1.grade]).capitalize() if t1 != null else "", "tiers": tiers}
	var group: Array[Dictionary] = out["class" if path.has(fam) else "extra"]
	group.append(row)
return out
```

### `_limit(group: String) -> int`
```logic
return StatRules.MAX_CLASS_SLOTS if group == "class" else StatRules.MAX_EXTRA_SLOTS
```

### `use_default() -> void`
```logic
using_default = true
_fill_default()
```

### `limit(group: String) -> int`
```logic
return _limit(group)
```

