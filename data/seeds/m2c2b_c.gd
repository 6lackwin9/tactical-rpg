extends SceneTree
# M2C2B phase C seed (idempotent): D1-A intimidate family and riposte tiers (answered reverse model: AP 3/2/1,
# mitigation 0.15/0.05/0.00, rounds 2/3/3), then the warrior path.
# Run: tools/fc.sh run data/seeds/m2c2b_c.gd
const Lib = preload("res://data/seeds/m2c2b_lib.gd")

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var errs: Array[String] = []
	errs.append_array(Lib.attack_family({"id": "intimidate", "name": "Intimidate", "ap": [1, 2, 3], "blocks": 1, "power": 0.0, "stance": "DEFENSIVE", "rmin": [1, 1, 1], "rmax": [2, 2, 3], "status": "weaken", "pot": [70, 75, 80], "mag": [15, 20, 25], "rounds": [2, 2, 3], "learned": true, "text": "Roar at a foe."}))
	errs.append_array(Lib.defend_family({"id": "riposte", "name": "Riposte", "ap": [3, 2, 1], "blocks": 2, "mit": [0.15, 0.05, 0.0], "rounds": [2, 3, 3], "grant": "riposte", "learned": true, "text": "Adjacent melee attackers you survive are countered for 50% of your attack power (once per attacker per round). Any stance change ends it; moving does not."}))
	errs.append_array(Lib.apply_paths())
	Lib.finish(errs, self)
