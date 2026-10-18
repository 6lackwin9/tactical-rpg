class_name PlaytestReplayer
extends RefCounted

# ==============================================================================
# Tactical RPG - Playtest Replayer (PLAYTEST1 rules 4, 5, 10, 11)
# Replays one recorded battle on a fresh CombatSession: EncounterSource.create(encounter), seed, start();
# per round: check current_round, write the recorded rng_state, decode and set the plans in recorded
# order, require plan_is_valid, confirm_plan(false), resolve to REPLAY, compare the outcome, advance.
# Round results: match, diverge (first differing unit and field), plan_broken (decode or validity),
# unverified (after the first non-match), played (no comparison: sweeps, variants), preset (sweep fallback).
# Options: take-over (stop planning round N), variant patch (+ then stop / recorded), seed override
# without rng writes (sweep) with a PlanPresets fallback, and an optional recorder for --write.
# run_file() is the headless tool's whole logic (tools/replay.gd only parses arguments and prints).
# ==============================================================================

const RESOLVE_STEPS: int = 5000

var battle: Dictionary = {}
var seed_value: int = 0
var write_rng: bool = true          # write each recorded round's rng_state (off for seed sweeps)
var compare: bool = true            # compare outcomes with the recording
var take_over: int = -1             # > 0: replay rounds 1..N-1, advance into round N and stop (planning)
var variant_round: int = -1         # > 0: plans of this round come from variant_plans
var variant_plans: Array = []
var then_mode: String = "stop"      # after the variant round: stop | recorded
var fallback: String = "stop"       # when a plan breaks (or recorded plans run out): stop | PlanPresets id
var recorder: PlaytestRecorder = null

var session: CombatSession = null
var rounds: Array[Dictionary] = []
var errors: Array[String] = []      # setup / load problems (nothing was replayed)
var warnings: Array[String] = []
var end: Dictionary = {}
var broken_at: int = -1

static func for_battle(p_battle: Dictionary) -> PlaytestReplayer:
	var r := PlaytestReplayer.new()
	r.battle = p_battle
	r.seed_value = DictUtil.to_int(p_battle.get("seed", 0))
	return r

# Runs the replay; true when it could start (see `errors` otherwise).
func run() -> bool:
	errors = PlaytestRecord.battle_errors(battle)
	if not errors.is_empty():
		return false
	var recorded: Array[Dictionary] = PlaytestRecord.concluded_rounds(battle)
	var all_rounds: Array = DictUtil.array_of(battle.get("rounds", []))
	if all_rounds.size() > recorded.size():
		warnings.append("Round %d was not concluded in the recording (not replayed)." % (recorded.size() + 1))
	if take_over > 0 and take_over > recorded.size() + 1:
		errors.append("Take-over round %d needs rounds 1..%d recorded (the recording has %d)." % [take_over, take_over - 1, recorded.size()])
		return false
	var encounter_id: String = str(battle.get("encounter", ""))
	var enc: CombatEncounter = EncounterSource.create(encounter_id)
	enc.rng_seed = seed_value
	session = CombatSession.new(enc)
	if recorder != null:
		recorder.attach(session, encounter_id, "tool")
	session.start()
	var roster_entries: Array = DictUtil.array_of(battle.get("roster", []))
	PlanCodec.apply_pouches(session, DictUtil.array_of(battle.get("pouches", [])), warnings)   # ITEMUSE1 (older recordings: none)
	if recorder != null and not DictUtil.array_of(battle.get("pouches", [])).is_empty():
		recorder.battle()["pouches"] = PlanCodec.pouches(session)   # the replay's own recording keeps the rebuilt pouches
	var live_roster: Array = PlanCodec.roster(session)
	if live_roster.size() != roster_entries.size():
		warnings.append("Roster has %d units in this build, %d in the recording." % [live_roster.size(), roster_entries.size()])
	var on_preset: bool = false
	var r: int = 1
	while true:
		if session.current_round != r:
			_add(r, "plan_broken", "session is at round %d, the recording at round %d" % [session.current_round, r])
			break
		if take_over > 0 and r >= take_over:
			break
		var rec: Dictionary = recorded[r - 1] if r <= recorded.size() else {}
		var plans: Array = DictUtil.array_of(rec.get("plans", []))
		var source: String = "recorded"
		if variant_round > 0 and r == variant_round:
			plans = variant_plans
			source = "variant"
		elif variant_round > 0 and r > variant_round:
			if then_mode != "recorded" or rec.is_empty():
				break
		elif on_preset:
			source = "preset"
		elif rec.is_empty():
			if fallback == "stop":
				break
			on_preset = true
			source = "preset"
		if write_rng and not rec.is_empty():
			session.resolver.rng.state = str(rec.get("rng_state", "0")).to_int()
			if recorder != null:
				recorder.refresh_rng_state()
		var note: String = ""
		if source != "preset":
			var problem: String = _apply_plans(plans, roster_entries, r)
			if not problem.is_empty():
				if fallback == "stop":
					_add(r, "plan_broken", problem)
					broken_at = r
					break
				on_preset = true
				source = "preset"
				note = problem
		if source == "preset":
			session.plan.clear_all()
			session.plan_apply_preset_all(fallback)
			var preset_problem: String = PlanCodec.invalid_text(session, r)
			if not preset_problem.is_empty():
				_add(r, "plan_broken", "preset %s: %s" % [fallback, preset_problem])
				broken_at = r
				break
		if not session.confirm_plan(false):
			_add(r, "plan_broken", "round %d: confirm refused: %s" % [r, session.status_text])
			broken_at = r
			break
		if not _resolve():
			_add(r, "plan_broken", "round %d did not reach the replay phase in %d steps" % [r, RESOLVE_STEPS])
			broken_at = r
			break
		var outcome: Dictionary = RoundOutcome.capture(session)
		var entry: Dictionary = {"round": r, "result": "played", "detail": note, "outcome": outcome, "source": source}
		var rec_outcome: Dictionary = DictUtil.dict_of(rec.get("outcome", {}))
		if not rec_outcome.is_empty():
			entry["diffs"] = RoundOutcome.differences(rec_outcome, outcome)
		if compare and source == "recorded":
			var cmp: Dictionary = RoundOutcome.compare(rec_outcome, outcome)
			if DictUtil.to_bool(cmp.get("match", false)):
				entry["result"] = "match"
			else:
				entry["result"] = "diverge"
				entry["diff"] = cmp
				entry["detail"] = RoundOutcome.diff_text(cmp)
		elif source == "preset":
			entry["result"] = "preset"
		rounds.append(entry)
		if str(entry["result"]) == "diverge":
			break
		if session.combat_over or not session.can_advance_round():
			break
		var next: int = r + 1
		var more: bool = next <= recorded.size() or on_preset or (take_over == next)
		if variant_round > 0 and r >= variant_round and then_mode != "recorded":
			more = false
		if fallback != "stop" and not session.combat_over:
			more = true
		if not more:
			break
		session.advance_round()
		r = next
	if compare:
		for k: int in range(recorded.size()):
			var num: int = k + 1
			if not _has_round(num):
				rounds.append({"round": num, "result": "unverified", "detail": ""})
	var done: int = session.current_round if session.phase == CombatSession.Phase.REPLAY else session.current_round - 1
	end = PlaytestRecord.end_of(session, done)
	return true

func _apply_plans(plans: Array, roster_entries: Array, r: int) -> String:
	var decoded: Dictionary = PlanCodec.decode_plans(session, plans, roster_entries, r)
	warnings.append_array(JsonUtil.strings(decoded.get("warnings", [])))
	var decode_errors: Array[String] = JsonUtil.strings(decoded.get("errors", []))
	if not decode_errors.is_empty():
		return "; ".join(decode_errors)
	PlanCodec.apply(session, decoded)
	return PlanCodec.invalid_text(session, r)

func _resolve() -> bool:
	for i: int in range(RESOLVE_STEPS):
		if session.phase == CombatSession.Phase.REPLAY:
			return true
		session.step_next()
	return session.phase == CombatSession.Phase.REPLAY

func _add(r: int, result: String, detail: String) -> void:
	rounds.append({"round": r, "result": result, "detail": detail})

func _has_round(num: int) -> bool:
	for e: Dictionary in rounds:
		if DictUtil.to_int(e.get("round", 0)) == num:
			return true
	return false

# The result of round `num` ("" when it has none).
func result_of(num: int) -> String:
	for e: Dictionary in rounds:
		if DictUtil.to_int(e.get("round", 0)) == num:
			return str(e.get("result", ""))
	return ""

# True when every compared round matched and nothing broke.
func all_match() -> bool:
	if not errors.is_empty():
		return false
	for e: Dictionary in rounds:
		if not ["match", "played", "preset"].has(str(e.get("result", ""))):
			return false
	return true

func first_problem() -> Dictionary:
	for e: Dictionary in rounds:
		if ["diverge", "plan_broken"].has(str(e.get("result", ""))):
			return e
	return {}

# Party HP left in percent of the party's max HP (sweep rows).
func party_hp_pct() -> int:
	var hp: int = 0
	var max_total: int = 0
	if session == null:
		return 0
	for c: Combatant in session.allies:
		hp += maxi(0, c.hp)
		max_total += c.max_hp
	return roundi(100.0 * hp / max_total) if max_total > 0 else 0

# ------------------------------------------------------------------------------
# Headless tool (tools/replay.gd): load, validate, staleness, then replay / variant / seed sweep
# ------------------------------------------------------------------------------

# opts: {battle: int (-1 = all), variant: path, then: stop|recorded, write: path, seeds: String,
# fallback: stop|<preset>, trail_report: bool (only the trail report, exit 0), content_root: String}. Returns {exit, lines, report}; exit 0 all match (stale only
# warns), 1 divergence or broken plan, 2 load or validation error.
static func run_file(path: String, opts: Dictionary = {}) -> Dictionary:
	var lines: Array[String] = []
	var report: Dictionary = {"path": path, "errors": [], "warnings": [], "battles": []}
	var loaded: Dictionary = PlaytestStore.load_json(path)
	if not DictUtil.to_bool(loaded.get("ok", false)):
		return _refuse(lines, report, [str(loaded.get("error", "could not load"))])
	var raw: Dictionary = DictUtil.dict_of(loaded.get("data", {}))
	var data_errors: Array[String] = PlaytestRecord.errors(raw)
	if not data_errors.is_empty():
		return _refuse(lines, report, data_errors)
	var data: Dictionary = PlaytestRecord.normalize(raw)
	var rec_id: String = str(data.get("id", ""))
	report["id"] = rec_id
	if not rec_id.is_empty() and rec_id != path.get_file().get_basename():
		lines.append("WARN id %s does not match the file name %s" % [rec_id, path.get_file()])
	var battles: Array = DictUtil.array_of(data.get("battles", []))
	lines.append("recording %s: origin %s, %d battle(s), %d note(s), engine %s, rev %s" % [rec_id, str(data["origin"]), battles.size(),
		DictUtil.array_of(data.get("notes", [])).size(), str(data["engine"]), str(data["source_rev"])])
	var engine_now: String = str(Engine.get_version_info().get("string", ""))
	if not str(data["engine"]).is_empty() and str(data["engine"]) != engine_now:
		lines.append("WARN recorded with engine %s, replaying with %s" % [str(data["engine"]), engine_now])
	var stale: Dictionary = ContentFingerprint.diff(DictUtil.dict_of(data.get("content", {})), ContentFingerprint.compute(str(opts.get("content_root", ContentFingerprint.DEFAULT_ROOT))))
	report["stale"] = stale
	if DictUtil.to_bool(stale["stale"]):
		lines.append("STALE content differs: %d changed, %d added, %d removed (replay still runs)" % [DictUtil.array_of(stale["changed"]).size(),
			DictUtil.array_of(stale["added"]).size(), DictUtil.array_of(stale["removed"]).size()])
		for key: String in ["changed", "added", "removed"]:
			for p: String in JsonUtil.strings(stale[key]):
				lines.append("STALE %s %s" % [key, p])
	else:
		lines.append("content up to date (%s)" % str(DictUtil.dict_of(data.get("content", {})).get("total", "")))

	if DictUtil.to_bool(opts.get("trail_report", false)):
		var trail_report: Dictionary = TrailReport.build(data)
		lines.append_array(TrailReport.lines(trail_report))
		report["trail_report"] = trail_report
		report["exit"] = 0
		lines.append("RESULT match")
		return {"exit": 0, "lines": lines, "report": report}
	var variant: Dictionary = {}
	var variant_path: String = str(opts.get("variant", ""))
	if not variant_path.is_empty():
		var vload: Dictionary = PlaytestStore.load_json(variant_path)
		var vdata: Dictionary = DictUtil.dict_of(vload.get("data", {}))
		var verrors: Array[String] = variant_errors(vdata) if DictUtil.to_bool(vload.get("ok", false)) else [str(vload.get("error", ""))] as Array[String]
		if not verrors.is_empty():
			return _refuse(lines, report, verrors)
		variant = vdata
	var only: int = DictUtil.to_int(opts.get("battle", -1), -1)
	if not variant.is_empty():
		only = DictUtil.to_int(variant.get("battle", 0))
	if only >= battles.size():
		return _refuse(lines, report, ["Battle %d does not exist (the recording has %d)." % [only, battles.size()]])
	var seeds_text: String = str(opts.get("seeds", ""))
	var exit_code: int = 0
	for i: int in range(battles.size()):
		if only >= 0 and i != only:
			continue
		var b: Dictionary = DictUtil.dict_of(battles[i])
		var berrors: Array[String] = PlaytestRecord.battle_errors(b)
		if not berrors.is_empty():
			return _refuse(lines, report, berrors)
		var code: int = 0
		if not seeds_text.is_empty():
			code = _sweep_lines(b, i, seeds_text, str(opts.get("fallback", "stop")), lines, report)
		elif not variant.is_empty():
			code = _variant_lines(data, b, i, variant, opts, lines, report)
		else:
			code = _replay_lines(b, i, lines, report)
		exit_code = maxi(exit_code, code)
		if exit_code == 2:
			return {"exit": 2, "lines": lines, "report": report}
	lines.append("RESULT %s" % ["match", "mismatch", "error"][exit_code])
	report["exit"] = exit_code
	return {"exit": exit_code, "lines": lines, "report": report}

static func _refuse(lines: Array[String], report: Dictionary, problems: Array[String]) -> Dictionary:
	for p: String in problems:
		lines.append("ERROR %s" % p)
	report["errors"] = problems
	report["exit"] = 2
	lines.append("RESULT error")
	return {"exit": 2, "lines": lines, "report": report}

static func _replay_lines(b: Dictionary, i: int, lines: Array[String], report: Dictionary) -> int:
	var rp: PlaytestReplayer = for_battle(b)
	rp.run()
	var head: String = "battle %d %s seed %d" % [i, str(b.get("encounter", "")), rp.seed_value]
	for w: String in rp.warnings:
		lines.append("WARN %s: %s" % [head, w])
	for e: Dictionary in rp.rounds:
		var detail: String = str(e.get("detail", ""))
		lines.append("%s round %d %s%s" % [head, DictUtil.to_int(e.get("round", 0)), str(e.get("result", "")), (": " + detail) if not detail.is_empty() else ""])
	var rec_end: Dictionary = DictUtil.dict_of(b.get("end", {}))
	lines.append("%s end recorded %s/%d, replay %s/%d" % [head, str(rec_end.get("result", "")), DictUtil.to_int(rec_end.get("rounds", 0)),
		str(rp.end.get("result", "")), DictUtil.to_int(rp.end.get("rounds", 0))])
	var ok: bool = rp.all_match()
	if ok and not JsonUtil.deep_equal(rec_end, rp.end):
		ok = false
		lines.append("%s end differs" % head)
	_report_battle(report, i, rp, {"recorded_end": rec_end})
	return 0 if ok else 1

static func _variant_lines(data: Dictionary, b: Dictionary, i: int, variant: Dictionary, opts: Dictionary, lines: Array[String], report: Dictionary) -> int:
	var rp: PlaytestReplayer = for_battle(b)
	rp.compare = false
	rp.variant_round = DictUtil.to_int(variant.get("round", 0))
	rp.variant_plans = DictUtil.array_of(variant.get("plans", []))
	rp.then_mode = str(opts.get("then", "stop"))
	var out_path: String = str(opts.get("write", ""))
	if not out_path.is_empty():
		rp.recorder = PlaytestRecorder.open("agent", out_path.get_base_dir(), out_path.get_file().get_basename())
		rp.recorder.data["parent"] = {"id": str(data.get("id", "")), "battle": i, "round": rp.variant_round}
	rp.run()
	var head: String = "battle %d variant round %d (then %s)" % [i, rp.variant_round, rp.then_mode]
	for w: String in rp.warnings:
		lines.append("WARN %s: %s" % [head, w])
	var code: int = 0
	for e: Dictionary in rp.rounds:
		var num: int = DictUtil.to_int(e.get("round", 0))
		var diffs: Array = DictUtil.array_of(e.get("diffs", []))
		var detail: String = str(e.get("detail", ""))
		lines.append("%s round %d %s, %d difference(s)%s" % [head, num, str(e.get("source", e.get("result", ""))), diffs.size(), (": " + detail) if not detail.is_empty() else ""])
		for d: Variant in diffs:
			lines.append("DIFF %s" % RoundOutcome.diff_text(DictUtil.dict_of(d)))
		if str(e.get("result", "")) == "plan_broken":
			code = 1
	var rec_end: Dictionary = DictUtil.dict_of(b.get("end", {}))
	lines.append("%s end recorded %s/%d, variant %s/%d" % [head, str(rec_end.get("result", "")), DictUtil.to_int(rec_end.get("rounds", 0)),
		str(rp.end.get("result", "")), DictUtil.to_int(rp.end.get("rounds", 0))])
	if rp.recorder != null:
		var written: Error = rp.recorder.flush()
		lines.append(("wrote %s" % rp.recorder.path) if written == OK else "ERROR could not write %s (%s)" % [rp.recorder.path, error_string(written)])
		rp.recorder.detach()
	_report_battle(report, i, rp, {"recorded_end": rec_end, "variant_round": rp.variant_round})
	return code

static func _sweep_lines(b: Dictionary, i: int, seeds_text: String, p_fallback: String, lines: Array[String], report: Dictionary) -> int:
	var parsed: Dictionary = PlaytestSweep.parse_seeds(seeds_text)
	if not str(parsed.get("error", "")).is_empty():
		lines.append("ERROR %s" % str(parsed["error"]))
		report["errors"] = [str(parsed["error"])]
		return 2
	var seeds: Array[int] = []
	seeds.assign(DictUtil.array_of(parsed.get("seeds", [])))
	if p_fallback != "stop" and not PlanPresets.IDS.has(p_fallback):
		lines.append("ERROR unknown fallback '%s' (stop or %s)" % [p_fallback, ", ".join(PlanPresets.IDS)])
		return 2
	var sweep: Dictionary = PlaytestSweep.run(b, seeds, p_fallback)
	var head: String = "battle %d sweep" % i
	for row: Dictionary in PlaytestSweep.rows_of(sweep):
		lines.append("%s seed %d: %s, rounds %d, party HP %d%%" % [head, DictUtil.to_int(row["seed"]), str(row["result"]), DictUtil.to_int(row["rounds"]), DictUtil.to_int(row["hp_pct"])])
	for text: String in PlaytestSweep.summary_lines(sweep):
		lines.append("%s %s" % [head, text])
	var battles: Array = DictUtil.array_of(report.get("battles", []))
	battles.append({"index": i, "sweep": sweep})
	report["battles"] = battles
	return 0

static func _report_battle(report: Dictionary, i: int, rp: PlaytestReplayer, extra: Dictionary) -> void:
	var entry: Dictionary = {"index": i, "seed": rp.seed_value, "rounds": _strip_outcomes(rp.rounds), "end": rp.end, "warnings": rp.warnings, "errors": rp.errors}
	entry.merge(extra, true)
	var battles: Array = DictUtil.array_of(report.get("battles", []))
	battles.append(entry)
	report["battles"] = battles

static func _strip_outcomes(list: Array[Dictionary]) -> Array:
	var out: Array = []
	for e: Dictionary in list:
		var copy: Dictionary = e.duplicate(true)
		copy.erase("outcome")
		out.append(copy)
	return out

static func variant_errors(v: Dictionary) -> Array[String]:
	var out: Array[String] = []
	if str(v.get("format", "")) != PlaytestRecord.VARIANT_FORMAT:
		out.append("Not a variant patch (format '%s')." % str(v.get("format", "")))
		return out
	if DictUtil.to_int(v.get("version", 0)) != 1:
		out.append("Variant version %s is not supported (1)." % str(v.get("version", "")))
	if DictUtil.to_int(v.get("round", 0)) < 1:
		out.append("Variant round must be >= 1.")
	if not (v.get("plans", null) is Array):
		out.append("Variant plans missing.")
	return out
