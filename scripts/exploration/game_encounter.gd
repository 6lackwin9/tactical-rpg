class_name GameEncounter
extends WorldEncounter

# ==============================================================================
# Tactical RPG - Game encounter (S9, spec EXP1-L 2.6): a world encounter fought by the SAVED party.
# The authored `party` roster is replaced by the active members of a PartyState (PartyBuilder: level, spend, gear,
# loadout), deployed by the normal ally placement for that many allies. `zone_override` (non-empty) replaces the
# authored arena zone (TRAV1 route biome). Enemies, waves and rules stay the authored ones.
# ==============================================================================

const PARTY_PREFIX: String = "__party_"

var party_state: PartyState = null
var zone_override: String = ""
var _built: Dictionary = {}   # roster id (PARTY_PREFIX + n) -> Combatant, rebuilt on every build()
var _bases: Dictionary = {}   # roster id -> base unit id
var _member_ids: Dictionary = {}   # Combatant -> party member id (hero_key)

func _init(p_world_id: String = "", p_local_id: String = "", p_party: PartyState = null, p_zone: String = "") -> void:
	super(p_world_id, p_local_id)
	party_state = p_party
	zone_override = p_zone
	load_definition()

func load_definition() -> void:
	super()
	if not zone_override.is_empty():
		zone_id = zone_override

func build() -> void:
	_built.clear()
	_bases.clear()
	_member_ids.clear()
	if party_state != null:
		for m: UnitState in party_state.active_members():
			var c: Combatant = PartyBuilder.build(m)
			if c != null:
				var rid: String = "%s%d" % [PARTY_PREFIX, _built.size()]
				_built[rid] = c
				_bases[rid] = m.base_unit
				_member_ids[c] = m.id
	super()

func _ids(key: String) -> Array[String]:
	if key != "party":
		return super(key)
	var out: Array[String] = []
	for rid: Variant in _built:
		out.append(str(rid))
	return out

func _unit_entry(unit_ref: String) -> Dictionary:
	if _bases.has(unit_ref):
		return ContentCatalog.get_entry("units", str(_bases[unit_ref]))
	return super(unit_ref)

func _spawn(unit_id: String, display_name: String) -> Combatant:
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

# The party member id (AUTO1 per-hero tactics are saved under it); other units as in CombatEncounter.
func hero_key(c: Combatant) -> String:
	if _member_ids.has(c):
		return str(_member_ids[c])
	return super(c)

# The saved party's cap (`PartyState.max_active`); without a party the world's cap.
func party_slots() -> int:
	return party_state.max_active if party_state != null else super()
