---
class: PartyScreenTestSuite
extends: RefCounted
description: "Tactical RPG - Party Screen Test Suite (UI2-L, S7) Acceptance checks 1-9 of docs/specs/UI2_L.md on the party, stats, level-up, loadout, gear and battle results models (check 10 = the existing suites). Fixture: a NewGame of the shipped Ashlands start party plus one benched level-5 member, held in memory as CurrentGame; saves go to a sandbox; shipped data is only read (items picked by rule)."
---

# Properties
* `TAG`: String = "PartyScreen" (const)
* `BENCH`: String = "ps_bench" (const)
* `VIEW`: Vector2 = Vector2(1280, 720) (const)

# Methods

### `static _report(label: String, checks: Dictionary) -> bool`
```logic
return TestHelpers.report(TAG, label, checks)
```

### `static run(tree_parent: Node = null) -> bool`
```logic
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
CurrentGame.end()
SaveGame.reset_base_dir()
TestHelpers.remove_sandbox(SANDBOX)
return ok
```

### `static _fresh() -> bool`
```logic
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
```

### `static _cap() -> int`
```logic
return WorldCatalog.party_max("ashlands")
```

### `static _first() -> UnitState`
```logic
return CurrentGame.state().party.members[0]
```

### `static _leave_unspent(m: UnitState, points: int) -> void`
```logic
m.spend = {}
if m.unspent() < points:
	m.total_xp = XpCurve.xp_for_level(m.level() + 1)
m.spend = {"DEX": m.unspent() - points}
```

### `static _test_level_up() -> bool`
```logic
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
```

### `static _test_preview() -> bool`
```logic
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
```

### `static _test_auto() -> bool`
```logic
var m: UnitState = _first()
_leave_unspent(m, 3)
var copy: UnitState = m.duplicate_state()
LevelUp.auto_for(copy)
var lm := LevelUpModel.new(m.id)
lm.auto()
var drafted: UnitState = lm.drafted_state()
return _report("Check 4: auto matches LevelUp.auto_for", {"same spend": JsonUtil.deep_equal(drafted.spend, copy.spend), "points used": lm.points_left() == 0})
```

### `static _test_active() -> bool`
```logic
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
```

### `static _test_loadout() -> bool`
```logic
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
```

### `static _test_new_tiers() -> bool`
```logic
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
```

### `static _item_for(m: UnitState, slot: String, usable: bool, skip: String = "") -> String`
```logic
var prof: Proficiency = m.archetype().make_proficiency()
for e: Dictionary in ContentCatalog.entries("items"):
	var id: String = str(e["id"])
	var wt: String = str(e.get("weapon_type", ""))
	if id == skip or id == ContentSchema.NO_ITEM or wt.is_empty() or not ContentFactory.item_fits_slot(e, slot.to_upper(), true):
		continue
	if prof.can_use(wt) == usable:
		return id
return ""
```

### `static _test_gear() -> bool`
```logic
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
```

### `static _test_results() -> bool`
```logic
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
```

### `static _test_review_f2() -> bool`
```logic
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
```

### `static _test_router() -> bool`
```logic
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
```

### `static _pouch_item(m: UnitState, skip: String = "") -> String`
```logic
for e: Dictionary in ContentCatalog.entries("items"):
	var id: String = str(e["id"])
	if id == skip or id == ContentSchema.NO_ITEM or str(e.get("item_type", "")) != "CONSUMABLE":
		continue
	var probe := GearModel.new(m.id)
	probe.stash[id] = 1
	if probe.pouch_candidates().any(func(c: Dictionary) -> bool: return str(c["item"]) == id):
		return id
return ""
```

### `static _gear_host(tree_parent: Node, item: String, stock: int) -> Dictionary`
```logic
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
```

### `static _drop_host(f: Dictionary) -> void`
```logic
var holder: Node = f["holder"] as Node
holder.get_parent().remove_child(holder)
holder.free()
```

### `static _btn(host: Node, node_name: String) -> Button`
```logic
return host.find_child(node_name, true, false) as Button
```

### `static _press(host: MainMenuView, b: Button) -> void`
```logic
b.pressed.emit()
SettingsUiTestSuite.layout(host)
```

### `static _stash_count(m: GearModel, item: String) -> int`
```logic
return DictUtil.to_int(m.stash.get(item, 0))
```

### `static _test_pouch_screen(tree_parent: Node) -> bool`
```logic
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
	var scroll: ScrollContainer = host.find_child("GearScroll", true, false) as ScrollContainer
	var section: Control = host.find_child("PouchSection", true, false) as Control
	c[tag + ": section built"] = body != null and scroll != null and section != null
	if body != null and scroll != null and section != null:
		c[tag + ": no sideways overflow"] = body.get_combined_minimum_size().x <= scroll.size.x + 1.0
		c[tag + ": section as wide as its rows"] = section.get_rect().size.x >= section.get_combined_minimum_size().x - 1.0
	var shown: Array[String] = []
	for n: Node in host.find_children("*", "Button", true, false):
		var b: Button = n as Button
		if b.is_visible_in_tree() and (b.text == "Confirm" or b.text == "Back") and not area.encloses(b.get_global_rect()):
			shown.append(b.text)
	c[tag + ": Confirm and Back inside 1280x720"] = shown.is_empty() and _btn(host, add_name) != null
	_drop_host(f)
SettingsFile.set_value("ui/text_scale", 1.0)
all_ok = _report("ITEMUSE1-U: pouch section fits at every text size", c) and all_ok
return all_ok
```

