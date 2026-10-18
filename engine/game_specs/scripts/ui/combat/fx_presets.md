---
class: FxPresets
extends: RefCounted
description: "Tactical RPG - Combat FX presets (S14-L2) One code table of tween recipes and timings (presentation timing is not content) and the pure mapping from an ActionLogEntry to the presets CombatPresenter plays. Durations are seconds at speed 1.0 (FxSettings.scaled divides them by the speed)."
---

# Properties
* `MAX_LIVE`: int = 24        # live tweens before a new play kills the backlog (fast replay) (const)
* `MELEE_BUMP`: float = 0.3   # share of the way toward the target a melee bump travels (const)
* `SHAKE_OFFSETS`: Array[Vector2] = [Vector2(4, 0), Vector2(-4, 0), Vector2(2, 0), Vector2.ZERO] (const)
* `STATUS_ICON_PX`: int = 24 (const)
* `STATUS_RISE_PX`: float = 16.0 (const)
* `FLOAT_RISE`: Vector2 = Vector2(8, -28) (const)
* `MISS_SIDESTEP`: Vector2 = Vector2(0, -6)   # miss: the target dodges this far, then returns (const)
* `AMBUSH_TEXT_PX`: Vector2 = Vector2(281, 180)   # board-centred banner (no token; 12x9 board, 648 px wide) (const)
* `PRESETS`: Dictionary = { (const)
* `FLOAT_DELAY`: float = 0.2   # float_text: the fade starts this long after the rise (const)
* `out`: Array[float] = []
* `row`: Dictionary = _row(preset_id)
* `raw`: Variant = row.get("durations", [])
* `list`: Array = raw
* `out`: Array[String] = []

# Methods

### `static duration(preset_id: String, step: int = 0) -> float`
```logic
var list: Array[float] = durations(preset_id)
return list[step] if step >= 0 and step < list.size() else 0.0
```

### `static tween_count(preset_id: String) -> int`
```logic
var raw: Variant = _row(preset_id).get("tweens", 0)
return raw if raw is int else 0
```

### `static color(preset_id: String) -> Color`
```logic
var raw: Variant = _row(preset_id).get("color", Color.WHITE)
return raw if raw is Color else Color.WHITE
```

### `static _row(preset_id: String) -> Dictionary`
```logic
var raw: Variant = PRESETS.get(preset_id, {})
return raw if raw is Dictionary else {}
```

### `static _distance(entry: ActionLogEntry, distance: int) -> int`
```logic
if distance >= 0:
	return distance
if entry.coord_from == Vector2i(-1, -1) or entry.coord_to == Vector2i(-1, -1):
	return 1
var d: Vector2i = entry.coord_to - entry.coord_from
return absi(d.x) + absi(d.y)
```

### `static status_id_of(entry: ActionLogEntry) -> String`
```logic
var tag: String = entry.outcome_tag.strip_edges().to_upper()
var text: String = entry.status_text.strip_edges().to_upper()
for id: String in StatusRegistry.ids():
	var shown: String = StatusEffects.display_name(id).to_upper()
	if tag == shown or text == shown:
		return id
return ""
```

### `static float_text(entry: ActionLogEntry) -> String`
```logic
match entry.category:
	"ATTACK":
		if entry.outcome_tag == "MISS":
			return "Miss"
		var txt: String = "-%d%s" % [entry.damage, " [MIT]" if "MITIGAT" in entry.outcome_tag else ""]
		return txt + ("  💀 DEFEATED!" if entry.outcome_tag == "DEFEATED" else "")
	"DEFEND":
		return "🛡 DEFEND [%d%%]" % entry.value_num
	"BUFF":
		if entry.skill_subtype == "Support" and entry.value_num > 0:
			return "+%d HP" % entry.value_num
		return "✦ %s" % entry.outcome_tag
	"MOVE":
		return "⚑ WITHDRAWN!"
	"EVENT":
		return "⚠️ AMBUSH!"
return ""
```

### `static float_color(entry: ActionLogEntry) -> Color`
```logic
match entry.category:
	"ATTACK":
		if entry.outcome_tag == "MISS":
			return Color(0.8, 0.8, 0.8)
		if entry.outcome_tag == "DEFEATED":
			return Color(1.0, 0.25, 0.25)
		return Color(0.4, 0.85, 1.0) if "MITIGAT" in entry.outcome_tag else Color(1.0, 0.35, 0.35)
	"DEFEND":
		return Color(0.4, 0.8, 1.0)
	"BUFF":
		if entry.skill_subtype == "Support" and entry.value_num > 0:
			return Color(0.35, 0.95, 0.55)
		return Color(1.0, 0.85, 0.25)
	"MOVE":
		return Color(0.35, 0.95, 0.55)
	"EVENT":
		return Color(1.0, 0.4, 0.4)
return Color.WHITE
```

### `static team_accent(is_ally: bool) -> Color`
```logic
return UnitArt.team_color(UnitArt.Team.FRIENDLY if is_ally else UnitArt.Team.HOSTILE)
```

