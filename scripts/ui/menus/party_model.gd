class_name PartyModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Party model (UI2-L): the roster with the active members (at most max_active, RECRUIT1-L), the selected member, gold, and the doors to
# level-up, loadout and gear. Toggling active edits PartyState.active_ids directly (rule 2).
# ==============================================================================

const MAX_ACTIVE: String = "The active party holds at most %d."
const KEEP_ONE: String = "Keep at least one active member."

const TAB_STATS: String = "stats"
const TAB_LEVEL_UP: String = "level_up"
const TAB_SKILLS: String = "skills"
const TAB_GEAR: String = "gear"
const TABS: Array[String] = [TAB_STATS, TAB_LEVEL_UP, TAB_SKILLS, TAB_GEAR]
const TAB_LABELS: Dictionary = {TAB_STATS: "Stats", TAB_LEVEL_UP: "Level up", TAB_SKILLS: "Skills", TAB_GEAR: "Gear"}

var router: MenuRouter = null
var selected: String = ""
var tab: String = TAB_STATS
var _subs: Dictionary = {}   # tab -> the draft model of the selected member (LevelUpModel, LoadoutModel, GearModel); dropped on a member or tab switch

# UIR-3 (UI-REVIEW1 Q2 B): the Character popup. `p_member` / `p_tab` pick the first member and tab ("" = the first member, Stats).
func _init(p_router: MenuRouter, p_member: String = "", p_tab: String = "") -> void:
	router = p_router
	var gs: GameState = CurrentGame.state()
	if gs != null and not gs.party.members.is_empty():
		selected = gs.party.members[0].id
		if not p_member.is_empty() and gs.party.member(p_member) != null:
			selected = p_member
	if TABS.has(p_tab):
		tab = p_tab

# The draft models of the selected member, built on first use so each tab starts from the live state.
func level_model() -> LevelUpModel:
	var found: Variant = _subs.get(TAB_LEVEL_UP)
	if found is LevelUpModel:
		return found
	var made := LevelUpModel.new(selected)
	_subs[TAB_LEVEL_UP] = made
	return made

func loadout_model() -> LoadoutModel:
	var found: Variant = _subs.get(TAB_SKILLS)
	if found is LoadoutModel:
		return found
	var made := LoadoutModel.new(selected)
	_subs[TAB_SKILLS] = made
	return made

func gear_model() -> GearModel:
	var found: Variant = _subs.get(TAB_GEAR)
	if found is GearModel:
		return found
	var made := GearModel.new(selected)
	_subs[TAB_GEAR] = made
	return made

# True while the open tab holds an uncommitted draft (the router asks before Back / X / right click leaves it).
func has_draft() -> bool:
	var sub: Variant = _subs.get(tab)
	return sub is RefCounted and DictUtil.to_bool((sub as RefCounted).call("has_draft"))

# The one discard rule: a tab or member switch with a draft open asks first; Yes drops the draft and switches.
func switch_tab(id: String) -> Array[String]:
	var errors: Array[String] = []
	if not TABS.has(id):
		errors.append("Unknown tab '%s'." % id)
	elif id != tab:
		_guarded(_apply_tab.bind(id))
	return errors

func _apply_tab(id: String) -> void:
	_subs.clear()
	tab = id
	_changed()

func _guarded(action: Callable) -> void:
	if has_draft() and router != null:
		router.ask(MenuRouter.DISCARD_QUESTION, action)
	else:
		action.call()

func _changed() -> void:
	if router != null:
		router.changed.emit()

# Q / E: the previous (-1) or next (+1) member, wrapping.
func cycle_member(step: int) -> Array[String]:
	var ids: Array[String] = _party().member_ids()
	if ids.size() < 2:
		return [] as Array[String]
	var at: int = maxi(0, ids.find(selected))
	return select(ids[posmod(at + step, ids.size())])

func unspent() -> int:
	var m: UnitState = _party().member(selected)
	return m.unspent() if m != null else 0

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

# The world's active cap (RECRUIT1-L 1.3: WorldCatalog.party_max via PartyState.max_active).
func max_active() -> int:
	return _party().max_active

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
	elif id != selected:
		_guarded(_apply_member.bind(id))
	return errors

func _apply_member(id: String) -> void:
	_subs.clear()
	selected = id
	_changed()

func toggle_active(id: String) -> Array[String]:
	var errors: Array[String] = []
	var party: PartyState = _party()
	if party.member(id) == null:
		errors.append("No party member '%s'." % id)
		return errors
	if party.active_ids.is_empty():   # the implicit first four become explicit on the first edit
		party.active_ids = party.member_ids().slice(0, party.max_active)
	if party.active_ids.has(id):
		if party.active_ids.size() <= 1:
			errors.append(KEEP_ONE)
		else:
			party.active_ids.erase(id)
	elif party.active_ids.size() >= party.max_active:
		errors.append(MAX_ACTIVE % party.max_active)
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

# Opens the Level up, Skills (LOADOUT) or Gear tab for the selected member (the old screen ids still work as callers).
func open(id: String) -> Array[String]:
	var errors: Array[String] = []
	var tabs: Dictionary = {MenuRouter.LEVEL_UP: TAB_LEVEL_UP, MenuRouter.LOADOUT: TAB_SKILLS, MenuRouter.GEAR: TAB_GEAR}
	if not tabs.has(id):
		errors.append("Unknown party screen '%s'." % id)
	elif _party().member(selected) == null:
		errors.append("Pick a party member first.")
	else:
		errors = switch_tab(str(tabs[id]))
	return errors
