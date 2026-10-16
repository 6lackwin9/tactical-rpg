class_name TestHelpers
extends RefCounted

# Shared statics for the headless suites: the PASS/FAIL report lines, a hand-built combatant, hand-built plan contexts,
# bare-resolver step runs, replay stepping, sandbox catalog forks, and a few skill fixtures.
# Expectations in the suites come from rules and the catalog, never magic numbers.

# Every sandbox lives under this folder as <pid>_<n>_<tag>, so parallel test runs never share files.
const SANDBOX_ROOT: String = "user://test_sandboxes"

static var _sandboxes: Dictionary = {}   # tag -> path, stable for the life of the process

# ------------------------------------------------------------------------------
# Report lines (print tags stay "    [<tag>] <label>: PASS|FAIL ..." so log greps keep working)
# ------------------------------------------------------------------------------

# The keys of `checks` whose value is not true, joined with ", ".
static func failed(checks: Dictionary) -> String:
	return ", ".join(_failed_keys(checks))

static func report(tag: String, label: String, checks: Dictionary) -> bool:
	var keys: Array[String] = _failed_keys(checks)
	var ok: bool = keys.is_empty()
	print("    [%s] %s: %s%s" % [tag, label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(keys)])
	return ok

static func report_ok(tag: String, label: String, ok: bool, detail: String = "") -> bool:
	print("    [%s] %s: %s%s" % [tag, label, "PASS" if ok else "FAIL", (" (%s)" % detail) if not detail.is_empty() else ""])
	return ok

static func _failed_keys(checks: Dictionary) -> Array[String]:
	var keys: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			keys.append(str(key))
	return keys

# ------------------------------------------------------------------------------
# Hand-built units and plans
# ------------------------------------------------------------------------------

# A bare combatant with fixed strike power and no armor (expected numbers stay simple), placed at `pos`.
static func unit(unit_name: String, grid: CombatGrid, pos: Vector2i, hp: int = 400, speed: int = 10, attack_range: int = 1, power: int = 20, learn: Array[String] = []) -> Combatant:
	var c := Combatant.new(unit_name, hp, speed, attack_range)
	c.power_override = power
	c.armor_override = 0
	for skill_id: String in learn:
		c.learn_skill(skill_id)
	grid.place_combatant(c, pos)
	return c

static func resolve_until_replay(session: CombatSession, max_steps: int = 600) -> void:
	for i: int in range(max_steps):
		if session.phase == CombatSession.Phase.REPLAY:
			return
		session.step_next()

static func hand_ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant], cap: int = 3) -> PlanContext:
	var profiles: Dictionary = {}
	for c: Combatant in allies + enemies:
		var p: AiProfile = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.25, cap)
		if c.power_override <= 0:
			c.power_override = 15   # hand-built test units: fixed strike power
		if c.armor_override < 0:
			c.armor_override = 0    # and no armor, so expected numbers stay simple
		profiles[c] = p
	return PlanContext.create(grid, allies, enemies, profiles)

# Commits `steps` for `actor` and plays one full round on a bare resolver.
static func run_steps(actor: Combatant, ctx: PlanContext, steps: Array[PlanStep]) -> Array[String]:
	var resolver := CombatResolver.new()
	resolver.grid = ctx.grid
	var messages_out: Array[String] = []
	resolver.log_emitted.connect(func(msg: String) -> void: messages_out.append(msg))
	var cp := CombatPlan.new(ctx)
	cp.set_steps(actor, steps)
	cp.commit(resolver)
	resolver.resolve_round()
	return messages_out

# ------------------------------------------------------------------------------
# Sandboxes: a user:// folder unique to this process and tag, emptied by fork and removed by cleanup
# ------------------------------------------------------------------------------

# The sandbox folder for `tag`: SANDBOX_ROOT/<pid>_<n>_<tag>. The same tag returns the same path within a process;
# the process id keeps parallel runs (other agents, the IDE) apart, the counter keeps the tags of one process apart.
static func sandbox(tag: String) -> String:
	if not _sandboxes.has(tag):
		_sandboxes[tag] = "%s/%d_%d_%s" % [SANDBOX_ROOT, OS.get_process_id(), _sandboxes.size(), tag]
	return str(_sandboxes[tag])

# Forks the shipped content catalog into `path` (emptied first) and switches to it. True on success.
static func fork(path: String) -> bool:
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	remove_sandbox(path)
	return ContentCatalog.fork_to(path) == OK and ContentCatalog.get_root() == path   # never write to res://

# Back to the shipped content catalog; removes the sandbox folder.
static func cleanup(path: String) -> void:
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	remove_sandbox(path)
	SkillCatalog.clear_cache()

# Forks the shipped worlds into `path` (emptied first) and switches to it. True on success.
static func fork_worlds(path: String) -> bool:
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	remove_sandbox(path)
	return WorldCatalog.fork_to(path) == OK and WorldCatalog.get_root() == path   # never write to res://

# Back to the shipped worlds; removes the sandbox folder.
static func cleanup_worlds(path: String) -> void:
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	remove_sandbox(path)

# Deletes a sandbox folder and everything in it (only below SANDBOX_ROOT). SANDBOX_ROOT itself stays: removing it
# could race with a parallel process that is creating its own sandbox there.
static func remove_sandbox(path: String) -> void:
	if not path.begins_with(SANDBOX_ROOT + "/"):
		push_error("TestHelpers.remove_sandbox refuses a path outside %s: %s" % [SANDBOX_ROOT, path])
		return
	_remove_tree(path)

static func _remove_tree(path: String) -> void:
	if not DirAccess.dir_exists_absolute(path):
		return
	for sub: String in DirAccess.get_directories_at(path):
		_remove_tree(path.path_join(sub))
	for f: String in DirAccess.get_files_at(path):
		DirAccess.remove_absolute(path.path_join(f))
	DirAccess.remove_absolute(path)

# ------------------------------------------------------------------------------
# Skill fixtures
# ------------------------------------------------------------------------------

# A minimal ATTACK base block (a ranked entry without ranks); `extra` overrides or adds keys.
static func attack(id: String, extra: Dictionary = {}) -> Dictionary:
	var base: Dictionary = {"id": id, "name": id.capitalize(), "kind": "ATTACK", "level": 2, "blocks": 2, "power_mult": 1.0, "description": "{name}: {power}"}
	base.merge(extra, true)
	return base

# Messages of a problem list ({"field", "rank", "message"} dictionaries).
static func messages(problems: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for p: Dictionary in problems:
		out.append(str(p["message"]))
	return out

static func any_contains(list: Array[String], needle: String) -> bool:
	for text: String in list:
		if text.contains(needle):
			return true
	return false
