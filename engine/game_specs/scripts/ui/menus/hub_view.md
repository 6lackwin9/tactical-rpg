---
class: HubView
extends: VBoxContainer
description: "Tactical RPG - Game menu view (UI1-L hub; UIR-2, UI-REVIEW1 Q1 B): a ScreenFrame with the world as title, where the party is, one tile per member (name, class, level, HP; reserves muted), the entries (Party, Journal, Save game, Load game, Settings, Quit to main menu) in a two-column grid, and a detail line that tells what the focused entry does or why it can't open. Return to map (Back) closes the menu over the paused map. Thin: everything comes from HubModel."
---

# Properties
* `TILE_COLUMNS`: int = 4 (const)
* `ENTRY_COLUMNS`: int = 2 (const)
* `TILE_W`: int = 200 (const)
* `detail`: Label = null

# Methods

### `build(host: MainMenuView, model: RefCounted) -> void`
```logic
var m: HubModel = model as HubModel
size_flags_horizontal = Control.SIZE_EXPAND_FILL
size_flags_vertical = Control.SIZE_EXPAND_FILL
var back: Button = null
if m.can_return():
	back = MainMenuView.button("Return to map", host.act.bind(m.return_to_map))
	back.tooltip_text = "Close the menu (Esc)"
var frame: ScreenFrame = ScreenFrame.make(m.world_name(), back)
add_child(frame)
var where: String = m.location_name()
var loc: Label = MainMenuView.text(where if not where.is_empty() else "On the road", UiTheme.COL_MUTED)
loc.name = "HubLocation"
loc.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
frame.column.add_child(loc)
var tiles := GridContainer.new()
tiles.name = "HubParty"
tiles.columns = TILE_COLUMNS
tiles.add_theme_constant_override("h_separation", 8)
tiles.add_theme_constant_override("v_separation", 8)
tiles.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
frame.column.add_child(tiles)
for p: Dictionary in m.party():
	_tile(tiles, p)
var grid := GridContainer.new()
grid.name = "HubEntries"
grid.columns = ENTRY_COLUMNS
grid.add_theme_constant_override("h_separation", 12)
grid.add_theme_constant_override("v_separation", 10)
grid.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
frame.column.add_child(grid)
var first: Button = null
for id: String in HubModel.ENTRIES:
	var why: String = m.reason(id)
	var b: Button = _entry(grid, m.label(id), host.act.bind(m.open.bind(id)), why.is_empty(), why if not why.is_empty() else m.describe(id))
	b.name = "Hub_" + id
	if first == null and not b.disabled:
		first = b
var quit_why: String = m.quit_reason()
var quit: Button = _entry(grid, "Quit to main menu", host.act.bind(m.quit_to_menu), quit_why.is_empty(), quit_why if not quit_why.is_empty() else "Back to the title screen (asks first when there is unsaved progress).")
quit.name = "HubQuit"
detail = MainMenuView.text("", UiTheme.COL_MUTED)
detail.name = "HubDetail"
detail.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
frame.column.add_child(detail)
if first != null:
	detail.text = str(first.get_meta("detail", ""))
	set_meta(MainMenuView.FOCUS_META, first)
```

### `_tile(parent: GridContainer, p: Dictionary) -> void`
```logic
var active: bool = DictUtil.to_bool(p["active"])
var card: VBoxContainer = UiTheme.card(parent, UiTheme.COL_CARD_BG if active else Color(UiTheme.COL_PANEL_BG, 1.0))
var tile: Control = card.get_parent() as Control
tile.name = "HubTile_" + str(p["id"])
tile.custom_minimum_size = Vector2(TILE_W, 0)
var who: String = str(p["name"])
card.add_child(MainMenuView.text(who, UiTheme.COL_HEADER if active else UiTheme.COL_MUTED))
var cls: String = str(p["class"]) if str(p["class"]) != who else ""   # a member named after its class shows it once
card.add_child(MainMenuView.text("%sLv %d%s" % [cls + "  " if not cls.is_empty() else "", DictUtil.to_int(p["level"]), "" if active else "  (reserve)"], UiTheme.COL_TEXT if active else UiTheme.COL_MUTED))
var max_hp: int = DictUtil.to_int(p["max_hp"])
card.add_child(MainMenuView.text("HP %d / %d" % [DictUtil.to_int(p["hp"]), max_hp] if max_hp > 0 else "HP -", UiTheme.COL_TEXT if active else UiTheme.COL_MUTED))
```

### `_entry(parent: Control, text: String, callback: Callable, enabled: bool, hint: String) -> Button`
```logic
var b: Button = MainMenuView.button(text, callback, enabled)
b.size_flags_horizontal = Control.SIZE_FILL
b.tooltip_text = hint
b.set_meta("detail", hint)
b.focus_entered.connect(_show_detail.bind(hint))
b.mouse_entered.connect(_show_detail.bind(hint))
parent.add_child(b)
return b
```

### `_show_detail(hint: String) -> void`
```logic
if detail != null:
	detail.text = hint
```

