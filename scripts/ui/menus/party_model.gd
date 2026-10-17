class_name PartyModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Party model (UI2-L): the roster with the active four, the selected member, gold, and the doors to
# level-up, loadout and gear. Toggling active edits PartyState.active_ids directly (rule 2).
# ==============================================================================

const MAX_ACTIVE: String = "The active party holds at most %d."
const KEEP_ONE: String = "Keep at least one active member."

var router: MenuRouter = null
var selected: String = ""

func _init(p_router: MenuRouter) -> void:
	router = p_router
	var gs: GameState = CurrentGame.state()
	if gs != null and not gs.party.members.is_empty():
		selected = gs.party.members[0].id

func _party() -> PartyState:
	return CurrentGame.state().party

# [{id, name, class, level, active, unspent}]
func members() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var active: Array[UnitState] = _party().active_members()
	for m: UnitState in _party().members:
		out.append({"id": m.id, "name": PartyPreview.name_of(m), "class": PartyPreview.class_name_of(m), "level": m.level(), "active": active.has(m), "unspent": m.unspent()})
	return out

func active_count() -> int:
	return _party().active_members().size()

# True when any member has unspent level-up points.
func badge() -> bool:
	for m: UnitState in _party().members:
		if m.unspent() > 0:
			return true
	return false

func gold() -> int:
	return _party().gold

func select(id: String) -> Array[String]:
	var errors: Array[String] = []
	if _party().member(id) == null:
		errors.append("No party member '%s'." % id)
	else:
		selected = id
	return errors

func toggle_active(id: String) -> Array[String]:
	var errors: Array[String] = []
	var party: PartyState = _party()
	if party.member(id) == null:
		errors.append("No party member '%s'." % id)
		return errors
	if party.active_ids.is_empty():   # the implicit first four become explicit on the first edit
		party.active_ids = party.member_ids().slice(0, PartyState.PARTY_MAX)
	if party.active_ids.has(id):
		if party.active_ids.size() <= 1:
			errors.append(KEEP_ONE)
		else:
			party.active_ids.erase(id)
	elif party.active_ids.size() >= PartyState.PARTY_MAX:
		errors.append(MAX_ACTIVE % PartyState.PARTY_MAX)
	else:
		party.active_ids.append(id)
	return errors

func stats() -> StatsModel:
	return StatsModel.new(selected)

# Passive display names of the selected member (source "-": no source is recorded yet, rule 7).
func passives() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var m: UnitState = _party().member(selected)
	if m == null:
		return out
	for p: String in m.passives:
		out.append({"id": p, "name": str(ContentCatalog.get_entry("passives", p).get("name", p)), "source": "-"})
	return out

# Opens level-up, loadout or gear for the selected member.
func open(id: String) -> Array[String]:
	var errors: Array[String] = []
	if not [MenuRouter.LEVEL_UP, MenuRouter.LOADOUT, MenuRouter.GEAR].has(id):
		errors.append("Unknown party screen '%s'." % id)
	elif _party().member(selected) == null:
		errors.append("Pick a party member first.")
	else:
		router.push(id, {"member": selected})
	return errors
