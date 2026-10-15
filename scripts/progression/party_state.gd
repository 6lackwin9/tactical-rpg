class_name PartyState
extends RefCounted

# ==============================================================================
# Tactical RPG - Party State (S7): the roster, who is active in a fight, the shared stash.
# The roster may be larger than PARTY_MAX; only PARTY_MAX members take the field and earn XP.
# ==============================================================================

const PARTY_MAX: int = 4

var members: Array[UnitState] = []
var active_ids: Array[String] = []     # subset of member ids; empty = the first PARTY_MAX members
var stash: Dictionary = {}             # item id -> count (>= 1)

func member(unit_id: String) -> UnitState:
	for m: UnitState in members:
		if m.id == unit_id:
			return m
	return null

func member_ids() -> Array[String]:
	var out: Array[String] = []
	for m: UnitState in members:
		out.append(m.id)
	return out

# The members that take the field (and earn XP), in roster order for the default.
func active_members() -> Array[UnitState]:
	var out: Array[UnitState] = []
	if active_ids.is_empty():
		for m: UnitState in members:
			if out.size() < PARTY_MAX:
				out.append(m)
		return out
	for m: UnitState in members:
		if active_ids.has(m.id) and out.size() < PARTY_MAX:
			out.append(m)
	return out

func add_member(m: UnitState) -> void:
	members.append(m)

func add_to_stash(item_id: String, count: int = 1) -> void:
	stash[item_id] = DictUtil.to_int(stash.get(item_id, 0)) + count

func to_dict() -> Dictionary:
	var rows: Array = []
	for m: UnitState in members:
		rows.append(m.to_dict())
	return {"members": rows, "active_ids": active_ids.duplicate(), "stash": stash.duplicate()}

static func from_dict(d: Dictionary) -> PartyState:
	var p := PartyState.new()
	for raw: Variant in DictUtil.array_of(d.get("members", [])):
		p.members.append(UnitState.from_dict(DictUtil.dict_of(raw)))
	p.active_ids = JsonUtil.strings(d.get("active_ids", []))
	p.stash = JsonUtil.ints(d.get("stash", {}))
	return p

func equals(other: PartyState) -> bool:
	return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())

func validate() -> Array[String]:
	var errors: Array[String] = []
	if members.is_empty():
		errors.append("A party needs at least one member.")
	var seen: Array[String] = []
	for m: UnitState in members:
		if seen.has(m.id):
			errors.append("Duplicate member id '%s'." % m.id)
		seen.append(m.id)
		errors.append_array(m.validate())
	if active_ids.size() > PARTY_MAX:
		errors.append("At most %d active members (got %d)." % [PARTY_MAX, active_ids.size()])
	var listed: Array[String] = []
	for a: String in active_ids:
		if not seen.has(a):
			errors.append("Active id '%s' is not a member." % a)
		if listed.has(a):
			errors.append("Active id '%s' is listed twice." % a)
		listed.append(a)
	for key: Variant in stash:
		if not ContentCatalog.has("items", str(key)):
			errors.append("Stash item '%s' does not exist." % str(key))
		elif DictUtil.to_int(stash[key]) < 1:
			errors.append("Stash count of '%s' must be >= 1." % str(key))
	return errors
