class_name QuestRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Quest Rules (S8, spec M4-L)
# Constants and pure validation helpers for quest templates (QuestCatalog), the
# quest fields of world quests, world loot entries (WorldCatalog kind `loot`),
# flag conditions and the stored quest blob. No state, no RNG.
# World quests bind a lore-free template to world targets; `variants` override a
# whitelisted set of fields when flags hold; `loot_pool` offers world loot entries.
# ==============================================================================

const OBJECTIVE_KINDS: Array[String] = ["reach_location", "win_encounter", "flag"]
const MAX_OBJECTIVES: int = 8
const REWARD_KEYS: Array[String] = ["xp", "items", "passives", "skills"]
const XP_MAX: int = 2000
const ITEM_COUNT_MAX: int = 99
const MAX_ITEM_KEYS: int = 8

# Soft reward budget (warnings only; the balance pass may move them).
const SOFT_MAX_PASSIVES: int = 1
const SOFT_MAX_SKILLS: int = 1
const SOFT_MAX_XP: int = 300
const SOFT_MAX_ITEMS: int = 6
const SOFT_MAX_PICKS: int = 1

const MAX_VARIANTS: int = 8
const VARIANT_KEYS: Array[String] = ["id", "when_flags", "unless_flags", "set"]
const VARIANT_FIELDS: Array[String] = ["giver", "turn_in", "description", "reward", "rewards_from", "steps", "links", "available", "loot_pool", "loot_picks"]

# Link key -> the world kind its refs name.
const LINK_KINDS: Dictionary = {
	"follows": "quests", "leads_to": "quests", "related_quests": "quests",
	"related_characters": "characters", "related_locations": "locations", "related_factions": "factions",
}

const MAX_POOL: int = 8
const MAX_PICKS: int = 3
const LOOT_KINDS: Array[String] = ["item", "passive", "skill"]
const RARITIES: Array[String] = ["common", "rare", "unique"]

const STORED_STATES: Array[String] = ["active", "ready", "completed"]
const BLOB_KEYS: Array[String] = ["state", "objective"]

# Step keys an objective kind binds (the first is required).
const BINDINGS: Dictionary = {
	"reach_location": ["location"],
	"win_encounter": ["encounter", "location"],
	"flag": ["flag", "value"],
}

static func required_binding(kind: String) -> String:
	var keys: Array = BINDINGS.get(kind, [])
	return str(keys[0]) if not keys.is_empty() else ""

static func binding_keys(kind: String) -> Array[String]:
	var out: Array[String] = []
	for k: Variant in DictUtil.array_of(BINDINGS.get(kind, [])):
		out.append(str(k))
	return out

# ------------------------------------------------------------------------------
# Values
# ------------------------------------------------------------------------------

# Callers normalise first (JsonUtil.normalize: whole floats become ints), so a non-whole number fails.
static func is_int_value(v: Variant) -> bool:
	return v is int

static func flag_value_ok(v: Variant) -> bool:
	return v is bool or v is int or v is String

# ------------------------------------------------------------------------------
# Rewards (templates and loot entries share these)
# ------------------------------------------------------------------------------

# "" when `ref` can be granted as a reward of `kind` (item, passive, skill), else why not.
static func reward_ref_error(kind: String, ref: String) -> String:
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

static func rewards_errors(raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("Rewards must be an object {xp, items, passives, skills}.")
		return errors
	var rewards: Dictionary = JsonUtil.normalize(raw)
	for key: Variant in rewards:
		if not REWARD_KEYS.has(str(key)):
			errors.append("Unknown reward key '%s' (allowed: %s)." % [str(key), ", ".join(REWARD_KEYS)])
	if rewards.has("xp"):
		var xp: Variant = rewards["xp"]
		if not is_int_value(xp):
			errors.append("Rewards xp must be a whole number.")
		elif DictUtil.to_int(xp) < 0 or DictUtil.to_int(xp) > XP_MAX:
			errors.append("Rewards xp must be 0..%d." % XP_MAX)
	if rewards.has("items"):
		if not (rewards["items"] is Dictionary):
			errors.append("Rewards items must be an object {item_id: count}.")
		else:
			var items: Dictionary = rewards["items"]
			if items.size() > MAX_ITEM_KEYS:
				errors.append("Rewards list at most %d items." % MAX_ITEM_KEYS)
			for id: Variant in items:
				var why: String = reward_ref_error("item", str(id))
				if not why.is_empty():
					errors.append("Rewards items: %s." % why)
				var count: Variant = items[id]
				if not is_int_value(count) or DictUtil.to_int(count) < 1 or DictUtil.to_int(count) > ITEM_COUNT_MAX:
					errors.append("Rewards items: '%s' count must be a whole number 1..%d." % [str(id), ITEM_COUNT_MAX])
	for pair: Array in [["passives", "passive"], ["skills", "skill"]]:
		var key: String = str(pair[0])
		if not rewards.has(key):
			continue
		if not (rewards[key] is Array):
			errors.append("Rewards %s must be a list of ids." % key)
			continue
		var seen: Dictionary = {}
		for v: Variant in DictUtil.array_of(rewards[key]):
			var id: String = str(v)
			if not (v is String):
				errors.append("Rewards %s: '%s' is not an id." % [key, id])
				continue
			if seen.has(id):
				errors.append("Rewards %s lists '%s' twice." % [key, id])
			seen[id] = true
			var why: String = reward_ref_error(str(pair[1]), id)
			if not why.is_empty():
				errors.append("Rewards %s: %s." % [key, why])
	return errors

static func item_total(rewards: Dictionary) -> int:
	var n: int = 0
	var items: Dictionary = DictUtil.dict_of(rewards.get("items", {}))
	for id: Variant in items:
		n += DictUtil.to_int(items[id])
	return n

static func rewards_warnings(raw: Variant) -> Array[String]:
	var out: Array[String] = []
	var rewards: Dictionary = DictUtil.dict_of(JsonUtil.normalize(raw))
	var passives: int = DictUtil.array_of(rewards.get("passives", [])).size()
	var skills: int = DictUtil.array_of(rewards.get("skills", [])).size()
	if passives > SOFT_MAX_PASSIVES:
		out.append("Rewards %d passives (soft cap %d)." % [passives, SOFT_MAX_PASSIVES])
	if skills > SOFT_MAX_SKILLS:
		out.append("Rewards %d skills (soft cap %d)." % [skills, SOFT_MAX_SKILLS])
	if DictUtil.to_int(rewards.get("xp", 0)) > SOFT_MAX_XP:
		out.append("Rewards %d xp (soft cap %d)." % [DictUtil.to_int(rewards.get("xp", 0)), SOFT_MAX_XP])
	if item_total(rewards) > SOFT_MAX_ITEMS:
		out.append("Rewards %d items in total (soft cap %d)." % [item_total(rewards), SOFT_MAX_ITEMS])
	return out

# ------------------------------------------------------------------------------
# Templates
# ------------------------------------------------------------------------------

# Problems of a template's own content (id uniqueness is QuestCatalog's).
static func template_errors(entry: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(entry.get("id", ""))
	if not JsonUtil.id_ok(id):
		errors.append("ID '%s' must be lowercase snake_case (a-z, 0-9, _)." % id)
	if not (entry.get("name", "") is String) or str(entry.get("name", "")).strip_edges().is_empty():
		errors.append("Name is required.")
	if not (entry.get("description", "") is String):
		errors.append("Description must be text.")
	var objectives: Variant = entry.get("objectives", null)
	if not (objectives is Array):
		errors.append("Objectives must be a list of {\"kind\": ...}.")
	else:
		var list: Array = objectives
		if list.is_empty() or list.size() > MAX_OBJECTIVES:
			errors.append("A template has 1..%d objectives (has %d)." % [MAX_OBJECTIVES, list.size()])
		for i: int in range(list.size()):
			if not (list[i] is Dictionary):
				errors.append("Objective %d must be an object {\"kind\": ...}." % (i + 1))
				continue
			var obj: Dictionary = list[i]
			var kind: String = str(obj.get("kind", ""))
			if not OBJECTIVE_KINDS.has(kind):
				errors.append("Objective %d: kind '%s' is not one of %s." % [i + 1, kind, ", ".join(OBJECTIVE_KINDS)])
			for key: Variant in obj:
				if str(key) != "kind":
					errors.append("Objective %d: unknown key '%s' (targets are bound by the world quest's steps)." % [i + 1, str(key)])
	errors.append_array(rewards_errors(entry.get("rewards", {})))
	if not (entry.get("tags", []) is Array):
		errors.append("Tags must be a list of strings.")
	return errors

static func template_objective_kinds(template: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for obj: Variant in DictUtil.array_of(template.get("objectives", [])):
		out.append(str(DictUtil.dict_of(obj).get("kind", "")))
	return out

# ------------------------------------------------------------------------------
# Flag conditions
# ------------------------------------------------------------------------------

# `raw`: Array of {"key": K, "value": V} (value defaults to true).
static func condition_errors(raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("%s must be a list of {\"key\", \"value\"} conditions." % label)
		return errors
	var list: Array = JsonUtil.normalize(raw)
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("%s %d must be an object {\"key\", \"value\"}." % [label, i + 1])
			continue
		var cond: Dictionary = list[i]
		var key: Variant = cond.get("key", "")
		if not (key is String) or not GameState.flag_key_ok(str(key)):
			errors.append("%s %d: flag key '%s' must be non-empty [a-z0-9_.]." % [label, i + 1, str(key)])
		if cond.has("value") and not flag_value_ok(cond["value"]):
			errors.append("%s %d: value must be a bool, a whole number or a String." % [label, i + 1])
		for k: Variant in cond:
			if not ["key", "value"].has(str(k)):
				errors.append("%s %d: unknown key '%s'." % [label, i + 1, str(k)])
	return errors

# Strict type equality: 1 does not match true; a missing flag reads as false.
static func condition_met(gs: GameState, cond: Dictionary) -> bool:
	var want: Variant = JsonUtil.normalize(cond.get("value", true))
	var have: Variant = gs.get_flag(str(cond.get("key", "")), false)
	return typeof(have) == typeof(want) and have == want

# ------------------------------------------------------------------------------
# Stored quest blob (GameState.quests)
# ------------------------------------------------------------------------------

static func blob_key_ok(key: String) -> bool:
	if key.count(":") != 1:
		return false
	var parts: PackedStringArray = key.split(":")
	return JsonUtil.id_ok(parts[0]) and JsonUtil.id_ok(parts[1])

static func blob_entry_errors(key: Variant, value: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (key is String) or not blob_key_ok(str(key)):
		errors.append("quests: key '%s' must be a qualified id world:id." % str(key))
	if not (value is Dictionary):
		errors.append("quests.%s must be an object {state, objective}." % str(key))
		return errors
	var d: Dictionary = value
	for k: Variant in d:
		if not BLOB_KEYS.has(str(k)):
			errors.append("quests.%s: unknown key '%s'." % [str(key), str(k)])
	for k: String in BLOB_KEYS:
		if not d.has(k):
			errors.append("quests.%s: missing '%s'." % [str(key), k])
	if d.has("state") and (not (d["state"] is String) or not STORED_STATES.has(str(d["state"]))):
		errors.append("quests.%s: state '%s' is not one of %s." % [str(key), str(d.get("state", "")), ", ".join(STORED_STATES)])
	if d.has("objective"):
		var o: Variant = d["objective"]
		if not (o is int) or DictUtil.to_int(o) < 0 or DictUtil.to_int(o) > MAX_OBJECTIVES:
			errors.append("quests.%s: objective must be a whole number 0..%d." % [str(key), MAX_OBJECTIVES])
	return errors

static func validate_blob(blob: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in blob:
		errors.append_array(blob_entry_errors(key, blob[key]))
	return errors

# ------------------------------------------------------------------------------
# World quests (called by WorldCatalog._quest_errors)
# ------------------------------------------------------------------------------

static func world_quest_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var template_id: String = str(e.get("template", ""))
	var template: Dictionary = {}
	if not template_id.is_empty():
		template = QuestCatalog.get_entry(template_id)
		if template.is_empty():
			errors.append("Template: unknown quest template '%s'." % template_id)
	var has_template: bool = not template_id.is_empty()
	errors.append_array(condition_errors(e.get("requires_flags", []), "Requires flags"))
	errors.append_array(condition_errors(e.get("complete_flags", []), "Complete flags"))
	if DictUtil.to_bool(e.get("auto_start", false)) and not has_template:
		errors.append("Auto start needs a template (a lore-only quest never runs).")
	var steps: Array = DictUtil.array_of(JsonUtil.normalize(e.get("steps", [])))
	var kinds: Array[String] = template_objective_kinds(template)
	if has_template and not template.is_empty():
		if steps.size() != kinds.size():
			errors.append("The template has %d objectives but the quest has %d steps (one step per objective)." % [kinds.size(), steps.size()])
		for i: int in range(mini(steps.size(), kinds.size())):
			errors.append_array(step_binding_errors(world_id, i, kinds[i], DictUtil.dict_of(steps[i]), true))
	errors.append_array(cycle_errors(world_id, e))
	errors.append_array(links_errors(world_id, e, e.get("links", {}), "Links"))
	var pool: Variant = e.get("loot_pool", [])
	var pool_size: int = DictUtil.array_of(pool).size()
	errors.append_array(loot_pool_errors(world_id, pool, "Loot pool"))
	if pool_size > 0 and not has_template:
		errors.append("Loot pool needs a template (a lore-only quest grants nothing).")
	errors.append_array(picks_errors(e.get("loot_picks", 1), "Loot picks"))
	if pool_size > 0 and DictUtil.to_int(e.get("loot_picks", 1)) > pool_size:
		errors.append("Loot picks (%d) is more than the pool holds (%d)." % [DictUtil.to_int(e.get("loot_picks", 1)), pool_size])
	var variants: Variant = e.get("variants", [])
	if not (variants is Array):
		errors.append("Variants must be a list.")
	elif not DictUtil.array_of(variants).is_empty():
		if not has_template:
			errors.append("Variants need a template (a lore-only quest never runs).")
		errors.append_array(variants_errors(world_id, e, kinds))
	return errors

# One templated step: required binding present and valid; text required. `full`: every key must be present
# (base steps); variant partial steps pass full = false.
static func step_binding_errors(world_id: String, index: int, kind: String, step: Dictionary, full: bool) -> Array[String]:
	var errors: Array[String] = []
	var label: String = "Quest step %d (%s)" % [index + 1, kind]
	var req: String = required_binding(kind)
	if full and str(step.get(req, "")).strip_edges().is_empty():
		errors.append("%s needs \"%s\"." % [label, req])
	elif step.has(req) and str(step[req]).strip_edges().is_empty():
		errors.append("%s: \"%s\" can't be emptied." % [label, req])
	if not full and step.has("text") and (not (step["text"] is String) or str(step["text"]).strip_edges().is_empty()):
		errors.append("%s: \"text\" can't be emptied." % label)
	for key: String in binding_keys(kind):
		if not step.has(key):
			continue
		var v: Variant = step[key]
		var s: String = str(v)
		match key:
			"location":
				if not s.is_empty():
					errors.append_array(WorldCatalog.ref_errors(world_id, label + " location", "locations", s))
			"encounter":
				if not s.is_empty():
					errors.append_array(WorldCatalog.ref_errors(world_id, label + " encounter", "encounters", s))
			"flag":
				if not (v is String) or not GameState.flag_key_ok(s):
					errors.append("%s: flag key '%s' must be non-empty [a-z0-9_.]." % [label, s])
			"value":
				if not flag_value_ok(v):
					errors.append("%s: value must be a bool, a whole number or a String." % label)
	return errors

# `requires` may not lead back to the quest (refs resolve from the quest's world).
static func cycle_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var own_qid: String = WorldCatalog.qid(world_id, str(e.get("id", "")))
	var seen: Dictionary = {}
	var queue: Array[String] = JsonUtil.strings(e.get("requires", []))
	var guard: int = 0
	while not queue.is_empty() and guard < 512:
		guard += 1
		var ref: String = queue.pop_front()
		var hit: Dictionary = WorldCatalog.resolve(world_id, "quests", ref)
		if hit.is_empty():
			continue
		var hit_qid: String = str(hit.get("_qid", ""))
		if hit_qid == own_qid or str(hit.get("override", "")) == own_qid:
			errors.append("Requires: the quest chain is circular (it leads back to '%s')." % str(e.get("id", "")))
			break
		if seen.has(hit_qid):
			continue
		seen[hit_qid] = true
		queue.append_array(JsonUtil.strings(hit.get("requires", [])))
	return errors

static func links_errors(world_id: String, e: Dictionary, raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("%s must be an object {follows, leads_to, related_*}." % label)
		return errors
	var links: Dictionary = raw
	var own_qid: String = WorldCatalog.qid(world_id, str(e.get("id", "")))
	for key: Variant in links:
		var k: String = str(key)
		if not LINK_KINDS.has(k):
			errors.append("%s: unknown key '%s' (allowed: %s)." % [label, k, ", ".join(PackedStringArray(LINK_KINDS.keys()))])
			continue
		if not (links[key] is Array):
			errors.append("%s %s must be a list of refs." % [label, k])
			continue
		var seen: Dictionary = {}
		for v: Variant in DictUtil.array_of(links[key]):
			var ref: String = str(v)
			if seen.has(ref):
				errors.append("%s %s lists '%s' twice." % [label, k, ref])
			seen[ref] = true
			var ref_kind: String = str(LINK_KINDS[k])
			errors.append_array(WorldCatalog.ref_errors(world_id, "%s %s" % [label, k], ref_kind, ref))
			# No quest link may name the quest itself (follows, leads_to and related_quests).
			if ref_kind == "quests" and (ref == str(e.get("id", "")) or ref == own_qid or str(WorldCatalog.resolve(world_id, "quests", ref).get("_qid", "")) == own_qid):
				errors.append("%s %s: a quest can't name itself." % [label, k])
	return errors

static func loot_pool_errors(world_id: String, raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("%s must be a list of loot refs." % label)
		return errors
	var list: Array = raw
	if list.size() > MAX_POOL:
		errors.append("%s holds at most %d entries (has %d)." % [label, MAX_POOL, list.size()])
	var seen: Dictionary = {}
	for v: Variant in list:
		var ref: String = str(v)
		var hit: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
		var key: String = str(hit.get("_qid", ref))
		if seen.has(key):
			errors.append("%s lists '%s' twice." % [label, ref])
		seen[key] = true
		errors.append_array(WorldCatalog.ref_errors(world_id, label, "loot", ref))
	return errors

static func picks_errors(raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	var v: Variant = JsonUtil.normalize(raw)
	if not (v is int) or DictUtil.to_int(v) < 0 or DictUtil.to_int(v) > MAX_PICKS:
		errors.append("%s must be a whole number 0..%d." % [label, MAX_PICKS])
	return errors

static func variants_errors(world_id: String, e: Dictionary, kinds: Array[String]) -> Array[String]:
	var errors: Array[String] = []
	var list: Array = DictUtil.array_of(JsonUtil.normalize(e.get("variants", [])))
	if list.size() > MAX_VARIANTS:
		errors.append("A quest has at most %d variants (has %d)." % [MAX_VARIANTS, list.size()])
	var ids: Dictionary = {}
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("Variant %d must be an object {id, when_flags, unless_flags, set}." % (i + 1))
			continue
		var v: Dictionary = list[i]
		var vid: String = str(v.get("id", ""))
		var label: String = "Variant '%s'" % vid if not vid.is_empty() else "Variant %d" % (i + 1)
		if not JsonUtil.id_ok(vid):
			errors.append("%s: id must be lowercase snake_case (a-z, 0-9, _)." % label)
		elif ids.has(vid):
			errors.append("%s: id is used twice in this quest." % label)
		ids[vid] = true
		for key: Variant in v:
			if not VARIANT_KEYS.has(str(key)):
				errors.append("%s: unknown key '%s' (allowed: %s)." % [label, str(key), ", ".join(VARIANT_KEYS)])
		errors.append_array(condition_errors(v.get("when_flags", []), label + " when flag"))
		errors.append_array(condition_errors(v.get("unless_flags", []), label + " unless flag"))
		if DictUtil.array_of(v.get("when_flags", [])).is_empty() and DictUtil.array_of(v.get("unless_flags", [])).is_empty():
			errors.append("%s has no condition (an unconditional variant: edit the quest instead)." % label)
		var set_raw: Variant = v.get("set", null)
		if not (set_raw is Dictionary) or DictUtil.dict_of(set_raw).is_empty():
			errors.append("%s needs a non-empty \"set\" object." % label)
			continue
		errors.append_array(variant_set_errors(world_id, e, kinds, DictUtil.dict_of(set_raw), label))
	return errors

static func variant_set_errors(world_id: String, e: Dictionary, kinds: Array[String], set_d: Dictionary, label: String) -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in set_d:
		var k: String = str(key)
		var value: Variant = set_d[key]
		if not VARIANT_FIELDS.has(k):
			errors.append("%s: '%s' can't be overridden (allowed: %s)." % [label, k, ", ".join(VARIANT_FIELDS)])
			continue
		match k:
			"giver", "turn_in":
				if not (value is String):
					errors.append("%s: %s must be a character ref or \"\"." % [label, k])
				elif not str(value).is_empty():
					errors.append_array(WorldCatalog.ref_errors(world_id, "%s %s" % [label, k], "characters", str(value)))
			"description", "reward":
				if not (value is String):
					errors.append("%s: %s must be text." % [label, k])
			"rewards_from":
				if not (value is String) or QuestCatalog.get_entry(str(value)).is_empty():
					errors.append("%s: rewards_from names unknown quest template '%s'." % [label, str(value)])
			"available":
				if not (value is bool):
					errors.append("%s: available must be true or false." % label)
			"links":
				errors.append_array(links_errors(world_id, e, value, label + " links"))
			"loot_pool":
				errors.append_array(loot_pool_errors(world_id, value, label + " loot pool"))
			"loot_picks":
				errors.append_array(picks_errors(value, label + " loot picks"))
			"steps":
				errors.append_array(variant_steps_errors(world_id, kinds, value, label))
	return errors

static func variant_steps_errors(world_id: String, kinds: Array[String], raw: Variant, label: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("%s: steps must be an object {\"<index>\": {partial step}}." % label)
		return errors
	var steps: Dictionary = raw
	if kinds.is_empty():
		return errors   # unknown template: reported once already; per-step index errors would only add noise
	for key: Variant in steps:
		var s: String = str(key)
		if not s.is_valid_int() or s.to_int() < 0 or s.to_int() >= kinds.size():
			errors.append("%s: step index '%s' is out of range (0..%d)." % [label, s, kinds.size() - 1])
			continue
		var idx: int = s.to_int()
		if not (steps[key] is Dictionary):
			errors.append("%s: step %s must be an object." % [label, s])
			continue
		var partial: Dictionary = steps[key]
		var allowed: Array[String] = ["text"]
		allowed.append_array(binding_keys(kinds[idx]))
		for pk: Variant in partial:
			if not allowed.has(str(pk)):
				errors.append("%s: step %s (%s) can't set '%s' (allowed: %s)." % [label, s, kinds[idx], str(pk), ", ".join(allowed)])
		errors.append_array(step_binding_errors(world_id, idx, kinds[idx], partial, false))
	return errors

# ------------------------------------------------------------------------------
# World loot entries (called by WorldCatalog._loot_errors)
# ------------------------------------------------------------------------------

static func loot_entry_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var kind: String = str(e.get("reward_kind", "item"))
	var ref: String = str(e.get("ref", ""))
	if LOOT_KINDS.has(kind):
		var why: String = reward_ref_error(kind, ref)
		if not why.is_empty():
			errors.append("Reward: %s." % why)
	var count: Variant = JsonUtil.normalize(e.get("count", 1))
	if kind == "item":
		if not (count is int) or DictUtil.to_int(count) < 1 or DictUtil.to_int(count) > ITEM_COUNT_MAX:
			errors.append("Count must be 1..%d for an item." % ITEM_COUNT_MAX)
	elif not (count is int) or DictUtil.to_int(count) != 1:
		errors.append("Count must be 1 for a %s." % kind)
	errors.append_array(condition_errors(e.get("requires_flags", []), "Requires flags"))
	return errors

# ------------------------------------------------------------------------------
# Soft warnings over a world (Forge and tests)
# ------------------------------------------------------------------------------

# One-sided chains: A leads_to B without B follows A (or the reverse).
static func link_warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for q: Dictionary in WorldCatalog.merged_entries(world_id, "quests"):
		var links: Dictionary = DictUtil.dict_of(q.get("links", {}))
		for pair: Array in [["leads_to", "follows"], ["follows", "leads_to"]]:
			for ref: String in JsonUtil.strings(links.get(str(pair[0]), [])):
				var other: Dictionary = WorldCatalog.resolve(world_id, "quests", ref)
				if other.is_empty():
					continue
				var back: bool = false
				for r2: String in JsonUtil.strings(DictUtil.dict_of(other.get("links", {})).get(str(pair[1]), [])):
					if str(WorldCatalog.resolve(world_id, "quests", r2).get("_qid", "")) == str(q["_qid"]):
						back = true
				if not back:
					out.append("%s %s %s, but %s does not list it in %s." % [str(q["_qid"]), str(pair[0]), str(other["_qid"]), str(other["_qid"]), str(pair[1])])
	return out

# Worst case over the base and every variant: generic rewards plus ONE pooled special of a kind above that kind's soft
# cap, and loot_picks above SOFT_MAX_PICKS.
static func reward_warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for q: Dictionary in WorldCatalog.merged_entries(world_id, "quests"):
		var template_id: String = str(q.get("template", ""))
		if template_id.is_empty():
			continue
		var reward_sets: Array[Dictionary] = [DictUtil.dict_of(QuestCatalog.get_entry(template_id).get("rewards", {}))]
		var pools: Array[String] = JsonUtil.strings(q.get("loot_pool", []))
		var picks: int = DictUtil.to_int(q.get("loot_picks", 1)) if not pools.is_empty() else 0
		for v: Variant in DictUtil.array_of(q.get("variants", [])):
			var set_d: Dictionary = DictUtil.dict_of(DictUtil.dict_of(v).get("set", {}))
			if set_d.has("rewards_from"):
				reward_sets.append(DictUtil.dict_of(QuestCatalog.get_entry(str(set_d["rewards_from"])).get("rewards", {})))
			if set_d.has("loot_pool"):
				for ref: String in JsonUtil.strings(set_d["loot_pool"]):
					if not pools.has(ref):
						pools.append(ref)
			if set_d.has("loot_picks"):
				picks = maxi(picks, DictUtil.to_int(set_d["loot_picks"]))
		var special: Dictionary = {"passive": 0, "skill": 0, "item": 0}
		for ref: String in pools:
			var loot: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
			if loot.is_empty():
				continue
			var kind: String = str(loot.get("reward_kind", "item"))
			var amount: int = DictUtil.to_int(loot.get("count", 1)) if kind == "item" else 1
			special[kind] = maxi(DictUtil.to_int(special.get(kind, 0)), amount)
		var qid: String = str(q["_qid"])
		for r: Dictionary in reward_sets:
			var passives: int = DictUtil.array_of(r.get("passives", [])).size() + DictUtil.to_int(special["passive"])
			var skills: int = DictUtil.array_of(r.get("skills", [])).size() + DictUtil.to_int(special["skill"])
			var items: int = item_total(r) + DictUtil.to_int(special["item"])
			if DictUtil.to_int(special["passive"]) > 0 and passives > SOFT_MAX_PASSIVES:
				out.append("%s: %d passives with a pooled passive (soft cap %d)." % [qid, passives, SOFT_MAX_PASSIVES])
			if DictUtil.to_int(special["skill"]) > 0 and skills > SOFT_MAX_SKILLS:
				out.append("%s: %d skills with a pooled skill (soft cap %d)." % [qid, skills, SOFT_MAX_SKILLS])
			if DictUtil.to_int(special["item"]) > 0 and items > SOFT_MAX_ITEMS:
				out.append("%s: %d items with a pooled item (soft cap %d)." % [qid, items, SOFT_MAX_ITEMS])
		if picks > SOFT_MAX_PICKS:
			out.append("%s: %d loot picks (soft cap %d)." % [qid, picks, SOFT_MAX_PICKS])
	return out

# ------------------------------------------------------------------------------
# Resolution (pure: the world entry + gs.flags; no RNG, no clock)
# ------------------------------------------------------------------------------

# True when every `when_flags` condition holds and no `unless_flags` condition does.
static func variant_applies(gs: GameState, variant: Dictionary) -> bool:
	for c: Variant in DictUtil.array_of(variant.get("when_flags", [])):
		if not condition_met(gs, DictUtil.dict_of(c)):
			return false
	for c: Variant in DictUtil.array_of(variant.get("unless_flags", [])):
		if condition_met(gs, DictUtil.dict_of(c)):
			return false
	return true

# The quest as the flags make it now: a copy of `entry` (json numbers normalised) with the template's `objectives` and
# `rewards` attached, then every applying variant's `set` merged in list order (field replace; `steps` per index and
# key; `rewards_from` swaps the rewards). Adds `available` (default true), `applied_variants` (ids in order) and
# `turn_in` falls back to the giver when empty.
# Every schema field is present with its WorldSchema default (loot_picks, loot_pool...), so readers never re-state defaults.
static func resolve(gs: GameState, entry: Dictionary) -> Dictionary:
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
