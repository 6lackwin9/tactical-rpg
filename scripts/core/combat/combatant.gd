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
var status_magnitudes: Dictionary = {}  # active status -> magnitude (unit from the registry row; see StatusEffects.has_magnitude)
var pending_status_magnitudes: Dictionary = {}   # queued status -> magnitude once active
var status_sources: Dictionary = {}     # active status -> name of the unit that last applied it (S12S-L)
var pending_status_sources: Dictionary = {}   # queued status -> name of the unit that last applied it
var riposted: Dictionary = {}         # attackers this unit already countered this round (RiposteRules)
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
var loadout_locked: bool = false:              # set by CombatSession while a battle runs: the loadout and skill unlocks can't change
	set(value):
		if value:
			_warm_unlock_cache()
		loadout_locked = value
# SK2 skill unlocks. path_levels = copy of the class path ({level, skill}); empty = no level gate (hand-built units).
var path_levels: Array[Dictionary] = []
var unlock_bonus_family: Dictionary = {}   # family id -> levels earlier (passives; max wins, never summed)
var unlock_bonus_group: Dictionary = {}    # skill group -> levels earlier (passives)
var _path_unlock: Dictionary = {}          # path family -> base unlock level U (lowest step level)
var _unlock_cache: Dictionary = {}         # family -> effective unlock E; cleared by refresh_path_skills, frozen while locked
var passives: Array[String] = []        # ids of the passives applied to this unit (see Passives)
var ap_regen_bonus: int = 0             # extra AP regained each round (passives)
var potency_bonus: Dictionary = {}      # skill family id -> float bonus (+0.25 = +25% magnitude); empty = none (see potency_for)
var potency_group: Dictionary = {}      # skill group ("spell" / "heal") -> float bonus from passives
# Gear-sourced bonuses (item `effects`): cleared and rebuilt from the equipment by rebuild_gear_effects, never permanent.
var gear_potency_family: Dictionary = {}   # skill family id -> float bonus
var gear_potency_group: Dictionary = {}    # skill group -> float bonus
var gear_status_resist: Dictionary = {}    # status -> resistance %
var gear_status_potency: int = 0           # +points of hit chance on statuses inflicted by attack skills
var gear_spell_reach: int = 0              # raw sum of gear spell_reach (read through spell_reach_bonus, which clamps it)
var passive_status_potency: int = 0        # the same from passives
var passive_spell_reach: int = 0
var foresight_source: String = ""          # id of the first applied passive with the `foresight` effect ("" = none; INTENT1)
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
	# Owning a class-path or extra family owns all its tiers (their castability is tier_unlocked's business).
	# An explicitly learned skill (granted_skills) is owned as that exact row only.
	if not path_levels.is_empty():   # only class-derived units own whole ladders; hand-built units own the exact rows they list
		var fam: String = SkillCatalog.family_of(skill_id)
		for id: String in known_skills:
			if SkillCatalog.family_of(id) == fam:
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
	return 1.0 + DictUtil.float_of(potency_bonus, family) + DictUtil.float_of(gear_potency_family, family)

# potency_for(family) plus the bonuses of every skill group `skill` belongs to (passives and gear).
# SkillDef.power_for and SupportRules.heal_amount read this; with no bonuses it equals potency_for.
func potency_for_skill(skill: SkillDef) -> float:
	var total: float = potency_for(skill.family_id())
	for group: String in Passives.GROUPS:
		if skill.in_group(group):
			total += DictUtil.float_of(potency_group, group) + DictUtil.float_of(gear_potency_group, group)
	return total

# Extra hit-chance points on statuses inflicted by this unit's attack skills.
func status_potency_bonus() -> int:
	return gear_status_potency + passive_status_potency

# Extra maximum reach of spell skills: gear is clamped to Passives.GEAR_REACH_CAP, passives add on top.
func spell_reach_bonus() -> int:
	return mini(gear_spell_reach, Passives.GEAR_REACH_CAP) + passive_spell_reach

# True when an applied passive carries the `foresight` effect: enemies targeting this unit show their whole round plan
# (IntentRules.reveals, CombatSession.enemy_intents). Display only; never read by planning or resolution.
func has_foresight() -> bool:
	return not foresight_source.is_empty()

# Resistance % to `status` from passives and gear (100 = immune).
func resist_for(status: String) -> int:
	return mini(100, DictUtil.int_of(status_resist, status) + DictUtil.int_of(gear_status_resist, status))

# Clears every gear-sourced bonus and re-applies the `effects` of the equipped items (called on equip changes), then
# the effects of their unlocked tiers (source "gear:<id>:tier<n>", n = 1-based tier position; FORGE1_ITEMS 4.2).
func rebuild_gear_effects() -> void:
	gear_potency_family.clear()
	gear_potency_group.clear()
	gear_status_resist.clear()
	gear_status_potency = 0
	gear_spell_reach = 0
	if inventory == null:
		return
	for eq: Equipment in inventory._iter_equipped_gear():
		Passives.apply_effects(self, eq.effects, "gear:" + eq.id, true)
	if stats == null:
		return
	for eq: Equipment in inventory._iter_equipped_gear():
		for n: int in eq.get_active_tier_numbers(stats, stats.proficiency):
			var fx: Array = eq.tier_effects(n)
			if not fx.is_empty():
				Passives.apply_effects(self, fx, "gear:%s:tier%d" % [eq.id, n], true)

# Adds the stat bonuses of the unlocked gear tiers as Equipment.TIER_SOURCE modifiers (old ones removed first). Tiers are
# evaluated on stats without tier modifiers, so the result never depends on itself (FORGE1_ITEMS 4.3).
func rebuild_gear_tier_stats() -> void:
	if stats == null:
		return
	stats.remove_modifier(Equipment.TIER_SOURCE)
	if inventory == null:
		return
	var totals: Dictionary = {}
	for eq: Equipment in inventory._iter_equipped_gear():
		var bonuses: Dictionary = eq.get_tier_stat_bonuses(stats, stats.proficiency)
		for stat: Variant in bonuses:
			totals[stat] = DictUtil.int_of(totals, str(stat)) + DictUtil.to_int(bonuses[stat])
	for stat: Variant in totals:
		var value: int = DictUtil.to_int(totals[stat])
		if value != 0:
			stats.add_modifier(StatModifier.new(str(stat), value, 0.0, Equipment.TIER_SOURCE, "always", -1))

# The weapon type that counts for skill gating: the main-hand weapon's type, or UNARMED_TOKEN with an empty main hand.
func wielded_type(hand: SkillDef.WeaponHand = SkillDef.WeaponHand.MAIN) -> String:
	if inventory == null:
		return SkillDef.UNARMED_TOKEN
	var main_weapon: Equipment = inventory.get_main_weapon()
	var weapon: Equipment = main_weapon
	if hand == SkillDef.WeaponHand.OFF:
		# The off hand: its weapon, or bare hands (also when a two-handed main weapon occupies both hands).
		weapon = null if main_weapon != null and main_weapon.is_two_handed else inventory.get_offhand_weapon()
	return weapon.weapon_type if weapon != null else SkillDef.UNARMED_TOKEN

# Known AND trained for: skills with min_proficiency need the right gear type at that
# proficiency level (class-derived units only; hand-built units are unrestricted).
# The checked weapon is the one the skill uses (SkillDef.weapon_hand): main = main hand, off = off hand or bare hands,
# both = both weapons must pass. Unarmed (no weapon in that hand) is the token `unarmed`; its proficiency is the class's `fist` level.
func can_use(skill: SkillDef) -> bool:
	if not knows_skill(skill.id) or not tier_unlocked(skill):
		return false
	if known_skills.is_empty() or skill.min_proficiency <= 0 or stats == null or stats.proficiency == null:
		return true
	var hands: Array[SkillDef.WeaponHand] = [SkillDef.WeaponHand.MAIN]
	if skill.weapon_hand == SkillDef.WeaponHand.OFF:
		hands = [SkillDef.WeaponHand.OFF]
	elif skill.weapon_hand == SkillDef.WeaponHand.BOTH:
		hands = [SkillDef.WeaponHand.MAIN, SkillDef.WeaponHand.OFF]
	for hand: SkillDef.WeaponHand in hands:
		var type: String = wielded_type(hand)
		if not skill.weapon_types.is_empty() and not skill.weapon_types.has(type):
			return false
		if stats.proficiency.level_of(SkillDef.UNARMED_PROFICIENCY if type == SkillDef.UNARMED_TOKEN else type) < skill.min_proficiency:
			return false
	return true

# True when every family in the skill's `requires` is owned (an empty list = no requirement). FORGE1 11.3.
func can_own_family(family_id: String) -> bool:
	var head: SkillDef = SkillCatalog.tier_one(family_id)
	if head == null:
		return true
	for required: String in head.requires:
		if not owns_skill(required):
			return false
	return true

# ------------------------------------------------------------------------------
# Skill unlocks (SK2): class path levels decide ownership, tiers open at fixed steps after it.
# ------------------------------------------------------------------------------

# Installs the class path (copied) and recomputes ownership. Used by ContentFactory.unit.
func set_class_path(steps: Array[Dictionary]) -> void:
	path_levels = []
	_path_unlock.clear()
	for step: Dictionary in steps:
		path_levels.append(step.duplicate())
		var fam: String = SkillCatalog.family_of(str(step.get("skill", "")))
		var lvl: int = DictUtil.to_int(step.get("level", 1), 1)
		if not _path_unlock.has(fam) or lvl < DictUtil.int_of(_path_unlock, fam):
			_path_unlock[fam] = lvl
	_unlock_cache.clear()
	refresh_path_skills()

# KNOW for unlocks: base + spend + permanent flat bonuses (gear, passives); temporary combat buffs never count.
func unlock_know() -> int:
	if stats == null:
		return 0
	var total: int = stats.get_base_stat("KNOW")
	for mod: StatModifier in stats.modifiers:
		if mod.stat_name.to_upper() == "KNOW" and mod.is_permanent() and mod.matches_context({}):
			total += mod.flat_bonus
	return total

# Levels earlier a class-path family opens: the larger of the passive bonus (family, then each group it belongs to)
# and the KNOW reduction (advanced families only), capped at StatRules.MAX_UNLOCK_REDUCTION.
func unlock_reduction(family: String) -> int:
	var t1: SkillDef = SkillCatalog.tier_one(family)
	if t1 == null or not _path_unlock.has(family):
		return 0
	var passive_r: int = DictUtil.int_of(unlock_bonus_family, family)
	for group: String in Passives.GROUPS:
		if t1.in_group(group):
			passive_r = maxi(passive_r, DictUtil.int_of(unlock_bonus_group, group))
	var know_r: int = StatRules.know_unlock_reduction(unlock_know()) if t1.grade != SkillDef.Grade.NOVICE else 0
	return mini(StatRules.MAX_UNLOCK_REDUCTION, maxi(passive_r, know_r))

# E(f): the unit level at which `family` is owned. Class-path families: the path level minus the reduction, never below
# the floor (1, or ADVANCED_UNLOCK_FLOOR for advanced families) and never raised. Off-path families (extras, learned):
# StatRules.default_unlock of the tier-1 AP level. Frozen while loadout_locked.
func family_unlock_level(family: String) -> int:
	if _unlock_cache.has(family):
		return DictUtil.int_of(_unlock_cache, family)
	var t1: SkillDef = SkillCatalog.tier_one(family)
	var result: int
	if not _path_unlock.has(family):
		result = StatRules.default_unlock(t1.level if t1 != null else 1)
	else:
		var base: int = DictUtil.int_of(_path_unlock, family)
		var floor_level: int = StatRules.ADVANCED_UNLOCK_FLOOR if t1 != null and t1.grade != SkillDef.Grade.NOVICE else 1
		result = mini(base, maxi(floor_level, base - unlock_reduction(family)))
	_unlock_cache[family] = result
	return result

# Unit level at which `skill` itself is castable: the family's unlock plus the row's tier offset (clamped to MAX_LEVEL).
func skill_unlock_level(skill: SkillDef) -> int:
	return mini(StatRules.MAX_LEVEL, family_unlock_level(skill.family_id()) + skill.tier_offset())

# The level gate of can_use. Basics, engine skills and units without a class path (hand-built) are never gated.
func tier_unlocked(skill: SkillDef) -> bool:
	if path_levels.is_empty() or skill.system or skill.basic or stats == null:
		return true
	return stats.level >= skill_unlock_level(skill)

# Recomputes the owned class-path skills (known_skills) from the unit level, KNOW and unlock passives; extras and
# granted skills are kept. Does nothing while loadout_locked (a battle freezes ownership).
func refresh_path_skills() -> void:
	if loadout_locked or path_levels.is_empty() or stats == null:
		return
	_unlock_cache.clear()
	var owned: Array[String] = []
	for step: Dictionary in path_levels:
		var id: String = str(step.get("skill", ""))
		if stats.level >= family_unlock_level(SkillCatalog.family_of(id)) and not owned.has(id):
			owned.append(id)
	for id: String in known_skills:
		if not owned.has(id) and not _is_path_skill(id):
			owned.append(id)   # extras stay owned
	known_skills = owned

func _is_path_skill(skill_id: String) -> bool:
	for step: Dictionary in path_levels:
		if str(step.get("skill", "")) == skill_id:
			return true
	return false

func _warm_unlock_cache() -> void:
	_unlock_cache.clear()   # never freeze a stale cache
	for fam: Variant in _path_unlock:
		family_unlock_level(str(fam))

# Learns a skill beyond the class path (an in-game event); persists on the unit.
func learn_skill(skill_id: String, check_requires: bool = true) -> void:
	if check_requires and not can_own_family(SkillCatalog.family_of(skill_id)):
		push_warning("learn_skill: '%s' needs %s first." % [skill_id, ", ".join(SkillCatalog.tier_one(SkillCatalog.family_of(skill_id)).requires)])
		return
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
	status_magnitudes.erase(status)
	pending_status_magnitudes.erase(status)
	status_sources.erase(status)
	pending_status_sources.erase(status)

# Outgoing skill power factor from an active weaken: 1 - magnitude/100 (1.0 when not weakened; magnitude capped by the registry).
func weaken_factor() -> float:
	if not has_status(StatusEffects.WEAKEN):
		return 1.0
	var cap: int = StatusRegistry.cap_of(StatusEffects.WEAKEN)
	var magnitude: int = clampi(DictUtil.int_of(status_magnitudes, StatusEffects.WEAKEN), 0, cap if cap > 0 else 100)
	return 1.0 - float(magnitude) / 100.0

# Debuff resistance (%): from INT via the stat sheet.
func debuff_resistance() -> float:
	return stats.get_debuff_resistance(get_weapon_context()) if stats != null else 0.0

# Tiles one Move command can cover (one block): faster units cover more ground.
# Haste adds StatusEffects.HASTE_TILES on top of the 1-4 base, slow takes StatusEffects.SLOW_TILES off
# (floor 1); haste and slow on one unit cancel.
func move_per_block() -> int:
	var base: int = clampi(floori(float(speed + 2) / 5.0), 1, 4)
	var hasted: int = StatusEffects.HASTE_TILES if has_status(StatusEffects.HASTE) else 0
	var slowed: int = StatusEffects.SLOW_TILES if has_status(StatusEffects.SLOW) else 0
	return maxi(1, base + hasted - slowed)

# Magnitude of an ACTIVE status instance (0 when the status is not active or carries none).
func active_magnitude(status: String) -> int:
	return DictUtil.int_of(status_magnitudes, status) if has_status(status) else 0

# Stat-context dictionary used for weapon-conditional bonuses (passives etc.).
func get_weapon_context() -> Dictionary:
	var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
	return {"weapon": weapon.weapon_type} if weapon != null else {}

# Evaluation order (FORGE1_ITEMS 4.3): static gear stat modifiers -> tier stat modifiers (tiers read stats without them)
# -> gear effects incl. tier effects -> derived numbers. Items without tiers give the same stats and effects as before.
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
		rebuild_gear_tier_stats()
	rebuild_gear_effects()
	if stats != null:
		var burden: int = worn_burden()
		var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
		var base_rng: int = weapon.base_range if weapon != null else 1
		var ctx: Dictionary = {"weapon": weapon.weapon_type} if weapon != null else {}
		max_hp = maxi(1, roundi(float(stats.get_max_hp(ctx)) * hp_scale))
		hp = mini(hp, max_hp)
		speed = maxi(1, stats.get_initiative_speed(burden, ctx) - carry_speed_penalty)
		attack_range = stats.get_effective_range(base_rng, ctx)
		refresh_path_skills()   # gear can change KNOW

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