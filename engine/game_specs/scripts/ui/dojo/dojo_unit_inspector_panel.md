---
class: DojoUnitInspectorPanel
extends: DojoPanel
description: "Tactical RPG - Dojo Component 1: Unit Inspector Identity, level, 6 base attributes and weapon masteries for the active hero. Hover a stat for its breakdown. Spending attribute points opens an overlay (DojoAllocationPanel) so this panel's layout never changes."
---

# Properties
* `_avatar`: TextureRect
* `_name_lbl`: Label
* `_class_lbl`: Label
* `_level_lbl`: Label
* `_points_btn`: Button
* `_stat_boxes`: Dictionary = {}    # String -> UiTipPanel
* `_stat_values`: Dictionary = {}   # String -> Label
* `_mastery_vbox`: VBoxContainer
* `_texture_cache`: Dictionary = {} # String -> Texture2D

# Methods

### `_init() -> void`
```logic
super._init()
panel_id = "inspector"
panel_title = "Unit Inspector"
panel_short_title = "Inspector"
panel_icon = "🧍"
default_min_size = Vector2(270, 200)

var scroll := ScrollContainer.new()
scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
add_child(scroll)

var vbox := VBoxContainer.new()
vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
vbox.add_theme_constant_override("separation", 6)
scroll.add_child(vbox)

_build_identity(vbox)
_build_stats(vbox)

vbox.add_child(UiTheme.header("PROFICIENCY & PASSIVES"))
_mastery_vbox = VBoxContainer.new()
_mastery_vbox.add_theme_constant_override("separation", 2)
vbox.add_child(_mastery_vbox)
```

### `_build_identity(parent: Control) -> void`
```logic
var card := PanelContainer.new()
card.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_CARD_BG, UiTheme.COL_BORDER, 4, 8, 6))
parent.add_child(card)

var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 8)
card.add_child(row)

_avatar = TextureRect.new()
_avatar.custom_minimum_size = Vector2(46, 46)
_avatar.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
_avatar.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
row.add_child(_avatar)

var info := VBoxContainer.new()
info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
info.add_theme_constant_override("separation", 1)
row.add_child(info)

_name_lbl = UiTheme.label("Hero", UiTheme.FONT_LG, Color.WHITE)
info.add_child(_name_lbl)
_class_lbl = UiTheme.label("Archetype", UiTheme.FONT_SM, UiTheme.COL_MUTED)
info.add_child(_class_lbl)
_level_lbl = UiTheme.label("Level 1 / 10", UiTheme.FONT_SM, UiTheme.COL_WARN)
_level_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
info.add_child(_level_lbl)

_points_btn = UiTheme.button("Unspent Points: 0", "Spend free attribute points", _on_points_pressed)
parent.add_child(_points_btn)
```

### `_build_stats(parent: Control) -> void`
```logic
parent.add_child(UiTheme.header("CORE ATTRIBUTES  (hover for breakdown)"))
var grid := GridContainer.new()
grid.columns = 2
grid.add_theme_constant_override("h_separation", 6)
grid.add_theme_constant_override("v_separation", 4)
parent.add_child(grid)

for def: Dictionary in DojoStatDefs.STATS:
	var key: String = str(def.get("k", ""))
	var col: Color = def.get("c", Color.WHITE)

	var box := UiTipPanel.new()
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	box.mouse_filter = Control.MOUSE_FILTER_STOP
	box.add_theme_stylebox_override("panel", UiTheme.box(Color(0.11, 0.14, 0.20, 0.9), UiTheme.COL_BORDER, 4, 6, 3))
	grid.add_child(box)

	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_theme_constant_override("separation", 3)
	box.add_child(row)

	var tag := UiTheme.label(key, UiTheme.FONT_MD, col)
	tag.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_child(tag)
	row.add_spacer(false)

	var value := UiTheme.label("10", UiTheme.FONT_MD, Color.WHITE)
	value.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_child(value)

	_stat_boxes[key] = box
	_stat_values[key] = value
```

### `_on_points_pressed() -> void`
```logic
if not session.is_allocating_stats:
	session.begin_allocation()
if session.is_allocating_stats:
	overlay_requested.emit(DojoAllocationPanel.new())
```

### `_token_texture(hero: Combatant) -> Texture2D`
```logic
var key: String = hero.name.to_lower()
if not _texture_cache.has(key):
	_texture_cache[key] = UnitArt.token(key, UnitArt.Team.FRIENDLY)
var raw: Variant = _texture_cache[key]
if raw is Texture2D:
	var tex: Texture2D = raw
	return tex
return null
```

### `refresh() -> void`
```logic
if session == null or session.active_hero == null or session.active_hero.stats == null:
	return
var hero: Combatant = session.active_hero
var st: CharacterStats = hero.stats
var ctx: Dictionary = hero.get_weapon_context()

_avatar.texture = _token_texture(hero)
_name_lbl.text = hero.name
_class_lbl.text = "Archetype: %s" % (st.archetype.display_name if st.archetype != null else "Hero")
_level_lbl.text = "Level %d / %d  •  HP %d / %d" % [st.level, CharacterStats.MAX_LEVEL, hero.hp, hero.max_hp]

var free_pts: int = st.unspent_attribute_points
if session.is_allocating_stats:
	_points_btn.text = "Allocating… %d left (click to reopen)" % session.pending_free_points
	_points_btn.add_theme_color_override("font_color", UiTheme.COL_WARN)
else:
	_points_btn.text = "Unspent Points: %d%s" % [free_pts, " (click to allocate)" if free_pts > 0 else ""]
	_points_btn.add_theme_color_override("font_color", UiTheme.COL_GOOD if free_pts > 0 else UiTheme.COL_MUTED)

for key: String in DojoSession.STAT_KEYS:
	var eff: int = st.get_effective_stat(key, ctx)
	var bonus: int = eff - st.get_base_stat(key)
	var text: String = "%d" % eff
	if bonus > 0:
		text += " (+%d)" % bonus
	var pending: int = session.get_pending(key)
	if session.is_allocating_stats and pending > 0:
		text += " [+%d]" % pending
	var value_raw: Variant = _stat_values.get(key, null)
	if value_raw is Label:
		var value_lbl: Label = value_raw
		value_lbl.text = text
	var box_raw: Variant = _stat_boxes.get(key, null)
	if box_raw is UiTipPanel:
		var box: UiTipPanel = box_raw
		box.tooltip_text = _stat_tooltip(key, hero, ctx)

_render_proficiency(hero)
```

### `_render_proficiency(hero: Combatant) -> void`
```logic
UiTheme.clear_children(_mastery_vbox)
var st: CharacterStats = hero.stats
if st.proficiency == null or (st.proficiency.entries.is_empty() and st.proficiency.bonus_levels.is_empty()):
	_mastery_vbox.add_child(UiTheme.label("— No trained gear —", UiTheme.FONT_SM, UiTheme.COL_MUTED))
else:
	var types: Array[String] = []
	for key: Variant in st.proficiency.entries:
		types.append(str(key))
	for key: Variant in st.proficiency.bonus_levels:
		if not types.has(str(key)):
			types.append(str(key))
	for gear: String in types:
		var level: int = st.proficiency.level_of(gear)
		var bonus: int = DictUtil.int_of(st.proficiency.bonus_levels, gear)
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		row.add_child(UiTheme.label(gear.capitalize(), UiTheme.FONT_SM, UiTheme.COL_TEXT))
		var text: String = "Lv %d%s  •  scaling x%.2f  •  +%d Wep Power, -%d Req" % [level, (" (+%d passive)" % bonus) if bonus > 0 else "", st.proficiency.scaling_of(gear), st.proficiency.get_weapon_power_bonus(gear), st.proficiency.get_weapon_requirement_discount(gear)]
		var detail: Label = UiTheme.label(text, 9, UiTheme.COL_WARN)
		detail.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		detail.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		detail.custom_minimum_size = Vector2(60, 0)
		row.add_child(detail)
		_mastery_vbox.add_child(row)
if not hero.passives.is_empty():
	var names: Array[String] = []
	for id: String in hero.passives:
		names.append(str(ContentCatalog.get_entry("passives", id).get("name", id)))
	var passives_lbl: Label = UiTheme.label("Passives: " + ", ".join(names), UiTheme.FONT_SM, UiTheme.COL_INFO)
	passives_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_mastery_vbox.add_child(passives_lbl)
```

### `_stat_tooltip(key: String, hero: Combatant, ctx: Dictionary) -> String`
```logic
var st: CharacterStats = hero.stats
var role: String = ""
var full_name: String = key
var kind: String = "Passive"
for def: Dictionary in DojoStatDefs.STATS:
	if str(def.get("k", "")) == key:
		role = str(def.get("role", ""))
		full_name = str(def.get("name", key))
		kind = str(def.get("kind", "Passive"))
var base_val: int = st.get_base_stat(key)
var eff_val: int = st.get_effective_stat(key, ctx)

var lines: Array[String] = []
lines.append("[color=#a5d8ff][b]%s (%s)[/b][/color]  [color=#94a3b8]%s stat[/color]" % [full_name, key, kind])
lines.append("[color=#94a3b8]%s[/color]" % role)
lines.append("Base [b]%d[/b]  (Lv %d %s)  →  Effective [color=#4ade80][b]%d[/b][/color]" % [base_val, st.level, st.archetype.display_name if st.archetype != null else "Hero", eff_val])

var bonus_lines: Array[String] = []
for mod: StatModifier in st.modifiers:
	if mod.stat_name.to_upper() == key and mod.source_id.begins_with("passive:") and mod.matches_context(ctx):
		bonus_lines.append("• %s: %+d" % [str(ContentCatalog.get_entry("passives", mod.source_id.trim_prefix("passive:")).get("name", mod.source_id)), mod.flat_bonus])
if hero.inventory != null:
	for eq: Equipment in hero.inventory._iter_equipped_gear():
		var gear_bonus: int = 0
		var raw_bonus: Variant = eq.stat_bonuses.get(key, 0)
		if raw_bonus is int:
			gear_bonus = raw_bonus
		if gear_bonus != 0:
			bonus_lines.append("• %s: %+d" % [eq.name, gear_bonus])
for mod: StatModifier in st.modifiers:
	if mod.source_id != "gear" and mod.stat_name.to_upper() == key and mod.matches_context(ctx):
		bonus_lines.append("• Buff (%s): %+d" % [mod.source_id, mod.flat_bonus])
if not bonus_lines.is_empty():
	lines.append("[color=#fbbf24]Bonuses:[/color]\n%s" % "\n".join(bonus_lines))
return "\n".join(lines)
```

