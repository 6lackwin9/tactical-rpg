class_name Bal2SimTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - BAL2 balance sim test suite (task BAL2-L commit 1b, docs/specs/BAL2_SIM.md 4-8)
# The sim helpers under tools/sim/: arguments of the party / role / sweep modes, the section 8
# targets and their X marks, the role loadout rule, the ranking check, the sweep compositions, and
# the metrics collector's conservation on real fights in a sandbox catalog fork. Expectations derive
# from the rules (owned class-path families, PARTY_MAX, HP bookkeeping), not from shipped numbers.
# ==============================================================================

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimTargets = preload("res://tools/sim/sim_targets.gd")
const SimRoles = preload("res://tools/sim/sim_roles.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimParty = preload("res://tools/sim/sim_party.gd")

const TAG: String = "BAL2-SIM"
const TEST_LEVEL: int = 5
const LOADOUT_LEVELS: Array[int] = [1, 3, 5, 7]   # the role loadout levels of BAL2_SIM 6.3

static var SANDBOX: String = TestHelpers.sandbox("bal2_sim_test")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING BAL2 SIM TEST (BAL2-L commit 1b)        ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_args() and ok
	ok = _test_targets() and ok
	ok = _test_role_map() and ok
	ok = _test_role_loadouts() and ok
	ok = _test_ranking() and ok
	ok = _test_compositions() and ok
	ok = _test_collector() and ok
	ok = _test_collector_detects() and ok
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return ok

static func _parse(tokens: Array[String]) -> SimArgs:
	var a := SimArgs.new()
	a.parse(PackedStringArray(tokens))
	return a

# Section 1 / 5 / 6.4 / 7 keys.
static func _test_args() -> bool:
	var checks: Dictionary = {}
	var too_many: Array[String] = ["party", "5"]
	var comp: Array[String] = []
	for i: int in range(PartyState.PARTY_MAX + 1):
		comp.append(SimArgs.CLASSES[i % SimArgs.CLASSES.size()])
	too_many.append("comp=" + ",".join(comp))
	var big := SimArgs.new()
	checks["comp of PARTY_MAX + 1 refused"] = not big.parse(PackedStringArray(too_many))
	checks["PARTY_MAX error text"] = big.error.contains("party size > PARTY_MAX (%d)" % PartyState.PARTY_MAX)
	var party: SimArgs = _parse(["party"])
	checks["party: metrics default on"] = party.error.is_empty() and party.metrics
	checks["party: default comp"] = party.comp == SimArgs.DEFAULT_COMP
	checks["party: default enc = the generated five"] = party.encounters == SimArgs.GENERATED
	checks["duel: metrics default off"] = not _parse(["duel"]).metrics
	var custom: SimArgs = _parse(["party", "3", "comp=rogue,rogue", "enc=mob,line", "elevel=+2", "loadout=role"])
	checks["custom party parsed"] = custom.error.is_empty() and custom.level == 3 and custom.comp.size() == 2
	checks["enc list"] = custom.encounters == (["mob", "line"] as Array[String])
	checks["elevel +2 explicit"] = custom.elevel == 2 and custom.elevel_set
	checks["loadout=role accepted"] = custom.loadout == "role"
	var neg: SimArgs = _parse(["party", "elevel=-1"])
	checks["elevel -1"] = neg.elevel == -1 and neg.elevel_set
	checks["unknown key refused"] = not SimArgs.new().parse(PackedStringArray(["sweep", "nope=1"]))
	checks["unknown encounter refused"] = not SimArgs.new().parse(PackedStringArray(["party", "enc=no_such_encounter_xyz"]))
	var all_enc: SimArgs = _parse(["party", "enc=all"])
	checks["enc=all = generated + catalog list"] = all_enc.encounters.size() == SimArgs.GENERATED.size() + ContentCatalog.ids("encounters").size()
	checks["role: default start distance ROLE_DISTANCE"] = _parse(["role"]).distance == SimArgs.ROLE_DISTANCE
	checks["role: distance=default keeps the deployment"] = _parse(["role", "distance=default"]).distance == 0
	checks["party / duel: default deployment"] = party.distance == 0 and _parse(["duel"]).distance == 0
	var role: SimArgs = _parse(["role", "role=aoe", "class=mage"])
	checks["role / class filters"] = role.roles == (["aoe"] as Array[String]) and role.classes == (["mage"] as Array[String])
	var t: SimArgs = _parse(["party", "target=clear_line:60-80", "target=swap_delta:-5-5"])
	checks["target overrides"] = t.error.is_empty() and SimTargets.lo(t.targets, "clear_line") == 60.0 and SimTargets.hi(t.targets, "clear_line") == 80.0 \
		and SimTargets.lo(t.targets, "swap_delta") == -5.0 and SimTargets.hi(t.targets, "swap_delta") == 5.0
	checks["target with an unknown key refused"] = not SimArgs.new().parse(PackedStringArray(["party", "target=nope:1-2"]))
	checks["defaults untouched by an override"] = SimTargets.lo(SimTargets.defaults(), "clear_line") == DictUtil.to_float(DictUtil.array_of(SimTargets.DEFAULTS["clear_line"])[0])
	var sweep: SimArgs = _parse(["sweep", "dupes=on", "sample=7"])
	checks["sweep dupes / sample"] = sweep.dupes and sweep.sample == 7
	checks["header names the party keys"] = party.header().contains("comp ") and party.header().contains("enc ")
	return TestHelpers.report(TAG, "Party / role / sweep arguments (PARTY_MAX, enc, elevel, target overrides)", checks)

# Section 8: decided defaults and the X mark rule.
static func _test_targets() -> bool:
	var checks: Dictionary = {}
	var d: Dictionary = SimTargets.defaults()
	var decided: Dictionary = {"clear_mob": [90.0, 100.0], "clear_line": [70.0, 90.0], "clear_wave": [55.0, 80.0], "clear_elite": [50.0, 75.0],
		"clear_boss": [30.0, 50.0], "swap_delta": [-10.0, 10.0], "line_min": [50.0, 100.0], "duel": [30.0, 70.0], "scenario_clear": [50.0, 85.0]}
	for key: Variant in decided:
		var pair: Array = decided[key]
		checks["%s decided %s" % [str(key), str(pair)]] = SimTargets.lo(d, str(key)) == DictUtil.to_float(pair[0]) and SimTargets.hi(d, str(key)) == DictUtil.to_float(pair[1])
	for key: Variant in d:
		var k: String = str(key)
		var lo: float = SimTargets.lo(d, k)
		var hi: float = SimTargets.hi(d, k)
		checks["%s lo <= hi" % k] = lo <= hi
		checks["%s bounds inside" % k] = SimTargets.mark(d, k, lo) == "" and SimTargets.mark(d, k, hi) == ""
		checks["%s below lo marked X" % k] = SimTargets.mark(d, k, lo - 1.0) == " X"
		checks["%s range below the band does not overlap" % k] = not SimTargets.overlaps(d, k, lo - 2.0, lo - 1.0)
		checks["%s range touching lo overlaps" % k] = SimTargets.overlaps(d, k, lo - 1.0, lo)
	checks["band text 70-90"] = SimTargets.band_text(d, "clear_line") == "70-90"
	checks["band text >=90"] = SimTargets.band_text(d, "clear_mob") == ">=90"
	checks["band text +-10"] = SimTargets.band_text(d, "swap_delta") == "+-10"
	return TestHelpers.report(TAG, "Section 8 targets: decided defaults, bounds inclusive, X outside", checks)

# 6.2: every class has a primary role; the user's rogue / brawler answers.
static func _test_role_map() -> bool:
	var checks: Dictionary = {}
	for c: String in SimArgs.CLASSES:
		var p: String = SimRoles.primary_role(c)
		checks["%s primary is a role" % c] = SimArgs.ROLES.has(p) and SimRoles.fit(c, p) == "P"
		checks["%s has a loadout for its primary role" % c] = DictUtil.dict_of(SimRoles.LOADOUTS.get(c, {})).has(p)
		for s: String in SimRoles.secondary_roles(c):
			checks["%s secondary %s is a role, not the primary" % [c, s]] = SimArgs.ROLES.has(s) and s != p and SimRoles.fit(c, s) == "S"
	checks["rogue = melee striker"] = SimRoles.primary_role("rogue") == "striker"
	checks["brawler = AoE"] = SimRoles.primary_role("brawler") == "aoe"
	for r: String in SimArgs.ROLES:
		checks["role %s has a scenario" % r] = SimScenarios.SCENARIOS.has(str(SimScenarios.ROLE_SCENARIO.get(r, "")))
		checks["role %s has KPIs" % r] = not SimRoles.kpis(r).is_empty()
	return TestHelpers.report(TAG, "Class -> role map (rogue striker, brawler AoE) and a scenario per role", checks)

# 6.3: the role list cut to owned families, filled in path order up to MAX_CLASS_SLOTS; it validates.
static func _test_role_loadouts() -> bool:
	var checks: Dictionary = {}
	for c: String in SimArgs.CLASSES:
		for r: String in SimArgs.ROLES:
			for lvl: int in LOADOUT_LEVELS:
				var got: Array[String] = SimRoles.role_loadout(c, r, lvl)
				var owned: Array[String] = SimRoles.owned_families(c, lvl)
				var tag: String = "%s %s L%d" % [c, r, lvl]
				var all_owned: bool = true
				var unique: Dictionary = {}
				for f: String in got:
					all_owned = all_owned and owned.has(f)
					unique[f] = true
				checks["%s: owned families only" % tag] = all_owned
				checks["%s: no duplicates" % tag] = unique.size() == got.size()
				checks["%s: size = min(slots, owned)" % tag] = got.size() == mini(StatRules.MAX_CLASS_SLOTS, owned.size())
				# The owned role families come first, in the table order.
				var table: Dictionary = DictUtil.dict_of(SimRoles.LOADOUTS[c])
				var key: String = r if table.has(r) else SimRoles.primary_role(c)
				var wanted: Array[String] = []
				for f: Variant in DictUtil.array_of(table[key]):
					wanted.append(str(f))
				var swap: Array = DictUtil.array_of(SimRoles.L7_SWAPS.get("%s:%s" % [c, key], []))
				if lvl >= SimRoles.L7_LEVEL and swap.size() == 2 and wanted.has(str(swap[0])):
					wanted[wanted.find(str(swap[0]))] = str(swap[1])
				var prefix: Array[String] = []
				for f: String in wanted:
					if owned.has(f) and not prefix.has(f):
						prefix.append(f)
				prefix = prefix.slice(0, StatRules.MAX_CLASS_SLOTS)
				checks["%s: owned role families first" % tag] = got.slice(0, prefix.size()) == prefix
	# The lists validate as unit loadouts (sandbox upsert).
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		for c: String in SimArgs.CLASSES:
			for r: String in SimArgs.ROLES:
				var entry: Dictionary = {"id": "bal2sim_%s_%s" % [c, r], "name": "T", "side": "party", "class_id": c, "level": TEST_LEVEL, "tier": "hero",
					"loadout": SimRoles.role_loadout(c, r, TEST_LEVEL)}
				var errors: Array[String] = ContentCatalog.upsert("units", entry)
				checks["%s %s L%d loadout validates %s" % [c, r, TEST_LEVEL, str(errors)]] = errors.is_empty()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Role loadouts: cut to owned, filled by path order, valid unit loadouts", checks)

# 8.1 ranking: primaries in the top 2, a secondary above every off-role class.
static func _test_ranking() -> bool:
	var checks: Dictionary = {}
	# tank: warrior P, brawler S. Ranger (off-role) on top pushes the warrior to rank 3.
	var marks: Dictionary = SimRoles.rank_marks("tank", {"warrior": 50.0, "brawler": 60.0, "ranger": 70.0, "priest": 10.0, "rogue": 20.0, "mage": 30.0})
	checks["primary at rank 3 marked"] = str(marks["warrior"]) == "rank 3 X"
	checks["secondary below an off-role class marked"] = str(marks["brawler"]) == "rank 2 X"
	checks["off-role never marked"] = str(marks["ranger"]) == "rank 1"
	var good: Dictionary = SimRoles.rank_marks("tank", {"warrior": 70.0, "brawler": 60.0, "ranger": 50.0, "priest": 10.0, "rogue": 20.0, "mage": 30.0})
	checks["primary first, secondary second: no X"] = str(good["warrior"]) == "rank 1" and str(good["brawler"]) == "rank 2"
	# striker ranks the lowest priority_down_round first.
	var s: Dictionary = SimRoles.rank_marks("striker", {"rogue": 3.0, "priest": 4.0, "brawler": 5.0, "warrior": 9.0, "ranger": 8.0, "mage": 7.0})
	checks["striker: lower is better"] = str(s["rogue"]) == "rank 1" and str(s["priest"]) == "rank 2" and str(s["brawler"]) == "rank 3"
	return TestHelpers.report(TAG, "Role ranking marks (top 2 primary, secondary above off-role, striker lowest first)", checks)

# 7: 15 distinct 4-class compositions, 126 multisets, a deterministic sample.
static func _test_compositions() -> bool:
	var checks: Dictionary = {}
	var n: int = SimArgs.CLASSES.size()
	var k: int = SimParty.COMP_SIZE
	var distinct: int = _choose(n, k)
	var multisets: int = _choose(n + k - 1, k)
	var plain := SimParty.new(_parse(["sweep"]), null, null)
	var comps: Array = plain.compositions()
	checks["distinct count C(%d,%d)" % [n, k]] = comps.size() == distinct
	var keys: Dictionary = {}
	var ok_distinct: bool = true
	for raw: Variant in comps:
		var comp: Array = DictUtil.array_of(raw)
		var u: Dictionary = {}
		for c: Variant in comp:
			u[c] = true
		ok_distinct = ok_distinct and u.size() == k
		keys[",".join(comp)] = true
	checks["each composition has %d different classes" % k] = ok_distinct
	checks["compositions unique"] = keys.size() == comps.size()
	var dup := SimParty.new(_parse(["sweep", "dupes=on"]), null, null)
	checks["multiset count C(%d,%d)" % [n + k - 1, k]] = dup.compositions().size() == multisets
	var sample_n: int = 10
	var a: Array = SimParty.new(_parse(["sweep", "dupes=on", "sample=%d" % sample_n]), null, null).compositions()
	var b: Array = SimParty.new(_parse(["sweep", "dupes=on", "sample=%d" % sample_n]), null, null).compositions()
	checks["sample size"] = a.size() == sample_n
	checks["sample deterministic (seed %d)" % SimParty.SAMPLE_SEED] = str(a) == str(b)
	return TestHelpers.report(TAG, "Sweep compositions (distinct, multisets, seeded sample)", checks)

static func _choose(n: int, k: int) -> int:
	var r: int = 1
	for i: int in range(k):
		r = r * (n - i) / (i + 1)
	return r

# 4: conservation on real fights (a generated wave encounter and a striker scenario) in a sandbox.
static func _test_collector() -> bool:
	var checks: Dictionary = {}
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var args: SimArgs = _parse(["party", str(TEST_LEVEL), "quick"])
		var runner := SimRunner.new(args)
		var metrics := SimMetrics.new()
		runner.observer = metrics
		var party: Array[String] = SimScenarios.define_party(runner, SimArgs.DEFAULT_COMP, TEST_LEVEL)
		var enc: Dictionary = SimScenarios.encounter(runner, "wave", TEST_LEVEL, 0, false, args.elite_class, args.boss_class)
		var foes: Array[String] = []
		foes.assign(DictUtil.array_of(enc["enemies"]))
		var base: Dictionary = DictUtil.dict_of(enc["base"])
		var res: Dictionary = runner.fight(party, foes, args.seeds()[0], base)
		checks["wave fight ran"] = DictUtil.to_bool(res.get("ok", false))
		var wave_n: int = DictUtil.array_of(base.get("wave_units", [])).size()
		checks["records: party + enemies + wave"] = metrics.units.size() == party.size() + foes.size() + wave_n
		_conservation(metrics, party, checks, "wave")
		checks["fight record keys"] = metrics.fight_record.has("cleared") and metrics.fight_record.has("priority_down_round")
		var sc: Dictionary = SimScenarios.scenario(runner, "striker_priority", "rogue", TEST_LEVEL)
		var sp: Array[String] = []
		sp.assign(DictUtil.array_of(sc["party"]))
		var se: Array[String] = []
		se.assign(DictUtil.array_of(sc["enemies"]))
		metrics.priority_name = str(sc["priority"])
		var res2: Dictionary = runner.fight(sp, se, args.seeds()[0])
		checks["striker fight ran"] = DictUtil.to_bool(res2.get("ok", false))
		_conservation(metrics, sp, checks, "striker")
		var down: int = DictUtil.to_int(metrics.fight_record["priority_down_round"])
		checks["priority_down_round in 1..cap+1"] = down >= 1 and down <= args.cap + 1
		var test_rec: Dictionary = metrics.units[str(sc["test"])]
		checks["priority_dmg <= dmg_dealt"] = DictUtil.int_of(test_rec, "priority_dmg") <= DictUtil.int_of(test_rec, "dmg_dealt")
		checks["no METRIC FAIL %s" % str(metrics.fail_lines)] = metrics.fail_lines.is_empty()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Metrics collector: HP conservation per unit and per side on real fights", checks)

# The checks must catch a mismatch: synthetic HP changes and log entries fed through the public
# signals (Combatant.health_changed via take_damage / heal, CombatSession.log_appended).
static func _test_collector_detects() -> bool:
	var checks: Dictionary = {}
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var args: SimArgs = _parse(["party", str(TEST_LEVEL), "quick"])
		var runner := SimRunner.new(args)
		var party: Array[String] = SimScenarios.define_party(runner, ["warrior"] as Array[String], TEST_LEVEL)
		var foes: Array[String] = SimScenarios.define_enemies(runner, [["warrior", "standard"]], TEST_LEVEL, 1, "warrior", "brawler")
		checks["units written"] = runner.build(party[0]) != null and runner.build(foes[0]) != null
		var entry_def: Dictionary = {"id": "bal2sim_probe", "name": "Probe", "zone": "stone_arena", "party": party, "enemies": foes, "max_rounds": 3, "rng_seed": 1}
		var session := CombatSession.new(CatalogEncounter.from_entry(entry_def))
		session.start()
		var metrics := SimMetrics.new()
		metrics.quiet = true   # the failures below are expected; the suite log must not show them as FAIL
		metrics.on_start(session)
		var a: Combatant = session.allies[0]
		var e: Combatant = session.enemies[0]
		var half: int = 5
		var hit: int = 2 * half
		# 1. A hit whose logged damage matches the HP lost: credited, no fail.
		e.take_damage(hit, true)
		session.log_appended.emit(_entry("ATTACK", a, e, "Reach 1-1", hit, 0))
		var rec_a: Dictionary = metrics.units[a.name]
		checks["matching hit credited to the actor"] = DictUtil.int_of(rec_a, "dmg_dealt") == hit and metrics.fail_lines.is_empty()
		# 2. The log reports less than the HP lost: log_damage fail.
		e.take_damage(hit, true)
		session.log_appended.emit(_entry("ATTACK", a, e, "Reach 1-1", half, 0))
		checks["log/HP damage mismatch fails"] = _has_fail(metrics, "log_damage")
		# 3. A gain under a non-heal entry is not a heal: other_gain.
		a.take_damage(2 * hit, true)
		session.log_appended.emit(_entry("ATTACK", e, a, "Reach 1-1", 2 * hit, 0))
		a.heal(hit)
		session.log_appended.emit(_entry("MOVE", a, null, "", 0, 0))
		checks["gain outside a heal entry is not heal_received"] = DictUtil.int_of(rec_a, "heal_received") == 0 and DictUtil.int_of(rec_a, "other_gain") == hit
		# 4. A heal entry whose amount differs from the HP gained: log_heal fail.
		a.heal(half)
		session.log_appended.emit(_entry("BUFF", a, a, "Support", hit, hit))
		checks["log/HP heal mismatch fails"] = _has_fail(metrics, "log_heal")
		# 5. A loss no entry explains, then the end of the fight: other_loss and the per-unit equation fail.
		e.take_damage(hit, true)
		metrics.on_end(session, {})
		checks["unexplained loss fails"] = _has_fail(metrics, "%s other_loss" % e.name)
		checks["phantom gain fails the per-unit equation"] = _has_fail(metrics, "%s hp (" % a.name) and _has_fail(metrics, "%s other_gain" % a.name)
		session.dispose()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Metrics collector self-checks catch log/HP mismatches and phantom heals", checks)

static func _entry(category: String, actor: Combatant, target: Combatant, subtype: String, damage: int, value: int) -> ActionLogEntry:
	var e := ActionLogEntry.new(1, 1, category, actor.name, target.name if target != null else "")
	e.skill_name = "Probe"
	e.skill_subtype = subtype
	e.damage = damage
	e.value_num = value
	return e

static func _has_fail(metrics: SimMetrics, needle: String) -> bool:
	for line: String in metrics.fail_lines:
		if line.contains(needle):
			return true
	return false

static func _conservation(metrics: SimMetrics, party: Array[String], checks: Dictionary, tag: String) -> void:
	var dealt: Array[int] = [0, 0]
	var taken: Array[int] = [0, 0]
	var party_kills: int = 0
	var enemies_down: int = 0
	var ok_units: bool = true
	var ok_frac: bool = true
	for n: Variant in metrics.units:
		var r: Dictionary = metrics.units[n]
		var side: int = 0 if party.has(str(n)) else 1
		dealt[side] += DictUtil.int_of(r, "dmg_dealt")
		taken[side] += DictUtil.int_of(r, "dmg_taken")
		var expect: int = DictUtil.int_of(r, "hp_start") - DictUtil.int_of(r, "dmg_taken") - DictUtil.int_of(r, "ff_taken") - DictUtil.int_of(r, "dot_taken") + DictUtil.int_of(r, "heal_received")
		ok_units = ok_units and expect == DictUtil.int_of(r, "hp_end") and DictUtil.int_of(r, "other_loss") == 0 and DictUtil.int_of(r, "other_gain") == 0
		ok_frac = ok_frac and DictUtil.float_of(r, "min_hp_frac") <= DictUtil.float_of(r, "hp_end_frac") + 0.0001 and DictUtil.int_of(r, "dmg_dealt_ranged") <= DictUtil.int_of(r, "dmg_dealt")
		if side == 0:
			party_kills += DictUtil.int_of(r, "kills")
		elif DictUtil.int_of(r, "hp_end") == 0:
			enemies_down += 1
	checks["%s: per unit hp_start - taken - ff - dot + heal == hp_end" % tag] = ok_units
	checks["%s: party dealt == enemy taken" % tag] = dealt[0] == taken[1]
	checks["%s: enemy dealt == party taken" % tag] = dealt[1] == taken[0]
	checks["%s: party kills <= enemies at 0 HP" % tag] = party_kills <= enemies_down
	checks["%s: min_hp_frac <= hp_end_frac, ranged part <= dealt" % tag] = ok_frac
