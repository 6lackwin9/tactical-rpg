extends SceneTree

# Reference screenshots of every active Godot screen (the engine's screen-replication targets). Saves and settings go to
# a sandbox under user://aegis_tour_<pid>, so the player's real autosave and settings are never touched.
#   godot --path <project> --resolution 1280x720 --audio-driver Dummy -s <abs path> -- out=<abs dir>
# Writes <out>/<nn>_<screen>.png and <out>/screens.json (id, title, file, notes).

var out_dir: String = ""
var sandbox: String = ""
var steps: Array[Dictionary] = []   # {id, title, run: Callable, wait: int}
var index: int = -1
var wait_left: int = 0
var shots: Array = []
var menu: Node = null

func _initialize() -> void:
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("out="):
			out_dir = arg.substr(4)
	DirAccess.make_dir_recursive_absolute(out_dir)
	sandbox = "user://aegis_tour_%d" % OS.get_process_id()
	DirAccess.make_dir_recursive_absolute(sandbox)
	SaveGame.set_base_dir(sandbox.path_join("saves"))
	SettingsFile.use_path(sandbox.path_join("settings.cfg"))
	var packed: PackedScene = load("res://scenes/main_menu.tscn")
	menu = packed.instantiate()
	root.add_child(menu)
	_plan()

func _router() -> MenuRouter:
	return menu.get("router") as MenuRouter

func _region() -> Node:
	return menu.get("game_view") as Node

func _add(id: String, title: String, run: Callable, wait: int = 12) -> void:
	steps.append({"id": id, "title": title, "run": run, "wait": wait})

func _plan() -> void:
	_add("main_menu", "Main menu", func() -> void: pass, 20)
	_add("new_game", "New game: world cards", func() -> void: _router().push(MenuRouter.NEW_GAME))
	_add("load_slots", "Load slots", func() -> void:
		_router().pop()
		_router().push(MenuRouter.LOAD))
	for t: int in range(4):
		var tab: int = t
		_add("settings_tab%d" % tab, "Settings tab %d" % tab, func() -> void:
			if _router().top() != MenuRouter.SETTINGS:
				_router().pop()
				_router().push(MenuRouter.SETTINGS)
			var model: Variant = _router().model()
			if model is SettingsModel:
				(model as SettingsModel).tab = tab
			menu.call("render"))
	_add("region_map", "Region map (exploration) after New game", func() -> void:
		_router().pop()
		_router().push(MenuRouter.NEW_GAME)
		var ng: Variant = _router().model()
		if ng is NewGameModel:
			(ng as NewGameModel).start(1), 40)
	_add("dialogue", "Dialogue panel (Hest the Quartermaster)", func() -> void:
		var r: Node = _region()
		if r != null:
			r.call("talk", "hest_the_quartermaster"), 20)
	_add("shop", "Shop panel (Hest's stores)", func() -> void:
		var r: Node = _region()
		if r != null:
			r.call("_on_shop_opened", str(WorldCatalog.resolve("ashlands", "shops", "hest_stores").get("_qid", "hest_stores"))), 20)
	_add("world_map", "World map (travel)", func() -> void:
		var r: Node = _region()
		if r != null:
			var shop: Variant = r.get("shop_panel")
			if shop != null and shop.has_method("close"):
				shop.call("close")
			var dlg: Variant = r.get("dialogue_panel")
			if dlg != null and dlg.has_method("leave"):
				dlg.call("leave")
			r.call("open_world_map"), 30)
	_add("hub", "Game menu (hub) over the map", func() -> void:
		var r: Node = _region()
		if r != null:
			r.call("close_world_map")
		_router().push(MenuRouter.HUB), 20)
	for tab: String in ["stats", "level_up", "skills", "gear"]:
		var t: String = tab
		_add("party_" + t, "Character screen, %s tab" % t, func() -> void:
			if _router().top() != MenuRouter.PARTY:
				_router().push(MenuRouter.PARTY, {"tab": t})
			else:
				var pm: Variant = _router().model()
				if pm is PartyModel:
					(pm as PartyModel).tab = t
				menu.call("render"))
	_add("journal", "Quest journal", func() -> void:
		_router().pop()
		_router().push(MenuRouter.JOURNAL))
	_add("save_slots", "Save slots (in play)", func() -> void:
		_router().pop()
		_router().push(MenuRouter.SAVE))
	_add("battle_results", "Battle results (sample award)", func() -> void:
		_router().pop()
		_router().pop()
		_router().push(MenuRouter.RESULTS, {"award": {}, "advanced": [], "outcome": "won"}), 20)
	_add("content_forge", "Content Forge (dev tool)", func() -> void: _swap("res://scenes/content_forge.tscn"), 30)
	_add("world_forge", "World Forge (dev tool)", func() -> void: _swap("res://scenes/world_forge.tscn"), 30)
	_add("gear_dojo", "Gear & Skill Dojo (dev tool)", func() -> void: _swap("res://scenes/gear_dojo.tscn"), 30)
	_add("scene_runner", "Scene Runner (dev tool)", func() -> void: _swap("res://scenes/scene_runner.tscn"), 30)

func _swap(path: String) -> void:
	for child: Node in root.get_children():
		child.queue_free()
	var packed: PackedScene = load(path)
	root.add_child(packed.instantiate())

func _process(_delta: float) -> bool:
	if wait_left > 0:
		wait_left -= 1
		return false
	if index >= 0 and index < steps.size():
		var done: Dictionary = steps[index]
		var file: String = "%02d_%s.png" % [index, str(done["id"])]
		var img: Image = root.get_texture().get_image()
		img.save_png(out_dir.path_join(file))
		shots.append({"id": done["id"], "title": done["title"], "file": file})
		print("§ shot %s" % file)
	index += 1
	if index >= steps.size():
		var f: FileAccess = FileAccess.open(out_dir.path_join("screens.json"), FileAccess.WRITE)
		f.store_string(JSON.stringify({"screens": shots}, "  "))
		f.close()
		SaveGame.reset_base_dir()
		return true
	var step: Dictionary = steps[index]
	var run: Callable = step["run"]
	run.call()
	wait_left = DictUtil.to_int(step["wait"])
	return false
