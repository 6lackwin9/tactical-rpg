class_name BattleReportTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Battle report test suite (BREPORT1-L, docs/specs/BREPORT1.md sections 1-3)
# Sim parity (one sim fight seen by SimMetrics and by a live-attached BattleMetrics: equal records, details and KPI
# lines), the combo signal (test-only break / exploit / flow / area skills in a sandbox fork), the collector's combo
# counts, rewinds (a reset round leaves no trace; a round 1 redo starts over), freezing, bleed and regen credit,
# columns, best moment, row states for won / lost / fled, plumbing (settle, router, model validation) and rule 8 (the
# collector changes no round outcome). Units come from the sim's sandbox definitions; expectations derive from the
# rules and from the resolver's own step rows, never from shipped numbers.
# ==============================================================================

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")

const TAG: String = "BREPORT1"
const LEVEL: int = 5
const SCENE: String = "res://scenes/overworld/region_map_debug.tscn"
const WORLD: String = "ashlands"
const BREAK: String = "t_br_break"
const EXPLOIT: String = "t_br_exploit"
const FLOW: String = "t_br_flow"
const MAX_STEPS: int = 600

static var SANDBOX: String = TestHelpers.sandbox("battle_report_test")

# Sees one sim fight twice: the sim's own collector through the runner hooks, and a BattleMetrics attached to the
# session like the game does. Also counts the resolver's combo signals.
class TeeObserver extends BattleObserver:
	var sim: BattleMetrics = null
	var live: BattleMetrics = BattleMetrics.new()
	var combo_events: int = 0
	func on_start(session: CombatSession) -> void:
		sim.on_start(session)
		live.attach(session)
		session.resolver.combo_resolved.connect(_count)
	func _count(_kind: String, _actor: Combatant, _target: Combatant, _status: String, _skill: String, _killed: bool) -> void:
		combo_events += 1
	func on_step(session: CombatSession) -> void:
		sim.on_step(session)
	func on_round(session: CombatSession) -> void:
		sim.on_round(session)
	func on_end(session: CombatSession, result: Dictionary) -> void:
		sim.on_end(session, result)
		live.finish()

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING BATTLE REPORT TEST (BREPORT1-L)         ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_columns() and ok
	ok = _test_best_moment() and ok
	ok = _test_problems() and ok
	if TestHelpers.fork(SANDBOX) and _fixture_skills():
		ok = _test_combo_signal() and ok
		ok = _test_parity() and ok
		ok = _test_combo_counts() and ok
		ok = _test_rewind() and ok
		ok = _test_redo_round_one() and ok
		ok = _test_frozen() and ok
		ok = _test_dot_and_regen() and ok
		ok = _test_status_source() and ok
		ok = _test_no_outcome_change() and ok
	else:
		ok = TestHelpers.report_ok(TAG, "sandbox fork and fixture skills", false) and ok
	TestHelpers.cleanup(SANDBOX)
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	ok = _test_model() and ok
	ok = _test_game_battles(tree_parent) and ok
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

static func _fixture_skills() -> bool:
	var one: Dictionary = {"level": 1, "blocks": 1, "power_mult": 1.0}
	var rows: Array[Dictionary] = [
		TestHelpers.attack(BREAK, _with(one, {"disrupt": "STANCE_BREAK"})),
		TestHelpers.attack(EXPLOIT, _with(one, {"exploit": [{"status": StatusEffects.BLEED, "power_mult": 1.2}]})),
		TestHelpers.attack(FLOW, _with(one, {"ends_in_stance": "DEFENSIVE"})),
	]
	var ok: bool = true
	for row: Dictionary in rows:
		var errors: Array[String] = ContentCatalog.upsert_skill(row)
		if not errors.is_empty():
			print("[%s] fixture %s: %s" % [TAG, str(row["id"]), str(errors)])
			ok = false
	SkillCatalog.clear_cache()
	return ok and SkillCatalog.find(BREAK) != null and SkillCatalog.find(EXPLOIT) != null and SkillCatalog.find(FLOW) != null

static func _with(base: Dictionary, extra: Dictionary) -> Dictionary:
	var out: Dictionary = base.duplicate(true)
	out.merge(extra, true)
	return out

static func _runner() -> SimRunner:
	var args := SimArgs.new()
	args.parse(PackedStringArray(["party", str(LEVEL), "quick"]))
	return SimRunner.new(args)

# A started battle of sandbox sim units: the default sim party against two standard foes.
static func _session(runner: SimRunner, seed_value: int, comp: Array[String] = SimArgs.DEFAULT_COMP) -> CombatSession:
	var party: Array[String] = SimScenarios.define_party(runner, comp, LEVEL)
	var foes: Array[String] = SimScenarios.define_enemies(runner, [["warrior", "standard"], ["rogue", "standard"]], LEVEL, 1, "warrior", "brawler")
	for id: String in party + foes:
		var _built: Combatant = runner.build(id)
	var entry: Dictionary = {"id": "t_breport", "name": "Report test", "zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12,
		"ap_regen": 4, "party": party, "enemies": foes, "max_rounds": 8, "rng_seed": seed_value}
	var s := CombatSession.new(CatalogEncounter.from_entry(entry))
	s.start()
	return s

# Plans the round with `preset` for the whole party and resolves it. False = the plan was refused.
static func _play_round(s: CombatSession, preset: String) -> bool:
	s.plan_apply_preset_all(preset)
	if not s.confirm_plan(false):
		return false
	for i: int in range(MAX_STEPS):
		if s.phase == CombatSession.Phase.REPLAY:
			break
		s.step_next()
	return s.phase == CombatSession.Phase.REPLAY

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])

static func _command(actor: Combatant, foe: Combatant, skill_id: String, grid: CombatGrid) -> BattleCommand:
	var entry: PlanEntry = PlanEntry.create(SkillCatalog.find(skill_id), 1, 0)
	entry.target = foe
	entry.skill_ap = entry.skill.ap_cost(actor)
	return entry.build_command(actor, grid)

static func _active(c: Combatant, status: String) -> void:
	StatusEffects.queue(c, status, 2, 0, "Setup")
	StatusEffects.roll_over(c, -1)

# combo_resolved rows of one resolved round on `r`.
static func _combos_of(r: CombatResolver) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	r.combo_resolved.connect(func(kind: String, actor: Combatant, target: Combatant, status: String, skill: String, killed: bool) -> void:
		out.append({"kind": kind, "actor": actor, "target": target, "status": status, "skill": skill, "killed": killed}))
	r.resolve_round()
	return out

static func _sum_values(d: Dictionary) -> int:
	var n: int = 0
	for k: Variant in d:
		n += DictUtil.int_of(d, k)
	return n

# ------------------------------------------------------------------------------
# 1.3 Combo signal
# ------------------------------------------------------------------------------

static func _test_combo_signal() -> bool:
	var checks: Dictionary = {}
	# A stance-break skill on a guarding (Defensive) foe: one break.
	var g1: CombatGrid = _grid()
	var a1: Combatant = TestHelpers.unit("Atk", g1, Vector2i(3, 4), 900)
	var f1: Combatant = TestHelpers.unit("Guard", g1, Vector2i(4, 4), 900)
	f1.current_stance = CombatTypes.StanceType.DEFENSIVE
	var r1 := CombatResolver.new()
	r1.grid = g1
	var c1: BattleCommand = _command(a1, f1, BREAK, g1)
	c1.stance_on_start = -1
	r1.schedule_action(a1, c1, 1)
	var rows1: Array[Dictionary] = _combos_of(r1)
	checks["break on a guarding foe: one 'break' on it"] = rows1.size() == 1 and str(rows1[0]["kind"]) == "break" and rows1[0]["target"] == f1 and rows1[0]["actor"] == a1 and not DictUtil.to_bool(rows1[0]["killed"])
	checks["the break row names the skill"] = rows1.size() == 1 and str(rows1[0]["skill"]) == c1.command_name
	# An exploit hit on a bleeding foe: one exploit with status bleed.
	var g2: CombatGrid = _grid()
	var a2: Combatant = TestHelpers.unit("Atk", g2, Vector2i(3, 4), 900)
	var f2: Combatant = TestHelpers.unit("Bleeder", g2, Vector2i(4, 4), 900)
	_active(f2, StatusEffects.BLEED)
	var r2 := CombatResolver.new()
	r2.grid = g2
	var c2: BattleCommand = _command(a2, f2, EXPLOIT, g2)
	c2.stance_on_start = -1
	r2.schedule_action(a2, c2, 1)
	var rows2: Array[Dictionary] = _combos_of(r2)
	checks["exploit on a bleeding foe: one 'exploit' with status bleed"] = rows2.size() == 1 and str(rows2[0]["kind"]) == "exploit" and str(rows2[0]["status"]) == StatusEffects.BLEED and rows2[0]["target"] == f2
	checks["BattleCommand.exploited lists it"] = c2.exploited.size() == 1 and str(c2.exploited[0]["status"]) == StatusEffects.BLEED
	# A flow move: one flow, target = the actor, status "".
	var g3: CombatGrid = _grid()
	var a3: Combatant = TestHelpers.unit("Flower", g3, Vector2i(3, 4), 900)
	var f3: Combatant = TestHelpers.unit("Foe", g3, Vector2i(4, 4), 900)
	var r3 := CombatResolver.new()
	r3.grid = g3
	var c3: BattleCommand = _command(a3, f3, FLOW, g3)
	r3.schedule_action(a3, c3, 1)
	var rows3: Array[Dictionary] = _combos_of(r3)
	checks["flow: one 'flow' on the actor with no status"] = rows3.size() == 1 and str(rows3[0]["kind"]) == "flow" and rows3[0]["target"] == a3 and str(rows3[0]["status"]).is_empty()
	# An area attack exploiting two victims: two exploit rows.
	var area_skill: SkillDef = ContentFactory.skill(TestHelpers.attack("t_br_area", {"level": 1, "blocks": 1, "area": "PLUS", "exploit": [{"status": StatusEffects.BLEED, "power_mult": 1.2}]}))
	var g4: CombatGrid = _grid()
	var caster: Combatant = TestHelpers.unit("Caster", g4, Vector2i(2, 4))
	var x: Combatant = TestHelpers.unit("X", g4, Vector2i(4, 4), 900)
	var y: Combatant = TestHelpers.unit("Y", g4, Vector2i(5, 4), 900)
	_active(x, StatusEffects.BLEED)
	_active(y, StatusEffects.BLEED)
	var r4 := CombatResolver.new()
	r4.grid = g4
	var area := AreaAttackCommand.new(caster, g4, [Vector2i(4, 4), Vector2i(5, 4)] as Array[Vector2i], area_skill.power_for(caster), 1)
	area.exploit = area_skill.exploit
	r4.schedule_action(caster, area, 1)
	var rows4: Array[Dictionary] = _combos_of(r4)
	var kinds_ok: bool = rows4.size() == 2
	for row: Dictionary in rows4:
		kinds_ok = kinds_ok and str(row["kind"]) == "exploit"
	checks["area exploiting two victims: two rows"] = kinds_ok and rows4.size() == 2 and rows4[0]["target"] != rows4[1]["target"]
	# A failed command (its target is down): nothing.
	var g5: CombatGrid = _grid()
	var a5: Combatant = TestHelpers.unit("Atk", g5, Vector2i(3, 4))
	var f5: Combatant = TestHelpers.unit("Gone", g5, Vector2i(4, 4))
	var r5 := CombatResolver.new()
	r5.grid = g5
	r5.register_combatant(f5)
	var c5: BattleCommand = _command(a5, f5, FLOW, g5)
	c5.stance_on_start = -1
	r5.schedule_action(a5, c5, 1)
	f5.hp = 0
	var rows5: Array[Dictionary] = _combos_of(r5)
	checks["a failed command emits none"] = c5.failed and rows5.is_empty()
	return TestHelpers.report(TAG, "combo signal: break, exploit, flow, area x2, failed none (1.3)", checks)

# The collector's combos / combo_counts / best_finish from combo_resolved (public signal, emitted on a live session).
static func _test_combo_counts() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(_runner(), 11)
	var m := BattleMetrics.new()
	m.attach(s)
	var hero: Combatant = s.allies[0]
	var foe: Combatant = s.enemies[0]
	s.resolver.combo_resolved.emit("exploit", hero, foe, StatusEffects.BLEED, "Test Skill", true)
	s.resolver.combo_resolved.emit("break", hero, foe, "", "Test Skill", false)
	s.resolver.combo_resolved.emit("flow", hero, hero, "", "Test Skill", false)
	var rec: Dictionary = DictUtil.dict_of(m.units[hero.name])
	var det: Dictionary = DictUtil.dict_of(m.detail[hero.name])
	var counts: Dictionary = DictUtil.dict_of(det["combo_counts"])
	checks["combos = 3 = the sum of combo_counts"] = DictUtil.int_of(rec, "combos") == 3 and _sum_values(counts) == 3
	checks["one of each kind"] = DictUtil.int_of(counts, "exploit") == 1 and DictUtil.int_of(counts, "break") == 1 and DictUtil.int_of(counts, "flow") == 1
	var finish: Dictionary = DictUtil.dict_of(det["best_finish"])
	checks["a killing exploit on a foe is the best finish"] = str(finish.get("combo", "")) == "exploit" and str(finish.get("target", "")) == foe.name and str(finish.get("skill", "")) == "Test Skill"
	checks["the foe's record is untouched"] = DictUtil.int_of(DictUtil.dict_of(m.units[foe.name]), "combos") == 0
	m.freeze()
	s.resolver.combo_resolved.emit("flow", hero, hero, "", "Test Skill", false)
	checks["frozen: a later combo is not counted"] = DictUtil.int_of(DictUtil.dict_of(m.units[hero.name]), "combos") == 3
	s.dispose()
	return TestHelpers.report(TAG, "collector combo counts and best finish (1.2)", checks)

# ------------------------------------------------------------------------------
# Rule 4 / acceptance: sim parity
# ------------------------------------------------------------------------------

static func _test_parity() -> bool:
	var checks: Dictionary = {}
	var runner: SimRunner = _runner()
	var tee := TeeObserver.new()
	tee.sim = SimMetrics.new()
	runner.observer = tee
	var party: Array[String] = SimScenarios.define_party(runner, SimArgs.DEFAULT_COMP, LEVEL)
	var enc: Dictionary = SimScenarios.encounter(runner, "wave", LEVEL, 0, false, runner.args.elite_class, runner.args.boss_class)
	var foes: Array[String] = []
	foes.assign(DictUtil.array_of(enc["enemies"]))
	var res: Dictionary = runner.fight(party, foes, runner.args.seeds()[0], DictUtil.dict_of(enc["base"]))
	checks["the sim fight ran"] = DictUtil.to_bool(res.get("ok", false))
	checks["both collectors have every unit"] = not tee.sim.units.is_empty() and tee.sim.units.keys() == tee.live.units.keys()
	var fields_ok: bool = true
	var bad: Array[String] = []
	for n: Variant in tee.sim.units:
		var a: Dictionary = DictUtil.dict_of(tee.sim.units[n])
		var b: Dictionary = DictUtil.dict_of(tee.live.units.get(n, {}))
		for f: String in BattleMetrics.FIELDS:
			if str(a.get(f, null)) != str(b.get(f, null)):
				fields_ok = false
				bad.append("%s.%s %s/%s" % [str(n), f, str(a.get(f, null)), str(b.get(f, null))])
	checks["equal records for every unit and field %s" % str(bad.slice(0, 4))] = fields_ok
	checks["equal details (status / combo counts, best moments)"] = tee.sim.detail == tee.live.detail
	var kpi_ok: bool = true
	for n: Variant in tee.sim.units:
		var acc_s: Dictionary = {}
		var acc_l: Dictionary = {}
		SimMetrics.accumulate(acc_s, DictUtil.dict_of(tee.sim.units[n]))
		SimMetrics.accumulate(acc_l, DictUtil.dict_of(tee.live.units.get(n, {})))
		kpi_ok = kpi_ok and SimMetrics.kpi_line(str(n), acc_s) == SimMetrics.kpi_line(str(n), acc_l)
	checks["the KPI lines are byte-identical"] = kpi_ok
	var combos: int = 0
	var counts_ok: bool = true
	for n: Variant in tee.live.units:
		var c: int = DictUtil.int_of(DictUtil.dict_of(tee.live.units[n]), "combos")
		combos += c
		counts_ok = counts_ok and c == _sum_values(DictUtil.dict_of(DictUtil.dict_of(tee.live.detail[n])["combo_counts"]))
	checks["combos sum to the resolver's combo signals (%d)" % tee.combo_events] = combos == tee.combo_events
	checks["each unit's combos = its combo_counts"] = counts_ok
	checks["no METRIC FAIL in either"] = tee.sim.fail_lines.is_empty() and tee.live.fail_lines.is_empty()
	checks["the live collector is frozen after finish()"] = tee.live.frozen
	return TestHelpers.report(TAG, "sim parity: SimMetrics and live BattleMetrics agree on one fight (rule 4)", checks)

# ------------------------------------------------------------------------------
# Rule 2: rewinds
# ------------------------------------------------------------------------------

static func _test_rewind() -> bool:
	var checks: Dictionary = {}
	var runner: SimRunner = _runner()
	var a: CombatSession = _session(runner, 23)
	var ma := BattleMetrics.new()
	ma.quiet = true
	ma.attach(a)
	var played: bool = _play_round(a, "balanced") and not a.combat_over
	a.advance_round()
	played = played and _play_round(a, "aggressive")
	checks["rounds 1-2 played (round 1 does not end the fight)"] = played and a.current_round == 2
	var after_first_plan: Dictionary = ma.units.duplicate(true)
	a.reset_round()
	checks["reset_round rewinds round 2"] = a.current_round == 2 and a.phase == CombatSession.Phase.PLANNING
	var rng_state: int = a.resolver.rng.state
	played = _play_round(a, "defensive")
	ma.finish()
	# The same battle with only the second plan in round 2 (the RNG resumes where the rewound battle's does).
	var b: CombatSession = _session(runner, 23)
	var mb := BattleMetrics.new()
	mb.quiet = true
	mb.attach(b)
	var played_b: bool = _play_round(b, "balanced")
	b.advance_round()
	b.resolver.rng.state = rng_state
	played_b = played_b and _play_round(b, "defensive")
	mb.finish()
	checks["both replays of round 2 ran"] = played and played_b
	checks["the rewound battle's records equal the battle that played only the second plan"] = ma.units == mb.units
	checks["...and its details"] = ma.detail == mb.detail
	checks["the undone plan left a different record (the test can tell)"] = after_first_plan != ma.units
	var ra: Dictionary = BattleReport.build(ma, a, "won")
	var rb: Dictionary = BattleReport.build(mb, b, "won")
	checks["equal reports"] = ra == rb
	a.dispose()
	b.dispose()
	return TestHelpers.report(TAG, "rewind: a reset round leaves no trace (rule 2)", checks)

static func _test_redo_round_one() -> bool:
	var checks: Dictionary = {}
	var runner: SimRunner = _runner()
	var s: CombatSession = _session(runner, 29)
	var m := BattleMetrics.new()
	m.quiet = true
	m.attach(s)
	var fresh: Dictionary = m.units.duplicate(true)
	checks["round 1 played"] = _play_round(s, "aggressive")
	var dealt: int = 0
	for n: Variant in m.units:
		dealt += DictUtil.int_of(DictUtil.dict_of(m.units[n]), "dmg_dealt")
	checks["round 1 dealt damage (the test can tell)"] = dealt > 0
	s.reset_round()   # round 1: start() rebuilds the units
	var zero: bool = true
	for n: Variant in m.units:
		var rec: Dictionary = DictUtil.dict_of(m.units[n])
		for f: String in ["dmg_dealt", "dot_dealt", "dmg_taken", "dot_taken", "ff_taken", "heal_done", "buffs_applied", "debuffs_applied", "combos", "kills"]:
			zero = zero and DictUtil.int_of(rec, f) == 0
	checks["after the redo every count is 0"] = zero
	checks["the records equal a fresh start"] = m.units == fresh
	checks["the collector follows the rebuilt units"] = _play_round(s, "aggressive") and _any_dealt(m)
	m.freeze()
	s.dispose()
	return TestHelpers.report(TAG, "round 1 redo (start) starts the records over (rule 2)", checks)

static func _any_dealt(m: BattleMetrics) -> bool:
	for n: Variant in m.units:
		if DictUtil.int_of(DictUtil.dict_of(m.units[n]), "dmg_dealt") > 0:
			return true
	return false

# ------------------------------------------------------------------------------
# Rule 3: frozen; rule 4: bleed and regen credit
# ------------------------------------------------------------------------------

static func _test_frozen() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(_runner(), 31)
	var m := BattleMetrics.new()
	m.quiet = true
	m.attach(s)
	checks["round 1 played"] = _play_round(s, "balanced")
	m.finish()
	var first: Dictionary = BattleReport.build(m, s, "won")
	var kept: Dictionary = m.units.duplicate(true)
	# Replay the round over its snapshots, then rewind and play it again: the frozen records do not move.
	s.select_round(1)
	s.toggle_auto_play()
	for i: int in range(MAX_STEPS):
		if not s.is_auto_playing:
			break
		s.auto_play_tick()
	checks["after the replay the records are unchanged"] = m.units == kept
	checks["the report built again is the same"] = BattleReport.build(m, s, "won") == first
	s.reset_round()
	var _again: bool = _play_round(s, "aggressive")
	checks["a later rewind and replay change nothing"] = m.units == kept and m.frozen
	s.dispose()
	return TestHelpers.report(TAG, "frozen at the end: Replay and rewinds cannot change it (rule 3)", checks)

static func _test_dot_and_regen() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(_runner(), 37)
	var hero: Combatant = null
	var priest: Combatant = null
	for c: Combatant in s.allies:
		if c.name.contains("priest"):
			priest = c
		elif hero == null:
			hero = c
	var foe: Combatant = s.enemies[0]
	checks["fixture: a priest and another hero"] = priest != null and hero != null
	if priest == null or hero == null:
		return TestHelpers.report(TAG, "bleed and regen credit", checks)
	StatusEffects.grant_now(foe, StatusEffects.BLEED, 3, 0, hero.name)
	hero.hp = maxi(1, roundi(float(hero.max_hp) * 0.5))   # room to regenerate
	StatusEffects.grant_now(hero, StatusEffects.REGEN, 3, 0, priest.name)
	var m := BattleMetrics.new()
	m.quiet = true
	m.attach(s)
	var bleed_by_hero: int = 0
	var regen_by_priest: int = 0
	var steps_seen: Array[Dictionary] = []
	s.round_concluded.connect(func(_r: int, _over: bool) -> void: steps_seen.assign(s.resolver.last_round_end_steps))
	checks["round 1 played to its end"] = _play_round(s, "defensive")
	for row: Dictionary in steps_seen:
		if str(row["status"]) == StatusEffects.BLEED and str(row["source"]) == hero.name:
			bleed_by_hero += DictUtil.to_int(row["amount"])
		if str(row["status"]) == StatusEffects.REGEN and str(row["source"]) == priest.name:
			regen_by_priest += DictUtil.to_int(row["amount"])
	checks["the round end had a bleed step from the hero (%d) and a regen step from the priest (%d)" % [bleed_by_hero, regen_by_priest]] = bleed_by_hero > 0 and regen_by_priest > 0
	m.finish()
	var report: Dictionary = BattleReport.build(m, s, "won")
	var hero_row: Dictionary = _row(report, hero.name)
	var priest_row: Dictionary = _row(report, priest.name)
	var hero_rec: Dictionary = DictUtil.dict_of(m.units[hero.name])
	checks["the hero's Damage includes the bleed (dealt_dot = the bleed steps)"] = DictUtil.int_of(hero_row, "dealt_dot") == bleed_by_hero \
		and DictUtil.int_of(hero_row, "dealt") == DictUtil.int_of(hero_rec, "dmg_dealt") + bleed_by_hero
	checks["the victim's Taken includes it"] = DictUtil.int_of(DictUtil.dict_of(m.units[foe.name]), "dot_taken") >= bleed_by_hero and DictUtil.int_of(_row_enemy_taken(m, foe), "taken") >= bleed_by_hero
	checks["the hero's Damage tip splits hits and bleed"] = BattleReport.dealt_tip(hero_row).begins_with("%d from hits, %d from bleed" % [DictUtil.int_of(hero_row, "dealt") - bleed_by_hero, bleed_by_hero])
	checks["the priest's Healed includes the regen"] = DictUtil.int_of(priest_row, "regen") == regen_by_priest and DictUtil.int_of(priest_row, "healed") >= regen_by_priest
	checks["the Healed tip names the regeneration"] = BattleReport.healed_tip(priest_row) == "Includes %d from regeneration." % regen_by_priest
	checks["Healed is a shown column"] = DictUtil.array_of(report["columns"]).has("healed")
	checks["the report is valid"] = BattleReport.problems(report).is_empty()
	s.dispose()
	return TestHelpers.report(TAG, "bleed counts for its source, regen for its granter (rule 4)", checks)

# ------------------------------------------------------------------------------
# STATCRED1-L: a status counts for the unit that applied it, not for the next entry's actor
# ------------------------------------------------------------------------------

static func _entry(actor: Combatant, category: String) -> ActionLogEntry:
	return ActionLogEntry.new(1, 1, category, actor.name, actor.name)

static func _total(m: BattleMetrics, field: String) -> int:
	var n: int = 0
	for unit_name: Variant in m.units:
		n += DictUtil.int_of(DictUtil.dict_of(m.units[unit_name]), field)
	return n

static func _test_status_source() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(_runner(), 41)
	var a: Combatant = s.allies[0] if s.allies.size() > 1 else null
	var b: Combatant = s.allies[1] if s.allies.size() > 1 else null
	var foe: Combatant = s.enemies[0] if not s.enemies.is_empty() else null
	checks["fixture: two heroes and a foe"] = a != null and b != null and foe != null
	if a == null or b == null or foe == null:
		return TestHelpers.report(TAG, "status credit goes to its applier", checks)
	var m := BattleMetrics.new()
	m.quiet = true
	m.attach(s)
	var before_a: Dictionary = DictUtil.dict_of(m.units[a.name]).duplicate()
	var before_b: Dictionary = DictUtil.dict_of(m.units[b.name]).duplicate()
	# A queues a buff on B and a debuff on the foe, then grants itself an active buff (a guard's riposte); B acts next.
	StatusEffects.queue(b, StatusEffects.HASTE, 2, 0, a.name)
	StatusEffects.queue(foe, StatusEffects.SLOW, 1, 0, a.name)
	StatusEffects.grant_now(a, StatusEffects.FORTIFY, 1, 0, a.name)
	s.log_appended.emit(_entry(b, "MOVE"))
	var rec_a: Dictionary = DictUtil.dict_of(m.units[a.name])
	var rec_b: Dictionary = DictUtil.dict_of(m.units[b.name])
	var counts_a: Dictionary = DictUtil.dict_of(DictUtil.dict_of(m.detail[a.name]).get("status_counts", {}))
	var counts_b: Dictionary = DictUtil.dict_of(DictUtil.dict_of(m.detail[b.name]).get("status_counts", {}))
	var buffs_a: int = DictUtil.int_of(rec_a, "buffs_applied") - DictUtil.int_of(before_a, "buffs_applied")
	var debuffs_a: int = DictUtil.int_of(rec_a, "debuffs_applied") - DictUtil.int_of(before_a, "debuffs_applied")
	checks["A's buffs_applied counts both of its buffs (+%d)" % buffs_a] = buffs_a == 2
	checks["A's buff_rounds count their rounds"] = DictUtil.int_of(rec_a, "buff_rounds") - DictUtil.int_of(before_a, "buff_rounds") == 3
	checks["A's debuffs_applied counts its slow (+%d)" % debuffs_a] = debuffs_a == 1
	checks["A's Effects detail lists haste, slow and fortify"] = DictUtil.int_of(counts_a, StatusEffects.HASTE) == 1 \
		and DictUtil.int_of(counts_a, StatusEffects.SLOW) == 1 and DictUtil.int_of(counts_a, StatusEffects.FORTIFY) == 1
	checks["B, who acted next, gets none of them"] = counts_b.is_empty() \
		and DictUtil.int_of(rec_b, "buffs_applied") == DictUtil.int_of(before_b, "buffs_applied") \
		and DictUtil.int_of(rec_b, "debuffs_applied") == DictUtil.int_of(before_b, "debuffs_applied")
	# A status with no recorded source still counts for the entry's actor; a boss-phase grant counts for its enemy
	# recipient (no unit owns every phase), and for no one on the party side.
	var foe_buffs: int = DictUtil.int_of(DictUtil.dict_of(m.units[foe.name]), "buffs_applied")
	StatusEffects.queue(a, StatusEffects.REGEN, 1, 0, "")
	StatusEffects.grant_now(foe, StatusEffects.FORTIFY, 1, 0, CatalogEncounter.PHASE_SOURCE + "t_statcred")
	StatusEffects.grant_now(b, StatusEffects.FORTIFY, 1, 0, CatalogEncounter.PHASE_SOURCE + "t_statcred")
	var total_before: int = _total(m, "buffs_applied")
	s.log_appended.emit(_entry(b, "MOVE"))
	counts_b = DictUtil.dict_of(DictUtil.dict_of(m.detail[b.name]).get("status_counts", {}))
	var counts_foe: Dictionary = DictUtil.dict_of(DictUtil.dict_of(m.detail[foe.name]).get("status_counts", {}))
	checks["an unsourced buff counts for the entry's actor"] = DictUtil.int_of(counts_b, StatusEffects.REGEN) == 1
	checks["a phase grant counts for its enemy recipient"] = DictUtil.int_of(counts_foe, StatusEffects.FORTIFY) == 1 \
		and DictUtil.int_of(DictUtil.dict_of(m.units[foe.name]), "buffs_applied") == foe_buffs + 1
	checks["a phase grant on a hero counts for no one (two new buffs in total)"] = _total(m, "buffs_applied") - total_before == 2 \
		and not counts_b.has(StatusEffects.FORTIFY)
	# A status applied after the round's last entry is credited when the round concludes (before the next roll-over).
	StatusEffects.queue(b, StatusEffects.HASTE, 3, 0, a.name)   # longer than the queued 2 rounds: a visible re-apply
	var a_buffs: int = DictUtil.int_of(DictUtil.dict_of(m.units[a.name]), "buffs_applied")
	s.round_concluded.emit(s.current_round, false)
	checks["a buff after the last entry counts for A at the round's end"] = DictUtil.int_of(DictUtil.dict_of(m.units[a.name]), "buffs_applied") == a_buffs + 1
	var report: Dictionary = BattleReport.build(m, s, "won")
	checks["A's report row shows its four effects"] = DictUtil.int_of(_row(report, a.name), "effects") >= 4
	m.freeze()
	s.dispose()
	return TestHelpers.report(TAG, "status credit goes to its applier, not the next actor (STATCRED1-L)", checks)

static func _row(report: Dictionary, unit_name: String) -> Dictionary:
	for raw: Variant in DictUtil.array_of(report.get("rows", [])):
		var r: Dictionary = DictUtil.dict_of(raw)
		if str(r.get("name", "")) == unit_name:
			return r
	return {}

static func _row_enemy_taken(m: BattleMetrics, foe: Combatant) -> Dictionary:
	var rec: Dictionary = DictUtil.dict_of(m.units[foe.name])
	return {"taken": DictUtil.int_of(rec, "dmg_taken") + DictUtil.int_of(rec, "dot_taken") + DictUtil.int_of(rec, "ff_taken")}

# Rule 8: the same seeded battle with and without a collector ends every round in the same state.
static func _test_no_outcome_change() -> bool:
	var checks: Dictionary = {}
	var runner: SimRunner = _runner()
	var states: Array[Array] = []
	for with_metrics: bool in [false, true]:
		var s: CombatSession = _session(runner, 41)
		var m := BattleMetrics.new()
		m.quiet = true
		if with_metrics:
			m.attach(s)
		var per_round: Array = []
		for _r: int in range(3):
			if not _play_round(s, "balanced"):
				break
			per_round.append(_state(s))
			if s.combat_over or not s.can_advance_round():
				break
			s.advance_round()
		states.append(per_round)
		m.freeze()
		s.dispose()
	checks["rounds were played"] = not states[0].is_empty()
	checks["every round ends identically with the collector attached"] = states[0] == states[1]
	return TestHelpers.report(TAG, "no gameplay effect: identical round outcomes (rule 8)", checks)

static func _state(s: CombatSession) -> Dictionary:
	var out: Dictionary = {}
	for c: Combatant in s.allies + s.enemies:
		out[c.name] = [c.hp, c.ap, s.grid.get_position(c), c.current_stance, c.statuses.duplicate(), c.status_rounds_left.duplicate(), c.is_withdrawn]
	return out

# ------------------------------------------------------------------------------
# Rules 5-6 and 1.4 (pure)
# ------------------------------------------------------------------------------

static func _rowdata(name_text: String, values: Dictionary = {}) -> Dictionary:
	var row: Dictionary = {"ref": name_text, "name": name_text, "guest": false, "state": "standing", "status_counts": {}, "combo_counts": {}}
	for id: String in BattleReport.NUMBERS:
		row[id] = DictUtil.int_of(values, id)
	return row

static func _test_columns() -> bool:
	var checks: Dictionary = {}
	var no_healer: Array[Dictionary] = [_rowdata("A", {"dealt": 30, "taken": 5}), _rowdata("B", {"dealt": 0, "taken": 0, "effects": 2})]
	var cols: Array[String] = BattleReport.columns_for(no_healer)
	checks["a party without a healer has no Healed column"] = not cols.has("healed")
	checks["an all-zero Combos column is hidden"] = not cols.has("combos")
	checks["Damage and Taken always present (even all zero)"] = BattleReport.columns_for([_rowdata("Z")] as Array[Dictionary]) == BattleReport.ALWAYS
	checks["a non-zero Effects column shows, in table order"] = cols == (["name", "dealt", "taken", "effects"] as Array[String])
	var all_cols: Array[String] = BattleReport.columns_for([_rowdata("H", {"healed": 4, "combos": 1, "effects": 1})] as Array[Dictionary])
	checks["all six in table order when all are used"] = all_cols == BattleReport.COLUMNS
	var tip_row: Dictionary = _rowdata("T", {"dealt": 50, "dealt_dot": 8, "ff_dealt": 3, "taken": 20, "taken_dot": 6})
	tip_row["status_counts"] = {StatusEffects.BLEED: 2}
	tip_row["combo_counts"] = {"exploit": 1, "break": 0, "flow": 2}
	checks["Damage tip"] = BattleReport.dealt_tip(tip_row) == "42 from hits, 8 from bleed, poison and burn. Hit allies for 3."
	checks["Taken tip"] = BattleReport.taken_tip(tip_row) == "14 from hits, 6 from effects."
	checks["Effects tip names the status with its count"] = BattleReport.effects_tip(tip_row) == "%s x2" % StatusEffects.display_name(StatusEffects.BLEED)
	checks["Combos tip lists used kinds only"] = BattleReport.combos_tip(tip_row) == "Exploit x1, Flow x2"
	checks["no regen: no Healed tip"] = BattleReport.healed_tip(tip_row).is_empty()
	return TestHelpers.report(TAG, "columns and tips (rule 5)", checks)

static func _test_best_moment() -> bool:
	var checks: Dictionary = {}
	var rows: Array[Dictionary] = [_rowdata("Healer"), _rowdata("Striker")]
	var save: Dictionary = {"ally": "Striker", "hp": 7, "round": 2, "order": 30}
	var big_hit: Dictionary = {"amount": 90, "skill": "Smash", "target": "Ogre", "round": 1, "order": 5}
	var details: Array[Dictionary] = [{"best_save": save}, {"best_hit": big_hit}]
	checks["a save beats a bigger hit"] = BattleReport.best_moment(rows, details) == "Healer healed Striker at 7 HP, and Striker made it through."
	var finish: Dictionary = {"skill": "Opportune Cut", "target": "Goblin", "combo": "exploit", "status": StatusEffects.BLEED, "round": 2, "order": 40}
	var details2: Array[Dictionary] = [{"best_hit": big_hit}, {"best_finish": finish, "best_hit": {"amount": 10, "skill": "Cut", "target": "Goblin", "round": 2, "order": 40}}]
	checks["a combo finish beats a bigger plain hit"] = BattleReport.best_moment(rows, details2) == "Striker finished Goblin with Opportune Cut (exploited %s)." % StatusEffects.display_name(StatusEffects.BLEED)
	var brk: Dictionary = {"skill": "Crush", "target": "Goblin", "combo": "break", "status": "", "round": 1, "order": 3}
	checks["a break finish says 'broke the guard'"] = BattleReport.best_moment([_rowdata("A")] as Array[Dictionary], [{"best_finish": brk}] as Array[Dictionary]) == "A finished Goblin with Crush (broke the guard)."
	var early: Dictionary = {"amount": 40, "skill": "Jab", "target": "Wolf", "round": 1, "order": 2}
	var late: Dictionary = {"amount": 40, "skill": "Hook", "target": "Wolf", "round": 1, "order": 9}
	checks["a tie picks the earlier entry"] = BattleReport.best_moment(rows, [{"best_hit": late}, {"best_hit": early}] as Array[Dictionary]) == "Striker's Jab hit Wolf for 40."
	checks["the bigger hit wins"] = BattleReport.best_moment(rows, [{"best_hit": big_hit}, {"best_hit": early}] as Array[Dictionary]) == "Healer's Smash hit Ogre for 90."
	var low: Dictionary = {"ally": "B", "hp": 3, "round": 3, "order": 50}
	var high: Dictionary = {"ally": "B", "hp": 9, "round": 1, "order": 1}
	checks["of two saves the closer call (lower HP) wins"] = BattleReport.best_moment(rows, [{"best_save": high}, {"best_save": low}] as Array[Dictionary]).begins_with("Striker healed B at 3 HP")
	checks["no hits: no line"] = BattleReport.best_moment(rows, [{}, {}] as Array[Dictionary]).is_empty()
	return TestHelpers.report(TAG, "best moment: save, combo finish, biggest hit, ties (rule 6)", checks)

static func _valid_report() -> Dictionary:
	return {"version": BattleReport.VERSION, "outcome": "won", "rounds": 2, "rows": [_rowdata("A", {"dealt": 10, "taken": 3})], "columns": ["name", "dealt", "taken"], "best": ""}

static func _test_problems() -> bool:
	var checks: Dictionary = {}
	checks["a well-formed report has no problems"] = BattleReport.problems(_valid_report()).is_empty()
	checks["{} has problems"] = not BattleReport.problems({}).is_empty()
	var neg: Dictionary = _valid_report()
	DictUtil.dict_of(DictUtil.array_of(neg["rows"])[0])["dealt"] = -1
	checks["a negative number fails"] = not BattleReport.problems(neg).is_empty()
	var col: Dictionary = _valid_report()
	col["columns"] = ["name", "dealt", "taken", "style"]
	checks["an unknown column fails"] = not BattleReport.problems(col).is_empty()
	var typ: Dictionary = _valid_report()
	typ["best"] = 3
	checks["a wrong type fails"] = not BattleReport.problems(typ).is_empty()
	var missing: Dictionary = _valid_report()
	DictUtil.dict_of(DictUtil.array_of(missing["rows"])[0]).erase("combos")
	checks["a missing row key fails"] = not BattleReport.problems(missing).is_empty()
	return TestHelpers.report(TAG, "report validation (1.4)", checks)

# ------------------------------------------------------------------------------
# 1.5 Plumbing: model and router
# ------------------------------------------------------------------------------

static func _test_model() -> bool:
	var checks: Dictionary = {}
	var good: Dictionary = _valid_report()
	var rows: Array = DictUtil.array_of(good["rows"])
	var guest: Dictionary = _rowdata("Guest", {"dealt": 0, "taken": 4})
	guest["guest"] = true
	guest["state"] = "down"
	rows.append(guest)
	good["best"] = "A's Jab hit Wolf for 10."
	var m := BattleResultsModel.new(null, {}, [], "won", {}, good)
	checks["a valid report is shown"] = m.has_report()
	checks["columns carry labels"] = m.report_columns() == ([{"id": "name", "label": "Hero"}, {"id": "dealt", "label": "Damage"}, {"id": "taken", "label": "Taken"}] as Array[Dictionary])
	var out: Array[Dictionary] = m.report_rows()
	checks["one row per report row"] = out.size() == 2
	checks["numbers as text, '-' for 0"] = out.size() == 2 and str(DictUtil.dict_of(out[0]["cells"])["dealt"]) == "10" and str(DictUtil.dict_of(out[1]["cells"])["dealt"]) == "-"
	checks["guest and down tags on the name"] = out.size() == 2 and str(DictUtil.dict_of(out[1]["cells"])["name"]) == "Guest (guest) (down)" and DictUtil.to_bool(out[1]["guest"])
	checks["tips per cell"] = out.size() == 2 and str(DictUtil.dict_of(out[0]["tips"])["taken"]) == "3 from hits, 0 from effects."
	checks["best moment line"] = m.best_moment() == "Best moment: A's Jab hit Wolf for 10."
	var neg: Dictionary = _valid_report()
	DictUtil.dict_of(DictUtil.array_of(neg["rows"])[0])["taken"] = -5
	checks["a report with a negative number is hidden"] = not BattleResultsModel.new(null, {}, [], "won", {}, neg).has_report()
	var col: Dictionary = _valid_report()
	col["columns"] = ["name", "mystery"]
	var hidden := BattleResultsModel.new(null, {}, [], "lost", {}, col)
	checks["a report with an unknown column is hidden (no rows, no line)"] = not hidden.has_report() and hidden.report_rows().is_empty() and hidden.best_moment().is_empty()
	checks["no report argument: no report"] = not BattleResultsModel.new(null, {}).has_report()
	return TestHelpers.report(TAG, "results model: columns, rows, tips, best line, hidden when invalid (1.5)", checks)

# ------------------------------------------------------------------------------
# Game battles: settle carries the report; states; lost and fled; frozen under Replay; router args
# ------------------------------------------------------------------------------

static func _test_game_battles(tree_parent: Node) -> bool:
	var packed: PackedScene = load(SCENE) as PackedScene
	if packed == null or tree_parent == null:
		return TestHelpers.report_ok(TAG, "region map scene loads (headless without a tree: load only)", packed != null)
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var view: RegionMapView = packed.instantiate() as RegionMapView
	tree_parent.add_child(view)
	var ok: bool = TestHelpers.report_ok(TAG, "new game in the region map scene", view.start_game(WORLD).is_empty() and view.gs != null)
	if view.gs != null:
		ok = _test_fled_states(view) and ok
		ok = _test_lost(view) and ok
		ok = _test_no_battle(view) and ok
	tree_parent.remove_child(view)
	view.free()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

static func _fight(view: RegionMapView, enc: String) -> void:
	view.pending_fight = {"kind": "fight", "source": "encounter", "encounter": enc, "marker": "", "elite": false,
		"from": [view.gs.position.x, view.gs.position.y]}

# Opens the first of the world's own encounters that starts a battle (read only; no specific shipped id).
static func _launch(view: RegionMapView) -> BattleScreen:
	for e: Dictionary in WorldCatalog.own_entries(WORLD, "encounters"):
		_fight(view, str(e.get("id", "")))
		view.launch_battle()
		if view.battle != null:
			return view.battle
	view.pending_fight = {}
	return null

static func _test_fled_states(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var settled: Array[Dictionary] = []
	var on_settled: Callable = func(res: Dictionary) -> void: settled.append(res)
	view.fight_settled.connect(on_settled)
	var b: BattleScreen = _launch(view)
	checks["the fight opens the arena"] = b != null and b.session != null
	if b == null or b.session == null:
		view.fight_settled.disconnect(on_settled)
		return TestHelpers.report(TAG, "game battle: fled, states, frozen", checks)
	checks["no report before the outcome"] = b.report().is_empty()
	var s: CombatSession = b.session
	var _played: bool = _play_round(s, "balanced")
	var heroes: Array[Combatant] = s.encounter.heroes()
	var left: Combatant = heroes[0]
	left.is_withdrawn = true
	var down: Combatant = heroes[1] if heroes.size() > 1 else null
	for c: Combatant in s.allies:
		if c != left:
			c.hp = 0
	s.outcome = ""   # the forced end below decides (BattleBridge.outcome)
	s.combat_over = true
	var outcome: String = b.check()
	checks["no hero standing and one alive: fled (%s)" % outcome] = outcome == "fled"
	view.settle(outcome, true)   # results popup mode: the arena stays for Replay
	var report: Dictionary = DictUtil.dict_of(settled[0].get("report", {})) if settled.size() == 1 else {}
	checks["fight_settled carries a valid report"] = not report.is_empty() and BattleReport.problems(report).is_empty()
	checks["the report says fled"] = str(report.get("outcome", "")) == "fled"
	var rows: Array = DictUtil.array_of(report.get("rows", []))
	checks["one row per party unit, in roster order"] = rows.size() == s.allies.size() and str(DictUtil.dict_of(rows[0])["name"]) == s.allies[0].name
	checks["the escaped hero is 'left'"] = str(_row(report, left.name).get("state", "")) == "left"
	checks["a downed hero is 'down'"] = down == null or str(_row(report, down.name).get("state", "")) == "down"
	checks["rows carry the member id as ref"] = str(_row(report, left.name).get("ref", "")) == s.encounter.hero_key(left)
	var copy: Dictionary = report.duplicate(true)
	var again: Dictionary = b.report()
	checks["report() twice is the same dict"] = is_same(again, b.report()) and again == copy
	var replaying: bool = b.replay()
	for i: int in range(5000):
		if not b.replaying:
			break
		s.auto_play_tick()
	checks["Replay ran to its end"] = replaying and not b.replaying
	checks["the report is unchanged after Replay"] = b.report() == copy
	view.close_settled_battle()
	view.fight_settled.disconnect(on_settled)
	return TestHelpers.report(TAG, "game battle: fled report, left / down states, frozen under Replay (rules 1, 3, 7)", checks)

static func _test_lost(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var settled: Array[Dictionary] = []
	var on_settled: Callable = func(res: Dictionary) -> void: settled.append(res)
	view.fight_settled.connect(on_settled)
	var b: BattleScreen = _launch(view)
	checks["the fight opens the arena"] = b != null and b.session != null
	if b != null and b.session != null:
		for c: Combatant in b.session.allies:
			c.hp = 0
		b.session.combat_over = true
		var allies: int = b.session.allies.size()
		b.finish()
		var report: Dictionary = DictUtil.dict_of(settled[0].get("report", {})) if settled.size() == 1 else {}
		checks["a lost fight carries a full report"] = str(report.get("outcome", "")) == "lost" and DictUtil.array_of(report.get("rows", [])).size() == allies and BattleReport.problems(report).is_empty()
		var all_down: bool = true
		for raw: Variant in DictUtil.array_of(report.get("rows", [])):
			all_down = all_down and str(DictUtil.dict_of(raw).get("state", "")) == "down"
		checks["every row is down"] = all_down
	view.fight_settled.disconnect(on_settled)
	return TestHelpers.report(TAG, "game battle: lost report (rule 1)", checks)

static func _test_no_battle(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var settled: Array[Dictionary] = []
	var on_settled: Callable = func(res: Dictionary) -> void: settled.append(res)
	view.fight_settled.connect(on_settled)
	var own: Array[Dictionary] = WorldCatalog.own_entries(WORLD, "encounters")
	_fight(view, str(own[0].get("id", "")) if not own.is_empty() else "")
	view.settle("won")   # the debug outcome buttons: no battle
	var report: Dictionary = DictUtil.dict_of(settled[0].get("report", {"x": 1})) if settled.size() == 1 else {"x": 1}
	checks["settle without a battle gives report = {}"] = settled.size() == 1 and settled[0].has("report") and report.is_empty()
	CurrentGame.start(view.gs)
	var router := MenuRouter.new(MenuRouter.HUB)
	router.push(MenuRouter.RESULTS, {"award": {}, "advanced": [], "outcome": "won", "report": report})
	var empty_model: BattleResultsModel = router.model() as BattleResultsModel
	checks["the model has no report"] = empty_model != null and not empty_model.has_report()
	var router2 := MenuRouter.new(MenuRouter.HUB)
	router2.push(MenuRouter.RESULTS, {"award": {}, "advanced": [], "outcome": "won"})
	var old_args: BattleResultsModel = router2.model() as BattleResultsModel
	checks["old router args without report still open the popup"] = old_args != null and not old_args.has_report()
	var router3 := MenuRouter.new(MenuRouter.HUB)
	router3.push(MenuRouter.RESULTS, {"award": {}, "advanced": [], "outcome": "won", "report": _valid_report()})
	var with_report: BattleResultsModel = router3.model() as BattleResultsModel
	checks["the router passes a report to the model"] = with_report != null and with_report.has_report()
	CurrentGame.end()
	view.fight_settled.disconnect(on_settled)
	return TestHelpers.report(TAG, "plumbing: no battle, router args with and without report (1.5)", checks)
