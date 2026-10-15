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
const BASE_AP: int = 10

var max_ap: int = BASE_AP            # AP available each round (restored at round start)
var ap: int = BASE_AP                # runtime AP: spent as planned commands start, regained by Rest
var statuses: Dictionary = {}           # active this round: status name -> true (see StatusEffects)
var pending_statuses: Dictionary = {}   # inflicted this round, active next round
var status_rounds_left: Dictionary = {} # active status -> rounds left, this one included (multi-round buffs / bleed)
var pending_status_rounds: Dictionary = {}   # queued status -> rounds it will last once active
var riposted: Dictionary = {}           # attackers this unit already countered this round (RiposteRules)
var status_resist: Dictionary = {}      # passive resistance per status (+%; 100 = immune)
var ap_cost_modifier: int = 0       # Effects shifting every skill's AP cost (base cost = skill level)
var power_override: int = 0    # authored strike power (> 0 replaces weapon + unit power; see DamageCalculator)
var armor_override: int = -1    # authored armor rating (>= 0 replaces gear mitigation + STR / 4)
var lore_ref: String = ""           # world character it plays (WorldCatalog qualified id), empty for plain catalog units
var known_skills: Array[String] = []   # skills this unit may plan: class path up to its level + extras (empty = unrestricted)
var granted_skills: Array[String] = []  # skills learned beyond the class path (passives, in-game events); persist on the unit
var active_class_families: Array[String] = []  # active loadout: class skill families (max StatRules.MAX_CLASS_SLOTS)
var active_extra_families: Array[String] = []  # active loadout: extra-slot families (max StatRules.MAX_EXTRA_SLOTS)
var active_families: Array[String] = []        # class + extra families; empty = no restriction (hand-built units)
var loadout_locked: bool = false               # set by CombatSession while a battle runs: the loadout can't change
var passives: Array[String] = []        # ids of the passives applied to this unit (see Passives)
var ap_regen_bonus: int = 0             # extra AP regained each round (passives)
var potency_bonus: Dictionary = {}      # skill family id -> float bonus (+0.25 = +25% magnitude); empty = none (see potency_for)
var hp_scale: float = 1.0               # tier multiplier on class-derived HP
var power_scale: float = 1.0            # tier multiplier on attack power (DamageCalculator)
var tier: String = "hero"               # StatRules.TIERS key
var catalog_id: String = ""         # ContentCatalog unit id it was spawned from (empty = built in code)
var carry_speed_penalty: int = 0   # Party overload penalty applied by PartyCarry (shared by all units)
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

# Ranged units can't use their ranged attack point-blank: they need distance 2+
# (adjacent foes get an off-hand melee attack instead).
func min_attack_range() -> int:
	return 2 if attack_range > 1 else 1

# Move, Rest and other system skills are always known; class-derived units know their
# class path (up to their level), learned extras and granted skills.
func knows_skill(skill_id: String) -> bool:
	if not owns_skill(skill_id):
		return false
	# An active loadout hides owned skills whose family is not slotted (basics and engine skills stay).
	return active_families.is_empty() or SkillCatalog.is_always_active(skill_id) or active_families.has(SkillCatalog.family_of(skill_id))

# Owned (class path + extras + learned), regardless of the active loadout.
func owns_skill(skill_id: String) -> bool:
	if known_skills.has(skill_id) or granted_skills.has(skill_id) or SkillCatalog.is_system(skill_id):
		return true
	# Unrestricted (hand-built) units know every skill except those that must be learned explicitly.
	return known_skills.is_empty() and not SkillCatalog.is_learned_only(skill_id)

# Non-basic families this unit owns (class path + extras + learned), in owned-list order.
func owned_families() -> Array[String]:
	var out: Array[String] = []
	var ids: Array[String] = []
	ids.append_array(known_skills)
	ids.append_array(granted_skills)
	for id: String in ids:
		var fam: String = SkillCatalog.family_of(id)
		if not out.has(fam) and not SkillCatalog.is_always_active(id):
			out.append(fam)
	return out

# Sets the active loadout (outside combat only). Returns an error text, empty on success; on error nothing changes.
func set_loadout(class_families: Array[String], extra_families: Array[String]) -> String:
	if loadout_locked:
		return "The loadout can't be changed during combat."
	if class_families.size() > StatRules.MAX_CLASS_SLOTS:
		return "At most %d class skill families can be active (got %d)." % [StatRules.MAX_CLASS_SLOTS, class_families.size()]
	if extra_families.size() > StatRules.MAX_EXTRA_SLOTS:
		return "At most %d extra skill families can be active (got %d)." % [StatRules.MAX_EXTRA_SLOTS, extra_families.size()]
	var owned: Array[String] = owned_families()
	var seen: Array[String] = []
	for fam: String in class_families + extra_families:
		if seen.has(fam):
			return "Family '%s' is listed twice." % fam
		seen.append(fam)
		if SkillCatalog.is_basic_family(fam):
			return "'%s' is a basic family (always active, no slot)." % fam
		if not owned.has(fam):
			return "This unit does not own the skill family '%s'." % fam
	active_class_families = class_families.duplicate()
	active_extra_families = extra_families.duplicate()
	_refresh_active()
	return ""

# Back to "no restriction" (every owned skill usable). Outside combat only.
func clear_loadout() -> String:
	if loadout_locked:
		return "The loadout can't be changed during combat."
	active_class_families.clear()
	active_extra_families.clear()
	_refresh_active()
	return ""

func _refresh_active() -> void:
	active_families.clear()
	active_families.append_array(active_class_families)
	active_families.append_array(active_extra_families)

# Multiplier a unit applies to the magnitude of a skill of `family` (1.0 + bonus; 1.0 = no bonus).
# Gear affixes / passives ("skill_potency") fill potency_bonus; the planner multiplies it into the
# skill's power_mult (attacks) or heal (support). DamageCalculator.compute never sees it.
func potency_for(family: String) -> float:
	return 1.0 + DictUtil.float_of(potency_bonus, family)

# The weapon type that counts for skill gating: the main-hand weapon's type, or UNARMED_TOKEN with an empty main hand.
func wielded_type() -> String:
	var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
	return weapon.weapon_type if weapon != null else SkillDef.UNARMED_TOKEN

# Known AND trained for: skills with min_proficiency need the right gear type at that
# proficiency level (class-derived units only; hand-built units are unrestricted).
# Unarmed (no main-hand weapon) is the token `unarmed`; its proficiency is the class's `fist` level.
func can_use(skill: SkillDef) -> bool:
	if not knows_skill(skill.id):
		return false
	if known_skills.is_empty() or skill.min_proficiency <= 0 or stats == null or stats.proficiency == null:
		return true
	var type: String = wielded_type()
	if not skill.weapon_types.is_empty() and not skill.weapon_types.has(type):
		return false
	return stats.proficiency.level_of(SkillDef.UNARMED_PROFICIENCY if type == SkillDef.UNARMED_TOKEN else type) >= skill.min_proficiency

# Learns a skill beyond the class path (an in-game event); persists on the unit.
func learn_skill(skill_id: String) -> void:
	if not granted_skills.has(skill_id):
		granted_skills.append(skill_id)
		# With an active loadout, a newly learned non-basic family takes a free extra slot (else it stays owned but inactive).
		var fam: String = SkillCatalog.family_of(skill_id)
		if not active_families.is_empty() and not SkillCatalog.is_always_active(skill_id) and not active_families.has(fam) and active_extra_families.size() < StatRules.MAX_EXTRA_SLOTS:
			active_extra_families.append(fam)
			_refresh_active()

# Armor burden after training (Proficiency.burden_reduction per worn piece).
func worn_burden() -> int:
	var total: int = 0
	if inventory == null:
		return 0
	for eq: Equipment in inventory._iter_equipped_gear():
		var cut: int = stats.proficiency.burden_reduction(eq.weapon_type) if stats != null and stats.proficiency != null else 0
		total += maxi(0, eq.burden - cut)
	return total

func has_status(status: String) -> bool:
	return statuses.has(status)

func remove_status(status: String) -> void:
	statuses.erase(status)
	status_rounds_left.erase(status)
	pending_statuses.erase(status)
	pending_status_rounds.erase(status)

# Debuff resistance (%): from INT via the stat sheet.
func debuff_resistance() -> float:
	return stats.get_debuff_resistance(get_weapon_context()) if stats != null else 0.0

# Tiles one Move command can cover (one block): faster units cover more ground.
# Haste adds StatusEffects.HASTE_TILES on top of the 1-4 base.
func move_per_block() -> int:
	var base: int = clampi(floori(float(speed + 2) / 5.0), 1, 4)
	return base + (StatusEffects.HASTE_TILES if has_status(StatusEffects.HASTE) else 0)

# Stat-context dictionary used for weapon-conditional bonuses (passives etc.).
func get_weapon_context() -> Dictionary:
	var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
	return {"weapon": weapon.weapon_type} if weapon != null else {}

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

		var burden: int = worn_burden()
		var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
		var base_rng: int = weapon.base_range if weapon != null else 1
		var ctx: Dictionary = {"weapon": weapon.weapon_type} if weapon != null else {}
		max_hp = maxi(1, roundi(float(stats.get_max_hp(ctx)) * hp_scale))
		hp = mini(hp, max_hp)
		speed = maxi(1, stats.get_initiative_speed(burden, ctx) - carry_speed_penalty)
		attack_range = stats.get_effective_range(base_rng, ctx)

# Attack power (weapon power + unit power, or the authored strike power): see DamageCalculator.
func get_effective_power(is_ranged: bool = false, use_offhand: bool = false) -> int:
	return DamageCalculator.power_of(self, is_ranged, use_offhand)
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
		remove_status(StatusEffects.RIPOSTE)   # riposte is a stance-bound buff
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
	riposted.clear()
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