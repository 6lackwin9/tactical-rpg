class_name PlaytestTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Playtest Test Suite (PLAYTEST1-L commit 1, acceptance checks 1-13)
# Plan codec, record / replay round trip, redo, divergence, content staleness, validation, take-over and
# branch, variants, seed sweep, notes and trail, release guard, trail report, UI and asset report.
# Recordings and fixture files live under TestHelpers.sandbox("playtest"); the encounter is picked by rule
# (the smallest shipped encounter still running after MIN_ROUNDS preset-played rounds; recordings stop
# there), shipped data is only read. Expectations come from the replay rules, never from shipped numbers.
# ==============================================================================

const TAG: String = "Playtest"
const MIN_ROUNDS: int = 3
const PRESETS: Array[String] = ["balanced", "aggressive", "focus_weakest", "defensive"]

static var SANDBOX: String = TestHelpers.sandbox("playtest")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING PLAYTEST TEST (PLAYTEST1-L)             ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX)
	PlaytestStore.use_root(SANDBOX.path_join("rec"))
	PlaytestStore.force_enabled = 1
	var ok: bool = true
	var enc_id: String = _pick_encounter()
	ok = TestHelpers.report_ok(TAG, "Fixture: the smallest shipped encounter still running after %d rounds (picked by rule)" % MIN_ROUNDS, not enc_id.is_empty(), enc_id) and ok
	ok = _test_codec(enc_id) and ok
	var base: Dictionary = _test_round_trip(enc_id)
	ok = DictUtil.to_bool(base.get("ok", false)) and ok
	var path: String = str(base.get("path", ""))
	if not path.is_empty():
		ok = _test_divergence(path) and ok
		ok = _test_staleness(path) and ok
		ok = _test_take_over(path) and ok
		ok = _test_variant(path) and ok
		ok = _test_sweep(path) and ok
	ok = _test_redo(enc_id) and ok
	ok = _test_validation(enc_id) and ok
	ok = _test_notes_trail(enc_id) and ok
	ok = _test_release_guard(enc_id) and ok
	ok = _test_trail_report() and ok
	ok = _test_ui_asset_report() and ok
	PlaytestStore.force_enabled = -1
	PlaytestStore.use_root(PlaytestStore.USER_ROOT)
	AssetRegistry.resolve_observer = Callable()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# The catalog encounter with the fewest units (then the lowest id) that is still running after MIN_ROUNDS
# preset-played rounds; every recorded battle below stops there, which keeps the suite fast.
static func _pick_encounter() -> String:
	var sized: Array[Dictionary] = []
	for id: String in ContentCatalog.ids("encounters"):
		if EncounterSource.exists(id):
			var enc: CombatEncounter = EncounterSource.create(id)
			enc.build()
			sized.append({"id": id, "units": enc.allies.size() + enc.enemies.size()})
	sized.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		return DictUtil.to_int(a["units"]) < DictUtil.to_int(b["units"]) if a["units"] != b["units"] else str(a["id"]) < str(b["id"]))
	for entry: Dictionary in sized:
		var session := CombatSession.new(EncounterSource.create(str(entry["id"])))
		session.start()
		var running: bool = _play(session, MIN_ROUNDS) >= MIN_ROUNDS and not session.combat_over and session.can_advance_round()
		session.dispose()
		if running:
			return str(entry["id"])
	return ""

# Plays the live session with the PRESETS cycle until the battle ends or `max_round` concluded; returns rounds played.
static func _play(session: CombatSession, max_round: int) -> int:
	var played: int = 0
	while true:
		if not _play_round(session, PRESETS[(session.current_round - 1) % PRESETS.size()]):
			break
		played = session.current_round
		if session.combat_over or not session.can_advance_round() or session.current_round >= max_round:
			break
		session.advance_round()
	return played

static func _play_round(session: CombatSession, preset: String) -> bool:
	session.plan_apply_preset_all(preset)
	if not session.confirm_plan(false):
		return false
	TestHelpers.resolve_until_replay(session, PlaytestReplayer.RESOLVE_STEPS)
	return session.phase == CombatSession.Phase.REPLAY

# A recorded battle (attached before start) played with the PRESETS cycle; the recorder flushed.
static func _recorded_battle(enc_id: String, max_round: int = MIN_ROUNDS) -> PlaytestRecorder:
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	var session := CombatSession.new(EncounterSource.create(enc_id))
	rec.attach(session)
	session.start()
	_play(session, max_round)
	rec.flush()
	rec.detach()
	return rec

static func _load(path: String) -> Dictionary:
	return PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(path).get("data", {})))

static func _write(name: String, data: Dictionary) -> String:
	var path: String = SANDBOX.path_join(name)
	PlaytestStore.save_json(path, data)
	return path

static func _lines_with(lines: Variant, needle: String) -> int:
	var n: int = 0
	for line: String in JsonUtil.strings(lines):
		if line.begins_with(needle):
			n += 1
	return n

# ------------------------------------------------------------------------------
# 1. Codec
# ------------------------------------------------------------------------------

static func _test_codec(enc_id: String) -> bool:
	var c: Dictionary = {}
	var props: Array[String] = []
	var step_script: Script = PlanStep.new().get_script()
	for p: Dictionary in step_script.get_script_property_list():
		if DictUtil.to_int(p.get("usage", 0)) & PROPERTY_USAGE_SCRIPT_VARIABLE:
			props.append(str(p["name"]))
	var uncovered: Array[String] = []
	for p: String in props:
		if not PlanCodec.FIELDS.has(p):
			uncovered.append(p)
	c["every PlanStep property has a codec entry (%s)" % ", ".join(uncovered)] = uncovered.is_empty() and not props.is_empty()
	c["no codec entry without a PlanStep property"] = PlanCodec.FIELDS.size() == props.size()
	if enc_id.is_empty():
		return TestHelpers.report(TAG, "1. Codec covers PlanStep; encode / decode / encode is stable per mode", c)
	var session := CombatSession.new(EncounterSource.create(enc_id))
	session.start()
	var samples: Dictionary = _mode_samples(session)
	var covered: Array[String] = []
	for mode: Variant in samples:
		covered.append(str(mode))
	for need: String in ["ENGAGE", "MOVE", "GOTO", "WITHDRAW", "SELF"]:
		c["mode %s built" % need] = samples.has(need)
	var roster: Array = PlanCodec.roster(session)
	for mode: Variant in samples:
		var pair: Array = DictUtil.array_of(samples[mode])
		var actor: Combatant = pair[0]
		var step: PlanStep = pair[1]
		var first: Dictionary = PlanCodec.encode_step(session, step)
		var via_json: Dictionary = DictUtil.dict_of(JsonUtil.normalize(JSON.parse_string(JSON.stringify(first))))
		var errors: Array[String] = []
		var warnings: Array[String] = []
		var decoded: PlanStep = PlanCodec.decode_step(session, actor, via_json, roster, "round 1, unit x, step 1", errors, warnings)
		c["%s decodes (%s)" % [str(mode), "; ".join(errors)]] = decoded != null and errors.is_empty() and warnings.is_empty()
		if decoded == null:
			continue
		c["%s encode, decode, encode equal" % str(mode)] = JsonUtil.deep_equal(first, PlanCodec.encode_step(session, decoded))
		c["%s same blocks and skills" % str(mode)] = _entries_text(session, actor, step) == _entries_text(session, actor, decoded)
	session.dispose()
	return TestHelpers.report(TAG, "1. Codec covers PlanStep; encode / decode / encode is stable per mode (%s)" % ", ".join(covered), c)

# mode name -> [actor, step]: planner options of every ally (target and cell options), hand-built MOVE / WITHDRAW.
static func _mode_samples(session: CombatSession) -> Dictionary:
	var out: Dictionary = {}
	for a: Combatant in session.allies:
		var options: Array[PlanOption] = session.plan.options(a)
		var pos: Vector2i = session.grid.get_position(a)
		for d: Vector2i in CombatPlan.DIRS:
			options.append_array(session.plan.cell_options(a, pos + d * 2))
		for o: PlanOption in options:
			var mode: String = str(PlanStep.Mode.keys()[o.step.mode])
			if o.valid and not out.has(mode):
				out[mode] = [a, o.step]
	var first: Combatant = session.allies[0]
	if not out.has("MOVE"):
		out["MOVE"] = [first, PlanStep.move(SkillCatalog.lowest(SkillDef.Kind.MOVE), Vector2i(1, 0), 1)]
	if not out.has("WITHDRAW"):
		out["WITHDRAW"] = [first, PlanStep.withdraw()]
	return out

static func _entries_text(session: CombatSession, actor: Combatant, step: PlanStep) -> String:
	var plan := CombatPlan.new(session.encounter.plan_context())
	var steps: Array[PlanStep] = [step]
	plan.set_steps(actor, steps)
	var parts: Array[String] = []
	for e: PlanEntry in plan.entries_for(actor):
		parts.append("%d-%d:%s" % [e.start_block, e.end_block(), e.skill.id if e.skill != null else ""])
	return ",".join(parts)

# ------------------------------------------------------------------------------
# 2. Round trip
# ------------------------------------------------------------------------------

static func _test_round_trip(enc_id: String) -> Dictionary:
	var c: Dictionary = {}
	if enc_id.is_empty():
		return {"ok": TestHelpers.report(TAG, "2. Round trip", {"fixture": false}), "path": ""}
	var rec: PlaytestRecorder = _recorded_battle(enc_id)
	var data: Dictionary = _load(rec.path)
	var b: Dictionary = PlaytestRecord.battle_at(data, 0)
	var rounds: Array = DictUtil.array_of(b.get("rounds", []))
	c["file written and valid"] = FileAccess.file_exists(rec.path) and PlaytestRecord.errors(data).is_empty() and PlaytestRecord.battle_errors(b).is_empty()
	c["one battle, %d rounds" % MIN_ROUNDS] = DictUtil.array_of(data["battles"]).size() == 1 and rounds.size() == MIN_ROUNDS
	c["roster allies then enemies"] = DictUtil.array_of(b["roster"]).size() > 1 and str(DictUtil.dict_of(DictUtil.dict_of(DictUtil.array_of(b["roster"])[0]).get("ref", {})).get("side", "")) == "ally"
	c["plans recorded each round"] = rounds.all(func(r: Variant) -> bool: return not DictUtil.array_of(DictUtil.dict_of(r).get("plans", [])).is_empty())
	c["edits counted (preset_all)"] = rounds.all(func(r: Variant) -> bool: return DictUtil.to_int(DictUtil.dict_of(r).get("edits", 0)) >= 1)
	c["end set"] = str(DictUtil.dict_of(b["end"]).get("result", "")) != "abandoned" or DictUtil.to_int(DictUtil.dict_of(b["end"]).get("rounds", 0)) == rounds.size()
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(b)
	c["replay starts"] = rp.run()
	var all_match: bool = rp.rounds.size() == rounds.size()
	for e: Dictionary in rp.rounds:
		all_match = all_match and str(e["result"]) == "match"
	c["every round match (%s)" % str(rp.first_problem().get("detail", ""))] = all_match
	c["same end"] = JsonUtil.deep_equal(rp.end, b["end"])
	var tool: Dictionary = PlaytestReplayer.run_file(rec.path)
	c["tool exit 0"] = DictUtil.to_int(tool["exit"]) == 0
	var ok: bool = TestHelpers.report(TAG, "2. A preset-played battle replays with every round match and the same end", c)
	return {"ok": ok, "path": rec.path if ok else ""}

# ------------------------------------------------------------------------------
# 3. Redo
# ------------------------------------------------------------------------------

static func _test_redo(enc_id: String) -> bool:
	var c: Dictionary = {}
	if enc_id.is_empty():
		return TestHelpers.report(TAG, "3. Redo", {"fixture": false})
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	var session := CombatSession.new(EncounterSource.create(enc_id))
	rec.attach(session)
	session.start()
	c["round 1 played"] = _play_round(session, PRESETS[0]) and session.can_advance_round()
	session.advance_round()
	c["round 2 first attempt resolved"] = _play_round(session, PRESETS[1])
	var first_state: String = str(rec.round_entry(2).get("rng_state", ""))
	var first_plans: Array = DictUtil.array_of(rec.round_entry(2).get("plans", [])).duplicate(true)
	session.reset_round()
	c["round 2 second attempt resolved"] = _play_round(session, PRESETS[3])
	rec.flush()
	rec.detach()
	session.dispose()
	var data: Dictionary = _load(rec.path)
	var b: Dictionary = PlaytestRecord.battle_at(data, 0)
	var r2: Dictionary = {}
	for raw: Variant in DictUtil.array_of(b.get("rounds", [])):
		if DictUtil.to_int(DictUtil.dict_of(raw).get("round", 0)) == 2:
			r2 = DictUtil.dict_of(raw)
	var discarded: Array = DictUtil.array_of(r2.get("discarded", []))
	c["one battle entry"] = DictUtil.array_of(data["battles"]).size() == 1 and DictUtil.array_of(b["rounds"]).size() == 2
	c["attempts == 2"] = DictUtil.to_int(r2.get("attempts", 0)) == 2
	c["one discarded plan, the first attempt's"] = discarded.size() == 1 and JsonUtil.deep_equal(discarded[0], first_plans)
	c["rng_state differs from the first attempt's"] = not first_state.is_empty() and str(r2.get("rng_state", "")) != first_state
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(b)
	rp.run()
	c["replays as match (%s)" % str(rp.first_problem().get("detail", ""))] = rp.all_match() and rp.result_of(1) == "match" and rp.result_of(2) == "match"
	return TestHelpers.report(TAG, "3. A redone round records attempts, the discarded plan and its new RNG state, and replays", c)

# ------------------------------------------------------------------------------
# 4. Divergence
# ------------------------------------------------------------------------------

static func _test_divergence(path: String) -> bool:
	var c: Dictionary = {}
	var data: Dictionary = _load(path)
	var b: Dictionary = PlaytestRecord.battle_at(data, 0)
	var rounds: Array = DictUtil.array_of(b["rounds"])
	var r1: Dictionary = DictUtil.dict_of(rounds[0])
	var unit0: Dictionary = DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(r1["outcome"])["units"])[0])
	unit0["hp"] = DictUtil.to_int(unit0["hp"]) + 1
	data["id"] = "diverge"
	var bad_path: String = _write("diverge.json", data)
	var tool: Dictionary = PlaytestReplayer.run_file(bad_path)
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(PlaytestRecord.battle_at(_load(bad_path), 0))
	rp.run()
	var problem: Dictionary = rp.first_problem()
	var diff: Dictionary = DictUtil.dict_of(problem.get("diff", {}))
	c["diverge at round 1"] = rp.result_of(1) == "diverge" and DictUtil.to_int(problem.get("round", 0)) == 1
	c["names the unit and field"] = JsonUtil.deep_equal(diff.get("ref", null), unit0["ref"]) and str(diff.get("field", "")) == "hp"
	c["expected and actual"] = DictUtil.to_int(diff.get("expected", 0)) == DictUtil.to_int(unit0["hp"]) and DictUtil.to_int(diff.get("actual", 0)) == DictUtil.to_int(unit0["hp"]) - 1
	var later: bool = true
	for k: int in range(2, rounds.size() + 1):
		later = later and rp.result_of(k) == "unverified"
	c["later rounds unverified"] = later and rounds.size() >= 2
	c["tool exit 1"] = DictUtil.to_int(tool["exit"]) == 1
	return TestHelpers.report(TAG, "4. A changed recorded hp is reported as diverge at that round, unit and field", c)

# ------------------------------------------------------------------------------
# 5. Staleness
# ------------------------------------------------------------------------------

static func _test_staleness(path: String) -> bool:
	var c: Dictionary = {}
	var root: String = SANDBOX.path_join("fp")
	DirAccess.make_dir_recursive_absolute(root.path_join("sub"))
	_text(root.path_join("a.json"), "{\"a\": 1}")
	_text(root.path_join("sub/b.json"), "{\"b\": 1}")
	_text(root.path_join("c.txt"), "not json")
	var before: Dictionary = ContentFingerprint.compute(root)
	_text(root.path_join("sub/b.json"), "{\"b\": 2}")
	var after: Dictionary = ContentFingerprint.compute(root)
	var d: Dictionary = ContentFingerprint.diff(before, after)
	c["only *.json, recursively"] = DictUtil.dict_of(before["files"]).size() == 2
	c["total changes"] = str(before["total"]) != str(after["total"]) and str(before["total"]).length() == ContentFingerprint.CUT
	c["exactly that path changed"] = JsonUtil.deep_equal(d["changed"], [root.path_join("sub/b.json")]) and DictUtil.array_of(d["added"]).is_empty() and DictUtil.array_of(d["removed"]).is_empty()
	c["stale"] = DictUtil.to_bool(d["stale"])
	c["same content not stale"] = not DictUtil.to_bool(ContentFingerprint.diff(after, ContentFingerprint.compute(root))["stale"])
	var tool: Dictionary = PlaytestReplayer.run_file(path, {"content_root": root})
	c["replay still runs (warns, exit 0)"] = DictUtil.to_int(tool["exit"]) == 0 and _lines_with(tool["lines"], "STALE") > 0
	return TestHelpers.report(TAG, "5. A changed sandbox JSON file changes the total and is listed as changed; replay still runs", c)

static func _text(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f != null:
		f.store_string(text)
		f.close()

# ------------------------------------------------------------------------------
# 6. Validation
# ------------------------------------------------------------------------------

static func _test_validation(enc_id: String) -> bool:
	var c: Dictionary = {}
	var newer: Dictionary = PlaytestRecord.create("newer")
	newer["version"] = 2
	newer["content"] = {"total": "x"}
	c["version 2 refused"] = TestHelpers.any_contains(PlaytestRecord.errors(newer), "Newer recording")
	c["version 2 file: tool exit 2"] = DictUtil.to_int(PlaytestReplayer.run_file(_write("newer.json", newer))["exit"]) == 2
	var old: Dictionary = newer.duplicate(true)
	old["version"] = 0
	c["version 0 refused"] = not PlaytestRecord.errors(old).is_empty()
	if not enc_id.is_empty():
		var session := CombatSession.new(EncounterSource.create(enc_id))
		session.start()
		var roster: Array = PlanCodec.roster(session)
		var actor_ref: Variant = PlanCodec.unit_ref(session, session.allies[0])
		var bad_mode: Array = [{"unit": actor_ref, "steps": [{"mode": "TELEPORT", "skill": "", "target": null, "dir": [0, 0], "tiles": 1, "cell": [-1, -1]}]}]
		var mode_errors: Array[String] = JsonUtil.strings(PlanCodec.decode_plans(session, bad_mode, roster, 3)["errors"])
		c["unknown mode refused with round, unit and step"] = mode_errors.size() == 1 and mode_errors[0].contains("round 3") and mode_errors[0].contains("unit ally 0") and mode_errors[0].contains("step 1") and mode_errors[0].contains("unknown mode")
		var far: Dictionary = {"side": "enemy", "i": session.enemies.size() + 5, "name": "Nobody"}
		var bad_ref: Array = [{"unit": actor_ref, "steps": [
			{"mode": "SELF", "skill": "", "target": null, "dir": [0, 0], "tiles": 1, "cell": [-1, -1]},
			{"mode": "ENGAGE", "skill": "", "target": far, "dir": [0, 0], "tiles": 1, "cell": [-1, -1]}]}]
		var ref_errors: Array[String] = JsonUtil.strings(PlanCodec.decode_plans(session, bad_ref, roster, 2)["errors"])
		c["ref index out of range refused with round, unit and step"] = ref_errors.size() == 1 and ref_errors[0].contains("round 2") and ref_errors[0].contains("unit ally 0") and ref_errors[0].contains("step 2") and ref_errors[0].contains("out of range")
		var renamed: Dictionary = DictUtil.dict_of(actor_ref).duplicate()
		renamed["name"] = "Someone Else"
		var warn_only: Dictionary = PlanCodec.decode_plans(session, [{"unit": renamed, "steps": []}], roster, 1)
		c["a name mismatch alone only warns"] = DictUtil.to_bool(warn_only["ok"]) and DictUtil.array_of(warn_only["warnings"]).size() == 1
		session.dispose()
	var minimal: Dictionary = {"format": "playtest", "version": 1, "id": "minimal", "content": {"total": "x"},
		"battles": [{"encounter": enc_id, "seed": 7, "roster": [{"ref": {"side": "ally", "i": 0, "name": "A"}}], "rounds": [{"round": 1, "rng_state": "5"}]}]}
	var norm: Dictionary = PlaytestRecord.normalize(minimal)
	var nb: Dictionary = PlaytestRecord.battle_at(norm, 0)
	var nr: Dictionary = DictUtil.dict_of(DictUtil.array_of(nb["rounds"])[0])
	c["missing recording fields take defaults"] = str(norm["origin"]) == "user" and norm["parent"] == null and DictUtil.array_of(norm["trail"]).is_empty() and DictUtil.to_int(norm["trail_dropped"]) == 0 and DictUtil.array_of(norm["notes"]).is_empty() and str(norm["scenario"]).is_empty()
	c["missing battle fields take defaults"] = str(nb["seed_source"]) == "encounter" and str(DictUtil.dict_of(nb["end"])["result"]) == "abandoned"
	c["missing round fields take defaults"] = DictUtil.to_int(nr["attempts"]) == 1 and DictUtil.array_of(nr["discarded"]).is_empty() and DictUtil.array_of(nr["plans"]).is_empty() and DictUtil.dict_of(nr["outcome"]).is_empty()
	c["unknown keys kept"] = PlaytestRecord.normalize({"format": "playtest", "extra": 3}).has("extra")
	var scenario: Dictionary = {"format": PlaytestRecord.SCENARIO_FORMAT, "version": 1, "id": "qa_scenario", "title": "t", "why": "w", "encounter": enc_id, "seed": 11}
	c["a scenario with encounter and seed is valid"] = enc_id.is_empty() or PlaytestRecord.scenario_errors(scenario).is_empty()
	scenario.erase("seed")
	c["a scenario without a seed is refused"] = TestHelpers.any_contains(PlaytestRecord.scenario_errors(scenario), "seed")
	var big: String = "9007199254740993"   # 2^53 + 1: not representable as a JSON double
	var low: String = "-9223372036854775807"
	var big_data: Dictionary = minimal.duplicate(true)
	var big_round: Dictionary = DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(DictUtil.array_of(big_data["battles"])[0])["rounds"])[0])
	big_round["rng_state"] = big
	big_round["alt_state"] = low
	var loaded: Dictionary = PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(_write("big.json", big_data))["data"]))
	var lr: Dictionary = DictUtil.dict_of(DictUtil.array_of(PlaytestRecord.battle_at(loaded, 0)["rounds"])[0])
	c["rng_state above 2^53 survives save and load"] = str(lr["rng_state"]) == big and str(lr["rng_state"]).to_int() == 9007199254740993 and str(lr["alt_state"]).to_int() == -9223372036854775807
	return TestHelpers.report(TAG, "6. Validation: newer version, unknown mode, ref out of range refused; defaults; big RNG states", c)

# ------------------------------------------------------------------------------
# 7. Take-over and branch
# ------------------------------------------------------------------------------

static func _test_take_over(path: String) -> bool:
	var c: Dictionary = {}
	var data: Dictionary = _load(path)
	var b: Dictionary = PlaytestRecord.battle_at(data, 0)
	var rounds: Array = DictUtil.array_of(b["rounds"])
	var n: int = mini(MIN_ROUNDS, rounds.size())
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(b)
	rp.take_over = n
	c["replay starts"] = rp.run()
	var s: CombatSession = rp.session
	c["current_round == N"] = s.current_round == n
	c["phase PLANNING"] = s.phase == CombatSession.Phase.PLANNING and s.is_planning()
	var before_ok: bool = true
	for k: int in range(1, n):
		before_ok = before_ok and rp.result_of(k) == "match"
	c["rounds 1..N-1 match"] = before_ok
	var expected: Array = DictUtil.array_of(DictUtil.dict_of(DictUtil.dict_of(rounds[n - 2])["outcome"])["units"])
	var actual: Array = DictUtil.array_of(RoundOutcome.capture(s)["units"])
	var same: bool = expected.size() == actual.size()
	for i: int in range(mini(expected.size(), actual.size())):
		for field: String in ["hp", "pos", "stance", "withdrawn"]:   # AP and statuses roll over into round N
			same = same and JsonUtil.deep_equal(DictUtil.dict_of(expected[i])[field], DictUtil.dict_of(actual[i])[field])
	c["unit state equals round N-1's outcome (hp, pos, stance, withdrawn)"] = same
	var branch: PlaytestRecorder = PlaytestRecorder.branch(data, 0, n)
	branch.attach(s)
	var bb: Dictionary = branch.battle()
	var copied: Array = DictUtil.array_of(bb["rounds"])
	c["parent"] = JsonUtil.deep_equal(branch.data["parent"], {"id": str(data["id"]), "battle": 0, "round": n})
	c["seed_source branch"] = str(bb["seed_source"]) == "branch"
	var verbatim: bool = copied.size() == n
	for k: int in range(n - 1):
		verbatim = verbatim and JsonUtil.deep_equal(copied[k], rounds[k])
	c["rounds 1..N-1 copied verbatim, round N opened"] = verbatim and DictUtil.to_int(DictUtil.dict_of(copied[n - 1])["round"]) == n
	c["round N played live"] = _play_round(s, PRESETS[(n - 1) % PRESETS.size()]) and not DictUtil.dict_of(branch.round_entry(n).get("outcome", {})).is_empty()
	branch.flush()
	branch.detach()
	var again: PlaytestReplayer = PlaytestReplayer.for_battle(PlaytestRecord.battle_at(_load(branch.path), 0))
	again.run()
	c["the branch replays as match"] = again.all_match() and again.result_of(n) == "match"
	return TestHelpers.report(TAG, "7. Take-over replays to round N (planning) and branches with parent and rounds 1..N-1", c)

# ------------------------------------------------------------------------------
# 8. Variant
# ------------------------------------------------------------------------------

static func _test_variant(path: String) -> bool:
	var c: Dictionary = {}
	var data: Dictionary = _load(path)
	var b: Dictionary = PlaytestRecord.battle_at(data, 0)
	var r2: Dictionary = DictUtil.dict_of(DictUtil.array_of(b["rounds"])[1])
	var same: Dictionary = {"format": PlaytestRecord.VARIANT_FORMAT, "version": 1, "battle": 0, "round": 2, "plans": r2["plans"]}
	var tool_same: Dictionary = PlaytestReplayer.run_file(path, {"variant": _write("same.variant.json", same), "then": "recorded"})
	c["identical patch: no DIFF line"] = DictUtil.to_int(tool_same["exit"]) == 0 and _lines_with(tool_same["lines"], "DIFF ") == 0
	var acted: bool = false
	for raw: Variant in DictUtil.array_of(r2["plans"]):
		acted = acted or not DictUtil.array_of(DictUtil.dict_of(raw).get("steps", [])).is_empty()
	c["fixture: the party acted in recorded round 2"] = acted
	var changed: Dictionary = same.duplicate(true)
	changed["plans"] = []   # the party idles in round 2
	var out_path: String = SANDBOX.path_join("repro/variant-out.json")
	var tool_changed: Dictionary = PlaytestReplayer.run_file(path, {"variant": _write("changed.variant.json", changed), "write": out_path})
	c["changed patch: at least one DIFF line"] = _lines_with(tool_changed["lines"], "DIFF ") >= 1
	var rounds_before: bool = false
	for raw: Variant in DictUtil.array_of(DictUtil.dict_of(DictUtil.array_of(DictUtil.dict_of(tool_changed["report"])["battles"])[0])["rounds"]):
		var e: Dictionary = DictUtil.dict_of(raw)
		if DictUtil.to_int(e["round"]) == 1:
			rounds_before = DictUtil.array_of(e.get("diffs", [1])).is_empty()
	c["rounds before N unchanged"] = rounds_before
	var written: Dictionary = _load(out_path)
	c["--write saves an agent recording with a parent"] = FileAccess.file_exists(out_path) and str(written.get("origin", "")) == "agent" and DictUtil.to_int(DictUtil.dict_of(written.get("parent", {})).get("round", 0)) == 2
	var wb: Dictionary = PlaytestRecord.battle_at(written, 0)
	c["the written variant replays as match"] = DictUtil.to_int(PlaytestReplayer.run_file(out_path)["exit"]) == 0 and DictUtil.array_of(wb.get("rounds", [])).size() == 2 and str(wb["seed_source"]) == "tool"
	return TestHelpers.report(TAG, "8. Variant: an identical patch gives no DIFF, a changed one does and leaves earlier rounds", c)

# ------------------------------------------------------------------------------
# 9. Sweep
# ------------------------------------------------------------------------------

static func _test_sweep(path: String) -> bool:
	var c: Dictionary = {}
	var b: Dictionary = PlaytestRecord.battle_at(_load(path), 0)
	var s: int = DictUtil.to_int(b["seed"])
	var parsed: Dictionary = PlaytestSweep.parse_seeds("%d,%d..%d" % [s, s + 1, s + 2])
	c["seed list parsed"] = DictUtil.array_of(parsed["seeds"]).size() == 3 and str(parsed["error"]).is_empty()
	var seeds: Array[int] = [s, s + 1]
	c["more than %d seeds refused" % PlaytestSweep.MAX_SEEDS] = not str(PlaytestSweep.parse_seeds("1..%d" % (PlaytestSweep.MAX_SEEDS + 1))["error"]).is_empty()
	var sweep: Dictionary = PlaytestSweep.run(b, seeds)
	var plain: PlaytestReplayer = PlaytestReplayer.for_battle(b)
	plain.run()
	c["recorded seed row equals the plain replay's end"] = JsonUtil.deep_equal(DictUtil.dict_of(sweep["recorded_row"])["end"], plain.end)
	var counts: Dictionary = DictUtil.dict_of(DictUtil.dict_of(sweep["summary"])["counts"])
	var total: int = 0
	for key: Variant in counts:
		total += DictUtil.to_int(counts[key])
	c["counts add up to the seed count"] = total == seeds.size() and DictUtil.array_of(sweep["rows"]).size() == seeds.size()
	var one: Array[int] = [s]
	c["one seed: no outlier"] = DictUtil.array_of(DictUtil.dict_of(PlaytestSweep.run(b, one)["summary"])["outliers"]).is_empty()
	var tool: Dictionary = PlaytestReplayer.run_file(path, {"seeds": str(s + 2)})
	c["tool sweep exit 0 with a summary line"] = DictUtil.to_int(tool["exit"]) == 0 and _lines_with(tool["lines"], "battle 0 sweep summary") == 1
	return TestHelpers.report(TAG, "9. Seed sweep: recorded seed row, counts, single-seed outliers", c)

# ------------------------------------------------------------------------------
# 10. Notes and trail
# ------------------------------------------------------------------------------

static func _test_notes_trail(enc_id: String) -> bool:
	var c: Dictionary = {}
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	var session: CombatSession = null
	if not enc_id.is_empty():
		session = CombatSession.new(EncounterSource.create(enc_id))
		rec.attach(session)
		session.start()
	var ctl: Dictionary = PlaytestRecorder.describe_control("/root/Arena/Ui/Bar/Box/Confirm", "Button", "Confirm the plan " + "x".repeat(60), "t".repeat(80))
	var ctx: Dictionary = {"scene": "res://scenes/combat_arena.tscn", "mouse": Vector2(10.4, 20.6), "viewport": Vector2i(1280, 720), "ctl": ctl,
		"tags": ["ui", "bug"], "text": "The confirm button is hard to find.", "image": ""}
	var note: Dictionary = rec.note_stamp(ctx)
	var missing: Array[String] = []
	for key: String in PlaytestRecord.note_defaults():
		if not note.has(key):
			missing.append(key)
	c["every G field (%s)" % ", ".join(missing)] = missing.is_empty() and note.size() == PlaytestRecord.note_defaults().size()
	c["ctl: last 4 path segments, 40 / 60 chars"] = str(ctl["path"]) == "Ui/Bar/Box/Confirm" and str(ctl["text"]).length() == 40 and str(ctl["tip"]).length() == 60
	c["mouse and viewport pairs"] = JsonUtil.deep_equal(note["mouse"], [10, 21]) and JsonUtil.deep_equal(note["viewport"], [1280, 720])
	if session != null:
		c["battle, round, phase from the session"] = DictUtil.to_int(note["battle"]) == 0 and DictUtil.to_int(note["round"]) == 1 and str(note["phase"]) == "planning"
	var added: Dictionary = rec.add_note(ctx)
	c["empty image allowed"] = DictUtil.to_bool(added["ok"]) and DictUtil.array_of(rec.data["notes"]).size() == 1
	c["note event in the trail"] = str(DictUtil.dict_of(DictUtil.array_of(rec.data["trail"]).back()).get("type", "")) == "note"
	var empty_text: Dictionary = ctx.duplicate()
	empty_text["text"] = ""
	c["empty text refused"] = not DictUtil.to_bool(rec.add_note(empty_text)["ok"])
	c["n unique on save"] = PlaytestRecord.errors(rec.data).is_empty()
	var keys: PlaytestRecorder = PlaytestRecorder.open("user")
	c["key skipped while a text field has focus"] = not keys.add_key("A", "", true) and DictUtil.array_of(keys.data["trail"]).is_empty()
	c["key recorded otherwise"] = keys.add_key("A", "ctrl+", false) and DictUtil.array_of(keys.data["trail"]).size() == 1
	c["wheel clicks skipped"] = not keys.add_click("wheel_up", Vector2.ZERO, null, null, false)
	var full: Array = []
	full.resize(PlaytestRecord.TRAIL_MAX - 1)
	full.fill({"t": 0, "type": "hover"})
	keys.data["trail"] = full
	keys.data["trail_dropped"] = 0
	var last_in: bool = keys.add_trail("view", {"round": 1, "block": 0})
	var over: bool = keys.add_trail("view", {"round": 1, "block": 1})
	c["trail stops at %d events" % PlaytestRecord.TRAIL_MAX] = last_in and not over and DictUtil.array_of(keys.data["trail"]).size() == PlaytestRecord.TRAIL_MAX
	c["trail_dropped counts"] = DictUtil.to_int(keys.data["trail_dropped"]) == 1
	rec.detach()
	if session != null:
		session.dispose()
	return TestHelpers.report(TAG, "10. Note stamp fills every field; trail cap and dropped count; keys skipped in text fields", c)

# ------------------------------------------------------------------------------
# 11. Release guard
# ------------------------------------------------------------------------------

static func _test_release_guard(enc_id: String) -> bool:
	var c: Dictionary = {}
	PlaytestStore.force_enabled = 0
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	if not enc_id.is_empty():
		var session := CombatSession.new(EncounterSource.create(enc_id))
		rec.attach(session)
		session.start()
		_play(session, 1)
		rec.detach()
		session.dispose()
	c["a concluded round wrote no file"] = not FileAccess.file_exists(rec.path)
	c["flush refused"] = rec.flush() == ERR_UNAVAILABLE and not FileAccess.file_exists(rec.path)
	rec.add_note({"text": "release note"})
	c["a note wrote no file"] = not FileAccess.file_exists(rec.path)
	rec.start_hooks()
	c["no hooks"] = not rec.hooks_active() and not AssetRegistry.resolve_observer.is_valid()
	PlaytestStore.force_enabled = 1
	return TestHelpers.report(TAG, "11. With the debug check forced off the recorder writes nothing", c)

# ------------------------------------------------------------------------------
# 12. Trail report (clicks, hovers, errors) and the Logger hook
# ------------------------------------------------------------------------------

static func _ctl(path: String, cls: String) -> Dictionary:
	return {"path": path, "cls": cls, "text": "", "tip": ""}

static func _counts(data: Dictionary) -> Dictionary:
	return DictUtil.dict_of(TrailReport.build(data)["counts"])

static func _test_trail_report() -> bool:
	var c: Dictionary = {}
	var go: Dictionary = _ctl("Root/Bar/Go", "Button")
	var trail: Array = [
		{"t": 0, "type": "scene", "path": "res://scenes/combat_arena.tscn", "leaked": 0},
		{"t": 2000, "type": "click", "button": "left", "pos": [5, 5], "ctl": _ctl("Root/Info/Label", "Label"), "handled": false},
		{"t": 4000, "type": "click", "button": "left", "pos": [9, 9], "ctl": _ctl("Root/Bar/Ok", "Button"), "handled": true},
		{"t": 4050, "type": "button", "ctl": _ctl("Root/Bar/Ok", "Button")},
	]
	for k: int in range(4):
		trail.append({"t": 8000 + k * 250, "type": "click", "button": "left", "pos": [1, 1], "ctl": _ctl("Root/Bar/Plus", "Button"), "handled": true})
		trail.append({"t": 8020 + k * 250, "type": "button", "ctl": _ctl("Root/Bar/Plus", "Button")})
	trail.append({"t": 12000, "type": "hover", "ctl": _ctl("Root/Bar/Help", "Button"), "ms": 3500})
	trail.append({"t": 15000, "type": "click", "button": "left", "pos": [3, 3], "ctl": go, "handled": true})
	trail.append({"t": 15100, "type": "error", "level": "script", "msg": "Invalid access", "where": "res://x.gd:3"})
	var data: Dictionary = PlaytestRecord.create("trail")
	data["trail"] = trail
	data["notes"] = [{"n": 1, "t": 1000, "text": "x"}]
	var report: Dictionary = TrailReport.build(data)
	var counts: Dictionary = DictUtil.dict_of(report["counts"])
	c["one dead_click"] = DictUtil.to_int(counts["dead_click"]) == 1
	c["one repeat_click"] = DictUtil.to_int(counts["repeat_click"]) == 1
	c["one long_hover"] = DictUtil.to_int(counts["long_hover"]) == 1
	c["one error"] = DictUtil.to_int(counts["error"]) == 1
	c["nothing else"] = DictUtil.array_of(report["flags"]).size() == 4
	var error_detail: String = ""
	var dead_note: int = 0
	for raw: Variant in DictUtil.array_of(report["flags"]):
		var f: Dictionary = DictUtil.dict_of(raw)
		if str(f["kind"]) == "error":
			error_detail = str(f["detail"])
		if str(f["kind"]) == "dead_click":
			dead_note = DictUtil.to_int(f["note"])
	c["the error names the click before it"] = error_detail.contains("click Root/Bar/Go t=15000")
	c["a flag names the note within 30 s"] = dead_note == 1
	c["FLAG lines"] = TrailReport.lines(report).filter(func(l: String) -> bool: return l.begins_with("FLAG ")).size() == 4
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	rec.start_hooks()
	push_error("Playtest logger probe while recording")
	rec.drain_errors()
	var logged: int = _type_count(rec.data, "error")
	var logged_msg: String = ""
	for raw: Variant in DictUtil.array_of(rec.data["trail"]):
		if str(DictUtil.dict_of(raw).get("type", "")) == "error":
			logged_msg = str(DictUtil.dict_of(raw).get("msg", ""))
	rec.stop_hooks()
	push_error("Playtest logger probe after recording")
	rec.drain_errors()
	c["Logger records push_error while recording"] = logged == 1 and logged_msg.contains("Playtest logger probe while recording")
	c["nothing after recording stops"] = _type_count(rec.data, "error") == 1 and not rec.hooks_active()
	var path: String = _write("trail.json", _with_content(data))
	var tool: Dictionary = PlaytestReplayer.run_file(path, {"trail_report": true})
	c["--trail-report exits 0 with FLAG lines"] = DictUtil.to_int(tool["exit"]) == 0 and _lines_with(tool["lines"], "FLAG ") == 4
	return TestHelpers.report(TAG, "12. Trail report: dead, repeat, long hover and error flags; the Logger hook", c)

static func _type_count(data: Dictionary, type: String) -> int:
	var n: int = 0
	for raw: Variant in DictUtil.array_of(data.get("trail", [])):
		if str(DictUtil.dict_of(raw).get("type", "")) == type:
			n += 1
	return n

static func _with_content(data: Dictionary) -> Dictionary:
	var out: Dictionary = data.duplicate(true)
	out["content"] = {"algo": ContentFingerprint.ALGO, "total": "test", "files": {}}
	return out

# ------------------------------------------------------------------------------
# 13. UI open / close, leaks and asset fallbacks
# ------------------------------------------------------------------------------

static func _test_ui_asset_report() -> bool:
	var c: Dictionary = {}
	var menu: Dictionary = _ctl("Root/Menu", "PopupPanel")
	var trail: Array = [
		{"t": 0, "type": "scene", "path": "res://scenes/a.tscn", "leaked": 0},
		{"t": 200, "type": "panel", "ctl": _ctl("Root/Hud", "CombatPanel"), "open": true},   # the scene's own panel: never flagged
		{"t": 3000, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/OpenMenu", "Button"), "handled": true},
		{"t": 3100, "type": "modal", "ctl": menu, "open": true},
		{"t": 5000, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/OpenMenu", "Button"), "handled": true},
		{"t": 5100, "type": "modal", "ctl": menu, "open": true},
		{"t": 6000, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/Menu/Close", "Button"), "handled": true},
		{"t": 6100, "type": "modal", "ctl": menu, "open": false},
		{"t": 8000, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/X", "Button"), "handled": true},
		{"t": 9500, "type": "panel", "ctl": _ctl("Root/Late", "CombatPanel"), "open": true},
		{"t": 9600, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/Late/Close", "Button"), "handled": true},
		{"t": 9700, "type": "panel", "ctl": _ctl("Root/Late", "CombatPanel"), "open": false},
		{"t": 12000, "type": "button", "ctl": _ctl("Root/InvButton", "Button")},
		{"t": 12700, "type": "panel", "ctl": _ctl("Root/Inventory", "CombatPanel"), "open": true},
		{"t": 13000, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/Inventory/Close", "Button"), "handled": true},
		{"t": 13100, "type": "panel", "ctl": _ctl("Root/Inventory", "CombatPanel"), "open": false},
		{"t": 15000, "type": "click", "button": "left", "pos": [0, 0], "ctl": _ctl("Root/StuckButton", "Button"), "handled": true},
		{"t": 15100, "type": "panel", "ctl": _ctl("Root/Stuck", "CombatPanel"), "open": true},
		{"t": 20000, "type": "scene", "path": "res://scenes/b.tscn", "leaked": 1},
	]
	var data: Dictionary = PlaytestRecord.create("ui")
	data["trail"] = trail
	var counts: Dictionary = _counts(data)
	for kind: String in ["double_open", "unprompted_open", "slow_open", "left_open", "leak"]:
		c["one %s" % kind] = DictUtil.to_int(counts[kind]) == 1
	var total: int = 0
	for key: Variant in counts:
		total += DictUtil.to_int(counts[key])
	c["nothing else"] = total == 5

	var assets: String = SANDBOX.path_join("assets")
	DirAccess.make_dir_recursive_absolute(assets.path_join("files"))
	AssetCatalog.use_path(assets.path_join("manifest.json"), assets.path_join("files"))
	AssetRegistry.set_quiet(true)   # the missing-file warning would read as a failure in the logs
	var placeholder: Dictionary = AssetRegistry.export_placeholder("tile", "grass", "v0")
	c["sandbox row stored"] = AssetCatalog.upsert({"namespace": "tile", "key": "grass", "variant": "v0", "path": str(placeholder["path"])}).is_empty()
	DirAccess.remove_absolute(str(placeholder["file"]))
	var silent: PlaytestRecorder = PlaytestRecorder.open("user")
	AssetRegistry.resolve("tile", "qa_playtest_unknown", "v0")
	AssetRegistry.resolve("tile", "grass", "v0")
	c["observer empty: nothing logged"] = _type_count(silent.data, "asset") == 0 and not AssetRegistry.resolve_observer.is_valid()
	var rec: PlaytestRecorder = PlaytestRecorder.open("user")
	rec.scene = "res://scenes/a.tscn"
	rec.start_hooks()
	for _pass: int in range(2):
		AssetRegistry.resolve("tile", "qa_playtest_unknown", "v0")
		AssetRegistry.resolve("tile", "grass", "v0")
	var sources: Array[String] = []
	for raw: Variant in DictUtil.array_of(rec.data["trail"]):
		var e: Dictionary = DictUtil.dict_of(raw)
		if str(e.get("type", "")) == "asset":
			sources.append(str(e["source"]))
	sources.sort()
	c["one fallback and one missing_file event (deduplicated)"] = JsonUtil.deep_equal(sources, ["fallback", "missing_file"])
	rec.stop_hooks()
	AssetRegistry.resolve("tile", "qa_playtest_other", "v0")
	c["nothing after the hooks stop"] = _type_count(rec.data, "asset") == 2 and not AssetRegistry.resolve_observer.is_valid()
	c["asset events are flagged"] = DictUtil.to_int(_counts(rec.data)["asset_fallback"]) == 2
	AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
	AssetRegistry.clear_warnings()
	AssetRegistry.set_quiet(false)
	return TestHelpers.report(TAG, "13. UI open / close, leak and asset flags; the asset observer logs once per source", c)
