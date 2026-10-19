class_name CombatHeroPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Selected-hero panel (COMBAT-UI3; under the compact party cards, takes the rest of the party column)
# The hero being planned (or the hero under the pointer / keyboard focus): name and stance, speed and reach, the
# plan's AP and next block with its warnings, effects, gear (read-only text: gear changes outside battle), then the
# plan steps and the empty blocks. Height: CombatHudMetrics.hero_panel_h; it scrolls only when its text cannot fit.
# Replaces the ally side of the old floating hover card. All text follows UiTheme.hud_font().
# ==============================================================================

var card: PanelContainer
var scroll: ScrollContainer
var body: VBoxContainer
var shown: Combatant = null
var _width: float = float(CombatHudMetrics.LEFT_W)

func _init() -> void:
	super._init()
	panel_id = "hero"
	panel_title = "Selected hero"
	panel_icon = "👤"
	default_min_size = Vector2(CombatHudMetrics.LEFT_W, 120)
	card = PanelContainer.new()
	card.name = "HeroPanel"
	card.clip_contents = true
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	card.size_flags_vertical = Control.SIZE_EXPAND_FILL
	card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.08, 0.11, 0.20, 0.97), Color(0.16, 0.23, 0.37, 1.0), 6, 8, 6))
	add_child(card)
	scroll = ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	card.add_child(scroll)
	body = VBoxContainer.new()
	body.name = "HeroPanelBody"
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 2)
	scroll.add_child(body)

func _on_bound() -> void:
	session.hover_changed.connect(func(_c: Combatant) -> void: refresh())

func apply_metrics(m: CombatHudMetrics) -> void:
	_width = float(m.left_w)
	custom_minimum_size = Vector2(m.left_w, m.hero_panel_h)
	refresh()

# The hero on the panel: a hovered or focused party member, else the hero being planned.
func pick_unit() -> Combatant:
	var hovered: Combatant = session.hovered_combatant
	if hovered != null and session.is_ally(hovered):
		return hovered
	if session.focused_hero != null:
		return session.focused_hero
	return session.allies[0] if not session.allies.is_empty() else null

func refresh() -> void:
	if session == null or session.grid == null:
		return
	shown = pick_unit()
	for child: Node in body.get_children():
		body.remove_child(child)
		child.queue_free()
	if shown == null:
		return
	var c: Combatant = shown
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 6)
	body.add_child(head)
	var name_lbl: Label = UiTheme.label(c.name, UiTheme.hud_font(), UiTheme.COL_TITLE)
	name_lbl.name = "HeroName"
	name_lbl.clip_text = true
	name_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head.add_child(name_lbl)
	head.add_child(UiTheme.label(CombatTypes.stance_to_string(c.current_stance), UiTheme.hud_font(), UiTheme.COL_MUTED))
	var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
	_line("Speed %d · Reach %s" % [c.speed, str(band.y) if band.x == band.y else "%d-%d" % [band.x, band.y]], UiTheme.COL_MUTED)
	var planning: bool = session.is_planning() and session.plan != null and c.is_active()
	if planning:
		var st: CombatPlan.SimState = session.plan.compile(c)
		_line("AP left %d/%d · next block %d" % [st.ap_left, c.max_ap, mini(st.next_block, session.total_blocks)], Color(0.56, 0.71, 0.96))
		if st.error_index >= 0:
			_line("Step %d can't happen: %s" % [st.error_index + 1, st.error_text], UiTheme.COL_BAD, "HeroWarning")
		elif st.runs_dry():
			_line("AP debt %d: less AP next round" % st.debt(), UiTheme.COL_BAD, "HeroWarning")
	var effects: Array[String] = []
	if c.attack_boost_rounds_remaining > 0:
		effects.append("Focus (%d rds)" % c.attack_boost_rounds_remaining)
	for status: Variant in c.statuses:
		effects.append(StatusEffects.chip_text(c, str(status)))
	if not effects.is_empty():
		_line("Effects: " + ", ".join(PackedStringArray(effects)), Color(0.7, 0.9, 0.7))
	var gear: String = "None"
	if c.inventory != null:
		var wpn: Equipment = c.inventory.get_main_weapon()
		gear = wpn.name if wpn != null else "Unarmed"
	_line("Gear: " + gear, Color(0.9, 0.8, 0.4))   # read-only text: gear changes outside battle
	if not planning:
		return
	var steps: Array[PlanStep] = session.plan.steps_of(c)
	_line("Plan", Color(0.56, 0.71, 0.96))
	if steps.is_empty():
		_line("No plan yet", UiTheme.COL_WARN, "HeroPlanEmpty")
	for i: int in range(steps.size()):
		_line("%d. %s" % [i + 1, steps[i].title()], UiTheme.COL_TEXT)
	var empty: Array[int] = session.plan_empty_blocks(c)
	if not empty.is_empty():
		_line("Empty: %s" % block_list(empty), UiTheme.COL_WARN, "HeroEmptyBlocks")

# "B4-6" / "B2, B5-6": runs of block numbers.
static func block_list(blocks: Array[int]) -> String:
	var parts: Array[String] = []
	var i: int = 0
	while i < blocks.size():
		var j: int = i
		while j + 1 < blocks.size() and blocks[j + 1] == blocks[j] + 1:
			j += 1
		parts.append(("B%d" % blocks[i]) if j == i else ("B%d-%d" % [blocks[i], blocks[j]]))
		i = j + 1
	return ", ".join(PackedStringArray(parts))

func _line(text: String, color: Color, node_name: String = "") -> Label:
	var lbl: Label = UiTheme.label(text, UiTheme.hud_font(), color)
	if not node_name.is_empty():
		lbl.name = node_name
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var inner: float = _width - 2.0 * 8.0 - 2.0
	var fs: int = UiTheme.hud_font()
	var font: Font = lbl.get_theme_font("font")
	var h: float = font.get_multiline_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, inner, fs, -1,
		TextServer.BREAK_WORD_BOUND | TextServer.BREAK_ADAPTIVE | TextServer.BREAK_MANDATORY).y
	lbl.custom_minimum_size = Vector2(inner, ceilf(h))
	body.add_child(lbl)
	return lbl

# All shown text (tests read it).
func lines() -> Array[String]:
	var out: Array[String] = []
	for node: Node in body.find_children("*", "Label", true, false):
		var lbl: Label = node as Label
		if not lbl.is_queued_for_deletion():
			out.append(lbl.text)
	return out
