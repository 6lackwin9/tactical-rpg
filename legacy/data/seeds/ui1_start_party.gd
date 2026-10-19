extends SceneTree
# UI1-A asset seed (idempotent): the Ashlands fixed starting party (tidewrack inherits it).
# Run: tools/fc.sh run data/seeds/ui1_start_party.gd

const PARTY: Array[String] = ["warrior", "ranger", "rogue", "mage"]

func _init() -> void:
	var errors: Array[String] = WorldCatalog.update_manifest("ashlands", {"start_party": PARTY})
	errors.append_array(WorldCatalog.manifest_errors("ashlands"))
	errors.append_array(WorldCatalog.manifest_errors("tidewrack"))
	if errors.is_empty():
		var err: Error = WorldCatalog.save_world("ashlands")
		print("save_world ", err)
		if err != OK:
			errors.append("save_world failed: %s" % error_string(err))
		print("start_party tidewrack ", WorldCatalog.start_party("tidewrack"), " playable ", NewGame.playable_worlds())
	if not errors.is_empty():
		print("ERROR ", errors)
	quit(1 if not errors.is_empty() else 0)
