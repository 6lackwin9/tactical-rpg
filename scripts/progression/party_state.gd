class_name PartyState
extends RefCounted

# ==============================================================================
# Tactical RPG - Party State (S7): the roster, who is active in a fight, the shared stash.
# The roster may be larger than PARTY_MAX; only PARTY_MAX members take the field and earn XP.
# S11 (DLG1-L, save v4): party gold 0..GOLD_MAX from quests, chests, selling and dialogue gifts.
# S10 (TRAV2-L, save v6): travel supplies 0..SUPPLIES_MAX; a party counter, not a stash item.
# ==============================================================================

const PARTY_MAX: int = 4
const GOLD_MAX: int = 9999999
const SUPPLIES_MAX: int = 99
const SUPPLIES_START: int = 10        # new-game fallback and the v5 -> v6 migration value

var members: Array[UnitState] = []
var active_ids: Array[String] = []     # subset of member ids; empty = the first PARTY_MAX members
var stash: Dictionary = {}             # item id -> count (>= 1)
var gold: int = 0                      # a non-whole saved value loads as -1 so validate() refuses it
var supplies: int = 0                  # same load rule as gold

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

# Removes `count` of `item_id`; false (and no change) when fewer are held. A count reaching 0 drops the key.
func remove_from_stash(item_id: String, count: int = 1) -> bool:
	var held: int = DictUtil.to_int(stash.get(item_id, 0))
	if count < 1 or held < count:
		return false
	if held == count:
		stash.erase(item_id)
	else:
		stash[item_id] = held - count
	return true

# Adds gold (clamped to GOLD_MAX); returns the amount actually added.
func add_gold(amount: int) -> int:
	var before: int = gold
	gold = clampi(gold + maxi(amount, 0), 0, GOLD_MAX)
	return gold - before

# Adds supplies (clamped to 0..SUPPLIES_MAX); returns the amount actually added.
func add_supplies(n: int) -> int:
	var before: int = supplies
	supplies = clampi(supplies + maxi(n, 0), 0, SUPPLIES_MAX)
	return maxi(supplies - before, 0)

# Spends up to `n` supplies (never below 0); returns the amount actually spent.
func spend_supplies(n: int) -> int:
	var spent: int = mini(maxi(n, 0), maxi(supplies, 0))
	supplies -= spent
	return spent

func to_dict() -> Dictionary:
	var rows: Array = []
	for m: UnitState in members:
		rows.append(m.to_dict())
	return {"members": rows, "active_ids": active_ids.duplicate(), "stash": stash.duplicate(), "gold": gold,
		"supplies": supplies}

static func from_dict(d: Dictionary) -> PartyState:
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
	if gold < 0 or gold > GOLD_MAX:
		errors.append("gold must be 0..%d." % GOLD_MAX)
	if supplies < 0 or supplies > SUPPLIES_MAX:
		errors.append("supplies must be 0..%d." % SUPPLIES_MAX)
	return errors
