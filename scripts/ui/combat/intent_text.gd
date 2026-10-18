class_name IntentText
extends RefCounted

# ==============================================================================
# Tactical RPG - Intent text (INTENT1-U)
# Words for an enemy intent record (CombatSession.enemy_intents / IntentRules): the hostile card's "Next:" line, the
# hover card's Intent and plan lines, the timeline banner and the forecast line. Pure text, display only.
# ==============================================================================

# "B1" or "B1-2" for a (start, end) block pair.
static func block_text(blocks: Vector2i) -> String:
	return "B%d" % blocks.x if blocks.x == blocks.y else "B%d-%d" % [blocks.x, blocks.y]

# What one action does, without its blocks: "Shield Bash on Bryn", "Move toward Sella", "Area Attack at (4, 3)", "Defend".
# `rec` holds kind, skill, target (Combatant or null) and cell (works for a record and for a plan line).
static func core(rec: Dictionary) -> String:
	var kind: String = str(rec.get("kind", ""))
	if kind == IntentRules.ARRIVING:
		return "Arriving"
	if kind == IntentRules.IDLE:
		return "Holding"
	var skill: String = str(rec.get("skill", ""))
	var target: Variant = rec.get("target", null)
	var cell: Vector2i = rec.get("cell", IntentRules.NO_CELL)
	if target is Combatant:
		var unit: Combatant = target
		return "%s %s %s" % [skill, "toward" if kind == "move" or kind == "flee" else "on", unit.name]
	if cell != IntentRules.NO_CELL:
		return "%s %s %s" % [skill, "at" if kind == "area" else "to", str(cell)]
	return skill

# "Shield Bash on Bryn (B1)"; "Arriving" has no blocks.
static func first_line(rec: Dictionary) -> String:
	if str(rec.get("kind", "")) == IntentRules.ARRIVING:
		return "Arriving"
	if str(rec.get("kind", "")) == IntentRules.IDLE:
		return "Holding"
	return "%s (%s)" % [core(rec), block_text(rec.get("blocks", Vector2i.ZERO))]

# "B3 Shield Bash on Bryn": one line of a revealed plan.
static func plan_line(rec: Dictionary) -> String:
	return "%s %s" % [block_text(rec.get("blocks", Vector2i.ZERO)), core(rec)]

# "Read by Bryn (Warden's Eye)": who reads the plan and through which passive.
static func read_by(rec: Dictionary) -> String:
	var names: Array[String] = []
	var source: String = ""
	for raw: Variant in DictUtil.array_of(rec.get("readers", [])):
		if raw is Combatant:
			var holder: Combatant = raw
			names.append(holder.name)
			if source.is_empty():
				source = holder.foresight_source
	var passive: String = str(ContentCatalog.get_entry("passives", source).get("name", source)) if not source.is_empty() else ""
	return "Read by %s%s" % [", ".join(names), (" (%s)" % passive) if not passive.is_empty() else ""]
