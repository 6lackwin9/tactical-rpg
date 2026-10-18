---
class: PlaybackBanner
extends: PanelContainer
description: "Tactical RPG - Playback Banner (PLAYTEST1 rule 6) Shown by the arena over a played-back recording: `PLAYBACK <id> battle k round r/n`, the stale badge, a Notes drop-down (jump = select_round(round) + select_block(block), block 0 = round start) and \"Take over this round\". In take-over mode it only names the branch."
---

# Properties
* `TOP_OFFSET`: float = 90.0 (const)
* `session`: CombatSession = null
* `rec_id`: String = ""
* `battle`: int = 0
* `recorded_rounds`: int = 0
* `notes`: Array[Dictionary] = []
* `branch_round`: int = -1           # > 0: the banner of a take-over (live) battle
* `title`: Label = null
* `stale_badge`: Label = null
* `notes_pick`: OptionButton = null
* `take_over_button`: Button = null
* `out`: Array[Dictionary] = []
* `note`: Dictionary = DictUtil.dict_of(raw)

# Signals
* `take_over_requested(round_num: int)`

# Methods

### `_init() -> void`
```logic
name = "PlaybackBanner"
mouse_filter = Control.MOUSE_FILTER_STOP
set_anchors_preset(Control.PRESET_CENTER_TOP)
grow_horizontal = Control.GROW_DIRECTION_BOTH
offset_top = TOP_OFFSET   # below the top docks (rosters, timeline), over the grid title
offset_bottom = TOP_OFFSET
add_theme_stylebox_override("panel", UiTheme.box(Color(0.10, 0.06, 0.16, 0.94), UiTheme.COL_WARN, 8, 10, 2))
var row := HBoxContainer.new()
row.add_theme_constant_override("separation", 10)
add_child(row)
title = UiTheme.label("PLAYBACK", UiTheme.FONT_MD, UiTheme.COL_WARN)
row.add_child(title)
stale_badge = UiTheme.label("STALE", UiTheme.FONT_SM, UiTheme.COL_BAD)
stale_badge.tooltip_text = "The game data changed since this recording; it still replays, but results may differ."
stale_badge.mouse_filter = Control.MOUSE_FILTER_PASS
stale_badge.visible = false
row.add_child(stale_badge)
notes_pick = OptionButton.new()
notes_pick.tooltip_text = "Jump to a note: its round and block"
notes_pick.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
notes_pick.item_selected.connect(_on_note_picked)
row.add_child(notes_pick)
take_over_button = UiTheme.button("⤷ Take over this round", "Replay to the start of the viewed round, then plan it yourself (a new recording)",
	func() -> void: take_over_requested.emit(session.viewed_round if session != null else 1), UiTheme.FONT_SM)
row.add_child(take_over_button)
```

### `setup(p_session: CombatSession, data: Dictionary, p_battle: int, stale: bool, p_branch_round: int = -1) -> void`
```logic
session = p_session
rec_id = str(data.get("id", ""))
battle = p_battle
branch_round = p_branch_round
recorded_rounds = PlaytestRecord.concluded_rounds(PlaytestRecord.battle_at(data, battle)).size()
stale_badge.visible = stale
notes = notes_of(data, battle)
notes_pick.clear()
notes_pick.add_item("Notes (%d)" % notes.size())
for note: Dictionary in notes:
	notes_pick.add_item("#%d r%d b%d  %s" % [DictUtil.to_int(note["n"]), DictUtil.to_int(note["round"]), DictUtil.to_int(note["block"]), str(note["text"]).left(40)])
notes_pick.visible = branch_round < 0 and not notes.is_empty()
take_over_button.visible = branch_round < 0
if not session.changed.is_connected(refresh):
	session.changed.connect(refresh)
refresh()
```

### `show_error(path: String, error: String) -> void`
```logic
session = null
title.text = "PLAYTEST NOT PLAYED BACK: %s  ·  %s  ·  live battle, not recorded" % [path.get_file(), error]
title.add_theme_color_override("font_color", UiTheme.COL_BAD)
stale_badge.visible = false
notes_pick.visible = false
take_over_button.visible = false
```

### `refresh() -> void`
```logic
if session == null:
	return
if branch_round > 0:
	title.text = "TAKE-OVER of %s battle %d from round %d  ·  live round %d" % [rec_id, battle, branch_round, session.current_round]
else:
	title.text = "PLAYBACK %s battle %d round %d/%d" % [rec_id, battle, session.viewed_round, recorded_rounds]
```

### `static jump(p_session: CombatSession, note: Dictionary) -> void`
```logic
p_session.select_round(DictUtil.to_int(note.get("round", 1), 1))
p_session.select_block(DictUtil.to_int(note.get("block", 0)))
```

### `_on_note_picked(index: int) -> void`
```logic
if index <= 0 or index > notes.size() or session == null:
	return
jump(session, notes[index - 1])
notes_pick.select(0)
```

