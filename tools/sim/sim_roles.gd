extends RefCounted

# Balance sim roles (BAL2_SIM.md section 6): the class -> role map, the role loadouts and the role
# KPIs with their section 8.1 bands. Static data and pure functions; loaded with preload (no
# class_name). The `role` mode driver lives in tools/sim/sim_role_mode.gd.

const SimMetrics = preload("res://tools/sim/sim_metrics.gd")
const SimTargets = preload("res://tools/sim/sim_targets.gd")

const ROLE_LABEL: Dictionary = {"tank": "Tank", "ranged": "Ranged", "aoe": "AoE", "support": "Support", "striker": "Striker"}

# 6.2 (rogue = melee striker, brawler = melee AoE: user 2026-10-17).
const PRIMARY: Dictionary = {"warrior": "tank", "ranger": "ranged", "priest": "support", "brawler": "aoe", "rogue": "striker", "mage": "aoe"}
const SECONDARY: Dictionary = {"warrior": ["support"], "ranger": ["aoe"], "priest": ["striker"], "brawler": ["tank", "striker"], "rogue": [], "mage": ["ranged"]}

# 6.3 role loadouts at L5; L7_SWAPS replace one family from level 7 on.
const LOADOUTS: Dictionary = {
	"warrior": {"tank": ["taunt_shout", "shield_bash", "intimidate", "shield_wall"],
		"support": ["taunt_shout", "intimidate", "shield_bash", "sunder", "riposte"]},
	"ranger": {"ranged": ["aimed_shot", "pinning_shot", "leap_back", "barbed_arrow", "deadeye"],
		"aoe": ["aimed_shot", "pinning_shot", "leap_back", "volley", "deadeye"]},
	"priest": {"support": ["rally", "mend", "purify", "greater_mend", "renewal"],
		"striker": ["smite", "rally", "hex", "mend", "greater_mend"]},
	"brawler": {"aoe": ["body_blow", "lunge", "sweep_kick", "cyclone_kick", "haymaker"],
		"tank": ["body_blow", "lunge", "sweep_kick", "cyclone_kick", "haymaker"],
		"striker": ["body_blow", "lunge", "charge_in", "hamstring", "haymaker"]},
	"rogue": {"striker": ["quick_stab", "pounce", "lacerate", "backflip", "hamstring"]},
	"mage": {"aoe": ["arcane_bolt", "arc_lance", "firebomb", "glacial_burst", "focus"],
		"ranged": ["arcane_bolt", "arc_lance", "hex", "focus", "firebomb"]},
}
const L7_LEVEL: int = 7   # the bracketed L7 deltas of 6.3 apply from this level on
const L7_SWAPS: Dictionary = {"priest:striker": ["mend", "judgement"], "rogue:striker": ["hamstring", "assassinate"], "mage:aoe": ["focus", "inferno"]}

static func primary_role(class_id: String) -> String:
	return str(PRIMARY.get(class_id, "tank"))

static func secondary_roles(class_id: String) -> Array[String]:
	var out: Array[String] = []
	for r: Variant in DictUtil.array_of(SECONDARY.get(class_id, [])):
		out.append(str(r))
	return out

# "P" primary, "S" secondary, "-" off-role.
static func fit(class_id: String, role: String) -> String:
	if primary_role(class_id) == role:
		return "P"
	return "S" if secondary_roles(class_id).has(role) else "-"

# The class's loadout for `role` (its primary loadout when it has none for the role), with the L7
# swap, cut to the families owned at `level` and filled by the default rule (path order) up to
# StatRules.MAX_CLASS_SLOTS.
static func role_loadout(class_id: String, role: String, level: int) -> Array[String]:
	var table: Dictionary = DictUtil.dict_of(LOADOUTS.get(class_id, {}))
	var key: String = role if table.has(role) else primary_role(class_id)
	var wanted: Array[String] = []
	for f: Variant in DictUtil.array_of(table.get(key, [])):
		wanted.append(str(f))
	var swap: Array = DictUtil.array_of(L7_SWAPS.get("%s:%s" % [class_id, key], []))
	if level >= L7_LEVEL and swap.size() == 2 and wanted.has(str(swap[0])):
		wanted[wanted.find(str(swap[0]))] = str(swap[1])
	var owned: Array[String] = owned_families(class_id, level)
	var out: Array[String] = []
	for f: String in wanted:
		if owned.has(f) and not out.has(f):
			out.append(f)
	for f: String in owned:
		if out.size() >= StatRules.MAX_CLASS_SLOTS:
			break
		if not out.has(f):
			out.append(f)
	return out

# Class-path families reached by `level` (unreduced path levels), path order, always-active skipped.
static func owned_families(class_id: String, level: int) -> Array[String]:
	var out: Array[String] = []
	for step: Dictionary in ContentFactory.class_archetype(ContentCatalog.get_entry("classes", class_id)).path_steps():
		var skill_id: String = str(step.get("skill", ""))
		if skill_id.is_empty() or SkillCatalog.is_always_active(skill_id) or DictUtil.to_int(step.get("level", 1), 1) > level:
			continue
		var fam: String = SkillCatalog.family_of(skill_id)
		if not out.has(fam):
			out.append(fam)
	return out

# ------------------------------------------------------------------------------
# KPIs (6.1) and bands (8.1). A variant record `v` holds: test, party, refs (SimMetrics accumulators),
# fights, cleared, rounds_sum, rounds_cleared_sum, survivors_sum, prio_by5, prio_round_sum, baseline_survivors.
# Each KPI: {"id", "label", "band" (SimTargets key or ""), "pct" (prints as integer percent), "lower" (lower ranks better)}.
# The first KPI of a role is its ranking KPI.
# ------------------------------------------------------------------------------

static func kpis(role: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	match role:
		"tank":
			out.append(_k("taken_share", "dmg_taken share%", "tank_taken_share", true))
			out.append(_k("attacks_share", "atk_rcv share%", "tank_attacks_share", true))
			out.append(_k("taunt_frac", "taunt/alive%", "", true))
			out.append(_k("ally_hp", "allies hp_end%", "tank_ally_hp", true))
			out.append(_k("hold_frac", "hold/alive%", "", true))
			out.append(_k("survival", "survival%", "tank_survival", true))
		"ranged":
			out.append(_k("dist", "nearest_foe", "ranged_dist", false))
			out.append(_k("ranged_share", "ranged dmg%", "ranged_share", true))
			out.append(_k("attacks_per_round", "atk_rcv/rnd", "ranged_attacks_per_round", false))
			out.append(_k("dmg_per_round", "dmg/rnd", "", false))
			out.append(_k("survival", "survival%", "", true))
		"aoe":
			out.append(_k("foes_per_cast", "foes/cast", "aoe_foes_per_cast", false))
			out.append(_k("allies_per_cast", "allies/cast", "aoe_allies_per_cast", false))
			out.append(_k("clear_rounds", "rnds to clear", "", false))
			out.append(_k("dmg_share", "dmg share%", "", true))
			out.append(_k("swarm_clear", "swarm clear%", "aoe_swarm_clear", true))
		"support":
			out.append(_k("heal_share", "heal/party hp%", "support_heal_share", true))
			out.append(_k("saves", "saves/fight", "support_saves", false))
			out.append(_k("cleanses", "cleanse/fight", "", false))
			out.append(_k("buff_rounds", "buff rds/fight", "", false))
			out.append(_k("debuff_rounds", "debuff rds/f", "", false))
			out.append(_k("survivor_gain", "surv vs brawler", "support_survivor_gain", false))
		"striker":
			var k: Dictionary = _k("down_round", "prio down rnd", "", false)
			k["lower"] = true
			out.append(k)
			out.append(_k("down_by5", "down by r5%", "striker_down_by_5", true))
			out.append(_k("priority_share", "prio dmg%", "striker_priority_share", true))
			out.append(_k("dmg_per_round", "dmg/rnd", "", false))
			out.append(_k("survival", "survival%", "", true))
	return out

static func _k(id: String, label: String, band: String, pct: bool) -> Dictionary:
	return {"id": id, "label": label, "band": band, "pct": pct, "lower": false}

# Value of KPI `id` for variant record `v` (percent KPIs in percent).
static func kpi_value(id: String, v: Dictionary) -> float:
	var t: Dictionary = DictUtil.dict_of(v.get("test", {}))
	var p: Dictionary = DictUtil.dict_of(v.get("party", {}))
	var refs: Dictionary = DictUtil.dict_of(v.get("refs", {}))
	var fights: float = maxf(1.0, DictUtil.to_float(v.get("fights", 0.0)))
	var rounds: float = maxf(1.0, DictUtil.to_float(v.get("rounds_sum", 0.0)))
	match id:
		"taken_share":
			return _share(t, p, "dmg_taken")
		"attacks_share":
			return _share(t, p, "attacks_received")
		"taunt_frac":
			return SimMetrics.ratio(t, "taunt_blocks", "blocks_alive") * 100.0
		"ally_hp":
			return SimMetrics.per_fight(refs, "hp_end_frac") * 100.0
		"hold_frac":
			return SimMetrics.ratio(t, "hold_blocks", "blocks_alive") * 100.0
		"survival":
			return SimMetrics.per_fight(t, "survived") * 100.0
		"dist":
			return SimMetrics.ratio(t, "dist_sum", "dist_samples")
		"ranged_share":
			return SimMetrics.ratio(t, "dmg_dealt_ranged", "dmg_dealt") * 100.0
		"attacks_per_round":
			return DictUtil.to_float(t.get("attacks_received", 0.0)) / rounds
		"dmg_per_round":
			return DictUtil.to_float(t.get("dmg_dealt", 0.0)) / rounds
		"foes_per_cast":
			return SimMetrics.ratio(t, "area_foes_hit", "area_casts")
		"allies_per_cast":
			return SimMetrics.ratio(t, "area_allies_hit", "area_casts")
		"clear_rounds":
			var cleared: float = DictUtil.to_float(v.get("cleared", 0.0))
			return DictUtil.to_float(v.get("rounds_cleared_sum", 0.0)) / cleared if cleared > 0.0 else DictUtil.to_float(v.get("cap", 0.0)) + 1.0
		"dmg_share":
			return _share(t, p, "dmg_dealt")
		"swarm_clear", "clear":
			return DictUtil.to_float(v.get("cleared", 0.0)) / fights * 100.0
		"heal_share":
			return SimMetrics.ratio({"heal_done": t.get("heal_done", 0.0), "max_hp": p.get("max_hp", 0.0)}, "heal_done", "max_hp") * 100.0
		"saves":
			return SimMetrics.per_fight(t, "near_death_saves")
		"cleanses":
			return SimMetrics.per_fight(t, "cleanses")
		"buff_rounds":
			return SimMetrics.per_fight(t, "buff_rounds")
		"debuff_rounds":
			return SimMetrics.per_fight(t, "debuff_rounds")
		"survivor_gain":
			return DictUtil.to_float(v.get("survivors_sum", 0.0)) / fights - DictUtil.to_float(v.get("baseline_survivors", 0.0))
		"survivors":
			return DictUtil.to_float(v.get("survivors_sum", 0.0)) / fights
		"down_round":
			return DictUtil.to_float(v.get("prio_round_sum", 0.0)) / fights
		"down_by5":
			return DictUtil.to_float(v.get("prio_by5", 0.0)) / fights * 100.0
		"priority_share":
			return SimMetrics.ratio(t, "priority_dmg", "dmg_dealt") * 100.0
	return 0.0

static func _share(t: Dictionary, p: Dictionary, field: String) -> float:
	var total: float = DictUtil.to_float(p.get(field, 0.0))
	return DictUtil.to_float(t.get(field, 0.0)) / total * 100.0 if total > 0.0 else 0.0

# {"mean", "floor", "ceil"} of KPI `id` over the variant records.
static func spread(id: String, variants: Array[Dictionary]) -> Dictionary:
	var lo: float = INF
	var hi: float = -INF
	var sum: float = 0.0
	for v: Dictionary in variants:
		var x: float = kpi_value(id, v)
		lo = minf(lo, x)
		hi = maxf(hi, x)
		sum += x
	var n: int = variants.size()
	return {"mean": sum / float(n) if n > 0 else 0.0, "floor": lo if n > 0 else 0.0, "ceil": hi if n > 0 else 0.0}

# "mean [floor..ceiling]" with " X" when `marked` and the range misses the band.
static func cell(k: Dictionary, s: Dictionary, targets: Dictionary, marked: bool) -> String:
	var fmt: String = "%d" if DictUtil.to_bool(k["pct"]) else "%.2f"
	var mean_v: float = DictUtil.to_float(s["mean"])
	var floor_v: float = DictUtil.to_float(s["floor"])
	var ceil_v: float = DictUtil.to_float(s["ceil"])
	var text: String = (fmt + " [" + fmt + ".." + fmt + "]") % ([roundi(mean_v), roundi(floor_v), roundi(ceil_v)] if DictUtil.to_bool(k["pct"]) else [mean_v, floor_v, ceil_v])
	var band: String = str(k["band"])
	if marked and not band.is_empty() and not SimTargets.overlaps(targets, band, floor_v, ceil_v):
		text += " X"
	return text

# Ranking check (8.1) on the first KPI: primary classes in the top 2, a secondary class above every
# off-role class. `means`: class -> mean. Returns class -> "rank N" (+ " X" on a miss).
static func rank_marks(role: String, means: Dictionary) -> Dictionary:
	var lower: bool = DictUtil.to_bool(kpis(role)[0]["lower"])
	var order: Array[String] = []
	for c: Variant in means:
		order.append(str(c))
	order.sort_custom(func(a: String, b: String) -> bool:
		var va: float = DictUtil.to_float(means[a])
		var vb: float = DictUtil.to_float(means[b])
		if is_equal_approx(va, vb):
			return a < b
		return va < vb if lower else va > vb)
	var best_off: int = order.size()
	for i: int in range(order.size()):
		if fit(order[i], role) == "-":
			best_off = mini(best_off, i)
	var out: Dictionary = {}
	for i: int in range(order.size()):
		var c: String = order[i]
		var miss: bool = (fit(c, role) == "P" and i >= 2) or (fit(c, role) == "S" and i > best_off)
		out[c] = "rank %d%s" % [i + 1, " X" if miss else ""]
	return out
