---
class: UiOverlayLayer
extends: Control
description: "Tactical RPG - UI Overlay Layer Hosts overlay windows opened by panels (UiPanel.overlay_requested) so a panel can get more visual space WITHOUT growing or reflowing itself: pickers, filter editors, allocation dialogs... Overlays are draggable UiWindows centered over the host, share the host's session, and close via ✕ or overlay_close_requested. The layer itself never blocks input outside its windows."
---

# Properties
* `session`: RefCounted = null  # any session exposing `signal changed()`
* `_windows`: Dictionary = {} # String (panel_id) -> UiWindow

# Methods

### `_init() -> void`
```logic
set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
mouse_filter = Control.MOUSE_FILTER_IGNORE
```

### `set_session(p_session: RefCounted) -> void`
```logic
session = p_session
```

### `host_panel(panel: UiPanel) -> void`
```logic
if not panel.overlay_requested.is_connected(open_overlay):
	panel.overlay_requested.connect(open_overlay)
```

### `open_overlay(content: UiPanel) -> void`
```logic
# One overlay per panel id: re-requesting replaces the old content.
close_overlay(content.panel_id)
if content.ui_session == null and session != null:
	content.bind(session)

var win := UiWindow.new()
win.show_collapse_button = false
win.setup(content)
win.close_requested.connect(func(w: UiWindow) -> void: close_overlay(w.window_id))
content.overlay_close_requested.connect(func() -> void: close_overlay(content.panel_id))
content.overlay_requested.connect(open_overlay) # overlays may open further overlays
add_child(win)
_windows[content.panel_id] = win

win.size = win.custom_minimum_size.max(content.default_min_size)
var host_size: Vector2 = size if size.x > 0.0 else Vector2(1280, 720)
# Cascade so stacked overlays never sit exactly on top of each other.
var cascade: Vector2 = Vector2(32, 32) * float((_windows.size() - 1) % 4)
win.position = (((host_size - win.size) / 2.0) + cascade).max(Vector2.ZERO)
win.move_to_front()
```

### `close_overlay(panel_id: String) -> void`
```logic
var win_raw: Variant = _windows.get(panel_id, null)
if win_raw is UiWindow:
	var win: UiWindow = win_raw
	_windows.erase(panel_id)
	if is_instance_valid(win):
		win.queue_free()
```

### `has_overlay(panel_id: String) -> bool`
```logic
return _windows.has(panel_id)
```

### `overlay_count() -> int`
```logic
return _windows.size()
```

### `close_all() -> void`
```logic
for panel_id: String in _windows.keys():
	close_overlay(panel_id)
```

