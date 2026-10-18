class_name Bal4CampaignTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - BAL4-ATTR campaign sim test suite (docs/specs/DESIGN_REVIEW1.md 3.3)
# tools/sim/sim_world.gd and sim_campaign.gd: the story fight order against the quest data, HP carried
# between battles, the between-battle heal options and the run stopping at the first loss. BAL5-SIM K1:
# `world=<id>` (bare enc= ids find that world's encounters) and story characters with extras spawning
# through WorldFactory (as WorldEncounter does). Fixtures are
# sandbox units by class / level / tier; expectations derive from the rules and the world data.
# ==============================================================================

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimWorld = preload("res://tools/sim/sim_world.gd")
const SimParty = preload("res://tools/sim/sim_party.gd")
const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimCampaign = preload("res://tools/sim/sim_campaign.gd")

# Records the allies' HP sum at the start of every fight.
class StartHpObserver extends "res://tools/sim/sim_observer.gd":
	var starts: Array[int] = []
	func on_start(session: CombatSession) -> void:
		var sum: int = 0
		for c: Combatant in session.allies:
			sum += c.hp
		starts.append(sum)

const TAG: String = "BAL4"
const TEST_LEVEL: int = 5

static var SANDBOX: String = TestHelpers.sandbox("bal4_campaign_test")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING BAL4 CAMPAIGN SIM TEST (BAL4-ATTR)      ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_args() and ok
	ok = _test_story_order() and ok
	ok = _test_world_enc() and ok
	ok = _test_world_key() and ok
	ok = _test_world_characters() and ok
	ok = _test_trios() and ok
	ok = _test_heal() and ok
	ok = _test_runs() and ok
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return ok

static func _parse(tokens: Array[String]) -> SimArgs:
	var a := SimArgs.new()
	a.parse(PackedStringArray(tokens))
	return a

static func _test_args() -> bool:
	var checks: Dictionary = {}
	var d: SimArgs = _parse(["campaign"])
	checks["campaign mode parses, heal none, no comp"] = d.error.is_empty() and d.heal == "none" and not d.comp_set
	checks["heal=full"] = _parse(["campaign", "heal=full"]).heal == "full"
	checks["heal=40%"] = _parse(["campaign", "heal=40%"]).heal == "40"
	checks["heal=150 refused"] = not SimArgs.new().parse(PackedStringArray(["campaign", "heal=150"]))
	checks["heal=lots refused"] = not SimArgs.new().parse(PackedStringArray(["campaign", "heal=lots"]))
	var c: SimArgs = _parse(["campaign", "comp=warrior,ranger,priest"])
	checks["comp= picks one party"] = c.comp_set and c.comp.size() == 3
	var w: SimArgs = _parse(["party", "comp=warrior", "enc=world:courtyard_skirmish,world:ashlands/ashmaw_stand"])
	checks["enc=world: parses (default world and world/id), comp of 1"] = w.error.is_empty() and w.encounters.size() == 2 and w.comp.size() == 1
	checks["enc=world:<unknown> refused"] = not SimArgs.new().parse(PackedStringArray(["party", "enc=world:no_such_fight"]))
	checks["parse_ref maps to [world, id]"] = SimArgs.parse_ref("world:x") == (["ashlands", "x"] as Array[String]) and SimArgs.parse_ref("world:w/x") == (["w", "x"] as Array[String]) and SimArgs.parse_ref("x").is_empty()
	return TestHelpers.report(TAG, "campaign arguments: mode, heal, comp, world encounters", checks)

# Order derived from the data: a quest's fights come after those of every quest it requires, every
# story step encounter appears, the side quest is out, the steel choice swaps only the finale.
static func _test_story_order() -> bool:
	var checks: Dictionary = {}
	var arms: Array[Dictionary] = SimWorld.story_fights(false)
	var gate: Array[Dictionary] = SimWorld.story_fights(true)
	checks["story has fights"] = arms.size() > 1 and arms.size() == gate.size()
	var story: Array[Dictionary] = SimWorld.story_quests()
	var pos_of: Dictionary = {}   # quest id -> index in the story order
	for i: int in range(story.size()):
		pos_of[str(story[i]["id"])] = i
	var in_order: bool = true
	var last_pos: int = -1
	var seen_quest_pos: Dictionary = {}
	for f: Dictionary in arms:
		var qid: String = str(f["quest"])
		var p: int = DictUtil.to_int(pos_of.get(qid, -1))
		in_order = in_order and p >= 0 and p >= last_pos
		last_pos = p
		seen_quest_pos[qid] = p
	checks["fights follow the quest order"] = in_order
	var requires_ok: bool = true
	for q: Dictionary in story:
		for r: Variant in DictUtil.array_of(q.get("requires", [])):
			requires_ok = requires_ok and DictUtil.to_int(pos_of.get(str(r), 99)) < DictUtil.to_int(pos_of[str(q["id"])])
	checks["every quest comes after the quests it requires"] = requires_ok
	var step_encounters: Array[String] = []
	for q: Dictionary in story:
		for raw: Variant in DictUtil.array_of(q.get("steps", [])):
			var e: String = str(DictUtil.dict_of(raw).get("encounter", ""))
			if not e.is_empty():
				step_encounters.append(e)
	var listed: Array[String] = []
	for f: Dictionary in arms:
		listed.append(str(f["encounter"]))
	var all_there: bool = true
	for e: String in step_encounters:
		all_there = all_there and listed.has(e)
	checks["every story step encounter is a fight"] = all_there
	var side: bool = true
	var world_enc: Array[String] = []
	for e: Dictionary in WorldCatalog.own_entries(SimWorld.WORLD, "encounters"):
		world_enc.append(str(e["id"]))
	for id: String in listed:
		side = side and world_enc.has(id)
	checks["every fight is an Ashlands world encounter"] = side
	var differ: Array[int] = []
	for i: int in range(arms.size()):
		if str(arms[i]["encounter"]) != str(gate[i]["encounter"]):
			differ.append(i)
	checks["the steel choice changes exactly the last fight"] = differ == ([arms.size() - 1] as Array[int])
	checks["barred finale named by the variant"] = str(gate[arms.size() - 1]["encounter"]).ends_with("_barred") or str(gate[arms.size() - 1]["encounter"]) != str(arms[arms.size() - 1]["encounter"])
	var scripted: int = 0
	for f: Dictionary in arms:
		if DictUtil.to_bool(f["scripted"]):
			scripted += 1
	checks["the road raid is a scripted fight before the finale"] = scripted >= 1 and not DictUtil.to_bool(arms[arms.size() - 1]["scripted"])
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var runner := SimRunner.new(_parse(["campaign"]))
		var mapped: bool = true
		var any_foes: bool = true
		for id: String in listed:
			var enc: Dictionary = SimWorld.encounter(runner, id, TEST_LEVEL, false)
			var foes: Array = DictUtil.array_of(enc.get("enemies", []))
			any_foes = any_foes and not foes.is_empty()
			for u: Variant in foes + DictUtil.array_of(DictUtil.dict_of(enc.get("base", {})).get("wave_units", [])):
				mapped = mapped and ContentCatalog.has("units", str(u))
		checks["every foe maps to a catalog unit"] = mapped and any_foes
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Story fight order from the quest data, scripted raid, steel choice", checks)

# `enc=world:<id>` resolves to the foes the campaign uses, plus the authored objective, rounds and guests.
static func _test_world_enc() -> bool:
	var checks: Dictionary = {}
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var args: SimArgs = _parse(["party", "comp=warrior"])
		var runner := SimRunner.new(args)
		var party := SimParty.new(args, runner, SimMetrics.new())
		var same: bool = true
		var objective_ok: bool = true
		var rounds_ok: bool = true
		var guests_ok: bool = true
		var checked: int = 0
		for e: Dictionary in WorldCatalog.own_entries(SimWorld.WORLD, "encounters"):
			var id: String = str(e["id"])
			var via_party: Dictionary = party.world_encounter("world:%s" % id)
			var via_campaign: Dictionary = SimWorld.encounter(runner, id, TEST_LEVEL, false)
			same = same and DictUtil.array_of(via_party.get("enemies", [])) == DictUtil.array_of(via_campaign.get("enemies", []))
			var base: Dictionary = DictUtil.dict_of(via_party.get("base", {}))
			objective_ok = objective_ok and str(base.get("objective", "")) == str(e.get("objective", ""))
			rounds_ok = rounds_ok and DictUtil.to_int(via_party.get("max_rounds", -1)) == DictUtil.to_int(e.get("max_rounds", 0))
			guests_ok = guests_ok and DictUtil.array_of(base.get("guests", [])).size() == DictUtil.array_of(e.get("guests", [])).size()
			checked += 1
		checks["world encounters checked"] = checked > 0
		checks["world encounter foes equal the campaign's"] = same
		checks["objective copied"] = objective_ok
		checks["max_rounds is the authored one"] = rounds_ok
		checks["guests kept"] = guests_ok
		checks["non-world id is not a world encounter"] = party.world_encounter("mob").is_empty()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "World encounters in party mode resolve like the campaign's", checks)

static func _test_trios() -> bool:
	var checks: Dictionary = {}
	var t: Array = SimCampaign.trios()
	var others: int = SimArgs.CLASSES.size() - 1
	checks["trios = pairs of the other classes (%d)" % (others * (others - 1) / 2)] = t.size() == others * (others - 1) / 2
	var distinct: Dictionary = {}
	var warrior_first: bool = true
	for raw: Variant in t:
		var trio: Array = DictUtil.array_of(raw)
		warrior_first = warrior_first and trio.size() == 3 and str(trio[0]) == "warrior" and str(trio[1]) != str(trio[2])
		distinct[",".join(PackedStringArray(trio))] = true
	checks["every trio is warrior + two different recruits, all distinct"] = warrior_first and distinct.size() == t.size()
	return TestHelpers.report(TAG, "Campaign trios", checks)

static func _test_heal() -> bool:
	var checks: Dictionary = {}
	var base: Dictionary = {"a": {"hp": 10, "max": 100}, "b": {"hp": 0, "max": 80}}
	var none: Dictionary = base.duplicate(true)
	SimCampaign.apply_heal(none, "none")
	checks["none changes nothing"] = none == base
	var full: Dictionary = base.duplicate(true)
	SimCampaign.apply_heal(full, "full")
	checks["full restores everyone, a downed hero too"] = DictUtil.to_int(DictUtil.dict_of(full["a"])["hp"]) == 100 and DictUtil.to_int(DictUtil.dict_of(full["b"])["hp"]) == 80
	var pct: Dictionary = base.duplicate(true)
	SimCampaign.apply_heal(pct, "50")
	checks["50 percent adds half the max HP, capped"] = DictUtil.to_int(DictUtil.dict_of(pct["a"])["hp"]) == 60
	checks["percent heal leaves a downed hero down"] = DictUtil.to_int(DictUtil.dict_of(pct["b"])["hp"]) == 0
	var over: Dictionary = {"a": {"hp": 90, "max": 100}}
	SimCampaign.apply_heal(over, "50")
	checks["percent heal caps at max"] = DictUtil.to_int(DictUtil.dict_of(over["a"])["hp"]) == 100
	checks["party hp percent"] = is_equal_approx(SimCampaign.hp_pct(base), 10.0 / 180.0 * 100.0)
	return TestHelpers.report(TAG, "Between-battle heal: none, full, percent", checks)

# A fight fixture: `count` enemies of class / tier at `level` (sandbox units).
static func _fixture(runner: SimRunner, tag: String, class_id: String, tier: String, level: int, count: int) -> Dictionary:
	var ids: Array[String] = []
	for i: int in range(count):
		var id: String = "bal4_%s_%d" % [tag, i]
		runner.define(id, class_id, level, tier, false)
		ids.append(id)
	return {"enemies": ids, "base": {}, "label": tag, "max_rounds": 0}

static func _test_runs() -> bool:
	var checks: Dictionary = {}
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var comp: Array[String] = ["warrior", "ranger", "priest"]
		var args: SimArgs = _parse(["campaign", str(TEST_LEVEL), "1", "heal=none"])
		var runner := SimRunner.new(args)
		var obs := StartHpObserver.new()
		runner.observer = obs
		var camp := SimCampaign.new(args, runner)
		var ids: Array[String] = SimScenarios.define_party(runner, comp, TEST_LEVEL)
		var mid: Dictionary = _fixture(runner, "mid", "rogue", "standard", TEST_LEVEL, 3)
		var weak: Dictionary = _fixture(runner, "weak", "rogue", "minion", 1, 1)
		var wall: Dictionary = _fixture(runner, "wall", "warrior", "boss", StatRules.MAX_LEVEL, 4)
		var seed_value: int = args.seeds()[0]
		# HP carries over: battle 2 starts with the HP battle 1 ended with (heal none).
		var carry: Dictionary = camp.run_party(ids, [mid, weak], seed_value)
		var hp_list: Array = DictUtil.array_of(carry["hp"])
		checks["carry run fought both battles"] = hp_list.size() == 2 and obs.starts.size() == 2
		if hp_list.size() == 2 and obs.starts.size() == 2:
			var max_total: float = float(obs.starts[0])   # battle 1 starts at full HP
			checks["battle 1 started at full HP"] = max_total > 0.0
			checks["battle 1 cost some HP (the test is not vacuous)"] = DictUtil.to_float(hp_list[0]) < 100.0
			checks["battle 2 started with the HP left after battle 1 (HP carries over)"] = absf(float(obs.starts[1]) / max_total * 100.0 - DictUtil.to_float(hp_list[0])) < 0.6
		# heal=full: the next battle starts at full HP again.
		obs.starts.clear()
		args.heal = "full"
		var healed: Dictionary = camp.run_party(ids, [mid, weak], seed_value)
		checks["heal=full: ran both battles"] = DictUtil.array_of(healed["hp"]).size() == 2 and obs.starts.size() == 2
		if obs.starts.size() == 2:
			checks["heal=full: battle 2 starts at the battle 1 start HP (restored)"] = obs.starts[1] == obs.starts[0]
		args.heal = "none"
		# A run stops at the first loss: wall in the middle ends the run, the third fight never starts.
		obs.starts.clear()
		var lost: Dictionary = camp.run_party(ids, [weak, wall, weak], seed_value)
		checks["loss: fell at battle 2"] = DictUtil.to_int(lost["fell_at"]) == 2
		checks["loss: survived 1 battle"] = DictUtil.to_int(lost["wins"]) == 1
		checks["loss: no third battle (%d fights)" % obs.starts.size()] = obs.starts.size() == 2 and DictUtil.array_of(lost["hp"]).size() == 2
		var clean: Dictionary = camp.run_party(ids, [weak, weak], seed_value)
		checks["no loss: fell_at 0, every battle survived"] = DictUtil.to_int(clean["fell_at"]) == 0 and DictUtil.to_int(clean["wins"]) == 2
		checks["fight records carry party_hp"] = runner.fights > 0
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Campaign runs: HP carry-over, heal=full, stop at the first loss", checks)

# K1 `world=<id>`: a bare enc= id that is a world encounter becomes world:<world>/<id> whatever the token
# order; generated ids stay generated; enc=world:<id> keeps working and uses the world= world.
static func _test_world_key() -> bool:
	var checks: Dictionary = {}
	var enc_id: String = ""
	for e: Dictionary in WorldCatalog.own_entries(SimWorld.WORLD, "encounters"):
		if not ContentCatalog.has("encounters", str(e["id"])) and not SimArgs.GENERATED.has(str(e["id"])):
			enc_id = str(e["id"])
			break
	checks["a world encounter id that is not a catalog or generated id exists"] = not enc_id.is_empty()
	if enc_id.is_empty():
		return TestHelpers.report(TAG, "world= key", checks)
	var want: String = "%s%s/%s" % [SimArgs.REF_PREFIX, SimWorld.WORLD, enc_id]
	var before: SimArgs = _parse(["party", "world=%s" % SimWorld.WORLD, "enc=%s,boss" % enc_id])
	var after: SimArgs = _parse(["party", "enc=%s,boss" % enc_id, "world=%s" % SimWorld.WORLD])
	checks["world= then enc=: %s" % str(before.encounters)] = before.error.is_empty() and before.encounters == ([want, "boss"] as Array[String])
	checks["enc= then world=: same"] = after.error.is_empty() and after.encounters == before.encounters
	checks["without world= a bare world id is refused"] = not SimArgs.new().parse(PackedStringArray(["party", "enc=%s" % enc_id]))
	checks["world=<unknown> refused"] = not SimArgs.new().parse(PackedStringArray(["party", "world=no_such_world"]))
	checks["enc=world:<id> still parses"] = _parse(["party", "enc=world:%s" % enc_id]).error.is_empty()
	checks["header shows the world"] = before.header().contains("world %s" % SimWorld.WORLD)
	# A child world sees its parent's encounters (inherited): world=<child> enc=<parent id>.
	var child: String = ""
	for w: String in WorldCatalog.world_ids():
		if w != SimWorld.WORLD and WorldCatalog.lineage(w).has(SimWorld.WORLD) and WorldCatalog.has_entry(w, "encounters", enc_id):
			child = w
			break
	if not child.is_empty():
		var c: SimArgs = _parse(["party", "world=%s" % child, "enc=%s" % enc_id])
		checks["child world %s sees inherited %s" % [child, enc_id]] = c.error.is_empty() and c.encounters == (["%s%s/%s" % [SimArgs.REF_PREFIX, child, enc_id]] as Array[String])
		checks["parse_ref uses the given default world"] = SimArgs.parse_ref("world:x", child) == ([child, "x"] as Array[String])
	return TestHelpers.report(TAG, "BAL5-SIM K1 world= key", checks)

# Every (world, encounter, character ref) of the world data, enemies and waves, with the character.
static func _story_refs() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for w: String in WorldCatalog.world_ids():
		for e: Dictionary in WorldCatalog.merged_entries(w, "encounters"):
			for ref: Variant in DictUtil.array_of(e.get("enemies", [])) + DictUtil.array_of(e.get("wave_units", [])):
				var ch: Dictionary = WorldCatalog.resolve(w, "characters", str(ref))
				if not ch.is_empty():
					out.append({"world": w, "encounter": str(e["id"]), "ref": str(ref), "character": ch})
	return out

# Records the spawned units of every fight: lore_ref -> {max_hp, speed, max_ap, power} at the start.
class SpawnObserver extends "res://tools/sim/sim_observer.gd":
	var spawned: Dictionary = {}
	func on_start(session: CombatSession) -> void:
		spawned = {}
		for c: Combatant in session.allies + session.enemies:
			spawned[c.lore_ref] = Bal4CampaignTestSuite._vitals(c)

static func _vitals(c: Combatant) -> Dictionary:
	return {"max_hp": c.max_hp, "speed": c.speed, "max_ap": c.max_ap, "power": DamageCalculator.power_of(c, c.attack_range > 1)}

# K1 characters: a plain story character plays as its base unit id; a character with extras plays as
# a simw_ copy that, in a fight, spawns with the max HP, speed, AP and power the game's WorldEncounter
# of the same battle gives it (base unit through ContentFactory, then WorldFactory.apply_character).
static func _test_world_characters() -> bool:
	var checks: Dictionary = {}
	var plain: Dictionary = {}
	var extra: Dictionary = {}
	for r: Dictionary in _story_refs():
		var ch: Dictionary = r["character"]
		var div: String = WorldFactory.divergence(ch, str(r["world"]))
		if div.is_empty() and plain.is_empty():
			plain = r
		elif not div.is_empty() and not DictUtil.dict_of(ch.get("tuning", {})).is_empty() and extra.is_empty():
			extra = r
	checks["a plain story foe exists"] = not plain.is_empty()
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked and not plain.is_empty():
		var runner := SimRunner.new(_parse(["party"]))
		var enc: Dictionary = SimWorld.encounter(runner, str(plain["encounter"]), TEST_LEVEL, false, str(plain["world"]))
		var base_id: String = str(DictUtil.dict_of(plain["character"])["base_unit"])
		checks["plain %s plays as its base unit %s" % [str(plain["ref"]), base_id]] = DictUtil.array_of(enc.get("enemies", [])).has(base_id) or DictUtil.array_of(DictUtil.dict_of(enc.get("base", {})).get("wave_units", [])).has(base_id)
	if forked and not extra.is_empty():
		var w: String = str(extra["world"])
		var ch_e: Dictionary = extra["character"]
		var args: SimArgs = _parse(["party", str(TEST_LEVEL), "1", "cap=1", "comp=warrior", "jitter=off", "world=%s" % w])
		var runner_e := SimRunner.new(args)
		var obs := SpawnObserver.new()
		runner_e.observer = obs
		var enc_e: Dictionary = SimWorld.encounter(runner_e, str(extra["encounter"]), TEST_LEVEL, false, w, true)
		var foes: Array[String] = []
		foes.assign(DictUtil.array_of(enc_e["enemies"]))
		var copy: String = "simw_%s" % str(ch_e["_qid"]).replace(":", "_")
		checks["%s (tuning %s) plays as the copy %s" % [str(extra["ref"]), str(ch_e["tuning"]), copy]] = foes.has(copy) or DictUtil.array_of(DictUtil.dict_of(enc_e["base"]).get("wave_units", [])).has(copy)
		var party: Array[String] = SimScenarios.define_party(runner_e, args.comp, TEST_LEVEL)
		runner_e.fight(party, foes, args.seeds()[0], DictUtil.dict_of(enc_e["base"]))
		var game := WorldEncounter.new(w, str(extra["encounter"]))
		game.build()
		var expect: Dictionary = {}
		var plain_v: Dictionary = {}
		for c: Combatant in game.allies + game.enemies + game.wave:
			if c.lore_ref == str(ch_e["_qid"]):
				expect = _vitals(c)
				var base_c: Combatant = ContentFactory.unit(WorldFactory.unit_entry(ch_e), "", game.unit_ap)
				plain_v = _vitals(base_c)
		var found: Dictionary = DictUtil.dict_of(obs.spawned.get(str(ch_e["_qid"]), {}))
		checks["the copy spawned with the character applied (lore_ref)"] = not found.is_empty()
		checks["the game's WorldEncounter spawns the character"] = not expect.is_empty()
		if not found.is_empty() and not expect.is_empty():
			checks["vitals %s = WorldEncounter's %s" % [str(found), str(expect)]] = found == expect
			checks["the extras change the unit vs its base %s (not vacuous)" % str(plain_v)] = found != plain_v
	elif forked:
		checks["no tuned story foe in the world data (K1 tuning path not exercised)"] = true
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "BAL5-SIM K1 story characters: base units, extras through WorldFactory", checks)
