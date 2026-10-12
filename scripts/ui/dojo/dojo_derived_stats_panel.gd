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
	{"id": "mit", "label": "Flat Mitigation", "color": Color(0.7, 0.85, 1.0)},
	{"id": "burden", "label": "Armor Burden", "color": Color(0.9, 0.8, 0.35)},
	{"id": "range", "label": "Attack Range", "color": Color(0.6, 0.9, 0.8)},
	{"id": "crit", "label": "Crit Hit Rate", "color": Color(0.95, 0.75, 0.3)},
	{"id": "resist", "label": "Debuff Resist", "color": Color(0.7, 0.5, 0.95)},
]

var _rows: Dictionary = {}    # String -> DojoTipPanel
var _values: Dictionary = {}  # String -> Label
var _burden_bar: ProgressBar

func _init() -> void:
	super._init()
	panel_id = "derived"
	panel_title = "Derived Stats"
	panel_short_title = "Derived"
	panel_icon = "📈"
	default_min_size = Vector2(230, 130)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 3)
	scroll.add_child(vbox)

	vbox.add_child(DojoTheme.header("HOVER A ROW FOR ITS FORMULA", DojoTheme.COL_MUTED))

	for def: Dictionary in ROW_DEFS:
		var id: String = str(def.get("id", ""))
		var col: Color = def.get("color", Color.WHITE)

		var row := DojoTipPanel.new()
		row.mouse_filter = Control.MOUSE_FILTER_STOP
		row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_theme_stylebox_override("panel", DojoTheme.box(Color(0.11, 0.14, 0.20, 0.85), Color(0.18, 0.26, 0.38, 0.7), 3, 6, 3))
		vbox.add_child(row)

		var inner := VBoxContainer.new()
		inner.mouse_filter = Control.MOUSE_FILTER_PASS
		inner.add_theme_constant_override("separation", 1)
		row.add_child(inner)

		var line := HBoxContainer.new()
		line.mouse_filter = Control.MOUSE_FILTER_PASS
		inner.add_child(line)

		var tag := DojoTheme.label(str(def.get("label", "")), DojoTheme.FONT_SM, col)
		tag.mouse_filter = Control.MOUSE_FILTER_PASS
		line.add_child(tag)
		line.add_spacer(false)

		var value := DojoTheme.label("-", DojoTheme.FONT_SM, Color.WHITE)
		value.mouse_filter = Control.MOUSE_FILTER_PASS
		line.add_child(value)

		if id == "burden":
			_burden_bar = ProgressBar.new()
			_burden_bar.custom_minimum_size = Vector2(0, 5)
			_burden_bar.show_percentage = false
			_burden_bar.mouse_filter = Control.MOUSE_FILTER_PASS
			_burden_bar.add_theme_stylebox_override("background", DojoTheme.box(Color(0.15, 0.12, 0.08), Color(0, 0, 0, 0), 2, 0, 0, 0))
			_burden_bar.add_theme_stylebox_override("fill", DojoTheme.box(Color(0.9, 0.75, 0.3), Color(0, 0, 0, 0), 2, 0, 0, 0))
			inner.add_child(_burden_bar)

		_rows[id] = row
		_values[id] = value

func _set_row(id: String, text: String, tip: String) -> void:
	var value_raw: Variant = _values.get(id, null)
	if value_raw is Label:
		var value_lbl: Label = value_raw
		value_lbl.text = text
	var row_raw: Variant = _rows.get(id, null)
	if row_raw is DojoTipPanel:
		var row: DojoTipPanel = row_raw
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
	var burden: int = inv.get_total_armor_burden() if inv != null else 0
	var carry_max: int = st.get_carry_weight_capacity(ctx)

	# --- Offensive power ---
	var melee: int = hero.get_effective_power(false)
	var melee_wep: String = "0 (no melee weapon)"
	if main_w != null and main_w.is_melee_weapon():
		melee_wep = "%d" % main_w.get_effective_equipment_power(st, st.mastery)
	_set_row("melee", "%d ATK%s" % [melee, " (ranged eqpd)" if (main_w != null and main_w.is_ranged_weapon()) else ""],
		_tip("MELEE POWER", "Weapon Power + STR × 1.5", "%s + (%d STR × 1.5 = %d) = [b]%d[/b]" % [melee_wep, eff_str, int(roundf(float(eff_str) * 1.5)), melee]))

	var ranged: int = hero.get_effective_power(true)
	var ranged_wep: String = "0 (no ranged weapon)"
	if main_w != null and main_w.is_ranged_weapon():
		ranged_wep = "%d" % main_w.get_effective_equipment_power(st, st.mastery)
	_set_row("ranged", "%d ATK%s" % [ranged, " (melee eqpd)" if (main_w != null and main_w.is_melee_weapon()) else ""],
		_tip("RANGED POWER", "Weapon Power + DEX × 1.5", "%s + (%d DEX × 1.5 = %d) = [b]%d[/b]" % [ranged_wep, eff_dex, int(roundf(float(eff_dex) * 1.5)), ranged]))

	if off_w != null:
		var off_stat: String = off_w.scaling_stat
		var off_val: int = st.get_effective_stat(off_stat, ctx)
		var off_pwr: int = hero.get_offhand_power()
		var needs_switch: bool = inv.needs_weapon_switch()
		var switch_note: String = ""
		if needs_switch:
			switch_note = "\n[color=#fbbf24]2H main weapon equipped: Switch Weapon (1 block) to use the off-hand weapon.[/color]"
		_set_row("offhand", "%d ATK%s" % [off_pwr, " (needs switch)" if needs_switch else ""],
			_tip("OFF-HAND POWER", "Off-hand Weapon Power + %s × 1.5" % off_stat, "%d + (%d %s × 1.5 = %d) = [b]%d[/b]%s" % [off_w.get_effective_equipment_power(st, st.mastery), off_val, off_stat, int(roundf(float(off_val) * 1.5)), off_pwr, switch_note]))
	else:
		_set_row("offhand", "-", _tip("OFF-HAND POWER", "Off-hand Weapon Power + Scaling Stat × 1.5", "No offensive off-hand weapon equipped."))

	# --- Defensive & utility ---
	_set_row("speed", "%d (burden -%d)" % [hero.speed, burden],
		_tip("INITIATIVE SPEED", "8 + DEX - Burden", "8 + %d DEX - %d burden = [b]%d[/b]" % [eff_dex, burden, st.get_initiative_speed(burden, ctx)]))
	_set_row("hp", "%d" % st.get_max_hp(ctx),
		_tip("MAXIMUM HEALTH", "60 + CONS × 8 + STR × 2", "60 + %d (CONS × 8) + %d (STR × 2) = [b]%d[/b]" % [eff_cons * 8, eff_str * 2, st.get_max_hp(ctx)]))

	var arm_mit: int = inv.get_total_armor_mitigation() if inv != null else 0
	var shd_mit: int = inv.get_total_shield_mitigation() if inv != null else 0
	var gear_mit: int = inv.get_total_flat_mitigation() if inv != null else 0
	var str_mit: int = int(roundf(float(eff_str) * 0.25))
	_set_row("mit", "+%d (gear %d, STR %d)" % [gear_mit + str_mit, gear_mit, str_mit],
		_tip("FLAT MITIGATION", "Gear Mitigation + STR × 0.25", "%d (armor %d + off-hand %d) + %d (STR × 0.25) = [b]+%d[/b]" % [gear_mit, arm_mit, shd_mit, str_mit, gear_mit + str_mit]))

	var status: String = "[color=#4ade80]Within limit (spare %d)[/color]" % (carry_max - burden) if burden <= carry_max else "[color=#f87171]OVERENCUMBERED[/color]"
	_set_row("burden", "%d / %d" % [burden, carry_max],
		_tip("BURDEN & CARRY CAPACITY", "Capacity = 30 + STR × 3 + CONS", "30 + %d + %d = [b]%d[/b]\nEquipped burden %d / %d - %s" % [eff_str * 3, eff_cons, carry_max, burden, carry_max, status]))
	_burden_bar.max_value = maxi(1, carry_max)
	_burden_bar.value = burden

	_set_row("range", "%d tile(s)" % hero.attack_range,
		_tip("ATTACK RANGE", "Weapon Range + 1 if PERC ≥ 18", "Weapon range %d, PERC %d → [b]%d[/b]" % [main_w.base_range if main_w != null else 1, st.get_effective_stat("PERC", ctx), hero.attack_range]))

	var crit_rate: float = st.get_crit_rate(ctx)
	var crit_mult: float = st.get_crit_damage_multiplier(ctx)
	_set_row("crit", "%.1f%% (%.1f×)" % [crit_rate, crit_mult],
		_tip("CRIT RATE & DAMAGE", "5%% + DEX × 1.5%%  •  damage ×%.1f" % crit_mult, "5.0%% + (%d DEX × 1.5%% = %.1f%%) = [b]%.1f%%[/b]" % [eff_dex, float(eff_dex) * 1.5, crit_rate]))

	var resist: float = st.get_debuff_resistance(ctx)
	_set_row("resist", "%.1f%%" % resist,
		_tip("DEBUFF RESISTANCE", "INT × 2.5%", "%d INT × 2.5%% = [b]%.1f%%[/b]\nReduces debuff duration, infliction chance and penalty severity." % [eff_int, resist]))

func _tip(title: String, formula: String, calc: String) -> String:
	return "[color=#a5d8ff][b]%s[/b][/color]\n[color=#94a3b8]Formula:[/color] %s\n[color=#94a3b8]Live:[/color] %s" % [title, formula, calc]
