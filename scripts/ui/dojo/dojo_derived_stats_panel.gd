class_name DojoDerivedStatsPanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 5: Derived Stats
# Combat metrics computed from stats and gear. Hover any row to see the exact
# formula and the live numbers plugged into it.
# ==============================================================================

const ROW_DEFS: Array[Dictionary] = [
	{"id": "melee", "label": "Melee Power", "color": Color(0.95, 0.45, 0.4)},
	{"id": "ranged", "label": "Ranged Power", "color": Color(0.4, 0.9, 0.5)},
	{"id": "offhand", "label": "Off-Hand Power", "color": Color(0.9, 0.6, 0.4)},
	{"id": "speed", "label": "Initiative Speed", "color": Color(0.35, 0.85, 0.95)},
	{"id": "hp", "label": "Maximum Health", "color": Color(0.95, 0.4, 0.4)},
	{"id": "mit", "label": "Armor", "color": Color(0.7, 0.85, 1.0)},
	{"id": "burden", "label": "Unit Burden", "color": Color(0.9, 0.8, 0.35)},
	{"id": "carry", "label": "Party Carry", "color": Color(0.5, 0.85, 0.65)},
	{"id": "range", "label": "Attack Range", "color": Color(0.6, 0.9, 0.8)},
	{"id": "crit", "label": "Crit Hit Rate", "color": Color(0.95, 0.75, 0.3)},
	{"id": "resist", "label": "Debuff Resist", "color": Color(0.7, 0.5, 0.95)},
]

var _rows: Dictionary = {}    # String -> UiTipPanel
var _values: Dictionary = {}  # String -> Label
var _carry_bar: ProgressBar

func _init() -> void:
	super._init()
	panel_id = "derived"
	panel_title = "Derived Stats"
	panel_short_title = "Derived"
	panel_icon = "📈"
	default_min_size = Vector2(230, 130)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 3)
	scroll.add_child(vbox)

	vbox.add_child(UiTheme.header("HOVER A ROW FOR ITS FORMULA", UiTheme.COL_MUTED))

	for def: Dictionary in ROW_DEFS:
		var id: String = str(def.get("id", ""))
		var col: Color = def.get("color", Color.WHITE)

		var row := UiTipPanel.new()
		row.mouse_filter = Control.MOUSE_FILTER_STOP
		row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_theme_stylebox_override("panel", UiTheme.box(Color(0.11, 0.14, 0.20, 0.85), Color(0.18, 0.26, 0.38, 0.7), 3, 6, 3))
		vbox.add_child(row)

		var inner := VBoxContainer.new()
		inner.mouse_filter = Control.MOUSE_FILTER_PASS
		inner.add_theme_constant_override("separation", 1)
		row.add_child(inner)

		var line := HBoxContainer.new()
		line.mouse_filter = Control.MOUSE_FILTER_PASS
		inner.add_child(line)

		var tag := UiTheme.label(str(def.get("label", "")), UiTheme.FONT_SM, col)
		tag.mouse_filter = Control.MOUSE_FILTER_PASS
		line.add_child(tag)
		line.add_spacer(false)

		var value := UiTheme.label("-", UiTheme.FONT_SM, Color.WHITE)
		value.mouse_filter = Control.MOUSE_FILTER_PASS
		line.add_child(value)

		if id == "carry":
			_carry_bar = ProgressBar.new()
			_carry_bar.custom_minimum_size = Vector2(0, 5)
			_carry_bar.show_percentage = false
			_carry_bar.mouse_filter = Control.MOUSE_FILTER_PASS
			_carry_bar.add_theme_stylebox_override("background", UiTheme.box(Color(0.15, 0.12, 0.08), Color(0, 0, 0, 0), 2, 0, 0, 0))
			_carry_bar.add_theme_stylebox_override("fill", UiTheme.box(Color(0.9, 0.75, 0.3), Color(0, 0, 0, 0), 2, 0, 0, 0))
			inner.add_child(_carry_bar)

		_rows[id] = row
		_values[id] = value

func _set_row(id: String, text: String, tip: String) -> void:
	var value_raw: Variant = _values.get(id, null)
	if value_raw is Label:
		var value_lbl: Label = value_raw
		value_lbl.text = text
	var row_raw: Variant = _rows.get(id, null)
	if row_raw is UiTipPanel:
		var row: UiTipPanel = row_raw
		row.tooltip_text = tip

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.stats == null:
		return
	var hero: Combatant = session.active_hero
	var st: CharacterStats = hero.stats
	var inv: Inventory = hero.inventory
	var main_w: Equipment = inv.get_main_weapon() if inv != null else null
	var off_w: Equipment = inv.get_offhand_weapon() if inv != null else null
	var ctx: Dictionary = DojoSession.weapon_context(hero)

	var eff_str: int = st.get_effective_stat("STR", ctx)
	var eff_dex: int = st.get_effective_stat("DEX", ctx)
	var eff_int: int = st.get_effective_stat("INT", ctx)
	var eff_cons: int = st.get_effective_stat("CONS", ctx)
	var burden: int = hero.worn_burden()
	var report: Dictionary = session.carry_report

	# --- Offensive power (DamageCalculator: weapon power + unit power) ---
	var melee_res: Dictionary = DamageCalculator.attack_power(hero, false, false)
	_set_row("melee", "%d ATK%s" % [DictUtil.to_int(melee_res["attack_power"]), " (ranged eqpd)" if (main_w != null and main_w.is_ranged_weapon()) else ""], _power_tip("MELEE POWER", melee_res))
	var ranged_res: Dictionary = DamageCalculator.attack_power(hero, true, false)
	_set_row("ranged", "%d ATK%s" % [DictUtil.to_int(ranged_res["attack_power"]), " (melee eqpd)" if (main_w != null and main_w.is_melee_weapon()) else ""], _power_tip("RANGED POWER", ranged_res))
	if off_w != null:
		var off_res: Dictionary = DamageCalculator.attack_power(hero, false, true)
		var needs_switch: bool = inv.needs_weapon_switch()
		var switch_note: String = "\n[color=#fbbf24]2H main weapon equipped: Switch Weapon (1 block) to use the off-hand weapon.[/color]" if needs_switch else ""
		_set_row("offhand", "%d ATK%s" % [DictUtil.to_int(off_res["attack_power"]), " (needs switch)" if needs_switch else ""], _power_tip("OFF-HAND POWER", off_res) + switch_note)
	else:
		_set_row("offhand", "-", _tip("OFF-HAND POWER", "Off-hand Weapon Power + Unit Power", "No offensive off-hand weapon equipped."))

	# --- Defensive & utility ---
	_set_row("speed", "%d (burden -%d, carry -%d)" % [hero.speed, burden, hero.carry_speed_penalty],
		_tip("INITIATIVE SPEED", "8 + DEX - Burden - Party overload penalty", "8 + %d DEX - %d burden - %d party overload = [b]%d[/b]" % [eff_dex, burden, hero.carry_speed_penalty, hero.speed]))
	_set_row("hp", "%d" % st.get_max_hp(ctx),
		_tip("MAXIMUM HEALTH", "60 + CONS × 8 + STR × 2", "60 + %d (CONS × 8) + %d (STR × 2) = [b]%d[/b]" % [eff_cons * 8, eff_str * 2, st.get_max_hp(ctx)]))

	var arm_mit: int = inv.get_total_armor_mitigation() if inv != null else 0
	var shd_mit: int = inv.get_total_shield_mitigation() if inv != null else 0
	var gear_mit: int = inv.get_total_flat_mitigation() if inv != null else 0
	var rating: int = DamageCalculator.armor_rating(hero)
	var cut: int = roundi(DamageCalculator.armor_reduction(rating) * 100.0)
	_set_row("mit", "%d rating  →  -%d%% damage" % [rating, cut],
		_tip("ARMOR", "rating = Gear Mitigation + STR / 4;  damage taken × (1 - rating / (rating + 50))", "%d gear (armor %d + off-hand %d) + STR %d / 4 = [b]%d[/b] → [b]-%d%%[/b] damage" % [gear_mit, arm_mit, shd_mit, eff_str, rating, cut]))

	# Unit burden: per unit, slows only the wearer. Party carry: one shared load for everyone.
	var heavy_pieces: Array[String] = []
	if inv != null:
		for eq: Equipment in inv._iter_equipped_gear():
			if eq.burden > 0:
				heavy_pieces.append("%s %d" % [eq.name, eq.burden])
	var pieces_text: String = "\n" + "  •  ".join(heavy_pieces) if not heavy_pieces.is_empty() else ""
	_set_row("burden", "%d  →  -%d speed" % [burden, burden],
		_tip("UNIT BURDEN", "Σ burden of THIS unit's equipped gear", "[b]%d[/b] burden → initiative speed -%d (only %s is slowed)%s" % [burden, burden, hero.name, pieces_text]))

	var tier: String = str(report.get("tier", "LIGHT"))
	var load_weight: float = UiTheme.dict_float(report, "load")
	var capacity: int = UiTheme.dict_int(report, "capacity", 1)
	_set_row("carry", "%.1f / %d  %s" % [load_weight, capacity, tier.capitalize()],
		_tip("PARTY CARRY", "30 + Σ (STR × 3 + CONS) of all members", "Load [b]%.1f[/b] / capacity [b]%d[/b] (%s). Shared by the whole party: stash + equipped gear + pouches. Overloaded = every unit slows (now -%d)." % [load_weight, capacity, tier.capitalize(), hero.carry_speed_penalty]))
	_carry_bar.max_value = maxi(1, capacity)
	_carry_bar.value = minf(load_weight, float(capacity))

	_set_row("range", "%d tile(s)" % hero.attack_range,
		_tip("ATTACK RANGE", "Weapon Range + 1 if PERC ≥ 18", "Weapon range %d, PERC %d → [b]%d[/b]" % [main_w.base_range if main_w != null else 1, st.get_effective_stat("PERC", ctx), hero.attack_range]))

	var crit_rate: float = st.get_crit_rate(ctx)
	var crit_mult: float = st.get_crit_damage_multiplier(ctx)
	_set_row("crit", "%.1f%% (%.1f×)" % [crit_rate, crit_mult],
		_tip("CRIT RATE & DAMAGE", "5%% + 1%% per DEX above 10  •  damage ×%.1f (seeded roll)" % crit_mult, "5.0%% + %d (DEX %d above 10) = [b]%.1f%%[/b]" % [maxi(0, eff_dex - 10), eff_dex, crit_rate]))

	var resist: float = st.get_debuff_resistance(ctx)
	_set_row("resist", "%.1f%%" % resist,
		_tip("DEBUFF RESISTANCE", "INT × 2.5%", "%d INT × 2.5%% = [b]%.1f%%[/b]\nReduces debuff duration, infliction chance and penalty severity." % [eff_int, resist]))

func _power_tip(title: String, res: Dictionary) -> String:
	if DictUtil.to_bool(res["authored"]):
		return _tip(title, "Authored strike power", "[b]%d[/b]" % DictUtil.to_int(res["attack_power"]))
	if str(res["stat_name"]) == "STR+DEX":
		return _tip(title, "Unarmed: Weapon 0 + Unit Power (STR + DEX)", "STR + DEX = [b]%d[/b]" % DictUtil.to_int(res["attack_power"]))
	var weapon_txt: String = "%.0f" % DictUtil.to_float(res["weapon_power"])
	if not DictUtil.to_bool(res["weapon_fits"]):
		weapon_txt = "0 (weapon doesn't suit this attack)"
	elif DictUtil.to_int(res["tier_boost"]) > 0 or DictUtil.to_int(res["training_pwr"]) > 0 or DictUtil.to_float(res["pen_ratio"]) > 0.0:
		weapon_txt += " (base %d + tier %d + training %d, -%d%% deficit)" % [DictUtil.to_int(res["base_w_power"]), DictUtil.to_int(res["tier_boost"]), DictUtil.to_int(res["training_pwr"]), roundi(DictUtil.to_float(res["pen_ratio"]) * 100.0)]
	return _tip(title, "Weapon Power (base + tier + training, deficit) + Unit Power (stat × %.1f)" % DictUtil.to_float(res["unit_scaling"]),
		"%s + (%d %s × %.1f = %.0f) = [b]%d[/b]" % [weapon_txt, DictUtil.to_int(res["stat_val"]), str(res["stat_name"]), DictUtil.to_float(res["unit_scaling"]), DictUtil.to_float(res["unit_power"]), DictUtil.to_int(res["attack_power"])])

func _tip(title: String, formula: String, calc: String) -> String:
	return "[color=#a5d8ff][b]%s[/b][/color]\n[color=#94a3b8]Formula:[/color] %s\n[color=#94a3b8]Live:[/color] %s" % [title, formula, calc]
