extends SceneTree

# PLAYTEST1 headless replay tool (docs/specs/PLAYTEST1_L.md rules 9-12; logic in PlaytestReplayer.run_file).
#   godot --headless --path . -s tools/replay.gd -- <file or id> [args]      (tools/fc.sh replay <file> [args])
#   --battle=K            only battle K (default all)
#   --json                one machine-readable report line instead of the text lines
#   --variant=<patch>     replay with a variant patch (playtest_variant), --then=stop|recorded, --write=<out.json>
#   --seeds=<a,b|a..b>    seed sweep (max 200), --fallback=stop|<PlanPresets id>
#   --trail-report        flag UI trail moments (dead clicks, repeats, open / close problems, errors, asset fallbacks)
#   --render=notes|rounds --out=<dir>   (rule 13, NOT headless) loads the arena in playback mode, jumps to each
#                         note (or round end), waits two frames and saves note_<n>.png / round_<r>.png:
#     xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . -s tools/replay.gd -- <file> --render=notes --out=<dir>
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
		_out("usage: replay <file or id> [--battle=K] [--json] [--variant=<patch> [--then=stop|recorded] [--write=<out>]] [--seeds=<list|a..b> [--fallback=stop|<preset>]] [--trail-report] [--render=notes|rounds --out=<dir>]")
		quit(2)
		return
	if opts.has("battle"):
		opts["battle"] = str(opts["battle"]).to_int()
	if opts.has("render"):
		_render.call_deferred(PlaytestStore.resolve(file), str(opts["render"]), str(opts.get("out", "")), DictUtil.to_int(opts.get("battle", -1), -1))
		return
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

# ------------------------------------------------------------------------------
# --render (rule 13)
# ------------------------------------------------------------------------------

const ARENA_SCENE: String = "res://scenes/combat_arena.tscn"
const SETTLE_FRAMES: int = 2

func _render(path: String, mode: String, out_dir: String, only_battle: int) -> void:
	var code: int = await _render_all(path, mode, out_dir, only_battle)
	ContentCatalog.clear_cache()
	SkillCatalog.clear_cache()
	quit(code)

func _render_all(path: String, mode: String, out_dir: String, only_battle: int) -> int:
	if mode != "notes" and mode != "rounds":
		_out("ERROR --render must be notes or rounds")
		return 2
	if out_dir.is_empty():
		_out("ERROR --render needs --out=<dir>")
		return 2
	if DisplayServer.get_name() == "headless":
		_out("ERROR --render needs a display: run without --headless under xvfb-run (see the header of tools/replay.gd)")
		return 2
	var loaded: Dictionary = PlaytestStore.load_json(path)
	if not DictUtil.to_bool(loaded.get("ok", false)):
		_out("ERROR %s" % str(loaded.get("error", "")))
		return 2
	var raw: Dictionary = DictUtil.dict_of(loaded["data"])
	var problems: Array[String] = PlaytestRecord.errors(raw)
	if not problems.is_empty():
		_out("ERROR %s" % "; ".join(problems))
		return 2
	var data: Dictionary = PlaytestRecord.normalize(raw)
	var real_out: String = ProjectSettings.globalize_path(out_dir if out_dir.contains("://") or out_dir.is_absolute_path() else "res://" + out_dir)
	if DirAccess.make_dir_recursive_absolute(real_out) != OK:
		_out("ERROR cannot create %s" % real_out)
		return 2
	var written: int = 0
	for raw_battle: Variant in DictUtil.array_of(data["battles"]):
		var k: int = DictUtil.to_int(DictUtil.dict_of(raw_battle)["index"])
		if only_battle >= 0 and k != only_battle:
			continue
		var shots: int = await _render_battle(path, data, k, mode, real_out)
		if shots < 0:
			return 2
		written += shots
	_out("render %s: %d image(s) in %s" % [mode, written, real_out])
	return 0

# Images written for battle k (-1 = the arena could not play it back).
func _render_battle(path: String, data: Dictionary, k: int, mode: String, real_out: String) -> int:
	var arena: Node = (load(ARENA_SCENE) as PackedScene).instantiate()
	arena.set("playtest", path)
	arena.set("battle", k)
	root.add_child(arena)
	await _settle()
	var load_error: String = str(arena.get("load_error"))
	var raw_session: Variant = arena.get("session")
	if not load_error.is_empty() or not (raw_session is CombatSession):
		_out("ERROR battle %d not played back: %s" % [k, load_error])
		root.remove_child(arena)
		arena.queue_free()
		return -1
	var session: CombatSession = raw_session
	var written: int = 0
	if mode == "notes":
		for note: Dictionary in PlaybackBanner.notes_of(data, k):
			PlaybackBanner.jump(session, note)
			if await _shot(real_out.path_join("note_%d.png" % DictUtil.to_int(note["n"]))):
				written += 1
	else:
		for entry: Dictionary in PlaytestRecord.concluded_rounds(PlaytestRecord.battle_at(data, k)):
			var r: int = DictUtil.to_int(entry["round"])
			show_round_end(session, r)
			if await _shot(real_out.path_join("round_%d.png" % r) if only_one_battle(data) else real_out.path_join("b%d_round_%d.png" % [k, r])):
				written += 1
	root.remove_child(arena)
	arena.queue_free()
	await process_frame
	return written

static func only_one_battle(data: Dictionary) -> bool:
	return DictUtil.array_of(data["battles"]).size() <= 1

# The state after the last action of round r (its round end).
static func show_round_end(session: CombatSession, r: int) -> void:
	session.select_round(r)
	var last: ActionLogEntry = null
	for entry: ActionLogEntry in session.log_entries:
		if entry.round_num == r:
			last = entry
	if last != null:
		session.select_log_entry(last)
	else:
		session.select_block(session.total_blocks)

func _settle() -> void:
	for i: int in range(SETTLE_FRAMES):
		await process_frame

func _shot(file: String) -> bool:
	await _settle()
	var image: Image = root.get_texture().get_image()
	var ok: bool = image != null and not image.is_empty() and image.save_png(file) == OK
	_out("%s %s" % ["wrote" if ok else "ERROR could not write", file])
	return ok
