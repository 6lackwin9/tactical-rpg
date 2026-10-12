class_name UiTheme
extends RefCounted

# ==============================================================================
# Tactical RPG - Gear Dojo Shared Palette & Widget Helpers
# Single source for colors, font sizes, panel styleboxes and small widget
# factories so every dojo component looks and behaves consistently.
# ==============================================================================

const FONT_SM: int = 10
const FONT_MD: int = 11
const FONT_LG: int = 13

const COL_APP_BG: Color = Color(0.05, 0.07, 0.10, 1.0)
const COL_PANEL_BG: Color = Color(0.08, 0.11, 0.16, 0.98)
const COL_CARD_BG: Color = Color(0.11, 0.15, 0.21, 0.95)
const COL_BORDER: Color = Color(0.22, 0.32, 0.46, 0.85)
const COL_HEADER: Color = Color(0.65, 0.85, 1.0)
const COL_TITLE: Color = Color(0.95, 0.85, 0.35)
const COL_TEXT: Color = Color(0.88, 0.92, 0.96)
const COL_MUTED: Color = Color(0.55, 0.65, 0.75)
const COL_GOOD: Color = Color(0.4, 0.9, 0.55)
const COL_BAD: Color = Color(0.97, 0.45, 0.45)
const COL_WARN: Color = Color(0.98, 0.78, 0.3)
const COL_INFO: Color = Color(0.4, 0.8, 1.0)

static func box(
	bg: Color = COL_CARD_BG,
	border: Color = COL_BORDER,
	radius: int = 4,
	margin_h: int = 6,
	margin_v: int = 4,
	border_w: int = 1
) -> StyleBoxFlat:
	var style := StyleBoxFlat.new()
	style.bg_color = bg
	style.border_color = border
	style.set_border_width_all(border_w)
	style.set_corner_radius_all(radius)
	style.content_margin_left = margin_h
	style.content_margin_right = margin_h
	style.content_margin_top = margin_v
	style.content_margin_bottom = margin_v
	return style

static func label(text: String, size: int = FONT_MD, color: Color = COL_TEXT) -> Label:
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", size)
	lbl.add_theme_color_override("font_color", color)
	return lbl

static func header(text: String, color: Color = COL_HEADER) -> Label:
	return label(text, FONT_SM, color)

static func button(text: String, tooltip: String, callback: Callable, size: int = FONT_SM) -> Button:
	var btn := Button.new()
	btn.text = text
	btn.tooltip_text = tooltip
	btn.focus_mode = Control.FOCUS_NONE
	btn.add_theme_font_size_override("font_size", size)
	btn.pressed.connect(callback)
	return btn

static func card(parent: Control, bg: Color = COL_CARD_BG) -> VBoxContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", box(bg))
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(panel)
	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 2)
	panel.add_child(vbox)
	return vbox

static func clear_children(node: Node) -> void:
	for child: Node in node.get_children():
		node.remove_child(child)
		child.queue_free()

static func rarity_color(rarity: Item.ItemRarity) -> Color:
	match rarity:
		Item.ItemRarity.UNCOMMON:
			return Color(0.4, 0.9, 0.5)
		Item.ItemRarity.RARE:
			return Color(0.35, 0.75, 1.0)
		Item.ItemRarity.EPIC:
			return Color(0.75, 0.45, 1.0)
		Item.ItemRarity.LEGENDARY:
			return Color(1.0, 0.8, 0.2)
		_:
			return Color(0.85, 0.85, 0.85)

static func consumable_icon(con: Consumable) -> String:
	match con.consumable_type:
		Consumable.ConsumableType.HEALTH_POTION:
			return "🧪"
		Consumable.ConsumableType.THROWABLE_BOMB:
			return "💣"
		_:
			return "✦"

static func consumable_effect_text(con: Consumable) -> String:
	if con.heal_amount > 0:
		return "+%d HP" % con.heal_amount
	if con.damage_amount > 0:
		return "%d DMG" % con.damage_amount
	return "+%d%% ATK (%d rds)" % [int(roundf(con.buff_percent * 100.0)), con.buff_duration_rounds]

# One-line stat summary for gear cards: power (with tier/deficit), mitigation, burden, 2H, stat bonuses.
static func equipment_summary(eq: Equipment, st: CharacterStats) -> String:
	var parts: Array[String] = []
	if eq.base_power > 0:
		var tier: int = eq.get_unlocked_threshold_power_boost(st, st.mastery) if st != null else 0
		var pen: Dictionary = eq.get_wield_penalty(st, st.mastery) if st != null else {}
		if tier > 0:
			parts.append("Pwr %d (+%d tier)" % [eq.base_power + tier, tier])
		elif dict_bool(pen, "is_penalized"):
			var ratio: float = dict_float(pen, "power_penalty_ratio")
			parts.append("Pwr %d (-%d%% deficit)" % [maxi(1, int(roundf(float(eq.base_power) * (1.0 - ratio)))), int(roundf(ratio * 100.0))])
		else:
			parts.append("Pwr %d" % eq.base_power)
	if eq.mitigation > 0:
		parts.append("Mit +%d" % eq.mitigation)
	if eq.burden > 0:
		parts.append("Burden %d" % eq.burden)
	if eq.base_range > 1:
		parts.append("Range %d" % eq.base_range)
	if eq.is_two_handed:
		parts.append("2H")
	var bonus: String = eq.get_stat_bonuses_display()
	if not bonus.is_empty():
		parts.append(bonus)
	return " • ".join(parts)

static func tooltip_theme() -> Theme:
	var theme := Theme.new()
	theme.set_font_size("font_size", "TooltipLabel", FONT_SM)
	theme.set_stylebox("panel", "TooltipPanel", box(Color(0.08, 0.11, 0.16, 0.97), Color(0.28, 0.38, 0.52, 0.9), 4, 6, 4))
	return theme

# Safe Variant extraction helpers for Dictionary-based breakdown payloads.
static func dict_int(d: Dictionary, key: String, fallback: int = 0) -> int:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is int else fallback

static func dict_float(d: Dictionary, key: String, fallback: float = 0.0) -> float:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is float else fallback

static func dict_bool(d: Dictionary, key: String, fallback: bool = false) -> bool:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is bool else fallback
