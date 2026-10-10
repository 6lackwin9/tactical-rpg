class_name ActionLogEntry
extends RefCounted

# ==============================================================================
# Tactical RPG - Action Log Entry Model
# Standardized action event structure for timeline cells and expanded stream log
# ==============================================================================

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
var coord_from: Vector2i = Vector2i(-1, -1)
var coord_to: Vector2i = Vector2i(-1, -1)
