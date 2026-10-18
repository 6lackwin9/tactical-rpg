---
class: DojoCarryBar
extends: UiTipPanel
description: "Tactical RPG - Party Carry Bar Compact party-wide load meter (stash + equipped + pouches vs shared capacity) with a BBCode hover breakdown. Unit burden is shown separately per unit."
---

# Properties
* `TIER_COLORS`: Dictionary = { (const)
* `_title_lbl`: Label
* `_status_lbl`: Label
* `_bar`: ProgressBar
* `_fill_style`: StyleBoxFlat

# Methods

### `_init() -> void`
```logic
mouse_filter = Control.MOUSE_FILTER_STOP
add_theme_stylebox_override("panel", UiTheme.box(Color(0.10, 0.14, 0.19, 0.95), UiTheme.COL_BORDER, 4, 8, 5))

var vbox := VBoxContainer.new()
vbox.mouse_filter = Control.MOUSE_FILTER_PASS
vbox.add_theme_constant_override("separation", 3)
add_child(vbox)

var top := HBoxContainer.new()
top.mouse_filter = Control.MOUSE_FILTER_PASS
vbox.add_child(top)
_title_lbl = UiTheme.label("🎒 PARTY CARRY", UiTheme.FONT_SM, UiTheme.COL_HEADER)
_title_lbl.mouse_filter = Control.MOUSE_FILTER_PASS
top.add_child(_title_lbl)
top.add_spacer(false)
_status_lbl = UiTheme.label("-", UiTheme.FONT_SM, UiTheme.COL_GOOD)
_status_lbl.mouse_filter = Control.MOUSE_FILTER_PASS
top.add_child(_status_lbl)

_bar = ProgressBar.new()
_bar.show_percentage = false
_bar.custom_minimum_size = Vector2(0, 7)
_bar.mouse_filter = Control.MOUSE_FILTER_PASS
_bar.add_theme_stylebox_override("background", UiTheme.box(Color(0.14, 0.12, 0.10), Color(0, 0, 0, 0), 3, 0, 0, 0))
_fill_style = UiTheme.box(Color(0.3, 0.8, 0.45), Color(0, 0, 0, 0), 3, 0, 0, 0)
_bar.add_theme_stylebox_override("fill", _fill_style)
vbox.add_child(_bar)
```

### `show_report(report: Dictionary) -> void`
```logic
if report.is_empty():
	return
var capacity: int = UiTheme.dict_int(report, "capacity", 1)
var load_weight: float = UiTheme.dict_float(report, "load")
var tier: String = str(report.get("tier", "LIGHT"))
var penalty: int = UiTheme.dict_int(report, "speed_penalty")
var tier_color: Color = TIER_COLORS.get(tier, UiTheme.COL_GOOD)

_bar.max_value = float(maxi(1, capacity))
_bar.value = minf(load_weight, float(capacity))
_fill_style.bg_color = tier_color
var status: String = "%.1f / %d  %s" % [load_weight, capacity, tier.capitalize()]
if penalty > 0:
	status += "  (-%d speed)" % penalty
_status_lbl.text = status
_status_lbl.add_theme_color_override("font_color", tier_color)
tooltip_text = _build_tooltip(report, tier, penalty)
```

### `_build_tooltip(report: Dictionary, tier: String, penalty: int) -> String`
```logic
var lines: Array[String] = []
lines.append("[color=#a5d8ff][b]PARTY CARRY[/b][/color]  [color=#94a3b8]shared by the whole party[/color]")
lines.append("[color=#94a3b8]Capacity:[/color] %d base + Σ (STR × 3 + CONS) of every member" % PartyCarry.BASE_CAPACITY)
var members_raw: Variant = report.get("members", [])
if members_raw is Array:
	var parts: Array[String] = []
	for m: Variant in members_raw:
		if m is Dictionary:
			var member: Dictionary = m
			parts.append("%s +%d" % [str(member.get("name", "?")), UiTheme.dict_int(member, "contribution")])
	if not parts.is_empty():
		lines.append("   " + "  •  ".join(parts))
lines.append("   = [b]%d[/b]" % UiTheme.dict_int(report, "capacity"))
lines.append("[color=#94a3b8]Load:[/color] stash %.1f + equipped %.1f + pouches %.1f = [b]%.1f[/b]" % [
	UiTheme.dict_float(report, "stash_weight"), UiTheme.dict_float(report, "equipped_weight"),
	UiTheme.dict_float(report, "pouch_weight"), UiTheme.dict_float(report, "load")
])
match tier:
	"OVERLOADED":
		lines.append("[color=#f87171][b]Overloaded![/b] Every unit loses %d initiative speed (1 per %d weight over).[/color]" % [penalty, int(PartyCarry.OVERLOAD_STEP)])
	"HEAVY":
		lines.append("[color=#fbbf24]Heavy: above %d%% of capacity. Going over it slows the whole party.[/color]" % int(PartyCarry.HEAVY_RATIO * 100.0))
	_:
		lines.append("[color=#4ade80]Light: no penalty.[/color]")
lines.append("[color=#94a3b8]Burden is per unit: it only slows the unit wearing the gear.[/color]")
return "\n".join(lines)
```

