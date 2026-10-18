extends RefCounted

# Balance sim `campaign` mode (BAL4-ATTR, docs/specs/DESIGN_REVIEW1.md 3.3): the real party of three
# (warrior + two recruits of the other five classes: all 10 trios, or comp=) plays the Ashlands story
# fights in quest order (tools/sim/sim_world.gd) for both steel choices. HP carries over from battle
# to battle; statuses reset (every fight is a new session). A hero at 0 stays down and sits out the
# later fights; the run ends at the first battle that is not cleared (a loss or the round limit, which
# is the encounter's authored max_rounds). Between battles `heal` applies: none (default), full
# (everyone, a downed hero revives: the M3 baseline) or a percent of max HP for heroes still up.
# Spend / preset are the defaults (class, balanced). Loaded with preload (no class_name).

const SimArgs = preload("res://tools/sim/sim_args.gd")
const SimRunner = preload("res://tools/sim/sim_runner.gd")
const SimScenarios = preload("res://tools/sim/sim_scenarios.gd")
const SimWorld = preload("res://tools/sim/sim_world.gd")

const STEELS: Array[String] = ["arms", "gate"]   # arms = ashmaw_stand (full finale), gate = ashmaw_stand_barred
const SEED_STEP: int = 101                         # battle i of a run uses seed + i * SEED_STEP

var args: SimArgs
var runner: SimRunner

func _init(p_args: SimArgs, p_runner: SimRunner) -> void:
	args = p_args
	runner = p_runner

# The 10 trios: the warrior plus every pair of the five other classes (class list order).
static func trios() -> Array:
	var others: Array[String] = []
	for c: String in SimArgs.CLASSES:
		if c != "warrior":
			others.append(c)
	var out: Array = []
	for i: int in range(others.size()):
		for j: int in range(i + 1, others.size()):
			out.append(["warrior", others[i], others[j]] as Array[String])
	return out

# Hero HP after the between-battle heal. `state`: unit id -> {"hp", "max"}; `mode` none | full | "<pct>".
static func apply_heal(state: Dictionary, mode: String) -> void:
	for id: Variant in state:
		var s: Dictionary = DictUtil.dict_of(state[id])
		var hp: int = DictUtil.to_int(s["hp"])
		var max_hp: int = DictUtil.to_int(s["max"])
		if mode == "full":
			hp = max_hp
		elif mode != "none" and hp > 0:
			hp = mini(max_hp, hp + ceili(float(max_hp) * float(int(mode)) / 100.0))
		s["hp"] = hp
		state[id] = s

static func hp_pct(state: Dictionary) -> float:
	var hp: float = 0.0
	var max_hp: float = 0.0
	for id: Variant in state:
		var s: Dictionary = DictUtil.dict_of(state[id])
		hp += float(DictUtil.to_int(s["hp"]))
		max_hp += float(DictUtil.to_int(s["max"]))
	return hp / maxf(1.0, max_hp) * 100.0

# One run of party `ids` through `fights` ([{"enc": encounter dict, "label"}]) with seed `seed_value`:
# {"wins", "fell_at" (1-based battle, 0 = none), "hp": [party HP % after each fought battle, before the heal],
#  "alive": [heroes up after each battle]}.
func run_party(ids: Array[String], fights: Array, seed_value: int) -> Dictionary:
	var state: Dictionary = {}
	var wins: int = 0
	var fell_at: int = 0
	var hp_after: Array[float] = []
	var alive_after: Array[int] = []
	for i: int in range(fights.size()):
		var enc: Dictionary = DictUtil.dict_of(fights[i])
		var up: Array[String] = []
		runner.start_hp = {}
		for id: String in ids:
			if not state.has(id) or DictUtil.to_int(DictUtil.dict_of(state[id])["hp"]) > 0:
				up.append(id)
				if state.has(id):
					runner.start_hp[id] = DictUtil.to_int(DictUtil.dict_of(state[id])["hp"])
		var foes: Array[String] = []
		foes.assign(DictUtil.array_of(enc["enemies"]))
		runner.rounds_override = DictUtil.to_int(enc.get("max_rounds", 0))
		var res: Dictionary = runner.fight(up, foes, seed_value + i * SEED_STEP, DictUtil.dict_of(enc["base"]))
		runner.start_hp = {}
		runner.rounds_override = 0
		if not DictUtil.to_bool(res.get("ok", false)):
			fell_at = i + 1
			break
		var end_hp: Dictionary = DictUtil.dict_of(res["party_hp"])
		var end_max: Dictionary = DictUtil.dict_of(res["party_max"])
		for id: String in up:
			state[id] = {"hp": DictUtil.to_int(end_hp.get(id, 0)), "max": DictUtil.to_int(end_max.get(id, 1))}
		for id: String in ids:   # fix the max of heroes that never fought yet (all fight battle 1, so none)
			if not state.has(id):
				state[id] = {"hp": 0, "max": 1}
		hp_after.append(hp_pct(state))
		var n_alive: int = 0
		for id: Variant in state:
			if DictUtil.to_int(DictUtil.dict_of(state[id])["hp"]) > 0:
				n_alive += 1
		alive_after.append(n_alive)
		if not DictUtil.to_bool(res.get("won", res["cleared"])) or n_alive == 0:
			fell_at = i + 1
			break
		wins += 1
		apply_heal(state, args.heal)
	return {"wins": wins, "fell_at": fell_at, "hp": hp_after, "alive": alive_after}

func fights_for(steel: String) -> Array:
	var out: Array = []
	var foe_level: int = clampi(args.level + args.elevel, 1, StatRules.MAX_LEVEL)
	for f: Dictionary in SimWorld.story_fights(steel == "gate"):
		var enc: Dictionary = SimWorld.encounter(runner, str(f["encounter"]), foe_level, args.elevel_set)
		if not enc.is_empty():
			out.append(enc)
	return out

func campaign_mode() -> void:
	var parties: Array = [args.comp] if args.comp_set else trios()
	var story: Array[String] = []
	for steel: String in STEELS:
		var labels: Array[String] = []
		for f: Dictionary in SimWorld.story_fights(steel == "gate"):
			labels.append(str(f["encounter"]))
		story.append("%s: %s" % [steel, " > ".join(labels)])
	out("CAMPAIGN party level %d (foes at authored levels%s), heal %s, %d seeds, %d parties; story order %s" % [
		args.level, (" %+d" % args.elevel) if args.elevel_set else "", args.heal, args.seed_count, parties.size(), " | ".join(story)])
	for raw: Variant in parties:
		var comp: Array[String] = []
		comp.assign(DictUtil.array_of(raw))
		var ids: Array[String] = SimScenarios.define_party(runner, comp, args.level)
		var overall_wins: float = 0.0
		var overall_total: float = 0.0
		for steel: String in STEELS:
			var fights: Array = fights_for(steel)
			var total: int = fights.size()
			var wins_sum: float = 0.0
			var clears: int = 0
			var fell: Dictionary = {}
			var hp_sum: Array[float] = []
			var hp_n: Array[int] = []
			for s: int in args.seeds():
				var r: Dictionary = run_party(ids, fights, s)
				var hp_list: Array = DictUtil.array_of(r["hp"])
				var hp_text: Array[String] = []
				for k: int in range(hp_list.size()):
					hp_text.append("%d" % roundi(DictUtil.to_float(hp_list[k])))
					while hp_sum.size() <= k:
						hp_sum.append(0.0)
						hp_n.append(0)
					hp_sum[k] += DictUtil.to_float(hp_list[k])
					hp_n[k] += 1
				var fell_at: int = DictUtil.to_int(r["fell_at"])
				wins_sum += float(DictUtil.to_int(r["wins"]))
				if fell_at == 0:
					clears += 1
				else:
					fell[fell_at] = DictUtil.to_int(fell.get(fell_at, 0)) + 1
				out("  %s steel %s seed %d: survived %d/%d, %s, party hp%% after each battle [%s]" % [",".join(comp), steel, s, DictUtil.to_int(r["wins"]), total,
					"cleared the campaign" if fell_at == 0 else "fell at battle %d (%s)" % [fell_at, str(DictUtil.dict_of(fights[fell_at - 1])["label"])], ", ".join(hp_text)])
			var avg_hp: Array[String] = []
			for k: int in range(hp_sum.size()):
				avg_hp.append("%d" % roundi(hp_sum[k] / float(maxi(1, hp_n[k]))))
			var fell_text: Array[String] = []
			for b: int in range(1, total + 1):
				if fell.has(b):
					fell_text.append("b%d x%d" % [b, DictUtil.to_int(fell[b])])
			var seeds_n: float = float(maxi(1, args.seed_count))
			out("%-24s steel %-4s survived %.1f/%d, campaign cleared %d/%d, fell: %s, mean party hp%% after: [%s]" % [",".join(comp), steel, wins_sum / seeds_n, total,
				clears, args.seed_count, ", ".join(fell_text) if not fell_text.is_empty() else "never", ", ".join(avg_hp)])
			overall_wins += wins_sum
			overall_total += float(total * args.seed_count)
		out("%-24s TRIO summary (both steel choices): survived %.1f of %.1f battles per run" % [",".join(comp), overall_wins / float(maxi(1, args.seed_count * STEELS.size())),
			overall_total / float(maxi(1, args.seed_count * STEELS.size()))])

static func out(text: String) -> void:
	print("§ " + text)
