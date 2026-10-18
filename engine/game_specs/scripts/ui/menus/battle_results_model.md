---
class: BattleResultsModel
extends: RefCounted
description: "Tactical RPG - Battle results model (UI2-L rule 8): XP and levels per member from the award dict (LevelUp.award_encounter / BattleBridge.apply()[\"xp\"]: member id -> {xp, old_level, new_level, unspent}); only listed (active) members show. The level-up prompt is skippable; continue_on() autosaves and returns to the caller. `outcome` (won / lost / fled, BATTLE1-L) sets the headline; only a won battle awards XP. BATTLE-FLOW: the screen is a popup over the combat arena; each row carries the XP bar values (fill = progress in the current level, gain = the part of it this battle earned), `rewards` lists every non-XP reward the battle gave, and `minimised` / `replay_handler` drive the minimise pill and Replay (the host wires the handler to the arena). BREPORT1-L: `report` is the battle's BattleReport dict ({} = none; a dict with BattleReport.problems is treated as none); report_columns / report_rows / best_moment give the view its table, cells (\"-\" for 0) and hover tips."
---

# Properties
* `router`: MenuRouter = null
* `award`: Dictionary = {}
* `advanced`: Array[String] = []   # quest qids advanced by this battle
* `outcome`: String = "won"
* `rewards`: Dictionary = {}       # non-XP rewards: gold, supplies, items {ref: count}, loot (qid or Array), reputation {faction: delta}
* `minimised`: bool = false
* `replay_handler`: Callable = Callable()   # set while the arena is still mounted
* `report`: Dictionary = {}        # BREPORT1: the battle report ({} = no report section)
* `HEADLINES`: Dictionary = {"won": "Victory", "lost": "Defeat", "fled": "Fled"} (const)
* `REWARD_KEYS`: Array[String] = ["gold", "supplies", "items", "loot", "reputation"] (const)
* `REPORT_LABELS`: Dictionary = {"name": "Hero", "dealt": "Damage", "taken": "Taken", "healed": "Healed", "effects": "Effects", "combos": "Combos"} (const)
* `NOTES`: Dictionary = { (const)
* `out`: Array[Dictionary] = []
* `gs`: GameState = CurrentGame.state()
* `a`: Dictionary = DictUtil.dict_of(award[m.id])
* `xp`: int = DictUtil.to_int(a.get("xp", 0))
* `level`: int = m.level()
* `base`: int = XpCurve.xp_for_level(level)
* `span`: int = XpCurve.xp_for_level(level + 1) - base
* `maxed`: bool = level >= StatRules.MAX_LEVEL or span <= 0
* `fill`: float = 1.0 if maxed else clampf(float(m.total_xp - base) / float(span), 0.0, 1.0)
* `gain`: float = 0.0 if maxed else clampf(float(m.total_xp - maxi(m.total_xp - xp, base)) / float(span), 0.0, fill)
* `out`: Array[String] = []
* `items`: Dictionary = DictUtil.dict_of(rewards.get("items", {}))
* `loot`: Variant = rewards.get("loot", "")
* `loot_refs`: Array[String] = JsonUtil.strings(loot)
* `rep`: Dictionary = DictUtil.dict_of(rewards.get("reputation", {}))
* `delta`: int = DictUtil.to_int(rep[faction])
* `quests`: String = rewards_text()
* `out`: Array[Dictionary] = []
* `id`: String = str(raw)
* `out`: Array[Dictionary] = []
* `row`: Dictionary = DictUtil.dict_of(raw)
* `state`: String = str(row.get("state", "standing"))
* `guest`: bool = DictUtil.to_bool(row.get("guest", false))
* `name_cell`: String = str(row.get("name", "")) + (" (guest)" if guest else "") + ("" if state == "standing" else " (%s)" % state)
* `cells`: Dictionary = {"name": name_cell}
* `n`: int = DictUtil.int_of(row, id)
* `tips`: Dictionary = {"name": "", "dealt": BattleReport.dealt_tip(row), "taken": BattleReport.taken_tip(row), "healed": BattleReport.healed_tip(row),
* `errors`: Array[String] = []
* `_started`: Variant = replay_handler.call()
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `_done`: bool = false
* `errors`: Array[String] = SaveGame.autosave(CurrentGame.state())

# Methods

### `_init(p_router: MenuRouter, p_award: Dictionary, p_advanced: Array[String] = [], p_outcome: String = "won", p_rewards: Dictionary = {}, p_report: Dictionary = {}) -> void`
```logic
router = p_router
award = p_award
advanced = p_advanced
outcome = p_outcome
rewards = p_rewards
report = p_report if BattleReport.problems(p_report).is_empty() else {}
```

### `static rewards_of(res: Dictionary) -> Dictionary`
```logic
var out: Dictionary = DictUtil.dict_of(res.get("rewards", {})).duplicate(true)
for key: String in REWARD_KEYS:
	if res.has(key) and not out.has(key):
		out[key] = res[key]
return out
```

### `headline() -> String`
```logic
return str(HEADLINES.get(outcome, outcome))
```

### `outcome_note() -> String`
```logic
return str(NOTES.get(outcome, ""))
```

### `prompt_level_up() -> bool`
```logic
for r: Dictionary in rows():
	if DictUtil.to_int(r["new_level"]) > DictUtil.to_int(r["old_level"]) and DictUtil.to_int(r["unspent"]) > 0:
		return true
return false
```

### `rewards_text() -> String`
```logic
var names: PackedStringArray = PackedStringArray()
var gs: GameState = CurrentGame.state()
for qid: String in advanced:
	names.append(str(QuestRunner.resolved(gs, qid).get("name", qid)) if gs != null else qid)
return "Quests advanced: " + ", ".join(names) if not names.is_empty() else ""
```

### `static _pretty(ref: String) -> String`
```logic
var tail: String = ref.get_slice(":", ref.get_slice_count(":") - 1)
return tail.replace("_", " ").capitalize()
```

### `has_report() -> bool`
```logic
return not report.is_empty() and not DictUtil.array_of(report.get("rows", [])).is_empty()
```

### `best_moment() -> String`
```logic
var best: String = str(report.get("best", "")) if has_report() else ""
return "" if best.is_empty() else "Best moment: " + best
```

### `set_minimised(value: bool) -> void`
```logic
if minimised == value:
	return
minimised = value
router.changed.emit()
```

### `toggle_minimised() -> void`
```logic
set_minimised(not minimised)
```

### `can_replay() -> bool`
```logic
return replay_handler.is_valid()
```

### `has_free_slot(id: String) -> bool`
```logic
if PartyPreview.member(id) == null:
	return false
var lm := LoadoutModel.new(id)
return lm.class_slots.size() < StatRules.MAX_CLASS_SLOTS or lm.extra_slots.size() < StatRules.MAX_EXTRA_SLOTS
```

