extends SceneTree
# BAL5-A asset seed (idempotent): docs/specs/BAL5_LEVERS.md section 2 (T3) and DESIGN_REVIEW3 section 3. Run: tools/fc.sh run data/seeds/bal5.gd

const APPLY_SHIELD_WALL_FORTIFY: bool = true   # shield_wall extra_grants fortify 4 / 6 / 8
const APPLY_POTION_PRICE: bool = true          # Hest potion shop row price -1 (15) -> 30
const POTION_PRICE: int = 30

var errs: Array[String] = []

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	if APPLY_SHIELD_WALL_FORTIFY:
		_shield_wall()
	ContentCatalog.end_system_edit()
	if APPLY_POTION_PRICE:
		_potion_price()
	for e: String in errs:
		print("ERROR: ", e)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	print("world save_all: ", WorldCatalog.save_all())
	quit()

func _shield_wall() -> void:
	var e: Dictionary = ContentCatalog.skill_entry("shield_wall").duplicate(true)
	if e.is_empty():
		errs.append("MISSING skills shield_wall")
		return
	var mags: Array[int] = [4, 6, 8]
	var rounds: Array[int] = [1, 2, 3]
	e["extra_grants"] = [{"status": "fortify", "rounds": rounds[0], "magnitude": mags[0]}]
	var ranks: Array = e.get("ranks", []).duplicate(true)
	for i: int in ranks.size():
		(ranks[i] as Dictionary)["extra_grants"] = [{"status": "fortify", "rounds": rounds[i + 1], "magnitude": mags[i + 1]}]
	e["ranks"] = ranks
	e["description"] = "{name}: -{mitigation} damage; grants taunt for {rounds} and {extra}. Foes that can reach you must target you, starting next round."
	errs.append_array(ContentCatalog.upsert_skill(e))

func _potion_price() -> void:
	var shop: Dictionary = WorldCatalog.get_own("ashlands", "shops", "hest_stores").duplicate(true)
	if shop.is_empty():
		errs.append("MISSING shop hest_stores")
		return
	for row: Variant in shop.get("stock", []):
		var r: Dictionary = row
		if r.get("item", "") == "con_heal_potion":
			r["price"] = POTION_PRICE
	errs.append_array(WorldCatalog.upsert("ashlands", "shops", shop))
