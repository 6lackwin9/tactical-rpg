class_name UnitState
extends RefCounted

# ==============================================================================
# Tactical RPG - Unit State (S7): the persistent snapshot of one party member.
# Plain data with to_dict / from_dict / validate. The level is DERIVED from total_xp (never
# stored); confirmed spend points are permanent. PartyBuilder turns a state into a Combatant.
# ==============================================================================

var id: String = ""
var base_unit: String = ""
var lore_ref: String = ""
var total_xp: int = 0
var spend: Dictionary = {}                 # stat -> confirmed points (missing = 0)
var learned_skills: Array[String] = []     # extras beyond the class path (Combatant.granted_skills)
var passives: Array[String] = []           # permanent passives
var loadout: Dictionary = {"class": [], "extra": []}   # family ids; empty = default loadout
var equipment: Dictionary = {}             # slot name (ContentSchema.LOADOUT_SLOTS) -> item id; unlisted = class default
var carried: Array[String] = []            # item ids in the unit's own carry

static func create(p_id: String, p_base_unit: String) -> UnitState:
	var s := UnitState.new()
	s.id = p_id
	s.base_unit = p_base_unit
	return s

# A state of `p_base_unit` at exactly the XP threshold of `level` (empty spend = the class's automatic spend).
static func at_level(p_base_unit: String, level: int, p_id: String = "") -> UnitState:
	var s: UnitState = create(p_id if not p_id.is_empty() else p_base_unit, p_base_unit)
	s.total_xp = XpCurve.xp_for_level(level)
	return s

# ------------------------------------------------------------------------------
# Derived (never stored)
# ------------------------------------------------------------------------------

func level() -> int:
	return XpCurve.level_for(total_xp)

func spent() -> int:
	return StatRules.total_of(spend)

func unspent() -> int:
	return StatRules.spend_pool(level()) - spent()

func class_id() -> String:
	return str(ContentCatalog.get_entry("units", base_unit).get("class_id", ""))

func archetype() -> ClassArchetype:
	return ContentFactory.class_of(ContentCatalog.get_entry("units", base_unit))

# Non-basic skill families this unit owns at its level: class path, the base unit's extras and learned skills.
func owned_families() -> Array[String]:
	var arch: ClassArchetype = archetype()
	var ids: Array[String] = []
	if arch != null:
		ids.append_array(arch.skills_at(level()))
	ids.append_array(JsonUtil.strings(ContentCatalog.get_entry("units", base_unit).get("extra_skills", [])))
	ids.append_array(learned_skills)
	return ContentFactory.families_of(ids)

# ------------------------------------------------------------------------------
# Serialisation
# ------------------------------------------------------------------------------

func to_dict() -> Dictionary:
	var clean_spend: Dictionary = {}
	for stat: String in StatRules.STATS:
		if DictUtil.to_int(spend.get(stat, 0)) > 0:
			clean_spend[stat] = DictUtil.to_int(spend[stat])
	return {
		"id": id, "base_unit": base_unit, "lore_ref": lore_ref, "total_xp": total_xp,
		"spend": clean_spend, "learned_skills": learned_skills.duplicate(), "passives": passives.duplicate(),
		"loadout": {"class": JsonUtil.strings(loadout.get("class", [])), "extra": JsonUtil.strings(loadout.get("extra", []))},
		"equipment": equipment.duplicate(), "carried": carried.duplicate(),
	}

static func from_dict(d: Dictionary) -> UnitState:
	var s := UnitState.new()
	s.id = str(d.get("id", ""))
	s.base_unit = str(d.get("base_unit", ""))
	s.lore_ref = str(d.get("lore_ref", ""))
	s.total_xp = DictUtil.to_int(d.get("total_xp", 0))
	s.spend = JsonUtil.ints(d.get("spend", {}))
	s.learned_skills = JsonUtil.strings(d.get("learned_skills", []))
	s.passives = JsonUtil.strings(d.get("passives", []))
	var lo: Dictionary = DictUtil.dict_of(d.get("loadout", {}))
	s.loadout = {"class": JsonUtil.strings(lo.get("class", [])), "extra": JsonUtil.strings(lo.get("extra", []))}
	var gear: Dictionary = {}
	for key: Variant in DictUtil.dict_of(d.get("equipment", {})):
		gear[str(key)] = str(DictUtil.dict_of(d.get("equipment", {}))[key])
	s.equipment = gear
	s.carried = JsonUtil.strings(d.get("carried", []))
	return s

func duplicate_state() -> UnitState:
	return UnitState.from_dict(to_dict())

func equals(other: UnitState) -> bool:
	return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())

# ------------------------------------------------------------------------------
# Validation (empty = valid)
# ------------------------------------------------------------------------------

func validate() -> Array[String]:
	var errors: Array[String] = []
	var at: String = "Unit '%s'" % id
	if not JsonUtil.id_ok(id):
		errors.append("Unit id '%s' must be non-empty [a-z0-9_]." % id)
	var entry: Dictionary = ContentCatalog.get_entry("units", base_unit)
	if entry.is_empty():
		errors.append("%s: unknown base unit '%s'." % [at, base_unit])
		return errors
	if str(entry.get("side", "")) != "party":
		errors.append("%s: base unit '%s' is not a party unit." % [at, base_unit])
	var arch: ClassArchetype = ContentFactory.class_of(entry)
	if arch == null or str(entry.get("stat_mode", "derived")) != "derived":
		errors.append("%s: base unit '%s' must be a class-derived unit." % [at, base_unit])
		return errors
	if total_xp < 0:
		errors.append("%s: total_xp can't be negative." % at)
	for key: Variant in spend:
		if not StatRules.STATS.has(str(key)):
			errors.append("%s: spend has unknown stat '%s'." % [at, str(key)])
		elif DictUtil.to_int(spend[key]) < 0:
			errors.append("%s: spend %s can't be negative." % [at, str(key)])
	if spent() > StatRules.spend_pool(level()):
		errors.append("%s: spend totals %d, above the level %d pool of %d." % [at, spent(), level(), StatRules.spend_pool(level())])
	for sk: String in learned_skills:
		if not ContentCatalog.has("skills", sk):
			errors.append("%s: unknown learned skill '%s'." % [at, sk])
	for p: String in passives:
		if not ContentCatalog.has("passives", p):
			errors.append("%s: unknown passive '%s'." % [at, p])
	errors.append_array(_loadout_errors(at))
	errors.append_array(_gear_errors(at, arch))
	for item_id: String in carried:
		if not ContentCatalog.has("items", item_id):
			errors.append("%s: unknown carried item '%s'." % [at, item_id])
	return errors

func _loadout_errors(at: String) -> Array[String]:
	var errors: Array[String] = []
	var cls: Array[String] = JsonUtil.strings(loadout.get("class", []))
	var ext: Array[String] = JsonUtil.strings(loadout.get("extra", []))
	if cls.size() > StatRules.MAX_CLASS_SLOTS:
		errors.append("%s: at most %d class families in the loadout." % [at, StatRules.MAX_CLASS_SLOTS])
	if ext.size() > StatRules.MAX_EXTRA_SLOTS:
		errors.append("%s: at most %d extra families in the loadout." % [at, StatRules.MAX_EXTRA_SLOTS])
	var owned: Array[String] = owned_families()
	var seen: Array[String] = []
	for fam: String in cls + ext:
		if seen.has(fam):
			errors.append("%s: loadout lists family '%s' twice." % [at, fam])
		seen.append(fam)
		if not owned.has(fam):
			errors.append("%s: loadout family '%s' is not owned." % [at, fam])
	return errors

func _gear_errors(at: String, arch: ClassArchetype) -> Array[String]:
	var errors: Array[String] = []
	var prof: Proficiency = arch.make_proficiency()
	for key: Variant in equipment:
		var slot: String = str(key)
		var item_id: String = str(equipment[key])
		if not ContentSchema.LOADOUT_SLOTS.has(slot):
			errors.append("%s: unknown equipment slot '%s'." % [at, slot])
			continue
		if not ContentCatalog.has("items", item_id):
			errors.append("%s: unknown item '%s' in %s." % [at, item_id, slot])
			continue
		var item: Dictionary = ContentCatalog.get_entry("items", item_id)
		if not ContentFactory.item_fits_slot(item, slot.to_upper(), true):   # saved (worn) gear is kept, not re-gated
			errors.append("%s: '%s' can't be equipped in %s." % [at, item_id, slot])
		elif not str(item.get("weapon_type", "")).is_empty() and not prof.can_use(str(item["weapon_type"])):
			errors.append("%s: class has no training in '%s' (%s)." % [at, str(item["weapon_type"]), item_id])
	return errors
