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
var active_buffs: Array[StatModifier] = []

var stats: CharacterStats
var inventory: Inventory

func _init(p_name: String, p_hp: int, p_speed: int, p_attack_range: int = 1, p_stats: CharacterStats = null) -> void:
	name = p_name
	max_hp = p_hp
	hp = p_hp
	speed = p_speed
	attack_range = p_attack_range
	stats = p_stats if p_stats != null else CharacterStats.new()
	inventory = Inventory.new()

func sync_from_stats_and_gear() -> void:
	if stats != null:
		# Sync equipped gear stat bonuses into character stats modifiers (Requirement 1)
		stats.remove_modifier("gear")
		if inventory != null:
			for eq: Equipment in inventory._iter_equipped_gear():
				for st_k: String in eq.stat_bonuses:
					var val_v: Variant = eq.stat_bonuses[st_k]
					var bonus_val: int = val_v if val_v is int else 0
					if bonus_val != 0:
						stats.add_modifier(StatModifier.new(st_k.to_upper(), bonus_val, 0.0, "gear", "always", -1))

		var burden: int = inventory.get_total_armor_burden() if inventory != null else 0
		var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
		var base_rng: int = weapon.base_range if weapon != null else 1
		var ctx: Dictionary = {"weapon": weapon.weapon_type} if weapon != null else {}
		max_hp = stats.get_max_hp(ctx)
		hp = mini(hp, max_hp)
		speed = stats.get_initiative_speed(burden, ctx)
		attack_range = stats.get_effective_range(base_rng, ctx)

func get_effective_power(is_ranged: bool = false, use_offhand: bool = false) -> int:
	if stats == null:
		return 20
	var weapon: Equipment = null
	if use_offhand and inventory != null:
		weapon = inventory.get_offhand_weapon()
	if weapon == null and inventory != null:
		weapon = inventory.get_main_weapon()

	var ctx: Dictionary = {"weapon": weapon.weapon_type} if weapon != null else {}
	var w_power: int = 0
	if weapon != null:
		if is_ranged:
			if weapon.is_ranged_weapon():
				w_power = weapon.get_effective_equipment_power(stats, stats.mastery)
		else:
			if weapon.is_melee_weapon():
				w_power = weapon.get_effective_equipment_power(stats, stats.mastery)

	var stat_to_scale: String = "STR"
	if weapon != null:
		if is_ranged:
			if weapon.is_ranged_weapon():
				stat_to_scale = weapon.scaling_stat
			else:
				stat_to_scale = "DEX"
		else:
			if weapon.is_melee_weapon():
				stat_to_scale = weapon.scaling_stat
			else:
				stat_to_scale = "STR"
	elif is_ranged:
		stat_to_scale = "DEX"

	match stat_to_scale:
		"DEX":
			return stats.get_ranged_power(w_power, ctx)
		"INT":
			return stats.get_tactical_power(w_power, ctx)
		_:
			return stats.get_melee_power(w_power, ctx)

func get_offhand_power() -> int:
	return get_effective_power(false, true)

func take_damage(amount: int, is_mitigated: bool = false) -> int:
	var actual_dmg: int = amount
	if not is_mitigated and defense_mitigation_ratio > 0.0:
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

func add_buff(mod: StatModifier) -> void:
	active_buffs.append(mod)

func remove_buff(source_id: String) -> void:
	active_buffs = active_buffs.filter(func(m: StatModifier) -> bool: return m.source_id != source_id)

func apply_attack_boost(multiplier: float, duration_rounds: int) -> void:
	attack_boost_multiplier = multiplier
	attack_boost_rounds_remaining = duration_rounds
	remove_buff("focus_attack_boost")
	add_buff(StatModifier.new("ATTACK", 0, multiplier - 1.0, "focus_attack_boost", "always", duration_rounds))
	print("[%s] gains Attack Boost (%.1fx) for %d rounds!" % [name, attack_boost_multiplier, attack_boost_rounds_remaining])

func tick_round_buffs() -> void:
	if attack_boost_rounds_remaining > 0:
		attack_boost_rounds_remaining -= 1
		if attack_boost_rounds_remaining <= 0:
			attack_boost_multiplier = 1.0
			print("[%s]'s Attack Boost expired." % name)

	var expired: Array[StatModifier] = []
	for mod: StatModifier in active_buffs:
		if mod.tick_round():
			expired.append(mod)
	for exp_mod: StatModifier in expired:
		active_buffs.erase(exp_mod)

func reset_block_transient_state() -> void:
	defense_mitigation_ratio = 0.0