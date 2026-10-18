class_name PartyScreenTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Party Screen Test Suite (UI2-L, S7)
# Acceptance checks 1-9 of docs/specs/UI2_L.md on the party, stats, level-up, loadout, gear and battle results models
# (check 10 = the existing suites). Fixture: a NewGame of the shipped Ashlands start party plus one benched level-5
# member, held in memory as CurrentGame; saves go to a sandbox; shipped data is only read (items picked by rule).
# ==============================================================================

const TAG: String = "PartyScreen"
static var SANDBOX: String = TestHelpers.sandbox("party_screen")
const BENCH: String = "ps_bench"

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING PARTY SCREEN TEST (UI2-L)               ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var ok: bool = _report("Sandbox saves under user://", {"sandboxed": SaveGame.base_dir().begins_with("user://")})
	ok = _fresh() and ok
	if ok:
		ok = _test_level_up() and ok
		ok = _fresh() and _test_preview() and ok
		ok = _fresh() and _test_auto() and ok
		ok = _fresh() and _test_active() and ok
		ok = _fresh() and _test_loadout() and ok
		ok = _fresh() and _test_new_tiers() and ok
		ok = _fresh() and _test_gear() and ok
		ok = _fresh() and _test_results() and ok
		ok = _fresh() and _test_router() and ok
		ok = _fresh() and _test_review_f2() and ok
	CurrentGame.end()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

# A new Ashlands game topped up with filler members to the world's party cap (all active), plus a benched level-5
# copy of the first member, as CurrentGame.
static func _fresh() -> bool:
	var res: Dictionary = NewGame.create_for_world("ashlands", 11)
	var gs: GameState = GameState.of(res["state"])
	if gs == null or gs.party.members.is_empty() or gs.party.members.size() > _cap():
		return _report("Fixture game", {"ashlands start party": false})
	var fill: int = 0
	while gs.party.members.size() < _cap():
		fill += 1
		gs.party.add_member(UnitState.at_level(gs.party.members[0].base_unit, 1, "ps_fill_%d" % fill))
	gs.party.add_member(UnitState.at_level(gs.party.members[0].base_unit, 5, BENCH))
	CurrentGame.start(gs)
	return true

# The world's active cap (the shipped Ashlands party_max).
static func _cap() -> int:
	return WorldCatalog.party_max("ashlands")

static func _first() -> UnitState:
	return CurrentGame.state().party.members[0]

# Leaves exactly `points` unspent on `m` (the rest goes into DEX).
static func _leave_unspent(m: UnitState, points: int) -> void:
	m.spend = {}
	if m.unspent() < points:
		m.total_xp = XpCurve.xp_for_level(m.level() + 1)
	m.spend = {"DEX": m.unspent() - points}

# Checks 1-2: drafting, the points limit, confirm equivalence with LevelUp.confirm, cancel.
static func _test_level_up() -> bool:
	var checks: Dictionary = {}
	var m: UnitState = _first()
	_leave_unspent(m, 2)
	var twin: UnitState = m.duplicate_state()
	var before_str: int = DictUtil.to_int(m.spend.get("STR", 0))
	var lm := LevelUpModel.new(m.id)
	checks["two adds"] = lm.add("STR").is_empty() and lm.add("STR").is_empty() and lm.points_left() == 0
	checks["third add refused"] = lm.add("STR") == [LevelUpModel.NO_POINTS]
	checks["remove refused at 0 drafted"] = not lm.remove("CONS").is_empty()
	checks["member untouched by the draft"] = m.equals(twin) and lm.has_draft()
	var draft: Dictionary = lm.draft.duplicate()
	checks["confirm"] = lm.confirm().is_empty() and DictUtil.to_int(m.spend.get("STR", 0)) == before_str + 2 and m.unspent() == 0
	checks["equivalence"] = LevelUp.confirm(twin, draft).is_empty() and twin.equals(m)
	_leave_unspent(m, 2)
	var orig: UnitState = m.duplicate_state()
	var lm2 := LevelUpModel.new(m.id)
	var _a: Array[String] = lm2.add("CONS")
	lm2.cancel()
	checks["cancel leaves the member equal"] = m.equals(orig) and not lm2.has_draft()
	return _report("Checks 1-2: level-up draft, confirm and cancel", checks)

# Check 3: before / after differ as PartyPreview.derived of the confirmed state; +1 CONS adds HP_PER_CONS HP x the tier factor.
static func _test_preview() -> bool:
	var checks: Dictionary = {}
	var m: UnitState = _first()
	_leave_unspent(m, 1)
	var lm := LevelUpModel.new(m.id)
	var before: Dictionary = lm.before()
	var _a: Array[String] = lm.add("CONS")
	var after: Dictionary = lm.after()
	var scale: float = PartyBuilder.build(m.duplicate_state(), false).hp_scale
	checks["before = current"] = JsonUtil.deep_equal(before, PartyPreview.derived(m))
	checks["hp +8 x tier factor"] = absi(DictUtil.to_int(after["max_hp"]) - DictUtil.to_int(before["max_hp"]) - roundi(float(CharacterStats.HP_PER_CONS) * scale)) <= 1
	checks["confirm gives the after block"] = lm.confirm().is_empty() and JsonUtil.deep_equal(PartyPreview.derived(m), after)
	checks["all derived keys"] = before.keys().size() == PartyPreview.DERIVED_LABELS.size()
	return _report("Check 3: derived block before and after", checks)

# Check 4: auto() drafts what LevelUp.auto_for spends on a copy.
static func _test_auto() -> bool:
	var m: UnitState = _first()
	_leave_unspent(m, 3)
	var copy: UnitState = m.duplicate_state()
	LevelUp.auto_for(copy)
	var lm := LevelUpModel.new(m.id)
	lm.auto()
	var drafted: UnitState = lm.drafted_state()
	return _report("Check 4: auto matches LevelUp.auto_for", {"same spend": JsonUtil.deep_equal(drafted.spend, copy.spend), "points used": lm.points_left() == 0})

# Check 5: the active party, capped at the world's party_max (rule 2).
static func _test_active() -> bool:
	var checks: Dictionary = {}
	var party: PartyState = CurrentGame.state().party
	var pm := PartyModel.new(null)
	checks["implicit full party"] = party.active_ids.is_empty() and pm.active_count() == _cap() and pm.max_active() == _cap()
	checks["beyond the cap refused"] = pm.toggle_active(BENCH) == [PartyModel.MAX_ACTIVE % _cap()]
	checks["first edit writes the active ids"] = party.active_ids == party.member_ids().slice(0, _cap())
	for id: String in party.member_ids().slice(1, _cap()):
		var _e: Array[String] = pm.toggle_active(id)
	checks["last one kept"] = pm.toggle_active(party.members[0].id) == [PartyModel.KEEP_ONE] and pm.active_count() == 1
	checks["bench joins"] = pm.toggle_active(BENCH).is_empty() and pm.active_count() == 2
	return _report("Check 5: active party limits", checks)

# Check 6: the class slot limit, basics never slottable, use_default.
static func _test_loadout() -> bool:
	var checks: Dictionary = {}
	var m: UnitState = CurrentGame.state().party.member(BENCH)
	var lo := LoadoutModel.new(BENCH)
	var basics: Array[String] = lo.basics()
	var slottable: Array[String] = []
	var fams: Dictionary = lo.families()
	for group: String in ["class", "extra"]:
		for row: Dictionary in fams[group]:
			slottable.append(str(row["family"]))
	var no_basic: bool = true
	for fam: String in slottable:
		no_basic = no_basic and not SkillCatalog.is_basic_family(fam)
	checks["no basic slottable"] = no_basic and not basics.is_empty()
	checks["basic refused"] = not lo.slot(basics[0], "class", 0).is_empty()
	var pool: Array[String] = slottable.duplicate()
	lo.class_slots.clear()
	var i: int = 0
	while lo.class_slots.size() < StatRules.MAX_CLASS_SLOTS + 1 and i < pool.size():
		var _e: Array[String] = lo.slot(pool[i], "class", 99)
		i += 1
	while lo.class_slots.size() < StatRules.MAX_CLASS_SLOTS + 1:
		lo.class_slots.append("ps_fake_%d" % lo.class_slots.size())
	var limit_msg: String = "Unit '%s': at most %d class families in the loadout." % [BENCH, StatRules.MAX_CLASS_SLOTS]
	var orig: Dictionary = m.loadout.duplicate(true)
	checks["sixth refused"] = lo.confirm().has(limit_msg) and JsonUtil.deep_equal(m.loadout, orig)
	lo.use_default()
	var default_built: Combatant = PartyBuilder.build(m.duplicate_state(), false)
	checks["use_default"] = JsonUtil.deep_equal(lo.draft_loadout(), {"class": [], "extra": []}) and lo.class_slots == default_built.active_class_families
	checks["confirm default"] = lo.confirm().is_empty() and JsonUtil.strings(m.loadout["class"]).is_empty()
	return _report("Check 6: loadout limits and default", checks)

# Check 7: new tiers since level 4 for a level-5 member, against two at_level builds; empty when nothing unlocked.
static func _test_new_tiers() -> bool:
	var m: UnitState = CurrentGame.state().party.member(BENCH)
	var c5: Combatant = PartyBuilder.build(UnitState.at_level(m.base_unit, 5), false)
	var c4: Combatant = PartyBuilder.build(UnitState.at_level(m.base_unit, 4), false)
	var want: Array[String] = []
	for fam: String in c5.owned_families():
		for sk: SkillDef in SkillCatalog.family_tiers(fam):
			if c5.tier_unlocked(sk) and not (c4.owned_families().has(fam) and c4.tier_unlocked(sk)):
				want.append(sk.id)
	var lm := LevelUpModel.new(BENCH)
	var got: Array[String] = []
	for t: Dictionary in lm.new_tiers(4):
		got.append(str(t["skill"]))
	want.sort()
	got.sort()
	return _report("Check 7: new tiers after a level-up", {"matches the rule": got == want, "none at the same level": lm.new_tiers(5).is_empty()})

# Picks an item fitting `slot` that the member's class can (`usable`) or can't use; "" when none.
static func _item_for(m: UnitState, slot: String, usable: bool, skip: String = "") -> String:
	var prof: Proficiency = m.archetype().make_proficiency()
	for e: Dictionary in ContentCatalog.entries("items"):
		var id: String = str(e["id"])
		var wt: String = str(e.get("weapon_type", ""))
		if id == skip or id == ContentSchema.NO_ITEM or wt.is_empty() or not ContentFactory.item_fits_slot(e, slot.to_upper(), true):
			continue
		if prof.can_use(wt) == usable:
			return id
	return ""

# Check 8: an unusable item returns the validator message; a valid swap moves the old item to the stash, cancel restores.
static func _test_gear() -> bool:
	var checks: Dictionary = {}
	var m: UnitState = _first()
	var party: PartyState = CurrentGame.state().party
	var slot: String = "main_hand"
	var bad: String = _item_for(m, slot, false)
	var a: String = _item_for(m, slot, true)
	var b: String = _item_for(m, slot, true, a)
	checks["fixture items"] = not bad.is_empty() and not a.is_empty() and not b.is_empty()
	if not DictUtil.to_bool(checks["fixture items"]):
		return _report("Check 8: gear", checks)
	for id: String in [bad, a, b]:
		party.add_to_stash(id)
	var orig_member: UnitState = m.duplicate_state()
	var orig_stash: Dictionary = party.stash.duplicate()
	var gm := GearModel.new(m.id)
	var trial: UnitState = m.duplicate_state()
	trial.equipment[slot] = bad
	var expected: Array[String] = []
	for e: String in trial.validate():
		if not e.contains("loadout"):
			expected.append(e)
	checks["unusable refused verbatim"] = not expected.is_empty() and gm.equip(slot, bad) == expected and not gm.work.equipment.has(slot)
	checks["equip a"] = gm.equip(slot, a).is_empty() and not gm.stash.has(a)
	checks["swap moves a back"] = gm.equip(slot, b).is_empty() and DictUtil.to_int(gm.stash.get(a, 0)) == 1 and not gm.stash.has(b)
	checks["originals untouched"] = m.equals(orig_member) and JsonUtil.deep_equal(party.stash, orig_stash) and gm.has_draft()
	gm.cancel()
	checks["cancel restores"] = gm.work.equals(m) and JsonUtil.deep_equal(gm.stash, orig_stash) and not gm.has_draft()
	var _e1: Array[String] = gm.equip(slot, a)
	var _e2: Array[String] = gm.unequip(slot)
	checks["unequip back to default"] = not gm.work.equipment.has(slot) and DictUtil.to_int(gm.stash.get(a, 0)) == 1
	var _e3: Array[String] = gm.equip(slot, b)
	checks["confirm writes both"] = gm.confirm().is_empty() and str(m.equipment[slot]) == b and not party.stash.has(b)
	return _report("Check 8: gear swap, refusal and cancel", checks)

# Check 9: battle results from a fixture award.
static func _test_results() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = CurrentGame.state()
	for m: UnitState in gs.party.members:
		m.spend = StatRules.auto_spend(m.archetype().growth_per_level, m.unspent()) if m.unspent() > 0 else m.spend
	var router := MenuRouter.new(MenuRouter.HUB)
	var small: Dictionary = LevelUp.award_encounter(gs.party, [{"level": 1, "tier": "minion"}] as Array[Dictionary])
	var rm := BattleResultsModel.new(router, small)
	var ids: Array[String] = []
	for r: Dictionary in rm.rows():
		ids.append(str(r["id"]))
	var levelled: bool = false
	for r: Dictionary in rm.rows():
		levelled = levelled or (DictUtil.to_int(r["new_level"]) > DictUtil.to_int(r["old_level"]) and DictUtil.to_int(r["unspent"]) > 0)
	checks["active members only"] = not ids.has(BENCH) and ids.size() == gs.party.active_members().size()
	checks["prompt iff a level with points"] = rm.prompt_level_up() == levelled
	var big: Dictionary = LevelUp.award_encounter(gs.party, [{"level": 10, "tier": "boss"}, {"level": 10, "tier": "boss"}] as Array[Dictionary])
	var rm2 := BattleResultsModel.new(router, big)
	var any_level: bool = false
	for r: Dictionary in rm2.rows():
		any_level = any_level or DictUtil.to_int(r["new_level"]) > DictUtil.to_int(r["old_level"])
	checks["levels gained prompt"] = any_level and rm2.prompt_level_up()
	router.push(MenuRouter.RESULTS, {"award": big})
	var res_model: BattleResultsModel = router.model() as BattleResultsModel
	var first: String = str(res_model.rows()[0]["id"])
	checks["level_up_now opens level-up"] = res_model.level_up_now(first).is_empty() and router.top() == MenuRouter.LEVEL_UP
	router.pop()
	checks["back to results"] = router.top() == MenuRouter.RESULTS
	var done: Array[int] = [0]
	router.results_done.connect(func() -> void: done[0] += 1)
	checks["continue autosaves"] = res_model.continue_on().is_empty() and SaveGame.exists(SaveGame.AUTOSAVE_SLOT) and not CurrentGame.is_dirty()
	checks["returns to the caller"] = router.top() == MenuRouter.HUB and done[0] == 1
	checks["points kept after skip"] = gs.party.members[0].unspent() > 0
	return _report("Check 9: battle results", checks)

# REVIEW-F2: gear errors tagged by source (a gear error naming "loadout" stays a gear error), null-safe level-up model,
# Q2 level-up Confirm asks first, "Open loadout" on the results card.
static func _test_review_f2() -> bool:
	var checks: Dictionary = {}
	var m: UnitState = _first()
	var gm := GearModel.new(m.id)
	gm.work.equipment["main_hand"] = "loadout_no_such_item"
	var gear_errs: Array[String] = gm.confirm()
	checks["gear error with 'loadout' stays a gear error"] = not gear_errs.is_empty() and gear_errs.any(func(e: String) -> bool: return e.contains("loadout_no_such_item")) and not m.equipment.has("main_hand")
	var split: Dictionary = gm.work.validate_split()
	checks["split keeps loadout errors apart"] = DictUtil.array_of(split["loadout"]).is_empty() and not DictUtil.array_of(split["other"]).is_empty()
	var ghost := LevelUpModel.new("nobody")
	checks["stale member: empty blocks"] = ghost.before().is_empty() and ghost.after().is_empty() and ghost.drafted_state() == null
	checks["stale member: confirm returns the error"] = ghost.confirm() == [LevelUpModel.NO_MEMBER] and ghost.ask_confirm(MenuRouter.new(MenuRouter.HUB)) == [LevelUpModel.NO_MEMBER]
	_leave_unspent(m, 2)
	var router := MenuRouter.new(MenuRouter.HUB)
	var lm := LevelUpModel.new(m.id)
	var _a: Array[String] = lm.add("STR")
	var before_str: int = DictUtil.to_int(m.spend.get("STR", 0))
	checks["Q2 confirm asks first"] = lm.ask_confirm(router).is_empty() and router.is_asking() and router.pending_message == LevelUpModel.SPEND_QUESTION and DictUtil.to_int(m.spend.get("STR", 0)) == before_str
	router.decline()
	checks["declined: nothing spent, draft kept"] = DictUtil.to_int(m.spend.get("STR", 0)) == before_str and lm.has_draft()
	var _b: Array[String] = lm.ask_confirm(router)
	router.confirm()
	checks["accepted: spent"] = lm.last_errors.is_empty() and DictUtil.to_int(m.spend.get("STR", 0)) == before_str + 1 and not lm.has_draft()
	var empty := LevelUpModel.new(m.id)
	checks["no draft: no question"] = empty.ask_confirm(router).is_empty() and not router.is_asking()
	var rm := BattleResultsModel.new(router, {m.id: {"xp": 1, "old_level": 1, "new_level": 2, "unspent": 0}})
	var lo := LoadoutModel.new(m.id)
	var free: bool = lo.class_slots.size() < StatRules.MAX_CLASS_SLOTS or lo.extra_slots.size() < StatRules.MAX_EXTRA_SLOTS
	checks["free slot rule"] = rm.has_free_slot(m.id) == free and not rm.has_free_slot("nobody")
	router.push(MenuRouter.RESULTS, {"award": {m.id: {"xp": 1, "old_level": 1, "new_level": 2, "unspent": 0}}})
	var shown: BattleResultsModel = router.model() as BattleResultsModel
	checks["open_loadout pushes the loadout screen"] = shown.open_loadout(m.id).is_empty() and router.top() == MenuRouter.LOADOUT and (router.model() as LoadoutModel).member_id == m.id
	router.pop()
	checks["back on results"] = router.top() == MenuRouter.RESULTS
	router.pop()
	checks["generic pop never pops results"] = router.top() == MenuRouter.RESULTS
	return _report("REVIEW-F2: gear error source, null-safe level-up, Q2 ask, Open loadout", checks)

# Router wiring: party screens need a game; the hub's Party entry opens; level-up asks before discarding a draft.
static func _test_router() -> bool:
	var checks: Dictionary = {}
	var router := MenuRouter.new(MenuRouter.HUB)
	var hub: HubModel = router.model() as HubModel
	checks["party opens"] = hub.can_open(MenuRouter.PARTY) and hub.open(MenuRouter.PARTY).is_empty() and router.top() == MenuRouter.PARTY
	var pm: PartyModel = router.model() as PartyModel
	_leave_unspent(_first(), 1)
	checks["badge"] = pm.badge()
	checks["open level-up"] = pm.open(MenuRouter.LEVEL_UP).is_empty() and router.top() == MenuRouter.LEVEL_UP
	var lm: LevelUpModel = router.model() as LevelUpModel
	var _a: Array[String] = lm.add("STR")
	router.pop()
	checks["draft asks"] = router.is_asking() and router.top() == MenuRouter.LEVEL_UP
	router.confirm()
	checks["discarded"] = router.top() == MenuRouter.PARTY and _first().unspent() == 1
	CurrentGame.end()
	checks["no game, no party"] = router.make_model(MenuRouter.PARTY) == null and router.make_model(MenuRouter.GEAR, {"member": "x"}) == null
	return _report("Router and hub wiring", checks)
