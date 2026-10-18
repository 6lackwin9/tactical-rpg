class_name BattleMetrics
extends BattleObserver

# ==============================================================================
# Tactical RPG - Battle metrics (BREPORT1-L 1.2; the counting part of the balance sim collector, moved from
# tools/sim/sim_metrics.gd, which now extends this and keeps only the KPI table helpers).
# Sim mode: the runner calls on_start / on_step / on_round / on_end. Live mode (the game, BREPORT1 rule 2):
# attach(session) follows mounted / round_started / round_reset / round_concluded, keeps a deep-copy checkpoint per
# round start and restores it when a round is reset, so undone rounds leave no trace; finish() closes the fight and
# freeze() stops listening. Combos come from CombatResolver.combo_resolved (exploit, break, flow). Per-unit `detail`
# (status and combo counts, regen, best hit / finish / save) feeds BattleReport; it is not accumulated or printed.
# ==============================================================================

# Balance sim metrics collector (BAL2_SIM.md section 4). One record per unit per fight, matched by
# display name. HP changes come from each unit's public Combatant.health_changed signal (exact,
# overkill already clamped) and are buffered per unit; session.log_appended (fired right after each
# action, then once per riposte) credits the buffer to that entry's actor. Changes still buffered at
# block_completed have no entry (they fail the checks); HP lost after the last block_completed of a
# round is the round-end damage (bleed, poison, burn: dot_taken, and dot_dealt of the status's source
# from CombatResolver.last_round_end_steps); HP gained there by a unit with active regen is a heal
# (heal_received + regen_received, heal_done of status_sources["regen"]; S12S-L rule 9). Statuses are diffed per entry, positions sampled per block. No engine
# change: public signals and state only.
# Self-checks print "METRIC FAIL <unit> <field>": the log's damage (capped at the victim's HP) must
# equal the HP the victims lost, a heal entry's amount the HP gained, an area entry's hit count the
# units it touched; a gain outside a heal entry or a loss without an attack entry is unexplained.

const NEAR_DEATH_FRAC: float = 0.25   # near_death_saves: an ally under this HP share, then healed
const EARLY_DOWN_ROUND: int = 3       # BAL5 K3: down_early = downed before this round (spec S1 trigger)

# Summed per unit per fight (the KPI block divides by fights / samples).
const FIELDS: Array[String] = [
	"dmg_dealt", "dmg_dealt_ranged", "dmg_taken", "dot_taken", "ff_dealt", "ff_taken", "attacks_received",
	"heal_done", "overheal", "heal_received", "cleanses", "buffs_applied", "buff_rounds", "debuffs_applied",
	"debuff_rounds", "area_casts", "area_foes_hit", "area_allies_hit", "kills", "blocks_alive", "survived",
	"hp_end_frac", "min_hp_frac", "near_death_saves", "dist_sum", "dist_samples", "blocks_adjacent",
	"hold_blocks", "taunt_blocks", "priority_dmg", "hp_start", "hp_end", "max_hp", "other_loss", "other_gain",
	"overheal_unknown", "regen_received", "dot_dealt",
	"heal_casts", "downed", "down_round", "down_early",   # BAL5 K3: heal entries attempted; first time at 0 HP (round; before EARLY_DOWN_ROUND)
	"combos",   # BREPORT1: exploits + stance breaks + flows (CombatResolver.combo_resolved)
]
const COMBO_KINDS: Array[String] = ["exploit", "break", "flow"]

var priority_name: String = ""     # display name of the scenario's priority target ("" = none)
var units: Dictionary = {}         # display name -> record of the last fight
var fight_record: Dictionary = {}  # last fight: cleared, rounds, party_hp_frac, enemy_hp_frac, hp_score, survivors, priority_down_round
var fail_lines: Array[String] = []
var quiet: bool = false            # true: record fail lines without printing (tests that expect failures)
var checks: int = 0                # self-checks evaluated (all fights)
var keep_totals: bool = false      # duel / tier / levels / group with metrics=on: sum every fight per unit name
var totals: Dictionary = {}        # display name -> accumulator (keep_totals)
# display name -> {status_counts: {status: n}, combo_counts: {exploit, break, flow}, regen_done, best_hit, best_finish,
# best_save, saves: [{ally, hp, round, order}]} (BREPORT1 1.2; best_save is chosen from `saves` in on_end)
var detail: Dictionary = {}
var frozen: bool = false           # freeze() ran: the records no longer change

var _live: CombatSession = null    # attach(): the live session followed through its round signals
var _checkpoints: Dictionary = {}  # live: round -> deep copy of the collector state at that round's start
var _rewinding: bool = false       # live: between round_reset / mounted and the next round_started (events ignored)
var _restart: bool = false         # live: the battle was rebuilt (mounted): on_start again at round_started
var _resolver: CombatResolver = null   # the resolver the block / combo hooks are connected to
var _session: CombatSession = null
var _by_name: Dictionary = {}      # display name -> Combatant
var _ally: Dictionary = {}         # Combatant -> bool (party side)
var _last_hp: Dictionary = {}      # Combatant -> HP at its last health_changed
var _buf: Dictionary = {}          # Combatant -> {"start", "loss", "gain", "touched"} since the last entry
var _hold: Dictionary = {}         # attacker -> {"start", "loss"}: its own loss in its action, for the riposte entries
var _hooks: Dictionary = {}        # Combatant -> bound health_changed callable
var _pending: Dictionary = {}      # Combatant -> {status: rounds} queued, at the last entry
var _active: Dictionary = {}       # Combatant -> {status: rounds} active, at the last entry
var _pos: Dictionary = {}          # Combatant -> last known tile
var _start_tile: Dictionary = {}   # Combatant -> first tile on the field
var _below: Dictionary = {}        # Combatant -> true once under NEAR_DEATH_FRAC
var _saves: Dictionary = {}        # healer -> {ally: true}
var _round_end: bool = false       # after the last block_completed of a round: losses are round-end damage
var _steps_seen: Array[Dictionary] = []   # the resolver's last_round_end_steps already credited (same array = done)
var _priority_down: int = 0

func on_start(session: CombatSession) -> void:
	_release()
	_session = session
	_rewinding = false
	_restart = false
	units.clear()
	detail.clear()
	fight_record.clear()
	for d: Dictionary in [_by_name, _ally, _last_hp, _buf, _hold, _pos, _start_tile, _below, _saves]:
		d.clear()
	_round_end = false
	_priority_down = 0
	_steps_seen = session.resolver.last_round_end_steps
	for c: Combatant in session.allies + session.enemies:
		_by_name[c.name] = c
		_ally[c] = session.allies.has(c)
		var rec: Dictionary = {}
		for f: String in FIELDS:
			rec[f] = 0
		rec["hp_start"] = c.hp
		rec["max_hp"] = c.max_hp
		rec["min_hp_frac"] = float(c.hp) / maxf(1.0, float(c.max_hp))
		units[c.name] = rec
		detail[c.name] = _new_detail()
		_last_hp[c] = c.hp
		_remember_statuses(c)
		_track_tile(c)
		var hook: Callable = _on_health.bind(c)
		c.health_changed.connect(hook)
		_hooks[c] = hook
	session.log_appended.connect(_on_log)
	_resolver = session.resolver
	_resolver.block_completed.connect(_on_block_completed)
	_resolver.combo_resolved.connect(_on_combo)

static func _new_detail() -> Dictionary:
	var combos: Dictionary = {}
	for kind: String in COMBO_KINDS:
		combos[kind] = 0
	return {"status_counts": {}, "combo_counts": combos, "regen_done": 0, "best_hit": {}, "best_finish": {}, "best_save": {}, "saves": []}

func on_step(_session_arg: CombatSession) -> void:
	_credit_round_end()
	_round_end = false
	_check_priority()

func on_round(_session_arg: CombatSession) -> void:
	for c: Combatant in _ally:   # roll-over changed statuses outside any entry: new baseline
		_remember_statuses(c)

func on_end(session: CombatSession, result: Dictionary) -> void:
	_credit_round_end()
	_flush_hold()
	_flush_unexplained()
	_check_priority()
	var cap: int = session.encounter.max_rounds
	for c: Combatant in _ally:
		var rec: Dictionary = units[c.name]
		rec["survived"] = 1 if c.is_active() else 0
		rec["hp_end"] = c.hp
		rec["hp_end_frac"] = float(c.hp if c.is_active() else 0) / maxf(1.0, float(c.max_hp))
		var saved: int = 0
		for ally: Variant in DictUtil.dict_of(_saves.get(c, {})):
			var a: Combatant = ally
			if a.is_active():
				saved += 1
		rec["near_death_saves"] = saved
		_pick_save(c)
	var survivors: int = 0
	for c: Combatant in session.allies:
		if c.is_active():
			survivors += 1
	fight_record = {"cleared": DictUtil.to_bool(result.get("cleared", false)), "rounds": DictUtil.to_int(result.get("rounds", 0)),
		"party_hp_frac": DictUtil.to_float(result.get("l", 0.0)), "enemy_hp_frac": DictUtil.to_float(result.get("r", 0.0)),
		"hp_score": DictUtil.to_float(result.get("hp", 0.0)), "survivors": survivors,
		"priority_down_round": _priority_down if _priority_down > 0 else cap + 1}
	_check_conservation()
	if keep_totals:
		for n: Variant in units:
			var acc: Dictionary = DictUtil.dict_of(totals.get(n, {}))
			accumulate(acc, DictUtil.dict_of(units[n]))
			totals[n] = acc
	_release()

func _release() -> void:
	for key: Variant in _hooks:
		var c: Combatant = key
		var hook: Callable = _hooks[key]
		if c.health_changed.is_connected(hook):
			c.health_changed.disconnect(hook)
	_hooks.clear()
	if _session != null:
		if _session.log_appended.is_connected(_on_log):
			_session.log_appended.disconnect(_on_log)
	if _resolver != null:
		if _resolver.block_completed.is_connected(_on_block_completed):
			_resolver.block_completed.disconnect(_on_block_completed)
		if _resolver.combo_resolved.is_connected(_on_combo):
			_resolver.combo_resolved.disconnect(_on_combo)
	_resolver = null
	_session = null

# ------------------------------------------------------------------------------
# Live mode (BREPORT1 1.2, rule 2): the game's session, with round checkpoints for rewinds
# ------------------------------------------------------------------------------

# Follows `session` from now on (its first round has started): records from here, checkpoint of this round.
func attach(session: CombatSession) -> void:
	detach()
	frozen = false
	_live = session
	session.mounted.connect(_on_mounted)
	session.round_started.connect(_on_round_started)
	session.round_reset.connect(_on_round_reset)
	session.round_concluded.connect(_on_round_concluded)
	on_start(session)
	_checkpoints.clear()
	_checkpoint(session.current_round)

# Stops listening (the records stay as they are).
func detach() -> void:
	if _live != null:
		for pair: Array in [[_live.mounted, _on_mounted], [_live.round_started, _on_round_started], [_live.round_reset, _on_round_reset], [_live.round_concluded, _on_round_concluded]]:
			var sig: Signal = pair[0]
			var hook: Callable = pair[1]
			if sig.is_connected(hook):
				sig.disconnect(hook)
	_live = null
	_rewinding = false
	_restart = false
	_release()

# The live fight is over: closes the records as on_end does (survival, saves, checks), then freezes them.
func finish() -> void:
	if _live != null and not frozen:
		on_end(_live, {})
	freeze()

# Detaches and keeps the records; later session changes (Replay, rewinds) no longer reach them.
func freeze() -> void:
	detach()
	_checkpoints.clear()
	frozen = true

func _on_mounted() -> void:
	_release()   # the units and the resolver were rebuilt: start over when the round begins
	_rewinding = true
	_restart = true

func _on_round_reset(round_num: int) -> void:
	_rewinding = true
	for r: Variant in _checkpoints.keys():
		if DictUtil.to_int(r) > round_num:
			_checkpoints.erase(r)

func _on_round_started(round_num: int) -> void:
	if _live == null:
		return
	if _restart:
		on_start(_live)
		_checkpoints.clear()
	elif _rewinding:
		_restore(round_num)
	else:
		_credit_round_end()
		_round_end = false
		_check_priority()
		on_round(_live)
	_rewinding = false
	_checkpoint(round_num)

func _on_round_concluded(_round_num: int, _over: bool) -> void:
	if _rewinding:
		return
	_credit_round_end()
	_round_end = false
	_check_priority()

func _checkpoint(round_num: int) -> void:
	_checkpoints[round_num] = {"units": units.duplicate(true), "detail": detail.duplicate(true), "below": _below.duplicate(),
		"saves": _saves.duplicate(true), "pos": _pos.duplicate(), "start_tile": _start_tile.duplicate(), "priority_down": _priority_down,
		"checks": checks, "fail_lines": fail_lines.duplicate()}

# Back to the checkpoint of `round_num` (its start); the units' HP, statuses and tiles are the new baseline.
func _restore(round_num: int) -> void:
	var cp: Dictionary = DictUtil.dict_of(_checkpoints.get(round_num, {}))
	if not cp.is_empty():
		units = DictUtil.dict_of(cp["units"]).duplicate(true)
		detail = DictUtil.dict_of(cp["detail"]).duplicate(true)
		_below = DictUtil.dict_of(cp["below"]).duplicate()
		_saves = DictUtil.dict_of(cp["saves"]).duplicate(true)
		_pos = DictUtil.dict_of(cp["pos"]).duplicate()
		_start_tile = DictUtil.dict_of(cp["start_tile"]).duplicate()
		_priority_down = DictUtil.to_int(cp["priority_down"])
		checks = DictUtil.to_int(cp["checks"])
		fail_lines.assign(DictUtil.array_of(cp["fail_lines"]))
	_buf = {}
	_hold.clear()
	_round_end = false
	if _session != null and _session.resolver != null:
		_steps_seen = _session.resolver.last_round_end_steps   # the undone round's end steps are never credited
	for c: Combatant in _ally:
		_last_hp[c] = c.hp
		_remember_statuses(c)

func _on_block_completed(block: int) -> void:
	if _session == null or _rewinding:
		return
	_flush_hold()
	_flush_unexplained()
	_sample_block()
	if block >= _session.total_blocks:
		_round_end = true   # apply_round_end_effects (bleed) follows in the same step

# ------------------------------------------------------------------------------
# HP changes and their entries
# ------------------------------------------------------------------------------

func _on_health(new_hp: int, c: Combatant) -> void:
	if _rewinding:
		return
	var d: int = new_hp - DictUtil.int_of(_last_hp, c)
	_last_hp[c] = new_hp
	var rec: Dictionary = units[c.name]
	rec["min_hp_frac"] = minf(DictUtil.float_of(rec, "min_hp_frac", 1.0), float(new_hp) / maxf(1.0, float(c.max_hp)))
	if new_hp > 0 and float(new_hp) < NEAR_DEATH_FRAC * float(c.max_hp):
		_below[c] = true
	if new_hp <= 0 and DictUtil.int_of(rec, "downed") == 0:
		var down_at: int = _session.current_round if _session != null else 0
		rec["downed"] = 1
		rec["down_round"] = down_at
		rec["down_early"] = 1 if down_at < EARLY_DOWN_ROUND else 0
	if _round_end:
		if d < 0:
			_add(c, "dot_taken", -d)
		elif d > 0 and c.has_status(StatusEffects.REGEN):
			_add(c, "heal_received", d)
			_add(c, "regen_received", d)
			var healer: Combatant = _unit(str(c.status_sources.get(StatusEffects.REGEN, "")))
			if healer != null:
				_add(healer, "heal_done", d)
				var hd: Dictionary = _detail(healer)
				hd["regen_done"] = DictUtil.int_of(hd, "regen_done") + d
		elif d > 0:
			_add(c, "other_gain", d)
		return
	var b: Dictionary = DictUtil.dict_of(_buf.get(c, {}))
	if b.is_empty():
		b = {"start": new_hp - d, "loss": 0, "gain": 0, "touched": true}
	if d < 0:
		b["loss"] = DictUtil.int_of(b, "loss") - d
	elif d > 0:
		b["gain"] = DictUtil.int_of(b, "gain") + d
	_buf[c] = b

func _on_log(e: ActionLogEntry) -> void:
	if _session == null or _rewinding:
		return
	var actor: Combatant = _unit(e.actor_name)
	if e.category == "ATTACK" and e.skill_name == "Riposte" and e.skill_subtype == "Counter":
		_riposte(actor, _unit(e.target_name), e)
		return
	_flush_hold()
	var changed: Dictionary = _buf
	_buf = {}
	if actor == null:
		_unexplained(changed)
		return
	if e.category == "ATTACK":
		_attack(actor, e, changed)
	elif e.category == "BUFF":
		_support(actor, e, changed)
	else:
		_unexplained(changed)
	for c: Combatant in _ally:
		_status_diff(actor, e, c)
		_remember_statuses(c)
		_track_tile(c)
	_check_priority()

# A single-target or area attack: every other unit's loss is a hit by `actor` (exact per victim).
func _attack(actor: Combatant, e: ActionLogEntry, changed: Dictionary) -> void:
	var area: bool = e.skill_subtype == "Area Attack"
	var target: Combatant = _unit(e.target_name)
	var lost: int = 0
	var capped_log: int = 0
	var touched: int = 0
	for key: Variant in changed:
		var v: Combatant = key
		var b: Dictionary = changed[key]
		if DictUtil.int_of(b, "gain") > 0:
			_add(v, "other_gain", DictUtil.int_of(b, "gain"))
		if v == actor:   # its own loss in its action: the riposte entries that follow explain it
			if DictUtil.int_of(b, "loss") > 0:
				_hold[actor] = {"start": DictUtil.int_of(b, "start"), "loss": DictUtil.int_of(b, "loss")}
			continue
		touched += 1
		if target == null and not area:
			target = v
		var loss: int = DictUtil.int_of(b, "loss")
		lost += loss
		capped_log += mini(DictUtil.int_of(b, "start"), e.damage)
		_hit(actor, v, loss, DictUtil.int_of(b, "start"), e)
		if area:
			_add(actor, "area_foes_hit" if _foes(actor, v) else "area_allies_hit", 1)
			if _foes(actor, v):
				_add(v, "attacks_received", 1)
	checks += 1
	if area:
		_add(actor, "area_casts", 1)
		var hits: int = int(e.outcome_tag.get_slice(" ", 0)) if e.outcome_tag != "MISS" else 0
		if touched != hits or lost > e.damage:
			_fail(actor.name, "log_area %s: %d hits / %d damage logged, %d units / %d HP lost" % [e.skill_name, hits, e.damage, touched, lost])
		return
	if target != null and _foes(actor, target):
		_add(target, "attacks_received", 1)
	var expected: int = mini(e.damage, capped_log) if touched > 0 else e.damage
	if touched > 1 or lost != expected:
		_fail(actor.name, "log_damage %s: %d logged (capped %d), %d HP lost by %d units" % [e.skill_name, e.damage, expected, lost, touched])

# A riposte: the defender's counter explains part of the attacker's held loss.
func _riposte(defender: Combatant, attacker: Combatant, e: ActionLogEntry) -> void:
	checks += 1
	var h: Dictionary = DictUtil.dict_of(_hold.get(attacker, {}))
	var start: int = DictUtil.int_of(h, "start")
	var left: int = DictUtil.int_of(h, "loss")
	var take: int = mini(e.damage, start)
	if defender == null or attacker == null or take > left:
		_fail(e.actor_name, "log_riposte: %d logged (capped %d), %d HP held" % [e.damage, take, left])
		take = mini(take, left)
	if attacker == null or defender == null:
		return
	_hit(defender, attacker, take, start, e)
	h["start"] = start - take
	h["loss"] = left - take
	_hold[attacker] = h

# A support entry: gains are heals by `actor` (the target's must equal the logged heal).
func _support(actor: Combatant, e: ActionLogEntry, changed: Dictionary) -> void:
	var target: Combatant = _unit(e.target_name)
	var target_gain: int = 0
	var heal_entry: bool = e.skill_subtype == "Support"
	for key: Variant in changed:
		var c: Combatant = key
		var b: Dictionary = changed[key]
		if DictUtil.int_of(b, "loss") > 0:
			_add(c, "other_loss", DictUtil.int_of(b, "loss"))
		var gain: int = DictUtil.int_of(b, "gain")
		if gain <= 0:
			continue
		if not heal_entry or _foes(actor, c):
			_add(c, "other_gain", gain)
			continue
		_add(c, "heal_received", gain)
		_add(actor, "heal_done", gain)
		if c == target:
			target_gain = gain
		if c != actor and _below.has(c):
			var saved: Dictionary = DictUtil.dict_of(_saves.get(actor, {}))
			saved[c] = true
			_saves[actor] = saved
		var before: int = DictUtil.int_of(b, "start")
		if c != actor and before > 0 and float(before) < NEAR_DEATH_FRAC * float(c.max_hp):   # BREPORT1 best_save candidate
			var saves: Array = DictUtil.array_of(_detail(actor).get("saves", []))
			saves.append({"ally": c.name, "hp": before, "round": _session.current_round, "order": _order()})
			_detail(actor)["saves"] = saves
	if heal_entry:
		_add(actor, "heal_casts", 1)
		checks += 1
		if target_gain != e.value_num:
			_fail(actor.name, "log_heal %s: %d logged, %d HP gained" % [e.skill_name, e.value_num, target_gain])
		var rec: Dictionary = units[actor.name]
		_overheal(actor, e, target_gain, rec)

# Buffered changes no entry explains (they fail the conservation check).
func _unexplained(changed: Dictionary) -> void:
	for key: Variant in changed:
		var c: Combatant = key
		var b: Dictionary = changed[key]
		_add(c, "other_loss", DictUtil.int_of(b, "loss"))
		_add(c, "other_gain", DictUtil.int_of(b, "gain"))

func _flush_unexplained() -> void:
	_unexplained(_buf)
	_buf = {}

# An attacker's own loss that no riposte entry explained.
func _flush_hold() -> void:
	for key: Variant in _hold:
		var c: Combatant = key
		var left: int = DictUtil.int_of(DictUtil.dict_of(_hold[key]), "loss")
		if left > 0:
			_add(c, "other_loss", left)
	_hold.clear()

func _hit(actor: Combatant, victim: Combatant, loss: int, start: int, e: ActionLogEntry) -> void:
	if loss <= 0:
		return
	if not _foes(actor, victim):
		_add(victim, "ff_taken", loss)
		_add(actor, "ff_dealt", loss)
		return
	_add(actor, "dmg_dealt", loss)
	_add(victim, "dmg_taken", loss)
	var d: Dictionary = _detail(actor)
	if loss > DictUtil.int_of(DictUtil.dict_of(d.get("best_hit", {})), "amount"):   # ties keep the earlier hit
		d["best_hit"] = {"amount": loss, "skill": e.skill_name, "target": victim.name, "round": _session.current_round, "order": _order()}
	if _session.grid.get_distance(_tile(actor), _tile(victim)) > 1:
		_add(actor, "dmg_dealt_ranged", loss)
	if not priority_name.is_empty() and victim.name == priority_name:
		_add(actor, "priority_dmg", loss)
	if start > 0 and start - loss <= 0:
		_add(actor, "kills", 1)

func _check_priority() -> void:
	if priority_name.is_empty() or _priority_down > 0 or _session == null:
		return
	var p: Combatant = _unit(priority_name)
	if p != null and not p.is_active():
		_priority_down = _session.current_round

# Overheal of a heal cast: the skill's heal amount (SupportRules.heal_amount) minus the HP gained.
func _overheal(healer: Combatant, e: ActionLogEntry, gained: int, rec: Dictionary) -> void:
	if e.outcome_tag == "FAILED":
		return
	var amount: int = -1
	for skill: SkillDef in SkillCatalog.all():
		if skill.display_name == e.skill_name and skill.heals() and healer.can_use(skill):
			amount = maxi(amount, SupportRules.heal_amount(healer, skill))
	if amount < 0:
		if gained > 0:
			rec["overheal_unknown"] = DictUtil.int_of(rec, "overheal_unknown") + 1
		return
	rec["overheal"] = DictUtil.int_of(rec, "overheal") + maxi(0, amount - gained)

# Statuses the entry's actor granted (buffs on its side) or inflicted (debuffs on foes), and debuffs
# it removed from its side (cleanses). Re-applying an already queued status at the same length is not
# visible and not counted.
func _status_diff(actor: Combatant, e: ActionLogEntry, c: Combatant) -> void:
	var old_pending: Dictionary = DictUtil.dict_of(_pending.get(c, {}))
	var old_active: Dictionary = DictUtil.dict_of(_active.get(c, {}))
	var new_pending: Dictionary = _status_rounds(c.pending_statuses, c.pending_status_rounds)
	var new_active: Dictionary = _status_rounds(c.statuses, c.status_rounds_left)
	var buffs: Array[String] = StatusEffects.buffs()
	var debuffs: Array[String] = StatusEffects.debuffs()
	for pair: Array in [[new_pending, old_pending], [new_active, old_active]]:
		var now: Dictionary = pair[0]
		var before: Dictionary = pair[1]
		for s: Variant in now:
			var rounds: int = DictUtil.int_of(now, s)
			if before.has(s) and DictUtil.int_of(before, s) >= rounds:
				continue
			if buffs.has(str(s)) and not _foes(actor, c):
				_add(actor, "buffs_applied", 1)
				_add(actor, "buff_rounds", rounds)
				_count_status(actor, str(s))
			elif debuffs.has(str(s)) and _foes(actor, c):
				_add(actor, "debuffs_applied", 1)
				_add(actor, "debuff_rounds", rounds)
				_count_status(actor, str(s))
	if e.category == "BUFF" and not _foes(actor, c) and c.is_alive():
		var had: Dictionary = old_pending.duplicate()
		had.merge(old_active)
		for s: Variant in had:
			if debuffs.has(str(s)) and not new_pending.has(s) and not new_active.has(s):
				_add(actor, "cleanses", 1)

func _count_status(actor: Combatant, status: String) -> void:
	var counts: Dictionary = DictUtil.dict_of(_detail(actor).get("status_counts", {}))
	counts[status] = DictUtil.int_of(counts, status) + 1
	_detail(actor)["status_counts"] = counts

static func _status_rounds(present: Dictionary, rounds: Dictionary) -> Dictionary:
	var result: Dictionary = {}
	for s: Variant in present:
		result[str(s)] = DictUtil.int_of(rounds, s, 1)
	return result

func _remember_statuses(c: Combatant) -> void:
	_pending[c] = _status_rounds(c.pending_statuses, c.pending_status_rounds)
	_active[c] = _status_rounds(c.statuses, c.status_rounds_left)

# Credits the round end's damage steps (bleed, poison, burn) to the status's source as dot_dealt, once
# per apply_round_end_effects call (the resolver replaces the array on every call).
func _credit_round_end() -> void:
	if _session == null or _session.resolver == null:
		return
	var steps: Array[Dictionary] = _session.resolver.last_round_end_steps
	if is_same(steps, _steps_seen):
		return
	_steps_seen = steps
	for row: Dictionary in steps:
		if str(row["status"]) == StatusEffects.REGEN:
			continue   # regen is credited live in _on_health
		var source: Combatant = _unit(str(row["source"]))
		if source != null:
			_add(source, "dot_dealt", DictUtil.to_int(row["amount"]))

# ------------------------------------------------------------------------------
# Per-block sampling
# ------------------------------------------------------------------------------

func _sample_block() -> void:
	var grid: CombatGrid = _session.grid
	for c: Combatant in _ally:
		_track_tile(c)
		if not grid.has_combatant(c):
			continue
		_add(c, "blocks_alive", 1)
		var here: Vector2i = grid.get_position(c)
		var nearest: int = -1
		for f: Combatant in _ally:
			if _foes(c, f) and grid.has_combatant(f):
				var d: int = grid.get_distance(here, grid.get_position(f))
				nearest = d if nearest < 0 else mini(nearest, d)
		if nearest >= 0:
			_add(c, "dist_sum", nearest)
			_add(c, "dist_samples", 1)
			if nearest <= 1:
				_add(c, "blocks_adjacent", 1)
		var start_raw: Variant = _start_tile.get(c, here)
		var start: Vector2i = start_raw if start_raw is Vector2i else here
		if grid.get_distance(here, start) <= 1:
			_add(c, "hold_blocks", 1)
		if c.statuses.has(StatusEffects.TAUNT):
			_add(c, "taunt_blocks", 1)

func _track_tile(c: Combatant) -> void:
	if _session != null and _session.grid != null and _session.grid.has_combatant(c):
		var t: Vector2i = _session.grid.get_position(c)
		_pos[c] = t
		if not _start_tile.has(c):
			_start_tile[c] = t

func _tile(c: Combatant) -> Vector2i:
	if _session.grid.has_combatant(c):
		return _session.grid.get_position(c)
	var raw: Variant = _pos.get(c, Vector2i(-99, -99))
	return raw if raw is Vector2i else Vector2i(-99, -99)

# ------------------------------------------------------------------------------
# Checks and helpers
# ------------------------------------------------------------------------------

# Per unit: hp_start - dmg_taken - ff_taken - dot_taken + heal_received == hp_end. Unexplained
# changes (other_loss / other_gain) are left out, so any of them breaks the equation and is also
# reported on its own. Per side: party dmg_dealt == enemy dmg_taken and the reverse.
func _check_conservation() -> void:
	var dealt: Array[int] = [0, 0]   # [party, enemy]
	var taken: Array[int] = [0, 0]
	for c: Combatant in _ally:
		var rec: Dictionary = units[c.name]
		var side: int = 0 if DictUtil.to_bool(_ally[c]) else 1
		dealt[side] += DictUtil.int_of(rec, "dmg_dealt")
		taken[side] += DictUtil.int_of(rec, "dmg_taken")
		checks += 1
		var expect: int = DictUtil.int_of(rec, "hp_start") - DictUtil.int_of(rec, "dmg_taken") - DictUtil.int_of(rec, "ff_taken") - DictUtil.int_of(rec, "dot_taken") + DictUtil.int_of(rec, "heal_received")
		if expect != DictUtil.int_of(rec, "hp_end"):
			_fail(c.name, "hp (%d expected, %d at the end)" % [expect, DictUtil.int_of(rec, "hp_end")])
		for f: String in ["other_loss", "other_gain"]:
			if DictUtil.int_of(rec, f) != 0:
				_fail(c.name, "%s %d" % [f, DictUtil.int_of(rec, f)])
	checks += 2
	if dealt[0] != taken[1]:
		_fail("party", "dmg_dealt %d != enemy dmg_taken %d" % [dealt[0], taken[1]])
	if dealt[1] != taken[0]:
		_fail("enemy", "dmg_dealt %d != party dmg_taken %d" % [dealt[1], taken[0]])

func _fail(unit_name: String, field: String) -> void:
	var line: String = "METRIC FAIL %s %s" % [unit_name, field]
	fail_lines.append(line)
	if not quiet:
		print("§ " + line)

func _unit(display_name: String) -> Combatant:
	var raw: Variant = _by_name.get(display_name, null)
	return raw if raw is Combatant else null

func _foes(a: Combatant, b: Combatant) -> bool:
	return DictUtil.to_bool(_ally.get(a, false)) != DictUtil.to_bool(_ally.get(b, false))

func _add(c: Combatant, field: String, amount: int) -> void:
	if amount == 0:
		return
	var rec: Dictionary = units[c.name]
	rec[field] = DictUtil.int_of(rec, field) + amount

# ------------------------------------------------------------------------------
# Report detail (BREPORT1 1.2): combos, best moments
# ------------------------------------------------------------------------------

# CombatResolver.combo_resolved: emitted before the action's log entry, so `order` is that entry's index.
func _on_combo(kind: String, actor: Combatant, target: Combatant, status: String, skill: String, killed: bool) -> void:
	if _session == null or _rewinding or actor == null or not units.has(actor.name):
		return
	_add(actor, "combos", 1)
	var d: Dictionary = _detail(actor)
	var counts: Dictionary = DictUtil.dict_of(d.get("combo_counts", {}))
	counts[kind] = DictUtil.int_of(counts, kind) + 1
	d["combo_counts"] = counts
	if killed and kind != "flow" and target != null and _foes(actor, target) and DictUtil.dict_of(d.get("best_finish", {})).is_empty():
		d["best_finish"] = {"skill": skill, "target": target.name, "combo": kind, "status": status, "round": _session.current_round, "order": _session.log_entries.size()}

# The best save of `healer`: of its heals on an ally under NEAR_DEATH_FRAC who is still alive at the end, the lowest
# HP before the heal, then the earliest.
func _pick_save(healer: Combatant) -> void:
	var d: Dictionary = _detail(healer)
	var best: Dictionary = {}
	for raw: Variant in DictUtil.array_of(d.get("saves", [])):
		var s: Dictionary = DictUtil.dict_of(raw)
		var ally: Combatant = _unit(str(s.get("ally", "")))
		if ally == null or not ally.is_alive():
			continue
		if best.is_empty() or DictUtil.int_of(s, "hp") < DictUtil.int_of(best, "hp"):
			best = s.duplicate()
	d["best_save"] = best

func _detail(c: Combatant) -> Dictionary:
	if not detail.has(c.name):
		detail[c.name] = _new_detail()
	return detail[c.name]

# Index of the log entry being credited (the last one appended).
func _order() -> int:
	return _session.log_entries.size() - 1 if _session != null else -1

# ------------------------------------------------------------------------------
# Aggregation over fights (static; used by party / role / sweep tables)
# ------------------------------------------------------------------------------

# Adds `rec` into `acc` (field sums plus "fights").
static func accumulate(acc: Dictionary, rec: Dictionary) -> void:
	for f: String in FIELDS:
		acc[f] = DictUtil.to_float(acc.get(f, 0.0)) + DictUtil.to_float(rec.get(f, 0.0))
	acc["fights"] = DictUtil.to_float(acc.get("fights", 0.0)) + 1.0
