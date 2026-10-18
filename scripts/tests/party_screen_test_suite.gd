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

static func run(tree_parent: Node = null) -> bool:
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
		if tree_parent != null:
			ok = _test_pouch_screen(tree_parent) and ok
			ok = _fresh() and _test_character_popup(tree_parent) and ok
			ok = _fresh() and _test_character_fit(tree_parent) and ok
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
	checks["open level-up is a tab"] = pm.open(MenuRouter.LEVEL_UP).is_empty() and router.top() == MenuRouter.PARTY and pm.tab == PartyModel.TAB_LEVEL_UP
	var lm: LevelUpModel = pm.level_model()
	var _a: Array[String] = lm.add("STR")
	router.pop()
	checks["draft asks"] = router.is_asking() and router.top() == MenuRouter.PARTY and router.pending_message == MenuRouter.DISCARD_QUESTION
	router.confirm()
	checks["discarded"] = router.top() == MenuRouter.HUB and _first().unspent() == 1
	CurrentGame.end()
	checks["no game, no party"] = router.make_model(MenuRouter.PARTY) == null and router.make_model(MenuRouter.GEAR, {"member": "x"}) == null
	return _report("Router and hub wiring", checks)

# ------------------------------------------------------------------------------
# ITEMUSE1-U: the battle pouch on the Gear screen (one screen per case; consumable picked by rule)
# ------------------------------------------------------------------------------

const VIEW: Vector2 = Vector2(1280, 720)

# A catalog item the pouch accepts: the first whose stash row shows up as a pouch candidate.
static func _pouch_item(m: UnitState, skip: String = "") -> String:
	for e: Dictionary in ContentCatalog.entries("items"):
		var id: String = str(e["id"])
		if id == skip or id == ContentSchema.NO_ITEM or str(e.get("item_type", "")) != "CONSUMABLE":
			continue
		var probe := GearModel.new(m.id)
		probe.stash[id] = 1
		if probe.pouch_candidates().any(func(c: Dictionary) -> bool: return str(c["item"]) == id):
			return id
	return ""

# A host on the Gear screen of the first member with `stock` of `item` in the stash.
static func _gear_host(tree_parent: Node, item: String, stock: int) -> Dictionary:
	var m: UnitState = _first()
	m.pouch.clear()
	var party: PartyState = CurrentGame.state().party
	party.stash.clear()
	for i: int in range(stock):
		party.add_to_stash(item)
	var holder := SubViewport.new()
	holder.size = Vector2i(VIEW)
	tree_parent.add_child(holder)
	var host: MainMenuView = (load("res://scenes/main_menu.tscn") as PackedScene).instantiate() as MainMenuView
	holder.add_child(host)
	host.router.push(MenuRouter.GEAR, {"member": m.id})
	SettingsUiTestSuite.layout(host)
	return {"holder": holder, "host": host, "member": m}

static func _drop_host(f: Dictionary) -> void:
	var holder: Node = f["holder"] as Node
	holder.get_parent().remove_child(holder)
	holder.free()

static func _btn(host: Node, node_name: String) -> Button:
	return host.find_child(node_name, true, false) as Button

static func _press(host: MainMenuView, b: Button) -> void:
	b.pressed.emit()
	SettingsUiTestSuite.layout(host)

static func _stash_count(m: GearModel, item: String) -> int:
	return DictUtil.to_int(m.stash.get(item, 0))

static func _test_pouch_screen(tree_parent: Node) -> bool:
	var m0: UnitState = _first()
	var item: String = _pouch_item(m0)
	if item.is_empty():
		return _report("ITEMUSE1-U: pouch on the Gear screen", {"fixture consumable": false})
	var all_ok: bool = true
	var party: PartyState = CurrentGame.state().party
	var add_name: String = "PouchAdd_" + item

	# Add moves one from the stash into the next slot.
	var f: Dictionary = _gear_host(tree_parent, item, UnitState.POUCH_MAX + 1)
	var host: MainMenuView = f["host"] as MainMenuView
	var gm: GearModel = host.router.model() as GearModel
	var c: Dictionary = {}
	c["three empty slots"] = host.find_children("PouchSlot*", "HBoxContainer", true, false).size() == UnitState.POUCH_MAX and gm.pouch_rows().is_empty()
	c["candidate count"] = gm.pouch_candidates().any(func(r: Dictionary) -> bool: return str(r["item"]) == item and DictUtil.to_int(r["count"]) == UnitState.POUCH_MAX + 1)
	_press(host, _btn(host, add_name))
	var gm2: GearModel = host.router.model() as GearModel
	c["slot 0 holds it"] = gm2.pouch_rows().size() == 1 and str(gm2.pouch_rows()[0]["item"]) == item and gm2.work.pouch[0] == item
	c["stash dropped by one"] = _stash_count(gm2, item) == UnitState.POUCH_MAX
	c["candidate count dropped"] = gm2.pouch_candidates().any(func(r: Dictionary) -> bool: return str(r["item"]) == item and DictUtil.to_int(r["count"]) == UnitState.POUCH_MAX)
	c["Remove button shown"] = _btn(host, "PouchRemove0") != null and _btn(host, "PouchRemove1") == null
	c["real stash untouched until Confirm"] = DictUtil.to_int(party.stash.get(item, 0)) == UnitState.POUCH_MAX + 1 and m0.pouch.is_empty()
	all_ok = _report("ITEMUSE1-U: Add moves one into the next slot", c) and all_ok
	_drop_host(f)

	# Add is disabled at POUCH_MAX.
	f = _gear_host(tree_parent, item, UnitState.POUCH_MAX + 1)
	host = f["host"] as MainMenuView
	for i: int in range(UnitState.POUCH_MAX):
		_press(host, _btn(host, add_name))
	c = {}
	var full_add: Button = _btn(host, add_name)
	c["pouch full"] = (host.router.model() as GearModel).pouch_rows().size() == UnitState.POUCH_MAX
	c["Add disabled"] = full_add != null and full_add.disabled
	c["one left in the stash"] = _stash_count(host.router.model() as GearModel, item) == 1
	all_ok = _report("ITEMUSE1-U: Add disabled at POUCH_MAX", c) and all_ok
	_drop_host(f)

	# Remove returns the item.
	f = _gear_host(tree_parent, item, 1)
	host = f["host"] as MainMenuView
	_press(host, _btn(host, add_name))
	c = {}
	c["emptied the stash row"] = not (host.router.model() as GearModel).stash.has(item)
	_press(host, _btn(host, "PouchRemove0"))
	var back: GearModel = host.router.model() as GearModel
	c["pouch empty again"] = back.pouch_rows().is_empty()
	c["item back in the stash"] = _stash_count(back, item) == 1
	c["Add enabled again"] = _btn(host, add_name) != null and not _btn(host, add_name).disabled
	all_ok = _report("ITEMUSE1-U: Remove returns the item", c) and all_ok
	_drop_host(f)

	# Back discards the draft (after the discard question).
	f = _gear_host(tree_parent, item, 1)
	host = f["host"] as MainMenuView
	var member: UnitState = f["member"] as UnitState
	_press(host, _btn(host, add_name))
	c = {}
	host.router.pop()
	c["asks before discarding"] = host.router.is_asking() and host.router.top() == MenuRouter.GEAR
	host.router.confirm()
	c["left the screen"] = host.router.top() != MenuRouter.GEAR
	c["pouch unchanged"] = member.pouch.is_empty()
	c["stash unchanged"] = DictUtil.to_int(party.stash.get(item, 0)) == 1
	all_ok = _report("ITEMUSE1-U: Back discards the pouch draft", c) and all_ok
	_drop_host(f)

	# Confirm writes UnitState.pouch and the stash.
	f = _gear_host(tree_parent, item, 2)
	host = f["host"] as MainMenuView
	member = f["member"] as UnitState
	_press(host, _btn(host, add_name))
	_press(host, _btn(host, add_name))
	c = {}
	var confirm: Button = null
	for n: Node in host.find_children("*", "Button", true, false):
		if (n as Button).text == "Confirm":
			confirm = n as Button
	c["Confirm enabled"] = confirm != null and not confirm.disabled
	_press(host, confirm)
	c["pouch written"] = member.pouch.size() == 2 and member.pouch[0] == item and member.pouch[1] == item
	c["stash written"] = DictUtil.to_int(party.stash.get(item, 0)) == 0
	all_ok = _report("ITEMUSE1-U: Confirm writes UnitState.pouch", c) and all_ok
	member.pouch.clear()
	_drop_host(f)

	# Fit at every text size, with a full pouch and a stocked stash.
	c = {}
	for scale: float in SettingsFile.TEXT_LEVELS:
		SettingsFile.set_value("ui/text_scale", scale)
		f = _gear_host(tree_parent, item, UnitState.POUCH_MAX + 1)
		host = f["host"] as MainMenuView
		_press(host, _btn(host, add_name))
		_press(host, _btn(host, add_name))
		var tag: String = "text %.2f" % scale
		var area := Rect2(Vector2.ZERO, VIEW)
		var body: Control = host.find_child("GearBody", true, false) as Control
		var scroll: ScrollContainer = host.find_child("FrameScroll", true, false) as ScrollContainer
		var section: Control = host.find_child("PouchSection", true, false) as Control
		c[tag + ": section built"] = body != null and scroll != null and section != null
		if body != null and scroll != null and section != null:
			c[tag + ": no sideways overflow"] = body.get_combined_minimum_size().x <= scroll.size.x + 1.0
			c[tag + ": section as wide as its rows"] = section.get_rect().size.x >= section.get_combined_minimum_size().x - 1.0
		var shown: Array[String] = []
		for n: Node in host.find_children("*", "Button", true, false):
			var b: Button = n as Button
			if b.is_visible_in_tree() and (b.text == "Confirm" or b.name == "CloseButton") and not area.encloses(b.get_global_rect()):
				shown.append(b.text)
		c[tag + ": Confirm and Back inside 1280x720"] = shown.is_empty() and _btn(host, add_name) != null
		_drop_host(f)
	SettingsFile.set_value("ui/text_scale", 1.0)
	all_ok = _report("ITEMUSE1-U: pouch section fits at every text size", c) and all_ok
	return all_ok

# ------------------------------------------------------------------------------
# UIR-3: the Character popup (tabs, member strip, level-up table, skills text, gear pouch, close routes)
# ------------------------------------------------------------------------------

# A host on the Character screen (PARTY) for `tab`; members.size() >= 2 by the fixture.
static func _char_host(tree_parent: Node, tab: String) -> Dictionary:
	var holder := SubViewport.new()
	holder.size = Vector2i(VIEW)
	tree_parent.add_child(holder)
	var host: MainMenuView = (load("res://scenes/main_menu.tscn") as PackedScene).instantiate() as MainMenuView
	holder.add_child(host)
	host.router.push(MenuRouter.PARTY, {"member": _first().id, "tab": tab})
	SettingsUiTestSuite.layout(host)
	return {"holder": holder, "host": host}

static func _key(code: Key) -> InputEventKey:
	var ev := InputEventKey.new()
	ev.keycode = code
	ev.pressed = true
	return ev

static func _right_click() -> InputEventMouseButton:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT
	ev.pressed = true
	return ev

static func _test_character_popup(tree_parent: Node) -> bool:
	var all_ok: bool = true
	SettingsFile.set_value("ui/text_scale", 1.0)
	var m0: UnitState = _first()
	_leave_unspent(m0, 2)

	# Tabs exist, switch, and the selected one is marked by style, not by text.
	var f: Dictionary = _char_host(tree_parent, "")
	var host: MainMenuView = f["host"] as MainMenuView
	var pm: PartyModel = host.router.model() as PartyModel
	var c: Dictionary = {}
	var tabs_ok: bool = true
	for id: String in PartyModel.TABS:
		tabs_ok = tabs_ok and _btn(host, "Tab_" + id) != null
	c["four tabs"] = tabs_ok and pm.tab == PartyModel.TAB_STATS
	c["selected tab styled"] = MenuStyle.is_selected(_btn(host, "Tab_" + PartyModel.TAB_STATS)) and not MenuStyle.is_selected(_btn(host, "Tab_" + PartyModel.TAB_GEAR))
	for id: String in PartyModel.TABS:
		_press(host, _btn(host, "Tab_" + id))
		c["switch to " + id] = pm.tab == id and MenuStyle.is_selected(_btn(host, "Tab_" + id))
	c["no Back button, an X"] = _btn(host, "BackButton") == null and _btn(host, "CloseButton") != null
	# Member strip and Q / E.
	var ids: Array[String] = CurrentGame.state().party.member_ids()
	c["strip per member"] = host.find_children("Member_*", "Button", true, false).size() == ids.size()
	_press(host, _btn(host, "Member_" + ids[1]))
	c["member pick"] = pm.selected == ids[1] and MenuStyle.is_selected(_btn(host, "Member_" + ids[1])) and not MenuStyle.is_selected(_btn(host, "Member_" + ids[0]))
	_view(host)._unhandled_input(_key(KEY_E))
	SettingsUiTestSuite.layout(host)
	c["E: next member"] = pm.selected == ids[2 % ids.size()]
	_view(host)._unhandled_input(_key(KEY_Q))
	_view(host)._unhandled_input(_key(KEY_Q))
	c["Q twice: back to the first"] = pm.selected == ids[0]
	_view(host)._unhandled_input(_key(KEY_Q))
	c["Q wraps to the last"] = pm.selected == ids[-1]
	all_ok = _report("UIR-3: tabs, member strip, Q / E", c) and all_ok
	_drop_host(f)

	# Level up: now / added / new, Revert, the one discard rule, Confirm apart from Auto / Revert.
	f = _char_host(tree_parent, PartyModel.TAB_LEVEL_UP)
	host = f["host"] as MainMenuView
	pm = host.router.model() as PartyModel
	c = {}
	var stat: String = StatRules.STATS[0]
	var at: int = StatRules.STATS.find(stat)
	var now: int = DictUtil.to_int(DictUtil.dict_of(StatsModel.new(m0.id).stats()[stat])["total"])
	_press(host, _btn(host, "Plus_" + stat))
	var cells: Array[String] = _grid_texts(host, "AllocationGrid")
	var row: int = 6 * (1 + at)
	c["row shows now / added / new"] = cells.size() >= row + 4 and cells[row] == stat and cells[row + 1] == str(now) and cells[row + 2] == "+1" and cells[row + 3] == str(now + 1)
	c["draft in the model"] = pm.level_model().has_draft() and pm.has_draft()
	var auto: Button = _btn(host, "AutoButton")
	var revert: Button = _btn(host, "RevertButton")
	var confirm: Button = _btn(host, "PrimaryButton")
	c["Auto and Revert beside the rows"] = auto != null and revert != null and host.find_child("LevelUpRow", true, false).is_ancestor_of(auto) and host.find_child("LevelUpRow", true, false).is_ancestor_of(revert)
	c["Confirm is the bar's primary"] = confirm != null and confirm.text == "Confirm" and host.find_child("ActionBar", true, false).is_ancestor_of(confirm) and not confirm.disabled
	c["Auto and Revert are not in the bar"] = not host.find_child("ActionBar", true, false).is_ancestor_of(auto)
	_press(host, _btn(host, "Tab_" + PartyModel.TAB_STATS))
	c["tab switch asks first"] = host.router.is_asking() and host.router.pending_message == MenuRouter.DISCARD_QUESTION and pm.tab == PartyModel.TAB_LEVEL_UP
	host.answer(false)
	c["No keeps the draft and the tab"] = pm.tab == PartyModel.TAB_LEVEL_UP and pm.level_model().has_draft()
	_press(host, _btn(host, "RevertButton"))
	c["Revert clears the draft"] = not pm.level_model().has_draft() and _btn(host, "RevertButton").disabled and _btn(host, "PrimaryButton").disabled
	_press(host, _btn(host, "Plus_" + stat))
	_press(host, _btn(host, "Tab_" + PartyModel.TAB_GEAR))
	host.answer(true)
	SettingsUiTestSuite.layout(host)
	c["Yes switches and drops the draft"] = pm.tab == PartyModel.TAB_GEAR and not pm.has_draft() and m0.unspent() == 2
	_press(host, _btn(host, "Tab_" + PartyModel.TAB_LEVEL_UP))
	c["no question without a draft"] = not host.router.is_asking() and pm.tab == PartyModel.TAB_LEVEL_UP
	# A member switch with a draft open asks too; the Gear board blocks Q / E.
	_press(host, _btn(host, "Plus_" + stat))
	var ids2: Array[String] = CurrentGame.state().party.member_ids()
	_press(host, _btn(host, "Member_" + ids2[1]))
	c["member switch with a draft asks"] = host.router.is_asking() and pm.selected == ids2[0]
	host.answer(false)
	_view(host)._unhandled_input(_key(KEY_E))
	c["Q / E with a draft asks"] = host.router.is_asking() and pm.selected == ids2[0]
	host.answer(true)
	c["Yes switches the member, no draft"] = pm.selected == ids2[1] and not pm.has_draft()
	var blocker := Control.new()
	host.modal_parent().add_child(blocker)
	_view(host)._unhandled_input(_key(KEY_E))
	c["Q / E ignored while a modal is open"] = pm.selected == ids2[1]
	blocker.free()
	all_ok = _report("UIR-3: level-up table, Revert, discard rule", c) and all_ok

	# Close routes: X and right click, both through the discard question while a draft is open.
	c = {}
	_press(host, _btn(host, "Plus_" + stat))
	_press(host, _btn(host, "CloseButton"))
	c["X with a draft asks"] = host.router.is_asking() and host.router.top() == MenuRouter.PARTY
	host.answer(false)
	host._input(_right_click())
	c["right click with a draft asks"] = host.router.is_asking() and host.router.top() == MenuRouter.PARTY
	host.answer(true)
	c["Yes closes, nothing spent"] = host.router.top() != MenuRouter.PARTY and m0.unspent() == 2
	host.router.push(MenuRouter.PARTY)
	SettingsUiTestSuite.layout(host)
	_press(host, _btn(host, "CloseButton"))
	c["X closes"] = host.router.top() != MenuRouter.PARTY
	host.router.push(MenuRouter.PARTY)
	SettingsUiTestSuite.layout(host)
	host._input(_right_click())
	c["right click closes"] = host.router.top() != MenuRouter.PARTY
	all_ok = _report("UIR-3: X and right click close", c) and all_ok
	_drop_host(f)

	# Confirm spends (after the question) and the table follows.
	f = _char_host(tree_parent, PartyModel.TAB_LEVEL_UP)
	host = f["host"] as MainMenuView
	pm = host.router.model() as PartyModel
	c = {}
	var before: int = DictUtil.to_int(m0.spend.get(stat, 0))
	_press(host, _btn(host, "Plus_" + stat))
	_press(host, _btn(host, "PrimaryButton"))
	c["Confirm asks"] = host.router.is_asking() and host.router.pending_message == LevelUpModel.SPEND_QUESTION
	host.answer(true)
	SettingsUiTestSuite.layout(host)
	c["spent"] = DictUtil.to_int(m0.spend.get(stat, 0)) == before + 1 and m0.unspent() == 1
	all_ok = _report("UIR-3: Confirm in the Level up tab", c) and all_ok
	_drop_host(f)

	# Skills: every row shows its description and only its active rank.
	f = _char_host(tree_parent, PartyModel.TAB_SKILLS)
	host = f["host"] as MainMenuView
	c = {}
	var lo := LoadoutModel.new(m0.id)
	var rows: Array[Dictionary] = []
	for group: String in ["class", "extra"]:
		for raw: Variant in DictUtil.array_of(lo.families()[group]):
			rows.append(DictUtil.dict_of(raw))
	c["fixture has skills"] = not rows.is_empty()
	var text_ok: bool = true
	var rank_ok: bool = true
	var slot_ok: bool = true
	for r: Dictionary in rows:
		var fam: String = str(r["family"])
		var line: Node = host.find_child("Family_" + fam, true, false)
		var desc: Label = host.find_child("Desc_" + fam, true, false) as Label
		text_ok = text_ok and desc != null and desc.text == str(r["description"])
		var shown: String = ""
		if line != null:
			for l: Node in line.find_children("*", "Label", true, false):
				shown += (l as Label).text + "\n"
		for t: Variant in DictUtil.array_of(r["tiers"]):
			var tier: int = DictUtil.to_int(DictUtil.dict_of(t)["tier"])
			var marker: String = "Rank " + SkillCatalog.roman(tier)
			rank_ok = rank_ok and (shown.contains(marker + " ") or shown.contains(marker + "\n")) == (tier == DictUtil.to_int(r["active_tier"]))
		var slot_btn: Button = _btn(host, "Slot_" + fam)
		slot_ok = slot_ok and slot_btn != null and slot_btn.text in ["Slotted", "Add", "Full"]
	# A Skills draft gets the question on a tab switch.
	var pm2: PartyModel = host.router.model() as PartyModel
	var free_row: Button = null
	for n: Node in host.find_children("Slot_*", "Button", true, false):
		if not (n as Button).disabled and free_row == null:
			free_row = n as Button
	var chip: Button = null
	for n: Node in host.find_children("Chips_class", "HFlowContainer", true, false):
		for ch: Node in n.get_children():
			if chip == null and ch is Button:
				chip = ch as Button
	if chip != null:
		_press(host, chip)
		_press(host, _btn(host, "Tab_" + PartyModel.TAB_STATS))
		c["skills draft: tab switch asks"] = pm2.loadout_model().has_draft() and host.router.is_asking() and pm2.tab == PartyModel.TAB_SKILLS
		host.answer(true)
		c["skills draft dropped by Yes"] = pm2.tab == PartyModel.TAB_STATS and not pm2.has_draft()
		_press(host, _btn(host, "Tab_" + PartyModel.TAB_SKILLS))
	else:
		c["skills fixture has a chip"] = false
	c["description per row"] = text_ok
	c["only the active rank"] = rank_ok
	c["slot buttons read Slotted / Add / Full"] = slot_ok
	var default_btn: Button = _btn(host, "DefaultButton")
	c["Use default and Reset in the bar, Confirm primary"] = default_btn != null and _btn(host, "ResetButton") != null and _btn(host, "PrimaryButton").text == "Confirm" and host.find_child("ActionBar", true, false).is_ancestor_of(default_btn)
	all_ok = _report("UIR-3: Skills tab text", c) and all_ok
	_drop_host(f)

	# Gear: the pouch section stays.
	f = _char_host(tree_parent, PartyModel.TAB_GEAR)
	host = f["host"] as MainMenuView
	c = {}
	c["pouch section"] = host.find_child("PouchSection", true, false) != null and host.find_children("PouchSlot*", "HBoxContainer", true, false).size() == UnitState.POUCH_MAX
	var pm3: PartyModel = host.router.model() as PartyModel
	var gm: GearModel = pm3.gear_model()
	var pouch: String = _pouch_item(_first())
	if not pouch.is_empty():
		gm.stash[pouch] = 1
		var _e: Array[String] = gm.pouch_add(pouch)
		host.render()
		SettingsUiTestSuite.layout(host)
		_press(host, _btn(host, "Tab_" + PartyModel.TAB_STATS))
		c["gear draft: tab switch asks"] = host.router.is_asking() and pm3.tab == PartyModel.TAB_GEAR
		host.answer(false)
	else:
		c["gear fixture consumable"] = false
	c["board and Reset in the bar"] = _btn(host, "BoardButton") != null and _btn(host, "ResetButton") != null
	c["no default tag on slots"] = host.find_children("*", "Label", true, false).all(func(n: Node) -> bool: return not (n as Label).text.contains("(default)"))
	all_ok = _report("UIR-3: Gear tab keeps the pouch", c) and all_ok
	_drop_host(f)
	return all_ok

static func _view(host: MainMenuView) -> CharacterView:
	return host.find_children("*", "VBoxContainer", true, false).filter(func(n: Node) -> bool: return n is CharacterView)[0] as CharacterView

# The texts of a grid's children in order ("" for non-labels).
static func _grid_texts(host: Node, grid_name: String) -> Array[String]:
	var out: Array[String] = []
	var grid: Node = host.find_child(grid_name, true, false)
	if grid != null:
		for n: Node in grid.get_children():
			out.append((n as Label).text if n is Label else "")
	return out

# 1280x720 at every text size: each tab keeps its close button, detail pane and action bar on screen, bar buttons apart.
static func _test_character_fit(tree_parent: Node) -> bool:
	var c: Dictionary = {}
	_leave_unspent(_first(), 2)
	var area := Rect2(Vector2.ZERO, VIEW)
	for scale: float in SettingsFile.TEXT_LEVELS:
		SettingsFile.set_value("ui/text_scale", scale)
		for tab: String in PartyModel.TABS:
			var f: Dictionary = _char_host(tree_parent, tab)
			var host: MainMenuView = f["host"] as MainMenuView
			var tag: String = "text %.2f %s" % [scale, tab]
			var bar: Control = host.find_child("ActionBar", true, false) as Control
			var detail: Control = host.find_child("DetailPane", true, false) as Control
			var close: Control = host.find_child("CloseButton", true, false) as Control
			c[tag + ": bar, detail, X on screen"] = bar != null and detail != null and close != null and area.encloses(bar.get_global_rect()) and area.encloses(detail.get_global_rect()) and area.encloses(close.get_global_rect())
			var rects: Array[Rect2] = []
			if bar != null:
				for n: Node in bar.get_children():
					var b: Button = n as Button
					if b != null and b.is_visible_in_tree():
						rects.append(b.get_global_rect())
			var apart: bool = true
			for i: int in range(rects.size()):
				for j: int in range(i + 1, rects.size()):
					apart = apart and not rects[i].intersects(rects[j])
			c[tag + ": bar buttons do not overlap"] = apart
			_drop_host(f)
	SettingsFile.set_value("ui/text_scale", 1.0)
	return _report("UIR-3: every tab fits at four text sizes", c)
