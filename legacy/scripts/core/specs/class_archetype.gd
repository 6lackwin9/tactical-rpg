class_name ClassArchetype
extends RefCounted

# ==============================================================================
# Tactical RPG - Class Archetype (the six base classes; data in the content catalog)
# A class is stats + gear + skills, lore-free:
#   base_stats / growth_per_level   follow StatRules (equal budget, equal growth)
#   proficiency                     {gear type: {"level": 1-3, "scaling": 1.0-2.0}}
#                                   what the class can wield and how well (Proficiency)
#   skill_path                      {family id: unlock level} (insertion order = entry position; FORGE1_CLASSES 3.5)
#   loadout                         default gear per slot (item ids); units may override
#   art / ai_role                   token & portrait key, default AI role
# Units (class + level + side + tier + spend) are built from it by ContentFactory;
# passives (from world sources) are never part of a class.
# ==============================================================================

var id: String = "warrior"
var display_name: String = "Warrior"
var description: String = ""
var archetype_name: String:
	get:
		return display_name
var base_stats: Dictionary = {}         # stat (String) -> level-1 base value (int)
var growth_per_level: Dictionary = {}   # stat (String) -> guaranteed gain per level (int)
var proficiency: Dictionary = {}        # gear type -> {"level": int, "scaling": float}
var skill_path: Dictionary = {}         # family head id (String) -> unlock level (int), in entry order
var loadout: Dictionary = {}            # slot key (main_hand ...) -> item id
var art: String = "warrior"
var ai_role: String = "AGGRESSOR"

func _init(p_id: String = "warrior", p_name: String = "Warrior") -> void:
	id = p_id
	display_name = p_name

func get_base_stat(stat_name: String) -> int:
	return DictUtil.int_of(base_stats, stat_name.to_upper(), 10)

func get_growth_for_stat(stat_name: String) -> int:
	return DictUtil.int_of(growth_per_level, stat_name.to_upper(), 0)

# A fresh Proficiency set for a unit of this class (passives add to it).
func make_proficiency() -> Proficiency:
	return Proficiency.from_dict(proficiency)

# Standard unlock levels the Forge offers first (every level a shipped path uses; FORGE1_CLASSES 3.5). Any 1..MAX_LEVEL is valid.
const STANDARD_PATH_LEVELS: Array[int] = [1, 3, 4, 5, 7]

# The path as ordered steps [{"level", "skill"}]: unlock level ascending, then entry position (dict order) ascending.
# With ClassForgeModel.ordered_path (which calls this) the only place that sorts a path.
func path_steps() -> Array[Dictionary]:
	return ordered_steps(skill_path)

static func ordered_steps(path: Dictionary) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	var position: int = 0
	for key: Variant in path:
		rows.append({"level": DictUtil.to_int(path[key], 1), "skill": str(key), "pos": position})
		position += 1
	rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		var la: int = DictUtil.to_int(a["level"])
		var lb: int = DictUtil.to_int(b["level"])
		if la != lb:
			return la < lb
		return DictUtil.to_int(a["pos"]) < DictUtil.to_int(b["pos"])
	)
	var out: Array[Dictionary] = []
	for row: Dictionary in rows:
		out.append({"level": row["level"], "skill": row["skill"]})
	return out

# Pre-CLASS1 list shape [{level, skill}] -> {skill: level}: first position wins, the lowest level of a repeated family wins,
# basics are dropped when `drop_basics` (they are owned by every unit, FORGE1 11.2). Used by the one-release converter.
static func path_from_list(steps: Array, drop_basics: bool = true) -> Dictionary:
	var out: Dictionary = {}
	for raw: Variant in steps:
		var step: Dictionary = DictUtil.dict_of(raw)
		var skill_id: String = str(step.get("skill", ""))
		if skill_id.is_empty():
			continue
		if drop_basics and DictUtil.to_bool(ContentCatalog.get_entry("skills", skill_id).get("basic", false)):
			continue
		var level: Variant = whole_level(step.get("level", 1))
		if not out.has(skill_id):
			out[skill_id] = level
		elif level is int and out[skill_id] is int and DictUtil.to_int(level) < DictUtil.to_int(out[skill_id]):
			out[skill_id] = level
	return out

# A path level as stored: whole numbers (JSON parses them as float) become int; anything else is kept for the validator.
static func whole_level(raw: Variant) -> Variant:
	if raw is float:
		var x: float = raw
		if x == floorf(x):
			return roundi(x)
	return raw

# Path skills (family heads) the class owns by `level` (unreduced path levels), in path_steps() order. Basics are not path
# content: every unit owns them from level 1 through the factory grant (ContentFactory.unit, SkillCatalog.basic_ids()).
func skills_at(level: int) -> Array[String]:
	var out: Array[String] = []
	for step: Dictionary in path_steps():
		if DictUtil.to_int(step.get("level", 1), 1) <= level:
			out.append(str(step.get("skill", "")))
	return out

# Class-based gear check: accessories and untyped gear are free; the rest needs a
# proficiency entry of level 1+ (units may widen it through passives).
func can_equip_gear(eq: Equipment) -> bool:
	if eq == null:
		return false
	if eq.item_type == Item.ItemType.ACCESSORY or eq.slot in [Equipment.EquipSlot.ACCESSORY_1, Equipment.EquipSlot.ACCESSORY_2]:
		return true
	if eq.weapon_type.is_empty() or proficiency.is_empty():
		return true
	return make_proficiency().can_use(eq.weapon_type)

# ------------------------------------------------------------------------------
# Catalog archetypes (res://data/catalog/classes.json, edited in the Content Forge)
# ------------------------------------------------------------------------------

# Class `class_id` from the content catalog (a plain 10-stat class if it is missing).
static func create(class_id: String) -> ClassArchetype:
	if ContentCatalog.has("classes", class_id):
		return ContentFactory.class_archetype(ContentCatalog.get_entry("classes", class_id))
	push_warning("ClassArchetype: unknown class '%s'" % class_id)
	return ClassArchetype.new(class_id, class_id.capitalize())

static func create_warrior() -> ClassArchetype:
	return create("warrior")

static func create_ranger() -> ClassArchetype:
	return create("ranger")
