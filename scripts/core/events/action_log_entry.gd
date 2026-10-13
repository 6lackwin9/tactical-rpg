class_name ActionLogEntry
extends RefCounted

# ==============================================================================
# Tactical RPG - Action Log Entry Model
# Standardized action event structure for timeline cells and expanded stream log
# ==============================================================================

# Test mode only: log cells explain damage on hover (set by the test harness, off in the real game).
static var show_formulas: bool = false

var round_num: int = 1
var block_num: int = 1
var ref_tag: String = "R1-B01"   # R<X>-B<Y>
var actor_name: String = ""
var actor_is_ally: bool = true
var target_name: String = ""
var target_is_ally: bool = false
var category: String = "ATTACK"  # ATTACK, DEFEND, BUFF, MOVE, TARGET, EVENT
var skill_name: String = ""
var skill_subtype: String = ""
var value_num: int = 0
var outcome_tag: String = "HIT"
var detail_text: String = ""
var status_text: String = ""
var damage: int = 0              # ATTACK: damage actually dealt
var breakdown: String = ""       # ATTACK: how that number came about (formula hover in test mode)
var coord_from: Vector2i = Vector2i(-1, -1)
var coord_to: Vector2i = Vector2i(-1, -1)

func _init(
	p_round: int = 1,
	p_block: int = 1,
	p_category: String = "ATTACK",
	p_actor: String = "",
	p_target: String = "",
	p_detail: String = "",
	p_value: int = 0,
	p_outcome: String = "HIT",
	p_from: Vector2i = Vector2i(-1, -1),
	p_to: Vector2i = Vector2i(-1, -1)
) -> void:
	round_num = p_round
	block_num = p_block
	category = p_category
	actor_name = p_actor
	target_name = p_target
	detail_text = p_detail
	value_num = p_value
	outcome_tag = p_outcome
	coord_from = p_from
	coord_to = p_to
	ref_tag = "R%d-B%02d" % [round_num, block_num]
