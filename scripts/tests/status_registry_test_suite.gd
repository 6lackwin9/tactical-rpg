class_name StatusRegistryTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Registry Test Suite (slice A, foundation cleanup)
# The registry is the single table of statuses; StatusEffects derives its lists from it. The
# derived lists must equal the literals they replaced, every StatusEffects id constant needs a
# row, and each stacking rule's math (with caps) is checked on explicit rules via stack_with.
# ==============================================================================

const TAG: String = "StatusRegistry"
const ORIGINALS: Array[String] = ["stagger", "stun", "root", "bleed", "weaken", "haste", "taunt", "riposte"]
const S12S: Array[String] = ["slow", "expose", "fortify", "regen", "poison", "burn"]
const PACE1: Array[String] = ["empower"]

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING STATUS REGISTRY TEST (slice A)          ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_registry_complete() and ok
	ok = _test_derived_lists() and ok
	ok = _test_stacking_rules() and ok
	ok = _test_stack_in_effects() and ok
	ok = _test_empower_row() and ok
	return ok

# Every String constant of StatusEffects is a status id (the numeric ones are tuning values).
static func _status_constants() -> Array[String]:
	var script: GDScript = StatusEffects
	var out: Array[String] = []
	var map: Dictionary = script.get_script_constant_map()
	for key: Variant in map:
		var value: Variant = map[key]
		if value is String:
			out.append(str(value))
	out.sort()
	return out

static func _test_registry_complete() -> bool:
	var checks: Dictionary = {}
	var consts: Array[String] = _status_constants()
	var ids: Array[String] = StatusRegistry.ids()
	var sorted_ids: Array[String] = ids.duplicate()
	sorted_ids.sort()
	checks["there are fifteen statuses (eight originals + six S12S-L + empower)"] = consts.size() == ORIGINALS.size() + S12S.size() + PACE1.size() and ids.size() == consts.size()
	var covered: bool = true
	for id: String in consts:
		covered = covered and StatusRegistry.has(id)
	checks["every StatusEffects id constant has a registry row"] = covered
	checks["every row is a StatusEffects id constant"] = sorted_ids == consts
	checks["the rows are well formed (kinds, rules, grants, names)"] = StatusRegistry.row_errors().is_empty()
	var expected_ids: Array[String] = ORIGINALS.duplicate()
	expected_ids.append_array(S12S)
	expected_ids.append_array(PACE1)
	checks["covers the eight originals, then slow, expose, fortify, regen, poison, burn, then empower"] = ids == expected_ids
	var all_known_rules: bool = true
	for id: String in ids:
		all_known_rules = all_known_rules and StatusRegistry.RULES.has(StatusRegistry.stacking_of(id)) and StatusRegistry.default_rounds(id) >= 1
	checks["every status has a known stacking rule and default rounds"] = all_known_rules
	checks["the originals refresh to the max duration; only weaken is capped (25)"] = ORIGINALS.all(func(id: String) -> bool: return StatusRegistry.stacking_of(id) == StatusRegistry.REFRESH_MAX_DURATION and StatusRegistry.cap_of(id) == (25 if id == StatusEffects.WEAKEN else 0))
	checks["poison is the only stack_potency status"] = ids.filter(func(id: String) -> bool: return StatusRegistry.stacking_of(id) == StatusRegistry.STACK_POTENCY) == [StatusEffects.POISON]
	checks["an unknown status has no row"] = not StatusRegistry.has("mystery") and StatusRegistry.row("mystery").is_empty()
	# row_errors catches bad rows (checked on a copy of the validation inputs through the public rules).
	checks["only debuffs are resistable"] = StatusRegistry.ids_where(StatusRegistry.KIND_BUFF, true).is_empty()
	return TestHelpers.report(TAG, "Registry covers every status constant and is well formed", checks)

static func _test_derived_lists() -> bool:
	var checks: Dictionary = {}
	# The literals the registry replaced (previous behaviour must be identical).
	var old_debuffs: Array[String] = [StatusEffects.STAGGER, StatusEffects.STUN, StatusEffects.ROOT, StatusEffects.BLEED, StatusEffects.WEAKEN]
	var new_debuffs: Array[String] = [StatusEffects.SLOW, StatusEffects.EXPOSE, StatusEffects.POISON, StatusEffects.BURN]
	old_debuffs.append_array(new_debuffs)
	checks["debuffs() = the old four plus weaken, then the S12S-L debuffs"] = StatusEffects.debuffs() == old_debuffs
	checks["buffs() = the old three, then fortify, regen and empower"] = StatusEffects.buffs() == [StatusEffects.HASTE, StatusEffects.TAUNT, StatusEffects.RIPOSTE, StatusEffects.FORTIFY, StatusEffects.REGEN, StatusEffects.EMPOWER]
	checks["support_buffs() gains fortify and regen (registry grant)"] = StatusEffects.support_buffs() == [StatusEffects.HASTE, StatusEffects.TAUNT, StatusEffects.FORTIFY, StatusEffects.REGEN]
	checks["defend_buffs() unchanged"] = StatusEffects.defend_buffs() == [StatusEffects.RIPOSTE]
	checks["resistable() = debuffs"] = StatusEffects.resistable() == StatusEffects.debuffs()
	var names: Dictionary = {StatusEffects.STAGGER: "Staggered", StatusEffects.STUN: "Stunned", StatusEffects.ROOT: "Rooted", StatusEffects.BLEED: "Bleeding", StatusEffects.WEAKEN: "Weakened", StatusEffects.HASTE: "Hasted", StatusEffects.TAUNT: "Taunting", StatusEffects.RIPOSTE: "Riposte"}
	var names_ok: bool = true
	for id: Variant in names:
		names_ok = names_ok and StatusEffects.display_name(str(id)) == str(names[id])
	checks["display_name() unchanged"] = names_ok
	checks["an unknown status is capitalised"] = StatusEffects.display_name("mystery") == "Mystery"
	checks["debuffs and buffs partition the registry"] = StatusEffects.debuffs().size() + StatusEffects.buffs().size() == StatusRegistry.ids().size()
	checks["catalog validation still accepts exactly the debuffs"] = ContentCatalog.validate_entry("skills", {"id": "t_reg_ok", "name": "R", "kind": "ATTACK", "status_effect": "root", "status_potency": 50}).is_empty() and not ContentCatalog.validate_entry("skills", {"id": "t_reg_bad", "name": "R", "kind": "ATTACK", "status_effect": "haste", "status_potency": 50}).is_empty()
	return TestHelpers.report(TAG, "Derived lists equal the literals they replaced", checks)

static func _test_stacking_rules() -> bool:
	var checks: Dictionary = {}
	var refresh: String = StatusRegistry.REFRESH_MAX_DURATION
	var extend: String = StatusRegistry.EXTEND_DURATION
	var stack_p: String = StatusRegistry.STACK_POTENCY
	var none: String = StatusRegistry.NONE
	# refresh_max_duration: the longer duration and the stronger potency win.
	var r1: Dictionary = StatusRegistry.stack_with(refresh, 0, 2, 40, 3, 30)
	checks["refresh: longer rounds win, stronger potency wins"] = r1["rounds"] == 3 and r1["potency"] == 40
	var r2: Dictionary = StatusRegistry.stack_with(refresh, 0, 3, 20, 1, 60)
	checks["refresh: a shorter reapplication does not shorten"] = r2["rounds"] == 3 and r2["potency"] == 60
	# extend_duration: rounds add up (capped by cap), potency is the stronger.
	var e1: Dictionary = StatusRegistry.stack_with(extend, 0, 2, 25, 3, 20)
	checks["extend: rounds add up"] = e1["rounds"] == 5 and e1["potency"] == 25
	var e2: Dictionary = StatusRegistry.stack_with(extend, 4, 3, 25, 3, 20)
	checks["extend: rounds cap at the cap"] = e2["rounds"] == 4
	# stack_potency: potency adds (capped), rounds = longer.
	var s1: Dictionary = StatusRegistry.stack_with(stack_p, 0, 2, 10, 1, 15)
	checks["stack_potency: potency adds, rounds = longer"] = s1["rounds"] == 2 and s1["potency"] == 25
	var s2: Dictionary = StatusRegistry.stack_with(stack_p, 25, 2, 20, 2, 20)
	checks["stack_potency: potency caps at the cap"] = s2["potency"] == 25 and s2["rounds"] == 2
	# none: the first application stays while it lasts.
	var n1: Dictionary = StatusRegistry.stack_with(none, 0, 1, 15, 3, 25)
	checks["none: the existing application is kept"] = n1["rounds"] == 1 and n1["potency"] == 15
	# cap with refresh/none limits potency (the weaken 25% case).
	var c1: Dictionary = StatusRegistry.stack_with(none, 25, 0, 0, 2, 40)
	checks["a fresh application is clamped to the cap"] = c1["rounds"] == 2 and c1["potency"] == 25
	var c2: Dictionary = StatusRegistry.stack_with(refresh, 25, 2, 25, 2, 30)
	checks["refresh: potency is capped"] = c2["potency"] == 25
	# Nothing existing: every rule returns the new application (rounds at least 1).
	var fresh_ok: bool = true
	for rule: String in StatusRegistry.RULES:
		var f: Dictionary = StatusRegistry.stack_with(rule, 0, 0, 0, 0, 30)
		fresh_ok = fresh_ok and f["rounds"] == 1 and f["potency"] == 30
	checks["with nothing existing every rule returns the new one (min 1 round)"] = fresh_ok
	var unknown: Dictionary = StatusRegistry.stack_with("bogus", 0, 2, 10, 1, 20)
	checks["an unknown rule behaves as refresh_max_duration"] = unknown["rounds"] == 2 and unknown["potency"] == 20
	return TestHelpers.report(TAG, "Each stacking rule's math and caps", checks)

static func _test_stack_in_effects() -> bool:
	var checks: Dictionary = {}
	# StatusEffects.stack goes through the registry row of the status (all refresh_max_duration today).
	var st: Dictionary = StatusEffects.stack(2, 50, 1, 70, StatusEffects.BLEED)
	checks["StatusEffects.stack uses the registry rule"] = st["rounds"] == 2 and st["potency"] == 70
	checks["an unknown status refreshes"] = StatusEffects.stack(1, 0, 3, 0, "mystery")["rounds"] == 3
	# queue / grant_now keep their outcomes: the longer duration wins, minimum one round.
	var c := Combatant.new("Reg", 100, 10, 1)
	StatusEffects.queue(c, StatusEffects.ROOT, 2)
	StatusEffects.queue(c, StatusEffects.ROOT, 1)
	checks["queue keeps the longer duration"] = DictUtil.int_of(c.pending_status_rounds, StatusEffects.ROOT) == 2
	StatusEffects.queue(c, StatusEffects.STUN, 0)
	checks["queue lasts at least one round"] = DictUtil.int_of(c.pending_status_rounds, StatusEffects.STUN) == 1
	StatusEffects.grant_now(c, StatusEffects.RIPOSTE, 2)
	StatusEffects.grant_now(c, StatusEffects.RIPOSTE, 3)
	StatusEffects.grant_now(c, StatusEffects.RIPOSTE, 1)
	checks["grant_now keeps the longer duration"] = DictUtil.int_of(c.status_rounds_left, StatusEffects.RIPOSTE) == 3 and c.statuses.has(StatusEffects.RIPOSTE)
	return TestHelpers.report(TAG, "StatusEffects applies the registry's stacking rule", checks)

# PACE1 3.1: the empower row (a buff no skill grants), and the relaxed grant rule that allows it.
static func _test_empower_row() -> bool:
	var checks: Dictionary = {}
	var r: Dictionary = StatusRegistry.row(StatusEffects.EMPOWER)
	checks["empower is a buff, not resistable"] = str(r.get("kind", "")) == StatusRegistry.KIND_BUFF and not DictUtil.to_bool(r.get("resistable", true))
	checks["empower: 2 rounds, refresh_max_duration, cap 25, Empowered"] = StatusRegistry.default_rounds(StatusEffects.EMPOWER) == 2 and StatusRegistry.stacking_of(StatusEffects.EMPOWER) == StatusRegistry.REFRESH_MAX_DURATION and StatusRegistry.cap_of(StatusEffects.EMPOWER) == 25 and StatusEffects.display_name(StatusEffects.EMPOWER) == "Empowered"
	checks["empower magnitude 1..25 pct"] = StatusRegistry.magnitude_min(StatusEffects.EMPOWER) == 1 and StatusRegistry.magnitude_max(StatusEffects.EMPOWER) == 25 and StatusRegistry.magnitude_unit(StatusEffects.EMPOWER) == StatusRegistry.UNIT_PCT and StatusEffects.has_magnitude(StatusEffects.EMPOWER)
	checks["no skill grants empower (grant \"\")"] = str(r.get("grant", "x")).is_empty() and not StatusEffects.support_buffs().has(StatusEffects.EMPOWER) and not StatusEffects.defend_buffs().has(StatusEffects.EMPOWER)
	var skill_errors: Array[String] = ContentCatalog.validate_entry("skills", {"id": "t_reg_emp", "name": "E", "kind": "SUPPORT", "grant_status": StatusEffects.EMPOWER})
	checks["a SUPPORT skill granting empower is refused (%s)" % "; ".join(skill_errors)] = TestHelpers.any_contains(skill_errors, "'empower' is not a support buff")
	var grant_opts: Array[String] = ContentSchema.options_of({"options_from": "status_buff"})
	checks["the skill grant_status options leave empower out, keep the skill-granted buffs"] = not grant_opts.has(StatusEffects.EMPOWER) and StatusEffects.support_buffs().all(func(b: String) -> bool: return grant_opts.has(b)) and StatusEffects.defend_buffs().all(func(b: String) -> bool: return grant_opts.has(b))
	var base: Dictionary = StatusRegistry.row(StatusEffects.EMPOWER).duplicate()
	var debuff_support: Dictionary = base.duplicate()
	debuff_support["kind"] = StatusRegistry.KIND_DEBUFF
	debuff_support["grant"] = "support"
	var buff_bogus: Dictionary = base.duplicate()
	buff_bogus["grant"] = "potion"
	checks["grant rule: a buff may have grant \"\""] = StatusRegistry.errors_of([base]).is_empty()
	checks["grant rule: a debuff with a skill grant is an error"] = not StatusRegistry.errors_of([debuff_support]).is_empty()
	checks["grant rule: an unknown grant is an error"] = not StatusRegistry.errors_of([buff_bogus]).is_empty()
	return TestHelpers.report(TAG, "PACE1 empower row: buff, 1..25 pct, cap 25, no skill grant", checks)
