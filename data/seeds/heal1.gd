extends SceneTree
# HEAL1-A seed (idempotent): `rest: true` on ashlands warden_keep and iron_reaches slag_gate.
# Run: tools/fc.sh run data/seeds/heal1.gd. Spec: docs/specs/HEAL1.md section 6.

var failed: bool = false

func _init() -> void:
	ContentCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	var rows: Array[Array] = [["ashlands", "warden_keep"], ["iron_reaches", "slag_gate"]]
	for row: Array in rows:
		var world: String = str(row[0])
		var loc: Dictionary = WorldCatalog.get_own(world, "locations", str(row[1]))
		loc["rest"] = true
		var errs: Array[String] = WorldCatalog.upsert(world, "locations", loc)
		if not errs.is_empty():
			failed = true
			print("ERROR ", row, ": ", errs)
		elif WorldCatalog.save_world(world) != OK:
			failed = true
			print("ERROR save ", world)
	print("heal1 seed ", "FAILED" if failed else "ok")
	quit(1 if failed else 0)
