extends SceneTree

# SAFE DELETE for a system asset (FORGE1 section 12.5).
#   tools/fc.sh safe_delete <kind> <id> [--apply]
# Scans tests, seeds, class paths, units, passives, `requires` and world files for the id (and every rank id of a skill).
# Refuses (exit 1, nothing changes) while any reference exists, and for basics, required and engine skills.
# With no reference and --apply: removes it through the system-edit scope, saves and prints validate_all.

func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() < 2:
		print("§ usage: safe_delete <kind> <id> [--apply]")
		quit(2)
		return
	var kind: String = args[0]
	var id: String = args[1]
	var apply: bool = args.has("--apply")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var result: Dictionary = AssetScan.safe_delete(kind, id, apply)
	var refs: Array = result["refs"]
	for ref: Variant in refs:
		print("§   referenced: %s" % str(ref))
	print("§ %s" % str(result["message"]))
	ContentCatalog.clear_cache()
	SkillCatalog.clear_cache()
	quit(0 if DictUtil.to_bool(result["ok"]) else 1)
