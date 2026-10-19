class_name FxPresets
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat FX presets (S14-L2)
# One code table of tween recipes and timings (presentation timing is not content)
# and the pure mapping from an ActionLogEntry to the presets CombatPresenter plays.
# Durations are seconds at speed 1.0 (FxSettings.scaled divides them by the speed).
# ==============================================================================

const MAX_LIVE: int = 24        # live tweens before a new play kills the backlog (fast replay)
const MELEE_BUMP: float = 0.3   # share of the way toward the target a melee bump travels
const SHAKE_OFFSETS: Array[Vector2] = [Vector2(4, 0), Vector2(-4, 0), Vector2(2, 0), Vector2.ZERO]
const STATUS_ICON_PX: int = 24
const STATUS_RISE_PX: float = 16.0
const FLOAT_RISE: Vector2 = Vector2(8, -28)
const MISS_SIDESTEP: Vector2 = Vector2(0, -6)   # miss: the target dodges this far, then returns
const AMBUSH_TEXT_PX: Vector2 = Vector2(281, 180)   # board-centred banner (no token; 12x9 board, 648 px wide)

# id -> {"durations": [...], "tweens": tweens one play creates, "color": modulate (pulses/flashes)}
const PRESETS: Dictionary = {
	"move": {"durations": [0.25], "tweens": 1},
	"attack_melee": {"durations": [0.08, 0.12], "tweens": 1},
	"attack_ranged": {"durations": [0.25], "tweens": 1},
	"hit": {"durations": [0.03, 0.03, 0.03, 0.04, 0.25], "tweens": 2, "color": Color(2.0, 2.0, 2.0)},
	"mitigated": {"durations": [0.03, 0.03, 0.03, 0.04, 0.25], "tweens": 2, "color": Color("#94a3b8")},
	"miss": {"durations": [0.08, 0.14], "tweens": 1},   # target sidesteps and returns; no shake, no flash
	"heal": {"durations": [0.3], "tweens": 1, "color": Color("#22c55e")},
	"buff": {"durations": [0.3], "tweens": 1, "color": Color("#eab308")},
	"defend": {"durations": [0.3], "tweens": 1, "color": Color("#94a3b8")},
	"status": {"durations": [0.55], "tweens": 1},
	"float_text": {"durations": [0.55, 0.25, 0.55], "tweens": 1},
	"defeat": {"durations": [0.4], "tweens": 1},
}
const FLOAT_DELAY: float = 0.2   # float_text: the fade starts this long after the rise

static func durations(preset_id: String) -> Array[float]:
	var out: Array[float] = []
	var row: Dictionary = _row(preset_id)
	var raw: Variant = row.get("durations", [])
	if raw is Array:
		var list: Array = raw
		for d: Variant in list:
			if d is float:
				out.append(d)
	return out

static func duration(preset_id: String, step: int = 0) -> float:
	var list: Array[float] = durations(preset_id)
	return list[step] if step >= 0 and step < list.size() else 0.0

static func tween_count(preset_id: String) -> int:
	var raw: Variant = _row(preset_id).get("tweens", 0)
	return raw if raw is int else 0

static func color(preset_id: String) -> Color:
	var raw: Variant = _row(preset_id).get("color", Color.WHITE)
	return raw if raw is Color else Color.WHITE

static func _row(preset_id: String) -> Dictionary:
	var raw: Variant = PRESETS.get(preset_id, {})
	return raw if raw is Dictionary else {}

# Presets for one log entry, in play order. `distance` is the grid distance actor -> target
# when the caller knows it (ATTACK entries carry no coordinates); -1 reads it from the
# entry's coords, and unknown coords count as melee (the pre-S14 lunge).
static func for_entry(entry: ActionLogEntry, distance: int = -1) -> Array[String]:
	var out: Array[String] = []
	if entry == null:
		return out
	match entry.category:
		"MOVE":
			if entry.coord_to != Vector2i(-1, -1):
				out.append("move")
			if "WITHDRAWN" in entry.outcome_tag:
				out.append("float_text")
		"ATTACK":
			out.append("attack_melee" if _distance(entry, distance) <= 1 else "attack_ranged")
			if entry.damage == 0 and not status_id_of(entry).is_empty():
				out.append("status")
			elif entry.outcome_tag == "MISS":
				out.append_array(["miss", "float_text"])
			else:
				out.append("mitigated" if "MITIGAT" in entry.outcome_tag else "hit")
				if entry.damage > 0:
					out.append("float_text")
		"DEFEND":
			out.append_array(["defend", "float_text"])
		"BUFF":
			if entry.skill_subtype == "Support" and entry.value_num > 0:
				out.append_array(["heal", "float_text"])
			else:
				out.append("buff")
				if not status_id_of(entry).is_empty():
					out.append("status")
				out.append("float_text")
		"EVENT":
			if "DEFEATED" in entry.detail_text or entry.outcome_tag == "DEFEATED":
				out.append("defeat")
			elif "AMBUSH" in entry.outcome_tag:
				out.append("float_text")
	return out

static func _distance(entry: ActionLogEntry, distance: int) -> int:
	if distance >= 0:
		return distance
	if entry.coord_from == Vector2i(-1, -1) or entry.coord_to == Vector2i(-1, -1):
		return 1
	var d: Vector2i = entry.coord_to - entry.coord_from
	return absi(d.x) + absi(d.y)

# The StatusRegistry id an entry names (its outcome tag or status text equals the status's
# display name, as SupportCommand writes it), or "" when it names none. Free text such as
# "Target HP: 30/50" or "Empowered" names no status, so no icon pops for it.
static func status_id_of(entry: ActionLogEntry) -> String:
	var tag: String = entry.outcome_tag.strip_edges().to_upper()
	var text: String = entry.status_text.strip_edges().to_upper()
	for id: String in StatusRegistry.ids():
		var shown: String = StatusEffects.display_name(id).to_upper()
		if tag == shown or text == shown:
			return id
	return ""

# Label text and colour of an entry's float_text preset (unchanged from the pre-S14 labels).
static func float_text(entry: ActionLogEntry) -> String:
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

static func float_color(entry: ActionLogEntry) -> Color:
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

# attack_ranged pulse: the actor's team accent.
static func team_accent(is_ally: bool) -> Color:
	return UnitArt.team_color(UnitArt.Team.FRIENDLY if is_ally else UnitArt.Team.HOSTILE)
