---
class: JournalModel
extends: RefCounted
description: "Tactical RPG - Journal model (UI3-L): the current world's known quests grouped by status, with the resolved (variant-applied) text, current objective, rewards and the loot still offered. Read only: every value comes from QuestRunner reads on refresh(); no mutator is ever called (rule 1). Quests never started stay hidden (rule 3). [status, player-facing label] in display order; other statuses are never shown."
---

# Properties
* `GROUPS`: Array = [["ready", "Ready to turn in"], ["active", "Active"], ["blocked", "On hold"], ["completed", "Completed"]] (const)
* `QUEST_LINKS`: Array[String] = ["follows", "leads_to", "related_quests"] (const)
* `selected`: String = ""
* `_groups`: Array[Dictionary] = []
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `out`: Array[String] = []
* `xp`: int = DictUtil.to_int(rewards.get("xp", 0))
* `items`: Dictionary = JsonUtil.ints(rewards.get("items", {}))
* `gold`: int = DictUtil.to_int(rewards.get("gold", 0))

# Methods

### `_init() -> void`
```logic
refresh()
```

### `refresh() -> void`
```logic
_groups = []
var gs: GameState = CurrentGame.state()
if gs == null:
	selected = ""
	return
for g: Array in GROUPS:
	var ids: Array[String] = QuestRunner.quests_with(gs, str(g[0]))
	if not ids.is_empty():
		var rows: Array[Dictionary] = []
		for qid: String in ids:
			rows.append({"qid": qid, "name": str(QuestRunner.resolved(gs, qid).get("name", qid))})
		_groups.append({"status": str(g[0]), "label": str(g[1]), "quests": ids, "rows": rows})
var all: Array[String] = shown()
if not all.has(selected):
	selected = all[0] if not all.is_empty() else ""
```

### `move(step: int) -> void`
```logic
var all: Array[String] = shown()
if all.is_empty():
	return
selected = all[clampi(all.find(selected) + step, 0, all.size() - 1)]
```

### `static character_name(world_id: String, ref: String) -> String`
```logic
if ref.is_empty():
	return ""
var hit: Dictionary = WorldCatalog.resolve(world_id, "characters", ref) if WorldCatalog.has_world(world_id) else {}
return str(hit["name"]) if hit.has("name") else "[%s]" % ref
```

### `static _catalog_name(kind: String, id: String) -> String`
```logic
var e: Dictionary = ContentCatalog.get_entry(kind, id)
return str(e["name"]) if e.has("name") else "[%s]" % id
```

### `entry(qid: String) -> Dictionary`
```logic
var gs: GameState = CurrentGame.state()
if gs == null or not shown().has(qid):
	return {}
var st: String = QuestRunner.status(gs, qid)
var q: Dictionary = QuestRunner.resolved(gs, qid)
var total: int = DictUtil.array_of(q.get("objectives", [])).size()
var done: int = total if st == "completed" else mini(total, DictUtil.to_int(DictUtil.dict_of(gs.quests.get(qid, {})).get("objective", 0)))
var out: Dictionary = {
	"qid": qid, "name": str(q.get("name", qid)), "description": str(q.get("description", "")), "status": st,
	"giver": "", "turn_in": "", "objective": "", "objectives_done": done, "objectives_total": total,
	"rewards": [] as Array[String], "offers": [] as Array[String], "picks_allowed": 0, "links": [] as Array[Dictionary],
}
if st == "blocked":
	return out
out["giver"] = character_name(gs.world_id, str(q.get("giver", "")))
out["turn_in"] = character_name(gs.world_id, str(q.get("turn_in", "")))
out["objective"] = str(QuestRunner.current_objective(gs, qid).get("text", ""))
out["rewards"] = reward_lines(DictUtil.dict_of(q.get("rewards", {})))
if st == "active" or st == "ready":
	var offers: Array[String] = []
	for lqid: String in QuestRunner.offers(gs, qid):
		var loot: Dictionary = WorldCatalog.resolve(gs.world_id, "loot", lqid)
		offers.append(str(loot["name"]) if loot.has("name") else "[%s]" % lqid)
	out["offers"] = offers
	out["picks_allowed"] = QuestRunner.picks_allowed(gs, qid)
var links: Array[Dictionary] = []
var all: Array[String] = shown()
var raw_links: Dictionary = DictUtil.dict_of(q.get("links", {}))
for kind: String in QUEST_LINKS:
	for ref: String in JsonUtil.strings(raw_links.get(kind, [])):
		var other: String = str(WorldCatalog.resolve(gs.world_id, "quests", ref).get("_qid", ""))
		if all.has(other):   # no spoilers: only quests the journal already shows
			links.append({"kind": kind, "qid": other, "name": str(QuestRunner.resolved(gs, other).get("name", other))})
out["links"] = links
return out
```

