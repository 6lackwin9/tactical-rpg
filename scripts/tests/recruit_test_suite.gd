class_name RecruitTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Recruit Test Suite (RECRUIT1-L step 2): the runtime party cap (PartyState.max_active from
# WorldCatalog.party_max, set by GameState.create / from_dict, trimming a longer active_ids), heroes from world
# characters (NewGame.hero_from_character, NewGame.create), names (PartyBuilder.display_name, Combatant.name, the party
# and new-game models), the dialogue effect `recruit` (precheck notes, apply XP rule, active_ids append, join note,
# hidden choice) and old four-hero saves. Fixtures: hand-built worlds in an EMPTY sandbox world root; catalog units are
# only read (picked by rule); every expectation is derived from the rules (XpCurve, the units' authored levels).
# ==============================================================================

const TAG: String = "Recruit"
static var SANDBOX: String = TestHelpers.sandbox("recruit")
const W: String = "rc_base"      # party_max 3, start_party [hero_a] (a character)
const KID: String = "rc_kid"     # extends rc_base: inherits start_party and party_max
const OPEN: String = "rc_open"   # no party_max (resolves to PartyState.PARTY_MAX), catalog-unit start party
const CAP: int = 3
const SEED: int = 11

static var _u: String = ""         # the start hero's base unit (party side, class-derived)
static var _v: String = ""         # the recruits' base unit (may equal _u when only one party unit exists)
static var _passive: String = ""   # granted by the fixture faction

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _party_units() -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and str(e.get("stat_mode", "derived")) == "derived":
			out.append(str(e["id"]))
	return out

static func _level_of(unit_id: String) -> int:
	return DictUtil.to_int(ContentCatalog.get_entry("units", unit_id).get("level", 1), 1)

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING RECRUIT TEST (RECRUIT1-L)               ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	var ok: bool = _test_shipped()
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var sandboxed: bool = WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty() and SaveGame.base_dir().begins_with("user://")
	ok = _report("Sandbox (empty world root and saves under user://)", {"sandboxed": sandboxed}) and ok
	var units: Array[String] = _party_units()
	var passives: Array[String] = ContentCatalog.ids("passives")
	ok = _report("Content fixtures picked by rule", {"a party unit": not units.is_empty(), "a passive": not passives.is_empty()}) and ok
	if sandboxed and not units.is_empty() and not passives.is_empty():
		_u = units[0]
		_v = units[1] if units.size() > 1 else units[0]
		_passive = passives[0]
		if _build_worlds():
			ok = _test_cap() and ok
			ok = _test_new_game() and ok
			ok = _test_names() and ok
			ok = _test_recruit() and ok
			ok = _test_full() and ok
			ok = _test_old_save() and ok
		else:
			ok = false
	else:
		ok = false
	CurrentGame.end()
	SaveGame.reset_base_dir()
	TestHelpers.cleanup_worlds(SANDBOX)
	return ok

# Shipped worlds keep validating; every playable shipped world's new game respects its resolved party_max.
static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["shipped worlds validate"] = WorldCatalog.validate_all().is_empty()
	var capped: bool = true
	for w: String in NewGame.playable_worlds():
		var gs: GameState = GameState.of(NewGame.create_for_world(w, SEED)["state"])
		capped = capped and gs != null and gs.party.max_active == WorldCatalog.party_max(w) and gs.party.active_members().size() <= gs.party.max_active and gs.validate().is_empty()
	checks["new games take the world's party_max"] = capped
	return _report("Shipped worlds: validate, new games capped by party_max", checks)

static func _build_worlds() -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.create_world(W, "RC Base"))
	errors.append_array(WorldCatalog.create_world(KID, "RC Kid", [W]))
	errors.append_array(WorldCatalog.create_world(OPEN, "RC Open"))
	errors.append_array(WorldCatalog.upsert(W, "locations", {"id": "keep", "name": "Keep"}))
	errors.append_array(WorldCatalog.upsert(OPEN, "locations", {"id": "yard", "name": "Yard"}))
	errors.append_array(WorldCatalog.upsert(W, "factions", {"id": "wardens", "name": "Wardens", "grants": [_passive]}))
	errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "hero_a", "name": "Hero Alpha", "role": "party", "base_unit": _u, "faction": "wardens"}))
	for pair: Array in [["rec_b", "Recruit Bravo"], ["rec_c", "Recruit Charlie"], ["rec_d", "Recruit Delta"]]:
		errors.append_array(WorldCatalog.upsert(W, "characters", {"id": pair[0], "name": pair[1], "role": "party", "base_unit": _v}))
	errors.append_array(WorldCatalog.upsert(W, "characters", {"id": "sergeant", "name": "Sergeant", "base_unit": _u, "dialogue": "d_muster"}))
	errors.append_array(WorldCatalog.upsert(W, "dialogues", {"id": "d_muster", "name": "Muster", "start": "hello", "services": false, "nodes": [
		{"id": "hello", "text": "Pick one.", "choices": [
			{"text": "Take Bravo", "to": "hello", "effects": [{"effect": "add_flag", "key": "rc.recruits", "delta": 1}, {"effect": "recruit", "character": "rec_b"}]},
			{"text": "Take Charlie", "to": "hello", "effects": [{"effect": "add_flag", "key": "rc.recruits", "delta": 1}, {"effect": "recruit", "character": "rec_c"}]},
			{"text": "Take Delta", "to": "hello", "effects": [{"effect": "add_flag", "key": "rc.recruits", "delta": 1}, {"effect": "recruit", "character": "rec_d"}]},
			{"text": "Farewell", "to": ""},
		]},
	]}))
	errors.append_array(WorldCatalog.update_manifest(W, {"start_location": "keep", "start_party": ["hero_a"], "party_max": CAP}))
	errors.append_array(WorldCatalog.update_manifest(OPEN, {"start_location": "yard", "start_party": [_u]}))
	var problems: Dictionary = {}
	for w: String in [W, KID, OPEN]:
		problems.merge(WorldCatalog.validate_world(w))
	if not errors.is_empty() or not problems.is_empty():
		print("      [%s] fixture problems: %s %s" % [TAG, str(errors), str(problems)])
	return _report("Fixture worlds (capped base with a character start party, inheriting child, open world)", {"stored": errors.is_empty(), "valid": problems.is_empty()})

# Four members of base unit _u (ids m1..m4), the first `active` of them listed in active_ids.
static func _four(active: int) -> PartyState:
	var p := PartyState.new()
	for i: int in range(4):
		p.add_member(UnitState.at_level(_u, _level_of(_u), "m%d" % (i + 1)))
	for i: int in range(active):
		p.active_ids.append("m%d" % (i + 1))
	return p

# 1.3: max_active defaults to PARTY_MAX, apply_cap trims, active_members / validate use it; create and from_dict set it.
static func _test_cap() -> bool:
	var checks: Dictionary = {}
	checks["resolver"] = WorldCatalog.party_max(W) == CAP and WorldCatalog.party_max(KID) == CAP and WorldCatalog.party_max(OPEN) == PartyState.PARTY_MAX
	checks["kid inherits start_party"] = WorldCatalog.start_party(KID) == ["hero_a"]
	checks["default max_active"] = PartyState.new().max_active == PartyState.PARTY_MAX
	var p: PartyState = _four(0)
	p.apply_cap(CAP)
	checks["implicit active = first max_active"] = p.active_members().size() == CAP and p.active_members()[0].id == "m1" and p.validate().is_empty()
	var listed: PartyState = _four(4)
	checks["four listed under PARTY_MAX valid"] = listed.validate().is_empty()
	listed.max_active = CAP
	checks["validate uses max_active"] = listed.validate() == ["At most %d active members (got %d)." % [CAP, 4]]
	listed.apply_cap(CAP)
	checks["apply_cap keeps the first ids"] = listed.active_ids == ["m1", "m2", "m3"] and listed.members.size() == 4 and listed.validate().is_empty()
	checks["create sets the cap"] = GameState.create(W, _four(0)).party.max_active == CAP and GameState.create(OPEN, _four(0)).party.max_active == PartyState.PARTY_MAX
	var d: Dictionary = GameState.create(OPEN, _four(4)).to_dict()
	checks["max_active not saved"] = not DictUtil.dict_of(d["party"]).has("max_active")
	d["world_id"] = W
	var loaded: GameState = GameState.from_dict(d)
	checks["from_dict trims a longer active_ids"] = loaded.party.max_active == CAP and loaded.party.active_ids == ["m1", "m2", "m3"] and loaded.party.members.size() == 4
	return _report("1.3: runtime party cap (max_active, apply_cap, create / from_dict, not saved)", checks)

# Rules 1 and 3: a new game from a character start party; the child world inherits it.
static func _test_new_game() -> bool:
	var checks: Dictionary = {}
	var res: Dictionary = NewGame.create_for_world(W, SEED)
	var gs: GameState = GameState.of(res["state"])
	checks["created"] = gs != null and gs.validate().is_empty()
	if gs == null:
		print("      [%s] new game: %s" % [TAG, str(res["errors"])])
		return _report("Rules 1 and 3: new game with a character hero", checks)
	var hero: UnitState = gs.party.member("hero_a")
	var c: Dictionary = WorldCatalog.resolve(W, "characters", "hero_a")
	checks["one member, the character id"] = gs.party.member_ids() == ["hero_a"]
	checks["base unit and lore_ref"] = hero != null and hero.base_unit == _u and hero.lore_ref == WorldCatalog.qid(W, "hero_a")
	checks["base unit's level"] = hero != null and hero.total_xp == XpCurve.xp_for_level(_level_of(_u))
	checks["passives: own + faction grants"] = hero != null and hero.passives == WorldFactory.character_passives(c, W) and hero.passives.has(_passive)
	checks["empty spend, default gear"] = hero != null and hero.spend.is_empty() and hero.equipment.is_empty() and hero.carried.is_empty()
	checks["max_active = party_max"] = gs.party.max_active == CAP
	var kid: GameState = GameState.of(NewGame.create_for_world(KID, SEED)["state"])
	checks["child inherits the hero and the cap"] = kid != null and kid.party.member_ids() == ["hero_a"] and kid.party.max_active == CAP
	var mixed: GameState = GameState.of(NewGame.create(W, [_u, "hero_a"] as Array[String], SEED)["state"])
	checks["mixed unit and character"] = mixed != null and mixed.party.member_ids() == [_u, "hero_a"] and mixed.party.member(_u).lore_ref.is_empty()
	checks["unknown entry refused"] = JsonUtil.strings(NewGame.create(W, ["rc_nobody"] as Array[String], SEED)["errors"]) == ["Unknown unit 'rc_nobody'."]
	checks["hero_from_character refuses an npc"] = NewGame.hero_from_character(W, "sergeant") == null
	return _report("Rules 1 and 3: new game with a character hero (id, base unit, lore_ref, level, passives)", checks)

# Rule 2: names on the Combatant, display_name, the party, results and new-game models.
static func _test_names() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = GameState.of(NewGame.create(W, [_u, "hero_a"] as Array[String], SEED)["state"])
	if gs == null:
		return _report("Rule 2: names", {"state": false})
	var hero: UnitState = gs.party.member("hero_a")
	var plain: UnitState = gs.party.member(_u)
	var unit_name: String = str(ContentCatalog.get_entry("units", _u).get("name", _u))
	checks["display_name: character name"] = PartyBuilder.display_name(hero) == "Hero Alpha"
	checks["display_name: unit name without lore_ref"] = PartyBuilder.display_name(plain) == unit_name
	var built: Combatant = PartyBuilder.build(hero)
	checks["Combatant.name"] = built != null and built.name == "Hero Alpha" and built.lore_ref == hero.lore_ref
	var plain_c: Combatant = PartyBuilder.build(plain)
	checks["plain Combatant keeps the unit name"] = plain_c != null and plain_c.name == unit_name
	var stale: UnitState = hero.duplicate_state()
	stale.lore_ref = "rc_nowhere:ghost"
	checks["unresolved lore_ref falls back"] = PartyBuilder.display_name(stale) == unit_name
	CurrentGame.start(gs)
	var pm := PartyModel.new(null)
	var names: Array[String] = []
	for row: Dictionary in pm.members():
		names.append(str(row["name"]))
	checks["party model names"] = names == [unit_name, "Hero Alpha"]
	checks["party model cap"] = pm.max_active() == CAP
	CurrentGame.end()
	checks["results / level-up name helper"] = PartyPreview.name_of(hero) == "Hero Alpha"
	var ng := NewGameModel.new(null)
	var shown: Array[String] = []
	for wrow: Dictionary in ng.worlds():
		if str(wrow["id"]) == W:
			for prow: Variant in DictUtil.array_of(wrow["party"]):
				var pd: Dictionary = DictUtil.dict_of(prow)
				shown.append("%s/%d" % [str(pd["name"]), DictUtil.to_int(pd["level"])])
	checks["new-game model shows the hero name"] = shown == ["Hero Alpha/%d" % _level_of(_u)]
	return _report("Rule 2: names (display_name, Combatant.name, party, results and new-game models)", checks)

static func _find(v: Dictionary, needle: String) -> int:
	var choices: Array = DictUtil.array_of(v.get("choices", []))
	for i: int in range(choices.size()):
		if str(DictUtil.dict_of(choices[i]).get("text", "")).contains(needle):
			return i
	return -1

# Rules 4-6: recruit joins at the warrior's XP, appends to an explicit active_ids, notes the join, hides itself.
static func _test_recruit() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = GameState.of(NewGame.create_for_world(W, SEED)["state"])
	if gs == null:
		return _report("Rules 4-6: recruit", {"state": false})
	var hero: UnitState = gs.party.member("hero_a")
	hero.total_xp = maxi(XpCurve.xp_for_level(_level_of(_v) + 1), hero.total_xp + 1)   # the warrior has outgrown the recruits' level
	var warrior_xp: int = hero.total_xp
	var r: DialogueRunner = DialogueRunner.open(gs, "sergeant")
	checks["opened"] = r.errors.is_empty()
	var res: Dictionary = r.choose(_find(r.view(), "Take Bravo"))
	var bravo: UnitState = gs.party.member("rec_b")
	checks["joined"] = DictUtil.to_bool(res["ok"]) and bravo != null and gs.party.member_ids() == ["hero_a", "rec_b"]
	checks["join note"] = JsonUtil.strings(res["notes"]).has("Recruit Bravo joins the party.")
	checks["joins at the warrior's XP"] = bravo != null and bravo.total_xp == warrior_xp
	checks["hero rule 1"] = bravo != null and bravo.base_unit == _v and bravo.lore_ref == WorldCatalog.qid(W, "rec_b") and bravo.spend.is_empty()
	checks["implicit active_ids stay implicit"] = gs.party.active_ids.is_empty() and gs.party.active_members().size() == 2
	checks["choice hidden once a member"] = _find(r.view(), "Take Bravo") == -1 and _find(r.view(), "Take Charlie") >= 0
	checks["other effects applied"] = gs.get_flag("rc.recruits", 0) == 1
	checks["already a member refused"] = r._precheck([{"effect": "recruit", "character": "rec_b"}]) == ["Recruit Bravo is already with you."]
	checks["state valid"] = gs.validate().is_empty()
	# A recruit below its base level threshold joins at that threshold; an explicit active_ids gains the id.
	var low: GameState = GameState.of(NewGame.create_for_world(W, SEED)["state"])
	low.party.member("hero_a").total_xp = 0
	low.party.active_ids = ["hero_a"] as Array[String]
	var r2: DialogueRunner = DialogueRunner.open(low, "sergeant")
	var res2: Dictionary = r2.choose(_find(r2.view(), "Take Charlie"))
	var charlie: UnitState = low.party.member("rec_c")
	checks["joins at the base level threshold"] = DictUtil.to_bool(res2["ok"]) and charlie != null and charlie.total_xp == XpCurve.xp_for_level(_level_of(_v))
	checks["explicit active_ids appended"] = low.party.active_ids == ["hero_a", "rec_c"] and low.validate().is_empty()
	checks["named on the field"] = charlie != null and PartyBuilder.build(charlie).name == "Recruit Charlie"
	return _report("Rules 4-6: recruit (XP rule, active_ids, join note, hidden once a member)", checks)

# Rule 4: a recruit past party_max is refused with the full-party note and changes nothing (all-or-nothing).
static func _test_full() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = GameState.of(NewGame.create_for_world(W, SEED)["state"])
	if gs == null:
		return _report("Rule 4: full party", {"state": false})
	var r: DialogueRunner = DialogueRunner.open(gs, "sergeant")
	var first: bool = DictUtil.to_bool(r.choose(_find(r.view(), "Take Bravo"))["ok"])
	var second: bool = DictUtil.to_bool(r.choose(_find(r.view(), "Take Charlie"))["ok"])
	checks["two recruits fill the cap"] = first and second and gs.party.members.size() == CAP
	var before: Dictionary = gs.to_dict()
	var third: Dictionary = r.choose(_find(r.view(), "Take Delta"))
	checks["third refused with the note"] = not DictUtil.to_bool(third["ok"]) and JsonUtil.strings(third["errors"]) == ["The party is full (%d of %d)." % [CAP, CAP]]
	checks["nothing changed"] = JsonUtil.deep_equal(gs.to_dict(), before) and gs.party.member("rec_d") == null
	checks["state valid"] = gs.validate().is_empty()
	return _report("Rule 4: third recruit refused (party full), all-or-nothing", checks)

# Rule 7: an old save of a capped world with four unnamed heroes loads three active, one in reserve; the muster
# refuses recruits because the roster is full; through SaveGame write / read too.
static func _test_old_save() -> bool:
	var checks: Dictionary = {}
	var old: GameState = GameState.of(NewGame.create(OPEN, [_u, _u, _u, _u] as Array[String], SEED)["state"])
	if old == null:
		return _report("Rule 7: old saves", {"state": false})
	old.party.active_ids = old.party.member_ids()
	checks["four active in an uncapped world"] = old.validate().is_empty() and old.party.active_members().size() == 4
	var d: Dictionary = old.to_dict()
	d["world_id"] = W
	var loaded: GameState = GameState.from_dict(d)
	checks["first three active, fourth in reserve"] = loaded.party.active_ids == old.party.member_ids().slice(0, CAP) and loaded.party.members.size() == 4 and loaded.party.active_members().size() == CAP
	checks["validate clean"] = loaded.validate().is_empty()
	var nameless: bool = true
	for m: UnitState in loaded.party.members:
		nameless = nameless and m.lore_ref.is_empty() and PartyBuilder.display_name(m) == str(ContentCatalog.get_entry("units", _u).get("name", _u))
	checks["old members keep no name"] = nameless
	var implicit: Dictionary = old.to_dict()
	implicit["world_id"] = W
	var implicit_party: Dictionary = implicit["party"]
	implicit_party["active_ids"] = []
	checks["implicit active capped too"] = GameState.from_dict(implicit).party.active_members().size() == CAP
	var r: DialogueRunner = DialogueRunner.open(loaded, "sergeant")
	var res: Dictionary = r.choose(_find(r.view(), "Take Bravo"))
	checks["muster refuses (roster full)"] = not DictUtil.to_bool(res["ok"]) and JsonUtil.strings(res["errors"]) == ["The party is full (4 of %d)." % CAP]
	var written: Array[String] = SaveGame.write(loaded, 1)
	var back: Dictionary = SaveGame.read(1)
	var read_gs: GameState = GameState.of(back["state"])
	checks["save round trip keeps the cap"] = written.is_empty() and DictUtil.to_bool(back["ok"]) and read_gs != null and read_gs.party.max_active == CAP and read_gs.party.active_members().size() == CAP and read_gs.party.members.size() == 4
	var _gone: bool = SaveGame.delete(1)
	return _report("Rule 7: old four-hero saves (three active, reserve, muster full, round trip)", checks)
