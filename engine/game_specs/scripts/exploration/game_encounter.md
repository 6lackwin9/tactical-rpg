---
class: GameEncounter
extends: WorldEncounter
description: "Tactical RPG - Game encounter (S9, spec EXP1-L 2.6): a world encounter fought by the SAVED party. The authored `party` roster is replaced by the active members of a PartyState (PartyBuilder: level, spend, gear, loadout), deployed by the normal ally placement for that many allies. `zone_override` (non-empty) replaces the authored arena zone (TRAV1 route biome). Enemies, waves and rules stay the authored ones. HEAL1-L: each member starts at its stored HP (HealRules.start_hp from hp_lost; PartyBuilder itself stays full HP) and write_back_hp stores the after-battle HP (rule 3.2) once, for won and fled; lost is settled by ExploreRunner.wake."
---

# Properties
* `PARTY_PREFIX`: String = "__party_" (const)
* `party_state`: PartyState = null
* `zone_override`: String = ""
* `_built`: Dictionary = {}   # roster id (PARTY_PREFIX + n) -> Combatant, rebuilt on every build()
* `_bases`: Dictionary = {}   # roster id -> base unit id
* `_member_ids`: Dictionary = {}   # Combatant -> party member id (hero_key)
* `_own_pouch`: Dictionary = {}    # Combatant -> Array[Consumable]: battle_pouch items built from the member's saved pouch
* `_unloaded`: Dictionary = {}     # Combatant -> Array[String]: saved pouch ids that found no battle slot (kept on write-back)
* `_hp_written`: bool = false      # write_back_hp stores once per build (a results Replay never writes again)
* `out`: Array[String] = []

# Methods

### `_init(p_world_id: String = "", p_local_id: String = "", p_party: PartyState = null, p_zone: String = "") -> void`
```logic
super(p_world_id, p_local_id)
party_state = p_party
zone_override = p_zone
load_definition()
```

### `load_definition() -> void`
```logic
super()
if not zone_override.is_empty():
	zone_id = zone_override
```

### `build() -> void`
```logic
_built.clear()
_bases.clear()
_member_ids.clear()
_own_pouch.clear()
_unloaded.clear()
_hp_written = false
if party_state != null:
	for m: UnitState in party_state.active_members():
		var report: Dictionary = PartyBuilder.build_report(m)
		var raw_c: Variant = report.get("combatant", null)
		var c: Combatant = raw_c if raw_c is Combatant else null
		if c != null:
			_own_pouch[c] = DictUtil.array_of(report.get("pouch", []))
			_unloaded[c] = JsonUtil.strings(report.get("pouch_unloaded", []))
			var rid: String = "%s%d" % [PARTY_PREFIX, _built.size()]
			_built[rid] = c
			_bases[rid] = m.base_unit
			_member_ids[c] = m.id
			c.hp = HealRules.start_hp(c.max_hp, m.hp_lost)
super()
```

### `_unit_entry(unit_ref: String) -> Dictionary`
```logic
if _bases.has(unit_ref):
	return ContentCatalog.get_entry("units", str(_bases[unit_ref]))
return super(unit_ref)
```

### `_spawn(unit_id: String, display_name: String) -> Combatant`
```logic
if not _built.has(unit_id):
	return super(unit_id, display_name)
var c: Combatant = _built[unit_id]
var entry: Dictionary = _unit_entry(unit_id)
if not units_by_id.has(unit_id):
	units_by_id[unit_id] = []
var list: Array = units_by_id[unit_id]
list.append(c)
unit_classes[c] = ContentFactory.art_of(entry)
ai_profiles[c] = ContentFactory.ai_profile(entry, c)
return c
```

### `hero_key(c: Combatant) -> String`
```logic
if _member_ids.has(c):
	return str(_member_ids[c])
return super(c)
```

### `write_back_pouches() -> void`
```logic
if party_state == null:
	return
for key: Variant in _member_ids:
	if not (key is Combatant):
		continue
	var c: Combatant = key
	if is_guest(c):
		continue
	var m: UnitState = party_state.member(str(_member_ids[c]))
	if m == null:
		continue
	var kept: Array[String] = []
	for raw: Variant in DictUtil.array_of(_own_pouch.get(c, [])):
		if raw is Consumable:
			var con: Consumable = raw
			if c.inventory.battle_pouch.has(con):
				kept.append(con.id)
	kept.append_array(JsonUtil.strings(_unloaded.get(c, [])))
	m.pouch = kept
```

### `write_back_hp(outcome: String) -> void`
```logic
if party_state == null or _hp_written or (outcome != "won" and outcome != "fled"):
	return
_hp_written = true
for key: Variant in _member_ids:
	if not (key is Combatant):
		continue
	var c: Combatant = key
	if is_guest(c):
		continue
	var m: UnitState = party_state.member(str(_member_ids[c]))
	if m == null:
		continue
	var hp: int = HealRules.hp_after(mini(c.hp if c.is_alive() else 0, c.max_hp), c.max_hp, outcome)
	m.hp_lost = maxi(c.max_hp - hp, 0)
```

### `party_slots() -> int`
```logic
return party_state.max_active if party_state != null else super()
```

