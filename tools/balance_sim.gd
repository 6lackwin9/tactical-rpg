extends SceneTree

# Balance simulator: headless AI-vs-AI battles on a sandbox copy of the catalog.
#   tools/fc.sh run tools/balance_sim.gd -- duel 5        class round-robin, hero vs hero at level 5
#   tools/fc.sh run tools/balance_sim.gd -- tier 5        each class: hero vs minion/standard/elite/boss of itself (1v1 and 1 vs 3)
#   tools/fc.sh run tools/balance_sim.gd -- levels        mirror matches: level N hero vs level N+2 hero, per class
# Score = share of battles the left unit wins (a timeout is decided by remaining HP fraction).
# Every matchup runs with sides swapped and several seeds, so side/position bias cancels out.
# Healthy targets: duels 35-65% per class (6 classes, each beats some and loses to some),
# a hero beats a same-level minion in most 1v1s and loses to a boss.

const CLASSES: Array[String] = ["warrior", "ranger", "shaman", "brawler", "scout", "berserker"]
const SEEDS: Array[int] = [11, 23, 37]
const SANDBOX: String = "user://balance_sim"

func _initialize() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	var mode: String = args[0] if args.size() > 0 else "duel"
	var level: int = int(args[1]) if args.size() > 1 else 5
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, SANDBOX))
	ContentCatalog.fork_to(SANDBOX)
	match mode:
		"duel":
			_duel(level)
		"tier":
			_tier(level)
		"levels":
			_levels()
		_:
			_out("modes: duel [level] | tier [level] | levels")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	quit()

func _unit(id: String, side: String, class_id: String, level: int, tier: String) -> void:
	var errors: Array[String] = ContentCatalog.upsert("units", {"id": id, "name": id, "side": side, "class_id": class_id, "level": level, "tier": tier, "flee_ratio": 0.0})
	if not errors.is_empty():
		_out("SIM unit error %s: %s" % [id, str(errors)])

# Plays one battle; returns the left (party) side's score 0..1.
func _fight(left: Array[String], right: Array[String], seed_value: int) -> float:
	var entry: Dictionary = {"id": "sim_fight", "name": "Sim", "zone": "stone_arena", "party": left, "enemies": right, "max_rounds": 6, "total_blocks": 6, "unit_ap": 12, "ap_regen": 4, "rng_seed": seed_value}
	var session := CombatSession.new(CatalogEncounter.from_entry(entry))
	session.start()
	for round_num: int in range(6):
		session.plan_apply_preset_all("balanced")
		if not session.confirm_plan():
			break
		for i: int in range(600):
			if session.phase == CombatSession.Phase.REPLAY:
				break
			session.step_next()
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var left_hp: float = 0.0
	var left_max: float = 0.0
	var right_hp: float = 0.0
	var right_max: float = 0.0
	for c: Combatant in session.allies:
		left_hp += float(c.hp if c.is_active() else 0)
		left_max += float(c.max_hp)
	for c: Combatant in session.enemies:
		right_hp += float(c.hp if c.is_active() else 0)
		right_max += float(c.max_hp)
	session.dispose()
	var l: float = left_hp / maxf(1.0, left_max)
	var r: float = right_hp / maxf(1.0, right_max)
	if r <= 0.0 and l > 0.0:
		return 1.0
	if l <= 0.0 and r > 0.0:
		return 0.0
	return 1.0 if l > r + 0.02 else (0.0 if r > l + 0.02 else 0.5)

# Average score of `a` vs `b` over seeds, sides swapped.
func _matchup(a_ids: Array[String], b_ids: Array[String]) -> float:
	var total: float = 0.0
	var n: int = 0
	for s: int in SEEDS:
		total += _fight(a_ids, b_ids, s)
		total += 1.0 - _fight(b_ids, a_ids, s)
		n += 2
	return total / float(n)

func _duel(level: int) -> void:
	for c: String in CLASSES:
		_unit("simp_%s" % c, "party", c, level, "hero")
		_unit("sime_%s" % c, "enemy", c, level, "hero")
	_out("DUEL level %d, hero vs hero, win rate of the ROW class" % level)
	var header: String = "%-10s" % ""
	for c: String in CLASSES:
		header += "%-10s" % c
	_out(header + "avg")
	for a: String in CLASSES:
		var line: String = "%-10s" % a
		var sum: float = 0.0
		var cnt: int = 0
		for b: String in CLASSES:
			if a == b:
				line += "%-10s" % "-"
				continue
			var p: Array[String] = ["simp_%s" % a]
			var e: Array[String] = ["sime_%s" % b]
			var rate: float = _matchup(p, e)
			sum += rate
			cnt += 1
			line += "%-10s" % ("%d%%" % roundi(rate * 100.0))
		_out(line + "%d%%" % roundi(sum / float(cnt) * 100.0))

func _tier(level: int) -> void:
	_out("TIER level %d: hero class vs its own class at a tier (win rate of the hero)" % level)
	for c: String in CLASSES:
		_unit("simp_%s" % c, "party", c, level, "hero")
		var line: String = "%-10s" % c
		for tier: String in ["minion", "standard", "elite", "boss"]:
			_unit("sime_%s_%s" % [c, tier], "enemy", c, level, tier)
			var one: Array[String] = ["sime_%s_%s" % [c, tier]]
			var three: Array[String] = ["sime_%s_%s" % [c, tier], "sime_%s_%s" % [c, tier], "sime_%s_%s" % [c, tier]]
			var p: Array[String] = ["simp_%s" % c]
			line += "%s 1v1 %d%% / 1v3 %d%%   " % [tier, roundi(_matchup(p, one) * 100.0), roundi(_matchup(p, three) * 100.0)]
		_out(line)

func _levels() -> void:
	_out("LEVELS: class at level N vs the same class at N+2 (win rate of the lower level)")
	for c: String in CLASSES:
		var line: String = "%-10s" % c
		for lvl: int in [1, 3, 5, 7]:
			_unit("simp_%s" % c, "party", c, lvl, "hero")
			_unit("sime_%s" % c, "enemy", c, lvl + 2, "hero")
			var p: Array[String] = ["simp_%s" % c]
			var e: Array[String] = ["sime_%s" % c]
			line += "L%d vs L%d: %d%%   " % [lvl, lvl + 2, roundi(_matchup(p, e) * 100.0)]
		_out(line)

func _out(text: String) -> void:
	print("§ " + text)
