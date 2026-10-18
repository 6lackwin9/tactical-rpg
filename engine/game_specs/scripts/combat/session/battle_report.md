---
class: BattleReport
extends: RefCounted
description: "Tactical RPG - Battle report (BREPORT1-L 1.4, rules 4-7): the per-hero table and the best-moment line of the results popup, built once from a finished BattleMetrics. Pure and static: build() reads the collector's records and details and the session's party side (heroes in roster order, then guests); problems() validates the dict shape."
---

# Properties
* `VERSION`: int = 1 (const)
* `COLUMNS`: Array[String] = ["name", "dealt", "taken", "healed", "effects", "combos"]   # table order (spec section "What the report shows") (const)
* `ALWAYS`: Array[String] = ["name", "dealt", "taken"]                                     # rule 5: the others only when some row is non-zero (const)
* `NUMBERS`: Array[String] = ["dealt", "dealt_dot", "ff_dealt", "taken", "taken_dot", "healed", "regen", "effects", "combos", "kills"] (const)
* `STATES`: Array[String] = ["standing", "down", "left"] (const)
* `COMBO_LABELS`: Dictionary = {"exploit": "Exploit", "break": "Stance break", "flow": "Flow"} (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `top`: Dictionary = {"version": TYPE_INT, "outcome": TYPE_STRING, "rounds": TYPE_INT, "rows": TYPE_ARRAY, "columns": TYPE_ARRAY, "best": TYPE_STRING}
* `row_types`: Dictionary = {"ref": TYPE_STRING, "name": TYPE_STRING, "guest": TYPE_BOOL, "state": TYPE_STRING, "status_counts": TYPE_DICTIONARY, "combo_counts": TYPE_DICTIONARY}
* `i`: int = 0
* `row`: Dictionary = raw

# Methods

### `static build(metrics: BattleMetrics, session: CombatSession, outcome: String) -> Dictionary`
```logic
var rows: Array[Dictionary] = []
var details: Array[Dictionary] = []
for c: Combatant in session.allies:
	var rec: Dictionary = DictUtil.dict_of(metrics.units.get(c.name, {}))
	var det: Dictionary = DictUtil.dict_of(metrics.detail.get(c.name, {}))
	var guest: bool = session.encounter.is_guest(c)
	var state: String = "left" if c.is_withdrawn else ("standing" if c.is_alive() else "down")
	var status_counts: Dictionary = DictUtil.dict_of(det.get("status_counts", {})).duplicate(true)
	var combo_counts: Dictionary = DictUtil.dict_of(det.get("combo_counts", {})).duplicate(true)
	rows.append({"ref": session.encounter.hero_key(c), "name": c.name, "guest": guest, "state": state,
		"dealt": DictUtil.int_of(rec, "dmg_dealt") + DictUtil.int_of(rec, "dot_dealt"), "dealt_dot": DictUtil.int_of(rec, "dot_dealt"),
		"ff_dealt": DictUtil.int_of(rec, "ff_dealt"),
		"taken": DictUtil.int_of(rec, "dmg_taken") + DictUtil.int_of(rec, "dot_taken") + DictUtil.int_of(rec, "ff_taken"),
		"taken_dot": DictUtil.int_of(rec, "dot_taken"), "healed": DictUtil.int_of(rec, "heal_done"), "regen": DictUtil.int_of(det, "regen_done"),
		"effects": DictUtil.int_of(rec, "buffs_applied") + DictUtil.int_of(rec, "debuffs_applied"), "combos": DictUtil.int_of(rec, "combos"),
		"kills": DictUtil.int_of(rec, "kills"), "status_counts": status_counts, "combo_counts": combo_counts})
	details.append(det)
return {"version": VERSION, "outcome": outcome, "rounds": session.current_round, "rows": rows, "columns": columns_for(rows),
	"best": best_moment(rows, details)}
```

### `static best_moment(rows: Array[Dictionary], details: Array[Dictionary]) -> String`
```logic
var save: Dictionary = _pick(rows, details, "best_save", func(m: Dictionary) -> int: return -DictUtil.int_of(m, "hp"))
if not save.is_empty():
	var ally: String = str(save["ally"])
	return "%s healed %s at %d HP, and %s made it through." % [str(save["who"]), ally, DictUtil.int_of(save, "hp"), ally]
var finish: Dictionary = _pick(rows, details, "best_finish", func(_m: Dictionary) -> int: return 0)
if not finish.is_empty():
	var how: String = "broke the guard" if str(finish.get("combo", "")) == "break" else "exploited %s" % StatusEffects.display_name(str(finish.get("status", "")))
	return "%s finished %s with %s (%s)." % [str(finish["who"]), str(finish["target"]), str(finish["skill"]), how]
var hit: Dictionary = _pick(rows, details, "best_hit", func(m: Dictionary) -> int: return DictUtil.int_of(m, "amount"))
if not hit.is_empty() and DictUtil.int_of(hit, "amount") >= 1:
	return "%s's %s hit %s for %d." % [str(hit["who"]), str(hit["skill"]), str(hit["target"]), DictUtil.int_of(hit, "amount")]
return ""
```

### `static _pick(rows: Array[Dictionary], details: Array[Dictionary], key: String, score: Callable) -> Dictionary`
```logic
var best: Dictionary = {}
var best_score: int = 0
for i: int in range(mini(rows.size(), details.size())):
	var m: Dictionary = DictUtil.dict_of(details[i].get(key, {}))
	if m.is_empty():
		continue
	var s: int = DictUtil.to_int(score.call(m))
	if best.is_empty() or s > best_score or (s == best_score and DictUtil.int_of(m, "order") < DictUtil.int_of(best, "order")):
		best = m.duplicate()
		best["who"] = str(rows[i]["name"])
		best_score = s
return best
```

### `static dealt_tip(row: Dictionary) -> String`
```logic
var dot: int = DictUtil.int_of(row, "dealt_dot")
var tip: String = "%d from hits, %d from bleed, poison and burn." % [DictUtil.int_of(row, "dealt") - dot, dot]
if DictUtil.int_of(row, "ff_dealt") > 0:
	tip += " Hit allies for %d." % DictUtil.int_of(row, "ff_dealt")
return tip
```

### `static taken_tip(row: Dictionary) -> String`
```logic
var dot: int = DictUtil.int_of(row, "taken_dot")
return "%d from hits, %d from effects." % [DictUtil.int_of(row, "taken") - dot, dot]
```

### `static healed_tip(row: Dictionary) -> String`
```logic
var regen: int = DictUtil.int_of(row, "regen")
return "Includes %d from regeneration." % regen if regen > 0 else ""
```

### `static effects_tip(row: Dictionary) -> String`
```logic
var parts: PackedStringArray = PackedStringArray()
var counts: Dictionary = DictUtil.dict_of(row.get("status_counts", {}))
var buffs: Array[String] = StatusEffects.buffs()
for s: Variant in counts:
	var n: int = DictUtil.int_of(counts, s)
	if n > 0:
		parts.append("%s%s x%d" % [StatusEffects.display_name(str(s)), " on allies" if buffs.has(str(s)) else "", n])
return ", ".join(parts)
```

### `static combos_tip(row: Dictionary) -> String`
```logic
var parts: PackedStringArray = PackedStringArray()
var counts: Dictionary = DictUtil.dict_of(row.get("combo_counts", {}))
for kind: String in BattleMetrics.COMBO_KINDS:
	var n: int = DictUtil.int_of(counts, kind)
	if n > 0:
		parts.append("%s x%d" % [str(COMBO_LABELS[kind]), n])
return ", ".join(parts)
```

