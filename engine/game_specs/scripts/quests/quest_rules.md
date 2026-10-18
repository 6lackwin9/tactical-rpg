---
class: QuestRules
extends: RefCounted
description: "Tactical RPG - Quest Rules (S8, spec M4-L) Constants and pure validation helpers for quest templates (QuestCatalog), the quest fields of world quests, world loot entries (WorldCatalog kind `loot`), flag conditions and the stored quest blob. No state, no RNG. World quests bind a lore-free template to world targets; `variants` override a whitelisted set of fields when flags hold; `loot_pool` offers world loot entries."
---

# Properties
* `OBJECTIVE_KINDS`: Array[String] = ["reach_location", "win_encounter", "flag"] (const)
* `MAX_OBJECTIVES`: int = 8 (const)
* `REWARD_KEYS`: Array[String] = ["xp", "items", "passives", "skills", "gold"] (const)
* `XP_MAX`: int = 2000 (const)
* `GOLD_MAX`: int = 5000   # DLG1-L (S11): party gold a template grants (const)
* `ITEM_COUNT_MAX`: int = 99 (const)
* `MAX_ITEM_KEYS`: int = 8 (const)
* `SOFT_MAX_PASSIVES`: int = 1 (const)
* `SOFT_MAX_SKILLS`: int = 1 (const)
* `SOFT_MAX_XP`: int = 300 (const)
* `SOFT_MAX_ITEMS`: int = 6 (const)
* `SOFT_MAX_PICKS`: int = 1 (const)
* `SOFT_MAX_GOLD`: int = 200 (const)
* `MAX_VARIANTS`: int = 8 (const)
* `VARIANT_KEYS`: Array[String] = ["id", "when_flags", "unless_flags", "set"] (const)
* `VARIANT_FIELDS`: Array[String] = ["giver", "turn_in", "description", "reward", "rewards_from", "steps", "links", "available", "loot_pool", "loot_picks"] (const)
* `LINK_KINDS`: Dictionary = { (const)
* `MAX_POOL`: int = 8 (const)
* `MAX_PICKS`: int = 3 (const)
* `LOOT_KINDS`: Array[String] = ["item", "passive", "skill"] (const)
* `RARITIES`: Array[String] = ["common", "rare", "unique"] (const)
* `OPS`: Array[String] = ["==", "!=", ">=", "<=", ">", "<"] (const)
* `ORDER_OPS`: Array[String] = [">=", "<=", ">", "<"] (const)
* `CONDITION_KEYS`: Array[String] = ["key", "value", "op"] (const)
* `REP_DELTA_MAX`: int = 50 (const)
* `MAX_REP_KEYS`: int = 4 (const)
* `STORED_STATES`: Array[String] = ["active", "ready", "completed"] (const)
* `BLOB_KEYS`: Array[String] = ["state", "objective"] (const)
* `BINDINGS`: Dictionary = { (const)
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `rewards`: Dictionary = JsonUtil.normalize(raw)
* `xp`: Variant = rewards["xp"]
* `gold`: Variant = rewards["gold"]
* `items`: Dictionary = rewards["items"]
* `why`: String = reward_ref_error("item", str(id))
* `count`: Variant = items[id]
* `key`: String = str(pair[0])
* `seen`: Dictionary = {}
* `id`: String = str(v)
* `why`: String = reward_ref_error(str(pair[1]), id)
* `out`: Array[String] = []
* `rewards`: Dictionary = DictUtil.dict_of(JsonUtil.normalize(raw))
* `passives`: int = DictUtil.array_of(rewards.get("passives", [])).size()
* `skills`: int = DictUtil.array_of(rewards.get("skills", [])).size()
* `errors`: Array[String] = []
* `id`: String = str(entry.get("id", ""))
* `objectives`: Variant = entry.get("objectives", null)
* `list`: Array = objectives
* `obj`: Dictionary = list[i]
* `kind`: String = str(obj.get("kind", ""))
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `list`: Array = JsonUtil.normalize(raw)
* `cond`: Dictionary = list[i]
* `key`: Variant = cond.get("key", "")
* `errors`: Array[String] = []
* `op_raw`: Variant = cond["op"]
* `op`: String = str(op_raw)
* `value`: Variant = JsonUtil.normalize(cond.get("value", true))
* `why`: String = RepRules.value_error(str(cond.get("key", "")), value)
* `SET_FLAG_LISTS`: Array[String] = ["complete_flags", "sets_flags"] (const)
* `errors`: Array[String] = condition_errors(raw, label)
* `list`: Array = JsonUtil.normalize(raw)
* `cond`: Dictionary = DictUtil.dict_of(list[i])
* `why`: String = RepRules.value_error(str(cond.get("key", "")), cond.get("value", true))
* `errors`: Array[String] = []
* `d`: Dictionary = value
* `o`: Variant = d["objective"]
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `template_id`: String = str(e.get("template", ""))
* `template`: Dictionary = {}
* `has_template`: bool = not template_id.is_empty()
* `steps`: Array = DictUtil.array_of(JsonUtil.normalize(e.get("steps", [])))
* `kinds`: Array[String] = template_objective_kinds(template)
* `pool`: Variant = e.get("loot_pool", [])
* `pool_size`: int = DictUtil.array_of(pool).size()
* `variants`: Variant = e.get("variants", [])
* `errors`: Array[String] = []
* `rep`: Dictionary = JsonUtil.normalize(raw)
* `r`: String = str(ref)
* `d`: Variant = rep[ref]
* `errors`: Array[String] = []
* `label`: String = "Quest step %d (%s)" % [index + 1, kind]
* `req`: String = required_binding(kind)
* `v`: Variant = step[key]
* `s`: String = str(v)
* `errors`: Array[String] = []
* `own_qid`: String = WorldCatalog.qid(world_id, str(e.get("id", "")))
* `seen`: Dictionary = {}
* `queue`: Array[String] = JsonUtil.strings(e.get("requires", []))
* `guard`: int = 0
* `ref`: String = queue.pop_front()
* `hit`: Dictionary = WorldCatalog.resolve(world_id, "quests", ref)
* `hit_qid`: String = str(hit.get("_qid", ""))
* `errors`: Array[String] = []
* `links`: Dictionary = raw
* `own_qid`: String = WorldCatalog.qid(world_id, str(e.get("id", "")))
* `k`: String = str(key)
* `seen`: Dictionary = {}
* `ref`: String = str(v)
* `ref_kind`: String = str(LINK_KINDS[k])
* `errors`: Array[String] = []
* `list`: Array = raw
* `seen`: Dictionary = {}
* `ref`: String = str(v)
* `hit`: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
* `key`: String = str(hit.get("_qid", ref))
* `errors`: Array[String] = []
* `v`: Variant = JsonUtil.normalize(raw)
* `errors`: Array[String] = []
* `list`: Array = DictUtil.array_of(JsonUtil.normalize(e.get("variants", [])))
* `ids`: Dictionary = {}
* `v`: Dictionary = list[i]
* `vid`: String = str(v.get("id", ""))
* `label`: String = "Variant '%s'" % vid if not vid.is_empty() else "Variant %d" % (i + 1)
* `set_raw`: Variant = v.get("set", null)
* `errors`: Array[String] = []
* `k`: String = str(key)
* `value`: Variant = set_d[key]
* `errors`: Array[String] = []
* `steps`: Dictionary = raw
* `s`: String = str(key)
* `idx`: int = s.to_int()
* `partial`: Dictionary = steps[key]
* `allowed`: Array[String] = ["text"]
* `errors`: Array[String] = []
* `kind`: String = str(e.get("reward_kind", "item"))
* `ref`: String = str(e.get("ref", ""))
* `why`: String = reward_ref_error(kind, ref)
* `count`: Variant = JsonUtil.normalize(e.get("count", 1))
* `out`: Array[String] = []
* `links`: Dictionary = DictUtil.dict_of(q.get("links", {}))
* `other`: Dictionary = WorldCatalog.resolve(world_id, "quests", ref)
* `back`: bool = false
* `out`: Array[String] = []
* `template_id`: String = str(q.get("template", ""))
* `reward_sets`: Array[Dictionary] = [DictUtil.dict_of(QuestCatalog.get_entry(template_id).get("rewards", {}))]
* `pools`: Array[String] = JsonUtil.strings(q.get("loot_pool", []))
* `picks`: int = DictUtil.to_int(q.get("loot_picks", 1)) if not pools.is_empty() else 0
* `set_d`: Dictionary = DictUtil.dict_of(DictUtil.dict_of(v).get("set", {}))
* `special`: Dictionary = {"passive": 0, "skill": 0, "item": 0}
* `loot`: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
* `kind`: String = str(loot.get("reward_kind", "item"))
* `amount`: int = DictUtil.to_int(loot.get("count", 1)) if kind == "item" else 1
* `qid`: String = str(q["_qid"])
* `passives`: int = DictUtil.array_of(r.get("passives", [])).size() + DictUtil.to_int(special["passive"])
* `skills`: int = DictUtil.array_of(r.get("skills", [])).size() + DictUtil.to_int(special["skill"])
* `items`: int = item_total(r) + DictUtil.to_int(special["item"])

# Methods

### `static required_binding(kind: String) -> String`
```logic
var keys: Array = BINDINGS.get(kind, [])
return str(keys[0]) if not keys.is_empty() else ""
```

### `static is_int_value(v: Variant) -> bool`
```logic
return v is int
```

### `static flag_value_ok(v: Variant) -> bool`
```logic
return v is bool or v is int or v is String
```

### `static reward_ref_error(kind: String, ref: String) -> String`
```logic
if ref.is_empty():
	return "a %s reward needs an id" % kind
match kind:
	"item":
		if ContentCatalog.get_entry("items", ref).is_empty():
			return "unknown item '%s'" % ref
	"passive":
		if ContentCatalog.get_entry("passives", ref).is_empty():
			return "unknown passive '%s'" % ref
	"skill":
		if ContentCatalog.skill_entry(ref).is_empty():
			return "unknown skill family '%s' (name the family head)" % ref
		if SkillCatalog.is_basic_family(ref):
			return "'%s' is a basic skill every unit owns; it can't be a reward" % ref
	_:
		return "unknown reward kind '%s'" % kind
return ""
```

### `static item_total(rewards: Dictionary) -> int`
```logic
var n: int = 0
var items: Dictionary = DictUtil.dict_of(rewards.get("items", {}))
for id: Variant in items:
	n += DictUtil.to_int(items[id])
return n
```

### `static condition_met(gs: GameState, cond: Dictionary) -> bool`
```logic
var want: Variant = JsonUtil.normalize(cond.get("value", true))
var flag_key: String = str(cond.get("key", ""))
var op: String = str(cond.get("op", "=="))
match op:
	"==", "!=":
		var have: Variant = gs.get_flag(flag_key, false)
		var same: bool = typeof(have) == typeof(want) and have == want
		return same if op == "==" else not same
	">=", "<=", ">", "<":
		var raw: Variant = gs.get_flag(flag_key, 0)
		if not (raw is int) or not (want is int):
			return false
		var a: int = DictUtil.to_int(raw)
		var b: int = DictUtil.to_int(want)
		match op:
			">=":
				return a >= b
			"<=":
				return a <= b
			">":
				return a > b
		return a < b
return false
```

### `static added_flag(gs: GameState, flag_key: String, delta: int, lo: Variant = null, hi: Variant = null) -> Dictionary`
```logic
var cur: Variant = gs.get_flag(flag_key, 0)
if not (cur is int):
	return {"error": "Flag '%s' is not a number." % flag_key}
var v: int = DictUtil.to_int(cur) + delta
if lo is int:
	v = maxi(v, DictUtil.to_int(lo))
if hi is int:
	v = mini(v, DictUtil.to_int(hi))
if RepRules.is_rep_key(flag_key):
	v = clampi(v, RepRules.MIN, RepRules.MAX)
return {"value": v, "error": ""}
```

### `static blob_key_ok(key: String) -> bool`
```logic
if key.count(":") != 1:
	return false
var parts: PackedStringArray = key.split(":")
return JsonUtil.id_ok(parts[0]) and JsonUtil.id_ok(parts[1])
```

### `static variant_applies(gs: GameState, variant: Dictionary) -> bool`
```logic
for c: Variant in DictUtil.array_of(variant.get("when_flags", [])):
	if not condition_met(gs, DictUtil.dict_of(c)):
		return false
for c: Variant in DictUtil.array_of(variant.get("unless_flags", [])):
	if condition_met(gs, DictUtil.dict_of(c)):
		return false
return true
```

### `static resolve(gs: GameState, entry: Dictionary) -> Dictionary`
```logic
var out: Dictionary = DictUtil.dict_of(JsonUtil.normalize(WorldSchema.normalize("quests", entry)))
for key: String in ["_world", "_qid"]:   # normalize drops "_" keys; keep the merge-view ones
	if entry.has(key):
		out[key] = entry[key]
var template: Dictionary = QuestCatalog.get_entry(str(out.get("template", "")))
out["objectives"] = DictUtil.array_of(template.get("objectives", [])).duplicate(true)
out["rewards"] = DictUtil.dict_of(template.get("rewards", {})).duplicate(true)
out["available"] = true
var applied: Array[String] = []
for raw: Variant in DictUtil.array_of(out.get("variants", [])):
	var v: Dictionary = DictUtil.dict_of(raw)
	if not variant_applies(gs, v):
		continue
	applied.append(str(v.get("id", "")))
	var set_d: Dictionary = DictUtil.dict_of(v.get("set", {}))
	for key: Variant in set_d:
		var k: String = str(key)
		if not VARIANT_FIELDS.has(k):
			continue
		match k:
			"steps":
				var steps: Array = DictUtil.array_of(out.get("steps", []))
				var partials: Dictionary = DictUtil.dict_of(set_d[key])
				for idx: Variant in partials:
					var i: int = str(idx).to_int()
					if str(idx).is_valid_int() and i >= 0 and i < steps.size():
						var step: Dictionary = DictUtil.dict_of(steps[i]).duplicate(true)
						step.merge(DictUtil.dict_of(partials[idx]).duplicate(true), true)
						steps[i] = step
				out["steps"] = steps
			"rewards_from":
				out["rewards"] = DictUtil.dict_of(QuestCatalog.get_entry(str(set_d[key])).get("rewards", {})).duplicate(true)
			_:
				out[k] = JsonUtil.normalize(set_d[key])   # a deep copy
if str(out.get("turn_in", "")).is_empty():
	out["turn_in"] = str(out.get("giver", ""))
out["applied_variants"] = applied
return out
```

