---
class: IntentText
extends: RefCounted
description: "Tactical RPG - Intent text (INTENT1-U) Words for an enemy intent record (CombatSession.enemy_intents / IntentRules): the hostile card's \"Next:\" line, the hover card's Intent and plan lines, the timeline banner and the forecast line. Pure text, display only. \"B1\" or \"B1-2\" for a (start, end) block pair."
---

# Methods

### `static block_text(blocks: Vector2i) -> String`
```logic
return "B%d" % blocks.x if blocks.x == blocks.y else "B%d-%d" % [blocks.x, blocks.y]
```

### `static core(rec: Dictionary) -> String`
```logic
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
```

### `static first_line(rec: Dictionary) -> String`
```logic
if str(rec.get("kind", "")) == IntentRules.ARRIVING:
	return "Arriving"
if str(rec.get("kind", "")) == IntentRules.IDLE:
	return "Holding"
return "%s (%s)" % [core(rec), block_text(rec.get("blocks", Vector2i.ZERO))]
```

### `static plan_line(rec: Dictionary) -> String`
```logic
return "%s %s" % [block_text(rec.get("blocks", Vector2i.ZERO)), core(rec)]
```

### `static read_by(rec: Dictionary) -> String`
```logic
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
```

