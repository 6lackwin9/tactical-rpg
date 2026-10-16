class_name SkillTestKit
extends RefCounted

# Shared statics for the FORGE1 skill suites (ranks, text, origin, model, form model): sandbox catalog forks, the check
# report line, and a few fixture builders. Expectations in the suites come from rules and the catalog, never magic numbers.

static func report(tag: String, label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [%s] %s: %s%s" % [tag, label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

# Forks the shipped catalog into `sandbox` (a user:// folder) and switches to it. True on success.
static func fork(sandbox: String) -> bool:
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, sandbox))
	return ContentCatalog.fork_to(sandbox) == OK

# Back to the shipped catalog and removes the sandbox files.
static func cleanup(sandbox: String) -> void:
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, sandbox))
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	SkillCatalog.clear_cache()

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
