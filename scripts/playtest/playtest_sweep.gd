class_name PlaytestSweep
extends RefCounted

# ==============================================================================
# Tactical RPG - Playtest Seed Sweep (PLAYTEST1 rule 11)
# Re-runs one recorded battle's plans under other seeds (no rng_state writes, no outcome comparison).
# A broken plan stops the row (`broken@r`) or, with a PlanPresets fallback, the party is drafted by
# that preset from that round on. Summary: result counts, median and min..max of rounds and party HP
# left, the recorded seed's row, outliers (outside 1.5 IQR, or a result other than the majority).
# A debugging aid on one recording, not a balance tool (balance stays tools/fc.sh sim).
# ==============================================================================

const MAX_SEEDS: int = 200
const COUNT_KEYS: Array[String] = ["won", "lost", "fled", "timeout", "abandoned", "broken"]

# "1,2,7" or "10..40" (or a mix "1,5..8") -> {seeds: Array[int], error}.
static func parse_seeds(text: String) -> Dictionary:
	var seeds: Array[int] = []
	for part: String in text.split(",", false):
		var p: String = part.strip_edges()
		if p.contains(".."):
			var ends: PackedStringArray = p.split("..")
			if ends.size() != 2 or not ends[0].is_valid_int() or not ends[1].is_valid_int() or ends[0].to_int() > ends[1].to_int():
				return {"seeds": [], "error": "Bad seed range '%s' (use a..b with a <= b)." % p}
			if ends[1].to_int() - ends[0].to_int() + 1 > MAX_SEEDS:
				return {"seeds": [], "error": "At most %d seeds per call." % MAX_SEEDS}
			for s: int in range(ends[0].to_int(), ends[1].to_int() + 1):
				seeds.append(s)
		elif p.is_valid_int():
			seeds.append(p.to_int())
		else:
			return {"seeds": [], "error": "Bad seed '%s'." % p}
	if seeds.is_empty():
		return {"seeds": [], "error": "No seeds given."}
	if seeds.size() > MAX_SEEDS:
		return {"seeds": [], "error": "At most %d seeds per call (got %d)." % [MAX_SEEDS, seeds.size()]}
	return {"seeds": seeds, "error": ""}

# {rows, recorded_row, summary}
static func run(battle: Dictionary, seeds: Array[int], fallback: String = "stop") -> Dictionary:
	var rows: Array = []
	for s: int in seeds:
		rows.append(row(battle, s, fallback))
	var recorded_seed: int = DictUtil.to_int(battle.get("seed", 0))
	var recorded_row: Dictionary = {}
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		if DictUtil.to_int(r["seed"]) == recorded_seed:
			recorded_row = r
			break
	if recorded_row.is_empty():
		recorded_row = row(battle, recorded_seed, fallback)
	return {"rows": rows, "recorded_row": recorded_row, "summary": summarize(rows)}

# One seed: {seed, result (end result or broken@r), bucket, rounds, hp_pct, end}.
static func row(battle: Dictionary, seed_value: int, fallback: String) -> Dictionary:
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(battle)
	rp.seed_value = seed_value
	rp.write_rng = false
	rp.compare = false
	rp.fallback = fallback
	if not rp.run():
		return {"seed": seed_value, "result": "broken@0", "bucket": "broken", "rounds": 0, "hp_pct": 0, "end": {}, "detail": "; ".join(rp.errors)}
	var result: String = str(rp.end.get("result", "abandoned"))
	var bucket: String = result
	var detail: String = ""
	if rp.broken_at > 0:
		result = "broken@%d" % rp.broken_at
		bucket = "broken"
		detail = str(rp.first_problem().get("detail", ""))
	return {"seed": seed_value, "result": result, "bucket": bucket, "rounds": DictUtil.to_int(rp.end.get("rounds", 0)), "hp_pct": rp.party_hp_pct(),
		"end": rp.end, "detail": detail}

static func rows_of(sweep: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for raw: Variant in DictUtil.array_of(sweep.get("rows", [])):
		out.append(DictUtil.dict_of(raw))
	return out

static func summarize(rows: Array) -> Dictionary:
	var counts: Dictionary = {}
	for key: String in COUNT_KEYS:
		counts[key] = 0
	var rounds: Array[float] = []
	var hp: Array[float] = []
	var by_result: Dictionary = {}
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		var bucket: String = str(r.get("bucket", "abandoned"))
		counts[bucket] = DictUtil.int_of(counts, bucket) + 1
		by_result[bucket] = DictUtil.int_of(by_result, bucket) + 1
		rounds.append(float(DictUtil.to_int(r.get("rounds", 0))))
		hp.append(float(DictUtil.to_int(r.get("hp_pct", 0))))
	var majority: String = ""
	var best: int = -1
	for key: String in COUNT_KEYS:
		if DictUtil.int_of(by_result, key) > best:
			best = DictUtil.int_of(by_result, key)
			majority = key
	var round_band: Vector2 = _fence(rounds)
	var hp_band: Vector2 = _fence(hp)
	var outliers: Array = []
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		var why: Array[String] = []
		var rr: float = float(DictUtil.to_int(r.get("rounds", 0)))
		var hh: float = float(DictUtil.to_int(r.get("hp_pct", 0)))
		if rr < round_band.x or rr > round_band.y:
			why.append("rounds")
		if hh < hp_band.x or hh > hp_band.y:
			why.append("hp")
		if str(r.get("bucket", "")) != majority:
			why.append("result")
		if not why.is_empty():
			outliers.append({"seed": DictUtil.to_int(r["seed"]), "why": why})
	return {"count": rows.size(), "counts": counts, "majority": majority, "rounds": _stats(rounds), "hp_pct": _stats(hp), "outliers": outliers}

static func summary_lines(sweep: Dictionary) -> Array[String]:
	var s: Dictionary = DictUtil.dict_of(sweep.get("summary", {}))
	var counts: Dictionary = DictUtil.dict_of(s.get("counts", {}))
	var parts: Array[String] = []
	for key: String in COUNT_KEYS:
		parts.append("%s %d" % [key, DictUtil.to_int(counts.get(key, 0))])
	var rs: Dictionary = DictUtil.dict_of(s.get("rounds", {}))
	var hs: Dictionary = DictUtil.dict_of(s.get("hp_pct", {}))
	var out: Array[String] = []
	out.append("summary %d seed(s): %s" % [DictUtil.to_int(s.get("count", 0)), ", ".join(parts)])
	out.append("rounds median %s, %s..%s; party HP left median %s%%, %s..%s%%" % [_num(rs, "median"), _num(rs, "min"), _num(rs, "max"), _num(hs, "median"), _num(hs, "min"), _num(hs, "max")])
	var rec: Dictionary = DictUtil.dict_of(sweep.get("recorded_row", {}))
	out.append("recorded seed %d: %s, rounds %d, party HP %d%%" % [DictUtil.to_int(rec.get("seed", 0)), str(rec.get("result", "")), DictUtil.to_int(rec.get("rounds", 0)), DictUtil.to_int(rec.get("hp_pct", 0))])
	var outliers: Array = DictUtil.array_of(s.get("outliers", []))
	if outliers.is_empty():
		out.append("outliers: none")
	for raw: Variant in outliers:
		var o: Dictionary = DictUtil.dict_of(raw)
		out.append("outlier seed %d (%s)" % [DictUtil.to_int(o.get("seed", 0)), ", ".join(JsonUtil.strings(o.get("why", [])))])
	return out

static func _num(d: Dictionary, key: String) -> String:
	var v: float = DictUtil.to_float(d.get(key, 0.0))
	return str(roundi(v)) if v == floorf(v) else "%.1f" % v

static func _stats(values: Array[float]) -> Dictionary:
	if values.is_empty():
		return {"median": 0.0, "min": 0.0, "max": 0.0}
	var sorted: Array[float] = values.duplicate()
	sorted.sort()
	return {"median": _quantile(sorted, 0.5), "min": sorted[0], "max": sorted[sorted.size() - 1]}

# (low fence, high fence) = (Q1 - 1.5 IQR, Q3 + 1.5 IQR), linear-interpolated quartiles.
static func _fence(values: Array[float]) -> Vector2:
	if values.is_empty():
		return Vector2.ZERO
	var sorted: Array[float] = values.duplicate()
	sorted.sort()
	var q1: float = _quantile(sorted, 0.25)
	var q3: float = _quantile(sorted, 0.75)
	var iqr: float = q3 - q1
	return Vector2(q1 - 1.5 * iqr, q3 + 1.5 * iqr)

static func _quantile(sorted: Array[float], q: float) -> float:
	var pos: float = (sorted.size() - 1) * q
	var lo: int = floori(pos)
	var hi: int = mini(lo + 1, sorted.size() - 1)
	return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo)
