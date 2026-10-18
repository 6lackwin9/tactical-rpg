---
class: TestHelpers
extends: RefCounted
description: "Shared statics for the headless suites: the PASS/FAIL report lines, a hand-built combatant, hand-built plan contexts, bare-resolver step runs, replay stepping, sandbox catalog forks, and a few skill fixtures. Expectations in the suites come from rules and the catalog, never magic numbers. Every sandbox lives under this folder as <pid>_<n>_<tag>, so parallel test runs never share files."
---

# Properties
* `SANDBOX_ROOT`: String = "user://test_sandboxes" (const)
* `keys`: Array[String] = []
* `resolver`: Variant
* `messages_out`: Array[String] = []
* `cp`: Variant
* `ITEM1_SEED`: String = "res://data/seeds/item1_migrate.gd" (const)
* `refused`: Array[String] = ["add_item_of_type refuses the shipped catalog (fork first)."]
* `entry`: Dictionary = {"id": id, "name": id.capitalize(), "value": value}
* `missing`: Array[String] = ["No item family of type %s." % item_type]
* `out`: Array[String] = []
* `KNOWN_SHIPPED_CLASS_WARNINGS`: Array[String] = ["families; only the first", "beyond the level cap"] (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `control`: Control = node as Control
* `live`: bool = false

# Methods

### `static failed(checks: Dictionary) -> String`
```logic
return ", ".join(_failed_keys(checks))
```

### `static report(tag: String, label: String, checks: Dictionary) -> bool`
```logic
var keys: Array[String] = _failed_keys(checks)
var ok: bool = keys.is_empty()
print("    [%s] %s: %s%s" % [tag, label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(keys)])
return ok
```

### `static report_ok(tag: String, label: String, ok: bool, detail: String = "") -> bool`
```logic
print("    [%s] %s: %s%s" % [tag, label, "PASS" if ok else "FAIL", (" (%s)" % detail) if not detail.is_empty() else ""])
return ok
```

### `static unit(unit_name: String, grid: CombatGrid, pos: Vector2i, hp: int = 400, speed: int = 10, attack_range: int = 1, power: int = 20, learn: Array[String] = []) -> Combatant`
```logic
var c := Combatant.new(unit_name, hp, speed, attack_range)
c.power_override = power
c.armor_override = 0
for skill_id: String in learn:
	c.learn_skill(skill_id)
grid.place_combatant(c, pos)
return c
```

### `static resolve_until_replay(session: CombatSession, max_steps: int = 600) -> void`
```logic
for i: int in range(max_steps):
	if session.phase == CombatSession.Phase.REPLAY:
		return
	session.step_next()
```

### `static hand_ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant], cap: int = 3) -> PlanContext`
```logic
var profiles: Dictionary = {}
for c: Combatant in allies + enemies:
	var p: AiProfile = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.25, cap)
	if c.power_override <= 0:
		c.power_override = 15   # hand-built test units: fixed strike power
	if c.armor_override < 0:
		c.armor_override = 0    # and no armor, so expected numbers stay simple
	profiles[c] = p
return PlanContext.create(grid, allies, enemies, profiles)
```

### `static sandbox(tag: String) -> String`
```logic
if not _sandboxes.has(tag):
	_sandboxes[tag] = "%s/%d_%d_%s" % [SANDBOX_ROOT, OS.get_process_id(), _sandboxes.size(), tag]
return str(_sandboxes[tag])
```

### `static fork(path: String) -> bool`
```logic
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
remove_sandbox(path)
return ContentCatalog.fork_to(path) == OK and ContentCatalog.get_root() == path   # never write to res://
```

### `static fork_migrated(path: String) -> Dictionary`
```logic
if not fork(path):
	return {"ok": false, "error": "fork failed"}
if ContentCatalog.items_inherit() or not ResourceLoader.exists(ITEM1_SEED):
	return {}
var seed_script: GDScript = load(ITEM1_SEED)
return DictUtil.dict_of(seed_script.call("migrate"))
```

### `static cleanup(path: String) -> void`
```logic
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
remove_sandbox(path)
SkillCatalog.clear_cache()
```

### `static fork_worlds(path: String) -> bool`
```logic
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
remove_sandbox(path)
return WorldCatalog.fork_to(path) == OK and WorldCatalog.get_root() == path   # never write to res://
```

### `static cleanup_worlds(path: String) -> void`
```logic
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
remove_sandbox(path)
```

### `static sandbox_services(dir: String) -> bool`
```logic
ServiceCatalog.use_path(dir.path_join("services.json"))
return ServiceCatalog.file_path().begins_with(SANDBOX_ROOT + "/")
```

### `static restore_services() -> void`
```logic
ServiceCatalog.use_path(ServiceCatalog.DEFAULT_PATH)
```

### `static remove_sandbox(path: String) -> void`
```logic
if not path.begins_with(SANDBOX_ROOT + "/"):
	push_error("TestHelpers.remove_sandbox refuses a path outside %s: %s" % [SANDBOX_ROOT, path])
	return
_remove_tree(path)
```

### `static _remove_tree(path: String) -> void`
```logic
if not DirAccess.dir_exists_absolute(path):
	return
for sub: String in DirAccess.get_directories_at(path):
	_remove_tree(path.path_join(sub))
for f: String in DirAccess.get_files_at(path):
	DirAccess.remove_absolute(path.path_join(f))
DirAccess.remove_absolute(path)
```

### `static attack(id: String, extra: Dictionary = {}) -> Dictionary`
```logic
var base: Dictionary = {"id": id, "name": id.capitalize(), "kind": "ATTACK", "level": 2, "blocks": 2, "power_mult": 1.0, "description": "{name}: {power}"}
base.merge(extra, true)
return base
```

### `static any_contains(list: Array[String], needle: String) -> bool`
```logic
for text: String in list:
	if text.contains(needle):
		return true
return false
```

