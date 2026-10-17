extends SceneTree

# Content Forge screenshot helper (FORGE-VIEW): opens the Content Forge on a tab and entry, optionally picks a preview
# rank, waits a few frames and saves the window as PNG. Not headless (needs a renderer):
#   xvfb-run -a -s "-screen 0 1920x1080x24" godot --rendering-driver opengl3 --path . --resolution 1600x900 \
#     -s tools/forge_shot.gd -- --kind=skills --select=aegis --rank=3 --out=/tmp/forge.png
#   --kind=<ContentSchema kind> (default skills)   --select=<entry id> (default: the first listed)
#   --rank=<n> (skills: the preview rank, default 1)   --out=<png path> (required)   --frames=<n> (default 8)
# Exit 0 when the PNG was written, 1 otherwise.

func _init() -> void:
	var opts: Dictionary = {"kind": "skills", "select": "", "rank": "1", "out": "", "frames": "8"}
	for a: String in OS.get_cmdline_user_args():
		if a.begins_with("--") and a.contains("="):
			opts[a.substr(2, a.find("=") - 2)] = a.substr(a.find("=") + 1)
	if str(opts["out"]).is_empty():
		print("usage: forge_shot -- --kind=<kind> --select=<id> [--rank=<n>] --out=<file.png>")
		quit(1)
		return
	_shoot.call_deferred(opts)

func _shoot(opts: Dictionary) -> void:
	var host: Node = (load("res://scenes/content_forge.tscn") as PackedScene).instantiate()
	root.add_child(host)
	await process_frame
	var forge: ContentForge = host.get("forge")
	forge.show_kind(str(opts["kind"]))
	if not str(opts["select"]).is_empty():
		forge.select(str(opts["select"]))
	if forge.has_method("set_preview_rank"):
		forge.call("set_preview_rank", str(opts["rank"]).to_int())
	for i: int in range(maxi(2, str(opts["frames"]).to_int())):
		await process_frame
	var image: Image = root.get_texture().get_image()
	var file: String = str(opts["out"])
	var ok: bool = image != null and not image.is_empty() and image.save_png(file) == OK
	print("forge_shot %s: %s" % ["wrote" if ok else "FAILED", file])
	quit(0 if ok else 1)
