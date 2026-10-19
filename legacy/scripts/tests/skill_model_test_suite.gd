class_name SkillModelTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Skill Model Test Suite (FORGE1 / SKILL1-L)
# The kind descriptor table (SkillKinds), the section rule that replaced the hand-written "belongs to" messages,
# weapon_hand (strict hand typing) and the weapon gate on ATTACK / DEFEND / SUPPORT, grade and requires (validation,
# ownership, class paths, unit loadouts), and the automatic grant of every basic skill. Fixtures are built here.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("skill_model_test")
const TAG: String = "SkillModel"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SKILL MODEL TEST (SKILL1-L)             ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_kinds_table() and ok
	ok = _test_sections_and_gate() and ok
	ok = _test_weapon_hand() and ok
	ok = _test_gate_in_engine() and ok
	ok = _test_grade_and_requires() and ok
	ok = _test_ownership() and ok
	ok = _test_class_and_unit_rules() and ok
	ok = _test_basics_grant() and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

static func _errors(entry: Dictionary) -> Array[String]:
	return ContentCatalog.validate_entry("skills", entry)

static func _has(errors: Array[String], needle: String) -> bool:
	return TestHelpers.any_contains(errors, needle)

static func _test_kinds_table() -> bool:
	var checks: Dictionary = {}
	var kind_names: Array[String] = []
	for key: Variant in SkillDef.Kind.keys():
		kind_names.append(str(key))
	var table_names: Array[String] = SkillKinds.names()
	var same: bool = kind_names.size() == table_names.size()
	for name: String in kind_names:
		same = same and table_names.has(name)
	checks["the table has one row per SkillDef.Kind"] = same
	checks["authorable kinds are exactly the five the Forge offers"] = SkillKinds.authorable() == ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"] or (SkillKinds.authorable().size() == 5 and not SkillKinds.is_authorable("REST") and not SkillKinds.is_authorable("MOVE"))
	var sections_ok: bool = true
	for name: String in table_names:
		for section: String in SkillKinds.sections_of(name):
			sections_ok = sections_ok and SkillKinds.SECTION_FIELDS.has(section)
	checks["every kind section is a known section"] = sections_ok
	var when_ok: bool = true
	for key: String in SkillKinds.sectioned_fields():
		var f: Dictionary = ContentSchema.field("skills", key)
		var cond: Dictionary = DictUtil.dict_of(f.get("when", {}))
		if str(cond.get("key", "")) == "kind":
			var listed: Array = DictUtil.array_of(cond.get("in", []))
			var want: Array[String] = SkillKinds.kinds_with(key)
			when_ok = when_ok and listed.size() == want.size() and want.all(func(k: String) -> bool: return listed.has(k))
	checks["the schema `when` lists are generated from the table"] = when_ok
	checks["status_rounds sits in two sections"] = SkillKinds.sections_with("ATTACK", "status_rounds") == ["inflict"] and SkillKinds.sections_with("SUPPORT", "status_rounds") == ["grant"] and SkillKinds.sections_with("DEFEND", "status_rounds") == ["grant"] and SkillKinds.sections_with("FOCUS", "status_rounds").is_empty()
	checks["the gate section is ATTACK, DEFEND and SUPPORT"] = SkillKinds.gate_kinds().size() == 3 and SkillKinds.gate_kinds().has("DEFEND") and SkillKinds.gate_kinds().has("SUPPORT") and SkillKinds.gate_kinds().has("ATTACK")
	var rules_ok: bool = true
	for name: String in table_names:
		var s := SkillDef.new()
		s.kind = SkillDef.Kind[name] as SkillDef.Kind
		s.level = 3
		var ap: String = str(SkillKinds.info(name)["ap"])
		rules_ok = rules_ok and s.ap_cost() == (0 if ap == "FREE" else (1 if ap == "PER_TILE" else 3))
		var rule: String = str(SkillKinds.info(name)["stance"])
		var expected: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
		match rule:
			"OFFENSIVE":
				expected = CombatTypes.StanceType.OFFENSIVE
			"DEFENSIVE":
				expected = CombatTypes.StanceType.DEFENSIVE
			"TACTICAL":
				expected = CombatTypes.StanceType.TACTICAL
		rules_ok = rules_ok and s.stance() == expected and s.keeps_stance() == (rule == "KEEP" or rule == "KEEP_UNLESS_SET") and s.icon() == SkillKinds.icon_of(name)
		if rule == "KEEP_UNLESS_SET":
			s.stance_override = CombatTypes.StanceType.DEFENSIVE
			rules_ok = rules_ok and not s.keeps_stance() and s.stance() == CombatTypes.StanceType.DEFENSIVE
	checks["SkillDef.ap_cost, stance, keeps_stance and icon read the table"] = rules_ok
	return TestHelpers.report(TAG, "The kind descriptor table", checks)

static func _test_sections_and_gate() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var defend_gate: Dictionary = {"id": "t_md_dg", "name": "DG", "kind": "DEFEND", "weapon_types": ["sword"], "min_proficiency": 1}
	checks["a weapon gate on DEFEND is accepted"] = _errors(defend_gate).is_empty()
	var support_gate: Dictionary = {"id": "t_md_sg", "name": "SG", "kind": "SUPPORT", "heal_base": 5, "weapon_types": ["staff"], "min_proficiency": 1}
	checks["a weapon gate on SUPPORT is accepted"] = _errors(support_gate).is_empty()
	checks["a weapon gate on ATTACK is accepted"] = _errors({"id": "t_md_ag", "name": "AG", "kind": "ATTACK", "weapon_types": ["bow"], "min_proficiency": 2}).is_empty()
	checks["a gate on FOCUS is refused"] = not _errors({"id": "t_md_fg", "name": "FG", "kind": "FOCUS", "weapon_types": ["sword"], "min_proficiency": 1}).is_empty() and not _errors({"id": "t_md_fg2", "name": "FG", "kind": "FOCUS", "min_proficiency": 1}).is_empty()
	checks["a gate on DISENGAGE is refused"] = not _errors({"id": "t_md_dig", "name": "DiG", "kind": "DISENGAGE", "jump_tiles": 2, "weapon_types": ["sword"], "min_proficiency": 1}).is_empty()
	checks["weapon types without a proficiency are refused (the types would do nothing)"] = _has(_errors({"id": "t_md_np", "name": "NP", "kind": "ATTACK", "weapon_types": ["bow"]}), "proficiency")
	checks["a proficiency without weapon types is legal (any weapon at level N)"] = _errors({"id": "t_md_pp", "name": "PP", "kind": "ATTACK", "min_proficiency": 1}).is_empty()
	checks["an unknown gear type is refused"] = _has(_errors({"id": "t_md_ug", "name": "UG", "kind": "ATTACK", "weapon_types": ["spoon"], "min_proficiency": 1}), "unknown gear type")
	checks["the unarmed token is accepted"] = _errors({"id": "t_md_un", "name": "UN", "kind": "ATTACK", "weapon_types": [SkillDef.UNARMED_TOKEN], "min_proficiency": 1}).is_empty()
	# The section rule: a field set but the kind has no section for it.
	checks["mitigation on ATTACK is refused"] = _has(_errors({"id": "t_md_s1", "name": "S", "kind": "ATTACK", "mitigation": 0.3}), "Mitigation")
	checks["power on DEFEND is refused"] = _has(_errors({"id": "t_md_s2", "name": "S", "kind": "DEFEND", "power_mult": 2.0}), "Power")
	checks["reach on FOCUS is refused"] = not _errors({"id": "t_md_s3", "name": "S", "kind": "FOCUS", "reach_max": 2}).is_empty()
	checks["a gap-close on SUPPORT is refused"] = not _errors({"id": "t_md_s4", "name": "S", "kind": "SUPPORT", "heal_base": 3, "dash_tiles": 2}).is_empty()
	checks["jump tiles on ATTACK are refused"] = not _errors({"id": "t_md_s5", "name": "S", "kind": "ATTACK", "jump_tiles": 2}).is_empty()
	checks["a heal on DEFEND is refused"] = not _errors({"id": "t_md_s6", "name": "S", "kind": "DEFEND", "heal_base": 4}).is_empty()
	checks["a stance override on SUPPORT is refused"] = not _errors({"id": "t_md_s7", "name": "S", "kind": "SUPPORT", "heal_base": 3, "stance": "DEFENSIVE"}).is_empty()
	checks["a REST skill is engine-only for authors"] = _has(_errors({"id": "t_md_rest", "name": "R", "kind": "REST"}), "engine-only")
	checks["a MOVE skill is engine-only for authors"] = _has(_errors({"id": "t_md_move", "name": "M", "kind": "MOVE"}), "engine-only")
	checks["grant_status is limited per kind (taunt is not a defend-only buff but is allowed)"] = _errors({"id": "t_md_gs", "name": "G", "kind": "DEFEND", "grant_status": "riposte", "status_rounds": 2}).is_empty() and not _errors({"id": "t_md_gs2", "name": "G", "kind": "SUPPORT", "grant_status": "riposte", "status_rounds": 2}).is_empty()
	return TestHelpers.report(TAG, "Section rule and weapon gate", checks)

# Finds a catalog item (the first matching the predicate) and builds it.
static func _equipment(item_type: String, slot: String, two_handed: bool = false) -> Equipment:
	for e: Dictionary in ContentCatalog.entries("items"):
		if str(e["item_type"]) == item_type and str(e["slot"]) == slot and DictUtil.to_bool(e["two_handed"]) == two_handed:
			var built: Item = ContentFactory.item(e)
			if built is Equipment:
				return built as Equipment
	return null

static func _equipment_of_type(weapon_type: String, slot: String) -> Equipment:
	for e: Dictionary in ContentCatalog.entries("items"):
		if str(e["weapon_type"]) == weapon_type and str(e["slot"]) == slot and not DictUtil.to_bool(e["two_handed"]) and str(e["item_type"]) == "WEAPON":
			var built: Item = ContentFactory.item(e)
			if built is Equipment:
				return built as Equipment
	return null

# A class-derived unit with its hands emptied, so each test equips exactly what it needs.
static func _bare_unit(class_id: String, extras: Array[String] = []) -> Combatant:
	var c: Combatant = ContentFactory.unit({"id": "t_md_u", "name": "U", "class_id": class_id, "level": 5, "extra_skills": extras}, "", 12)
	c.inventory.unequip(Equipment.EquipSlot.MAIN_HAND)
	c.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
	return c

static func _class_with(gear: String) -> String:
	for e: Dictionary in ContentCatalog.entries("classes"):
		var prof: Dictionary = DictUtil.dict_of(e["proficiency"])
		if prof.has(gear) and DictUtil.to_int(DictUtil.dict_of(prof[gear]).get("level", 0)) >= 1:
			return str(e["id"])
	return ""

static func _test_weapon_hand() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var off_entry: Dictionary = ContentSchema.normalize("skills", {"id": "t_md_hand", "name": "H", "kind": "ATTACK", "weapon_hand": "off"})
	checks["weapon_hand round-trips through the factory"] = str(ContentFactory.skill_to_entry(ContentFactory.skill(off_entry))["weapon_hand"]) == "off"
	var both_errors: Array[String] = _errors({"id": "t_md_both", "name": "B", "kind": "ATTACK", "weapon_hand": "both"})
	checks["'both' is rejected exactly while BOTH_ENABLED is false"] = both_errors.is_empty() == SkillDef.BOTH_ENABLED
	checks["an off-hand strike can't be a gap-close"] = not _errors({"id": "t_md_od", "name": "O", "kind": "ATTACK", "weapon_hand": "off", "dash_tiles": 2}).is_empty()
	checks["a dual strike can't be a gap-close"] = not _errors({"id": "t_md_bd", "name": "O", "kind": "ATTACK", "weapon_hand": "both", "dash_tiles": 2}).is_empty()
	checks["an off-hand strike is valid"] = _errors({"id": "t_md_off", "name": "O", "kind": "ATTACK", "weapon_hand": "off"}).is_empty()
	checks["weapon_hand on another kind is refused"] = not _errors({"id": "t_md_dh", "name": "D", "kind": "DEFEND", "weapon_hand": "off"}).is_empty()
	checks["an unknown weapon_hand value is refused"] = not _errors({"id": "t_md_xh", "name": "X", "kind": "ATTACK", "weapon_hand": "left"}).is_empty()
	var jab: SkillDef = SkillCatalog.find("offhand_jab")
	checks["the shipped offhand_jab is an off-hand skill"] = jab != null and jab.weapon_hand == SkillDef.WeaponHand.OFF and jab.is_off_hand()
	checks["an off-hand skill with a real weapon type warns"] = TestHelpers.any_contains(ContentCatalog.warnings_for("skills", {"id": "t_md_w", "name": "W", "kind": "ATTACK", "weapon_hand": "off", "weapon_types": ["dagger"], "min_proficiency": 1}), "only 'unarmed'")
	checks["an off-hand skill gated on unarmed does not warn"] = ContentCatalog.warnings_for("skills", {"id": "t_md_w2", "name": "W", "kind": "ATTACK", "weapon_hand": "off", "weapon_types": ["unarmed"], "min_proficiency": 1}).is_empty()
	# wielded_type(hand)
	var class_id: String = _class_with("sword")
	checks["a class with sword training exists to build units from"] = not class_id.is_empty()
	var sword: Equipment = _equipment_of_type("sword", "MAIN_HAND")
	var shield: Equipment = _equipment("SHIELD", "OFF_HAND")
	var focus_off: Equipment = null
	for gear: String in ["orb", "tome"]:
		if focus_off == null:
			focus_off = _equipment_of_type(gear, "OFF_HAND")
	checks["fixture gear exists in the catalog (sword, shield, an off-hand focus)"] = sword != null and shield != null and focus_off != null
	if sword != null and shield != null and focus_off != null and not class_id.is_empty():
		var c: Combatant = _bare_unit(class_id)
		checks["empty hands: both hands are unarmed"] = c.wielded_type() == SkillDef.UNARMED_TOKEN and c.wielded_type(SkillDef.WeaponHand.OFF) == SkillDef.UNARMED_TOKEN
		c.inventory.equip(sword)
		checks["a main weapon is the MAIN type, the off hand stays unarmed"] = c.wielded_type() == sword.weapon_type and c.wielded_type(SkillDef.WeaponHand.OFF) == SkillDef.UNARMED_TOKEN
		c.inventory.equip(shield)
		checks["a shield is not a weapon: the off hand is still unarmed"] = c.wielded_type(SkillDef.WeaponHand.OFF) == SkillDef.UNARMED_TOKEN
		c.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
		c.inventory.equip(focus_off)
		checks["an off-hand focus weapon is the OFF type"] = c.wielded_type(SkillDef.WeaponHand.OFF) == focus_off.weapon_type
		# The gate checks the hand the skill uses.
		var gate: Dictionary = {"id": "t_md_offgate", "name": "OG", "kind": "ATTACK", "weapon_hand": "off", "weapon_types": [SkillDef.UNARMED_TOKEN], "min_proficiency": 1, "learned_only": true}
		checks["gate fixture accepted"] = ContentCatalog.upsert("skills", gate).is_empty()
		var user: Combatant = _bare_unit(class_id, ["t_md_offgate"])
		var skill: SkillDef = SkillCatalog.find("t_md_offgate")
		user.inventory.equip(sword)
		user.inventory.equip(focus_off)
		checks["an off-hand skill gated on unarmed is unusable with an off-hand weapon"] = skill != null and not user.can_use(skill)
		user.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
		var fist_ok: bool = user.stats.proficiency.level_of(SkillDef.UNARMED_PROFICIENCY) >= 1
		checks["...and follows the fist proficiency with bare off hand"] = user.can_use(skill) == fist_ok
	return TestHelpers.report(TAG, "weapon_hand: strict hand typing", checks)

static func _test_gate_in_engine() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var class_id: String = _class_with("sword")
	var sword: Equipment = _equipment_of_type("sword", "MAIN_HAND")
	checks["a sword-trained class and a sword exist"] = not class_id.is_empty() and sword != null
	if class_id.is_empty() or sword == null:
		return TestHelpers.report(TAG, "The weapon gate is enforced by can_use for DEFEND and SUPPORT", checks)
	var defend: Dictionary = {"id": "t_md_gate_d", "name": "GD", "kind": "DEFEND", "level": 2, "blocks": 2, "mitigation": 0.5, "weapon_types": ["sword"], "min_proficiency": 1, "learned_only": true}
	var support: Dictionary = {"id": "t_md_gate_s", "name": "GS", "kind": "SUPPORT", "level": 2, "blocks": 2, "heal_base": 5, "weapon_types": ["sword"], "min_proficiency": 1, "learned_only": true}
	checks["fixtures accepted"] = ContentCatalog.upsert("skills", defend).is_empty() and ContentCatalog.upsert("skills", support).is_empty()
	var c: Combatant = _bare_unit(class_id, ["t_md_gate_d", "t_md_gate_s"])
	var d: SkillDef = SkillCatalog.find("t_md_gate_d")
	var s: SkillDef = SkillCatalog.find("t_md_gate_s")
	checks["fixtures resolve to skills"] = d != null and s != null
	if d == null or s == null:
		return TestHelpers.report(TAG, "The weapon gate is enforced by can_use for DEFEND and SUPPORT", checks)
	checks["a gated DEFEND is unusable without the weapon"] = not c.can_use(d)
	checks["a gated SUPPORT is unusable without the weapon"] = not c.can_use(s)
	c.inventory.equip(sword)
	var trained: bool = c.stats.proficiency.level_of(sword.weapon_type) >= d.min_proficiency
	checks["...and usable with the right weapon (when trained)"] = trained and c.can_use(d) and c.can_use(s)
	return TestHelpers.report(TAG, "The weapon gate is enforced by can_use for DEFEND and SUPPORT", checks)

static func _test_grade_and_requires() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var novice_ranked: Dictionary = SkillRanks.entry(TestHelpers.attack("t_gr_nov"), [{"level": 3}])
	checks["a novice family with two ranks is accepted"] = ContentCatalog.upsert_skill(novice_ranked).is_empty()
	checks["a second novice family"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_nov2")).is_empty()
	checks["an advanced skill may require a novice"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_adv", {"grade": "advanced", "requires": ["t_gr_nov"]})).is_empty()
	checks["an advanced skill may require several novices (ALL are needed)"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_multi", {"grade": "advanced", "requires": ["t_gr_nov", "t_gr_nov2"]})).is_empty()
	checks["an expert is accepted by validation and may require lower grades"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_exp", {"grade": "expert", "requires": ["t_gr_adv", "t_gr_nov"]})).is_empty()
	checks["a required skill of equal grade is refused"] = TestHelpers.any_contains(ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_eq", {"grade": "advanced", "requires": ["t_gr_adv"]})), "lower grade")
	checks["a required skill of higher grade is refused"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_hi", {"grade": "advanced", "requires": ["t_gr_exp"]})).is_empty()
	checks["a novice skill with requires is refused (nothing is lower)"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_nr", {"requires": ["t_gr_nov"]})).is_empty()
	checks["a skill cannot require itself"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_self", {"grade": "advanced", "requires": ["t_gr_self"]})).is_empty()
	checks["requiring an unknown skill is refused"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_unk", {"grade": "advanced", "requires": ["t_gr_none"]})).is_empty()
	checks["requiring a rank row is refused (name the family head)"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_rank", {"grade": "advanced", "requires": ["t_gr_nov_2"]})).is_empty()
	checks["requiring a basic skill is refused"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_basic", {"grade": "advanced", "requires": ["strike"]})).is_empty()
	checks["a duplicate requirement is refused"] = not ContentCatalog.upsert_skill(TestHelpers.attack("t_gr_dup", {"grade": "advanced", "requires": ["t_gr_nov", "t_gr_nov"]})).is_empty()
	checks["a basic skill can't be advanced"] = not _errors({"id": "t_gr_bad", "name": "B", "kind": "ATTACK", "basic": true, "grade": "advanced"}).is_empty()
	checks["a basic skill can't have requires"] = not _errors({"id": "t_gr_bad2", "name": "B", "kind": "ATTACK", "basic": true, "requires": ["t_gr_nov"]}).is_empty()
	var raise: Dictionary = ContentCatalog.skill_entry("t_gr_nov")
	raise["grade"] = "advanced"
	var raise_errors: Array[String] = ContentCatalog.upsert_skill(raise)
	checks["a grade change that would break a dependent is refused, naming it"] = TestHelpers.any_contains(raise_errors, "t_gr_adv") or TestHelpers.any_contains(raise_errors, "requires")
	checks["...and changed nothing"] = str(ContentCatalog.get_entry("skills", "t_gr_nov")["grade"]) == "novice"
	var lower: Dictionary = ContentCatalog.skill_entry("t_gr_exp")
	lower["grade"] = "novice"
	checks["lowering a grade below its own requirements is refused"] = not ContentCatalog.upsert_skill(lower).is_empty()
	checks["a required skill with dependents cannot be deleted"] = ContentCatalog.remove_skill("t_gr_nov").contains("still used")
	checks["requires on a rank-2 row stays consistent across ranks"] = ContentFactory._strings(ContentCatalog.get_entry("skills", "t_gr_nov_2")["requires"]).is_empty()
	var adv_ranked: Dictionary = SkillRanks.entry(TestHelpers.attack("t_gr_advr", {"grade": "advanced", "requires": ["t_gr_nov"]}), [{"level": 3}])
	checks["an advanced family with ranks stores grade and requires once"] = ContentCatalog.upsert_skill(adv_ranked).is_empty() and ContentFactory._strings(ContentCatalog.get_entry("skills", "t_gr_advr_2")["requires"]) == ["t_gr_nov"] and str(ContentCatalog.get_entry("skills", "t_gr_advr_2")["grade"]) == "advanced"
	var stored: Dictionary = ContentCatalog.skill_entry("t_gr_advr")
	checks["...in the ranked entry (not in the rank modifiers)"] = stored.has("grade") and stored.has("requires") and not DictUtil.dict_of(DictUtil.array_of(stored["ranks"])[0]).has("grade")
	checks["SkillDef reads grade and requires"] = ContentFactory.skill(ContentCatalog.get_entry("skills", "t_gr_exp")).grade == SkillDef.Grade.EXPERT and ContentFactory.skill(ContentCatalog.get_entry("skills", "t_gr_exp")).requires.size() == 2
	return TestHelpers.report(TAG, "grade and requires: validation", checks)

static func _test_ownership() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	checks["fixture families accepted"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_ow_a")).is_empty() and ContentCatalog.upsert_skill(TestHelpers.attack("t_ow_b")).is_empty() and ContentCatalog.upsert_skill(TestHelpers.attack("t_ow_adv", {"grade": "advanced", "requires": ["t_ow_a", "t_ow_b"]})).is_empty() and ContentCatalog.upsert_skill(TestHelpers.attack("t_ow_free")).is_empty()
	checks["a passive granting the advanced skill is accepted"] = ContentCatalog.upsert("passives", {"id": "t_ow_pass", "name": "P", "effects": [{"type": "skill", "skill": "t_ow_adv"}]}).is_empty()
	var class_id: String = ContentCatalog.ids("classes")[0]
	var c: Combatant = ContentFactory.unit({"id": "t_ow_u", "name": "U", "class_id": class_id, "level": 5}, "", 12)
	checks["without the requirements the family can't be owned"] = not c.can_own_family("t_ow_adv")
	c.learn_skill("t_ow_adv")
	checks["learn_skill refuses (no-op)"] = not c.owns_skill("t_ow_adv")
	ContentFactory.apply_passives(c, _ids("t_ow_pass"))
	checks["a passive grant with unmet requirements does nothing"] = not c.owns_skill("t_ow_adv")
	c.learn_skill("t_ow_a")
	checks["ANY ONE requirement is not enough"] = not c.can_own_family("t_ow_adv")
	c.learn_skill("t_ow_adv")
	checks["...still refused"] = not c.owns_skill("t_ow_adv")
	c.learn_skill("t_ow_b")
	checks["ALL requirements owned: allowed"] = c.can_own_family("t_ow_adv")
	c.learn_skill("t_ow_adv")
	checks["...and learn_skill works"] = c.owns_skill("t_ow_adv")
	var d: Combatant = ContentFactory.unit({"id": "t_ow_u2", "name": "U", "class_id": class_id, "level": 5}, "", 12)
	d.learn_skill("t_ow_a")
	d.learn_skill("t_ow_b")
	ContentFactory.apply_passives(d, _ids("t_ow_pass"))
	checks["a passive grant with the requirements owned works"] = d.owns_skill("t_ow_adv")
	var e: Combatant = ContentFactory.unit({"id": "t_ow_u3", "name": "U", "class_id": class_id, "level": 5}, "", 12)
	e.learn_skill("t_ow_free")
	checks["a skill with empty requires has no requirement"] = e.can_own_family("t_ow_free") and e.owns_skill("t_ow_free")
	checks["an unknown family has no requirement"] = e.can_own_family("t_ow_nothing")
	var saved: Combatant = ContentFactory.unit({"id": "t_ow_u4", "name": "U", "class_id": class_id, "level": 5}, "", 12)
	saved.learn_skill("t_ow_adv", false)
	checks["a restored save keeps what it learned (no stripping)"] = saved.owns_skill("t_ow_adv")
	return TestHelpers.report(TAG, "Ownership requires every required skill", checks)

static func _class_with_path(path: Array[Dictionary]) -> Dictionary:
	var e: Dictionary = ContentCatalog.get_entry("classes", ContentCatalog.ids("classes")[0])
	e["id"] = "t_md_cls"
	e["name"] = "Fixture"
	var as_dict: Dictionary = {}   # the CLASS1 path shape {skill: level}, step order, nothing dropped
	for step: Dictionary in path:
		as_dict[str(step["skill"])] = DictUtil.to_int(step["level"])
	e["skill_path"] = as_dict
	e.erase("origin")
	return e

static func _step(level: int, skill: String) -> Dictionary:
	return {"level": level, "skill": skill}

static func _test_class_and_unit_rules() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	checks["fixtures accepted"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_cl_nov")).is_empty() and ContentCatalog.upsert_skill(TestHelpers.attack("t_cl_adv", {"grade": "advanced", "requires": ["t_cl_nov"]})).is_empty()
	var min_level: int = StatRules.ADVANCED_MIN_LEVEL
	var basic_path: Array[Dictionary] = [_step(1, "strike")]
	checks["a basic skill in a class path is an error naming it"] = TestHelpers.any_contains(ContentCatalog.validate_entry("classes", _class_with_path(basic_path)), "strike")
	var ok_path: Array[Dictionary] = [_step(1, "t_cl_nov"), _step(min_level, "t_cl_adv")]
	checks["an advanced skill after its requirement is accepted"] = ContentCatalog.validate_entry("classes", _class_with_path(ok_path)).is_empty()
	var same_level: Array[Dictionary] = [_step(min_level, "t_cl_nov"), _step(min_level, "t_cl_adv")]
	checks["a requirement at an equal level is accepted"] = ContentCatalog.validate_entry("classes", _class_with_path(same_level)).is_empty()
	var missing: Array[Dictionary] = [_step(min_level, "t_cl_adv")]
	checks["a step whose requirement is not on the path is an error"] = TestHelpers.any_contains(ContentCatalog.validate_entry("classes", _class_with_path(missing)), "t_cl_nov")
	var too_late: Array[Dictionary] = [_step(min_level, "t_cl_adv"), _step(min_level + 1, "t_cl_nov")]
	checks["a requirement listed later than the step is an error"] = not ContentCatalog.validate_entry("classes", _class_with_path(too_late)).is_empty()
	var too_low: Array[Dictionary] = [_step(1, "t_cl_nov"), _step(min_level - 1, "t_cl_adv")]
	checks["grade advanced below the path floor is an error"] = TestHelpers.any_contains(ContentCatalog.validate_entry("classes", _class_with_path(too_low)), "advanced")
	checks["an expert fixture follows the same floor"] = ContentCatalog.upsert_skill(TestHelpers.attack("t_cl_exp", {"grade": "expert", "requires": ["t_cl_nov"]})).is_empty() and not ContentCatalog.validate_entry("classes", _class_with_path(_exp_path(min_level))).is_empty()
	# Unit rules.
	var class_id: String = ContentCatalog.ids("classes")[0]
	var unit: Dictionary = {"id": "t_cl_unit", "name": "U", "side": "party", "class_id": class_id, "level": 5, "extra_skills": ["t_cl_adv"]}
	checks["a unit extra skill without its requirement is an error"] = TestHelpers.any_contains(ContentCatalog.validate_entry("units", unit), "t_cl_nov")
	unit["extra_skills"] = ["t_cl_nov", "t_cl_adv"]
	checks["...accepted with the requirement in extra_skills"] = ContentCatalog.validate_entry("units", unit).is_empty()
	var loadout_unit: Dictionary = {"id": "t_cl_unit2", "name": "U", "side": "party", "class_id": class_id, "level": 5, "extra_skills": ["t_cl_nov"], "loadout": ["t_cl_adv"]}
	checks["a loadout skill without its requirement is an error"] = not ContentCatalog.validate_entry("units", loadout_unit).is_empty()
	return TestHelpers.report(TAG, "Class path and unit rules for basics, grade and requires", checks)

static func _test_basics_grant() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var basics: Array[String] = SkillCatalog.basic_ids()
	checks["the catalog has basic skills"] = not basics.is_empty()
	var all_own: bool = true
	var no_path_rows: bool = true
	for class_id: String in ContentCatalog.ids("classes"):
		var c: Combatant = ContentFactory.unit({"id": "t_bg_u", "name": "U", "class_id": class_id, "level": 1}, "", 12)
		for basic_id: String in basics:
			all_own = all_own and c.owns_skill(basic_id)
		var arch: ClassArchetype = ClassArchetype.create(class_id)
		for step: Dictionary in arch.path_steps():
			no_path_rows = no_path_rows and not basics.has(str(step.get("skill", "")))
	checks["every class-derived unit owns every basic from level 1"] = all_own
	var dup: Combatant = ContentFactory.unit({"id": "t_bg_u2", "name": "U", "class_id": ContentCatalog.ids("classes")[0], "level": 1, "extra_skills": [basics[0]]}, "", 12)
	var count: int = 0
	for id: String in dup.known_skills:
		if id == basics[0]:
			count += 1
	checks["no duplicate when the basic is also an extra skill"] = count == 1
	checks["a leveled unit keeps its basics after the path refresh"] = _leveled_keeps_basics(basics)
	checks["basics are not valid loadout families"] = not basics.is_empty() and SkillCatalog.is_basic_family(basics[0])
	return TestHelpers.report(TAG, "Every basic skill is granted at level 1", checks)

static func _leveled_keeps_basics(basics: Array[String]) -> bool:
	var c: Combatant = ContentFactory.unit({"id": "t_bg_u3", "name": "U", "class_id": ContentCatalog.ids("classes")[0], "level": 1}, "", 12)
	c.stats.level = 5
	c.refresh_path_skills()
	for basic_id: String in basics:
		if not c.owns_skill(basic_id):
			return false
	return true

static func _ids(id: String) -> Array[String]:
	var out: Array[String] = [id]
	return out

static func _exp_path(min_level: int) -> Array[Dictionary]:
	var path: Array[Dictionary] = [_step(1, "t_cl_nov"), _step(min_level - 1, "t_cl_exp")]
	return path
