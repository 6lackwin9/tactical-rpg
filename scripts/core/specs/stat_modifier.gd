class_name StatModifier
extends RefCounted

# ==============================================================================
# Tactical RPG - Stat Modifier Model
# Composable modifier supporting flat & percentage adjustments with context gating
# ==============================================================================

var stat_name: String = ""
var flat_bonus: int = 0
var percent_bonus: float = 0.0
var source_id: String = ""
var context_tag: String = ""
var duration_rounds: int = -1  # -1 = permanent / equip / trait, >= 0 = temporary buff

func _init(
	p_stat_name: String = "",
	p_flat: int = 0,
	p_percent: float = 0.0,
	p_source: String = "",
	p_context: String = "",
	p_duration: int = -1
) -> void:
	stat_name = p_stat_name
	flat_bonus = p_flat
	percent_bonus = p_percent
	source_id = p_source
	context_tag = p_context
	duration_rounds = p_duration

func is_permanent() -> bool:
	return duration_rounds < 0

func matches_context(context: Dictionary) -> bool:
	if context_tag.is_empty() or context_tag == "always":
		return true

	# Check context tag pairs like "weapon:sword" or "stance:offensive"
	if context_tag.contains(":"):
		var parts: PackedStringArray = context_tag.split(":")
		if parts.size() == 2:
			var key: String = parts[0]
			var expected_val: String = parts[1]
			if context.has(key):
				return str(context[key]).to_lower() == expected_val.to_lower()
			return false

	# Single tag check
	return context.has(context_tag) and bool(context[context_tag])

func tick_round() -> bool:
	if duration_rounds > 0:
		duration_rounds -= 1
		return duration_rounds == 0
	return false
