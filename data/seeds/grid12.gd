extends SceneTree
# GRID12 asset seed (idempotent): encounter exit tiles move from the old 9x9 east edge (8, 4)
# to the 12x9 east edge (11, 4), in the catalog and in every world's own encounters.
# Run: tools/fc.sh run data/seeds/grid12.gd

const OLD_EXIT: Array = [8, 4]
const NEW_EXIT: Array = [11, 4]
const WORLDS: Array[String] = ["ashlands", "iron_reaches", "tidewrack"]

var failed: bool = false

func _moved(exits: Array) -> Array:
	var out: Array = []
	for x: Variant in exits:
		var pair: Array = x if x is Array else []
		out.append(NEW_EXIT.duplicate() if pair.size() == 2 and DictUtil.to_int(pair[0]) == OLD_EXIT[0] and DictUtil.to_int(pair[1]) == OLD_EXIT[1] else pair)
	return out

func _init() -> void:
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var entry: Dictionary = e.duplicate(true)
		entry["exits"] = _moved(DictUtil.array_of(e.get("exits", [])))
		var errs: Array[String] = ContentCatalog.upsert("encounters", entry)
		if not errs.is_empty():
			failed = true
			print("ERROR catalog ", e["id"], ": ", errs)
	if not failed:
		print("save encounters ", ContentCatalog.save("encounters"))
	for w: String in WORLDS:
		var touched: bool = false
		for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
			var entry: Dictionary = e.duplicate(true)
			entry["exits"] = _moved(DictUtil.array_of(e.get("exits", [])))
			var errs: Array[String] = WorldCatalog.upsert(w, "encounters", entry)
			if not errs.is_empty():
				failed = true
				print("ERROR ", w, " ", e["id"], ": ", errs)
			touched = true
		if touched and not failed:
			print("save_world ", w, " ", WorldCatalog.save_world(w))
	quit(1 if failed else 0)
