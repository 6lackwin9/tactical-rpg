extends SceneTree
# M2C2B phase A seed (idempotent): all Dep-A skill families (attack, advanced, non-attack) and the class paths.
# Run: tools/fc.sh run data/seeds/m2c2b_a.gd
const Lib = preload("res://data/seeds/m2c2b_lib.gd")

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var errs: Array[String] = []
	# 1. Attack families
	errs.append_array(Lib.attack_family({"id": "shield_bash", "name": "Shield Bash", "ap": [2, 3, 4], "blocks": 2, "power": [0.8, 1.0, 1.2], "stance": "DEFENSIVE", "status": "stagger", "pot": [70, 75, 80]}))
	errs.append_array(Lib.attack_family({"id": "sunder", "name": "Sunder", "ap": [2, 3, 4], "blocks": 2, "power": [1.2, 1.5, 1.8], "status": "stagger", "pot": [50, 55, 60], "wt": ["axe", "mace", "hammer"], "minp": 2, "text": "Armor-breaking chop."}))
	errs.append_array(Lib.attack_family({"id": "hamstring", "name": "Hamstring", "ap": [2, 3, 4], "blocks": 2, "power": [0.8, 1.0, 1.2], "status": "root", "pot": [80, 85, 90], "rounds": [1, 1, 2]}))
	errs.append_array(Lib.attack_family({"id": "aimed_shot", "name": "Aimed Shot", "ap": [2, 3, 4], "blocks": 2, "power": [1.4, 1.75, 2.1], "wt": ["bow", "crossbow"], "minp": 2, "text": "A patient shot."}))
	errs.append_array(Lib.attack_family({"id": "pinning_shot", "name": "Pinning Shot", "ap": [2, 3, 4], "blocks": 2, "power": [0.8, 1.0, 1.2], "status": "root", "pot": [80, 85, 90], "rounds": [1, 1, 2], "wt": ["bow", "crossbow"], "minp": 1}))
	errs.append_array(Lib.attack_family({"id": "barbed_arrow", "name": "Barbed Arrow", "ap": [2, 3, 4], "blocks": 2, "power": [0.7, 0.9, 1.05], "status": "bleed", "pot": [60, 65, 70], "rounds": 2, "wt": ["bow", "crossbow"], "minp": 2}))
	errs.append_array(Lib.attack_family({"id": "volley", "name": "Volley", "ap": [2, 3, 4], "blocks": 2, "power": [0.7, 0.9, 1.05], "area": "PLUS", "rmin": 2, "rmax": 3, "wt": ["bow", "crossbow"], "minp": 1}))
	errs.append_array(Lib.attack_family({"id": "lunge", "name": "Lunge", "ap": [2, 3, 4], "blocks": 2, "power": [1.2, 1.5, 1.8], "rmin": 1, "rmax": 1, "dash": [3, 3, 4], "text": "Gap-close: the dash is your move for the round."}))
	errs.append_array(Lib.attack_family({"id": "sweep_kick", "name": "Sweep Kick", "ap": [2, 3, 4], "blocks": 2, "power": [0.7, 0.9, 1.05], "area": "SWEEP", "text": "Aim at a diagonal neighbour to pick the quadrant. Bare hands or feet."}))
	errs.append_array(Lib.attack_family({"id": "body_blow", "name": "Body Blow", "ap": [2, 3, 4], "blocks": 2, "power": [1.0, 1.25, 1.5], "status": "stagger", "pot": [50, 55, 60], "wt": ["fist", "mace", "unarmed"], "minp": 1}))
	errs.append_array(Lib.attack_family({"id": "haymaker", "name": "Haymaker", "ap": [3, 4, 5], "blocks": 3, "power": [2.0, 2.5, 3.0], "status": "stun", "pot": [50, 55, 60], "wt": ["fist", "mace", "hammer", "unarmed"], "minp": 2}))
	errs.append_array(Lib.attack_family({"id": "pounce", "name": "Pounce", "ap": [2, 3, 4], "blocks": 2, "power": [0.8, 1.0, 1.2], "rmin": 1, "rmax": 1, "dash": [3, 3, 4], "wt": ["dagger"], "minp": 1, "text": "Gap-close: the dash is your move for the round."}))
	errs.append_array(Lib.attack_family({"id": "quick_stab", "name": "Quick Stab", "ap": [1, 2, 3], "blocks": 1, "power": [0.6, 0.75, 0.9], "wt": ["dagger"], "minp": 1, "text": "A fast thrust."}))
	errs.append_array(Lib.attack_family({"id": "bleeding_cut", "name": "Bleeding Cut", "ap": [2, 3, 4], "blocks": 2, "power": [0.8, 1.0, 1.2], "status": "bleed", "pot": [70, 75, 80], "rounds": 2}))
	errs.append_array(Lib.attack_family({"id": "smite", "name": "Smite", "ap": [1, 2, 3], "blocks": 2, "power": [0.8, 1.0, 1.2], "rmin": 2, "rmax": 3, "wt": ["mace", "staff"], "minp": 1, "text": "Holy force from a distance."}))
	# 2. Advanced attack families (Dep A)
	errs.append_array(Lib.attack_family({"id": "deadeye", "name": "Deadeye", "ap": [3, 4, 5], "blocks": 3, "power": [2.0, 2.5, 3.0], "status": "stagger", "pot": [50, 55, 60], "wt": ["bow", "crossbow"], "minp": 3}))
	errs.append_array(Lib.attack_family({"id": "judgement", "name": "Judgement", "ap": [3, 4, 5], "blocks": 3, "power": [1.2, 1.5, 1.8], "rmin": 2, "rmax": 3, "status": "stagger", "pot": [40, 45, 50], "wt": ["mace", "staff"], "minp": 2}))
	errs.append_array(Lib.attack_family({"id": "charge_in", "name": "Charge In", "ap": [3, 4, 5], "blocks": 3, "power": [1.4, 1.75, 2.1], "rmin": 1, "rmax": 1, "dash": 4, "status": "stagger", "pot": [60, 65, 70], "text": "Gap-close: the dash is your move for the round."}))
	errs.append_array(Lib.attack_family({"id": "lacerate", "name": "Lacerate", "ap": [3, 4, 5], "blocks": 2, "power": [1.2, 1.5, 1.8], "status": "bleed", "pot": [90, 95, 100], "rounds": 2, "wt": ["dagger"], "minp": 2}))
	errs.append_array(Lib.attack_family({"id": "assassinate", "name": "Assassinate", "ap": [3, 4, 5], "blocks": 3, "power": [2.0, 2.5, 3.0], "wt": ["dagger"], "minp": 3}))
	# 3. Non-attack families
	errs.append_array(Lib.support_family({"id": "taunt_shout", "name": "Taunt Shout", "ap": [1, 2, 3], "blocks": 1, "rounds": [2, 3, 3], "target": "SELF", "grant": "taunt", "text": "Foes that can reach you must target you."}))
	errs.append_array(Lib.focus_family())
	errs.append_array(Lib.support_family({"id": "mend", "name": "Mend", "ap": [2, 3, 4], "blocks": 2, "target": "ALLY_OR_SELF", "heal_stat": "KNOW", "heal_base": [8, 10, 12], "heal_mult": [1.5, 1.8, 2.1], "text": "A friendly unit within 3 tiles, or yourself."}))
	errs.append_array(Lib.support_family({"id": "purify", "name": "Purify", "ap": [2, 3, 4], "blocks": 1, "target": "ALLY_OR_SELF", "cleanse": [["stun"], ["stun", "bleed"], ["all"]], "text": "Active or pending debuffs on a friendly unit within 3 tiles, or yourself. Buffs stay."}))
	errs.append_array(Lib.support_family({"id": "rally", "name": "Rally", "ap": [1, 2, 3], "blocks": 1, "rounds": [1, 2, 2], "target": "ALLY", "rmin": [0, 0, 1], "rmax": [0, 0, 4], "grant": "haste", "text": "+1 tile per move block."}))
	errs.append_array(Lib.support_family({"id": "greater_mend", "name": "Greater Mend", "ap": [3, 4, 5], "blocks": 3, "rounds": 1, "target": "ALLY_OR_SELF", "heal_stat": "KNOW", "heal_base": [10, 14, 18], "heal_mult": [1.8, 2.0, 2.2], "cleanse": [["stun"], ["all"], ["all"]], "grant": ["", "", "haste"], "text": "Within 3 tiles, or yourself."}))
	errs.append_array(Lib.support_family({"id": "renewal", "name": "Renewal", "ap": [3, 4, 5], "blocks": 2, "rounds": [1, 1, 2], "target": "ALLY", "heal_stat": "KNOW", "heal_base": [6, 8, 10], "heal_mult": [1.0, 1.2, 1.4], "grant": "haste", "text": "Within 3 tiles."}))
	errs.append_array(Lib.disengage_family("leap_back", "Leap Back", [2, 3, 3]))
	errs.append_array(Lib.disengage_family("backflip", "Backflip", [3, 3, 3]))
	# 5. Class paths (families present so far)
	errs.append_array(Lib.apply_paths())
	Lib.finish(errs, self)
