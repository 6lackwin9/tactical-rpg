---
class: ProgressionTestSuite
extends: RefCounted
description: "Tactical RPG - Progression Test Suite (M3 / S7) XP curve, kill XP, spend and level-up, state -> Combatant equivalence, passives and skills, state validation. Expectations are derived from the rules and the curve file, never literals."
---

# Properties
* `TAG`: String = "Progression" (const)
* `out`: Array[Dictionary] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING PROGRESSION TEST (M3 / S7)              ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
XpCurve.reset()
var ok: bool = true
ok = _test_curve() and ok
ok = _test_curve_loader() and ok
ok = _test_kill_xp() and ok
ok = _test_spend() and ok
ok = _test_encounter() and ok
ok = _test_equivalence() and ok
ok = _test_passives_skills() and ok
ok = _test_validation() and ok
ok = _test_party() and ok
XpCurve.reset()
return ok
```

### `static _first_party_unit() -> String`
```logic
return str(_party_units()[0]["id"])
```

### `static _test_curve() -> bool`
```logic
var file: Dictionary = XpCurve.load_file(XpCurve.PATH)
var steps: Array = DictUtil.array_of(DictUtil.dict_of(file["data"])["xp_to_next"])
var checks: Dictionary = {"file_valid": DictUtil.array_of(file["errors"]).is_empty(), "loaded_valid": XpCurve.load_errors().is_empty()}
var cumulative: int = 0
var inverse_ok: bool = true
var below_ok: bool = true
checks["level1_zero"] = XpCurve.xp_for_level(1) == 0
for lvl: int in range(2, StatRules.MAX_LEVEL + 1):
	cumulative += DictUtil.to_int(steps[lvl - 2])
	if XpCurve.xp_for_level(lvl) != cumulative:
		inverse_ok = false
	if XpCurve.level_for(XpCurve.xp_for_level(lvl) - 1) != lvl - 1:
		below_ok = false
for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
	if XpCurve.level_for(XpCurve.xp_for_level(lvl)) != lvl:
		inverse_ok = false
checks["thresholds_match_file"] = inverse_ok
checks["one_below_is_previous"] = below_ok
checks["capped_at_max"] = XpCurve.level_for(XpCurve.xp_for_level(StatRules.MAX_LEVEL) * 100) == StatRules.MAX_LEVEL
checks["xp_to_next_zero_at_max"] = XpCurve.xp_to_next(XpCurve.xp_for_level(StatRules.MAX_LEVEL)) == 0
checks["xp_to_next_matches"] = XpCurve.xp_to_next(0) == XpCurve.xp_for_level(2)
var u: UnitState = UnitState.at_level(_first_party_unit(), 4)
checks["at_level_threshold"] = u.total_xp == XpCurve.xp_for_level(4) and u.level() == 4
return TestHelpers.report(TAG, "xp curve", checks)
```

### `static _test_curve_loader() -> bool`
```logic
var good: Dictionary = XpCurve.DEFAULTS.duplicate(true)
var short: Dictionary = good.duplicate(true)
short["xp_to_next"] = [10, 20]
var bad_tier: Dictionary = good.duplicate(true)
DictUtil.dict_of(DictUtil.dict_of(bad_tier["kill_xp"])["tier_mult"]).erase("boss")
var bad_mult: Dictionary = good.duplicate(true)
DictUtil.dict_of(DictUtil.dict_of(bad_mult["kill_xp"])["tier_mult"])["elite"] = 0
var bad_delta: Dictionary = good.duplicate(true)
DictUtil.dict_of(bad_delta["kill_xp"])["level_delta_min"] = 1.5
var missing: Dictionary = XpCurve.load_file("user://no_such_xp_curve.json")
var checks: Dictionary = {
	"defaults_valid": XpCurve.validate(good).is_empty(),
	"short_rejected": not XpCurve.validate(short).is_empty(),
	"missing_tier_rejected": not XpCurve.validate(bad_tier).is_empty(),
	"zero_mult_rejected": not XpCurve.validate(bad_mult).is_empty(),
	"delta_bounds_rejected": not XpCurve.validate(bad_delta).is_empty(),
	"missing_file_defaults": not DictUtil.array_of(missing["errors"]).is_empty() and XpCurve.validate(DictUtil.dict_of(missing["data"])).is_empty(),
	"invalid_override_ignored": not XpCurve.set_override(short).is_empty() and XpCurve.xp_for_level(2) == DictUtil.to_int(DictUtil.array_of(XpCurve.data()["xp_to_next"])[0]),
}
return TestHelpers.report(TAG, "curve loader", checks)
```

### `static _test_kill_xp() -> bool`
```logic
var kill: Dictionary = DictUtil.dict_of(XpCurve.data()["kill_xp"])
var mult: Dictionary = DictUtil.dict_of(kill["tier_mult"])
var lvl: int = 5
var minion: int = XpCurve.kill_xp(lvl, "minion", lvl)
var standard: int = XpCurve.kill_xp(lvl, "standard", lvl)
var elite: int = XpCurve.kill_xp(lvl, "elite", lvl)
var boss: int = XpCurve.kill_xp(lvl, "boss", lvl)
var base: float = DictUtil.to_float(kill["base"]) + DictUtil.to_float(kill["per_level"]) * float(lvl)
var dmin: float = DictUtil.to_float(kill["level_delta_min"])
var dmax: float = DictUtil.to_float(kill["level_delta_max"])
var checks: Dictionary = {
	"ladder": minion < standard and standard < elite and elite < boss,
	"standard_formula": standard == maxi(1, roundi(base * DictUtil.to_float(mult["standard"]))),
	"boss_formula": boss == maxi(1, roundi(base * DictUtil.to_float(mult["boss"]))),
	"delta_max_clamp": XpCurve.kill_xp(10, "standard", 1) == maxi(1, roundi((DictUtil.to_float(kill["base"]) + DictUtil.to_float(kill["per_level"]) * 10.0) * DictUtil.to_float(mult["standard"]) * dmax)),
	"delta_min_clamp": XpCurve.kill_xp(1, "standard", 10) == maxi(1, roundi((DictUtil.to_float(kill["base"]) + DictUtil.to_float(kill["per_level"])) * DictUtil.to_float(mult["standard"]) * dmin)),
	"higher_foe_more_xp": XpCurve.kill_xp(6, "standard", 3) > XpCurve.kill_xp(6, "standard", 6) and XpCurve.kill_xp(6, "standard", 6) > XpCurve.kill_xp(6, "standard", 9),
	"min_one": XpCurve.kill_xp(1, "minion", 10) >= 1,
	"unknown_tier_ok": XpCurve.kill_xp(lvl, "nonsense", lvl) >= 1,
}
var tiny: Dictionary = XpCurve.DEFAULTS.duplicate(true)
DictUtil.dict_of(tiny["kill_xp"])["base"] = 0
DictUtil.dict_of(tiny["kill_xp"])["per_level"] = 0
checks["override_applied"] = XpCurve.set_override(tiny).is_empty() and XpCurve.kill_xp(5, "boss", 5) == 1
XpCurve.reset()
return TestHelpers.report(TAG, "kill xp", checks)
```

### `static _test_spend() -> bool`
```logic
var checks: Dictionary = {}
var unit_id: String = _first_party_unit()
var s: UnitState = UnitState.at_level(unit_id, 3)
var pool: int = StatRules.spend_pool(3)
checks["fresh_unspent_is_pool"] = s.unspent() == pool
var before: Dictionary = s.to_dict()
var too_much: Dictionary = {"STR": pool + 1}
checks["over_draft_refused"] = not LevelUp.confirm(s, too_much).is_empty() and JsonUtil.deep_equal(before, s.to_dict())
checks["unknown_stat_refused"] = not LevelUp.confirm(s, {"LUCK": 1}).is_empty() and JsonUtil.deep_equal(before, s.to_dict())
checks["negative_refused"] = not LevelUp.confirm(s, {"STR": -1}).is_empty() and JsonUtil.deep_equal(before, s.to_dict())
var draft: Dictionary = {"STR": 2, "DEX": 1}
checks["valid_confirm"] = LevelUp.confirm(s, draft).is_empty() and s.unspent() == pool - 3
checks["second_draft_over_remaining_refused"] = not LevelUp.confirm(s, {"INT": pool - 2}).is_empty()
var up: Dictionary = LevelUp.award(s, XpCurve.xp_for_level(4) - s.total_xp)
checks["levelup_adds_free_points"] = DictUtil.to_int(up["levels_gained"]) == 1 and DictUtil.to_int(up["unspent"]) == pool - 3 + StatRules.FREE_PER_LEVEL
checks["award_zero"] = DictUtil.to_int(LevelUp.award(s, 0)["levels_gained"]) == 0
checks["max_level_xp_grows"] = LevelUp.award(UnitState.at_level(unit_id, StatRules.MAX_LEVEL), 99999)["new_level"] == StatRules.MAX_LEVEL
var auto_ok: bool = true
var no_unspent_ok: bool = true
for cls: String in ContentCatalog.ids("classes"):
	var arch: ClassArchetype = ClassArchetype.create(cls)
	for lvl: int in range(1, StatRules.MAX_LEVEL + 1):
		var st: UnitState = UnitState.at_level(unit_id, lvl)
		LevelUp.auto(st, arch.growth_per_level)
		if st.unspent() != 0 or not StatRules.validate_spend(st.spend, lvl).is_empty():
			auto_ok = false
	var partial: UnitState = UnitState.at_level(unit_id, 6)
	LevelUp.confirm(partial, {"KNOW": 2})
	LevelUp.auto(partial, arch.growth_per_level)
	if partial.unspent() != 0 or DictUtil.to_int(partial.spend.get("KNOW", 0)) < 2:
		no_unspent_ok = false
checks["auto_spends_all_every_class_level"] = auto_ok
checks["auto_keeps_confirmed"] = no_unspent_ok
return TestHelpers.report(TAG, "spend and level up", checks)
```

### `static _test_encounter() -> bool`
```logic
var party := PartyState.new()
var ids: Array[String] = []
for i: int in range(PartyState.PARTY_MAX + 1):
	var m: UnitState = UnitState.at_level(_first_party_unit(), 1 + i, "m%d" % i)
	party.add_member(m)
	ids.append(m.id)
var enemies: Array[Dictionary] = [{"level": 3, "tier": "standard"}, {"level": 4, "tier": "elite"}]
var before: Dictionary = {}
for m: UnitState in party.members:
	before[m.id] = m.total_xp
var res: Dictionary = LevelUp.award_encounter(party, enemies)
var active: Array[UnitState] = party.active_members()
var full_ok: bool = active.size() == PartyState.PARTY_MAX and res.size() == active.size()
for m: UnitState in active:
	var expect: int = XpCurve.kill_xp(3, "standard", DictUtil.to_int(DictUtil.dict_of(res[m.id])["old_level"])) + XpCurve.kill_xp(4, "elite", DictUtil.to_int(DictUtil.dict_of(res[m.id])["old_level"]))
	if m.total_xp != DictUtil.to_int(before[m.id]) + expect or DictUtil.to_int(DictUtil.dict_of(res[m.id])["xp"]) != expect:
		full_ok = false
var bench: UnitState = party.members[party.members.size() - 1]
var checks: Dictionary = {
	"every_active_gets_full_xp": full_ok,
	"bench_gets_none": bench.total_xp == DictUtil.to_int(before[bench.id]) and not res.has(bench.id),
}
party.active_ids = [ids[0], ids[4]] as Array[String]
var res2: Dictionary = LevelUp.award_encounter(party, enemies)
checks["explicit_active_ids"] = res2.size() == 2 and res2.has(ids[0]) and res2.has(ids[4])
return TestHelpers.report(TAG, "encounter xp", checks)
```

### `static _gear_line(c: Combatant) -> Dictionary`
```logic
var out: Dictionary = {}
for slot: Variant in c.inventory.equipped_gear:
	var eq: Equipment = c.inventory.equipped_gear[slot]
	out[int(slot)] = eq.id
return out
```

### `static _test_equivalence() -> bool`
```logic
var checks: Dictionary = {}
var seen_classes: Array[String] = []
for entry: Dictionary in _party_units():
	var cls: String = str(entry["class_id"])
	if not seen_classes.has(cls):
		seen_classes.append(cls)
	for lvl: int in [1, 5, StatRules.MAX_LEVEL]:
		var ref_entry: Dictionary = entry.duplicate(true)
		ref_entry["level"] = lvl
		ref_entry["spend"] = {}
		var expected: Combatant = ContentFactory.unit(ref_entry)
		var state: UnitState = UnitState.at_level(str(entry["id"]), lvl)
		var built: Combatant = PartyBuilder.build(state)
		var key: String = "%s_L%d" % [str(entry["id"]), lvl]
		if built == null:
			checks[key] = false
			continue
		checks[key] = built.max_hp == expected.max_hp and built.speed == expected.speed and built.attack_range == expected.attack_range \
			and _stat_line(built) == _stat_line(expected) and built.known_skills == expected.known_skills \
			and built.active_families == expected.active_families and _gear_line(built) == _gear_line(expected) \
			and built.tier == expected.tier and built.catalog_id == expected.catalog_id and built.hp == built.max_hp
checks["covers_party_unit_classes"] = not seen_classes.is_empty() and seen_classes.size() <= ContentCatalog.ids("classes").size()
# Explicit full auto spend equals the empty (automatic) spend; build leaves the state untouched.
var uid: String = _first_party_unit()
var partial: UnitState = UnitState.at_level(uid, 4)
var snapshot: Dictionary = partial.to_dict()
var c1: Combatant = PartyBuilder.build(partial)
var full: UnitState = UnitState.at_level(uid, 4)
LevelUp.auto_for(full)
var c2: Combatant = PartyBuilder.build(full)
checks["auto_on_copy_state_unchanged"] = JsonUtil.deep_equal(snapshot, partial.to_dict()) and partial.unspent() > 0
checks["unspent_builds_like_auto"] = c1 != null and c2 != null and _stat_line(c1) == _stat_line(c2)
# A confirmed custom spend changes the built stats by exactly those points.
var custom: UnitState = UnitState.at_level(uid, 4)
var pool: int = StatRules.spend_pool(4)
LevelUp.confirm(custom, {"KNOW": pool})
var c3: Combatant = PartyBuilder.build(custom)
var base4: Combatant = PartyBuilder.build(UnitState.at_level(uid, 4))
var auto_know: int = DictUtil.int_of(StatRules.auto_spend(custom.archetype().growth_per_level, pool), "KNOW")
checks["custom_spend_applied"] = c3 != null and base4 != null and c3.stats.know_stat - base4.stats.know_stat == pool - auto_know
return TestHelpers.report(TAG, "builder equivalence", checks)
```

### `static _test_passives_skills() -> bool`
```logic
var checks: Dictionary = {}
var uid: String = _first_party_unit()
var passive_id: String = ""
var pids: Array[String] = ContentCatalog.ids("passives")
if not pids.is_empty():
	passive_id = pids[0]
var state: UnitState = UnitState.at_level(uid, 5)
var plain: Combatant = PartyBuilder.build(state)
var skill_id: String = ""
var owned: Array[String] = state.owned_families()
for sid: String in ContentCatalog.ids("skills"):
	var sk: SkillDef = ContentFactory.skill(ContentCatalog.get_entry("skills", sid))
	if not sk.system and not sk.basic and not plain.owns_skill(sid) and not owned.has(SkillCatalog.family_of(sid)):
		skill_id = sid
		break
checks["fixture_found"] = not passive_id.is_empty() and not skill_id.is_empty()
state.passives = [passive_id] as Array[String]
state.learned_skills = [skill_id] as Array[String]
var c: Combatant = PartyBuilder.build(state)
checks["valid_state"] = state.validate().is_empty()
checks["has_passive"] = c != null and c.passives.has(passive_id)
checks["has_learned_skill"] = c != null and c.granted_skills.has(skill_id) and c.owns_skill(skill_id)
checks["learned_family_owned"] = c != null and c.owned_families().has(SkillCatalog.family_of(skill_id))
var snapshot: Dictionary = state.to_dict()
if c != null:
	var temp_ids: Array[String] = []
	for pid: String in pids:
		if pid != passive_id:
			temp_ids.append(pid)
			break
	ContentFactory.apply_passives(c, temp_ids)   # a scripted fight's temporary passive: runtime only
	checks["temp_passive_applied_to_combatant"] = temp_ids.is_empty() or c.passives.has(temp_ids[0])
checks["temp_passive_never_written"] = JsonUtil.deep_equal(snapshot, state.to_dict()) and DictUtil.array_of(state.to_dict()["passives"]).size() == 1
# Stored loadout applied; an invalid one falls back with a warning.
var fams: Array[String] = c.owned_families() if c != null else []
var lo: UnitState = state.duplicate_state()
lo.loadout = {"class": [] as Array[String], "extra": [SkillCatalog.family_of(skill_id)] as Array[String]}
var rep: Dictionary = PartyBuilder.build_report(lo)
var lc: Combatant = rep["combatant"] as Combatant
checks["stored_loadout_applied"] = lc != null and lc.active_extra_families == [SkillCatalog.family_of(skill_id)] and lc.active_class_families.is_empty()
var too_many: UnitState = state.duplicate_state()
too_many.loadout = {"class": fams.slice(0, 1) as Array[String], "extra": fams.slice(0, 1) as Array[String]}
var rep2: Dictionary = PartyBuilder.build_report(too_many)
checks["duplicate_family_invalid"] = fams.size() >= 1 and not DictUtil.array_of(rep2["errors"]).is_empty() and rep2["combatant"] == null
# Gear override and carry.
var gear: UnitState = UnitState.at_level(uid, 3)
var default_c: Combatant = PartyBuilder.build(gear)
var main_default: String = default_c.inventory.get_main_weapon().id if default_c.inventory.get_main_weapon() != null else ""
var alt: String = ""
for item_entry: Dictionary in ContentCatalog.entries("items"):
	if str(item_entry.get("item_type", "")) == "WEAPON" and str(item_entry["id"]) != main_default:
		var cand: UnitState = gear.duplicate_state()
		cand.equipment = {"main_hand": str(item_entry["id"])}
		if cand.validate().is_empty():
			alt = str(item_entry["id"])
			break
if alt.is_empty():
	checks["gear_fixture_found"] = false
else:
	gear.equipment = {"main_hand": alt}
	gear.carried = [alt] as Array[String]
	var gc: Combatant = PartyBuilder.build(gear)
	checks["equipment_override"] = gc != null and gc.inventory.get_main_weapon().id == alt
	checks["carried_in_bag"] = gc != null and gc.inventory.items.size() == 1 and gc.inventory.items[0].id == alt
	checks["default_gear_not_in_bag_twice"] = gc != null and not gc.inventory.items.any(func(it: Item) -> bool: return it.id == main_default and main_default != alt)
return TestHelpers.report(TAG, "passives, skills, loadout, gear", checks)
```

### `static _test_validation() -> bool`
```logic
var uid: String = _first_party_unit()
var good: UnitState = UnitState.at_level(uid, 3)
var checks: Dictionary = {"good_valid": good.validate().is_empty()}
var unknown: UnitState = UnitState.at_level("no_such_unit_xyz", 1)
checks["unknown_base_unit"] = not unknown.validate().is_empty()
var bad_id: UnitState = UnitState.at_level(uid, 1, "Bad Id")
checks["bad_id"] = not bad_id.validate().is_empty()
var over: UnitState = UnitState.at_level(uid, 2)
over.spend = {"STR": StatRules.spend_pool(2) + 1}
checks["spend_above_pool"] = not over.validate().is_empty()
var stat: UnitState = UnitState.at_level(uid, 2)
stat.spend = {"LUCK": 1}
checks["spend_unknown_stat"] = not stat.validate().is_empty()
var foreign: UnitState = UnitState.at_level(uid, 2)
foreign.loadout = {"class": ["family_nobody_owns"] as Array[String], "extra": [] as Array[String]}
checks["unowned_family"] = not foreign.validate().is_empty()
var item: UnitState = UnitState.at_level(uid, 2)
item.equipment = {"main_hand": "no_such_item_xyz"}
checks["missing_item"] = not item.validate().is_empty()
var slot: UnitState = UnitState.at_level(uid, 2)
slot.equipment = {"tail": "x"}
checks["bad_slot"] = not slot.validate().is_empty()
var skill: UnitState = UnitState.at_level(uid, 2)
skill.learned_skills = ["no_such_skill_xyz"] as Array[String]
checks["missing_skill"] = not skill.validate().is_empty()
var pas: UnitState = UnitState.at_level(uid, 2)
pas.passives = ["no_such_passive_xyz"] as Array[String]
checks["missing_passive"] = not pas.validate().is_empty()
var carry: UnitState = UnitState.at_level(uid, 2)
carry.carried = ["no_such_item_xyz"] as Array[String]
checks["missing_carried"] = not carry.validate().is_empty()
var neg: UnitState = UnitState.at_level(uid, 2)
neg.total_xp = -1
checks["negative_xp"] = not neg.validate().is_empty()
var rt: UnitState = UnitState.from_dict(DictUtil.dict_of(JSON.parse_string(JSON.stringify(good.to_dict()))))
checks["json_roundtrip_equal"] = rt.equals(good) and rt.total_xp is int
return TestHelpers.report(TAG, "state validation", checks)
```

### `static _test_party() -> bool`
```logic
var uid: String = _first_party_unit()
var party := PartyState.new()
var checks: Dictionary = {"empty_party_invalid": not party.validate().is_empty()}
for i: int in range(PartyState.PARTY_MAX + 2):
	party.add_member(UnitState.at_level(uid, 1, "m%d" % i))
checks["roster_larger_than_active_valid"] = party.validate().is_empty()
checks["default_active_capped"] = party.active_members().size() == PartyState.PARTY_MAX
party.active_ids = ["m0", "m5"] as Array[String]
checks["explicit_active"] = party.active_members().size() == 2 and party.validate().is_empty()
party.active_ids = ["m0", "m1", "m2", "m3", "m4"] as Array[String]
checks["too_many_active_invalid"] = not party.validate().is_empty()
party.active_ids = ["ghost"] as Array[String]
checks["ghost_active_invalid"] = not party.validate().is_empty()
party.active_ids = [] as Array[String]
party.add_member(UnitState.at_level(uid, 1, "m0"))
checks["duplicate_id_invalid"] = not party.validate().is_empty()
party.members.pop_back()
party.stash = {"no_such_item_xyz": 1}
checks["stash_missing_item"] = not party.validate().is_empty()
var real_item: String = str(ContentCatalog.entries("items")[0]["id"])
party.stash = {real_item: 0}
checks["stash_count_min_1"] = not party.validate().is_empty()
party.stash = {real_item: 3}
checks["stash_valid"] = party.validate().is_empty()
checks["build_active"] = PartyBuilder.build_active(party).size() == PartyState.PARTY_MAX
return TestHelpers.report(TAG, "party state", checks)
```

