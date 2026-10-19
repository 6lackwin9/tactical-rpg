class_name Consumable
extends Item

# ==============================================================================
# Tactical RPG - Consumable Item Model
# Usable in combat via ItemCommand (Utility Stance) or during exploration
# ==============================================================================

enum ConsumableType {
	HEALTH_POTION,
	STAMINA_BREW,
	ANTIDOTE,
	THROWABLE_BOMB,
	FOCUS_INCENSE
}

var consumable_type: ConsumableType = ConsumableType.HEALTH_POTION
var heal_amount: int = 0
var damage_amount: int = 0
var buff_stat: String = ""
var buff_percent: float = 0.0
var buff_duration_rounds: int = 0
var block_cost: int = 1
var is_combat_usable: bool = true
var is_exploration_usable: bool = true
var use_reach: int = REACH_TYPE_DEFAULT   # ITEMUSE1 1.2: -1 = reach_for(consumable_type); 0 = self only

const REACH_TYPE_DEFAULT: int = -1
const REACH_MAX: int = 4
# Battle reach per type (ITEMUSE1 1.2). Types missing here (STAMINA_BREW, ANTIDOTE) are not offered in battle.
const TYPE_REACH: Dictionary = {
	ConsumableType.HEALTH_POTION: 1,    # self or an adjacent ally
	ConsumableType.FOCUS_INCENSE: 0,    # self
	ConsumableType.THROWABLE_BOMB: 3,   # one foe
}

func _init(
	p_id: String = "",
	p_name: String = "Potion",
	p_type: ConsumableType = ConsumableType.HEALTH_POTION,
	p_heal: int = 0,
	p_damage: int = 0,
	p_buff_stat: String = "",
	p_buff_pct: float = 0.0,
	p_duration: int = 0,
	p_block_cost: int = 1,
	p_weight: float = 0.5,
	p_value: int = 15,
	p_rarity: ItemRarity = ItemRarity.COMMON
) -> void:
	super._init(p_id, p_name, ItemType.CONSUMABLE, p_weight, p_value, p_rarity, "", 5)
	consumable_type = p_type
	heal_amount = p_heal
	damage_amount = p_damage
	buff_stat = p_buff_stat
	buff_percent = p_buff_pct
	buff_duration_rounds = p_duration
	block_cost = p_block_cost

# The type's default battle reach; -1 when the type is not offered in battle.
static func reach_for(p_type: ConsumableType) -> int:
	return DictUtil.to_int(TYPE_REACH.get(p_type, REACH_TYPE_DEFAULT))

# True when a consumable of `p_type` may be used in battle (ITEMUSE1 1.2).
static func offered_in_battle(p_type: ConsumableType) -> bool:
	return TYPE_REACH.has(p_type)

# This item's battle reach: its own use_reach, else the type default (-1 = not usable in battle).
func reach() -> int:
	return use_reach if use_reach >= 0 else reach_for(consumable_type)

func apply_to_combatant(target: Combatant) -> Dictionary:
	var outcome: Dictionary = {
		"healed": 0,
		"damage": 0,
		"buff_applied": false,
		"description": ""
	}

	if target == null:
		return outcome

	if heal_amount > 0 and target.is_alive():
		var before_hp: int = target.hp
		target.heal(heal_amount)
		outcome["healed"] = target.hp - before_hp
		outcome["description"] = "Restored %d HP" % outcome["healed"]

	if damage_amount > 0 and target.is_alive():
		var dealt: int = target.take_damage(damage_amount)
		outcome["damage"] = dealt
		outcome["description"] = "Inflicted %d damage" % dealt

	if buff_percent > 0.0 and target.is_alive():
		target.apply_attack_boost(1.0 + buff_percent, buff_duration_rounds)
		outcome["buff_applied"] = true
		outcome["description"] = "Applied +%d%% %s boost for %d rounds" % [int(buff_percent * 100), buff_stat, buff_duration_rounds]

	return outcome
