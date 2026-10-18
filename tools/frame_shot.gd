extends SceneTree

# ScreenFrame screenshot helper (UIR-1): renders a sample framed screen at a text size and saves the window as PNG.
#   xvfb-run -a -s "-screen 0 1280x720x24" godot --rendering-driver opengl3 --path . --resolution 1280x720 \
#     -s tools/frame_shot.gd -- --scale=1.0 --out=/tmp/frame.png
# Exit 0 when the PNG was written, 1 otherwise.

func _init() -> void:
	var opts: Dictionary = {"scale": "1.0", "out": ""}
	for a: String in OS.get_cmdline_user_args():
		if a.begins_with("--") and a.contains("="):
			opts[a.substr(2, a.find("=") - 2)] = a.substr(a.find("=") + 1)
	if str(opts["out"]).is_empty():
		print("usage: frame_shot -- --scale=<text scale> --out=<file.png>")
		quit(1)
		return
	_shoot.call_deferred(opts)

func _shoot(opts: Dictionary) -> void:
	var host := Control.new()
	host.set_anchors_preset(Control.PRESET_FULL_RECT)
	var th: Theme = MenuStyle.theme()
	th.default_font_size = roundi(MainMenuView.BASE_FONT * str(opts["scale"]).to_float())
	host.theme = th
	root.add_child(host)
	var bg := ColorRect.new()
	bg.color = MainMenuView.COL_BG
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	host.add_child(bg)
	var noop: Callable = func() -> void: pass
	var frame: ScreenFrame = ScreenFrame.make("Sample screen", MainMenuView.button("Back", noop), MainMenuView.button("Confirm", noop), [MainMenuView.button("Reset", noop)] as Array[Button])
	host.add_child(frame)
	for i: int in 24:
		var b: Button = MainMenuView.button("Row %d" % (i + 1), noop)
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		if i == 1:
			MenuStyle.mark_selected(b, true)
		frame.column.add_child(b)
	for i: int in 6:
		await process_frame
	var image: Image = root.get_texture().get_image()
	var ok: bool = image != null and not image.is_empty() and image.save_png(str(opts["out"])) == OK
	print("frame_shot %s: %s" % ["wrote" if ok else "FAILED", str(opts["out"])])
	quit(0 if ok else 1)
