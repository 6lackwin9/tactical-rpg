class_name MasterySystem
extends RefCounted

# ==============================================================================
# Tactical RPG - Mastery & Horizontal Progression System
# Tracks weapon proficiencies, lore backgrounds, and faction alignments
# ==============================================================================

signal mastery_level_changed(category: String, id: String, new_level: int)

# Mastery levels typically scale from 0 (untrained) to 5 (master)
var weapon_masteries: Dictionary = {}  # weapon_type (String) -> level (int)
var lore_masteries: Dictionary = {}    # lore_id (String) -> level (int)
var faction_scores: Dictionary = {}    # faction_id (String) -> score (int)

func set_weapon_mastery(weapon_type: String, level: int) -> void:
	var clean_type: String = weapon_type.to_lower()
	weapon_masteries[clean_type] = maxi(0, level)
	mastery_level_changed.emit("weapon", clean_type, weapon_masteries[clean_type])

func get_weapon_mastery(weapon_type: String) -> int:
	var clean_type: String = weapon_type.to_lower()
	if weapon_masteries.has(clean_type):
		var raw: Variant = weapon_masteries[clean_type]
		if raw is int:
			return raw
	return 0

func set_lore_mastery(lore_id: String, level: int) -> void:
	var clean_id: String = lore_id.to_lower()
	lore_masteries[clean_id] = maxi(0, level)
	mastery_level_changed.emit("lore", clean_id, lore_masteries[clean_id])

func get_lore_mastery(lore_id: String) -> int:
	var clean_id: String = lore_id.to_lower()
	if lore_masteries.has(clean_id):
		var raw: Variant = lore_masteries[clean_id]
		if raw is int:
			return raw
	return 0

func set_faction_score(faction_id: String, score: int) -> void:
	var clean_id: String = faction_id.to_lower()
	faction_scores[clean_id] = score

func get_faction_score(faction_id: String) -> int:
	var clean_id: String = faction_id.to_lower()
	if faction_scores.has(clean_id):
		var raw: Variant = faction_scores[clean_id]
		if raw is int:
			return raw
	return 0

# Weapon mastery allows wielding higher stat requirement weapons for character level
# Each mastery rank grants a 2-point requirement discount
func get_weapon_requirement_discount(weapon_type: String) -> int:
	var rank: int = get_weapon_mastery(weapon_type)
	return rank * 2

# Conditional stat bonuses applied when specific conditions/contexts are met
func get_conditional_stat_bonus(stat_name: String, context: Dictionary) -> int:
	var bonus: int = 0
	var clean_stat: String = stat_name.to_upper()

	# 1. Weapon-specific mastery stat bonuses
	if context.has("weapon"):
		var w_type: String = str(context["weapon"]).to_lower()
		var rank: int = get_weapon_mastery(w_type)
		if rank > 0:
			match w_type:
				"sword", "axe", "mace", "hammer":
					if clean_stat == "STR":
						bonus += rank * 2
				"bow", "dagger", "crossbow":
					if clean_stat == "DEX":
						bonus += rank * 2
				"staff", "wand", "orb":
					if clean_stat == "INT":
						bonus += rank * 2
				"shield":
					if clean_stat == "CONS":
						bonus += rank * 2

	# 2. Lore / Background knowledge bonuses
	if context.has("biome") and clean_stat == "KNOW":
		var biome: String = str(context["biome"]).to_lower()
		var lore_rank: int = get_lore_mastery(biome)
		bonus += lore_rank * 2

	if context.has("topic") and clean_stat == "KNOW":
		var topic: String = str(context["topic"]).to_lower()
		var lore_rank: int = get_lore_mastery(topic)
		bonus += lore_rank * 2

	return bonus

func duplicate_system() -> MasterySystem:
	var copy := MasterySystem.new()
	copy.weapon_masteries = weapon_masteries.duplicate()
	copy.lore_masteries = lore_masteries.duplicate()
	copy.faction_scores = faction_scores.duplicate()
	return copy
