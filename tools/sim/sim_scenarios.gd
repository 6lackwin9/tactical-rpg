extends RefCounted

# Balance sim encounters (BAL2_SIM.md sections 5 and 6.4): the generated party encounters, catalog
# encounters with the party replaced, the five role limit-test scenarios, and the fight-set runner
# that feeds the metrics collector. Every unit is a sandbox unit built from class + level + tier
# (no shipped unit numbers); catalog encounters are read, never written. Loaded with preload.

const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimRoles = preload("res://tools/sim/sim_roles.gd")

const WAVE_ROUND: int = 2
const WAVE_BLOCK: int = 1
const PRIORITY_ROUND: int = 5   # striker band (8.1): the priority target down by this round

# 5: generated encounters ([class, tier] per unit; "elite" / "boss" classes follow elite= / boss=;
# "wave" units arrive at WAVE_ROUND / WAVE_BLOCK). BAL3 3.3 archetypes (threat = sum of tier HP x
# tier power, a hero 1.0): mob 5 minions, line 4 standard, wave 3 standard + 3 minion wave,
# elite elite + 3 standard, boss boss + 2 standard + 2 minion wave.
const GENERATED: Dictionary = {
	"mob": {"enemies": [["brawler", "minion"], ["rogue", "minion"], ["ranger", "minion"], ["priest", "minion"], ["brawler", "minion"]]},
	"line": {"enemies": [["warrior", "standard"], ["brawler", "standard"], ["ranger", "standard"], ["rogue", "standard"]]},
	"wave": {"enemies": [["brawler", "standard"], ["rogue", "standard"], ["ranger", "standard"]],
		"wave": [["brawler", "minion"], ["rogue", "minion"], ["priest", "minion"]]},
	"elite": {"enemies": [["@elite", "elite"], ["rogue", "standard"], ["mage", "standard"], ["warrior", "standard"]]},
	"boss": {"enemies": [["@boss", "boss"], ["ranger", "standard"], ["priest", "standard"]],
		"wave": [["ranger", "minion"], ["priest", "minion"]]},
}

# 6.4: limit tests. "allies": reference classes after the test unit; the first enemy of
# striker_priority is the priority target. BAL3 3.2: "block" = [columns, rows] of the clumped
# enemy start (SimVariants.apply_block, after the start distance); "tier_until" = {"level", "slot",
# "tier"}: at levels up to "level" the enemy at "slot" uses "tier" instead.
const SCENARIOS: Dictionary = {
	"tank_hold": {"role": "tank", "allies": ["ranger", "mage"],
		"enemies": [["brawler", "standard"], ["rogue", "standard"], ["brawler", "standard"], ["rogue", "standard"]]},
	"ranged_dive": {"role": "ranged", "allies": ["warrior"],
		"enemies": [["rogue", "standard"], ["rogue", "standard"], ["brawler", "standard"]]},
	"aoe_swarm": {"role": "aoe", "allies": ["warrior", "priest"], "block": [2, 3],   # BAL5 A1: the trio (priest added)
		"enemies": [["brawler", "minion"], ["rogue", "minion"], ["brawler", "minion"], ["brawler", "minion"], ["rogue", "minion"], ["priest", "minion"]]},
	"support_attrition": {"role": "support", "allies": ["warrior", "brawler"],
		"enemies": [["ranger", "standard"], ["ranger", "standard"], ["mage", "standard"], ["rogue", "standard"]]},
	"striker_priority": {"role": "striker", "allies": ["warrior"], "priority": true, "tier_until": {"level": 3, "slot": 0, "tier": "standard"},
		"enemies": [["mage", "elite"], ["warrior", "standard"], ["warrior", "standard"]]},
}
# BAL5 S2 (key `s2=on`, only on its trigger): support_attrition with one ranger turned into a brawler.
const S2_ENEMIES: Array = [["ranger", "standard"], ["brawler", "standard"], ["mage", "standard"], ["rogue", "standard"]]
const S2_SCENARIO: String = "support_attrition"
# BAL5 B2 (BAL5-PH, key `phase=on|off`, default on): the generated boss's phase, as the BOSS1 Crown of
# Nails effects (EncounterPhases fields); BOSS_REF is replaced by the boss unit's id.
const BOSS_REF: String = "@boss"
const BOSS_PHASE: Dictionary = {"id": "sim_boss_phase", "name": "Sim boss phase", "pause_autoplay": false,
	"trigger": {"kind": "hp_below", "unit": BOSS_REF, "pct": 50},
	"effects": [
		{"type": "status", "unit": BOSS_REF, "status": "fortify", "rounds": 3, "magnitude": 5},
		{"type": "status", "unit": BOSS_REF, "status": "regen", "rounds": 3, "magnitude": 4},
		{"type": "ai", "unit": BOSS_REF, "reckless": true, "target_pref": "WEAKEST"},
	]}
const ROLE_SCENARIO: Dictionary = {"tank": "tank_hold", "ranged": "ranged_dive", "aoe": "aoe_swarm", "support": "support_attrition", "striker": "striker_priority"}

# Party units simp_<class>_<i> (i = slot, 1-based) with the loadout / spend arguments.
static func define_party(runner: SimRunner, comp: Array[String], level: int) -> Array[String]:
	var out: Array[String] = []
	for i: int in range(comp.size()):
		var id: String = "simp_%s_%d" % [comp[i], i + 1]
		runner.define(id, comp[i], level, "hero", true)
		out.append(id)
	return out

# Enemy units sime_<class>_<tier>_<i> (i continues across lists so names stay unique).
static func define_enemies(runner: SimRunner, list: Array, level: int, first_index: int, elite_class: String, boss_class: String) -> Array[String]:
	var out: Array[String] = []
	var i: int = first_index
	for raw: Variant in list:
		var pair: Array = DictUtil.array_of(raw)
		var class_id: String = str(pair[0])
		if class_id == "@elite":
			class_id = elite_class
		elif class_id == "@boss":
			class_id = boss_class
		var id: String = "sime_%s_%s_%d" % [class_id, str(pair[1]), i]
		runner.define(id, class_id, level, str(pair[1]), false)
		out.append(id)
		i += 1
	return out

# Encounter `enc_id` for a party at `level`: {"enemies": ids, "base": encounter fields, "label"}.
# Generated ids use level + elevel; catalog ids keep authored levels unless elevel was given.
# boss_phase: the generated `boss` carries BOSS_PHASE (its first enemy is the boss).
static func encounter(runner: SimRunner, enc_id: String, level: int, elevel: int, elevel_set: bool, elite_class: String, boss_class: String, boss_phase: bool = false) -> Dictionary:
	var foe_level: int = clampi(level + elevel, 1, StatRules.MAX_LEVEL)
	if GENERATED.has(enc_id):
		var g: Dictionary = GENERATED[enc_id]
		var foes: Array[String] = define_enemies(runner, DictUtil.array_of(g["enemies"]), foe_level, 1, elite_class, boss_class)
		var base: Dictionary = {}
		if g.has("wave"):
			base = {"wave_units": define_enemies(runner, DictUtil.array_of(g["wave"]), foe_level, foes.size() + 1, elite_class, boss_class),
				"wave_round": WAVE_ROUND, "wave_block": WAVE_BLOCK}
		if boss_phase and enc_id == "boss":
			base["phases"] = [boss_phase_for(foes[0])]
		return {"enemies": foes, "base": base, "label": enc_id}
	var entry: Dictionary = ContentCatalog.get_entry("encounters", enc_id).duplicate(true)
	var base_c: Dictionary = {}
	for key: String in ["zone", "arena", "exits", "total_blocks", "unit_ap", "ap_regen", "wave_round", "wave_block", "wave_label"]:
		if entry.has(key):
			base_c[key] = entry[key]
	var enemies: Array[String] = _catalog_ids(runner, DictUtil.array_of(entry.get("enemies", [])), foe_level, elevel_set)
	base_c["wave_units"] = _catalog_ids(runner, DictUtil.array_of(entry.get("wave_units", [])), foe_level, elevel_set)
	return {"enemies": enemies, "base": base_c, "label": enc_id}

# BOSS_PHASE with every BOSS_REF replaced by `boss_id`.
static func boss_phase_for(boss_id: String) -> Dictionary:
	var p: Dictionary = BOSS_PHASE.duplicate(true)
	DictUtil.dict_of(p["trigger"])["unit"] = boss_id
	for fx: Variant in DictUtil.array_of(p["effects"]):
		DictUtil.dict_of(fx)["unit"] = boss_id
	return p

# Catalog roster ids, or level-shifted sandbox copies simc_<id> when elevel was given.
static func _catalog_ids(runner: SimRunner, ids: Array, level: int, shifted: bool) -> Array[String]:
	var out: Array[String] = []
	for raw: Variant in ids:
		var id: String = str(raw)
		if not shifted:
			out.append(id)
			continue
		var e: Dictionary = ContentCatalog.get_entry("units", id)
		var copy: String = "simc_%s" % id
		runner.define(copy, str(e.get("class_id", "")), level, str(e.get("tier", "standard")), false, {"copy_of": id})
		out.append(copy)
	return out

# A role scenario for test class `class_id`: the test unit simt_<class> (role loadout, variants
# apply), plain references simr_<class>_<i>, enemies at the same level.
# {"party": ids, "enemies": ids, "priority": display name or "", "test": id, "refs": ids}.
static func scenario(runner: SimRunner, scenario_id: String, class_id: String, level: int, test_plain: bool = false, s2: bool = false) -> Dictionary:
	var s: Dictionary = SCENARIOS[scenario_id]
	var role: String = str(s["role"])
	var test_id: String = "simt_%s%s" % [class_id, "_ref" if test_plain else ""]
	if test_plain:
		runner.define(test_id, class_id, level, "hero", true, {"plain": true})
	else:
		runner.define(test_id, class_id, level, "hero", true, {"loadout": SimRoles.role_loadout(class_id, role, level)})
	var refs: Array[String] = []
	var allies: Array = DictUtil.array_of(s["allies"])
	for i: int in range(allies.size()):
		var rid: String = "simr_%s_%d" % [str(allies[i]), i + 2]
		runner.define(rid, str(allies[i]), level, "hero", true, {"plain": true})
		refs.append(rid)
	var enemies: Array[String] = define_enemies(runner, scenario_enemies(scenario_id, level, s2), level, 1, "warrior", "brawler")
	var party: Array[String] = [test_id]
	party.append_array(refs)
	return {"party": party, "enemies": enemies, "priority": enemies[0] if DictUtil.to_bool(s.get("priority", false)) else "", "test": test_id, "refs": refs}

# The [class, tier] enemy list of `scenario_id` at `level` ("tier_until" applied; s2: S2_ENEMIES for S2_SCENARIO).
static func scenario_enemies(scenario_id: String, level: int, s2: bool = false) -> Array:
	var s: Dictionary = SCENARIOS[scenario_id]
	var list: Array = (S2_ENEMIES if s2 and scenario_id == S2_SCENARIO else DictUtil.array_of(s["enemies"])).duplicate(true)
	var rule: Dictionary = DictUtil.dict_of(s.get("tier_until", {}))
	if not rule.is_empty() and level <= DictUtil.int_of(rule, "level"):
		var slot: int = DictUtil.int_of(rule, "slot")
		list[slot] = [str(DictUtil.array_of(list[slot])[0]), str(rule["tier"])]
	return list

# The enemy block size of `scenario_id` (Vector2i.ZERO = standard deployment).
static func scenario_block(scenario_id: String) -> Vector2i:
	var raw: Array = DictUtil.array_of(DictUtil.dict_of(SCENARIOS[scenario_id]).get("block", []))
	return Vector2i(DictUtil.to_int(raw[0]), DictUtil.to_int(raw[1])) if raw.size() == 2 else Vector2i.ZERO

# ------------------------------------------------------------------------------
# Fight sets
# ------------------------------------------------------------------------------

static func empty_set() -> Dictionary:
	return {"fights": 0.0, "cleared": 0.0, "rounds_sum": 0.0, "rounds_cleared_sum": 0.0, "survivors_sum": 0.0, "party_size_sum": 0.0,
		"party_hp_sum": 0.0, "hp_sum": 0.0, "prio_by5": 0.0, "prio_round_sum": 0.0, "units": {}, "units_won": {}, "skipped": 0.0}

# Runs `left` vs `enemies` for every seed with the runner's current spend variant and preset; adds
# fight and per-unit (display name) metrics into `acc` (BAL5 K3: also into "units_won" for cleared
# fights). Skips refused fights; jitter PLAN SKIP fights (SimRunner K6) are counted in "skipped".
static func run_seeds(runner: SimRunner, metrics: SimMetrics, left: Array[String], enemies: Array[String], base: Dictionary, seeds: Array[int], priority: String, acc: Dictionary) -> void:
	metrics.priority_name = priority
	for s: int in seeds:
		var res: Dictionary = runner.fight(left, enemies, s, base)
		if DictUtil.to_bool(res.get("skipped", false)):
			acc["skipped"] = DictUtil.to_float(acc["skipped"]) + 1.0
		if not DictUtil.to_bool(res.get("ok", false)):
			continue
		var f: Dictionary = metrics.fight_record
		acc["fights"] = DictUtil.to_float(acc["fights"]) + 1.0
		if DictUtil.to_bool(f["cleared"]):
			acc["cleared"] = DictUtil.to_float(acc["cleared"]) + 1.0
			acc["rounds_cleared_sum"] = DictUtil.to_float(acc["rounds_cleared_sum"]) + DictUtil.to_float(f["rounds"])
		acc["rounds_sum"] = DictUtil.to_float(acc["rounds_sum"]) + DictUtil.to_float(f["rounds"])
		acc["survivors_sum"] = DictUtil.to_float(acc["survivors_sum"]) + DictUtil.to_float(f["survivors"])
		acc["party_size_sum"] = DictUtil.to_float(acc["party_size_sum"]) + float(left.size())
		acc["party_hp_sum"] = DictUtil.to_float(acc["party_hp_sum"]) + DictUtil.to_float(f["party_hp_frac"])
		acc["hp_sum"] = DictUtil.to_float(acc["hp_sum"]) + DictUtil.to_float(f["hp_score"])
		if DictUtil.to_int(f["priority_down_round"]) <= PRIORITY_ROUND:
			acc["prio_by5"] = DictUtil.to_float(acc["prio_by5"]) + 1.0
		acc["prio_round_sum"] = DictUtil.to_float(acc["prio_round_sum"]) + DictUtil.to_float(f["priority_down_round"])
		var sets: Array[Dictionary] = [acc["units"]]
		if DictUtil.to_bool(f["cleared"]):
			sets.append(acc["units_won"])
		for per_unit: Dictionary in sets:
			for n: Variant in metrics.units:
				var u: Dictionary = DictUtil.dict_of(per_unit.get(n, {}))
				SimMetrics.accumulate(u, DictUtil.dict_of(metrics.units[n]))
				per_unit[n] = u

static func clear_pct(acc: Dictionary) -> float:
	return DictUtil.to_float(acc["cleared"]) / maxf(1.0, DictUtil.to_float(acc["fights"])) * 100.0

static func mean(acc: Dictionary, key: String) -> float:
	return DictUtil.to_float(acc[key]) / maxf(1.0, DictUtil.to_float(acc["fights"]))

# Field sums over the named units of a set (side totals for role shares); `key` "units_won": cleared fights only.
static func sum_units(acc: Dictionary, names: Array[String], key: String = "units") -> Dictionary:
	var out: Dictionary = {}
	var per_unit: Dictionary = acc[key]
	for n: String in names:
		var u: Dictionary = DictUtil.dict_of(per_unit.get(n, {}))
		for f: Variant in u:
			out[f] = DictUtil.to_float(out.get(f, 0.0)) + DictUtil.to_float(u[f])
	return out
