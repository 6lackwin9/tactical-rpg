extends SceneTree

# Writes a generated placeholder as a PNG to its folder-pattern path (S14-L1 rule 8), so a real file can start from it.
#   tools/fc.sh export_placeholder <ns> <key> <variant> [team]
# ns unit|item|tile|skill|status; team 0 friendly (default), 1 hostile, 2 neutral (items: the rarity).
# Unit tokens export the glyph layer only, cropped to the 104x104 inner square (the team rim is drawn by code).
# Prints "§ wrote <path> <w>x<h>". The manifest row is added separately (AssetCatalog.upsert in a seed).

func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() < 3:
		print("§ usage: export_placeholder <ns> <key> <variant> [team]")
		quit(2)
		return
	var team: int = int(args[3]) if args.size() > 3 else 0
	var result: Dictionary = AssetRegistry.export_placeholder(args[0], args[1], args[2], team)
	if not str(result["error"]).is_empty():
		print("§ %s" % str(result["error"]))
		quit(1)
		return
	var size: Vector2i = result["size"]
	print("§ wrote %s %dx%d" % [str(result["path"]), size.x, size.y])
	ContentCatalog.clear_cache()
	SkillCatalog.clear_cache()
	AssetCatalog.clear_cache()
	quit(0)
