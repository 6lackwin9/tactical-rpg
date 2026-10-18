---
class: DojoSession
extends: RefCounted
description: "Tactical RPG - Gear Dojo Session (headless state + rules glue) Owns heroes, training dummy, shared stash, stat allocation and the test log. UI panels only read this object and call its actions; every mutation ends in `changed`, so panels stay independent and can be opened/closed freely."
---

# Properties
* `DUMMY_MAX_HP`: int = 500 (const)
* `MAX_LOG_ENTRIES`: int = 300 (const)
* `STAT_KEYS`: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"] (const)
* `ROUND_AP_REGEN`: int = 4        # AP regained each round, as in battles (const)
* `HERO_AP`: int = 12              # the battle AP pool (const)
* `MAX_PARTY_SIZE`: int = 4 (const)
* `party_allies`: Array[Combatant] = []
* `active_hero`: Combatant = null
* `test_dummy`: Combatant = null
* `shared_stash`: Inventory = null
* `stash_filter`: StashFilter = StashFilter.new()
* `carry_report`: Dictionary = {}
* `force_crit`: bool = false
* `rng`: RandomNumberGenerator = RandomNumberGenerator.new()   # crit rolls (seeded on reset, so benchmarks repeat)
* `RNG_SEED`: int = 1337 (const)
* `selected_skill_id`: String = "strike"
* `dummy_skill_id`: String = "strike"
* `dummy_counter`: bool = false     # the dummy strikes back after every hero skill
* `round_num`: int = 1
* `is_allocating_stats`: bool = false
* `pending_allocated_stats`: Dictionary = {}
* `pending_free_points`: int = 0
* `log_entries`: Array[DojoLogEntry] = []
* `_action_counter`: int = 0
* `_owns_dual_wield`: bool = false
* `_saved_dual_wield`: bool = false
* `entry`: Variant
* `acting`: Combatant = actor if actor != null else active_hero
* `out`: Array[SkillDef] = []
* `out`: Array[SkillDef] = []
* `result`: Array[Equipment] = []
* `main_held`: Equipment = active_hero.inventory.get_main_weapon() if (active_hero != null and active_hero.inventory != null) else null
* `eq`: Equipment = item
* `result`: Array[Consumable] = []
* `con`: Consumable = item

# Signals
* `changed()`
* `entry_logged(entry: DojoLogEntry)`
* `log_cleared()`
* `stash_filter_changed()`
* `stage_fx(kind: String, on_hero: bool, text: String, color: Color)`

# Methods

### `_init(populate: bool = true) -> void`
```logic
if populate:
	_saved_dual_wield = Equipment.allow_plain_dual_wield
	Equipment.allow_plain_dual_wield = true
	_owns_dual_wield = true
	init_data()
else:
	_clear_allocation_state()
```

### `close() -> void`
```logic
if _owns_dual_wield:
	Equipment.allow_plain_dual_wield = _saved_dual_wield
	_owns_dual_wield = false
```

### `_notification(what: int) -> void`
```logic
if what == NOTIFICATION_PREDELETE and _owns_dual_wield:   # inline: methods cannot be called while freeing
	Equipment.allow_plain_dual_wield = _saved_dual_wield
	_owns_dual_wield = false
```

### `static wrap_party(allies: Array[Combatant], stash: Inventory) -> DojoSession`
```logic
var wrapped := DojoSession.new(false)
wrapped.party_allies = allies.duplicate()
wrapped.active_hero = allies[0] if not allies.is_empty() else null
wrapped.shared_stash = stash if stash != null else Inventory.new()
wrapped.carry_report = PartyCarry.apply(wrapped.party_allies, wrapped.shared_stash)
return wrapped
```

### `static weapon_context(hero: Combatant) -> Dictionary`
```logic
return hero.get_weapon_context() if hero != null else {}
```

### `_notify() -> void`
```logic
carry_report = PartyCarry.apply(party_allies, shared_stash)
changed.emit()
```

### `notify_filter_changed() -> void`
```logic
stash_filter_changed.emit()
```

### `set_slot_filter(slot_key: String) -> void`
```logic
stash_filter.slot_filter = slot_key
stash_filter_changed.emit()
```

### `reset_custom_filter() -> void`
```logic
stash_filter.reset_custom()
stash_filter_changed.emit()
```

### `init_data() -> void`
```logic
rng.seed = RNG_SEED
shared_stash = Inventory.new()
ItemCatalog.populate_party_starter_stash(shared_stash)

# Heroes and dummy are catalog units (class, level and loadout in data/catalog/units.json);
# derived units come out synced to their gear at full health.
var warrior: Combatant = ContentFactory.unit(ContentCatalog.get_entry("units", "dojo_warrior"))
var ranger: Combatant = ContentFactory.unit(ContentCatalog.get_entry("units", "dojo_ranger"))

party_allies = [warrior, ranger]
active_hero = warrior

test_dummy = ContentFactory.unit(ContentCatalog.get_entry("units", "training_dummy"))
test_dummy.max_hp = DUMMY_MAX_HP
test_dummy.hp = DUMMY_MAX_HP
test_dummy.defense_mitigation_ratio = 0.0
for hero: Combatant in party_allies:
	hero.max_ap = HERO_AP
	hero.ap = HERO_AP

stash_filter = StashFilter.new()
carry_report = PartyCarry.apply(party_allies, shared_stash)

force_crit = false
selected_skill_id = "strike"
dummy_skill_id = "strike"
dummy_counter = false
round_num = 1
_clear_allocation_state()
```

### `reset() -> void`
```logic
init_data()
log_entries.clear()
_action_counter = 0
log_cleared.emit()
log_action("EVENT", "Dojo Reset", "Heroes, stash, loadouts and dummy restored to baseline", 0, "RESET")
stage_fx.emit("text", true, "↺ Dojo Reset", UiTheme.COL_INFO)
_notify()
```

### `notify_external_change() -> void`
```logic
if active_hero != null:
	active_hero.sync_from_stats_and_gear()
_notify()
```

### `set_active_hero(hero: Combatant) -> void`
```logic
if hero == null or hero == active_hero:
	return
if is_allocating_stats:
	_clear_allocation_state()
active_hero = hero
_notify()
```

### `set_force_crit(enabled: bool) -> void`
```logic
force_crit = enabled
```

### `set_dummy_counter(enabled: bool) -> void`
```logic
dummy_counter = enabled
_notify()
```

### `set_dummy_power(value: int) -> void`
```logic
if test_dummy != null:
	test_dummy.power_override = maxi(0, value)
	_notify()
```

### `_log_failure(skill_name: String, reason: String) -> void`
```logic
log_action("EVENT", skill_name, reason, 0, "FAILED", "[color=#f87171]%s[/color]" % reason)
```

### `clear_log() -> void`
```logic
log_entries.clear()
log_cleared.emit()
```

### `get_action_availability() -> Dictionary`
```logic
var result: Dictionary = {}
if active_hero == null or active_hero.inventory == null:
	return result
var inv: Inventory = active_hero.inventory
var main_w: Equipment = inv.get_main_weapon()
var off_w: Equipment = inv.get_offhand_weapon()
var shield: Equipment = inv.get_shield()
var switch_needed: bool = inv.needs_weapon_switch()
var off_active: bool = inv.is_offhand_active()
var off_offensive: bool = off_w != null and off_w.base_power > 0

# Main hand (melee strike / ranged volley): blocked while the off-hand weapon is active.
var main_ready_tip: String = ""
if off_active:
	main_ready_tip = "%s is active - Switch Weapon (1 block) to draw %s" % [off_w.name, main_w.name]
if main_w == null:
	result["main"] = {"enabled": true, "tip": "Unarmed melee strike against the Dummy"}
	result["ranged"] = {"enabled": false, "tip": "Requires a ranged weapon (e.g. Shortbow)"}
elif main_w.is_ranged_weapon():
	result["main"] = {"enabled": false, "tip": "Cannot melee with ranged weapon (%s)" % main_w.name}
	result["ranged"] = {"enabled": not off_active, "tip": main_ready_tip if off_active else "Ranged volley with %s" % main_w.name}
else:
	result["main"] = {"enabled": not off_active, "tip": main_ready_tip if off_active else "Main-hand strike with %s" % main_w.name}
	result["ranged"] = {"enabled": false, "tip": "Cannot fire a volley with melee weapon (%s)" % main_w.name}

# Off-hand strike: free with 1H main weapons; a 2H main weapon needs the switch first.
if off_offensive:
	if switch_needed and not off_active:
		result["offhand"] = {"enabled": false, "tip": "2H %s is active - Switch Weapon (1 block) to ready %s" % [main_w.name, off_w.name]}
	else:
		result["offhand"] = {"enabled": true, "tip": "Off-hand strike with %s (scales with %s)" % [off_w.name, off_w.scaling_stat]}
elif shield != null:
	result["offhand"] = {"enabled": false, "tip": "%s is a shield (mitigation only)" % shield.name}
else:
	result["offhand"] = {"enabled": false, "tip": "No off-hand weapon equipped"}

# Dual Strike combo: both hands hit, only with a 1H melee main weapon + offensive off-hand weapon.
var can_dual: bool = main_w != null and main_w.is_melee_weapon() and not main_w.is_two_handed and off_offensive
if can_dual:
	result["dual"] = {"enabled": true, "tip": "Combo: strike with %s and %s in one action" % [main_w.name, off_w.name]}
else:
	result["dual"] = {"enabled": false, "tip": "Needs a 1H melee main weapon and an offensive off-hand weapon"}

if switch_needed:
	var next_w: Equipment = main_w if off_active else off_w
	result["switch"] = {"enabled": true, "tip": "1 block: ready %s (2H main + off-hand weapons share one active slot)" % next_w.name}
else:
	result["switch"] = {"enabled": false, "tip": "No switch needed: 1H weapons attack with both hands freely. Pair a 2H weapon with an off-hand weapon to use this."}
return result
```

### `perform_strike(kind: String) -> void`
```logic
if active_hero == null or test_dummy == null or active_hero.inventory == null:
	return
var inv: Inventory = active_hero.inventory
var main_w: Equipment = inv.get_main_weapon()
var off_w: Equipment = inv.get_offhand_weapon()
var title: String = "Main Strike"
var is_ranged: bool = false
var use_offhand: bool = false

match kind:
	"main":
		if main_w != null and main_w.is_ranged_weapon():
			_log_failure(title, "Cannot melee while wielding ranged weapon (%s)" % main_w.name)
			return
		if inv.is_offhand_active():
			_log_failure(title, "%s is active - Switch Weapon (1 block) to draw %s" % [off_w.name, main_w.name])
			return
	"offhand":
		title = "Off-Hand Strike"
		use_offhand = true
		if off_w == null:
			_log_failure(title, "No off-hand weapon equipped")
			return
		if off_w.is_shield() or off_w.base_power == 0:
			_log_failure(title, "%s is a shield (mitigation only, not offensive)" % off_w.name)
			return
		if inv.needs_weapon_switch() and not inv.is_offhand_active():
			_log_failure(title, "2H %s is active - Switch Weapon (1 block) to ready %s" % [main_w.name, off_w.name])
			return
	"ranged":
		title = "Ranged Volley"
		is_ranged = true
		if main_w == null or not main_w.is_ranged_weapon():
			_log_failure(title, "Requires a ranged weapon (wielding: %s)" % (main_w.name if main_w != null else "Unarmed"))
			return
		if inv.is_offhand_active():
			_log_failure(title, "%s is active - Switch Weapon (1 block) to draw %s" % [off_w.name, main_w.name])
			return
	"dual":
		_perform_dual_strike()
		return
	_:
		return

var hit: Dictionary = _apply_hit(title, is_ranged, use_offhand)
var actual_dmg: int = UiTheme.dict_int(hit, "dmg")
log_action("ATTACK", title, str(hit.get("detail", "")), actual_dmg, str(hit.get("outcome", "HIT")), str(hit.get("breakdown", "")), test_dummy.name)
_emit_hit_fx(hit)
_notify()
```

### `_perform_dual_strike() -> void`
```logic
var title: String = "Dual Strike"
var inv: Inventory = active_hero.inventory
var main_w: Equipment = inv.get_main_weapon()
var off_w: Equipment = inv.get_offhand_weapon()
if main_w == null or not main_w.is_melee_weapon() or main_w.is_two_handed:
	_log_failure(title, "Dual Strike needs a 1H melee main-hand weapon")
	return
if off_w == null or off_w.base_power == 0:
	_log_failure(title, "Dual Strike needs an offensive off-hand weapon")
	return

var first: Dictionary = _apply_hit("Dual Strike ▸ Main Hand", false, false)
var second: Dictionary = _apply_hit("Dual Strike ▸ Off-Hand", false, true)
var total: int = UiTheme.dict_int(first, "dmg") + UiTheme.dict_int(second, "dmg")
var any_crit: bool = UiTheme.dict_bool(first, "crit") or UiTheme.dict_bool(second, "crit")

var outcome: String = "HIT"
if not test_dummy.is_alive():
	outcome = "DEFEATED"
elif any_crit:
	outcome = "CRIT"
elif UiTheme.dict_bool(first, "mit"):
	outcome = "MITIGATED"
var detail: String = "%s %d + %s %d" % [main_w.name, UiTheme.dict_int(first, "dmg"), off_w.name, UiTheme.dict_int(second, "dmg")]
var breakdown: String = "%s\n\n%s\n\n[color=#4ade80][b]COMBO TOTAL: %d damage[/b][/color]" % [str(first.get("breakdown", "")), str(second.get("breakdown", "")), total]
log_action("ATTACK", title, detail, total, outcome, breakdown, test_dummy.name)
_emit_hit_fx(first)
_emit_hit_fx(second)
_notify()
```

### `_apply_hit(title: String, is_ranged: bool, use_offhand: bool, mult: float = 1.0) -> Dictionary`
```logic
var res: Dictionary = DamageCalculator.compute(active_hero, test_dummy, is_ranged, use_offhand, mult, force_crit, rng)
var actual_dmg: int = test_dummy.take_damage(UiTheme.dict_int(res, "final_dmg"), true)
var is_crit: bool = UiTheme.dict_bool(res, "is_crit")
var mitigated: bool = UiTheme.dict_float(res, "mit_ratio") > 0.0

var outcome: String = "HIT"
if not test_dummy.is_alive():
	outcome = "DEFEATED"
elif is_crit:
	outcome = "CRIT"
elif mitigated:
	outcome = "MITIGATED"

var weapon_name: String = "Unarmed"
var raw_w: Variant = res.get("weapon", null)
if raw_w is Equipment:
	var used_weapon: Equipment = raw_w
	weapon_name = used_weapon.name
return {
	"dmg": actual_dmg,
	"crit": is_crit,
	"mit": mitigated,
	"outcome": outcome,
	"detail": "%s • %d ATK" % [weapon_name, UiTheme.dict_int(res, "final_attack_power")],
	"breakdown": DojoBreakdown.build_strike(title, active_hero.name, test_dummy, res, actual_dmg),
}
```

### `_emit_hit_fx(hit: Dictionary) -> void`
```logic
var dmg: int = UiTheme.dict_int(hit, "dmg")
var is_crit: bool = UiTheme.dict_bool(hit, "crit")
var mitigated: bool = UiTheme.dict_bool(hit, "mit")
var fx_text: String = "-%d" % dmg
if is_crit:
	fx_text = "CRIT -%d" % dmg
elif mitigated:
	fx_text += " [MIT]"
var fx_color: Color = UiTheme.COL_INFO if (mitigated and not is_crit) else (UiTheme.COL_WARN if is_crit else UiTheme.COL_BAD)
stage_fx.emit("strike", true, fx_text, fx_color)
```

### `selected_skill() -> SkillDef`
```logic
var s: SkillDef = SkillCatalog.find(selected_skill_id)
return s if s != null else SkillCatalog.find("strike")
```

### `benchmark_skill() -> SkillDef`
```logic
var s: SkillDef = selected_skill()
return s if s.kind == SkillDef.Kind.ATTACK else SkillCatalog.find("strike")
```

### `select_skill(skill_id: String) -> void`
```logic
if SkillCatalog.find(skill_id) != null:
	selected_skill_id = skill_id
	_notify()
```

### `select_dummy_skill(skill_id: String) -> void`
```logic
var s: SkillDef = SkillCatalog.find(skill_id)
if s != null and s.kind == SkillDef.Kind.ATTACK:
	dummy_skill_id = skill_id
	_notify()
```

### `skill_ap(skill: SkillDef) -> int`
```logic
return skill.ap_cost(active_hero) + skill.switch_cost(active_hero.current_stance)
```

### `use_skill(skill_id: String = "") -> void`
```logic
if active_hero == null or test_dummy == null:
	return
var skill: SkillDef = SkillCatalog.find(skill_id) if not skill_id.is_empty() else selected_skill()
if skill == null:
	_log_failure("Skill", "Unknown skill '%s'" % skill_id)
	return
if not active_hero.is_alive():
	_log_failure(skill.display_name, "%s is down - Heal Hero first" % active_hero.name)
	return
if skill.kind == SkillDef.Kind.ATTACK and not test_dummy.is_alive():
	_log_failure(skill.display_name, "The dummy is defeated - use Full Heal")
	return
var cost: int = skill_ap(skill)
var switched: bool = skill.switch_cost(active_hero.current_stance) > 0
active_hero.ap -= cost
active_hero.set_stance(skill.stance_after(active_hero.current_stance))
var ap_note: String = "%d AP%s, %d left%s" % [cost, " (incl. +%d stance switch)" % CombatTypes.STANCE_SWITCH_COST if switched else "", active_hero.ap, " - AP debt!" if active_hero.ap < 0 else ""]
match skill.kind:
	SkillDef.Kind.ATTACK:
		var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory != null else null
		var is_ranged: bool = main_w != null and main_w.is_ranged_weapon() and not skill.is_off_hand()
		var hit: Dictionary = _apply_hit(skill.display_name, is_ranged, skill.is_off_hand(), skill.power_for(active_hero)) if skill.deals_damage() else {"detail": "No damage", "outcome": "STATUS"}
		var detail: String = "%s • %s" % [str(hit.get("detail", "")), ap_note]
		if skill.is_area():
			detail += " • area: the dummy is the only target here"
		if not skill.status_effect.is_empty() and test_dummy.is_alive():
			var hit_potency: int = skill.status_potency + active_hero.status_potency_bonus()   # + item / passive status_potency effects
			var chance: int = StatusEffects.apply_chance(test_dummy, skill.status_effect, hit_potency)
			var landed: bool = StatusEffects.try_inflict(test_dummy, skill.status_effect, hit_potency, rng, 1, skill.status_magnitude)
			detail += " • %s %s (%d%%)" % [StatusEffects.display_name(skill.status_effect), "next round" if landed else "resisted", chance]
		log_action("ATTACK", skill.display_name, detail, UiTheme.dict_int(hit, "dmg"), str(hit.get("outcome", "HIT")), str(hit.get("breakdown", "")), test_dummy.name)
		if skill.deals_damage():
			_emit_hit_fx(hit)
	SkillDef.Kind.DEFEND:
		active_hero.defense_mitigation_ratio = skill.mitigation
		log_action("DEFEND", skill.display_name, "Guard -%d%% until the next round • %s" % [roundi(skill.mitigation * 100.0), ap_note], roundi(skill.mitigation * 100.0), "GUARD")
		stage_fx.emit("glow", true, "🛡 %s -%d%%" % [skill.display_name, roundi(skill.mitigation * 100.0)], UiTheme.COL_INFO)
	SkillDef.Kind.FOCUS:
		active_hero.apply_attack_boost(skill.boost, skill.boost_rounds)
		log_action("BUFF", skill.display_name, "Attack x%.2f for %d rounds • %s" % [skill.boost, skill.boost_rounds, ap_note], roundi((skill.boost - 1.0) * 100.0), "BUFF x%.2f" % skill.boost)
		stage_fx.emit("glow", true, "✦ x%.2f ATK (%d rds)" % [skill.boost, skill.boost_rounds], Color(1.0, 0.85, 0.2))
	_:
		var before: int = active_hero.ap
		active_hero.ap = mini(active_hero.max_ap, active_hero.ap + skill.recover_ap)
		log_action("BUFF", skill.display_name, "+%d AP (now %d/%d), now %s" % [active_hero.ap - before, active_hero.ap, active_hero.max_ap, CombatTypes.stance_to_string(active_hero.current_stance)], active_hero.ap - before, "+%d AP" % (active_hero.ap - before))
		stage_fx.emit("text", true, "💤 +%d AP" % (active_hero.ap - before), UiTheme.COL_GOOD)
if dummy_counter and test_dummy.is_alive() and active_hero.is_alive():
	dummy_attack()
	return
_notify()
```

### `dummy_attack() -> void`
```logic
if active_hero == null or test_dummy == null:
	return
var skill: SkillDef = SkillCatalog.find(dummy_skill_id)
if skill == null:
	skill = SkillCatalog.find("strike")
if not test_dummy.is_alive():
	_log_failure("Dummy " + skill.display_name, "The dummy is defeated")
	return
if not active_hero.is_alive():
	_log_failure("Dummy " + skill.display_name, "%s is already down" % active_hero.name)
	return
if test_dummy.has_status(StatusEffects.STUN) or test_dummy.has_status(StatusEffects.STAGGER):
	var lost: String = StatusEffects.STUN if test_dummy.has_status(StatusEffects.STUN) else StatusEffects.STAGGER
	log_action("EVENT", "Dummy " + skill.display_name, "The dummy is %s and loses its action this round" % StatusEffects.display_name(lost).to_lower(), 0, StatusEffects.display_name(lost).to_upper(), "", active_hero.name, true, test_dummy)
	_notify()
	return
# Attacking: the dummy drops its guard and enters the skill's stance.
test_dummy.defense_mitigation_ratio = 0.0
test_dummy.set_stance(skill.stance())
var res: Dictionary = DamageCalculator.compute(test_dummy, active_hero, test_dummy.attack_range > 1 and not skill.is_off_hand(), skill.is_off_hand(), skill.power_mult, false, rng)
var dealt: int = active_hero.take_damage(UiTheme.dict_int(res, "final_dmg"), true)
var is_crit: bool = UiTheme.dict_bool(res, "is_crit")
var outcome: String = "DEFEATED" if not active_hero.is_alive() else ("CRIT" if is_crit else ("MITIGATED" if UiTheme.dict_float(res, "mit_ratio") > 0.0 else "HIT"))
var breakdown: String = DojoBreakdown.build_strike("Dummy " + skill.display_name, test_dummy.name, active_hero, res, dealt)
log_action("ATTACK", "Dummy " + skill.display_name, "%d ATK vs %s (%s stance)" % [UiTheme.dict_int(res, "final_attack_power"), active_hero.name, CombatTypes.stance_to_string(active_hero.current_stance)], dealt, outcome, breakdown, active_hero.name, true, test_dummy)
stage_fx.emit("strike", false, ("CRIT -%d" % dealt) if is_crit else "-%d" % dealt, UiTheme.COL_WARN if is_crit else UiTheme.COL_BAD)
_notify()
```

### `heal_hero() -> void`
```logic
if active_hero == null:
	return
active_hero.hp = active_hero.max_hp
log_action("EVENT", "Hero Heal", "%s restored to %d / %d HP" % [active_hero.name, active_hero.hp, active_hero.max_hp], active_hero.max_hp, "+HEAL")
stage_fx.emit("text", true, "💖 %d HP" % active_hero.max_hp, UiTheme.COL_GOOD)
_notify()
```

### `next_round() -> void`
```logic
if active_hero == null or test_dummy == null:
	return
round_num += 1
var notes: Array[String] = []
for c: Combatant in [active_hero, test_dummy]:
	c.tick_round_buffs()
	var active: Array[String] = StatusEffects.roll_over(c, ROUND_AP_REGEN)
	for status: String in active:
		notes.append("%s %s" % [c.name, StatusEffects.display_name(status).to_lower()])
active_hero.defense_mitigation_ratio = 0.0
log_action("EVENT", "Round %d" % round_num, "Focus ticks, guard drops, +%d AP (now %d/%d)%s" % [ROUND_AP_REGEN, active_hero.ap, active_hero.max_ap, ("; " + ", ".join(notes)) if not notes.is_empty() else ""], round_num, "ROUND")
stage_fx.emit("text", true, "⏭ Round %d" % round_num, UiTheme.COL_INFO)
_notify()
```

### `shift_stance() -> void`
```logic
if active_hero == null:
	return
var order: Array[CombatTypes.StanceType] = [CombatTypes.StanceType.NEUTRAL, CombatTypes.StanceType.OFFENSIVE, CombatTypes.StanceType.TACTICAL, CombatTypes.StanceType.DEFENSIVE]
var next: CombatTypes.StanceType = order[(order.find(active_hero.current_stance) + 1) % order.size()]
active_hero.set_stance(next)
var vs: float = CombatTypes.get_matchup_multiplier(next, test_dummy.current_stance) if test_dummy != null else 1.0
var label: String = CombatTypes.stance_to_string(next)
log_action("EVENT", "Stance Shift", "%s stance (x%.2f vs the dummy's %s)" % [label, vs, CombatTypes.stance_to_string(test_dummy.current_stance) if test_dummy != null else "-"], roundi((vs - 1.0) * 100.0), label.to_upper())
stage_fx.emit("glow" if next != CombatTypes.StanceType.NEUTRAL else "text", true, label.to_upper(), UiTheme.COL_BAD if next == CombatTypes.StanceType.OFFENSIVE else UiTheme.COL_MUTED)
_notify()
```

### `use_pouch_item(index: int) -> void`
```logic
if active_hero == null or active_hero.inventory == null:
	return
var pouch: Array[Consumable] = active_hero.inventory.battle_pouch
if pouch.is_empty():
	_log_failure("Battle Pouch", "Battle pouch is empty - load consumables in the Gear window")
	return
var slot_idx: int = clampi(index, 0, pouch.size() - 1)
var con: Consumable = pouch[slot_idx]
active_hero.inventory.remove_from_battle_pouch(con)

match con.consumable_type:
	Consumable.ConsumableType.HEALTH_POTION:
		var before: int = active_hero.hp
		active_hero.heal(con.heal_amount)
		var healed: int = active_hero.hp - before
		log_action("BUFF", con.name, "Restored %d HP" % healed, healed, "+%d HP" % healed)
		stage_fx.emit("glow", true, "+%d HP" % healed, UiTheme.COL_GOOD)
	Consumable.ConsumableType.THROWABLE_BOMB:
		var dealt: int = test_dummy.take_damage(con.damage_amount)
		log_action("ATTACK", con.name, "Explosive damage", dealt, "DEFEATED" if not test_dummy.is_alive() else "HIT", "", test_dummy.name)
		stage_fx.emit("strike", true, "-%d 💥" % dealt, Color(1.0, 0.6, 0.25))
	_:
		active_hero.apply_attack_boost(1.0 + con.buff_percent, con.buff_duration_rounds)
		log_action("BUFF", con.name, "+%d%% ATK for %d rounds" % [int(roundf(con.buff_percent * 100.0)), con.buff_duration_rounds], int(roundf(con.buff_percent * 100.0)), "BUFF +%d%%" % int(roundf(con.buff_percent * 100.0)))
		stage_fx.emit("glow", true, "✦ ELIXIR BOOST", Color(1.0, 0.8, 0.2))
_notify()
```

### `toggle_dummy_defense() -> void`
```logic
if test_dummy == null:
	return
if test_dummy.defense_mitigation_ratio > 0.0:
	test_dummy.defense_mitigation_ratio = 0.0
	test_dummy.set_stance(CombatTypes.StanceType.NEUTRAL)
	log_action("EVENT", "Dummy Stance", "Dummy neutral - hits deal full damage", 0, "NEUTRAL", "", test_dummy.name)
	stage_fx.emit("text", false, "0% Mit", UiTheme.COL_GOOD)
else:
	test_dummy.defense_mitigation_ratio = 0.5
	test_dummy.set_stance(CombatTypes.StanceType.DEFENSIVE)   # guarding = Defensive stance, as in the arena
	log_action("DEFEND", "Dummy Stance", "Dummy guarding (Defensive stance) - hits halved", 50, "MITIGATED 50%", "", test_dummy.name)
	stage_fx.emit("text", false, "🛡 50% Mit", UiTheme.COL_INFO)
_notify()
```

### `heal_dummy() -> void`
```logic
if test_dummy == null:
	return
test_dummy.hp = test_dummy.max_hp
log_action("EVENT", "Dummy Heal", "Training Dummy restored to %d / %d HP" % [test_dummy.hp, test_dummy.max_hp], test_dummy.max_hp, "+HEAL", "", test_dummy.name)
stage_fx.emit("text", false, "💖 %d HP" % test_dummy.max_hp, UiTheme.COL_GOOD)
_notify()
```

### `run_benchmark(count: int) -> void`
```logic
if active_hero == null or test_dummy == null or active_hero.inventory == null:
	return
var skill: SkillDef = benchmark_skill()
var main_w: Equipment = active_hero.inventory.get_main_weapon()
var is_ranged: bool = main_w != null and main_w.is_ranged_weapon() and not skill.is_off_hand()
var title: String = "Benchmark ×%d %s (%s)" % [count, skill.display_name, "Ranged" if is_ranged else "Melee"]

var base_res: Dictionary = DamageCalculator.compute(active_hero, test_dummy, is_ranged, skill.is_off_hand(), skill.power_for(active_hero), false, null)
var base_hit: int = UiTheme.dict_int(base_res, "final_dmg")
var crit_rate: float = clampf(UiTheme.dict_float(base_res, "crit_rate"), 0.0, 100.0)

var total: int = 0
var min_hit: int = 0
var max_hit: int = 0
var crits: int = 0
for i: int in range(count):
	var res: Dictionary = DamageCalculator.compute(active_hero, test_dummy, is_ranged, skill.is_off_hand(), skill.power_for(active_hero), false, rng)
	var dmg: int = UiTheme.dict_int(res, "final_dmg")
	total += dmg
	min_hit = dmg if i == 0 else mini(min_hit, dmg)
	max_hit = maxi(max_hit, dmg)
	if UiTheme.dict_bool(res, "is_crit"):
		crits += 1

var avg: float = float(total) / float(maxi(1, count))
var crit_mult: float = DamageCalculator.CRIT_MULT
var expected: float = float(base_hit) * (1.0 + (crit_rate / 100.0) * (crit_mult - 1.0))
var swings_to_kill: int = ceili(float(test_dummy.hp) / maxf(1.0, expected))

var stats: Dictionary = {
	"base_hit": base_hit, "min": min_hit, "max": max_hit, "avg": avg, "crits": crits,
	"crit_rate": crit_rate, "expected": expected, "dummy_hp": test_dummy.hp, "swings_to_kill": swings_to_kill,
}
var detail: String = "avg %.1f • min %d • max %d • %d crits" % [avg, min_hit, max_hit, crits]
log_action("ATTACK", title, detail, int(roundf(expected)), "SIM", DojoBreakdown.build_benchmark(title, count, stats), test_dummy.name)
_notify()
```

### `can_hero_equip(item: Equipment) -> bool`
```logic
if active_hero == null or active_hero.stats == null or active_hero.stats.archetype == null:
	return true
return active_hero.stats.archetype.can_equip_gear(item)
```

### `equip_item(item: Equipment, slot: Equipment.EquipSlot) -> bool`
```logic
if active_hero == null or active_hero.inventory == null or item == null:
	return false
if not _equip_internal(item, slot, true):
	return false
active_hero.sync_from_stats_and_gear()
_notify()
return true
```

### `unequip_slot(slot: Equipment.EquipSlot) -> void`
```logic
if active_hero == null or active_hero.inventory == null:
	return
var old_eq: Equipment = _unequip_to_stash(slot)
if old_eq != null:
	log_action("EVENT", "Unequip", "%s returned to stash" % old_eq.name, 0, "STASHED")
active_hero.sync_from_stats_and_gear()
_notify()
```

### `clear_all_gear() -> void`
```logic
if active_hero == null or active_hero.inventory == null:
	return
_stow_all_gear()
active_hero.sync_from_stats_and_gear()
log_action("EVENT", "Clear Gear", "All gear and pouch items returned to stash", 0, "STASHED")
_notify()
```

### `apply_loadout(preset_id: String) -> void`
```logic
if active_hero == null or active_hero.inventory == null:
	return
var preset: Dictionary = DojoLoadouts.find_preset(preset_id)
if preset.is_empty():
	return

_stow_all_gear()
var skipped: Array[String] = []
var equipped_count: int = 0

var gear_raw: Variant = preset.get("gear", [])
var gear: Array = gear_raw if gear_raw is Array else []
for entry: Variant in gear:
	if not (entry is Dictionary):
		continue
	var gear_entry: Dictionary = entry
	var slot_raw: Variant = gear_entry.get("slot", Equipment.EquipSlot.MAIN_HAND)
	var slot_val: Equipment.EquipSlot = slot_raw if slot_raw is Equipment.EquipSlot else Equipment.EquipSlot.MAIN_HAND
	var made: Variant = _spawn_from_factory(gear_entry.get("make", null))
	if not (made is Equipment):
		continue
	var proto: Equipment = made
	var piece: Equipment = _find_stash_equipment(proto.id)
	if piece == null:
		piece = proto
	if _equip_internal(piece, slot_val, false):
		equipped_count += 1
	else:
		skipped.append(proto.name)

var pouch_raw: Variant = preset.get("pouch", [])
var pouch_makers: Array = pouch_raw if pouch_raw is Array else []
for maker: Variant in pouch_makers:
	var made_con: Variant = _spawn_from_factory(maker)
	if not (made_con is Consumable):
		continue
	var proto_con: Consumable = made_con
	var con: Consumable = _find_stash_consumable(proto_con.id)
	if con == null:
		con = proto_con
	shared_stash.remove_item(con)
	active_hero.inventory.add_to_battle_pouch(con)

active_hero.sync_from_stats_and_gear()
var detail: String = "%s: %d pieces equipped" % [str(preset.get("name", preset_id)), equipped_count]
var breakdown: String = "[color=#4ade80]Applied loadout [b]%s[/b][/color]\n%s" % [str(preset.get("name", preset_id)), str(preset.get("desc", ""))]
if not skipped.is_empty():
	detail += " • skipped %s" % ", ".join(skipped)
	breakdown += "\n[color=#f87171]Skipped (not eligible for %s): %s[/color]" % [active_hero.name, ", ".join(skipped)]
log_action("EVENT", "Quick Setup", detail, equipped_count, "LOADOUT", breakdown)
stage_fx.emit("glow", true, "🎒 %s" % str(preset.get("name", "Loadout")), UiTheme.COL_GOOD)
_notify()
```

### `add_to_pouch(con: Consumable) -> bool`
```logic
if active_hero == null or active_hero.inventory == null or con == null:
	return false
if not active_hero.inventory.add_to_battle_pouch(con):
	_log_failure("Battle Pouch", "Battle pouch is full (3 slots)")
	return false
shared_stash.remove_item(con)
log_action("EVENT", "Load Pouch", "%s loaded into battle pouch" % con.name, 0, "LOADED")
_notify()
return true
```

### `remove_from_pouch(index: int) -> void`
```logic
if active_hero == null or active_hero.inventory == null:
	return
var pouch: Array[Consumable] = active_hero.inventory.battle_pouch
if index < 0 or index >= pouch.size():
	return
var con: Consumable = pouch[index]
active_hero.inventory.remove_from_battle_pouch(con)
shared_stash.add_item(con)
log_action("EVENT", "Unload Pouch", "%s returned to stash" % con.name, 0, "STASHED")
_notify()
```

### `switch_weapon() -> void`
```logic
if active_hero == null or active_hero.inventory == null:
	return
var inv: Inventory = active_hero.inventory
if not inv.needs_weapon_switch():
	_log_failure("Switch Weapon", "Nothing to switch: pair a 2H weapon with an off-hand weapon (1H weapons attack with both hands freely)")
	return
inv.toggle_active_weapon()
var ready_w: Equipment = inv.get_active_weapon()
var breakdown: String = "[color=#38bdf8][b]Switch Weapon[/b][/color]  costs 1 block.\nActive weapon is now [b]%s[/b]; the other weapon is sheathed until you switch back." % ready_w.name
log_action("EVENT", "Switch Weapon", "1 block • %s now active" % ready_w.name, 1, "SWITCHED", breakdown)
stage_fx.emit("text", true, "🔄 %s" % ready_w.name, UiTheme.COL_INFO)
_notify()
```

### `_spawn_from_factory(factory_raw: Variant) -> Variant`
```logic
if factory_raw is Callable:
	var factory: Callable = factory_raw
	return factory.call()
return null
```

### `_find_stash_equipment(item_id: String) -> Equipment`
```logic
if shared_stash == null:
	return null
for item: Item in shared_stash.items:
	if item is Equipment and item.id == item_id:
		var eq: Equipment = item
		return eq
return null
```

### `_find_stash_consumable(item_id: String) -> Consumable`
```logic
if shared_stash == null:
	return null
for item: Item in shared_stash.items:
	if item is Consumable and item.id == item_id:
		var con: Consumable = item
		return con
return null
```

### `_unequip_to_stash(slot: Equipment.EquipSlot) -> Equipment`
```logic
var inv: Inventory = active_hero.inventory
var eq: Equipment = inv.unequip(slot)
if eq == null:
	return null
inv.items.erase(eq)
if not shared_stash.items.has(eq):
	shared_stash.add_item(eq)
return eq
```

### `_stow_all_gear() -> void`
```logic
for slot: Equipment.EquipSlot in equip_slots():
	_unequip_to_stash(slot)
var inv: Inventory = active_hero.inventory
for con: Consumable in inv.battle_pouch.duplicate():
	inv.remove_from_battle_pouch(con)
	shared_stash.add_item(con)
```

### `_equip_internal(item: Equipment, slot: Equipment.EquipSlot, announce: bool) -> bool`
```logic
var inv: Inventory = active_hero.inventory
if not can_hero_equip(item):
	if announce:
		_log_failure("Equip", "%s cannot equip %s (class proficiency)" % [active_hero.name, item.name])
	return false
if not item.can_equip_in_slot(slot):
	if announce:
		_log_failure("Equip", "%s cannot be equipped in the %s slot" % [item.name, Equipment.slot_display_name(slot)])
	return false
if slot == Equipment.EquipSlot.OFF_HAND:
	# A 2H main weapon cannot carry a shield, but an off-hand weapon is fine (needs Switch Weapon to use).
	var main_w: Equipment = inv.get_main_weapon()
	if item.conflicts_with_main(main_w):
		if announce:
			_log_failure("Equip", "%s needs a one-handed main hand: 2H %s leaves the off hand empty" % [item.name, main_w.name])
		return false
	if main_w != null and main_w.is_two_handed and not item.is_weapon():
		if announce:
			_log_failure("Equip", "2H %s cannot be paired with %s - only an off-hand weapon (swap with Switch Weapon)" % [main_w.name, item.name])
		return false
if slot == Equipment.EquipSlot.MAIN_HAND and item.is_two_handed:
	# Shields cannot be held with a 2H weapon; an off-hand weapon stays as the switch-to weapon.
	var held_off: Equipment = inv.get_equipped(Equipment.EquipSlot.OFF_HAND)
	if held_off != null and (not held_off.is_weapon() or held_off.conflicts_with_main(item)):
		_unequip_to_stash(Equipment.EquipSlot.OFF_HAND)
		if announce:
			log_action("EVENT", "Unequip", "%s stowed: %s is 2-Handed" % [held_off.name, item.name], 0, "STASHED")
_unequip_to_stash(slot)
shared_stash.remove_item(item)
active_hero.inventory.equip(item, slot)
if announce:
	log_action("EVENT", "Equip", "%s ➜ %s" % [item.name, Equipment.slot_display_name(slot)], 0, "EQUIPPED")
return true
```

### `_clear_allocation_state() -> void`
```logic
is_allocating_stats = false
pending_free_points = 0
pending_allocated_stats = {"STR": 0, "DEX": 0, "INT": 0, "CONS": 0, "PERC": 0, "KNOW": 0}
```

### `get_pending(stat_key: String) -> int`
```logic
var raw: Variant = pending_allocated_stats.get(stat_key, 0)
return raw if raw is int else 0
```

### `begin_allocation() -> void`
```logic
if active_hero == null or active_hero.stats == null:
	return
if is_allocating_stats:
	cancel_allocation()
	return
if active_hero.stats.unspent_attribute_points <= 0:
	log_action("EVENT", "Allocate Stats", "No unspent attribute points available", 0, "NONE")
	return
_clear_allocation_state()
is_allocating_stats = true
pending_free_points = active_hero.stats.unspent_attribute_points
_notify()
```

### `allocate_point(stat_key: String) -> void`
```logic
if not is_allocating_stats or pending_free_points <= 0 or not pending_allocated_stats.has(stat_key):
	return
pending_free_points -= 1
pending_allocated_stats[stat_key] = get_pending(stat_key) + 1
_notify()
```

### `deallocate_point(stat_key: String) -> void`
```logic
if not is_allocating_stats:
	return
var current: int = get_pending(stat_key)
if current <= 0:
	return
pending_allocated_stats[stat_key] = current - 1
pending_free_points += 1
_notify()
```

### `reset_allocation() -> void`
```logic
if not is_allocating_stats or active_hero == null or active_hero.stats == null:
	return
var keep: bool = is_allocating_stats
_clear_allocation_state()
is_allocating_stats = keep
pending_free_points = active_hero.stats.unspent_attribute_points
_notify()
```

### `confirm_allocation() -> void`
```logic
if not is_allocating_stats or active_hero == null or active_hero.stats == null:
	return
var summary: Array[String] = []
var st: CharacterStats = active_hero.stats
for key: String in STAT_KEYS:
	var pts: int = get_pending(key)
	if pts <= 0:
		continue
	match key:
		"STR": st.str_stat += pts
		"DEX": st.dex_stat += pts
		"INT": st.int_stat += pts
		"CONS": st.cons_stat += pts
		"PERC": st.perc_stat += pts
		"KNOW": st.know_stat += pts
	summary.append("+%d %s" % [pts, key])
st.unallocated_attribute_points = pending_free_points
_clear_allocation_state()
active_hero.sync_from_stats_and_gear()
var detail: String = ", ".join(summary) if not summary.is_empty() else "No points spent"
log_action("EVENT", "Allocate Stats", "%s (remaining %d)" % [detail, st.unspent_attribute_points], 0, "CONFIRMED")
_notify()
```

### `cancel_allocation() -> void`
```logic
if not is_allocating_stats:
	return
_clear_allocation_state()
_notify()
```

