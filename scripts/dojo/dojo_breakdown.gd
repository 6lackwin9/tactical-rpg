class_name DojoBreakdown
extends RefCounted

# ==============================================================================
# Tactical RPG - Dojo Damage Breakdown Formatter
# Converts a DamageCalculator result into a step-by-step BBCode explanation.
# ==============================================================================

static func build_strike(title: String, attacker_name: String, target: Combatant, res: Dictionary, actual_dmg: int) -> String:
	var lines: Array[String] = []
	var raw_w: Variant = res.get("weapon", null)
	var w_name: String = "Unarmed"
	if raw_w is Equipment:
		var weapon: Equipment = raw_w
		w_name = weapon.name
	lines.append("[color=#fde047][b]%s[/b][/color]  %s ➜ %s  (%s)" % [title, attacker_name, target.name, w_name])

	if UiTheme.dict_bool(res, "authored"):
		lines.append("1-3. Attack Power: [color=#38bdf8][b]%d[/b][/color] (authored strike power)" % UiTheme.dict_int(res, "attack_power"))
	elif str(res.get("stat_name", "")) == "STR+DEX":
		lines.append("1. Weapon Power: 0 (unarmed)")
		lines.append("2. Unit Power: STR + DEX = [color=#38bdf8]%d[/color]" % UiTheme.dict_int(res, "stat_val"))
		lines.append("3. Attack Power: [color=#e2e8f0]%d[/color]" % UiTheme.dict_int(res, "attack_power"))
	else:
		if UiTheme.dict_bool(res, "weapon_fits"):
			var pwr_parts: Array[String] = ["Base %d" % UiTheme.dict_int(res, "base_w_power")]
			if UiTheme.dict_int(res, "tier_boost") > 0:
				pwr_parts.append("[color=#fbbf24]Tier +%d[/color]" % UiTheme.dict_int(res, "tier_boost"))
			if UiTheme.dict_int(res, "mastery_pwr") > 0:
				pwr_parts.append("[color=#a855f7]Mastery +%d[/color]" % UiTheme.dict_int(res, "mastery_pwr"))
			lines.append("1. Weapon Power: [color=#e2e8f0]%d[/color] (%s)" % [UiTheme.dict_int(res, "total_w_power"), ", ".join(pwr_parts)])
			var pen_ratio: float = UiTheme.dict_float(res, "pen_ratio")
			if pen_ratio > 0.0:
				lines.append("   [color=#f87171]⚠ Deficit -%d: %d × (1 - %d%%) = %.1f[/color]" % [UiTheme.dict_int(res, "def_amount"), UiTheme.dict_int(res, "total_w_power"), roundi(pen_ratio * 100.0), UiTheme.dict_float(res, "weapon_power")])
		else:
			lines.append("1. Weapon Power: 0 (this weapon doesn't suit the attack)")
		lines.append("2. Unit Power: %d %s × %.1f = [color=#38bdf8]+%.1f[/color]" % [UiTheme.dict_int(res, "stat_val"), str(res.get("stat_name", "STR")), UiTheme.dict_float(res, "unit_scaling", 1.5), UiTheme.dict_float(res, "unit_power")])
		lines.append("3. Attack Power: %.1f + %.1f = [color=#e2e8f0]%d[/color]" % [UiTheme.dict_float(res, "weapon_power"), UiTheme.dict_float(res, "unit_power"), UiTheme.dict_int(res, "attack_power")])
	var buff_mult: float = UiTheme.dict_float(res, "buff_mult", 1.0)
	if buff_mult != 1.0:
		lines.append("   [color=#c084fc]✦ Focus ×%.2f → [b]%d ATK[/b][/color]" % [buff_mult, UiTheme.dict_int(res, "final_attack_power")])
	var mult_parts: Array[String] = []
	var skill_mult: float = UiTheme.dict_float(res, "skill_mult", 1.0)
	if skill_mult != 1.0:
		mult_parts.append("Skill ×%.2f" % skill_mult)
	mult_parts.append("Guard ×%.2f (%d%%)" % [UiTheme.dict_float(res, "mit_mult", 1.0), roundi(UiTheme.dict_float(res, "mit_ratio") * 100.0)])
	mult_parts.append("Armor %d ×%.2f" % [UiTheme.dict_int(res, "armor_rating"), 1.0 - UiTheme.dict_float(res, "armor_ratio")])
	var stance_mult: float = UiTheme.dict_float(res, "stance_mult")
	if stance_mult != 0.0:
		mult_parts.append("[color=#fbbf24]Stance ×%.2f[/color]" % UiTheme.dict_float(res, "stance_factor", 1.0))
	else:
		mult_parts.append("Stance ×1.00")
	if UiTheme.dict_bool(res, "is_crit"):
		mult_parts.append("[color=#f43f5e]CRIT ×%.1f[/color]" % UiTheme.dict_float(res, "crit_dmg_mult", 1.0))
	else:
		mult_parts.append("Crit no (%.1f%% chance)" % UiTheme.dict_float(res, "crit_rate"))
	var effect: float = UiTheme.dict_float(res, "effect_factor", 1.0)
	if effect != 1.0:
		mult_parts.append("Effect ×%.2f" % effect)
	lines.append("4. Multipliers: %s" % " | ".join(mult_parts))

	lines.append("[color=#4ade80][b]RESULT: %d damage[/b][/color]  (%s HP %d / %d)" % [actual_dmg, target.name, target.hp, target.max_hp])
	return "\n".join(lines)

static func build_benchmark(title: String, count: int, stats: Dictionary) -> String:
	var lines: Array[String] = []
	lines.append("[color=#fde047][b]%s[/b][/color]  (%d simulated swings, dummy HP untouched)" % [title, count])
	lines.append("Per-hit (no crit): [color=#e2e8f0]%d[/color]" % UiTheme.dict_int(stats, "base_hit"))
	lines.append("Observed: min [color=#e2e8f0]%d[/color] / avg [color=#38bdf8]%.1f[/color] / max [color=#e2e8f0]%d[/color]" % [
		UiTheme.dict_int(stats, "min"), UiTheme.dict_float(stats, "avg"), UiTheme.dict_int(stats, "max")
	])
	lines.append("Crits: %d / %d  (rate %.1f%%)" % [UiTheme.dict_int(stats, "crits"), count, UiTheme.dict_float(stats, "crit_rate")])
	lines.append("Expected damage / swing: [color=#4ade80][b]%.1f[/b][/color]" % UiTheme.dict_float(stats, "expected"))
	lines.append("Swings to defeat dummy (%d HP): [color=#fbbf24]%d[/color]" % [UiTheme.dict_int(stats, "dummy_hp"), UiTheme.dict_int(stats, "swings_to_kill")])
	return "\n".join(lines)
