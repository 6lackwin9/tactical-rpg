class_name DojoStagePanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 3: Combat Region
# Hero token vs training dummy on a target mat with live vitals, plus floating
# damage text and strike/glow animations driven by DojoSession.stage_fx.
# ==============================================================================

const TOKEN_SIZE: float = 56.0
const HERO_X: float = 0.27
const DUMMY_X: float = 0.73
const MAT_Y: float = 0.5

class TokenView extends RefCounted:
	var root: Control
	var icon: TextureRect
	var name_lbl: Label
	var mini_bar: ProgressBar

class VitalsView extends RefCounted:
	var hp_lbl: Label
	var bar: ProgressBar
	var status: Label

var _mat: Control
var _hero: TokenView
var _dummy: TokenView
var _hero_vitals: VitalsView
var _dummy_vitals: VitalsView
var _texture_cache: Dictionary = {} # String -> Texture2D

func _init() -> void:
	super._init()
	panel_id = "stage"
	panel_title = "Combat Region"
	panel_short_title = "Combat"
	panel_icon = "🎯"
	default_min_size = Vector2(280, 230)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	add_child(vbox)

	_build_mat(vbox)

	var cards := HBoxContainer.new()
	cards.add_theme_constant_override("separation", 6)
	vbox.add_child(cards)
	_hero_vitals = _build_vitals_card(cards, "HERO", Color(0.4, 0.8, 1.0))
	_dummy_vitals = _build_vitals_card(cards, "TRAINING DUMMY", Color(0.95, 0.85, 0.4))

	var controls := HFlowContainer.new()
	controls.add_theme_constant_override("h_separation", 4)
	vbox.add_child(controls)
	controls.add_child(UiTheme.button("🛡 Toggle Dummy Defend", "Dummy guards (-50%, Defensive stance) or stands Neutral", func() -> void: session.toggle_dummy_defense()))
	controls.add_child(UiTheme.button("💖 Full Heal Dummy", "Restore the dummy to full HP", func() -> void: session.heal_dummy()))
	controls.add_child(UiTheme.button("⚔ Dummy Attacks", "The dummy strikes the active hero with its chosen skill (Console: Dummy Attacks)", func() -> void: session.dummy_attack()))

func _build_mat(parent: Control) -> void:
	var mat_panel := PanelContainer.new()
	mat_panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	mat_panel.custom_minimum_size = Vector2(0, 150)
	mat_panel.add_theme_stylebox_override("panel", UiTheme.box(Color(0.12, 0.16, 0.22, 0.95), Color(0.25, 0.38, 0.55, 0.85), 6, 0, 0, 2))
	parent.add_child(mat_panel)

	_mat = Control.new()
	_mat.clip_contents = true
	_mat.resized.connect(_layout_tokens)
	mat_panel.add_child(_mat)

	_hero = _make_token("Hero", Color(0.4, 0.8, 1.0))
	_dummy = _make_token("Test Dummy", Color(0.95, 0.85, 0.4))
	_dummy.icon.texture = UnitArt.token("dummy", UnitArt.Team.NEUTRAL)

func _make_token(label_text: String, label_color: Color) -> TokenView:
	var view := TokenView.new()
	view.root = Control.new()
	_mat.add_child(view.root)

	view.icon = TextureRect.new()
	view.icon.position = Vector2(-TOKEN_SIZE / 2.0, -TOKEN_SIZE / 2.0)
	view.icon.size = Vector2(TOKEN_SIZE, TOKEN_SIZE)
	view.icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	view.icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	view.icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	view.root.add_child(view.icon)

	view.name_lbl = UiTheme.label(label_text, UiTheme.FONT_SM, label_color)
	view.name_lbl.position = Vector2(-45, TOKEN_SIZE / 2.0 + 1)
	view.name_lbl.custom_minimum_size = Vector2(90, 14)
	view.name_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	view.root.add_child(view.name_lbl)

	view.mini_bar = _make_hp_bar(5)
	view.mini_bar.position = Vector2(-28, TOKEN_SIZE / 2.0 + 17)
	view.mini_bar.size = Vector2(56, 5)
	view.root.add_child(view.mini_bar)
	return view

func _make_hp_bar(height: float) -> ProgressBar:
	var bar := ProgressBar.new()
	bar.show_percentage = false
	bar.custom_minimum_size = Vector2(0, height)
	bar.add_theme_stylebox_override("background", UiTheme.box(Color(0.18, 0.08, 0.08, 0.9), Color(0, 0, 0, 0), 3, 0, 0, 0))
	bar.add_theme_stylebox_override("fill", UiTheme.box(Color(0.25, 0.75, 0.35), Color(0, 0, 0, 0), 3, 0, 0, 0))
	return bar

func _build_vitals_card(parent: Control, title: String, title_color: Color) -> VitalsView:
	var view := VitalsView.new()
	var card := UiTheme.card(parent)
	var top := HBoxContainer.new()
	card.add_child(top)
	top.add_child(UiTheme.label(title, UiTheme.FONT_SM, title_color))
	top.add_spacer(false)
	view.hp_lbl = UiTheme.label("0 / 0", UiTheme.FONT_SM, Color.WHITE)
	top.add_child(view.hp_lbl)

	view.bar = _make_hp_bar(7)
	card.add_child(view.bar)

	view.status = UiTheme.label("", 9, UiTheme.COL_GOOD)
	view.status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	card.add_child(view.status)
	return view

func _on_bound() -> void:
	session.stage_fx.connect(_on_stage_fx)

func _hero_home() -> Vector2:
	return Vector2(_mat.size.x * HERO_X, _mat.size.y * MAT_Y - 10.0)

func _dummy_home() -> Vector2:
	return Vector2(_mat.size.x * DUMMY_X, _mat.size.y * MAT_Y - 10.0)

func _layout_tokens() -> void:
	if _hero == null or _dummy == null:
		return
	_hero.root.position = _hero_home()
	_dummy.root.position = _dummy_home()

func _hero_texture(hero: Combatant) -> Texture2D:
	var key: String = hero.name.to_lower()
	if not _texture_cache.has(key):
		_texture_cache[key] = UnitArt.token(key, UnitArt.Team.FRIENDLY)
	var raw: Variant = _texture_cache[key]
	if raw is Texture2D:
		var tex: Texture2D = raw
		return tex
	return null

func refresh() -> void:
	if session == null or session.active_hero == null or session.test_dummy == null:
		return
	var hero: Combatant = session.active_hero
	var dummy: Combatant = session.test_dummy

	_hero.icon.texture = _hero_texture(hero)
	_hero.name_lbl.text = hero.name
	_set_vitals(_hero_vitals, _hero.mini_bar, hero)
	var hero_bits: Array[String] = ["%s stance" % str(CombatTypes.StanceType.keys()[hero.current_stance]).capitalize()]
	if hero.inventory != null and hero.inventory.needs_weapon_switch():
		var ready_w: Equipment = hero.inventory.get_active_weapon()
		hero_bits.append("🗡 %s ready" % ready_w.name)
	if hero.defense_mitigation_ratio > 0.0:
		hero_bits.append("🛡 Guard %d%%" % int(roundf(hero.defense_mitigation_ratio * 100.0)))
	if hero.attack_boost_rounds_remaining > 0:
		hero_bits.append("✦ ATK ×%.1f (%d rds)" % [hero.attack_boost_multiplier, hero.attack_boost_rounds_remaining])
	hero_bits.append("AP %d/%d" % [hero.ap, hero.max_ap])
	for status: Variant in hero.statuses:
		hero_bits.append("⚡ %s" % StatusEffects.chip_text(hero, str(status)))
	if not hero.is_alive():
		hero_bits = ["💀 Down - use Heal Hero"]
	_hero_vitals.status.text = " • ".join(hero_bits)

	_set_vitals(_dummy_vitals, _dummy.mini_bar, dummy)
	var mit_pct: int = int(roundf(dummy.defense_mitigation_ratio * 100.0))
	var dummy_bits: Array[String] = ["%s stance" % CombatTypes.stance_to_string(dummy.current_stance), "⚔ %d power" % DamageCalculator.power_of(dummy)]
	if mit_pct > 0:
		dummy_bits.append("🛡 Guard %d%%" % mit_pct)
	for status: Variant in dummy.statuses:
		dummy_bits.append("⚡ %s" % StatusEffects.chip_text(dummy, str(status)))
	for status: Variant in dummy.pending_statuses:
		dummy_bits.append("%s next round" % StatusEffects.display_name(str(status)))
	if not dummy.is_alive():
		_dummy_vitals.status.text = "💀 Defeated - use Full Heal"
		_dummy_vitals.status.add_theme_color_override("font_color", UiTheme.COL_BAD)
	else:
		_dummy_vitals.status.text = " • ".join(dummy_bits)
		_dummy_vitals.status.add_theme_color_override("font_color", UiTheme.COL_INFO if mit_pct > 0 else UiTheme.COL_GOOD)
	_layout_tokens()

func _set_vitals(vitals: VitalsView, mini_bar: ProgressBar, unit: Combatant) -> void:
	vitals.bar.max_value = maxi(1, unit.max_hp)
	vitals.bar.value = unit.hp
	vitals.hp_lbl.text = "%d / %d HP" % [unit.hp, unit.max_hp]
	mini_bar.max_value = maxi(1, unit.max_hp)
	mini_bar.value = unit.hp

# ------------------------------------------------------------------------------
# Visual FX
# ------------------------------------------------------------------------------

func _on_stage_fx(kind: String, on_hero: bool, text: String, color: Color) -> void:
	if not is_inside_tree():
		return
	var subject: TokenView = _hero if on_hero else _dummy
	match kind:
		"strike":
			var victim: TokenView = _dummy if on_hero else _hero
			_lunge(subject, victim)
			_spawn_text(victim.root, text, color)
		"glow":
			_glow(subject.root, color)
			_spawn_text(subject.root, text, color)
		_:
			_spawn_text(subject.root, text, color)

func _lunge(attacker: TokenView, victim: TokenView) -> void:
	var dir: float = signf(victim.root.position.x - attacker.root.position.x)
	var home: Vector2 = _hero_home() if attacker == _hero else _dummy_home()
	var victim_home: Vector2 = _hero_home() if victim == _hero else _dummy_home()
	var tw := create_tween()
	tw.tween_property(attacker.root, "position", home + Vector2(26.0 * dir, 0), 0.10).set_trans(Tween.TRANS_QUAD)
	tw.tween_property(attacker.root, "position", home, 0.12).set_trans(Tween.TRANS_QUAD)
	var tw2 := create_tween()
	tw2.tween_interval(0.08)
	tw2.tween_property(victim.root, "position", victim_home + Vector2(6, -3), 0.05)
	tw2.tween_property(victim.root, "position", victim_home + Vector2(-6, 3), 0.05)
	tw2.tween_property(victim.root, "position", victim_home, 0.05)

func _glow(target: Control, glow_col: Color) -> void:
	var tw := create_tween()
	tw.tween_property(target, "modulate", Color(glow_col.r * 1.6, glow_col.g * 1.6, glow_col.b * 1.6), 0.15)
	tw.tween_property(target, "modulate", Color.WHITE, 0.25)

func _spawn_text(anchor: Control, text: String, color: Color) -> void:
	var lbl := UiTheme.label(text, UiTheme.FONT_LG, color)
	lbl.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.9))
	lbl.add_theme_constant_override("outline_size", 3)
	lbl.position = anchor.position + Vector2(-34, -62)
	lbl.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_mat.add_child(lbl)
	var tw := create_tween()
	tw.parallel().tween_property(lbl, "position:y", lbl.position.y - 28.0, 0.8).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tw.parallel().tween_property(lbl, "modulate:a", 0.0, 0.8)
	tw.tween_callback(lbl.queue_free)
