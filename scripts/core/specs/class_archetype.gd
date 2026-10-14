class_name ClassArchetype
extends RefCounted

# ==============================================================================
# Tactical RPG - Class Archetype (the six base classes; data in the content catalog)
# A class is stats + gear + skills, lore-free:
#   base_stats / growth_per_level   follow StatRules (equal budget, equal growth)
#   proficiency                     {gear type: {"level": 1-3, "scaling": 1.0-2.0}}
#                                   what the class can wield and how well (Proficiency)
#   skill_path                      [{"level": n, "skill": id}]: skills learned by level
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
var skill_path: Array[Dictionary] = []  # [{"level": int, "skill": String}]
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

# Skills the class has learned by `level`, in path order.
func skills_at(level: int) -> Array[String]:
	var out: Array[String] = []
	for step: Dictionary in skill_path:
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

static func create_shaman() -> ClassArchetype:
	return create("shaman")

static func create_brawler() -> ClassArchetype:
	return create("brawler")

static func create_scout() -> ClassArchetype:
	return create("scout")

static func create_berserker() -> ClassArchetype:
	return create("berserker")
