---
class: PartyPreview
extends: RefCounted
description: "Tactical RPG - Party preview (UI2-L): the derived block of a party member, built by PartyBuilder with unspent points LEFT unspent (only confirmed and drafted points show), and the small display helpers the party screens share."
---

# Properties
* `DERIVED_LABELS`: Dictionary = { (const)

# Methods

### `static derived(state: UnitState) -> Dictionary`
```logic
var c: Combatant = PartyBuilder.build(state.duplicate_state(), false)
if c == null:
	return {}
return {
	"max_hp": c.max_hp, "speed": c.speed, "move_per_block": c.move_per_block(),
	"crit_pct": roundi(DamageCalculator.crit_chance(c)),
	"melee_power": DictUtil.to_int(DamageCalculator.attack_power(c, false)["attack_power"]),
	"ranged_power": DictUtil.to_int(DamageCalculator.attack_power(c, true)["attack_power"]),
	"armor_rating": DamageCalculator.armor_rating(c), "debuff_resist": roundi(c.debuff_resistance()), "ap": c.max_ap,
}
```

### `static name_of(m: UnitState) -> String`
```logic
return PartyBuilder.display_name(m)   # RECRUIT1-L rule 2: a named hero shows its character name
```

### `static class_name_of(m: UnitState) -> String`
```logic
var cls: Dictionary = ContentCatalog.get_entry("classes", m.class_id())
return str(cls.get("name", m.class_id()))
```

### `static member(member_id: String) -> UnitState`
```logic
var gs: GameState = CurrentGame.state()
return gs.party.member(member_id) if gs != null else null
```

