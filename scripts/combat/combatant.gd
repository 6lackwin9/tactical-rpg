class_name Combatant
extends RefCounted

signal health_changed(new_hp: int)
signal defeated()
signal withdrawn()
signal stance_changed(new_stance: CombatTypes.StanceType)
signal target_changed(new_target: Variant)

var name: String
var max_hp: int
var hp: int
var speed: int
var attack_range: int = 1

# Stance & Positioning State
var current_stance: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
var stance_hold_duration: int = 0
var active_target: Variant = null
var is_withdrawn: bool = false

# Transient Buff & Defense State
var attack_boost_multiplier: float = 1.0
var attack_boost_rounds_remaining: int = 0
var defense_mitigation_ratio: float = 0.0

func _init(p_name: String, p_hp: int, p_speed: int, p_attack_range: int = 1) -> void:
	name = p_name
	max_hp = p_hp
	hp = p_hp
	speed = p_speed
	attack_range = p_attack_range

func take_damage(amount: int) -> int:
	var actual_dmg: int = amount
	if defense_mitigation_ratio > 0.0:
		actual_dmg = maxi(1, int(roundf(float(amount) * (1.0 - defense_mitigation_ratio))))
	var was_alive: bool = is_alive()
	hp = maxi(0, hp - actual_dmg)
	health_changed.emit(hp)
	if was_alive and hp == 0:
		defeated.emit()
	return actual_dmg

func heal(amount: int) -> void:
	if not is_alive():
		return
	hp = mini(max_hp, hp + amount)
	health_changed.emit(hp)

func is_alive() -> bool:
	return hp > 0

func is_active() -> bool:
	return is_alive() and not is_withdrawn

func withdraw() -> void:
	if not is_withdrawn:
		is_withdrawn = true
		withdrawn.emit()
		print("[%s] withdrew from the battlefield!" % name)

func set_target(p_target: Variant) -> void:
	active_target = p_target
	target_changed.emit(active_target)

func clear_target() -> void:
	set_target(null)

func has_target() -> bool:
	return active_target != null

func set_stance(p_stance: CombatTypes.StanceType) -> void:
	if current_stance != p_stance:
		current_stance = p_stance
		stance_hold_duration = 0
		stance_changed.emit(current_stance)

func apply_attack_boost(multiplier: float, duration_rounds: int) -> void:
	attack_boost_multiplier = multiplier
	attack_boost_rounds_remaining = duration_rounds
	print("[%s] gains Attack Boost (%.1fx) for %d rounds!" % [name, attack_boost_multiplier, attack_boost_rounds_remaining])

func tick_round_buffs() -> void:
	if attack_boost_rounds_remaining > 0:
		attack_boost_rounds_remaining -= 1
		if attack_boost_rounds_remaining <= 0:
			attack_boost_multiplier = 1.0
			print("[%s]'s Attack Boost expired." % name)

func reset_block_transient_state() -> void:
	defense_mitigation_ratio = 0.0