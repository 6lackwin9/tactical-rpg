class_name LoadoutModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Loadout model (UI2-L rule 5): a draft of the member's active skill families, 5 class + 3 extra slots
# (StatRules), basics never slotted. An empty loadout means the default rule (the built unit's own pick); the first
# edit starts from those default slots. confirm() writes the draft and returns UnitState.validate messages verbatim.
# ==============================================================================

var member_id: String = ""
var class_slots: Array[String] = []
var extra_slots: Array[String] = []
var using_default: bool = true
var _original: Dictionary = {}

func _init(p_member: String) -> void:
	member_id = p_member
	var m: UnitState = PartyPreview.member(member_id)
	if m != null:
		_original = m.loadout.duplicate(true)
		cancel()

func _member() -> UnitState:
	return PartyPreview.member(member_id)

func _built(state: UnitState) -> Combatant:
	return PartyBuilder.build(state, false)

func cancel() -> void:
	var cls: Array[String] = JsonUtil.strings(_original.get("class", []))
	var ext: Array[String] = JsonUtil.strings(_original.get("extra", []))
	using_default = cls.is_empty() and ext.is_empty()
	class_slots = cls
	extra_slots = ext
	if using_default:
		_fill_default()

func _fill_default() -> void:
	var work: UnitState = _member().duplicate_state()
	work.loadout = {"class": [], "extra": []}
	var c: Combatant = _built(work)
	class_slots = c.active_class_families.duplicate() if c != null else ([] as Array[String])
	extra_slots = c.active_extra_families.duplicate() if c != null else ([] as Array[String])

func draft_loadout() -> Dictionary:
	if using_default:
		return {"class": [] as Array[String], "extra": [] as Array[String]}
	return {"class": class_slots.duplicate(), "extra": extra_slots.duplicate()}

func has_draft() -> bool:
	return not JsonUtil.deep_equal(draft_loadout(), {"class": JsonUtil.strings(_original.get("class", [])), "extra": JsonUtil.strings(_original.get("extra", []))})

# Owned non-basic families: {"class": [...], "extra": [...]} rows {family, name, grade, tiers: [{tier, name, unlock_level, castable}]}.
func families() -> Dictionary:
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

# Basic families (always active, never slotted).
func basics() -> Array[String]:
	var out: Array[String] = []
	var c: Combatant = _built(_member())
	if c == null:
		return out
	for id: String in c.known_skills + c.granted_skills:
		var fam: String = SkillCatalog.family_of(id)
		if SkillCatalog.is_basic_family(fam) and not out.has(fam):
			out.append(fam)
	return out

func _slots(group: String) -> Array[String]:
	return class_slots if group == "class" else extra_slots

func _limit(group: String) -> int:
	return StatRules.MAX_CLASS_SLOTS if group == "class" else StatRules.MAX_EXTRA_SLOTS

# Puts `family` into slot `index` of `group` (index past the end appends); limits are checked by confirm's validator,
# basics are refused here.
func slot(family: String, group: String, index: int) -> Array[String]:
	var errors: Array[String] = []
	if not ["class", "extra"].has(group):
		errors.append("Unknown slot group '%s'." % group)
		return errors
	if SkillCatalog.is_basic_family(family):
		errors.append("'%s' is a basic family (always active, no slot)." % family)
		return errors
	using_default = false
	var list: Array[String] = _slots(group)
	class_slots.erase(family)
	extra_slots.erase(family)
	if index >= 0 and index < list.size():
		list[index] = family
	else:
		list.append(family)
	return errors

func unslot(group: String, index: int) -> Array[String]:
	var errors: Array[String] = []
	var list: Array[String] = _slots(group)
	if index < 0 or index >= list.size():
		errors.append("No family in that slot.")
		return errors
	using_default = false
	list.remove_at(index)
	return errors

func use_default() -> void:
	using_default = true
	_fill_default()

func confirm() -> Array[String]:
	var m: UnitState = _member()
	var work: UnitState = m.duplicate_state()
	work.loadout = draft_loadout()
	var errors: Array[String] = []
	errors.assign(DictUtil.array_of(work.validate_split()["loadout"]))
	if errors.is_empty():
		m.loadout = work.loadout.duplicate(true)
		_original = m.loadout.duplicate(true)
	return errors

func limit(group: String) -> int:
	return _limit(group)
