---
class: Combatant
extends: RefCounted
---

# Properties
* `name`: String
* `max_hp`: int
* `hp`: int
* `speed`: int
* `attack_range`: int = 1
* `current_stance`: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
* `stance_hold_duration`: int = 0
* `active_target`: Variant = null
* `is_withdrawn`: bool = false
* `attack_boost_multiplier`: float = 1.0
* `attack_boost_rounds_remaining`: int = 0
* `defense_mitigation_ratio`: float = 0.0
* `BASE_AP`: int = 10 (const)
* `max_ap`: int = BASE_AP            # AP available each round (restored at round start)
* `ap`: int = BASE_AP                # runtime AP: spent as planned commands start, regained by Rest
* `statuses`: Dictionary = {}           # active this round: status name -> true (see StatusEffects)
* `pending_statuses`: Dictionary = {}   # inflicted this round, active next round
* `status_rounds_left`: Dictionary = {} # active status -> rounds left, this one included (multi-round buffs / bleed)
* `pending_status_rounds`: Dictionary = {}   # queued status -> rounds it will last once active
* `status_magnitudes`: Dictionary = {}  # active status -> magnitude (unit from the registry row; see StatusEffects.has_magnitude)
* `pending_status_magnitudes`: Dictionary = {}   # queued status -> magnitude once active
* `status_sources`: Dictionary = {}     # active status -> name of the unit that last applied it (S12S-L)
* `pending_status_sources`: Dictionary = {}   # queued status -> name of the unit that last applied it
* `riposted`: Dictionary = {}         # attackers this unit already countered this round (RiposteRules)
* `status_resist`: Dictionary = {}      # passive resistance per status (+%; 100 = immune)
* `ap_cost_modifier`: int = 0       # Effects shifting every skill's AP cost (base cost = skill level)
* `power_override`: int = 0    # authored strike power (> 0 replaces weapon + unit power; see DamageCalculator)
* `armor_override`: int = -1    # authored armor rating (>= 0 replaces gear mitigation + STR / 4)
* `lore_ref`: String = ""           # world character it plays (WorldCatalog qualified id), empty for plain catalog units
* `known_skills`: Array[String] = []   # skills this unit may plan: class path up to its level + extras (empty = unrestricted)
* `granted_skills`: Array[String] = []  # skills learned beyond the class path (passives, in-game events); persist on the unit
* `active_class_families`: Array[String] = []  # active loadout: class skill families (max StatRules.MAX_CLASS_SLOTS)
* `active_extra_families`: Array[String] = []  # active loadout: extra-slot families (max StatRules.MAX_EXTRA_SLOTS)
* `active_families`: Array[String] = []        # class + extra families; empty = no restriction (hand-built units)
* `loadout_locked`: bool = false:              # set by CombatSession while a battle runs: the loadout and skill unlocks can't change
* `path_levels`: Array[Dictionary] = []
* `unlock_bonus_family`: Dictionary = {}   # family id -> levels earlier (passives; max wins, never summed)
* `unlock_bonus_group`: Dictionary = {}    # skill group -> levels earlier (passives)
* `_path_unlock`: Dictionary = {}          # path family -> base unlock level U (lowest step level)
* `_unlock_cache`: Dictionary = {}         # family -> effective unlock E; cleared by refresh_path_skills, frozen while locked
* `passives`: Array[String] = []        # ids of the passives applied to this unit (see Passives)
* `ap_regen_bonus`: int = 0             # extra AP regained each round (passives)
* `potency_bonus`: Dictionary = {}      # skill family id -> float bonus (+0.25 = +25% magnitude); empty = none (see potency_for)
* `potency_group`: Dictionary = {}      # skill group ("spell" / "heal") -> float bonus from passives
* `gear_potency_family`: Dictionary = {}   # skill family id -> float bonus
* `gear_potency_group`: Dictionary = {}    # skill group -> float bonus
* `gear_status_resist`: Dictionary = {}    # status -> resistance %
* `gear_status_potency`: int = 0           # +points of hit chance on statuses inflicted by attack skills
* `gear_spell_reach`: int = 0              # raw sum of gear spell_reach (read through spell_reach_bonus, which clamps it)
* `passive_status_potency`: int = 0        # the same from passives
* `passive_spell_reach`: int = 0
* `foresight_source`: String = ""          # id of the first applied passive with the `foresight` effect ("" = none; INTENT1)
* `hp_scale`: float = 1.0               # tier multiplier on class-derived HP
* `power_scale`: float = 1.0            # tier multiplier on attack power (DamageCalculator)
* `tier`: String = "hero"               # StatRules.TIERS key
* `catalog_id`: String = ""         # ContentCatalog unit id it was spawned from (empty = built in code)
* `carry_speed_penalty`: int = 0   # Party overload penalty applied by PartyCarry (shared by all units)
* `active_buffs`: Array[StatModifier] = []
* `stats`: CharacterStats
* `inventory`: Inventory
* `out`: Array[String] = []
* `ids`: Array[String] = []
* `fam`: String = SkillCatalog.family_of(id)

# Signals
* `health_changed(new_hp: int)`
* `defeated()`
* `withdrawn()`
* `stance_changed(new_stance: CombatTypes.StanceType)`
* `target_changed(new_target: Variant)`

# Methods

### `_init(p_name: String, p_hp: int, p_speed: int, p_attack_range: int = 1, p_stats: CharacterStats = null) -> void`
```logic
name = p_name
max_hp = p_hp
hp = p_hp
speed = p_speed
attack_range = p_attack_range
stats = p_stats if p_stats != null else CharacterStats.new()
inventory = Inventory.new()
```

### `min_attack_range() -> int`
```logic
return 2 if attack_range > 1 else 1
```

### `knows_skill(skill_id: String) -> bool`
```logic
if not owns_skill(skill_id):
	return false
# An active loadout hides owned skills whose family is not slotted (basics and engine skills stay).
return active_families.is_empty() or SkillCatalog.is_always_active(skill_id) or active_families.has(SkillCatalog.family_of(skill_id))
```

### `owns_skill(skill_id: String) -> bool`
```logic
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
```

### `set_loadout(class_families: Array[String], extra_families: Array[String]) -> String`
```logic
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
```

### `clear_loadout() -> String`
```logic
if loadout_locked:
	return "The loadout can't be changed during combat."
active_class_families.clear()
active_extra_families.clear()
_refresh_active()
return ""
```

### `_refresh_active() -> void`
```logic
active_families.clear()
active_families.append_array(active_class_families)
active_families.append_array(active_extra_families)
```

### `potency_for(family: String) -> float`
```logic
return 1.0 + DictUtil.float_of(potency_bonus, family) + DictUtil.float_of(gear_potency_family, family)
```

### `potency_for_skill(skill: SkillDef) -> float`
```logic
var total: float = potency_for(skill.family_id())
for group: String in Passives.GROUPS:
	if skill.in_group(group):
		total += DictUtil.float_of(potency_group, group) + DictUtil.float_of(gear_potency_group, group)
return total
```

### `status_potency_bonus() -> int`
```logic
return gear_status_potency + passive_status_potency
```

### `spell_reach_bonus() -> int`
```logic
return mini(gear_spell_reach, Passives.GEAR_REACH_CAP) + passive_spell_reach
```

### `has_foresight() -> bool`
```logic
return not foresight_source.is_empty()
```

### `resist_for(status: String) -> int`
```logic
return mini(100, DictUtil.int_of(status_resist, status) + DictUtil.int_of(gear_status_resist, status))
```

### `rebuild_gear_effects() -> void`
```logic
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
```

### `rebuild_gear_tier_stats() -> void`
```logic
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
```

### `wielded_type(hand: SkillDef.WeaponHand = SkillDef.WeaponHand.MAIN) -> String`
```logic
if inventory == null:
	return SkillDef.UNARMED_TOKEN
var main_weapon: Equipment = inventory.get_main_weapon()
var weapon: Equipment = main_weapon
if hand == SkillDef.WeaponHand.OFF:
	# The off hand: its weapon, or bare hands (also when a two-handed main weapon occupies both hands).
	weapon = null if main_weapon != null and main_weapon.is_two_handed else inventory.get_offhand_weapon()
return weapon.weapon_type if weapon != null else SkillDef.UNARMED_TOKEN
```

### `can_use(skill: SkillDef) -> bool`
```logic
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
```

### `can_own_family(family_id: String) -> bool`
```logic
var head: SkillDef = SkillCatalog.tier_one(family_id)
if head == null:
	return true
for required: String in head.requires:
	if not owns_skill(required):
		return false
return true
```

### `set_class_path(steps: Array[Dictionary]) -> void`
```logic
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
```

### `unlock_know() -> int`
```logic
if stats == null:
	return 0
var total: int = stats.get_base_stat("KNOW")
for mod: StatModifier in stats.modifiers:
	if mod.stat_name.to_upper() == "KNOW" and mod.is_permanent() and mod.matches_context({}):
		total += mod.flat_bonus
return total
```

### `unlock_reduction(family: String) -> int`
```logic
var t1: SkillDef = SkillCatalog.tier_one(family)
if t1 == null or not _path_unlock.has(family):
	return 0
var passive_r: int = DictUtil.int_of(unlock_bonus_family, family)
for group: String in Passives.GROUPS:
	if t1.in_group(group):
		passive_r = maxi(passive_r, DictUtil.int_of(unlock_bonus_group, group))
var know_r: int = StatRules.know_unlock_reduction(unlock_know()) if t1.grade != SkillDef.Grade.NOVICE else 0
return mini(StatRules.MAX_UNLOCK_REDUCTION, maxi(passive_r, know_r))
```

### `family_unlock_level(family: String) -> int`
```logic
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
```

### `skill_unlock_level(skill: SkillDef) -> int`
```logic
return mini(StatRules.MAX_LEVEL, family_unlock_level(skill.family_id()) + skill.tier_offset())
```

### `tier_unlocked(skill: SkillDef) -> bool`
```logic
if path_levels.is_empty() or skill.system or skill.basic or stats == null:
	return true
return stats.level >= skill_unlock_level(skill)
```

### `refresh_path_skills() -> void`
```logic
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
```

### `_is_path_skill(skill_id: String) -> bool`
```logic
for step: Dictionary in path_levels:
	if str(step.get("skill", "")) == skill_id:
		return true
return false
```

### `_warm_unlock_cache() -> void`
```logic
_unlock_cache.clear()   # never freeze a stale cache
for fam: Variant in _path_unlock:
	family_unlock_level(str(fam))
```

### `learn_skill(skill_id: String, check_requires: bool = true) -> void`
```logic
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
```

### `worn_burden() -> int`
```logic
var total: int = 0
if inventory == null:
	return 0
for eq: Equipment in inventory._iter_equipped_gear():
	var cut: int = stats.proficiency.burden_reduction(eq.weapon_type) if stats != null and stats.proficiency != null else 0
	total += maxi(0, eq.burden - cut)
return total
```

### `has_status(status: String) -> bool`
```logic
return statuses.has(status)
```

### `remove_status(status: String) -> void`
```logic
statuses.erase(status)
status_rounds_left.erase(status)
pending_statuses.erase(status)
pending_status_rounds.erase(status)
status_magnitudes.erase(status)
pending_status_magnitudes.erase(status)
status_sources.erase(status)
pending_status_sources.erase(status)
```

### `weaken_factor() -> float`
```logic
if not has_status(StatusEffects.WEAKEN):
	return 1.0
var cap: int = StatusRegistry.cap_of(StatusEffects.WEAKEN)
var magnitude: int = clampi(DictUtil.int_of(status_magnitudes, StatusEffects.WEAKEN), 0, cap if cap > 0 else 100)
return 1.0 - float(magnitude) / 100.0
```

### `debuff_resistance() -> float`
```logic
return stats.get_debuff_resistance(get_weapon_context()) if stats != null else 0.0
```

### `move_per_block() -> int`
```logic
var base: int = clampi(floori(float(speed + 2) / 5.0), 1, 4)
var hasted: int = StatusEffects.HASTE_TILES if has_status(StatusEffects.HASTE) else 0
var slowed: int = StatusEffects.SLOW_TILES if has_status(StatusEffects.SLOW) else 0
return maxi(1, base + hasted - slowed)
```

### `active_magnitude(status: String) -> int`
```logic
return DictUtil.int_of(status_magnitudes, status) if has_status(status) else 0
```

### `get_weapon_context() -> Dictionary`
```logic
var weapon: Equipment = inventory.get_main_weapon() if inventory != null else null
return {"weapon": weapon.weapon_type} if weapon != null else {}
```

### `sync_from_stats_and_gear() -> void`
```logic
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
```

### `get_effective_power(is_ranged: bool = false, use_offhand: bool = false) -> int`
```logic
return DamageCalculator.power_of(self, is_ranged, use_offhand)
```

### `get_offhand_power() -> int`
```logic
return get_effective_power(false, true)
```

### `take_damage(amount: int, is_mitigated: bool = false) -> int`
```logic
var actual_dmg: int = amount
if not is_mitigated and defense_mitigation_ratio > 0.0:
	actual_dmg = maxi(1, int(roundf(float(amount) * (1.0 - defense_mitigation_ratio))))
var was_alive: bool = is_alive()
hp = maxi(0, hp - actual_dmg)
health_changed.emit(hp)
if was_alive and hp == 0:
	defeated.emit()
return actual_dmg
```

### `heal(amount: int) -> void`
```logic
if not is_alive():
	return
hp = mini(max_hp, hp + amount)
health_changed.emit(hp)
```

### `is_alive() -> bool`
```logic
return hp > 0
```

### `is_active() -> bool`
```logic
return is_alive() and not is_withdrawn
```

### `withdraw() -> void`
```logic
if not is_withdrawn:
	is_withdrawn = true
	withdrawn.emit()
	print("[%s] withdrew from the battlefield!" % name)
```

### `set_target(p_target: Variant) -> void`
```logic
active_target = p_target
target_changed.emit(active_target)
```

### `clear_target() -> void`
```logic
set_target(null)
```

### `has_target() -> bool`
```logic
return active_target != null
```

### `set_stance(p_stance: CombatTypes.StanceType) -> void`
```logic
if current_stance != p_stance:
	current_stance = p_stance
	stance_hold_duration = 0
	remove_status(StatusEffects.RIPOSTE)   # riposte is a stance-bound buff
	stance_changed.emit(current_stance)
```

### `add_buff(mod: StatModifier) -> void`
```logic
active_buffs.append(mod)
```

### `remove_buff(source_id: String) -> void`
```logic
active_buffs = active_buffs.filter(func(m: StatModifier) -> bool: return m.source_id != source_id)
```

### `apply_attack_boost(multiplier: float, duration_rounds: int) -> void`
```logic
attack_boost_multiplier = multiplier
attack_boost_rounds_remaining = duration_rounds
remove_buff("focus_attack_boost")
add_buff(StatModifier.new("ATTACK", 0, multiplier - 1.0, "focus_attack_boost", "always", duration_rounds))
print("[%s] gains Attack Boost (%.1fx) for %d rounds!" % [name, attack_boost_multiplier, attack_boost_rounds_remaining])
```

### `tick_round_buffs() -> void`
```logic
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
```

### `reset_block_transient_state() -> void`
```logic
defense_mitigation_ratio = 0.0
```

