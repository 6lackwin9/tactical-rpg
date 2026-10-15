extends RefCounted
# Shared helpers for the M2C2B seed scripts (data/seeds/m2c2b_*.gd). Authoring only; no game logic.

const ROMAN: Array[String] = ["", " II", " III"]

static func _at(v: Variant, i: int) -> Variant:
	if v is Array:
		var a: Array = v
		return a[i] if a.size() > 1 else a[0]
	return v

static func tier_id(fid: String, i: int) -> String:
	return fid if i == 0 else "%s_%d" % [fid, i + 1]

static func _pct(x: float) -> String:
	return str(int(round(x * 100.0)))

static func _learned(fid: String, spec: Dictionary) -> bool:
	if spec.has("learned"):
		return bool(spec["learned"])
	var old: Dictionary = ContentCatalog.get_entry("skills", fid)
	if not old.is_empty():
		return bool(old.get("learned_only", false))
	return true

## spec keys: id, name, ap[], blocks, power[], area, rmin[], rmax[], dash[], status, pot[], rounds[], mag[],
## wt[], minp, stance, text (flavour), learned (optional)
static func attack_family(spec: Dictionary) -> Array[String]:
	var errs: Array[String] = []
	var fid: String = spec["id"]
	for i: int in 3:
		var power: float = float(_at(spec["power"], i))
		var ap: int = int(_at(spec["ap"], i))
		var status: String = spec.get("status", "")
		var wt: Array = spec.get("wt", [])
		var e: Dictionary = {
			"id": tier_id(fid, i),
			"name": String(spec["name"]) + ROMAN[i],
			"kind": "ATTACK",
			"level": ap,
			"blocks": int(spec["blocks"]),
			"power_mult": power,
			"area": spec.get("area", "NONE"),
			"offhand": false,
			"stance": spec.get("stance", "AUTO"),
			"status_effect": status,
			"dash_tiles": int(_at(spec.get("dash", 0), i)),
			"status_rounds": int(_at(spec.get("rounds", 1), i)),
			"reach_min": int(_at(spec.get("rmin", 0), i)),
			"reach_max": int(_at(spec.get("rmax", 0), i)),
			"family": "" if i == 0 else fid,
			"tier": i + 1,
			"basic": false,
			"learned_only": _learned(fid, spec),
			"weapon_types": wt,
			"min_proficiency": int(spec.get("minp", 0)),
		}
		var d: String = "%s%s: %s%% power" % [spec["name"], ROMAN[i], _pct(power)] if power > 0.0 else "%s%s: no damage" % [spec["name"], ROMAN[i]]
		if status != "":
			e["status_potency"] = int(_at(spec["pot"], i))
			d += ", %s%% to %s" % [e["status_potency"], status.capitalize()]
			if spec.has("mag"):
				e["status_magnitude"] = int(_at(spec["mag"], i))
				d += " (-%s%% outgoing power)" % e["status_magnitude"]
			d += " for %s round%s" % [e["status_rounds"], "" if int(e["status_rounds"]) == 1 else "s"]
			if String(e["area"]) != "NONE":
				d += " on every enemy hit"
		d += "."
		if int(e["dash_tiles"]) > 0:
			d += " Dashes up to %s tiles." % e["dash_tiles"]
		if String(e["area"]) != "NONE":
			d += " %s area, hits allies too." % String(e["area"]).capitalize()
		if wt.size() > 0:
			d += " Needs %s training %s." % ["/".join(PackedStringArray(wt)), e["min_proficiency"]]
		if spec.has("text"):
			d += " " + String(spec["text"])
		e["description"] = d
		errs.append_array(ContentCatalog.upsert("skills", e))
	return errs

## spec keys: id, name, ap[], blocks[]|int, rounds[], target, rmin[], rmax[], heal_base[], heal_mult[], cleanse[][], grant,
## text
static func support_family(spec: Dictionary) -> Array[String]:
	var errs: Array[String] = []
	var fid: String = spec["id"]
	for i: int in 3:
		var e: Dictionary = {
			"id": tier_id(fid, i),
			"name": String(spec["name"]) + ROMAN[i],
			"kind": "SUPPORT",
			"level": int(_at(spec["ap"], i)),
			"blocks": int(_at(spec["blocks"], i)),
			"status_rounds": int(_at(spec.get("rounds", 1), i)),
			"reach_min": int(_at(spec.get("rmin", 0), i)),
			"reach_max": int(_at(spec.get("rmax", 0), i)),
			"target_mode": spec["target"],
			"heal_stat": spec.get("heal_stat", ""),
			"heal_base": int(_at(spec.get("heal_base", 0), i)),
			"heal_mult": float(_at(spec.get("heal_mult", 0.0), i)),
			"grant_status": _grant(spec, i),
			"cleanse_statuses": _cleanse(spec, i),
			"family": "" if i == 0 else fid,
			"tier": i + 1,
			"basic": false,
			"learned_only": _learned(fid, spec),
		}
		var parts: Array[String] = []
		if float(e["heal_mult"]) > 0.0 or int(e["heal_base"]) > 0:
			parts.append("heals %s + %s x KNOW" % [e["heal_base"], e["heal_mult"]])
		if String(e["grant_status"]) != "":
			parts.append("grants %s for %s round%s starting next round" % [e["grant_status"], e["status_rounds"], "" if int(e["status_rounds"]) == 1 else "s"])
		var cl: Array = e["cleanse_statuses"]
		if cl.size() > 0:
			parts.append("cleanses " + ", ".join(PackedStringArray(cl)))
		e["description"] = "%s%s: %s. %s" % [spec["name"], ROMAN[i], "; ".join(PackedStringArray(parts)), String(spec.get("text", ""))]
		errs.append_array(ContentCatalog.upsert("skills", e))
	return errs

static func _grant(spec: Dictionary, i: int) -> String:
	var g: Variant = spec.get("grant", "")
	if g is Array:
		var a: Array = g
		return String(a[i])
	return String(g)

static func _cleanse(spec: Dictionary, i: int) -> Array:
	var c: Array = spec.get("cleanse", [[], [], []])
	return c[i]

static func defend_family(spec: Dictionary) -> Array[String]:
	var errs: Array[String] = []
	var fid: String = spec["id"]
	for i: int in 3:
		var mit: float = float(_at(spec["mit"], i))
		var e: Dictionary = {
			"id": tier_id(fid, i),
			"name": String(spec["name"]) + ROMAN[i],
			"kind": "DEFEND",
			"level": int(_at(spec["ap"], i)),
			"blocks": int(spec["blocks"]),
			"status_rounds": int(_at(spec["rounds"], i)),
			"grant_status": spec["grant"],
			"mitigation": mit,
			"family": "" if i == 0 else fid,
			"tier": i + 1,
			"basic": false,
			"learned_only": _learned(fid, spec),
		}
		e["description"] = "%s%s: -%s%% damage; grants %s for %s round%s. %s" % [spec["name"], ROMAN[i], _pct(mit), spec["grant"], e["status_rounds"], "" if int(e["status_rounds"]) == 1 else "s", String(spec.get("text", ""))]
		errs.append_array(ContentCatalog.upsert("skills", e))
	return errs

static func focus_family() -> Array[String]:
	var errs: Array[String] = []
	var rounds: Array[int] = [2, 3, 3]
	var blocks: Array[int] = [2, 2, 1]
	for i: int in 3:
		var e: Dictionary = {
			"id": tier_id("focus", i), "name": "Focus" + ROMAN[i], "kind": "FOCUS", "level": 2 + i, "blocks": blocks[i],
			"description": "Focus%s: +50%% attack for %s rounds." % [ROMAN[i], rounds[i]],
			"family": "" if i == 0 else "focus", "tier": i + 1, "basic": false, "learned_only": _learned("focus", {}),
			"boost": 1.5, "boost_rounds": rounds[i],
		}
		errs.append_array(ContentCatalog.upsert("skills", e))
	return errs

static func disengage_family(fid: String, nm: String, jumps: Array[int]) -> Array[String]:
	var errs: Array[String] = []
	for i: int in 3:
		var e: Dictionary = {
			"id": tier_id(fid, i), "name": nm + ROMAN[i], "kind": "DISENGAGE", "level": 2 + i, "blocks": 1,
			"description": "%s%s: jump up to %s tiles away from the nearest enemy (or a chosen one) to break melee contact. Your move for the round: it comes first, leaps over units but not walls, and a rooted unit can't." % [nm, ROMAN[i], jumps[i]],
			"jump_tiles": jumps[i], "family": "" if i == 0 else fid, "tier": i + 1, "basic": false, "learned_only": _learned(fid, {}),
		}
		errs.append_array(ContentCatalog.upsert("skills", e))
	return errs

# ---- class paths ----

const PATHS: Dictionary = {
	"warrior": [[1, "taunt_shout"], [3, "shield_bash"], [4, "intimidate"], [5, "riposte"], [5, "sunder"], [5, "shield_wall"]],
	"ranger": [[3, "aimed_shot"], [3, "pinning_shot"], [3, "leap_back"], [5, "barbed_arrow"], [5, "volley"], [5, "deadeye"], [5, "focus"]],
	"priest": [[1, "smite"], [1, "rally"], [3, "mend"], [3, "purify"], [5, "hex"], [5, "greater_mend"], [5, "renewal"], [7, "judgement"]],
	"brawler": [[3, "lunge"], [3, "body_blow"], [3, "sweep_kick"], [4, "hamstring"], [5, "haymaker"], [5, "charge_in"], [5, "cyclone_kick"]],
	"rogue": [[1, "quick_stab"], [3, "pounce"], [3, "bleeding_cut"], [4, "backflip"], [5, "hamstring"], [5, "lacerate"], [7, "assassinate"]],
	"mage": [[1, "arcane_bolt"], [3, "arc_lance"], [3, "hex"], [4, "focus"], [5, "firebomb"], [5, "glacial_burst"], [7, "inferno"]],
}

## Rebuilds each class's non-basic path rows from PATHS, keeping only families whose T1 skill exists. Basics stay as shipped.
static func apply_paths() -> Array[String]:
	var errs: Array[String] = []
	for cid: String in PATHS.keys():
		var c: Dictionary = ContentCatalog.get_entry("classes", cid).duplicate(true)
		var path: Array = []
		for row: Variant in c["skill_path"]:
			var rd: Dictionary = row
			var sk: Dictionary = ContentCatalog.get_entry("skills", String(rd["skill"]))
			if bool(sk.get("basic", false)):
				path.append(rd)
		for pr: Variant in PATHS[cid]:
			var pa: Array = pr
			if not ContentCatalog.get_entry("skills", String(pa[1])).is_empty():
				path.append({"level": pa[0], "skill": pa[1]})
		c["skill_path"] = path
		errs.append_array(ContentCatalog.upsert("classes", c))
	return errs

static func finish(errs: Array[String], tree: SceneTree) -> void:
	print("upsert errors: ", errs)
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	tree.quit()
