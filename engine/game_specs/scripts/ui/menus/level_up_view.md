---
class: LevelUpView
extends: VBoxContainer
description: "Tactical RPG - Level-up view (UI2-L): + / - per stat on the draft, Auto, the derived block before → after, Confirm (permanent) and Cancel. Renders LevelUpModel."
---

# Methods

### `build(host: MainMenuView, model: RefCounted) -> void`
```logic
var m: LevelUpModel = model as LevelUpModel
var member: UnitState = PartyPreview.member(m.member_id)
add_theme_constant_override("separation", 10)
add_child(MainMenuView.title("Level up: %s" % (PartyPreview.name_of(member) if member != null else m.member_id)))
add_child(MainMenuView.text("Points left: %d   (confirmed points are permanent)" % m.points_left(), UiTheme.COL_MUTED, false))
var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 32)
add_child(row)
var grid := GridContainer.new()
grid.columns = 4
grid.add_theme_constant_override("h_separation", 12)
row.add_child(grid)
var spend: Dictionary = member.spend if member != null else {}
for s: String in StatRules.STATS:
	var drafted: int = DictUtil.to_int(m.draft.get(s, 0))
	grid.add_child(MainMenuView.text("%s  %d%s" % [s, DictUtil.to_int(spend.get(s, 0)), "  +%d" % drafted if drafted > 0 else ""], UiTheme.COL_GOOD if drafted > 0 else UiTheme.COL_TEXT, false))
	grid.add_child(MainMenuView.small(MainMenuView.button(" - ", _redraw.bind(host, m.remove.bind(s)), drafted > 0), 48))
	grid.add_child(MainMenuView.small(MainMenuView.button(" + ", _redraw.bind(host, m.add.bind(s)), m.points_left() > 0), 48))
	grid.add_child(Control.new())
row.add_child(PartyView.derived_grid(m.before(), m.after()))
var buttons := HBoxContainer.new()
add_child(buttons)
buttons.add_child(MainMenuView.button("Auto", _redraw.bind(host, func() -> Array[String]:
	m.auto()
	return [] as Array[String]), m.points_left() > 0))
buttons.add_child(MainMenuView.button("Confirm", _redraw.bind(host, m.ask_confirm.bind(host.router)), m.has_draft()))
buttons.add_child(MainMenuView.button("Cancel", _redraw.bind(host, func() -> Array[String]:
	m.cancel()
	return [] as Array[String]), m.has_draft()))
buttons.add_child(MainMenuView.button("Back", host.router.pop))
```

### `static _redraw(host: MainMenuView, action: Callable) -> void`
```logic
host.act(action)
host.render()
```

