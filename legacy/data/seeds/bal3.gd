extends SceneTree
# BAL3-A asset seed (idempotent): data levers of docs/specs/BAL3.md section 5. Each lever has a constant so the planner can flip it
# and re-run. Authoring only; every change goes through ContentCatalog.upsert. Run: tools/fc.sh run data/seeds/bal3.gd

const APPLY_BRAWLER_AOE: bool = true   # sweep_kick power_mult 0.7/0.9/1.05 -> 0.8/1.0/1.15
const APPLY_MAGE_AOE: bool = false      # firebomb reach_max 2 -> 3
const APPLY_MEND: bool = false          # mend heal_base 8/10/12 -> 10/12/14
const APPLY_RENEWAL_REGEN: bool = false # renewal grant haste -> regen
const RENEWAL_REGEN_MAGNITUDE: int = 3  # regen registry range 1..10 (% max HP per round)
const KITE_MITIGATION: int = 6          # shd_kite (BAL2 D1: 6); duel warrior lever lowers it
const APPLY_SMITE: bool = false         # smite power_mult +0.1 per tier

var errs: Array[String] = []

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	if APPLY_BRAWLER_AOE:
		_skill("sweep_kick", {"power_mult": [0.8, 1.0, 1.15]})
	if APPLY_MAGE_AOE:
		_skill("firebomb", {"reach_max": [3, 3, 3]})
	else:
		_skill("firebomb", {"reach_max": [2, 2, 2]})
	if APPLY_MEND:
		_skill("mend", {"heal_base": [10, 12, 14]})
	else:
		_skill("mend", {"heal_base": [8, 10, 12]})
	if APPLY_RENEWAL_REGEN:
		_skill("renewal", {"grant_status": ["regen", "regen", "regen"], "status_magnitude": [RENEWAL_REGEN_MAGNITUDE, RENEWAL_REGEN_MAGNITUDE, RENEWAL_REGEN_MAGNITUDE]},
			"{name}: heals {heal}; grants Regen ({magnitude} per round) for {rounds} starting next round. Within 3 tiles.")
	else:
		_skill("renewal", {"grant_status": ["haste", "haste", "haste"], "status_magnitude": [0, 0, 0]},
			"{name}: heals {heal}; grants haste for {rounds} starting next round. Within 3 tiles.")
	_edit("items", "shd_kite", {"mitigation": KITE_MITIGATION})
	if APPLY_SMITE:
		_skill("smite", {"power_mult": [0.9, 1.1, 1.3]})
	ContentCatalog.end_system_edit()
	for e: String in errs:
		print("ERROR: ", e)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()

func _edit(kind: String, id: String, changes: Dictionary) -> void:
	var e: Dictionary = ContentCatalog.get_entry(kind, id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING %s %s" % [kind, id])
		return
	for k: String in changes.keys():
		e[k] = changes[k]
	errs.append_array(ContentCatalog.upsert(kind, e))

# fields: field -> [rank 1, rank 2, rank 3] (rank modifiers are absolute values, SkillRanks); idempotent.
func _skill(id: String, fields: Dictionary, description: String = "") -> void:
	var e: Dictionary = ContentCatalog.skill_entry(id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING skills %s" % id)
		return
	var ranks: Array = e.get("ranks", []).duplicate(true)
	for field: String in fields.keys():
		var values: Array = fields[field]
		e[field] = values[0]
		for i: int in ranks.size():
			(ranks[i] as Dictionary)[field] = values[i + 1]
	e["ranks"] = ranks
	if not description.is_empty():
		e["description"] = description
	errs.append_array(ContentCatalog.upsert_skill(e))
