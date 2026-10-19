extends SceneTree
# ITEM1-A optional step: number-quoting item descriptions become {tokens} (description-only upserts).
# Kite and plate quoted pre-BAL2-A numbers; the tokens render the resolved values. Idempotent.
# Run: tools/fc.sh run data/seeds/item1_tokens.gd   (run after item1_migrate.gd)

const DESCRIPTIONS: Dictionary = {
	"wpn_claymore": "A heavy double-edged greatsword. Gains +{tier1.power} Power when wielded with {tier1.value}+ STR.",
	"wpn_warhammer": "Colossal warhammer requiring {req:STR} STR. Inflicts severe deficit penalty if under-statted.",
	"shd_buckler": "Light metal fist-shield providing {mitigation} mitigation without slowing down.",
	"shd_kite": "Sturdy teardrop shield granting +{mitigation} flat damage mitigation.",
	"arm_plate": "Interlocking steel plates providing {mitigation} mitigation with {burden} burden.",
}

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	for id: String in DESCRIPTIONS:
		var entry: Dictionary = ContentCatalog.get_entry("items", id).duplicate(true)
		entry["description"] = DESCRIPTIONS[id]
		var errs: Array = ContentCatalog.upsert("items", entry)
		if not errs.is_empty():
			print("ERROR %s: %s" % [id, str(errs)])
		else:
			print("%s -> %s" % [id, str(ItemText.render(str(entry["description"]), ContentCatalog.get_entry("items", id)))])
	ContentCatalog.end_system_edit()
	print("validate_all: ", ContentCatalog.validate_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
