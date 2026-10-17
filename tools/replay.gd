extends SceneTree

# PLAYTEST1 headless replay tool (docs/specs/PLAYTEST1_L.md rules 9-12; logic in PlaytestReplayer.run_file).
#   godot --headless --path . -s tools/replay.gd -- <file or id> [args]      (tools/fc.sh replay <file> [args])
#   --battle=K            only battle K (default all)
#   --json                one machine-readable report line instead of the text lines
#   --variant=<patch>     replay with a variant patch (playtest_variant), --then=stop|recorded, --write=<out.json>
#   --seeds=<a,b|a..b>    seed sweep (max 200), --fallback=stop|<PlanPresets id>
#   --trail-report        flag UI trail moments (dead clicks, repeats, open / close problems, errors, asset fallbacks)
#   --render=notes|rounds needs the arena playback mode (PLAYTEST1 commit 2)
# Output lines start with "§ ". Exit 0 all match (stale content only warns), 1 divergence or broken plan,
# 2 load or validation error.

func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var file: String = ""
	var opts: Dictionary = {}
	var as_json: bool = false
	for a: String in args:
		if a == "--json":
			as_json = true
		elif a == "--trail-report":
			opts["trail_report"] = true
		elif a.begins_with("--") and a.contains("="):
			var key: String = a.substr(2, a.find("=") - 2).replace("-", "_")
			opts[key] = a.substr(a.find("=") + 1)
		elif not a.begins_with("--"):
			file = a
		else:
			_out("ERROR unknown argument %s" % a)
			quit(2)
			return
	if file.is_empty():
		_out("usage: replay <file or id> [--battle=K] [--json] [--variant=<patch> [--then=stop|recorded] [--write=<out>]] [--seeds=<list|a..b> [--fallback=stop|<preset>]] [--trail-report]")
		quit(2)
		return
	if opts.has("render"):
		_out("ERROR --render needs the arena playback mode (PLAYTEST1 commit 2)")
		quit(2)
		return
	if opts.has("battle"):
		opts["battle"] = str(opts["battle"]).to_int()
	for key: String in ["variant", "write"]:
		if opts.has(key):
			opts[key] = PlaytestStore.resolve(str(opts[key]))
	var result: Dictionary = PlaytestReplayer.run_file(PlaytestStore.resolve(file), opts)
	if as_json:
		_out(JSON.stringify(result["report"]))
	else:
		for line: String in JsonUtil.strings(result["lines"]):
			_out(line)
	ContentCatalog.clear_cache()
	SkillCatalog.clear_cache()
	quit(DictUtil.to_int(result["exit"], 2))

func _out(line: String) -> void:
	print("§ %s" % line)
