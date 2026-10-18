class_name CombatEnemyCardPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Merged enemy card (COMBAT-UI3; replaces the forecast panel and the floating hover card, HOVER1-U)
# One fixed card under the hostile cards for one enemy: the hovered one (grid token or hostile card, also on
# keyboard focus), else the selected (pinned) one, else the planning hero's forecast target. Lines:
#   name + state word (morale state when the battle has morale, else defeated / withdrew / the stance)
#   HP · Speed · Reach
#   Next: <tell> (planning; "Targets you in B<n>" when it aims at the selected hero; a read foe's whole plan)
#   Your <hero>: deals about X · takes about Y (the basic strike both ways, guard included)
#   Can use: <skill names>
# Engine detail (formulas, multipliers, AI thresholds) stays out. Height: CombatHudMetrics.enemy_card_h (it scrolls
# only when the text cannot fit). All text follows UiTheme.hud_font().
# ==============================================================================

var card: PanelContainer
var scroll: ScrollContainer
var body: VBoxContainer
var shown: Combatant = null     # the enemy on the card (null = none in view)
var name_lbl: Label
var state_lbl: Label
var stats_lbl: Label
var next_lbl: Label
var threat_lbl: Label
var read_lbl: Label
var plan_lbl: Label
var versus_lbl: Label
var skills_lbl: Label
var _width: float = float(CombatHudMetrics.RIGHT_W)

func _init() -> void:
	super._init()
	panel_id = "enemy_card"
	panel_title = "Enemy card"
	panel_icon = "🎯"
	default_min_size = Vector2(CombatHudMetrics.RIGHT_W, 160)
	card = PanelContainer.new()
	card.name = "EnemyCard"
	card.clip_contents = true
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	card.size_flags_vertical = Control.SIZE_EXPAND_FILL
	card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.04, 0.06, 0.11, 0.98), Color(0.94, 0.55, 0.53, 0.9), 6, 8, 6))
	add_child(card)
	scroll = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	card.add_child(scroll)
	body = VBoxContainer.new()
	body.name = "EnemyCardBody"
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 2)
	scroll.add_child(body)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	body.add_child(head)
	name_lbl = UiTheme.label("", UiTheme.hud_font(), Color.WHITE)
	name_lbl.name = "EnemyName"
	name_lbl.clip_text = true
	name_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(name_lbl)
	state_lbl = UiTheme.label("", UiTheme.hud_font(), UiTheme.COL_MUTED)
	state_lbl.name = "EnemyState"
	head.add_child(state_lbl)
	stats_lbl = _wrapped("EnemyStats", UiTheme.COL_MUTED)
	next_lbl = _wrapped("EnemyNext", UiTheme.COL_WARN)
	threat_lbl = _wrapped("EnemyThreat", Color(1.0, 0.5, 0.45))
	read_lbl = _wrapped("EnemyReadBy", Color(0.55, 0.9, 1.0))
	plan_lbl = _wrapped("EnemyPlan", Color(0.85, 0.9, 0.95))
	versus_lbl = _wrapped("EnemyVersus", Color(0.56, 0.71, 0.96))
	skills_lbl = _wrapped("EnemySkills", UiTheme.COL_MUTED)

func _wrapped(node_name: String, color: Color) -> Label:
	var lbl: Label = UiTheme.label("", UiTheme.hud_font(), color)
	lbl.name = node_name
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(lbl)
	return lbl

func _on_bound() -> void:
	session.hover_changed.connect(func(_c: Combatant) -> void: refresh())

# The fixed card size (CombatHudMetrics): its width sets where the lines wrap.
func apply_metrics(m: CombatHudMetrics) -> void:
	_width = float(m.right_w)
	custom_minimum_size = Vector2(m.right_w, m.enemy_card_h)
	refresh()

# Which enemy the card shows: hovered, else selected (pinned), else the planning hero's forecast target, else the first in view.
func pick_unit() -> Combatant:
	var hovered: Combatant = session.hovered_combatant
	if hovered != null and not session.is_ally(hovered):
		return hovered
	var selected: Combatant = session.selected_combatant
	if selected != null and not session.is_ally(selected):
		return selected
	var target: Combatant = session.forecast_target(session.focused_hero) if session.focused_hero != null else null
	if target != null and not session.is_ally(target):
		return target
	var in_view: Array[Combatant] = session.enemies_in_view()
	return in_view[0] if not in_view.is_empty() else null

func refresh() -> void:
	if session == null or session.grid == null:
		return
	shown = pick_unit()
	var fs: int = UiTheme.hud_font()
	for lbl: Label in [name_lbl, state_lbl, stats_lbl, next_lbl, threat_lbl, read_lbl, plan_lbl, versus_lbl, skills_lbl]:
		lbl.add_theme_font_size_override("font_size", fs)
	if shown == null:
		name_lbl.text = "No hostiles in view"
		for lbl: Label in [state_lbl, stats_lbl, next_lbl, threat_lbl, read_lbl, plan_lbl, versus_lbl, skills_lbl]:
			_put(lbl, "")
		return
	var c: Combatant = shown
	name_lbl.text = c.name
	name_lbl.tooltip_text = c.name
	state_lbl.text = state_word(c)
	var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
	var reach: String = str(band.y) if band.x == band.y else "%d-%d" % [band.x, band.y]
	_put(stats_lbl, "HP %d/%d · Speed %d · Reach %s" % [c.hp, c.max_hp, c.speed, reach])
	var rec: Dictionary = session.intent_of(c)
	var next_text: String = ""
	var threat: String = ""
	var read_text: String = ""
	var plan_lines: Array[String] = []
	if not rec.is_empty():
		var kind: String = str(rec["kind"])
		next_text = IntentText.first_line(rec) if kind == IntentRules.ARRIVING or kind == IntentRules.IDLE else "Next: " + IntentText.first_line(rec)
		var hero: Combatant = session.focused_hero
		if hero != null and rec.get("target", null) == hero:
			threat = "Targets you in %s" % IntentText.block_text(rec.get("blocks", Vector2i.ZERO))
		if DictUtil.to_bool(rec["revealed"]):
			read_text = IntentText.read_by(rec)
			for raw: Variant in DictUtil.array_of(rec["plan"]):
				plan_lines.append(IntentText.plan_line(DictUtil.dict_of(raw)))
	_put(next_lbl, next_text)
	_put(threat_lbl, threat)
	_put(read_lbl, read_text)
	_put(plan_lbl, "\n".join(PackedStringArray(plan_lines)))
	_put(versus_lbl, versus_text(c))
	var skills: Array[String] = skill_names(c)
	_put(skills_lbl, ("Can use: " + ", ".join(PackedStringArray(skills))) if not skills.is_empty() else "")

# A wrapped line; its height is measured for the card's width so the layout never depends on the pass order.
func _put(lbl: Label, text: String) -> void:
	lbl.text = text
	lbl.visible = not text.is_empty()
	if text.is_empty():
		return
	var inner: float = _width - 2.0 * 8.0 - 2.0
	var fs: int = lbl.get_theme_font_size("font_size")
	var font: Font = lbl.get_theme_font("font")
	var h: float = font.get_multiline_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, inner, fs, -1,
		TextServer.BREAK_WORD_BOUND | TextServer.BREAK_ADAPTIVE | TextServer.BREAK_MANDATORY).y
	lbl.custom_minimum_size = Vector2(inner, ceilf(h))

# The state word beside the name: the morale state (MORALE1) when the battle has one, else defeated / withdrew, else the stance.
func state_word(c: Combatant) -> String:
	if not c.is_alive():
		return "Defeated"
	if c.is_withdrawn:
		return "Withdrew"
	var morale: String = session.encounter.morale_state_of(c)
	if not morale.is_empty():
		return morale.capitalize()
	return CombatTypes.stance_to_string(c.current_stance)

# "Your Warrior: deals about 49 · takes about 25": the basic strike both ways against the current guard ("" when not both on the field).
func versus_text(c: Combatant) -> String:
	var hero: Combatant = session.focused_hero
	if hero == null or not hero.is_active() or not c.is_active():
		return ""
	if not session.grid.has_combatant(hero) or not session.grid.has_combatant(c):
		return ""
	var basic: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
	var deals: int = maxi(1, roundi(float(DamageCalculator.expected(hero, c, basic, basic.stance())) * (1.0 - c.defense_mitigation_ratio)))
	var takes: int = maxi(1, roundi(float(DamageCalculator.expected(c, hero, basic, basic.stance())) * (1.0 - hero.defense_mitigation_ratio)))
	return "Your %s: deals about %d · takes about %d" % [hero.name, deals, takes]

# The skills the enemy may use this battle (its AI skill ceiling, what it knows and can wield), one name per family,
# system skills (move, rest) and the off-hand jab of a melee unit left out; at most SKILL_NAMES_MAX names, then "+n more".
func skill_names(c: Combatant) -> Array[String]:
	var cap: int = session.encounter.profile_of(c).skill_cap
	var names: Array[String] = []
	var families: Array[String] = []
	for s: SkillDef in SkillCatalog.all():
		if s.system or s.level > cap or not c.can_use(s) or (s.is_off_hand() and c.attack_range <= 1):
			continue
		var fam: String = SkillCatalog.family_of(s.id)
		if families.has(fam):
			continue
		families.append(fam)
		names.append(s.display_name)
	if names.size() <= CombatHudMetrics.SKILL_NAMES_MAX:
		return names
	var shown_names: Array[String] = []
	for i: int in range(CombatHudMetrics.SKILL_NAMES_MAX):
		shown_names.append(names[i])
	shown_names.append("+%d more" % (names.size() - CombatHudMetrics.SKILL_NAMES_MAX))
	return shown_names

# True when the card's lines fit its fixed height (no scrolling needed).
func content_fits() -> bool:
	return body.get_combined_minimum_size().y + 2.0 * 6.0 + 2.0 <= custom_minimum_size.y + 0.5

# All shown text, one line per label (tests and the renders read it).
func lines() -> Array[String]:
	var out: Array[String] = []
	for lbl: Label in [name_lbl, state_lbl, stats_lbl, next_lbl, threat_lbl, read_lbl, plan_lbl, versus_lbl, skills_lbl]:
		if lbl.visible and not lbl.text.is_empty():
			out.append(lbl.text)
	return out
