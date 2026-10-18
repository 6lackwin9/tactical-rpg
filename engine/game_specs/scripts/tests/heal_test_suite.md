---
class: HealTestSuite
extends: RefCounted
description: "Tactical RPG - Heal test suite (HEAL1-L, spec docs/specs/HEAL1.md section 7 checks 1-8, 10-11, 13-16). Runs in a sandbox world root with two hand-built connected worlds (H and HF, map-less locations): `camp` (start location), `keep` (rest place), `gate`, `lonely` (no route) in H; `outpost` (rest place) in HF. Party and enemy units are picked from the catalog by rule; every expected number is derived from HealRules and the built combatants."
---

# Properties
* `TAG`: String = "Heal" (const)
* `H`: String = "heal_run" (const)
* `HF`: String = "heal_far" (const)
* `SEED`: int = 4242 (const)
* `out`: Array[Dictionary] = []
* `d`: Dictionary = DictUtil.dict_of(e)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING HEAL TEST (HEAL1-L)                     ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
var ok: bool = _test_rules()
ok = _test_autoplay_stop() and ok
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
WorldCatalog.use_root(SANDBOX + "/worlds")
var sandboxed: bool = WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty()
ok = TestHelpers.report(TAG, "sandbox (empty world root under user://)", {"sandboxed": sandboxed}) and ok
if sandboxed and _build():
	ok = _test_battle_hp() and ok
	ok = _test_derived_hp() and ok
	ok = _test_rest_places() and ok
	ok = _test_make_camp() and ok
	ok = _test_defeat() and ok
elif sandboxed:
	ok = false
TestHelpers.cleanup_worlds(SANDBOX)
return ok
```

### `static _q(id: String, world: String = H) -> String`
```logic
return WorldCatalog.qid(world, id)
```

### `static _pick_unit(side: String) -> String`
```logic
for e: Dictionary in ContentCatalog.entries("units"):
	if str(e.get("side", "")) == side and ContentFactory.class_of(e) != null and (side != "party" or str(e.get("stat_mode", "derived")) == "derived"):
		return str(e["id"])
return ""
```

### `static _route(id: String, from: String, to: String) -> Dictionary`
```logic
return {"id": id, "name": id, "from": from, "to": to, "time": 1, "encounter_pct": 0}
```

### `static _build() -> bool`
```logic
_units = {"party": _pick_unit("party"), "enemy": _pick_unit("enemy")}
var errors: Array[String] = []
errors.append_array(WorldCatalog.create_world(H, "Heal Run"))
errors.append_array(WorldCatalog.create_world(HF, "Heal Far"))
errors.append_array(WorldCatalog.upsert(H, "characters", {"id": "hero", "name": "Hero HL", "role": "party", "base_unit": str(_units["party"])}))
errors.append_array(WorldCatalog.upsert(H, "characters", {"id": "ally", "name": "Ally HL", "role": "party", "base_unit": str(_units["party"])}))
errors.append_array(WorldCatalog.upsert(H, "characters", {"id": "foe", "name": "Foe HL", "role": "enemy", "base_unit": str(_units["enemy"])}))
errors.append_array(WorldCatalog.upsert(H, "encounters", {"id": "enc", "name": "Enc", "party": ["hero"], "enemies": ["foe"], "max_rounds": 2}))
errors.append_array(WorldCatalog.upsert(H, "encounters", {"id": "enc_guest", "name": "Enc guest", "party": ["hero"], "enemies": ["foe"], "max_rounds": 2, "guests": ["ally"]}))
for id: String in ["camp", "gate", "lonely"]:
	errors.append_array(WorldCatalog.upsert(H, "locations", {"id": id, "name": id.capitalize()}))
errors.append_array(WorldCatalog.upsert(H, "locations", {"id": "keep", "name": "Keep", "rest": true}))
errors.append_array(WorldCatalog.upsert(HF, "locations", {"id": "outpost", "name": "Outpost", "rest": true}))
errors.append_array(WorldCatalog.update_manifest(H, {"connects": [HF], "start_location": "camp"}))
errors.append_array(WorldCatalog.update_manifest(HF, {"connects": [H]}))
errors.append_array(WorldCatalog.upsert(H, "routes", _route("r_ck", "camp", "keep")))
errors.append_array(WorldCatalog.upsert(H, "routes", _route("r_kg", "keep", "gate")))
errors.append_array(WorldCatalog.upsert(H, "routes", _route("r_go", "gate", "%s:outpost" % HF)))
errors.append_array(WorldCatalog.upsert(HF, "routes", _route("rf_og", "outpost", "%s:gate" % H)))
var problems: Dictionary = WorldCatalog.validate_world(H)
problems.merge(WorldCatalog.validate_world(HF))
if not errors.is_empty() or not problems.is_empty():
	print("      heal fixture errors: %s %s" % [str(errors), str(problems)])
var checks: Dictionary = {"units picked": not str(_units["party"]).is_empty() and not str(_units["enemy"]).is_empty()}
checks["no errors"] = errors.is_empty()
checks["worlds validate"] = problems.is_empty()
checks["only keep and outpost rest"] = HealRules.is_rest_place(_new_gs(), _q("keep")) and HealRules.is_rest_place(_new_gs(), _q("outpost", HF)) and not HealRules.is_rest_place(_new_gs(), _q("gate"))
return TestHelpers.report(TAG, "fixtures (worlds H and HF, rest places keep and outpost, units picked by rule)", checks)
```

### `static _new_gs() -> GameState`
```logic
var uid: String = str(_units["party"])
var gs: GameState = GameState.of(NewGame.create(H, [uid, uid, uid] as Array[String], SEED)["state"])
if gs != null:
	gs.party.active_ids = [gs.party.members[0].id, gs.party.members[1].id] as Array[String]
return gs
```

### `static _at(loc: String) -> GameState`
```logic
var gs: GameState = _new_gs()
var _e: Dictionary = ExploreRunner.enter(gs, loc)
return gs
```

### `static _hurt(gs: GameState, lost: int) -> void`
```logic
for m: UnitState in gs.party.members:
	m.hp_lost = lost
```

### `static _nobody_hurt(gs: GameState) -> bool`
```logic
return not HealRules.hurt(gs.party)
```

### `static _encounter(gs: GameState, local: String) -> GameEncounter`
```logic
var raw: Variant = BattleBridge.encounter_for(gs, _q(local)).get("encounter", null)
return raw if raw is GameEncounter else null
```

### `static _test_rules() -> bool`
```logic
var c: Dictionary = {}
var rally: int = HealRules.rally(200)
c["rally is RALLY_PCT of max, rounded up"] = rally == ceili(200 * HealRules.RALLY_PCT / 100.0) and HealRules.rally(201) == ceili(201 * HealRules.RALLY_PCT / 100.0)
c["won at 80 of 200 -> 80 + rally (check 1)"] = HealRules.hp_after(80, 200, "won") == 80 + rally
c["won never above max"] = HealRules.hp_after(199, 200, "won") == 200
c["downed in a win gets up at the rally (check 2)"] = HealRules.hp_after(0, 200, "won") == rally
c["downed in a flight gets up at 1"] = HealRules.hp_after(0, 200, "fled") == 1
c["standing in a flight keeps its HP"] = HealRules.hp_after(80, 200, "fled") == 80
c["lost returns max"] = HealRules.hp_after(0, 200, "lost") == 200
c["start HP follows a higher max (check 7)"] = HealRules.start_hp(210, 70) == 140
c["start HP never below 1 (check 7)"] = HealRules.start_hp(60, 70) == 1
c["start HP full at hp_lost 0"] = HealRules.start_hp(200, 0) == 200
c["CAMP_COST 3, RALLY_PCT 25"] = HealRules.CAMP_COST == 3 and HealRules.RALLY_PCT == 25
return TestHelpers.report(TAG, "rules: rally, flight, defeat and start HP", c)
```

### `static _test_autoplay_stop() -> bool`
```logic
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
var hero: Combatant = TestHelpers.unit("Low", grid, Vector2i(0, 0), 200)
hero.hp = floori(hero.max_hp * 0.2)
var start: Dictionary = {hero: hero.hp}
var heroes: Array[Combatant] = [hero]
var c: Dictionary = {"starts below the safety line": hero.hp * 100 <= hero.max_hp * CombatAutoplay.SAFETY_HP_PCT}
c["no stop at the start HP"] = CombatAutoplay.safety_reason(heroes, start).is_empty()
hero.hp = maxi(hero.hp - 1, 1)
c["no stop on a further loss"] = CombatAutoplay.safety_reason(heroes, start).is_empty()
var _d: int = hero.take_damage(hero.hp, true)
c["stops when downed"] = not CombatAutoplay.safety_reason(heroes, start).is_empty()
return TestHelpers.report(TAG, "autoplay safety stop on a hero starting at 20% HP (check 11)", c)
```

### `static _test_battle_hp() -> bool`
```logic
var c: Dictionary = {}
var gs: GameState = _at(_q("gate"))
var a: UnitState = gs.party.members[0]
var b: UnitState = gs.party.members[1]
var reserve: UnitState = gs.party.members[2]
a.hp_lost = 30
reserve.hp_lost = 11
var enc: GameEncounter = _encounter(gs, "enc")
c["encounter built"] = enc != null
if enc == null:
	return TestHelpers.report(TAG, "stored HP through a battle", c)
var session := CombatSession.new(enc)
session.start()
var heroes: Array[Combatant] = enc.heroes()
c["two heroes fight (reserve stays out)"] = heroes.size() == 2
if heroes.size() != 2:
	session.dispose()
	return TestHelpers.report(TAG, "stored HP through a battle", c)
var ha: Combatant = heroes[0] if enc.hero_key(heroes[0]) == a.id else heroes[1]
var hb: Combatant = heroes[1] if ha == heroes[0] else heroes[0]
var max_a: int = ha.max_hp
var max_b: int = hb.max_hp
c["a starts at max - hp_lost"] = ha.hp == HealRules.start_hp(max_a, 30)
c["b starts full"] = hb.hp == max_b
var pb: Combatant = PartyBuilder.build(a.duplicate_state(), false)
c["PartyBuilder still builds full HP"] = pb.hp == pb.max_hp
var end_a: int = maxi(floori(max_a * 0.4), 1)
ha.hp = end_a
var _d: int = hb.take_damage(hb.hp, true)
enc.write_back_hp("won")
var want_a: int = max_a - HealRules.hp_after(end_a, max_a, "won")
var want_b: int = max_b - HealRules.rally(max_b)
c["won: standing hero rallies (check 1)"] = a.hp_lost == want_a
c["won: downed hero stored at the rally (check 2)"] = b.hp_lost == want_b
c["reserve never written (check 4)"] = reserve.hp_lost == 11
ha.hp = HealRules.start_hp(max_a, 30)   # a results Replay shows round 1 at the start HP ...
enc.write_back_hp("won")
c["second write-back changes nothing (check 10)"] = a.hp_lost == want_a and b.hp_lost == want_b
session.dispose()
var next: GameEncounter = _encounter(gs, "enc")
if next != null:
	var s2 := CombatSession.new(next)
	s2.start()
	for h: Combatant in next.heroes():
		var m: UnitState = gs.party.member(next.hero_key(h))
		c["next battle starts %s at the stored HP" % h.name] = m != null and h.hp == HealRules.start_hp(h.max_hp, m.hp_lost)
	var h0: Combatant = next.heroes()[0]
	var m0: UnitState = gs.party.member(next.hero_key(h0))
	var _d2: int = h0.take_damage(h0.hp, true)
	var before: int = m0.hp_lost
	next.write_back_hp("lost")
	c["lost writes nothing"] = m0.hp_lost == before
	s2.dispose()
var fled_gs: GameState = _at(_q("gate"))
var fled: GameEncounter = _encounter(fled_gs, "enc")
if fled != null:
	var s3 := CombatSession.new(fled)
	s3.start()
	var f0: Combatant = fled.heroes()[0]
	var f1: Combatant = fled.heroes()[1]
	var keep_hp: int = maxi(floori(f0.max_hp * 0.4), 1)
	f0.hp = keep_hp
	var _d3: int = f1.take_damage(f1.hp, true)
	fled.write_back_hp("fled")
	c["fled: standing hero keeps its HP (check 2)"] = fled_gs.party.member(fled.hero_key(f0)).hp_lost == f0.max_hp - keep_hp
	c["fled: downed hero gets up at 1 (check 2)"] = fled_gs.party.member(fled.hero_key(f1)).hp_lost == f1.max_hp - 1
	s3.dispose()
var g_gs: GameState = _at(_q("gate"))
var guest_enc: GameEncounter = _encounter(g_gs, "enc_guest")
if guest_enc != null:
	var s4 := CombatSession.new(guest_enc)
	s4.start()
	var size_before: int = g_gs.party.members.size()
	for g: Combatant in guest_enc.guests:
		var _d4: int = g.take_damage(g.hp, true)
	guest_enc.write_back_hp("won")
	c["guest present"] = not guest_enc.guests.is_empty()
	c["guest never written (check 4)"] = g_gs.party.members.size() == size_before and _nobody_hurt(g_gs)
	s4.dispose()
return TestHelpers.report(TAG, "stored HP through a battle (checks 1-4, 10)", c)
```

### `static _test_derived_hp() -> bool`
```logic
var c: Dictionary = {}
var gs: GameState = _new_gs()
var m: UnitState = gs.party.members[0]
m.hp_lost = 20
var max_before: int = HealRules.max_hp_of(m)
c["current_hp = start_hp of the derived max"] = JsonUtil.deep_equal(HealRules.current_hp(m), {"hp": HealRules.start_hp(max_before, 20), "max": max_before})
m.total_xp = XpCurve.xp_for_level(m.level() + 3)
var max_after: int = HealRules.max_hp_of(m)
c["level-up keeps hp_lost; HP follows the new max (check 7)"] = m.hp_lost == 20 and DictUtil.to_int(HealRules.current_hp(m)["hp"]) == clampi(max_after - 20, 1, max_after)
m.hp_lost = max_after + 50
c["a wound above max starts at 1 (check 7)"] = DictUtil.to_int(HealRules.current_hp(m)["hp"]) == 1
_hurt(gs, 40)
var hero: UnitState = NewGame.hero_from_character(H, "ally")
c["recruit joins with hp_lost 0 (check 8)"] = hero != null and hero.hp_lost == 0
c.merge(_unspent_hero_checks())
return TestHelpers.report(TAG, "derived HP, level-up, recruit (checks 7-8)", c)
```

### `static _unspent_hero_checks() -> Dictionary`
```logic
var c: Dictionary = {}
var gs: GameState = _at(_q("gate"))
var m: UnitState = gs.party.members[0]
m.total_xp = XpCurve.xp_for_level(m.level() + 2)
m.hp_lost = 25
c["unspent: fixture hero holds unspent points"] = m.unspent() > 0
var enc: GameEncounter = _encounter(gs, "enc")
if enc == null:
	c["unspent: encounter built"] = false
	return c
var session := CombatSession.new(enc)
session.start()
var hero: Combatant = null
for h: Combatant in enc.heroes():
	if enc.hero_key(h) == m.id:
		hero = h
c["unspent: hero fights"] = hero != null
if hero != null:
	var read: Dictionary = HealRules.current_hp(m)
	c["unspent: read-out max = battle max"] = DictUtil.to_int(read["max"]) == hero.max_hp and HealRules.max_hp_of(m) == hero.max_hp
	c["unspent: read-out HP = battle start HP"] = DictUtil.to_int(read["hp"]) == hero.hp and hero.hp == HealRules.start_hp(hero.max_hp, 25)
	enc.write_back_hp("fled")
	c["unspent: untouched wound round-trips"] = m.hp_lost == 25
	var restored: int = HealRules.rest(gs.party)
	c["unspent: rest restores exactly the wound"] = restored == mini(25, hero.max_hp - 1) and m.hp_lost == 0
session.dispose()
return c
```

### `static _test_rest_places() -> bool`
```logic
var c: Dictionary = {}
var gs: GameState = _at(_q("gate"))
c["start: last_rest empty"] = str(gs.explore["last_rest"]).is_empty()
_hurt(gs, 25)
var at_gate: Dictionary = ExploreRunner.enter(gs, _q("gate"))
c["entering a plain location changes nothing (check 5)"] = DictUtil.to_bool(at_gate["ok"]) and gs.party.members[2].hp_lost == 25 and _events_of(at_gate, "rested").is_empty() and str(gs.explore["last_rest"]).is_empty()
var at_keep: Dictionary = ExploreRunner.enter(gs, _q("keep"))
var rested: Array[Dictionary] = _events_of(at_keep, "rested")
c["hurt party rests at the keep, reserve too (check 5)"] = _nobody_hurt(gs) and rested.size() == 1 and str(rested[0].get("location", "")) == _q("keep") and DictUtil.to_int(rested[0].get("restored", 0)) > 0
c["last_rest = keep (check 15)"] = str(gs.explore["last_rest"]) == _q("keep")
c["state stays valid"] = gs.validate().is_empty()
var again: Dictionary = ExploreRunner.enter(gs, _q("keep"))
c["nobody hurt: no rested event (check 5)"] = _events_of(again, "rested").is_empty()
var fresh: GameState = _at(_q("keep"))
c["last_rest set with nobody hurt (check 15)"] = str(fresh.explore["last_rest"]) == _q("keep")
var _g: Dictionary = ExploreRunner.enter(fresh, _q("gate"))
c["entering the gate keeps last_rest (check 15)"] = str(fresh.explore["last_rest"]) == _q("keep")
return TestHelpers.report(TAG, "rest places (checks 5, 15)", c)
```

### `static _test_make_camp() -> bool`
```logic
var c: Dictionary = {}
var gs: GameState = _at(_q("gate"))
_hurt(gs, 15)
gs.party.supplies = HealRules.CAMP_COST + 2
var time_before: int = gs.time
var rolls_before: Dictionary = gs.rolls.duplicate(true)
c["can camp"] = HealRules.can_camp(gs).is_empty()
var res: Dictionary = HealRules.make_camp(gs)
c["camp spends CAMP_COST and rests everyone"] = DictUtil.to_bool(res["ok"]) and DictUtil.to_int(res["spent"]) == HealRules.CAMP_COST and gs.party.supplies == 2 and _nobody_hurt(gs)
c["camp advances no clock and rolls nothing"] = gs.time == time_before and JsonUtil.deep_equal(gs.rolls, rolls_before)
c["camp autosaves"] = _events_of(res, "autosave").size() == 1 and _events_of(res, "camped").size() == 1
var rested_snap: String = JSON.stringify(gs.to_dict(), "", true)
var no_need: Dictionary = HealRules.make_camp(gs)
c["nobody hurt: refused, unchanged"] = not DictUtil.to_bool(no_need["ok"]) and JsonUtil.strings(no_need["errors"]) == ([HealRules.REFUSE_RESTED] as Array[String]) and JSON.stringify(gs.to_dict(), "", true) == rested_snap
_hurt(gs, 15)
var short_snap: String = JSON.stringify(gs.to_dict(), "", true)
var short: Dictionary = HealRules.make_camp(gs)
c["2 supplies: refused, unchanged"] = not DictUtil.to_bool(short["ok"]) and JsonUtil.strings(short["errors"]) == ([HealRules.REFUSE_SUPPLIES] as Array[String]) and JSON.stringify(gs.to_dict(), "", true) == short_snap
var off: GameState = _at(_q("lonely"))
_hurt(off, 15)
off.party.supplies = 10
c["not at a world map node: refused"] = HealRules.can_camp(off) == HealRules.REFUSE_NOT_AT_NODE and not DictUtil.to_bool(HealRules.make_camp(off)["ok"]) and off.party.supplies == 10
return TestHelpers.report(TAG, "make camp (check 6)", c)
```

### `static _defeat_of(res: Dictionary) -> Dictionary`
```logic
var list: Array[Dictionary] = _events_of(res, "defeat")
return list[0] if list.size() == 1 else {}
```

### `static _keeps(gs: GameState, before: Dictionary) -> bool`
```logic
var now: Dictionary = gs.to_dict()
var ok: bool = true
for key: String in ["flags", "quests", "loot_claimed", "shops", "time"]:
	ok = ok and JsonUtil.deep_equal(now[key], before[key])
var p_now: Dictionary = DictUtil.dict_of(now["party"])
var p_before: Dictionary = DictUtil.dict_of(before["party"])
ok = ok and JsonUtil.deep_equal(p_now["gold"], p_before["gold"]) and JsonUtil.deep_equal(p_now["stash"], p_before["stash"])
for i: int in range(gs.party.members.size()):
	var m_before: Dictionary = DictUtil.dict_of(DictUtil.array_of(p_before["members"])[i])
	ok = ok and gs.party.members[i].total_xp == DictUtil.to_int(m_before["total_xp"])
return ok
```

### `static _test_defeat() -> bool`
```logic
var c: Dictionary = {}
var enc_ref: Dictionary = {"encounter": _q("enc")}
# Forced rest: a plain checkpoint with CAMP_COST + 2 supplies.
var gs: GameState = _at(_q("gate"))
_hurt(gs, 30)
gs.party.supplies = HealRules.CAMP_COST + 2
var before: Dictionary = gs.to_dict()
var lost: Dictionary = ExploreRunner.after_fight(gs, enc_ref, "lost")
var d: Dictionary = _defeat_of(lost)
c["map fight: forced_rest, -CAMP_COST, wake at the gate, nobody hurt (checks 3, 13)"] = str(d.get("mode", "")) == HealRules.MODE_FORCED_REST and DictUtil.to_int(d.get("spent", -1)) == HealRules.CAMP_COST and gs.party.supplies == 2 and gs.location_id == _q("gate") and str(d.get("location", "")) == _q("gate") and _nobody_hurt(gs)
c["defeat keeps XP, gold, items, flags, quests (check 16)"] = _keeps(gs, before)
# Back to town: fewer supplies after visiting the keep.
var town: GameState = _at(_q("keep"))
var _g: Dictionary = ExploreRunner.enter(town, _q("gate"))
_hurt(town, 30)
town.party.supplies = HealRules.CAMP_COST - 1
var short: Dictionary = DialogueRunner.after_fight(town, enc_ref, "lost")
var ds: Dictionary = _defeat_of(short)
c["dialogue fight: back_to_town keeps supplies, wakes at last_rest (checks 13, 16)"] = str(ds.get("mode", "")) == HealRules.MODE_BACK_TO_TOWN and DictUtil.to_int(ds.get("spent", -1)) == 0 and town.party.supplies == HealRules.CAMP_COST - 1 and town.location_id == _q("keep") and _nobody_hurt(town)
# Back to town with no rest place ever entered: the start location.
var none: GameState = _at(_q("gate"))
_hurt(none, 30)
none.party.supplies = 0
var to_start: Dictionary = ExploreRunner.after_fight(none, enc_ref, "lost")
c["0 supplies, last_rest empty: start location (check 13)"] = str(_defeat_of(to_start).get("mode", "")) == HealRules.MODE_BACK_TO_TOWN and none.location_id == MapRules.start_location(H) and none.location_id == _q("camp") and _nobody_hurt(none)
var stale: GameState = _at(_q("gate"))
stale.explore["last_rest"] = _q("gone")
stale.party.supplies = 0
var _s: Dictionary = ExploreRunner.after_fight(stale, enc_ref, "lost")
c["unresolved last_rest reads as the start location"] = stale.location_id == _q("camp")
# Road fight whose wake target is a rest place: free.
var road: GameState = _at(_q("gate"))
_hurt(road, 30)
road.party.supplies = HealRules.CAMP_COST + 2
var rest_res: Dictionary = TravelRunner.after_fight(road, {"encounter": _q("enc"), "from": _q("keep")}, "lost")
var dr: Dictionary = _defeat_of(rest_res)
c["road fight from the keep: rest_place, nothing spent (checks 14, 16)"] = str(dr.get("mode", "")) == HealRules.MODE_REST_PLACE and DictUtil.to_int(dr.get("spent", -1)) == 0 and road.party.supplies == HealRules.CAMP_COST + 2 and road.location_id == _q("keep") and _nobody_hurt(road)
# Another world's rest place as last_rest (a qualified id across `connects`).
var far: GameState = _at(_q("gate"))
var _o: Dictionary = ExploreRunner.enter(far, _q("outpost", HF))
var _b: Dictionary = ExploreRunner.enter(far, _q("gate"))
_hurt(far, 30)
far.party.supplies = 1
var far_res: Dictionary = TravelRunner.after_fight(far, {"encounter": _q("enc"), "from": _q("gate")}, "lost")
c["last_rest in a connected world: back there (check 14)"] = str(far.explore["last_rest"]) == _q("outpost", HF) and str(_defeat_of(far_res).get("mode", "")) == HealRules.MODE_BACK_TO_TOWN and far.location_id == _q("outpost", HF) and far.party.supplies == 1 and _nobody_hurt(far)
# A wake whose target can't be entered (stale last_safe) is refused and keeps the HP and the supplies.
var broken: GameState = _at(_q("gate"))
_hurt(broken, 30)
broken.party.supplies = HealRules.CAMP_COST + 2
broken.explore["last_safe"] = _q("gone")
var refused: Dictionary = ExploreRunner.wake(broken, _q("nowhere"))
c["stale last_safe: wake refused, HP and supplies kept"] = not DictUtil.to_bool(refused["ok"]) and broken.party.supplies == HealRules.CAMP_COST + 2 and broken.party.members[0].hp_lost == 30 and broken.location_id == _q("gate")
# Exactly CAMP_COST supplies: all spent.
var exact: GameState = _at(_q("gate"))
exact.party.supplies = HealRules.CAMP_COST
var _x: Dictionary = ExploreRunner.after_fight(exact, enc_ref, "lost")
c["exactly CAMP_COST supplies: all spent"] = exact.party.supplies == 0 and exact.location_id == _q("gate")
# Dev outcome buttons: won / fled without a battle leave the stored HP alone.
var dev: GameState = _at(_q("gate"))
_hurt(dev, 30)
var _w: Dictionary = ExploreRunner.after_fight(dev, enc_ref, "won")
var _f: Dictionary = TravelRunner.after_fight(dev, {"encounter": _q("enc"), "from": _q("gate")}, "fled")
c["dev won / fled change no HP (check 3)"] = dev.party.members[0].hp_lost == 30 and dev.party.members[2].hp_lost == 30
return TestHelpers.report(TAG, "defeat cost through every wake caller (checks 3, 13, 14, 16)", c)
```

