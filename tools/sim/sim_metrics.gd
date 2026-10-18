extends BattleMetrics

# Balance sim metrics collector (BAL2_SIM.md section 4). The counting (records, HP attribution, round-end credit,
# status diff, block sampling, self-checks, accumulate) moved to scripts/combat/session/battle_metrics.gd
# (BREPORT1-L 1.2) so the results screen and the sim share it; this file keeps only the sim's KPI table helpers.

static func per_fight(acc: Dictionary, field: String) -> float:
	return DictUtil.to_float(acc.get(field, 0.0)) / maxf(1.0, DictUtil.to_float(acc.get("fights", 0.0)))

static func ratio(acc: Dictionary, num: String, den: String) -> float:
	var d: float = DictUtil.to_float(acc.get(den, 0.0))
	return DictUtil.to_float(acc.get(num, 0.0)) / d if d > 0.0 else 0.0

static func kpi_header() -> String:
	return "  %-22s %6s %5s %6s %4s %4s %5s %6s %5s %6s %4s %9s %9s %11s %5s %5s %5s %5s %5s %5s %5s %5s %5s" % [
		"unit (per fight)", "dealt", "rng%", "taken", "dot", "ff", "atk_r", "heal", "ovh", "heal_r", "cln", "buffs/rds", "debuf/rds",
		"area c/f/a", "kills", "surv%", "hpend", "minhp", "saves", "dist", "adj%", "hold%", "taunt%"]

# One KPI line of aggregated record `acc` ("n/a" for overheal when a heal's skill was not found).
static func kpi_line(label: String, acc: Dictionary) -> String:
	var blocks: float = DictUtil.to_float(acc.get("blocks_alive", 0.0))
	var pct: Callable = func(field: String) -> String:
		return "%d" % roundi(DictUtil.to_float(acc.get(field, 0.0)) / blocks * 100.0) if blocks > 0.0 else "n/a"
	var ovh: String = "n/a" if DictUtil.to_float(acc.get("overheal_unknown", 0.0)) > 0.0 else "%.0f" % per_fight(acc, "overheal")
	return "  %-22s %6.0f %5d %6.0f %4.0f %4.0f %5.1f %6.0f %5s %6.0f %4.1f %4.1f/%-4.1f %4.1f/%-4.1f %3.1f/%3.1f/%3.1f %5.2f %5d %5.2f %5.2f %5.2f %5.2f %5s %5s %5s" % [
		label, per_fight(acc, "dmg_dealt"), roundi(ratio(acc, "dmg_dealt_ranged", "dmg_dealt") * 100.0), per_fight(acc, "dmg_taken"),
		per_fight(acc, "dot_taken"), per_fight(acc, "ff_taken"), per_fight(acc, "attacks_received"), per_fight(acc, "heal_done"), ovh,
		per_fight(acc, "heal_received"), per_fight(acc, "cleanses"), per_fight(acc, "buffs_applied"), per_fight(acc, "buff_rounds"),
		per_fight(acc, "debuffs_applied"), per_fight(acc, "debuff_rounds"), per_fight(acc, "area_casts"), per_fight(acc, "area_foes_hit"),
		per_fight(acc, "area_allies_hit"), per_fight(acc, "kills"), roundi(per_fight(acc, "survived") * 100.0), per_fight(acc, "hp_end_frac"),
		per_fight(acc, "min_hp_frac"), per_fight(acc, "near_death_saves"), ratio(acc, "dist_sum", "dist_samples"),
		pct.call("blocks_adjacent"), pct.call("hold_blocks"), pct.call("taunt_blocks")]
